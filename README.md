# repository-harness

Turn a software repository into a legible, agent-ready workspace.

`repository-harness` installs a small repository protocol and a safe updater.
The repository remains the system of record: product documents, decisions,
code, tests, CI, and runtime evidence define the work.

It is not a task database, story tracker, agent orchestrator, or application
runtime.

## What It Solves

Coding agents often fail for ordinary engineering reasons:

- important intent exists only in chat;
- the repository does not identify authoritative documents;
- small changes acquire unnecessary process;
- long changes lose decisions and recovery context;
- completion is claimed without behavior-level proof; and
- an agent invents product policy when the request leaves a material choice
  open.

Harness supports Claude Code and Codex. It provides compact `CLAUDE.md` and
`AGENTS.md` entrypoints, a navigable repository map, optional
knowledge-discovery tools, and a safe maintenance binary.

## Repository Understanding

Find current behavior in code, types, interfaces, commands, tests, and examples.
Keep product intent and rationale where the repository already owns them.
Identify missing capabilities needed to act, observe, or verify, and report
uncertainty rather than inventing commands or claiming unsupported results.

Consumer repositories own task workflow and planning. Harness selects relevant
skills automatically from the requested outcome, within the authorized scope.

Start with [`CLAUDE.md`](CLAUDE.md) or [`AGENTS.md`](AGENTS.md) and the repository map in
[`docs/README.md`](docs/README.md).

## What Gets Installed

The default core contains:

- compact `CLAUDE.md` and `AGENTS.md` entrypoints for Claude Code and Codex;
- repository context and navigation;
- optional product, decision, and execution-plan locations;
- optional templates for durable plans, decisions, application runbooks,
  session handoffs, evaluator rubrics, cross-repository interface contracts,
  and evidence-backed Harness improvements; and
- an invariant-encoding reference and task-triggered skills under
  `.claude/skills/` and `.agents/skills/` for discovery, audit, encoding, and improvement.

It does not install application architecture, product policy, validation
commands, credentials, a database, schemas, orchestration, or background
processes.

The exact payload is declared in
[`scripts/harness-install-files.txt`](scripts/harness-install-files.txt).

## Install

For automatic startup in every project on this machine, install once for both
Claude Code and Codex:

```bash
scripts/install-harness.sh --global --yes
```

From this checkout, that command builds the current core. Remote installation:

```bash
curl -fsSL "https://raw.githubusercontent.com/HustWibu/repository-harness/main/scripts/install-harness.sh?$(date +%s)" |
  bash -s -- --global --yes
```

The remote command requires a published release containing global support.
The core lives in `~/.local/share/repository-harness` (or under `XDG_DATA_HOME`).
The installer backs up and adds a managed block to `~/.claude/CLAUDE.md` and
`~/.codex/AGENTS.md`, and links skills into `~/.claude/skills/` and
`~/.agents/skills/`. A nonempty `AGENTS.override.md` receives the Codex block
instead. `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are respected. Existing unrelated
skills and configuration stay intact; same-name skill collisions stop before
installation. Start a new session after installation.

Use `--global --dry-run` to preview. Rerun `--global --yes` to update the core
and refresh registration. `--remove-global` removes only registered links and
managed instruction blocks; it preserves the installed core and user rules.
Global registration uses symlinks; managed core files remain regular files.
No per-project Harness install is needed. A repository's own instructions,
commands, and product decisions remain authoritative.

For an installation committed or kept locally in one repository:

From a target repository:

```bash
curl -fsSL "https://raw.githubusercontent.com/HustWibu/repository-harness/main/scripts/install-harness.sh?$(date +%s)" |
  bash -s -- --yes
