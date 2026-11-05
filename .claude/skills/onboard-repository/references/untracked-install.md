# Untracked Install

A local-only install (decision 0034) keeps this skill and the Harness files out
of Git, and a repository may ignore its `CLAUDE.md`. The evidence bundle pins
the skill, sources, and destinations with `git show <revision>:<path>`, so an
untracked file cannot be pinned.

1. In the boundary step, run `git ls-files --error-unmatch` for this skill's
   `SKILL.md` and for each intended destination. Report the result before
   tracing.
2. If the skill is untracked, say at the start that no evidence bundle can be
   emitted. Continue the inspection; in step 5 report "evidence bundle
   unavailable: untracked install" as a limitation instead of running the
   emitter.
3. If a destination is untracked or ignored, show the exact proposed wording
   with its file path. Approval applies to that wording; the later pass writes
   it without rendered hunk IDs and reports the resulting diff.
4. If a shared knowledge hub is named, in `docs/README.md` or in a
   `HARNESS-HUB` block of a loaded `CLAUDE.md`, propose knowledge beyond this
   repository's code into the hub instead: `<hub>/docs/services/<repository>/`,
   decisions into `<hub>/docs/decisions/`, contracts into
   `<hub>/docs/contracts/`. Writing the hub is a separate approved write in
   another repository.
