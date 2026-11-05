#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
claude_block="$root/scripts/claude-harness-block.md"
workflow="$root/docs/WORKFLOW.md"

extract_block() {
  awk '
    /<!-- HARNESS:BEGIN -->/ { in_block = 1 }
    in_block { print }
    /<!-- HARNESS:END -->/ { exit }
  ' "$1"
}

cmp -s <(extract_block "$root/CLAUDE.md") "$claude_block"

# Claude Code is the only supported host: no AGENTS.md entrypoint, no
# .agents/ skill tree, and no Codex skill metadata anywhere in the payload.
[[ ! -e "$root/AGENTS.md" && ! -e "$root/.agents" ]]
[[ ! -e "$root/scripts/agent-harness-block.md" ]]
! grep -Fq 'AGENTS.md' "$claude_block"
! grep -Eq '(^|/)AGENTS\.md$|^\.agents/|agents/openai\.yaml$' \
  "$root/scripts/harness-install-files.txt" "$root/scripts/engineering-wisdom-install-files.txt"

required_entry_text=(
  'Start with the requested outcome'
  'Inspection does not authorize edits or operation.'
  'missing capabilities'
  'configurable defaults are not authority'
  "repository's native safety and validation"
  'Only invoke a Harness skill when the user explicitly requests it.'
  'Consumer repositories own their task workflow and planning.'
)
for text in "${required_entry_text[@]}"; do
  grep -Fq "$text" "$claude_block"
done
! grep -Fq 'docs/plans/active/' "$claude_block"
! grep -Fq 'docs/patterns/encoding-invariants.md' "$claude_block"

# Entry size and word limits live in scripts/context-budget.tsv.
"$root/scripts/check-context-budget.sh" >/dev/null

required_workflow_text=(
  '## Authority And Scope'
  'stop and request the smallest'
  '## Missing Capabilities'
  'Keep unobserved state Unknown.'
  '## Trust Through Evidence'
  'structural checks from semantic or runtime proof'
  'Only invoke a Harness skill when the user explicitly requests it.'
)
for text in "${required_workflow_text[@]}"; do
  grep -Fq "$text" "$workflow"
done
! grep -Fq 'Select The Work Shape' "$workflow"
! grep -Fq '### Bounded Change' "$workflow"

! grep -Fq 'query matrix' "$claude_block"

payloads=(
  CLAUDE.md
  .claude/skills/audit-onboarding-proposal/SKILL.md
  .claude/skills/audit-onboarding-proposal/scripts/validate_evidence_capsule.py
  .claude/skills/encode-invariant/SKILL.md
  .claude/skills/improve-harness/SKILL.md
  .claude/skills/onboard-repository/SKILL.md
  .claude/skills/onboard-repository/references/evidence-capsule-v1.md
  .claude/skills/onboard-repository/references/evidence-capsule-v2.md
  .claude/skills/onboard-repository/references/boundary-edges.md
  .claude/skills/onboard-repository/references/untracked-install.md
  .claude/skills/onboard-repository/scripts/emit_evidence_bundle.py
  .claude/skills/onboard-repository/scripts/render_patch.py
  docs/WORKFLOW.md
  docs/README.md
  docs/patterns/encoding-invariants.md
  docs/patterns/multi-repository-evidence.md
  docs/product/README.md
  docs/plans/README.md
  docs/plans/active/README.md
  docs/plans/completed/README.md
  docs/decisions/README.md
  docs/templates/application-runbook.md
  docs/templates/decision.md
  docs/templates/exec-plan.md
  docs/templates/harness-improvement.md
  docs/templates/session-handoff.md
  docs/templates/evaluator-rubric.md
  docs/templates/interface-contract.md
)
for payload in "${payloads[@]}"; do
  grep -Fxq "$payload" "$root/scripts/harness-install-files.txt"
done

# Source/configuration contract only: host activation and agent compliance
# require separate runtime evidence. Cover every skill declared by either profile.
skill_count=0
while IFS= read -r skill; do
  case "$skill" in
    .claude/skills/*/SKILL.md)
      frontmatter=$(awk '/^---$/ { n++; next } n == 1 { print } n >= 2 { exit }' "$root/$skill")
      [[ "$(grep -c '^disable-model-invocation:' <<<"$frontmatter")" == 1 ]]
      grep -Fxq 'disable-model-invocation: true' <<<"$frontmatter"
      grep -Eq '^description: Use only when the user explicitly invokes `/[a-z-]+`' <<<"$frontmatter"
      skill_count=$((skill_count + 1))
      ;;
  esac
done < <(cat "$root/scripts/harness-install-files.txt" "$root/scripts/engineering-wisdom-install-files.txt")
[[ "$skill_count" == 5 ]]

invariant_skill="$root/.claude/skills/encode-invariant/SKILL.md"
grep -Fq 'Do not infer policy from conventions, code patterns, tests, defaults, or undocumented preferences.' \
  "$invariant_skill"
required_invariant_method=(
  "Reuse the repository's existing test, build, task, lint, scan, or validation"
  'Choose the lowest deterministic layer that sees the complete accepted'
  'name the violating item, the broken rule, the'
  'authority source, and a concrete compliant next action'
  'local validation command and observed result'
  'optional hook availability, if any'
  'CI invocation discovered or absent'
  'branch-protection enforcement verified or unverified'
)
for text in "${required_invariant_method[@]}"; do
  grep -Fq "$text" "$invariant_skill"
done

onboarding_skill="$root/.claude/skills/onboard-repository/SKILL.md"
grep -Fq 'Compare documented invariants with executable checks' "$onboarding_skill"
grep -Fq '**Unenforced rule:**' "$onboarding_skill"
grep -Fq '**Check lacking authority:**' "$onboarding_skill"
grep -Fq 'Do not add, edit, delete, enable, or execute a guard during onboarding.' \
  "$onboarding_skill"

grep -Fq 'read_source_text "scripts/claude-harness-block.md"' "$root/scripts/install-harness.sh"
grep -Fq 'REFRESH_CLAUDE_SHIM=1' "$root/scripts/install-harness.sh"
! grep -Eq 'AGENTS\.md|agent-harness-block|REFRESH_AGENT_SHIM|INSTALL_CLAUDE_SHIM' "$root/scripts/install-harness.sh"
grep -Fq 'ENGINEERING_WISDOM_PAYLOAD_MANIFEST="scripts/engineering-wisdom-install-files.txt"' "$root/scripts/install-harness.sh"
! grep -Fq 'CLI_PAYLOAD_MANIFEST' "$root/scripts/install-harness.sh"

# Linux and macOS only: no PowerShell bootstrap or Windows test remains.
[[ -z "$(find "$root/scripts" "$root/tests" -name '*.ps1' -print -quit)" ]]

echo "Claude Code authority, explicit skill configuration, canonical shim, and Linux/macOS-only installer passed"
