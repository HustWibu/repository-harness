#!/usr/bin/env python3
"""Check survey pages before they are proposed or after they are written.

Read-only. stdin is a JSON object mapping hub-relative page paths to their
full text (drafts need not exist on disk). `--members` names the members in
scope; `--previous` is the hub's current docs/system/overview.md, if any.
Prints a JSON verdict and exits 1 when any check fails.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True

STATUSES = {"MATCHED", "VERSION-MISMATCH", "DTO-DRIFT", "CONSUMER-ONLY", "PROVIDER-ORPHAN", "UNKNOWN"}
SYSTEM_PAGE = "docs/system/overview.md"
MEMBER_PAGE = re.compile(r"^docs/services/([A-Za-z0-9._-]+)/overview\.md$")
TODO = re.compile(r"<TODO: verify(?P<rest>[^>]*)>")
MARKER = re.compile(
    r"^<!-- harness-source: [A-Za-z0-9._-]+@[0-9a-f]{40}:[^#\s]+#L[1-9][0-9]*-L[1-9][0-9]* sha256=[0-9a-f]{64} -->$"
)


def section(text: str, heading: str) -> str:
    _, found, rest = text.partition(f"\n## {heading}\n")
    return rest.split("\n## ", 1)[0] if found else ""


def table(block: str) -> list[list[str]]:
    rows = [line.strip() for line in block.splitlines() if line.strip().startswith("|")]
    return [[cell.strip() for cell in row.strip("|").split("|")] for row in rows[2:]]


def unfenced(text: str) -> list[str]:
    lines, fenced = [], False
    for line in text.splitlines():
        if line.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
        elif not fenced:
            lines.append(line)
    return lines


def check(pages: dict[str, str], members: list[str], previous: str | None) -> list[str]:
    failures: list[str] = []
    surveyed = sorted(m.group(1) for p in pages if (m := MEMBER_PAGE.match(p)))
    if surveyed != sorted(members):
        failures.append(f"member pages {surveyed} do not match the scope {sorted(members)}")
    unexpected = [p for p in pages if p != SYSTEM_PAGE and not MEMBER_PAGE.match(p)]
    if unexpected:
        failures.append(f"pages outside the survey layout: {unexpected}")

    for path, text in sorted(pages.items()):
        for number, line in enumerate(text.splitlines(), 1):
            for todo in TODO.finditer(line):
                if not todo.group("rest").startswith(" — ") or len(todo.group("rest").strip(" —?")) < 8:
                    failures.append(f"{path}:{number}: TODO marker without a concrete question")
        for line in unfenced(text):
            if line.lstrip().startswith("<!-- harness-source:") and not MARKER.match(line.strip()):
                failures.append(f"{path}: malformed harness-source marker: {line.strip()[:80]}")

    system = pages.get(SYSTEM_PAGE)
    if system is None:
        return failures + [f"{SYSTEM_PAGE} is missing"]
    matrix = table(section(system, "Contract Matrix"))
    header = [line for line in section(system, "Contract Matrix").splitlines() if line.strip().startswith("|")][:1]
    columns = [c.strip().lower() for c in header[0].strip().strip("|").split("|")] if header else []
    try:
        status_at = columns.index("status")
        provider_at = columns.index("provider evidence")
        consumer_at = columns.index("consumer evidence")
    except ValueError:
        return failures + ["Contract Matrix needs Status, Provider evidence, and Consumer evidence columns"]
    counted = {status: 0 for status in STATUSES}
    for row in matrix:
        status = row[status_at] if status_at < len(row) else ""
        if status not in STATUSES:
            failures.append(f"Contract Matrix row {row[:2]} has status {status!r} outside the closed set")
            continue
        counted[status] += 1
        if status == "MATCHED":
            for at, side in ((provider_at, "provider"), (consumer_at, "consumer")):
                cell = row[at] if at < len(row) else ""
                if "#L" not in cell or "<TODO" in cell:
                    failures.append(f"MATCHED row {row[:2]} has no cited {side} source")
    for row in table(section(system, "Counts")):
        if len(row) >= 2 and row[0] in STATUSES:
            if not row[1].isdigit() or int(row[1]) != counted[row[0]]:
                failures.append(f"Counts says {row[0]} = {row[1]}, the matrix has {counted[row[0]]}")
        elif len(row) >= 2 and row[0].strip("*").lower() == "total":
            if not row[1].strip("*").isdigit() or int(row[1].strip("*")) != sum(counted.values()):
                failures.append(f"Counts total {row[1]} differs from {sum(counted.values())} matrix rows")

    if previous:
        kept = section(system, "Answers")
        for row in table(section(previous, "Answers")):
            line = "| " + " | ".join(row) + " |"
            if line not in kept:
                failures.append(f"earlier answer not kept verbatim: {line[:80]}")
    return failures


def self_test() -> None:
    marker = f"<!-- harness-source: a@{'0' * 40}:src/A.java#L1-L2 sha256={'1' * 64} -->"
    system = ("# System Overview\n\n## Contract Matrix\n"
              "| # | Channel | Provider | Consumer | Status | Provider evidence | Consumer evidence |\n"
              "| --- | --- | --- | --- | --- | --- | --- |\n"
              "| 1 | t | a | b | MATCHED | `A.java#L1-L2` | `B.java#L3-L4` |\n"
              "| 2 | u | a | b | UNKNOWN | `A.java#L5-L5` | <TODO: verify — who consumes topic u?> |\n\n"
              "## Counts\n| Status | Count |\n| --- | --- |\n| MATCHED | 1 |\n| UNKNOWN | 1 |\n| Total | 2 |\n\n"
              "## Answers\n| Question | Answer | Given by | Date |\n| --- | --- | --- | --- |\n| q | a | p | d |\n\n"
              f"## Sources\n{marker}\n")
    good = {SYSTEM_PAGE: system, "docs/services/a/overview.md": "# a\n", "docs/services/b/overview.md": "# b\n"}
    assert check(good, ["a", "b"], system) == [], check(good, ["a", "b"], system)
    broken = {
        "missing page": ({**good, "docs/services/b/overview.md": None}, ["a", "b"], None),
        "count": ({**good, SYSTEM_PAGE: system.replace("| MATCHED | 1 |", "| MATCHED | 2 |")}, ["a", "b"], None),
        "one-sided": ({**good, SYSTEM_PAGE: system.replace("`B.java#L3-L4`", "<TODO: verify — b side?>")}, ["a", "b"], None),
        "status": ({**good, SYSTEM_PAGE: system.replace("UNKNOWN | `A", "MAYBE | `A")}, ["a", "b"], None),
        "bare todo": ({**good, "docs/services/a/overview.md": "x <TODO: verify> y\n"}, ["a", "b"], None),
        "marker": ({**good, SYSTEM_PAGE: system.replace("sha256=1", "sha256=X")}, ["a", "b"], None),
        "answer": ({**good, SYSTEM_PAGE: system.replace("| q | a | p | d |", "| q | changed | p | d |")}, ["a", "b"], system),
    }
    for name, (pages, members, previous) in broken.items():
        pages = {path: text for path, text in pages.items() if text is not None}
        assert check(pages, members, previous), f"self-test case not caught: {name}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--members", default="")
    parser.add_argument("--previous", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print(json.dumps({"self_test": "passed", "valid": True}))
        return 0
    try:
        pages = json.load(sys.stdin)
        if not isinstance(pages, dict) or not all(isinstance(v, str) for v in pages.values()):
            raise ValueError("stdin must map page paths to page text")
        previous = args.previous.read_text(encoding="utf-8") if args.previous and args.previous.exists() else None
    except (ValueError, OSError) as exc:
        print(json.dumps({"valid": False, "error": str(exc)}), file=sys.stderr)
        return 1
    members = [m for m in args.members.split(",") if m]
    failures = check(pages, members, previous)
    print(json.dumps({"valid": not failures, "failures": failures}, ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
