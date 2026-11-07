#!/usr/bin/env python3
"""Register a maintained Harness core with both hosts without replacing user config."""
import argparse
from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import shutil
import tempfile
import uuid

BEGIN = '<!-- HARNESS:GLOBAL:BEGIN -->'
END = '<!-- HARNESS:GLOBAL:END -->'


def safe_path(path):
    for part in (path, *path.parents):
        if part.is_symlink():
            raise ValueError(f'refusing symlink in global instruction path: {part}')
    if path.exists() and not path.is_file():
        raise ValueError(f'not a regular file: {path}')


def replace_block(text, block):
    if BEGIN not in text and END not in text:
        return text + ('\n' if text and not text.endswith('\n\n') else '') + block
    if text.count(BEGIN) != 1 or text.count(END) != 1:
        raise ValueError('global instructions must contain exactly one complete Harness marker pair')
    start = text.index(BEGIN)
    end = text.index(END) + len(END)
    if end < start:
        raise ValueError('global Harness markers are out of order')
    return text[:start] + block.rstrip('\n') + text[end:]


def write_atomic(path, data, mode=0o600):
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix='.harness-', dir=path.parent)
    try:
        with os.fdopen(descriptor, 'wb') as handle:
            handle.write(data)
        os.chmod(temporary, mode)
        os.replace(temporary, path)
    finally:
        if os.path.lexists(temporary):
            os.unlink(temporary)


@contextmanager
def locked(root):
    root.mkdir(parents=True, exist_ok=True)
    lock = root / '.harness-global.lock'
    safe_path(lock)
    with lock.open('a') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def configuration(args):
    home = args.home.absolute()
    claude = Path(os.environ.get('CLAUDE_CONFIG_DIR', str(home / '.claude'))).absolute()
    codex = Path(os.environ.get('CODEX_HOME', str(home / '.codex'))).absolute()
    instructions = [claude / 'CLAUDE.md', codex / 'AGENTS.md']
    # Codex loads the override instead of AGENTS.md when it is nonempty.
    override = codex / 'AGENTS.override.md'
    safe_path(override)
    if override.exists() and override.stat().st_size:
        instructions[1] = override
    skills = [(claude / 'skills', '.claude'), (home / '.agents/skills', '.agents')]
    return instructions, skills


def link_check(destination, source):
    for parent in destination.parents:
        if parent.is_symlink():
            raise ValueError(f'refusing symlink in global skills parent: {parent}')
    if os.path.lexists(destination):
        if not destination.is_symlink() or Path(os.readlink(destination)) != source:
            raise ValueError(f'global skill collision: {destination}; existing skill kept')


