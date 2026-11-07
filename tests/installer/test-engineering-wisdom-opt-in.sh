#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
installer="$root/scripts/install-harness.sh"
manifest="$root/scripts/engineering-wisdom-install-files.txt"
temp=$(mktemp -d)
trap 'rm -rf "$temp"' EXIT

cargo build --quiet --manifest-path "$root/Cargo.toml" -p harness --locked
core_binary="$root/target/debug/harness"

install() {
  HARNESS_CORE_BINARY="$core_binary" "$installer" "$@"
}

# Cause: no advisory flag. Effect: the neutral core contains no wisdom pack and
# the optional manifest remains outside the core declaration.
default_target="$temp/default"
install --directory "$default_target" --yes >"$temp/default.out"
grep -Fq 'Engineering wisdom: excluded' "$temp/default.out"
[[ ! -e "$default_target/.claude/skills/engineering-wisdom" ]]
! grep -Fq '.claude/skills/engineering-wisdom/' \
  "$root/scripts/harness-install-files.txt"

# Cause: explicit advisory flag. Effect: every optional-manifest member is
# installed, and relevant requests can select it automatically.
opt_in_target="$temp/opt-in"
install --directory "$opt_in_target" --with-engineering-wisdom --yes \
  >"$temp/opt-in.out"
grep -Fq 'Engineering wisdom: included (explicit opt-in)' "$temp/opt-in.out"
while IFS= read -r relative || [[ -n "$relative" ]]; do
  relative="${relative%$'\r'}"
  case "$relative" in
    ""|\#*) continue ;;
  esac
  [[ -f "$opt_in_target/$relative" ]]
done <"$manifest"
! grep -Fxq 'disable-model-invocation: true' \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md"
grep -Fq '**Observation:**' \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md"
grep -Fq '**Trade-off:**' \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md"
grep -Fq '**Proposed repository-owned enforcement:**' \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md"
grep -Fq 'Do not rewrite application architecture' \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md"
grep -Fq 'Exercise composition roots and shipped artifacts' \
  "$opt_in_target/.claude/skills/engineering-wisdom/references/heuristics.md"
grep -Fq 'Decode, validate, and recover at input boundaries' \
  "$opt_in_target/.claude/skills/engineering-wisdom/references/heuristics.md"
grep -Fq 'Preserve semantics across adapters' \
  "$opt_in_target/.claude/skills/engineering-wisdom/references/heuristics.md"
grep -Fq 'Make automation failure honest' \
  "$opt_in_target/.claude/skills/engineering-wisdom/references/heuristics.md"
grep -Fq 'Bound cumulative state at its consumption boundary' \
  "$opt_in_target/.claude/skills/engineering-wisdom/references/heuristics.md"

# A later normal merge is non-activation, not removal.
before=$(shasum -a 256 \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md" | awk '{print $1}')
install --directory "$opt_in_target" --merge --yes >"$temp/reinstall.out"
after=$(shasum -a 256 \
  "$opt_in_target/.claude/skills/engineering-wisdom/SKILL.md" | awk '{print $1}')
[[ "$before" == "$after" ]]
grep -Fq 'Engineering wisdom: excluded' "$temp/reinstall.out"

echo "engineering-wisdom default exclusion and explicit opt-in passed"
