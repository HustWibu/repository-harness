---
name: survey-system
description: Use only when the user explicitly invokes `/survey-system`. Survey a multi-repository system from its hub: per-repository pages, a cross-repository contract matrix with closed statuses, pinned sources, and open questions. Read-only until the user approves the pages.
disable-model-invocation: true
---

# Survey System

Describe what a multi-repository system is today, from code and configuration,
so later work starts from evidence instead of a fresh scan. Run it in the hub
(decision 0036). It describes; it does not design, plan, or judge.

## Safety Contract

- Read applicable `CLAUDE.md`, `docs/README.md`, `docs/WORKFLOW.md`, and the
  complete [evidence pattern](../../../docs/patterns/multi-repository-evidence.md).
- Until the user approves the pages, use read-only commands only. Never edit,
  fetch, pull, check out, build, or start anything in a member.
- Write only in the hub, only the approved files, and never commit.

## 1. Fix The Boundary

For the hub and each member record the revision, branch, and whether the
worktree is dirty. Run `scripts/bin/harness hub sync --workspace --check` (or
`--check`) for detected, listed, failed, and unlisted repositories.

Flag a member checked out on a branch other than the one its repository map or
`CLAUDE.md` calls deployed, and a checkout with no code on that branch: report
it, never scan the wrong branch silently.

## 2. Agree The Scope

Show three lists: detected, in scope (the listed members the user wants), and
out of scope with a reason each. Never add a repository to scope on your own.
Stop until the user confirms the scope.

## 3. Start From What Exists

Read the artifacts that Authority By Truth Type names, a generated system map,
and earlier survey pages before reading code. Run `scripts/bin/harness hub
check-fresh` and rescan only members whose pages are missing or stale; keep
current pages. Read a generated artifact instead of recounting by hand.

## 4. Scan Each Member

One read-only subagent per member may run in parallel; give each this file and
`references/page-shapes.md`. Fill the member page shape:

- Prose may infer intent; tag every inference `Assumption:` inline.
- Every table row cites `path#Lstart-Lend` at the recorded revision. A row
  without one keeps its cell as `<TODO: verify — <question>>`; never drop or
  invent the row.
- Environment values come from the configuration source Authority By Truth
  Type names; a value committed in the member may describe development only.

## 5. Build The Contract Matrix

Join provided and consumed interfaces across members. Give every edge exactly
one status from the evidence pattern; `MATCHED` needs a cited source in both
repositories. An edge to a repository outside scope is listed with that
repository named, never matched.

## 6. Ask The Open Questions

Collect markers with `grep -rn '<TODO: verify — '`, merge exact duplicates
across members, and rank: environment targets, contract shape, authentication,
data ownership, then the rest. Ask at most max(3, min(8, 2 x members in
scope)) and keep the others inline. Record each answer verbatim with who gave
it in the system page; answers recorded earlier are copied unchanged and never
asked again.

## 7. Pin And Check

Pipe `<repository> <path> <start> <end>` lines for every cited range to
`scripts/pin_sources.py --hub <hub>` and place each marker as the page shapes
show; never compute or copy a digest by hand. Before proposing, verify:

1. every member in scope has exactly one page, and nothing out of scope has one;
2. matrix counts per status recompute from the matrix rows;
3. no `MATCHED` row lacks a source on each side;
4. every earlier recorded answer is present verbatim;
5. no bare `<TODO: verify>` marker remains.

Fix and recheck once; if a check still fails, stop and report it.

## 8. Propose, Then Write

Present the scope lists, the files to write, counts per status, edges to
repositories outside scope, open questions asked, deferred, and kept inline,
and the members with the most TODO markers. Write the pages only after the
user approves them, then run `scripts/bin/harness hub check-fresh` and report
its result. Leave committing to the user.

## Receipt

End the final answer with exactly one last line:
`HARNESS_SKILL: survey-system | done|stopped|skipped | <artifact path or reason>`.
Use `stopped` when scope, authority, or a self-check stops the work. The third
field is never empty, for example
`HARNESS_SKILL: survey-system | stopped | <missing scope or failed check>`.
