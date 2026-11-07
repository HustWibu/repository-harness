# Harness Product Model

Harness helps agents understand a repository, identify missing capabilities,
and build trust through executable or observable evidence.

## Principles

- Code, types, interfaces, commands, tests, examples, and runtime signals hold
  implementation knowledge. Documents retain product intent and rationale.
- Load the smallest useful context and link to existing sources.
- Missing capabilities and material policy choices stay explicit; an agent must
  not substitute assumptions for authority or evidence.
- Consumer repositories own their workflows, planning, application operation,
  architecture, and validation commands.
- Harness selects relevant skills automatically. Templates are optional.
- The maintenance binary preserves provenance and safely updates only its core.

## Installed Core

Core supplies compact `CLAUDE.md` and `AGENTS.md` entrypoints and repository map, context
about authority, capabilities and evidence, optional templates, and
task-triggered skills under `.claude/skills/` and `.agents/skills/` for
Claude Code and Codex (decision 0041).
It supplies no fabricated product domains, application commands, credentials,
task scheduler, database, or workflow engine.

## Evidence

Release claims cover fresh installation, repository navigation and authority,
and safe update/conflict/recovery. Installed guidance and skill metadata do not
prove an agent followed them. Consumer runtime behavior needs consumer-owned
proof.

## End-Of-Life Boundary

The SQLite control plane and protocol v1 remain historical products, available
through immutable releases and Git history. Existing consumer files are not
automatically removed by installation or updates.

Global installation (decision 0042) loads the shared protocol for every new
Claude Code and Codex session. It preserves user instructions, registers native
skill directories, and lets repository instructions retain product authority.
