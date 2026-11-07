# Test Suite Map

The normal entrypoint is `scripts/validate-premerge.sh`.

## Rust Core

`crates/harness/` unit and integration tests protect:

- path, hash, provenance, and distribution validation;
- clean architecture, including allowed and forbidden nested-module fixtures;
- dual-host install, status, doctor, upgrade, and local-file collision handling;
- three-way updates and conflict staging;
- complete-plan drift detection;
- checksum and release identity;
- symlink rejection;
- transaction rollback and executable recovery.

## Repository Contracts

| Location | Protects |
| --- | --- |
| `tests/workflow/` | Installed repository context and automatic skill configuration (textual checks), plus Claude/Codex onboarding transcript evidence (CLI integration) |
| `tests/installer/` | Global startup wiring, backup/rollback/removal, fresh dual-host installation, generated adapter parity, merge/override, shims, optional engineering advice, manifest integrity, and platform parity |
| `tests/docs/` | Current authority, links, EOL boundary, and validation entrypoints |
| `tests/maintenance/` | Core release classification and changelog rendering |
| `tests/release/` | Core workflow, exact assets, source identity, promotion, and post-merge recovery |

## Removed Compatibility Proof

SQLite schemas, snapshots, changesets, protocol-v1 commands, and
`harness-cli` release tests ended with decision 0027. Immutable historical
tags retain that proof; it is not run by the current product.

When adding a test, name the observable invariant and update this map. A
historical artifact alone is not a reason to keep an executable in pre-merge.
