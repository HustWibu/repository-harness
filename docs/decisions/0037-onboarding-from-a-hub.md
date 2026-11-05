---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0037 Onboarding From A Hub

Date: 2026-11-05

## Context

A trial of `/onboard-repository` inside one member of a 47-repository system
(`campaign-goal-service`) traced its Kafka path well but produced nothing
usable for the hub:

- every edge stayed unconfirmed because the session could not read siblings;
- its proposals targeted the member's `CLAUDE.md`, not the hub;
- under a local-only install every destination and the skill itself were
  untracked, so no evidence bundle could be pinned.

The evidence capsule pinned every source with `git show` in the tested
repository only, so a capsule produced in the hub could not cite member code.

## Decision

1. **Onboard members from the hub.** The skill runs in the hub and names the
   member; `references/boundary-edges.md` describes the flow. The hub's Related
   Repositories table locates members, and the request to onboard a member
   allows reading it and the members its edges reach, read-only.
2. **A capsule source may name another repository.** An optional
   `repository` on a v2 source names a Related Repositories row of the tested
   repository's `docs/README.md` at the tested revision, whose relative
   Location is a Git repository root; the source must carry that repository's
   own revision. The emitter and the validator resolve the same row and pin the
   same bytes. Sources without the field are unchanged, so existing v2
   capsules stay valid and no v3 schema is introduced; the producer and
   validator ship together in one payload.
3. **Destinations live in the hub**: `docs/services/<member>/`,
   `docs/contracts/`, `docs/decisions/`. A member is not edited.
4. **A generated system map is the starting point.** A hub may keep a script
   that rebuilds `docs/system-map.md` from member code and configuration. It
   belongs to the hub, not to Harness, because channels and configuration
   layouts differ per system.

## Alternatives Considered

1. **Run the skill once per member, inside each member.** Rejected: edges stay
   one-sided, results land outside the hub, and local-only installs cannot pin
   evidence.
2. **A new v3 capsule schema.** Rejected for now: one optional field covers
   cross-repository sources, and a version bump would fork the markers and the
   validator without new guarantees.
3. **Use an external code-graph index as the map.** Rejected as the source of
   record: the available index covered HTTP contracts only and was up to 118
   commits behind; it can still help a reader.

## Consequences

Positive:

- Both ends of an edge can be pinned in one capsule and checked by the
  validator.
- Knowledge from onboarding accumulates in one reviewed repository.

Tradeoffs:

- A source pinned in a member names a commit that exists only in that member's
  clone; validation needs the same workspace layout.
- The validator trusts the Related Repositories table at the tested revision to
  say which directory a name means.

## Follow-Up

- Proof: emitter and validator self-tests cover a related source and reject one
  without its own revision; a bundle emitted in the CPM hub citing
  `campaign-goal-service` validated, and a forged source hash was rejected.
- Run a full onboarding of one member from the hub and record whether the
  confirmed edges match `docs/system-map.md`.
