---
status: accepted
approved_by: "chiendv9"
approved_at: "2026-11-05"
superseded_by: ""
---

# 0035 Multi-Repository Knowledge Surfaces

Date: 2026-11-05

## Context

Harness assumed the system boundary is the Git root. In a microservice system
that is wrong: who sends to a service, the contract between services, the
configuration an environment really runs, and the plan for a change all live
partly in other repositories. Agents then guess, or treat a repository's
development configuration as truth. Requirements are in
`docs/research/multi-repo-requirements.md`.

## Decision

Add optional, technology-neutral surfaces. Empty means behavior as before.

1. `docs/README.md` gains Related Repositories and Authority By Truth Type,
   with one source of record per truth type and no machine-specific paths.
2. `docs/WORKFLOW.md` gains three short rules (under 150 words in total): name
   the owning repository instead of an ownerless Unknown; repository
   configuration may be development-only; each verified claim states its
   evidence scope, and contract consumers without evidence are listed as not
   verified. The consumer sets minimum scope per change type.
3. Templates: an Environments table and environment-faithful verification rule
   in the application runbook; a Repositories Touched table with a single home
   repository in the execution plan and session handoff; an Evidence scope
   criterion in the evaluator rubric; a new optional interface-contract
   template.
4. The invariant pattern gains step 8, Cross-Repository Invariants, with two
   patterns proven in both directions; `/encode-invariant` accepts a source in
   another repository only when Authority By Truth Type names it.
5. `/onboard-repository` records boundary edges where a traced path leaves the
   repository, from a reference file opened only then. It may read a sibling
   checkout read-only and only with the user's permission; otherwise edges stay
   unconfirmed.

Harness itself never reads, writes, or calls another repository during install,
update, or any command. Only an explicitly invoked skill may read one.
Amended by decision 0036: `harness hub sync`, run in a knowledge hub, writes
only its marked pointer block into listed member repositories.

## Alternatives Considered

1. **Keep single-repository scope.** Rejected: agents in the target systems
   already fail at the boundary, and the additions are optional.
2. **A separate file for the multi-repository map.** Rejected for now: one more
   payload file to find; the docs map is already the entry point.
3. **Ship verification scripts, a cross-repository tracker, or log readers.**
   Rejected by decisions 0027 and 0029 and gap analysis C-1, C-3, C-5.

## Consequences

Positive:

- Agents can name the owner of an out-of-repository fact and report how far
  their evidence reaches.

Tradeoffs:

- Entry reading grows by about 100 words.
- Hand-written counterpart lists go stale; the template asks for generation or
  a checked date.

## Follow-Up

- Run `/onboard-repository` on a real service that publishes messages to
  another repository, and confirm the report contains boundary edges.
