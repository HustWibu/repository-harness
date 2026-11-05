# Documentation Map

Start with the smallest authoritative surface. This map is the consumer's: list
this repository's own code, commands, tests, and product documents here, and
keep the sections below that apply.

## Harness Surfaces

- `WORKFLOW.md`: repository understanding, missing capabilities, authority,
  and evidence.
- `product/`, `decisions/`, `plans/`: optional homes for product behavior,
  lasting choices, and plans the user or consumer workflow asks for.
- [`patterns/encoding-invariants.md`](patterns/encoding-invariants.md): turn
  accepted architecture, reliability, security, and quality rules into native
  mechanical validation.
- `templates/`: optional decision, plan, runbook, session-handoff,
  evaluator-rubric, interface-contract, and Harness-improvement structures.
  The decision template carries checkable approval frontmatter.

## This Repository

Not yet described. Name the entry points, build and test commands, and where
product intent lives, as references to files rather than summaries.

## Consumer-Owned Truth

The consumer's README, product documents, architecture, code, tests, CI,
runtime signals, and application behavior remain authoritative. Harness does
not overwrite those with upstream product assumptions.

## Related Repositories

Optional. Fill this in when some truth this repository depends on lives in
another repository, as in a multi-repository or microservice system. Leave it
empty for a single repository; nothing else changes.

| Repository | Role for this repository | Location |
| --- | --- | --- |

Role is one of: contract, environment configuration, specification, shared
library, or another named role. Location is a URL or a path relative to this
repository, such as `../orders-service`; never an absolute path from one
machine.

### Shared Knowledge Hub

Optional (decision 0036). To keep one home for a multi-repository system's
knowledge, make one repository the hub: install Harness there, keep system
decisions, contracts, and plans in its `docs/`, and knowledge for one member
in `docs/services/<repository>/`. Every row above whose Location is a relative
path is a member. Run `scripts/bin/harness hub sync` in the hub to write a
pointer block into each member's `CLAUDE.md`; `--check` reports stale or
missing members without writing. When members are cloned side by side and do
not commit `CLAUDE.md`, use `hub sync --workspace` instead: it writes one block
into the `CLAUDE.md` of the directory holding the hub, which Claude Code loads
in every member below it.

### Authority By Truth Type

Each truth type has exactly one source of record. Every other copy only points
to it, and an agent treats a disagreeing copy as a finding, not as authority.

| Truth type | Source repository | Path in that repository |
| --- | --- | --- |
| Interface contract | | |
| Configuration for an environment | | |
| Requirements | | |
| Cross-repository progress | | |

Interface contracts recorded with `templates/interface-contract.md` live in a
directory named here, for example `docs/contracts/`.
