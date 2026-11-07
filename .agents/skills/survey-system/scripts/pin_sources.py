#!/usr/bin/env python3
"""Print harness-source markers for cited line ranges in hub members.

Read-only. Each stdin line is `<repository> <path> <start> <end>`; the
repository is a Related Repositories row of the hub's docs/README.md with a
relative Location. Each output line is the marker for that range at the
member's HEAD, in input order, so `harness hub check-fresh` can track it.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

sys.dont_write_bytecode = True

ROW_RE = re.compile(r"^\|\s*`?([A-Za-z0-9._-]+)`?\s*\|[^|]*\|\s*`?([^|`]+?)`?\s*\|$")
NAME_RE = re.compile(r"^[A-Za-z0-9._-]+$")


class PinError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PinError(message)


def members(map_text: str) -> dict[str, str]:
    """Related Repositories rows with a relative Location, by name."""
    _, found, rest = map_text.partition("## Related Repositories\n")
    require(found != "", "docs/README.md has no Related Repositories section")
    rows: dict[str, str] = {}
    table = [line.strip() for line in rest.split("\n#", 1)[0].splitlines() if line.strip().startswith("|")]
    for line in table[2:]:  # header and separator
        match = ROW_RE.match(line)
        if match and "://" not in match.group(2) and not match.group(2).startswith(("/", "~")):
            rows[match.group(1)] = match.group(2).strip()
    return rows


def marker(repository: str, revision: str, path: str, start: int, end: int, blob: bytes) -> str:
    # Same splitting as the onboarding emitter and `harness hub check-fresh`.
    lines = blob.splitlines(keepends=True)
    require(1 <= start <= end <= len(lines), f"{repository}:{path}#L{start}-L{end} is outside the file")
    digest = hashlib.sha256(b"".join(lines[start - 1:end])).hexdigest()
    return f"<!-- harness-source: {repository}@{revision}:{path}#L{start}-L{end} sha256={digest} -->"


def git(repository: Path, *args: str) -> bytes:
    result = subprocess.run(["git", "-C", str(repository), *args], capture_output=True, check=False)
    require(result.returncode == 0, f"git {' '.join(args)} failed in {repository}: "
            + result.stderr.decode("utf-8", errors="replace").strip())
    return result.stdout


def pin(hub: Path, requests: list[str]) -> list[str]:
    rows = members((hub / "docs/README.md").read_text(encoding="utf-8"))
    roots: dict[str, tuple[Path, str]] = {}
    out = []
    for number, request in enumerate(requests, 1):
        parts = request.split()
        require(len(parts) == 4, f"line {number}: expected `<repository> <path> <start> <end>`")
        name, path, start, end = parts
        require(NAME_RE.fullmatch(name) is not None, f"line {number}: invalid repository name")
        require(name in rows, f"line {number}: {name} is not a relative Related Repositories row")
        candidate = PurePosixPath(path)
        require(not candidate.is_absolute() and ".." not in candidate.parts,
                f"line {number}: path must be repository-relative")
        require(start.isdigit() and end.isdigit(), f"line {number}: line numbers must be integers")
        if name not in roots:
            root = (hub / rows[name]).resolve()
            top = Path(git(root, "rev-parse", "--show-toplevel").decode().strip()).resolve()
            require(top == root, f"{name} is not a Git repository root at {rows[name]}")
            roots[name] = (root, git(root, "rev-parse", "HEAD").decode().strip())
        root, revision = roots[name]
        out.append(marker(name, revision, path, int(start), int(end), git(root, "show", f"HEAD:{path}")))
    return out


def self_test() -> None:
    table = ("# Map\n\n## Related Repositories\n\n| Repository | Role | Location |\n| --- | --- | --- |\n"
             "| orders | service | `../orders` |\n| remote | library | https://x.invalid/r.git |\n\n### Next\n")
    require(members(table) == {"orders": "../orders"}, "member table parsing failed")
    line = marker("orders", "a" * 40, "src/App.java", 2, 2, b"class A {\r\n  send();\n}")
    require(line.endswith(f"sha256={hashlib.sha256(b'  send();' + bytes([10])).hexdigest()} -->"),
            "digest must cover the exact cited bytes")
    try:
        marker("orders", "a" * 40, "src/App.java", 3, 4, b"a\nb\nc\n")
    except PinError:
        pass
    else:
        raise PinError("a range past the end of the file was accepted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hub", type=Path, default=Path("."))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            self_test()
            print(json.dumps({"self_test": "passed", "valid": True}))
            return 0
        requests = [line for line in sys.stdin.read().splitlines() if line.strip()]
        print("\n".join(pin(args.hub.resolve(), requests)))
        return 0
    except (PinError, OSError) as exc:
        print(json.dumps({"error": str(exc), "valid": False}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
