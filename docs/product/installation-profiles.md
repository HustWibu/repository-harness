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
