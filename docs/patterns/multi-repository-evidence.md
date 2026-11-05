# Multi-Repository Evidence

Use these rules when knowledge spans repositories: a hub page about a member, a
contract between services, or a summary built from many repositories. Each one
is a check a reader or a script can repeat, not a style preference.

## Source Markers

A hub page states facts about member code. Pin each fact on the line after it:

```markdown
The consumer drops a batch while the flag is OFF.
<!-- harness-source: orders-service@<40-hex revision>:src/Consumer.java#L55-L58 sha256=<64-hex> -->
```

- The repository is a Related Repositories row of the hub; the revision is a
  commit of that member; the digest is SHA-256 of the exact cited lines with
  their line endings, the same digest the onboarding evidence capsule pins.
- `harness hub check-fresh`, run in the hub, re-reads every cited range at the
  member's HEAD and reports `current`, `moved` (same lines, new position),
  `stale` (the lines changed or the file is gone), or `error` (unlisted
  repository, malformed marker). It exits 1 on `stale` or `error`, so it can
  gate CI. A stale fact is re-checked against the code, not deleted unseen.

## Contract Status

An edge between repositories gets exactly one status:

| Status | Meaning | Evidence required |
| --- | --- | --- |
| `MATCHED` | Provider and consumer agree on channel, version, and shape | One pinned source in each repository |
| `VERSION-MISMATCH` | Same channel, different version | Both sources and the two versions |
| `DTO-DRIFT` | Same channel, different fields | Both sources and the differing fields |
| `CONSUMER-ONLY` | A consumer with no provider found | Consumer source; where the provider was sought |
| `PROVIDER-ORPHAN` | A provider with no consumer found | Provider source; where consumers were sought |
| `UNKNOWN` | Evidence is not enough to classify | An open question |

Never invent another status, and never write `MATCHED` from one side's code:
both sides are read, or the edge stays `UNKNOWN`. A channel used inside one
repository only is not a cross-repository edge.

## Changing A Contract

A change that alters a contract's shape (channel name, version, fields,
semantics, or adds a consumer or an external system) is not a local change.
Update the contract page, list every consumer under not verified until each
one is checked, and record the decision in the repository that owns the
contract. When it is unclear whether a change touches a contract, stop and
ask; guessing either way hides the risk.

## Counts Must Reconcile

A summary built from a machine-generated artifact repeats its counts, and the
counts must recompute from the artifact: for example, the edges that are not
`MATCHED` equal the artifact's mismatch rows. A summary that finds fewer
problems than the artifact is a broken synthesis, not a clean system; report
the discrepancy instead of adjusting either side. Read the artifact rather than
recounting by hand.

## Scope Honesty

Report three lists: repositories detected in the workspace, repositories
listed or scanned, and those left out with a reason. Never drop a detected
repository silently; `harness hub sync --workspace` prints Git repositories in
the workspace that Related Repositories does not list.

## Open Questions

- Mark a missing fact where it is needed:
  `<TODO: verify — one concrete, answerable question>`. One marker, one
  question; never a bare `<TODO: verify>`.
- Collect markers with `grep -rn '<TODO: verify — '`, merge exact duplicates
  across repositories, and ask a person only a few at a time, highest impact
  first; the rest stay inline.
- Record each answer verbatim with who gave it. A later regeneration copies
  recorded answers unchanged and never asks them again; losing a recorded
  answer is a failure of that regeneration.
