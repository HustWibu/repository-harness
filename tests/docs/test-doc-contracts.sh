#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

fail() {
  printf 'documentation contract failed: %s\n' "$*" >&2
  exit 1
}

require() {
  local file=$1
  local text=$2
  grep -Fq -- "$text" "$root/$file" || fail "$file omits: $text"
}

current_files=(
  README.md
  CLAUDE.md
  docs/WORKFLOW.md
  docs/ARCHITECTURE.md
  docs/HARNESS.md
  docs/README.md
  docs/patterns/encoding-invariants.md
  docs/product/README.md
  docs/product/installation-profiles.md
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
  docs/decisions/0019-repository-centered-default-workflow.md
  docs/decisions/0020-installation-profile-and-knowledge-boundaries.md
  docs/decisions/0024-rust-harness-core-maintenance-cli.md
  docs/decisions/0025-latest-release-self-update-and-human-directed-conflicts.md
  docs/decisions/0026-explicit-onboarding-skills-in-default-core.md
  docs/decisions/0027-end-protocol-v1-and-focus-repository-protocol.md
  docs/decisions/0028-authoritative-invariant-encoding.md
  docs/decisions/0029-measured-skills-context-budget-and-approval-frontmatter.md
  docs/decisions/0030-evidence-first-completion-and-handoff.md
  docs/decisions/0031-claude-code-only-host.md
  docs/decisions/0032-linux-and-macos-only.md
  docs/decisions/0035-multi-repository-knowledge-surfaces.md
  docs/research/multi-repo-requirements.md
  docs/research/application-legibility.md
  docs/research/harness-gap-analysis.md
  docs/research/harness-requirements-ideas.md
  .github/ISSUE_TEMPLATE/real-world-example.md
)
for file in "${current_files[@]}"; do
  [[ -f "$root/$file" ]] || fail "missing current artifact: $file"
done

require CLAUDE.md 'Start with the requested outcome'
require CLAUDE.md 'configurable defaults are not authority'
require CLAUDE.md 'Only invoke a Harness skill when the user explicitly requests it.'
require docs/WORKFLOW.md '## Authority And Scope'
require docs/WORKFLOW.md '## Missing Capabilities'
require docs/WORKFLOW.md '## Trust Through Evidence'
require docs/WORKFLOW.md '## Optional Resources'
require docs/WORKFLOW.md '## Definition Of Done'
require docs/WORKFLOW.md 'Missing evidence means not verified'
require docs/patterns/encoding-invariants.md '## 1. Establish Authority'
require docs/patterns/encoding-invariants.md '## 4. Prove Both Directions'
require docs/patterns/encoding-invariants.md '## 6. Fail Closed On Missing Evidence'
require docs/patterns/encoding-invariants.md '## 7. Promote Guidance To A Gate Deliberately'
require docs/templates/decision.md 'approved_by:'
require docs/patterns/encoding-invariants.md '## 5. Discover And Report Enforcement'
require docs/patterns/encoding-invariants.md '| Scope | Files, modules, configuration, or runtime objects covered |'
require docs/patterns/encoding-invariants.md 'Find the repository'
require docs/patterns/encoding-invariants.md '| Diagnostic | Violating item, broken rule, authority pointer, and next action |'
require docs/patterns/encoding-invariants.md '| Retirement | Condition under which the check should be removed or revisited |'
require docs/patterns/encoding-invariants.md '**Positive proof:**'
require docs/patterns/encoding-invariants.md '**Negative proof:**'
require docs/patterns/encoding-invariants.md '| Local validation |'
require docs/patterns/encoding-invariants.md '| Optional hook |'
require docs/patterns/encoding-invariants.md '| CI |'
require docs/patterns/encoding-invariants.md '| Branch protection |'
require docs/decisions/0028-authoritative-invariant-encoding.md 'Only explicit user invocation activates it.'
require docs/ARCHITECTURE.md 'one Rust binary'
require README.md '## What We Prove'
require README.md '## Protocol V1 End Of Life'
require docs/research/application-legibility.md 'research, not a release gate'
require docs/decisions/0027-end-protocol-v1-and-focus-repository-protocol.md '`harness-cli-v0.1.22`'
require .github/ISSUE_TEMPLATE/real-world-example.md '`docs/WORKFLOW.md`'
require .github/ISSUE_TEMPLATE/real-world-example.md '`docs/ARCHITECTURE.md`'

for heading in Outcome Context Scope Approach 'Risks And Recovery' Progress Decisions Validation Result; do
  require docs/templates/exec-plan.md "## $heading"
