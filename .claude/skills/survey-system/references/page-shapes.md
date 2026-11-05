# Survey Page Shapes

Pages live in the hub. Keep each section; write "None found" with where you
looked instead of deleting an empty one.

## Member Page: `docs/services/<repository>/overview.md`

```markdown
# <repository> Overview

Surveyed at `<revision>` on `<branch>` by `/survey-system`. Describes current
code; confirm before using it for a decision.

## Purpose
Prose; tag inferences `Assumption:`.

## Entry Points And Commands
| Kind | Name or command | Evidence |
(services, listeners, schedulers, CLIs; build and test commands the repository defines)

## Provided Interfaces
| Kind | Name or path | Version or shape | Evidence |
(HTTP routes, RPC services, topics produced, files or tables others read)

## Consumed Interfaces
| Kind | Target | Name or path | Evidence |
(clients, topics consumed, shared stores, libraries pinned by version)

## Data Stores
| Store | Collection, table, or index | Access | Evidence |

## Configuration Per Environment
| Environment | Source of record | Keys that matter | Evidence |

## Operational Notes
Switches, failure behavior, and limits observed in code, with evidence.

## Sources
One `harness-source` marker per line, in the order rows cite them.
```

The Evidence cell holds `path#Lstart-Lend` or `<TODO: verify — question>`. The
Sources section holds the markers `scripts/pin_sources.py` printed for those
ranges, so `harness hub check-fresh` can follow every row.

## System Page: `docs/system/overview.md`

```markdown
# System Overview

Surveyed <date> by `/survey-system`.

## Scope
| List | Repositories | Reason |
(detected, in scope, out of scope)

## Revisions
| Repository | Branch | Revision | Dirty |

## Contract Matrix
| Channel | Provider | Consumer | Status | Provider evidence | Consumer evidence |
Statuses come only from docs/patterns/multi-repository-evidence.md.

## Counts
| Status | Count |
Each count must recompute from the matrix rows.

## Edges Outside Scope
| Channel | Repository in scope | Repository outside scope |

## Shared Data
| Store | Name | Repositories | Evidence |

## Answers
| Question | Answer, verbatim | Given by | Date |
Copied unchanged on every later survey.

## Open Questions
Markers not asked this run, grouped by repository.

## Sources
One `harness-source` marker per line.
```
