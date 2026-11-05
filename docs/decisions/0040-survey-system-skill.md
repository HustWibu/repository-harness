---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0040 Survey System Skill

Date: 2026-11-05

## Context

A hub (decision 0036) needs a system-wide description: what each member
provides and consumes, which contracts hold across repositories, and what is
still unknown. `/onboard-repository` goes deep on one path in one repository;
nothing in Harness surveyed a whole system. Decision 0039 already defines the
contract statuses, source markers, scope honesty, and open-question rules such
a survey must follow.

## Decision

Ship an explicit `/survey-system` skill in the core payload:

1. It runs in the hub, read-only until the user approves the pages, and writes
   only hub files: `docs/services/<repository>/overview.md` per member and
   `docs/system/overview.md` for the system.
2. Scope is confirmed by the user from detected, listed, and unlisted
   repositories (`hub sync --workspace --check`); a member on a branch other
   than its deployed one is reported, never scanned silently.
3. It starts from existing artifacts and current pages, rescanning only
   members whose pages are missing or stale under `hub check-fresh`.
4. Prose may infer intent if tagged `Assumption:`; every table row cites a line
   range or keeps a `<TODO: verify — question>` marker. Edges use the closed
   statuses of `multi-repository-evidence.md`, and `MATCHED` needs a source in
   both repositories.
5. Open questions are deduplicated, ranked, capped at max(3, min(8, 2 x
   members)), and answered verbatim; earlier answers are never asked again.
6. `scripts/pin_sources.py` prints `harness-source` markers for cited ranges at
   each member's HEAD, with the same digest as the onboarding emitter and
   `hub check-fresh`; digests are never typed by hand.
7. Before proposing, it checks one page per member in scope, counts that
   recompute from the matrix, `MATCHED` rows sourced on both sides, earlier
   answers kept verbatim, and no bare TODO markers.

It describes the current system only: no design, plan, or workflow stage.

## Alternatives Considered

1. **Bundle an existing third-party system-overview skill.** Rejected: it was
   about 125 KB, tied to another workflow's directories and scripts, and
   maintained elsewhere; the useful mechanisms are already in decision 0039.
2. **Extend `/onboard-repository` to whole systems.** Rejected: its capsule
   and approval flow suit one small patch, not dozens of pages.

## Consequences

- A survey's tables stay checkable: `hub check-fresh` follows every pinned
  row after the pages are written.
- The skill relies on the agent for the scan itself; the self-checks and pins
  are mechanical, the classification is not.

## Follow-Up

- Proof: `pin_sources.py --self-test`; `tests/workflow/test-survey-system.sh`
  pins a range in a real member, sees `current` from `hub check-fresh`, sees a
  failure after the member changes, and rejects an unlisted repository and an
  out-of-range line. A marker the script printed for the CPM hub matched the
  existing marker byte for byte.
- Run `/survey-system` on a real hub and record what the self-checks caught.
