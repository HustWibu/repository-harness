# Installation Contract

Harness has one product profile and one independent advisory add-on. It supports
Claude Code and Codex (decision 0041), on Linux and macOS (decision 0032).

## Core

The exact core payload is declared in
`scripts/harness-install-files.txt`. It contains generic repository guidance,
optional knowledge and planning references, and task-triggered discovery,
audit, invariant-encoding, and improvement skills. Relevant requests select skills automatically; selection does not expand
authorization or impose a planning lifecycle.

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
advisory skill declared in
`scripts/engineering-wisdom-install-files.txt`.

Omitting the flag does not install or activate the skill. A later install
without the flag leaves an existing copy untouched. Removal is explicit and
stateless: delete `.claude/skills/engineering-wisdom/` and
`.agents/skills/engineering-wisdom/`.

Advice cannot establish consumer policy or authorize an architecture rewrite.

## Merge And Override

- `--merge`: preserve existing files and add missing managed
  paths.
- `--refresh-agent-shim`: append or refresh the same marked block in
  `AGENTS.md`, preserving consumer text and making a backup.
- `--refresh-claude-shim`: append or refresh the marked
  Harness block in an existing `CLAUDE.md` after a backup. Text outside the
  markers is kept; malformed markers stop the install.
- `--override`: back up and replace protected Harness paths
  (`CLAUDE.md`, `AGENTS.md` and `docs/`).
- `--force`: overwrite individual managed files with backups.
- `--dry-run`: preview without writing.
- `--local-only`: keep the install out of `git status` (decision 0034). The
  target must be a Git repository root. Installed paths, `.harness-core/`,
  `.harness-backup/`, and `scripts/bin/harness` are listed in a marked block
  in `.git/info/exclude`; `.gitignore` is not touched. Implies `--merge` and
  rejects `--override` and `--force`. Tracked payload paths are reported and
  skipped; either refresh flag still edits its tracked instruction file and warns
  that the change shows in `git status`. A later install or `harness update`
  rewrites the block to match the installed paths, and `harness doctor`
  reports a `local_only` check.
- `--remove-local-only`: delete only the marked block from
  `.git/info/exclude` and exit; installed files remain.

## Shared Knowledge Hub

A hub is a repository with the normal core install whose Related Repositories
table lists member repositories by relative path (decision 0036).
`harness hub sync` writes a marked pointer block into each member's
`CLAUDE.md` and `AGENTS.md`; members receive no core files. It refuses members that are not
Git repository roots or whose instruction files are symlinks, keeps text outside the
markers, and never commits. `--dry-run` previews and `--check` exits 1 when a
member is stale or unreachable, or when the hub lacks its
`docs/services/<repository>/`. `--workspace` writes one block into the
`CLAUDE.md` and `AGENTS.md` of the directory holding the hub instead, naming the hub by
absolute path, and writes a Codex pointer into each member
`AGENTS.md` because Codex discovery begins at the member Git root; it refuses when that directory is
inside a Git repository, and lists workspace repositories the hub does not.
`harness hub check-fresh` re-reads every `harness-source` marker in the hub's
`docs/` at the member's HEAD and exits 1 when cited lines changed
(decision 0039).

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

The former `--claude` selector is rejected because both hosts install by
default. The PowerShell bootstrap ended with Windows support (decision 0032).

Codex support (decision 0041) adds `AGENTS.md` and `.agents/skills/` to every
core installation. The advisory add-on installs for both hosts. Hub sync writes
both instruction files in member and workspace modes. Existing files remain
protected by the same merge, backup, local-only, and conflict policies.

## Global Installation

`--global --yes` installs one maintained core below
`${XDG_DATA_HOME:-~/.local/share}/repository-harness` and registers both hosts.
User instructions receive a backed-up marked block. Skill directories are
links to the core; a collision with another skill stops before installation.
User settings files are never rewritten. Custom `CLAUDE_CONFIG_DIR` and
`CODEX_HOME` are honored, including a nonempty Codex `AGENTS.override.md`.

`--global --dry-run` previews without creating the core or registration.
Rerun the global installer to update and refresh registration. Removal with
`--remove-global` deletes only owned links and instruction blocks. Activation
of global changes rolls back on failure and retains instruction backups under
`.harness-global-backup/` in the core. Core updates retain the existing
three-way merge and conflict behavior. `--global` cannot combine with a
repository directory, local-only mode, force, or override.

The global block directs missing Harness references to its installed docs;
application knowledge always comes from the current repository. Matching
skills may run automatically; their existing scope and approval gates remain.
Use native validation and CI for enforced rules, not instruction compliance.