```

Harness supports Linux and macOS (x64 and arm64). Windows is not supported.

Use `--merge` to preserve existing files and add only missing
Harness paths. Existing `CLAUDE.md` and `AGENTS.md` are kept as is; add
`--refresh-claude-shim` and/or `--refresh-agent-shim` to append or refresh
the marked Harness block in the corresponding file. Use `--override` only when replacement
is intentional. Use `--dry-run` to preview.

Use `--local-only` to try Harness on a shared repository without committing
it: every installed path goes to `.git/info/exclude`, tracked files are left
unchanged, and `--remove-local-only` deletes only those lines (decision 0034).

For many repositories, make one of them a shared knowledge hub: list the
members in its `docs/README.md` and run `scripts/bin/harness hub sync` there to
point each member's `CLAUDE.md` and `AGENTS.md` at the hub, or `hub sync --workspace` to write
pointers into `CLAUDE.md` and `AGENTS.md` of the directory holding them, plus a Codex pointer in each member Git root
(decisions 0036 and 0041).
`hub check-fresh` reports hub facts whose cited member code has changed
(decision 0039).

The bootstrap downloads a versioned `harness` binary and checksum, verifies
release identity, and delegates installation to that candidate.

## Maintain A Repository Installation

```bash
scripts/bin/harness status
scripts/bin/harness doctor
scripts/bin/harness update --dry-run
scripts/bin/harness update
```

The updater stores the exact upstream base under `.harness-core/`, performs a
three-way merge, backs up changed files, and activates the result
transactionally.

If local and upstream edits overlap, no managed file or executable changes.
Harness retains BASE, LOCAL, UPSTREAM, and RESOLVED copies plus the frozen
managed input set. After a human resolves the semantic choice:

```bash
scripts/bin/harness update --continue --dry-run
scripts/bin/harness update --continue
```

Use `scripts/bin/harness update --abort` to discard only the staged resolution.

## Measure Skill Use

```bash
scripts/bin/harness skill-report ~/.claude/projects/<project>/<session>.jsonl
```

`skill-report` is read-only and runs only when invoked. It reads the Claude
Code session files you name and reports, per recorded skill invocation,
invocations, `HARNESS_SKILL:` receipts, and input, cache-creation, cache-read,
and output tokens (`--json` for machines). It prints aggregates only, writes
nothing, and rejects other session formats. Receipts are self-reported, so the
report measures format compliance, not whether the work was correct.

## Optional Skills

Invariant enforcement routes accepted rules through repository-native
validation:

```text
/encode-invariant
```

Brownfield onboarding is explicit and read-only first:

```text
/onboard-repository
```

A multi-repository survey runs in a shared knowledge hub, read-only until you
approve its pages: per-repository pages, a contract matrix with closed
statuses, pinned sources that `hub check-fresh` follows, and open questions
(decision 0040):

```text
/survey-system
```

Harness improvement is also explicit and requires baseline-to-rerun evidence:

```text
/improve-harness
```

Engineering advice is a separate opt-in payload:

```bash
scripts/install-harness.sh --with-engineering-wisdom --yes /path/to/project
```

Harness selects skills automatically when a request matches their description.
You can also use `/<name>` in Claude Code or `$<name>` in Codex. Claude skills
allow model invocation; Codex adapters set `policy.allow_implicit_invocation:
true`. Skill selection grants no new permission to edit, operate, or publish.
Engineering wisdom still requires the installation opt-in flag.

The shared protocol comes from `scripts/claude-harness-block.md`. Claude skills
are the authoring source; `python3 scripts/sync-codex-skills.py` renders the
Codex adapters, and pre-merge validation rejects drift. Onboarding evidence
validation accepts Claude Code JSONL and Codex `response_item` JSONL with a
`final_answer` message. `skill-report` remains Claude Code only.

## What We Prove

Harness owns three release-evidence boundaries:

1. **Fresh installation:** the declared core is installed without fabricated
   application truth or hidden lifecycle state.
2. **Repository navigation:** an agent follows repository authority, avoids
   speculative product policy, and can stop at a real decision boundary.
3. **Safe maintenance:** updates verify identity and checksum, preserve local
   edits, stage conflicts, reject drift, and recover interrupted transactions.

Operating an arbitrary consumer application end to end remains consumer-owned
research. Harness does not claim that installation alone supplies runtimes,
fixtures, credentials, logs, or interface automation.

## Protocol V1 End Of Life

The former SQLite `harness-cli` and machine protocol v1 ended support on
2026-08-10. The last published compatibility release is
`harness-cli-v0.1.22`. Existing consumers may pin that immutable release, but
the current repository no longer builds, installs, tests, or publishes it.

Harness does not automatically delete legacy binaries, databases, schemas, or
state from consumer repositories.

See
[`decision 0027`](docs/decisions/0027-end-protocol-v1-and-focus-repository-protocol.md).

## Development

```bash
scripts/validate-premerge.sh
```

The contract runs Rust formatting, tests, Clippy, installer and workflow
checks, release guards, documentation checks, shell syntax, and
`git diff --check`.