done
for heading in Goal Verified 'Changed But Not Verified' 'Known Broken Or Open' 'Next Step' 'Clean-State Check'; do
  require docs/templates/session-handoff.md "## $heading"
done
for heading in 'Work Reviewed' Scores Verdict 'Required Changes'; do
  require docs/templates/evaluator-rubric.md "## $heading"
done

for heading in 'Related Repositories' 'Authority By Truth Type'; do
  require docs/README.md "## $heading"
done
require docs/README.md 'never an absolute path from one'
require docs/WORKFLOW.md 'evidence scope: in-repository, across a contract'
require docs/WORKFLOW.md 'list its consumers under not verified'
require docs/WORKFLOW.md 'Configuration checked into a repository may describe only a development'
require docs/templates/application-runbook.md '## Environments'
require docs/templates/application-runbook.md 'a missing key is a finding to report'
require docs/templates/evaluator-rubric.md '| Evidence scope |'
require docs/templates/exec-plan.md '## Repositories Touched'
require docs/templates/exec-plan.md 'Other repositories link here and do not copy'
require docs/templates/session-handoff.md '## Repositories Touched'
require docs/templates/interface-contract.md 'record the date each row was checked'

# Payload docs are copied to every consumer; a path from one machine is wrong
# everywhere else.
machine_path='(/Users/|/home/[A-Za-z]|~/)'
[[ -n "$(printf 'see /home/alice/orders\n' | grep -E "$machine_path")" ]] ||
  fail 'machine-path guard does not detect a home path'
while IFS= read -r payload; do
  [[ -f "$root/$payload" ]] || fail "core manifest target is missing: $payload"
  if [[ "$payload" == docs/*.md ]] && grep -nE "$machine_path" "$root/$payload" >&2; then
    fail "$payload contains a machine-specific absolute path; use a URL or relative path"
  fi
done < <(sed -e '/^[[:space:]]*#/d' -e '/^[[:space:]]*$/d' "$root/scripts/harness-install-files.txt")

compatibility_paths=(
  crates/harness-cli
  scripts/schema
  scripts/harness-cli-install-files.txt
  .github/workflows/harness-cli-release.yml
  docs/contracts/harness-orchestration-v1.md
  docs/compatibility
  docs/stories
  .harness/core-state
  .harness/changesets
)
for compatibility_path in "${compatibility_paths[@]}"; do
  target="$root/$compatibility_path"
  if [[ -d "$target" ]]; then
    [[ -z "$(find "$target" -type f -print -quit)" ]] ||
      fail "EOL compatibility files remain: $compatibility_path"
  else
    [[ ! -e "$target" ]] || fail "EOL compatibility path remains: $compatibility_path"
  fi
done

executables=(
  scripts/validate-premerge.sh
  tests/workflow/test-repository-workflow.sh
  tests/installer/test-install-harness-modes.sh
)
for executable in "${executables[@]}"; do
  [[ -x "$root/$executable" ]] || fail "documented gate is not executable: $executable"
done

required_gates=(
  'cargo fmt --all -- --check'
  'cargo test --workspace --locked'
  'cargo clippy --workspace --all-targets --locked -- -D warnings'
  'tests/installer/test-install-harness-modes.sh'
  'tests/docs/test-doc-contracts.sh'
  'tests/workflow/test-repository-workflow.sh'
  'tests/release/test-harness-release-workflow-contract.sh'
)
for gate in "${required_gates[@]}"; do
  require scripts/validate-premerge.sh "$gate"
done

require .github/workflows/premerge.yml 'run: scripts/validate-premerge.sh'
require .github/workflows/harness-release.yml 'run: scripts/validate-premerge.sh'

# Every distributed skill ends with a machine-countable receipt line.
while IFS= read -r skill; do
  name=$(basename "$(dirname "$skill")")
  grep -Fq -- "HARNESS_SKILL: $name | done|stopped|skipped |" "$root/$skill" ||
    fail "$skill omits its HARNESS_SKILL receipt instruction"
done < <(cat "$root/scripts/harness-install-files.txt" \
  "$root/scripts/engineering-wisdom-install-files.txt" |
  grep -E '^\.claude/skills/[^/]+/SKILL\.md$' | LC_ALL=C sort -u)
require .claude/skills/encode-invariant/SKILL.md 'Missing frontmatter means not approved'

"$root/tests/installer/assert-claude-authority-contract.sh" >/dev/null
"$root/tests/installer/assert-install-manifest-links.sh" >/dev/null

echo "current product, EOL boundary, manifest, authority, and validation references passed"
