#!/usr/bin/env python3
"""Render Codex adapters from the canonical Claude skills; --check detects drift."""
import argparse
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def outputs():
    yield ROOT / 'AGENTS.md', (ROOT / 'CLAUDE.md').read_bytes()
    for source in sorted((ROOT / '.claude/skills').rglob('*')):
        if not source.is_file() or '__pycache__' in source.parts:
            continue
        target = ROOT / '.agents/skills' / source.relative_to(ROOT / '.claude/skills')
        text = source.read_text().replace('.claude/skills/', '.agents/skills/')
        if source.suffix == '.md':
            text = text.replace('CLAUDE.md', 'AGENTS.md')
        text = re.sub(r'`/([a-z][a-z-]+)`', r'`$\1`', text)
        if source.name == 'SKILL.md':
            text = text.replace('disable-model-invocation: true\n', '')
            yield target.parent / 'agents/openai.yaml', b'policy:\n  allow_implicit_invocation: true\n'
        yield target, text.encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    stale = []
    for target, content in outputs():
        if target.is_file() and target.read_bytes() == content:
            continue
        if args.check:
            stale.append(str(target.relative_to(ROOT)))
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(content)
    if stale:
        parser.exit(1, 'Codex adapters out of date; run python3 scripts/sync-codex-skills.py:\n' + '\n'.join(stale) + '\n')


if __name__ == '__main__':
    main()