def plan(args):
    instructions, skill_roots = configuration(args)
    root = args.root.absolute()
    changes = []
    state_path = root / '.harness-global.json'
    safe_path(state_path)
    state = json.loads(state_path.read_text()) if state_path.exists() else {'links': [], 'instructions': []}
    if args.remove:
        # Validate recorded paths against native discovery locations before touching them.
        allowed_roots = {directory: edition for directory, edition in skill_roots}
        for record in state['links']:
            destination, source = Path(record['path']), Path(record['source'])
            edition = allowed_roots.get(destination.parent)
            expected = f'{edition}/skills/{destination.name}'
            if edition is None or record['relative'] != expected or source != root / expected:
                raise ValueError('invalid global registration state')
            link_check(destination, source)
            if destination.is_symlink():
                changes.append((destination, None))
        instructions = list(dict.fromkeys(instructions + [Path(path) for path in state['instructions']]))
        current, _ = configuration(args)
        allowed = {current[0], current[1].parent / 'AGENTS.md', current[1].parent / 'AGENTS.override.md'}
        for path in instructions:
            if path not in allowed:
                raise ValueError(f'global instruction location changed: {path}')
            safe_path(path)
            if path.exists():
                text = path.read_text()
                if BEGIN in text or END in text:
                    changes.append((path, replace_block(text, '').encode()))
        return changes, state_path, {'links': [], 'instructions': []}

    for path in instructions:
        safe_path(path)
        replace_block(path.read_text() if path.exists() else '', '')
    links = []
    for directory, edition in skill_roots:
        for name in args.skills:
            if not name or any(character not in 'abcdefghijklmnopqrstuvwxyz0123456789-' for character in name):
                raise ValueError(f'invalid skill name: {name}')
            relative = f'{edition}/skills/{name}'
            source, destination = root / relative, directory / name
            link_check(destination, source)
            if not args.preflight and not (source / 'SKILL.md').is_file():
                raise ValueError(f'missing installed skill: {source}')
            links.append({'path': str(destination), 'source': str(source), 'relative': relative})
            if not destination.is_symlink():
                changes.append((destination, source))
    if not args.preflight:
        core = (root / 'CLAUDE.md').read_text()
        start, end = '<!-- HARNESS:BEGIN -->', '<!-- HARNESS:END -->'
        if core.count(start) != 1 or core.count(end) != 1 or core.index(end) < core.index(start):
            raise ValueError('installed core has no complete Harness protocol block')
        protocol = core.split(start, 1)[1].split(end, 1)[0].lstrip('\n')
        protocol = protocol.replace('`docs/WORKFLOW.md`', f'`{root}/docs/WORKFLOW.md`')
        block = f"""{BEGIN}
{protocol.rstrip()}

Bundled tools: resolve paths from the discovered skill directory, not the
current working directory. For missing Harness references, use `{root}/docs/`.
Repository instructions own project-specific rules.
{END}
"""
        for path in instructions:
            old = path.read_text() if path.exists() else ''
            wanted = replace_block(old, block).encode()
            if not path.exists() or path.read_bytes() != wanted:
                changes.append((path, wanted))
    else:
        changes.extend((path, b'') for path in instructions)
    # Keep a previously opted-in advisory skill registered on subsequent core-only runs.
    by_path = {record['path']: record for record in state['links']}
    by_path.update({record['path']: record for record in links})
    return changes, state_path, {'links': list(by_path.values()), 'instructions': list(dict.fromkeys(state['instructions'] + [str(path) for path in instructions]))}


def apply(args):
    changes, state_path, state = plan(args)
    for path, value in changes:
        print(f'{"unlink" if value is None else "link" if isinstance(value, Path) else "update"}  {path}')
    if args.preflight or args.dry_run:
        return
    # Snapshot every affected global path before activation; restore on failure.
    saved = []
    backup = args.root / '.harness-global-backup' / uuid.uuid4().hex
    try:
        state_bytes = json.dumps(state, indent=2).encode() + b'\n'
        if not state_path.exists() or state_path.read_bytes() != state_bytes:
            changes.append((state_path, state_bytes))
        for index, (path, value) in enumerate(changes):
            old = Path(os.readlink(path)) if path.is_symlink() else path.read_bytes() if path.exists() else None
            mode = path.stat().st_mode & 0o777 if path.is_file() else 0o600
            saved.append((path, old, mode))
            if isinstance(old, bytes):
                backup.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, backup / f'{index}-{path.name}')
            if value is None:
                path.unlink()
            elif isinstance(value, Path):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.symlink_to(value, target_is_directory=True)
            else:
                write_atomic(path, value, mode)
    except BaseException:
        for path, old, mode in reversed(saved):
            if path.is_symlink() or path.exists():
                path.unlink()
            if isinstance(old, Path):
                path.symlink_to(old, target_is_directory=True)
            elif isinstance(old, bytes):
                write_atomic(path, old, mode)
        raise
    if backup.exists():
        print(f'backup  {backup}')
    print('Global Harness registration removed.' if args.remove else 'Global Harness active for new Claude Code and Codex sessions.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--home', type=Path, default=Path.home())
    parser.add_argument('--skills', nargs='*', default=[])
    parser.add_argument('--preflight', action='store_true')
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--remove', action='store_true')
    args = parser.parse_args()
    args.root = args.root.absolute()
    try:
        if args.remove and not (args.root / '.harness-global.json').exists():
            print('No global Harness registration to remove.')
            return
        if args.preflight or args.dry_run:
            apply(args)
        else:
            with locked(args.root):
                apply(args)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f'Global Harness: {error}\n')


if __name__ == '__main__':
    main()
