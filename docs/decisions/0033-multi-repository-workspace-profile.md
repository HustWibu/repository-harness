---
status: superseded
approved_by: "chiendv9"
approved_at: "2026-11-05"
superseded_by: "docs/decisions/0036-shared-knowledge-hub.md"
---

# 0033 Multi-Repository Workspace Profile

Date: 2026-11-05

## Context

Microservice systems are often cloned side by side under one parent directory
that is not itself a Git repository. Claude Code already loads a `CLAUDE.md`
from parent directories, so the parent is a natural place for a system-level
map: which repositories exist, which owns each contract, where each
environment's configuration lives. The updater, however, assumes one Git root:
`.harness-core/` sits inside the target, conflict recovery relies on the
consumer's Git history, and `--local-only` (decision 0034) needs
`.git/info/exclude`.

Decision 0035 already lets each repository name its related repositories and
the owner of each truth type. This decision only weighs whether Harness should
also install and maintain a parent-level profile. Nothing is implemented.

## Decision

Do not build a workspace profile yet. Keep Harness per repository and
rely on decision 0035 (Related Repositories and Authority By Truth Type in each
repository's `docs/README.md`). A user who wants a system-level map writes the
parent `CLAUDE.md` by hand; Harness neither installs nor updates it.

Revisit when at least two real systems use decision 0035 and their per-repository
maps disagree or duplicate enough to cause a recorded mistake.

If revisited, the design this decision would adopt is:

1. A parent-level `CLAUDE.md` holding the system map, and in each member
   repository a short shim that points up to it.
2. Updater state for the parent stored in `<parent>/.harness-core/`, the same
   layout as today, with the parent treated as an unversioned target.
3. Without Git history in the parent, every update backs up each file it
   replaces under `<parent>/.harness-backup/<timestamp>/` before writing, and a
   conflict stages a resolution session exactly as today; there is no
   automatic merge of a parent file the user edited since the last install.
   Recovery is restoring from that backup.

## Alternatives Considered

1. **Do nothing beyond decision 0035 (chosen for now).** Each repository
   carries its own map; the parent `CLAUDE.md` is user-owned. Cost: the map is
   repeated per repository and can drift. Benefit: no new install target,
   state location, or recovery path without version control.
2. **Install the system map into a dedicated specification repository.**
   Rejected as the default: not every system has one, and Claude Code does not
   load it automatically from sibling repositories.
3. **Manage the parent directory as a full Harness target now.** Rejected
   until evidence shows need: it adds a target type whose edits have no Git
   history, so conflict resolution and recovery rest only on backups.

## Consequences

Positive:

- No change to the installer, updater, or one-root assumption.
- Each repository's map stays reviewable in that repository's history.

Tradeoffs:

- Systems with many repositories repeat the map and must keep copies aligned;
  the cross-repository invariant pattern (step 8 of
  `docs/patterns/encoding-invariants.md`) can check that, owned by the consumer.

## Follow-Up

- Record real cases where per-repository maps drifted, as the evidence for
  revisiting this decision.
