---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0036 Shared Knowledge Hub

Date: 2026-11-05

## Context

Decision 0035 lets each repository describe its related repositories, but the
knowledge itself still lives per repository: decisions, contracts, and plans
are scattered across many service repositories, and a system-wide decision has
no single home. The owner of a system of about 45 service repositories wants
one place for that knowledge.

Decision 0033 deferred a parent-directory profile because a parent directory
usually has no Git history, so the updater could not merge or recover safely.
A hub that is itself a Git repository removes that objection.

## Decision

1. **The hub is an ordinary Git repository with a normal Harness install**, for
   example `system-knowledge`. System-wide decisions, interface contracts, and
   multi-repository plans live in its `docs/decisions/`, `docs/contracts/`, and
   `docs/plans/`. Knowledge specific to one member lives in
   `docs/services/<repository>/`.
2. **Members carry only a pointer.** A member repository holds no Harness core;
   its `CLAUDE.md` contains one marked block (`<!-- HARNESS-HUB:BEGIN -->` …
   `<!-- HARNESS-HUB:END -->`) giving the hub's relative path, where that
   member's knowledge lives, and the rule to record new decisions in the hub.
3. **The pointer is a path, not an import.** The block names the hub path and
   the agent opens hub files when needed; it does not `@`-import the hub, so a
   member session does not load hub text by default.
4. **The hub lists its members.** Rows of the Related Repositories table in the
   hub's `docs/README.md` whose Location is a relative path are members; rows
   with a URL are recorded but not touched. Members must be checked out at that
   path and be Git repository roots.
5. **`harness hub sync`, run in the hub, writes the pointer block** into each
   member's `CLAUDE.md`. It writes only inside the markers, keeps all other
   text, refuses symlinks and non-Git targets, never commits, and reports one
   line per member. `--dry-run` previews; `--check` writes nothing and exits 1
   when any member is stale or unreachable. A member whose
   `docs/services/<repository>/` is missing in the hub fails instead of being
   pointed at a directory that does not exist.
6. **`harness hub sync --workspace` writes one block instead**, into the
   `CLAUDE.md` of the directory that holds the hub. Claude Code loads that file
   in every repository below it, so members are only checked (Git root, below
   that directory, hub folder present) and not written. The block names the hub
   by absolute path because sessions run in members, not in that directory. It
   refuses when that directory is inside a Git repository. Use it when
   repositories are cloned side by side and their `CLAUDE.md` is not committed;
   use per-member sync when each member commits its `CLAUDE.md` and the pointer
   must reach other clones.

This amends decision 0035, which said Harness never writes another repository:
`harness hub sync`, when the user runs it in the hub, is the one exception, and
it writes only the marked block, either in each member or, with `--workspace`,
in the directory that holds the hub. Install, update, and every other command still
touch only their own target. Decision 0033 is superseded.

## Alternatives Considered

1. **Each service keeps its own decisions, the hub holds only shared ones.**
   Rejected: two places to search, and a decision's scope changes over time.
2. **`@`-import the hub's `CLAUDE.md` into each member.** Rejected for now:
   every member session would pay the hub's context cost; opening the hub on
   demand is enough while the pointer names exact paths.
3. **Members list the hub; the user writes each pointer by hand.** Rejected:
   about 45 repositories would drift; one command from the hub keeps them
   identical.
4. **A parent workspace directory as the hub (decision 0033's design).**
   Rejected: no Git history for merge and recovery.

## Consequences

Positive:

- One home for system knowledge, reviewed in one repository's history.
- Pointers are generated, so changing the hub path or layout is one sync.

Tradeoffs:

- Repositories must be cloned side by side at the listed relative paths; a
  different layout makes `hub sync` fail for that member.
- Harness skills are installed only in the hub, so they are available when
  Claude Code is opened in the hub, not in a member.
- Where a member tracks its `CLAUDE.md`, the pointer edit is an ordinary change
  its owners review and commit. Where `CLAUDE.md` is ignored or untracked, as
  in a workspace where 40 of 48 members ignore it, the pointer exists only on
  the machine that ran sync, so each developer runs sync; `--workspace` makes
  that one file instead of one per member.
- The workspace block is machine-local and names an absolute path; it is not
  shared through Git.

## Follow-Up

- Proof: unit tests in `crates/harness/src/domain/hub.rs`,
  `crates/harness/src/application/hub.rs`, and
  `crates/harness/src/infrastructure/hub_workspace.rs`, and the end-to-end test
  `crates/harness/tests/hub_cli.rs`.
- Try it on the real system and record whether members need more than the
  pointer.
