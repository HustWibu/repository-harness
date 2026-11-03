# Scripts

The normal validation entrypoint is:

```bash
scripts/validate-premerge.sh
```

## Installation

- `install-harness.sh`: Bash bootstrap for the versioned Rust `harness`
  candidate.
- `harness-install-files.txt`: exact embedded core payload.
- `engineering-wisdom-install-files.txt`: independent optional advisory
  payload.
- `claude-harness-block.md`: the managed Harness block of `CLAUDE.md`.

The bootstraps verify candidate checksum and reported version before delegating
install or update. They do not contain a database or compatibility profile.

## Core Release

- `build-harness-release.sh`: build one platform artifact and checksum.
- `harness-release-changed.sh`: classify changes that require a core release.
- `harness-release-tag`: current core release pointer.
- `verify-harness-release-identity.sh`: pretag and published-source identity
  guard.
- `verify-harness-release-assets.sh`: exact cross-platform asset inventory.
- `promote-harness-release-tag.sh`: promote a proven source commit.
- `render-changelog-files.py`: render bounded changed-file lists.

Release commands are called by GitHub workflows. Local development should use
the pre-merge entrypoint rather than publishing commands.

## Historical CLI

Protocol v1 and `harness-cli` are end-of-life. Their build, schema,
materialization, snapshot, changeset, release, and bootstrap scripts remain
available only through historical Git tags.
