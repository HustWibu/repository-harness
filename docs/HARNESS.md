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
- All Harness skills require explicit user invocation. Templates are optional.
- The maintenance binary preserves provenance and safely updates only its core.

## Installed Core

Core supplies a compact `CLAUDE.md` entrypoint and repository map, context
about authority, capabilities and evidence, optional templates, and
explicit-only Claude Code skills under `.claude/skills/`. Claude Code is the
only supported agent host (decision 0031).
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
