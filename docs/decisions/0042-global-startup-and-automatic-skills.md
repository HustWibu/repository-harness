---
status: accepted
approved_by: "User request: install once and automatically use Harness on both hosts"
approved_at: "2026-11-07"
superseded_by: ""
---

# 0042 Global Startup And Automatic Skills

The user needs Harness in every session without installing it per repository
or naming a skill. This supersedes the explicit-only activation policy in
0026, 0028, 0029, and 0041; their authority and evidence requirements remain.

Install one normal maintained core into the user's data directory. Register
native user instruction blocks for Claude Code and Codex, and skill links to
that core. Preserve surrounding user text, honor custom host homes and Codex's
nonempty override file, reject skill collisions and symlinked instructions,
back up changed instructions, and roll back failed global activation. Global
removal unregisters only owned links and blocks, retaining core and backups.

A compact protocol is always loaded through native global instructions. Skill
descriptions carry task triggers; Claude allows model invocation and Codex
sets `allow_implicit_invocation: true`. Skills are selected only when relevant.
Selection does not authorize additional edits, operations, or publishing.
Consumer authority, explicit write gates, and evidence contracts still apply.

Bundled helper paths resolve from a discovered skill location, so skills work
outside repositories containing their own installation. Missing repository
maps fall back to native inspection; missing Harness reference material comes
from the global core. No automatic project-file generation or agent launcher
is required. Re-run the global installer to update and refresh registration.

The optional engineering-wisdom payload still requires installation opt-in.
The existing repository installation remains supported. Instructions steer
agents; repository-native checks and CI enforce deterministic rules.

Discovery follows [Claude memory](https://code.claude.com/docs/en/memory),
[Claude skills](https://code.claude.com/docs/en/skills),
[Codex instructions](https://learn.chatgpt.com/docs/agent-configuration/agents-md),
and [Codex skills](https://learn.chatgpt.com/docs/build-skills).
