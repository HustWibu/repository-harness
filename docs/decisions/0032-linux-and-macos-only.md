---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0032 Linux And macOS Only

Date: 2026-11-03

## Context

Harness kept a PowerShell bootstrap (`scripts/install-harness.ps1`), its own
installer test, a `windows-latest` pre-merge job, and a `windows-x64` release
build. Every installer change had to be written twice, and the PowerShell
side could not be run on the Linux and macOS machines that develop and use
this fork. The Bash bootstrap already covers both of those platforms, and the
Linux pre-merge job already runs its tests.

## Decision

1. Harness supports Linux and macOS on x64 and arm64. Windows is not
   supported.
2. Remove `scripts/install-harness.ps1`,
   `tests/installer/test-install-harness-modes.ps1`, the `windows-installer`
   pre-merge job, and the `windows-x64` release build and smoke step.
3. Releases publish four binaries and four checksums. The installer writes
   only the `scripts/bin/harness` ignore rule.
4. Pre-merge validation no longer requires `ripgrep`; the documentation
   contract uses `grep -F`, which every supported platform ships.

## Consequences

- One bootstrap and one installer test remain, and all of pre-merge runs on a
  single Ubuntu job.
- Windows consumers have no supported install path. The Rust updater rejects
  Windows as an unsupported platform instead of looking for a Windows asset.
- Existing consumer `.gitignore` files keep any `scripts/bin/harness.exe` rule
  written by an earlier installer. It is harmless and is not removed.

## Open: Workflows Not Yet Updated

`.github/workflows/premerge.yml`, `.github/workflows/harness-release.yml`, and
`tests/release/test-harness-release-workflow-contract.sh` still describe the
Windows-era setup and are deliberately left for a later change. Until then:

- the `windows-installer` pre-merge job fails, because it runs the removed
  `tests/installer/test-install-harness-modes.ps1`;
- a release run fails at the `windows-x64` build, because
  `scripts/build-harness-release.sh` no longer accepts that target, and the
  asset inventory and published-asset count expect five platforms (ten
  assets) instead of four (eight);
- both workflows still install `ripgrep`, which validation no longer needs.

Finishing this decision means removing the Windows job, matrix entry, and
smoke step, setting the asset count to eight, dropping `ripgrep`, and changing
the workflow contract test to expect four platforms.

## Revisit When

A real consumer needs Windows. Restoring it means restoring the PowerShell
bootstrap, its test, the Windows job, and the release target together.
