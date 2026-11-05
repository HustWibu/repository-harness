# Evaluator Rubric: Title

Date: YYYY-MM-DD

<!--
Optional. Use it to review an agent's output with the same criteria each time.
Criteria restate docs/WORKFLOW.md; they add no consumer policy. The consumer's
own review rules and accepted decisions take precedence.
-->

## Work Reviewed

Link the change, request, and evidence that was submitted.

## Scores

Score each criterion 0, 1, or 2 and cite the evidence for the score.

| Criterion | 0 | 1 | 2 | Score | Evidence |
| --- | --- | --- | --- | --- | --- |
| Outcome | Misses the request | Partly meets it | Fully meets it | | |
| Scope | Edits outside the request | Minor spillover | Stays within the request | | |
| Evidence | None | Structural checks only | Proves the changed behavior | | |
| Safety | Breaks state or a boundary | Unstated risk | Preserves boundaries | | |
| Authority | Invents policy | Unstated assumption | Cites accepted authority | | |
| Honest report | Overclaims | Omits unverified parts | States verified and unverified | | |
| Evidence scope | Unstated | Stated for some claims | Each verified claim says in-repository, across a contract, or deployed | | |

## Verdict

Accept | Revise | Block

- Block when Safety, Authority, or Honest report scores 0.
- Revise when another criterion scores 0 or the total is below 9.
- Accept otherwise.

## Required Changes

- Change needed before acceptance, and the evidence that will show it is done.
