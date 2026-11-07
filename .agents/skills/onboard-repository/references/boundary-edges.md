# Boundary Edges

Open this when the traced operational path leaves the repository: it sends or
receives a message, calls another service over the network, or reads
configuration owned elsewhere. Without it the path can only end in **Unknown**;
with it the report says where the path goes and how sure that is.

## Record Each Edge

Add one row per edge after the operational-path table:

| Channel | Direction | Declared in this repository | Other repository | Confidence | Source |
| --- | --- | --- | --- | --- | --- |

- **Channel**: the exact name the code or configuration uses, such as a topic,
  queue, RPC method, or endpoint path. Mark a name resolved from a variable or
  property as **configurable** and give the key.
- **Direction**: produce, consume, call, or serve, from this repository's side.
- **Declared in**: file and line, or configuration key, in this repository.
- **Other repository**: the counterpart when evidence names it; otherwise
  **Unknown**. Do not infer it from naming similarity alone.
- **Confidence**: **confirmed** only after reading the counterpart's
  declaration; **unconfirmed** when only this repository's side was read;
  **Unknown** when the channel itself cannot be resolved.
- **Source**: the evidence for each side, as in the rest of the report.

Configuration in this repository may belong to a development environment only.
Record which environment a channel value comes from, and do not treat it as
the value another environment uses unless `docs/README.md` names this
repository as that environment's configuration source.

## Reading A Counterpart Repository

Read a counterpart only when both hold:

1. it is already checked out in the same workspace, located through Related
   Repositories in `docs/README.md` or a path the user gave; and
2. the user allowed reading it for this run.

Then apply the Safety Contract to it as well: read-only commands only, capture
its Git status before and after, and never fetch, pull, check out, build, or
edit it. Report its revision next to every edge it confirms.

If either condition fails, leave the edge **unconfirmed** and name the
repository and declaration a reader would need. Never clone or fetch a
repository to confirm an edge.

## Onboarding From A Hub

When the tested repository is a shared knowledge hub (decision 0036), run this
skill in the hub and name the member to onboard. The hub's Related Repositories
table locates every member, and the user's request to onboard a member allows
reading it and the members its edges reach. Cite member code with a source
`repository` and that member's revision, as `evidence-capsule-v2.md` defines;
both ends of an edge can then be confirmed. Start from the member's rows in
`docs/system-map.md` when the hub has one, and propose destinations in the hub:
`docs/services/<member>/`, `docs/contracts/`, and `docs/decisions/`. Give each
edge a status from `docs/patterns/multi-repository-evidence.md`, and when the
later pass writes a hub page, put a `harness-source` marker after each fact,
reusing the revision, range, and digest of its capsule source.

## Proposals

When edges reveal related repositories or truth owned elsewhere, the backfill
proposal may target the Related Repositories table and the Authority By Truth
Type table in `docs/README.md`. Propose only rows the evidence supports; leave
unknown locations empty rather than guess a path or URL. An edge that crosses
an interface contract may also propose a page from
`docs/templates/interface-contract.md` when the consumer keeps that directory.
