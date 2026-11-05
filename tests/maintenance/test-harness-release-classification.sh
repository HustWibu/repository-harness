#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
classifier="$root/scripts/harness-release-changed.sh"
workflow="$root/.github/workflows/post-merge-maintenance.yml"

positive_paths=(
  crates/harness/src/main.rs
  crates/harness/assets/docs/plans/README.md
  crates/harness/assets/docs/README.md
  .claude/skills/onboard-repository/SKILL.md
  .claude/skills/audit-onboarding-proposal/scripts/validate_evidence_capsule.py
  .claude/skills/encode-invariant/SKILL.md
  .claude/skills/improve-harness/SKILL.md
  docs/WORKFLOW.md
  docs/patterns/encoding-invariants.md
  docs/patterns/multi-repository-evidence.md
  docs/templates/application-runbook.md
  docs/templates/harness-improvement.md
  docs/templates/session-handoff.md
  docs/templates/evaluator-rubric.md
  docs/templates/interface-contract.md
  scripts/claude-harness-block.md
  scripts/harness-install-files.txt
  scripts/install-harness.sh
  scripts/build-harness-release.sh
  scripts/harness-release-changed.sh
  scripts/promote-harness-release-tag.sh
  scripts/verify-harness-release-assets.sh
  scripts/verify-harness-release-identity.sh
  .github/workflows/harness-release.yml
  .github/workflows/post-merge-maintenance.yml
  Cargo.toml
  Cargo.lock
)
for path in "${positive_paths[@]}"; do
  if ! printf '%s\n' "$path" | "$classifier"; then
    echo "Harness core change was not classified for release: $path" >&2
    exit 1
  fi
done

for unrelated in docs/HARNESS.md README.md docs/research/application-legibility.md; do
  if printf '%s\n' "$unrelated" | "$classifier"; then
    echo "unrelated path triggered Harness core publication: $unrelated" >&2
    exit 1
  fi
done

grep -Fq 'scripts/harness-release-changed.sh <<<"$changed_files"' "$workflow"
echo "Harness core post-merge release classification tests passed"
