# Installation Contract

Harness has one product profile and one independent advisory add-on. Claude
Code is the only supported agent host (decision 0031), on Linux and macOS
(decision 0032).

## Core

The exact core payload is declared in
`scripts/harness-install-files.txt`. It contains generic repository guidance,
optional knowledge and planning references, and explicit-only discovery,
audit, invariant-encoding, and improvement skills. Installation does not
activate skills or impose a task workflow.

The platform bootstrap installs a checksum-verified `harness` binary under
`scripts/bin/` and delegates installation or update to that candidate.

Core installation:

- records exact upstream bytes under `.harness-core/`;
- preserves consumer files through merge or human-directed conflict handling;
- backs up replaced files;
- does not install an application stack or product policy;
- does not install schemas, databases, orchestration, or background processes;
- does not delete pre-existing legacy Harness files.

## Engineering Wisdom Add-On

`--with-engineering-wisdom` copies the
explicit-only advisory skill declared in
`scripts/engineering-wisdom-install-files.txt`.

Omitting the flag does not install or activate the skill. A later install
without the flag leaves an existing copy untouched. Removal is explicit and
stateless: delete only `.claude/skills/engineering-wisdom/`.

Advice cannot establish consumer policy or authorize an architecture rewrite.

## Merge And Override

- `--merge`: preserve existing files and add missing managed
  paths.
- `--refresh-claude-shim`: append or refresh the marked
  Harness block in an existing `CLAUDE.md` after a backup. Text outside the
  markers is kept; malformed markers stop the install.
- `--override`: back up and replace protected Harness paths
  (`CLAUDE.md` and `docs/`).
- `--force`: overwrite individual managed files with backups.
- `--dry-run`: preview without writing.
- `--local-only`: keep the install out of `git status` (decision 0034). The
  target must be a Git repository root. Installed paths, `.harness-core/`,
  `.harness-backup/`, and `scripts/bin/harness` are listed in a marked block
  in `.git/info/exclude`; `.gitignore` is not touched. Implies `--merge` and
  rejects `--override` and `--force`. Tracked payload paths are reported and
  skipped; `--refresh-claude-shim` still edits a tracked `CLAUDE.md` and warns
  that the change shows in `git status`. A later install or `harness update`
  rewrites the block to match the installed paths, and `harness doctor`
  reports a `local_only` check.
- `--remove-local-only`: delete only the marked block from
  `.git/info/exclude` and exit; installed files remain.

## Shared Knowledge Hub

A hub is a repository with the normal core install whose Related Repositories
table lists member repositories by relative path (decision 0036).
`harness hub sync` writes a marked pointer block into each member's
`CLAUDE.md`; members receive no core files. It refuses members that are not
Git repository roots or whose `CLAUDE.md` is a symlink, keeps text outside the
markers, and never commits. `--dry-run` previews and `--check` exits 1 when a
member is stale or unreachable, or when the hub lacks its
`docs/services/<repository>/`. `--workspace` writes one block into the
`CLAUDE.md` of the directory holding the hub instead, naming the hub by
absolute path, and only checks members; it refuses when that directory is
inside a Git repository.

## Update

`harness update` verifies release identity and checksum, compares installed
base, local bytes, and incoming bytes, and applies the complete plan
transactionally.

Overlapping text edits stage a frozen resolution session. Structural conflicts
must be corrected before replanning. Successful activation writes provenance
last and replaces only the selected repository's executable after core files
succeed.

## Removed Profile

The former `--with-cli`, `--upgrade-cli`, `-WithCli`, and `-UpgradeCli`
profiles ended with protocol v1. Current installers reject those options.
Existing consumer databases and binaries are not automatically deleted.

The former `--claude` and `--refresh-agent-shim` options ended with the
`AGENTS.md` entrypoint, and the PowerShell bootstrap ended with Windows support
(decision 0032). The installer rejects the removed options.
