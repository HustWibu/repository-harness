---
status: proposed
approved_by: ""
approved_at: ""
superseded_by: ""
---

# 0031 Claude Code As The Only Agent Host

Date: 2026-11-03

## Context

Harness shipped one canonical `AGENTS.md` entrypoint plus an optional
`CLAUDE.md` shim that only imported it, and its skills lived under
`.agents/skills/` with Codex metadata in `agents/openai.yaml`. Claude Code
reads neither `AGENTS.md` nor `.agents/skills/` on its own, so a Claude Code
consumer needed an extra installer flag for instructions and had no working
skill discovery at all. The onboarding audit validator also authenticated only
Codex transcripts, while `harness skill-report` already read only Claude Code
sessions.

Supporting two hosts doubled the entrypoint and metadata surface without a
consumer who needed the second host.

## Decision

1. Claude Code is the only supported agent host.
2. `CLAUDE.md` is the single canonical, managed entrypoint. Its Harness block
   lives in `scripts/claude-harness-block.md` and carries the full guidance
   that `AGENTS.md` used to hold. `AGENTS.md` and
   `scripts/agent-harness-block.md` are removed from the source and the core
   payload.
3. Skills move to `.claude/skills/<name>/`. Each `SKILL.md` frontmatter sets
   `disable-model-invocation: true`, which replaces
   `agents/openai.yaml` and its `allow_implicit_invocation: false`. Skills are
   invoked as `/<name>`.
4. The installer protects `CLAUDE.md` and `docs/`. `--refresh-claude-shim`
   appends or refreshes the marked block in an existing `CLAUDE.md`. The
   former `--claude` and `--refresh-agent-shim` options are rejected.
5. The onboarding audit validator reads Claude Code session JSONL: the last
   non-sidechain assistant record with `stop_reason: "end_turn"` is the
   completed answer, and evidence bundles are read from `tool_result` content.

## Consequences

- Fresh installs work in Claude Code with no extra flag: instructions load from
  `CLAUDE.md` and skills appear as slash commands.
- Other agents, including Codex, are not supported. A consumer that needs one
  must maintain its own entrypoint outside Harness.
- Existing installations of an earlier core are not migrated automatically.
  `harness update --candidate` with this core removes unmodified `AGENTS.md`
  and `.agents/skills/` files as managed deletions (empty `.agents/`
  directories remain) and creates `CLAUDE.md` and `.claude/skills/`. A locally
  edited `AGENTS.md` and a pre-existing `CLAUDE.md` stop the update as
  structural conflicts. The consumer moves both files aside, reruns the update,
  then copies its own text back into `CLAUDE.md` outside the Harness markers.
  The updater does not choose for them.
- `harness update` without `--candidate` and the remote bootstraps still fetch
  releases from the upstream `hoangnb24/repository-harness` repository, which
  ships the `AGENTS.md` core. Until this fork publishes its own releases and
  points those URLs at them, only local-source installs and
  `update --candidate` deliver this decision.
- Decisions 0026 and 0029 remain valid for skill explicitness, receipts, and
  budgets; their references to `.agents/skills/` and `agents/openai.yaml` are
  historical.

## Revisit When

A real consumer needs a second agent host, or Claude Code changes how it
discovers project instructions or skills.
