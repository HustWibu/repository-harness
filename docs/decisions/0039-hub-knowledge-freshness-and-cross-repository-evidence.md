---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0039 Hub Knowledge Freshness And Cross-Repository Evidence

Date: 2026-11-05

## Context

A review of the VSAF/NTAPP `ntapp-system-overview` and
`ntapp-architecture-design` skills found mechanisms worth adopting: a source
hash that blocks stale verdicts, three evidence tiers with a closed contract
status vocabulary, detected-versus-scope reporting, a capped and deduplicated
question loop whose answers are never re-asked, a contract lock that escalates
changes to shared contracts, and quantitative cross-checks of a synthesis
against its source artifact. In those skills nearly all of it is prose that
the agent checks itself; the evidence tiers have no mechanical check.

Harness already pins every onboarding claim to a revision and a line-range
digest, but nothing re-checked those pins after a hub page was written, and
`hub sync --workspace` silently ignored workspace repositories the hub did not
list.

## Decision

1. **Source markers and `harness hub check-fresh`.** A hub page pins each fact
   about member code with a one-line `harness-source` marker naming the member,
   revision, path, line range, and SHA-256 of the cited lines. `harness hub
   check-fresh` re-reads each range at the member's HEAD and reports current,
   moved, stale, or error; it exits 1 on stale or error. Line splitting matches
   the onboarding emitter so both compute the same digest.
2. **Scope honesty.** `hub sync --workspace` lists Git repositories in the
   workspace that Related Repositories omits. It does not fail on them.
3. **`docs/patterns/multi-repository-evidence.md`** (installed) states the
   closed contract statuses, the rule that `MATCHED` needs a pinned source in
   both repositories, the contract-change rule, reconciled counts, scope
   honesty, and the open-question marker with verbatim, never re-asked
   answers.
4. **Workflow and gate rules.** `WORKFLOW.md` gains the conflict rule
   (consistency follows existing code; correctness and safety follow written
   rules) and points to the pattern. `encoding-invariants.md` gains hook gate
   design: wait for people at prompt submission, record only at stop, named
   switches, passing and blocking fixtures, agent scores and script decides,
   and waivers recorded with a verbatim reason.

Workflow stages, lanes, readiness rubrics, design templates, and the multi-agent
pipeline stay with VSAF; Harness keeps no workflow (decision 0035's boundary).

## Alternatives Considered

1. **Adopt VSAF's evidence tiers as prose only.** Rejected: VSAF shows prose
   tiers are not checked; Harness already has pinned sources to check.
2. **Fail `hub sync` on unlisted repositories.** Rejected: workspaces hold
   third-party clones and tools; reporting is enough to stop silent omission.
3. **Re-run onboarding to detect staleness.** Rejected: too costly per member;
   re-reading pinned ranges is cheap and deterministic.

## Consequences

- A hub page whose markers pass `check-fresh` still says what the cited code
  says; a changed range is caught without re-running onboarding.
- Markers name commits that exist only in local clones; a missing pinned
  revision still allows a HEAD comparison but cannot report a move.
- The contract statuses and question rules remain guidance until a script
  checks them.

## Follow-Up

- Proof: domain tests for marker parsing, splitting, and assessment; an
  application test for current, stale, unlisted, and malformed sources; an
  end-to-end test committing to a real member (current, moved, stale) and
  listing an unlisted repository.
- Write the first marked hub page through the onboarding later pass and run
  `check-fresh` on it.
