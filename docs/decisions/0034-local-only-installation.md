---
status: accepted
approved_by: "chiendv9"
approved_at: "2026-11-05"
superseded_by: ""
---

# 0034 Local-Only Installation

Date: 2026-11-05

## Context

People want to use Harness on a team repository before the team agrees to
commit it. Installing normally adds untracked files and a `.gitignore` rule,
so every `git status` and careless `git add .` risks committing Harness.
Decision 0020 defines the installation profile but has no mode that stays out
of version control.

## Decision

1. The bootstrap accepts `--local-only`. The target must be the root of a Git
   repository; otherwise it stops before writing anything.
2. It writes a marked block to the path Git reports for `info/exclude`:
   `# BEGIN harness local-only (managed by harness; do not edit)` …
   `# END harness local-only`. Lines outside the block belong to the user and
   are never changed.
3. The block lists every installed payload path, `.harness-core/`,
   `.harness-backup/`, and `scripts/bin/harness`, each anchored with a leading
   `/`. `.gitignore` is not touched.
4. Local-only implies `--merge` and rejects `--override` and `--force`,
   because those move or overwrite tracked files. Tracked payload paths are
   reported and skipped. `--refresh-claude-shim` remains an explicit request to
   edit a tracked `CLAUDE.md`; the bootstrap then warns that the change shows in
   `git status`.
5. The block marks the install as local-only. Any later successful install or
   update transaction rewrites the block to match the installed paths, and a
   later bootstrap run without the flag detects the block. `harness doctor`
   adds a `local_only` check that fails when an installed path is missing from
   the block.
6. `--remove-local-only` deletes only the block and exits; installed files stay.

## Alternatives Considered

1. **A separate decision versus extending 0020.** Chosen separate: the mode
   adds a write outside the target tree (inside `.git/`) and a new doctor
   check, which are externally observable.
2. **Write a broad pattern such as `/docs/`.** Rejected: it would hide the
   consumer's own new untracked documents.
3. **Use a global excludes file.** Rejected: it affects every repository on
   the machine.

## Consequences

Positive:

- `git status` stays clean after install and update; removal is exact.

Tradeoffs:

- Exclude rules do not apply to tracked files, so a Harness file the team
  already committed is never hidden; local-only only skips it.
- A target that is a subdirectory of a repository is refused rather than
  translating paths to the repository root.
- The block is per clone; a new clone needs the install again.

## Follow-Up

- Proof: `tests/installer/test-install-local-only.sh` and the `local_only`
  unit tests in `crates/harness/src/infrastructure/local_only.rs`.
