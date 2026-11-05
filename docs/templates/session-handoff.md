# Session Handoff: Title

Date: YYYY-MM-DD

<!--
Optional. Write this before the context window is mostly used, not after it
overflows, so the next session resumes from evidence instead of guessing.
Use an execution plan in docs/plans/active/ for work that spans many days.
-->

## Goal

State in one sentence the observable outcome this work is driving toward.

## Verified

- Behavior, and the command or observation that proved it, with its result.

## Changed But Not Verified

- File or behavior, and why it could not be checked yet.

## Known Broken Or Open

- Failing check, blocker, or unresolved question, with its owner if known.

## Repositories Touched

Optional. Use it when the work spans repositories; the plan's home repository
holds the full table.

| Repository | Branch | Pull request or change | State | Evidence so far |
| --- | --- | --- | --- | --- |

Unfinished work per repository:

- Repository: what remains and where it starts.

## Next Step

Name the single next action and the repository-native command or source that
starts it.

## Clean-State Check

- [ ] Repository-native checks ran, or are listed above as not run.
- [ ] No temporary files, debug code, or stray processes remain.
- [ ] Unfinished work is committed, stashed, or described above.
