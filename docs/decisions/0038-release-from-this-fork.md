---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0038 Release From This Fork

Date: 2026-11-05

## Context

The bootstrap, `scripts/install-harness.sh`, and `harness update` fetched the
release pointer and binaries from `hoangnb24/repository-harness`. That upstream
release (`harness-v0.1.10`) lacks decisions 0031 and 0034–0037, so installing
or updating a consumer silently removed them; the CPM hub had to receive its
skills by hand. The fork owner asked to release from this fork.

## Decision

1. Release URLs point at `HustWibu/repository-harness`: the release pointer
   and download base in `crates/harness/src/infrastructure/release_handoff.rs`,
   and the source, core, and binary URLs in `scripts/install-harness.sh` and
   the README bootstrap.
2. The first fork release is `harness-v0.2.0`: crate version, `Cargo.lock`, and
   `scripts/harness-release-tag` agree, as the release identity check requires.
   The version leaves 0.1.x so it never shares a number with an upstream
   release of different content.
3. Releases are cut by the existing `Harness Core Release` workflow
   (`workflow_dispatch`) in this fork; it uses `$GITHUB_REPOSITORY`, so it
   needs no change.

## Alternatives Considered

1. **Contribute the changes upstream and keep upstream URLs.** Not chosen by
   the fork owner; it would also leave installs without these changes until
   upstream accepted and released them.
2. **Reuse `harness-v0.1.10` in the fork.** Rejected: the same tag would name
   different content in two repositories.

## Consequences

- Installs and updates follow this fork; upstream releases no longer reach
  them unless merged here.
- Until the first fork release is published, the bootstrap and `harness
  update` find no release and fail.

## Follow-Up

- Publish `harness-v0.2.0` by running `Harness Core Release` in this fork with
  `release_tag=harness-v0.2.0` and `checkout_ref=main`, then verify a fresh
  bootstrap install and `harness update` in the CPM hub.
