---
status: accepted
approved_by: "User request: support both Claude Code and Codex"
approved_at: "2026-11-07"
superseded_by: ""
---

# 0041 Claude Code And Codex

Supersedes decision 0031. The user needs this harness on both hosts.

Both entrypoints carry the shared protocol in `scripts/claude-harness-block.md`.
Every core installs `CLAUDE.md`, `AGENTS.md`, and native skill trees for both
hosts. Existing instructions are preserved in merge mode; each has its own
explicit refresh flag. Updates use the existing managed-file three-way merge,
including structural conflicts for pre-existing Codex files.

Keep `.claude/skills/` as the authoring source and generate `.agents/skills/`
with `scripts/sync-codex-skills.py`. Generated files are regular managed files,
not symlinks. Translate instruction paths and invocation syntax, and set
Codex `policy.allow_implicit_invocation: false`. Pre-merge checks reject drift.
This preserves existing Claude installation paths and avoids independently
maintained workflows.

Hub sync checks and writes both entrypoints, preserving surrounding text.
Workspace mode also writes member `AGENTS.md` pointers: Codex
[instruction discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
starts at each Git root, so parent workspace instructions alone do not cover it.
Onboarding audit accepts Claude Code sessions and Codex response-item sessions
with an explicit final-answer phase. Token accounting in `skill-report`
remains Claude Code only; unsupported formats are rejected.

Metadata and installation checks prove packaging, not live agent compliance.

Codex discovery and invocation policy follow the
[official skill documentation](https://learn.chatgpt.com/docs/build-skills).
