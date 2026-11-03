---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0030 Evidence-First Completion, Handoff, Rubric, And Gate Retirement

Date: 2026-11-03

## Context

`docs/research/harness-gap-analysis.md` compared Harness with the Learn Harness
Engineering course. Four gaps fit the product boundary of decision 0020 because
they are text or optional templates and add no state or machinery:

- The rule that completion needs evidence existed in `docs/WORKFLOW.md` but had
  no visible "done" definition that agents and reviewers could cite.
- Work spanning two or three sessions had no light handoff. The execution plan
  suits multi-day work and is heavy for a short resume.
- Reviews of agent output used different criteria each time.
- The encoding pattern said when to add a gate but not when to remove one, so
  checks could outlive their reason.

## Decision

1. `docs/WORKFLOW.md` gains a short "Definition Of Done" section: done requires
   observed evidence for the changed behavior; reports separate verified, not
   verified, and known broken; missing evidence means not verified.
2. Core ships `docs/templates/session-handoff.md`, which combines the handoff
   and the clean-state checklist and advises writing it before the context
   window is mostly used.
3. Core ships `docs/templates/evaluator-rubric.md` with six criteria scored
   0 to 2 and an Accept, Revise, or Block verdict. Its criteria restate
   `docs/WORKFLOW.md` and add no consumer policy.
4. The encoding pattern's boundary table gains a "Retirement" field, and the
   promotion section asks for a retirement condition with every gate.

All additions are optional references. No skill, command, hook, or task state
is added, and no template contains stack-specific commands.

## Alternatives Considered

1. Separate handoff and clean-state templates. Rejected: most fields overlap,
   and two files double maintenance for one end-of-session moment.
2. Put the handoff advice in `AGENTS.md`. Rejected: it would cost context in
   every session for guidance needed only at the end of long work.
3. Ship a review skill with the rubric. Deferred: the gap analysis waits for
   `harness skill-report` data showing existing skills are followed well enough
   to justify another one.
4. Ship a generic done-gate script. Rejected for the reasons in decision 0029.

## Consequences

Positive:

- Agents and reviewers can cite one definition of done.
- Short multi-session work resumes from recorded evidence.
- Reviews become comparable across runs and reviewers.
- Gates carry their own removal condition.

Tradeoffs:

- Consumers that edited `docs/WORKFLOW.md` or the encoding pattern will meet
  the update merge or conflict flow on the next `harness update`.
- Templates are guidance; nothing proves they are used or filled honestly.
- The entry reading path grows by about fifty words, within its budget.

## Follow-Up

- Accept or revise this decision (set `status`, `approved_by`, `approved_at`).
- After several real uses, evaluate the rubric and handoff templates with
  `$improve-harness` and remove whichever shows no benefit.
