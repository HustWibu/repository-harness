# Documentation Map

Start with the smallest authoritative surface.

## Current Product

- `WORKFLOW.md`: repository understanding, missing capabilities, authority,
  and evidence.
- `ARCHITECTURE.md`: current product, code, state, update, and ownership
  boundaries.
- `HARNESS.md`: product principles and installed-core model.
- `product/`: current product behavior and installation contract.
- `decisions/`: lasting choices future work must inherit.
- `plans/`: optional locations for plans requested by the user or consumer workflow.
- [`patterns/encoding-invariants.md`](patterns/encoding-invariants.md): turn
  accepted architecture, reliability, security, and quality rules into native
  mechanical validation.
  It also covers fail-closed evidence gates and when to promote guidance to a
  gate.
- `research/`: exploratory notes, not product authority or release gates.
  `harness-gap-analysis.md` and `harness-requirements-ideas.md` (Vietnamese)
  compare this harness with the Learn Harness Engineering course.
  `multi-repo-requirements.md` (Vietnamese) holds the multi-repository
  requirements behind the optional Related Repositories surfaces.
- `templates/`: optional decision, plan, runbook, session-handoff,
  evaluator-rubric, interface-contract, and Harness-improvement structures.
  The decision template carries checkable approval frontmatter.

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
missing members without writing.

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

## Source Repository

- Root `README.md`: product overview, installation, maintenance, EOL, and
  development.
- `crates/harness/`: safe core installer/updater.
- `scripts/`: platform bootstrap, release, and validation entrypoints;
  `context-budget.tsv` holds the context size limits.
- `tests/`: behavior ownership and repository contract.

## History

The former SQLite control plane, protocol v1, story packets, migration evidence,
and compatibility documentation are preserved by Git history and immutable
`harness-cli-v*` tags. They are intentionally absent from the current tree so
search and agent retrieval return current product authority.
