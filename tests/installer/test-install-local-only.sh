#!/usr/bin/env bash
# Local-only installs stay out of git status (MR-LOCAL-1..5).
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
installer="$root/scripts/install-harness.sh"
temp=$(mktemp -d)
trap 'rm -rf "$temp"' EXIT

cargo build --quiet --manifest-path "$root/Cargo.toml" -p harness --locked
install() {
  HARNESS_CORE_BINARY="$root/target/debug/harness" "$installer" --yes "$@"
}
fail() {
  printf 'local-only installer test failed: %s\n' "$*" >&2
  exit 1
}
new_repo() {
  git init -q "$1"
  git -C "$1" -c user.email=t@example.invalid -c user.name=t commit -q --allow-empty -m init
}
assert_clean() {
  [[ -z "$(git -C "$1" status --porcelain)" ]] ||
    fail "$2: git status is not clean: $(git -C "$1" status --porcelain | head -3)"
}

# 1. Clean repository: everything installed, nothing visible to Git.
clean="$temp/clean"
new_repo "$clean"
install --local-only "$clean" >"$temp/clean.out"
[[ -f "$clean/docs/WORKFLOW.md" && -x "$clean/scripts/bin/harness" ]] || fail 'core not installed'
[[ ! -e "$clean/.gitignore" ]] || fail 'local-only install wrote .gitignore'
assert_clean "$clean" 'fresh local-only install'
"$clean/scripts/bin/harness" doctor --directory "$clean" | grep -Fq 'pass local_only' ||
  fail 'doctor does not recognize the local-only install'

# 3. Update keeps the block current: drop one line, rerun, status is clean again.
exclude="$clean/.git/info/exclude"
grep -Fxv '/docs/WORKFLOW.md' "$exclude" >"$temp/exclude" && cat "$temp/exclude" >"$exclude"
if "$clean/scripts/bin/harness" doctor --directory "$clean" >"$temp/doctor.out"; then
  fail 'doctor passed with a payload path missing from the exclude block'
fi
grep -Fq 'fail local_only' "$temp/doctor.out" || fail 'doctor did not name the local_only check'
"$clean/scripts/bin/harness" update --candidate --directory "$clean" >/dev/null
assert_clean "$clean" 'harness update on a local-only install'
install "$clean" >"$temp/rerun.out"
grep -Fq 'Local-only:' "$temp/rerun.out" || fail 'rerun without the flag did not detect local-only'
assert_clean "$clean" 'installer rerun on a local-only install'

# 4. Removal deletes only Harness lines.
tracked="$temp/tracked"
new_repo "$tracked"
printf 'user-before\n' >"$tracked/.git/info/exclude"
printf '# team instructions\n' >"$tracked/CLAUDE.md"
git -C "$tracked" add CLAUDE.md
git -C "$tracked" -c user.email=t@example.invalid -c user.name=t commit -q -m claude

# 2. A tracked CLAUDE.md is skipped and reported; refreshing the shim shows exactly one change.
install --local-only "$tracked" >"$temp/tracked.out"
grep -Fq 'skip     CLAUDE.md (tracked' "$temp/tracked.out" || fail 'tracked CLAUDE.md not reported'
assert_clean "$tracked" 'local-only install with tracked CLAUDE.md'
printf 'user-after\n' >>"$tracked/.git/info/exclude"
install --refresh-claude-shim "$tracked" >"$temp/refresh.out"
grep -Fq 'warning  CLAUDE.md is tracked' "$temp/refresh.out" || fail 'tracked shim refresh did not warn'
[[ "$(git -C "$tracked" status --porcelain)" == ' M CLAUDE.md' ]] ||
  fail "shim refresh changed more than CLAUDE.md: $(git -C "$tracked" status --porcelain)"
git -C "$tracked" checkout -q CLAUDE.md
install --remove-local-only "$tracked" >/dev/null
grep -Fq 'harness local-only' "$tracked/.git/info/exclude" && fail 'removal left Harness lines'
[[ "$(cat "$tracked/.git/info/exclude")" == $'user-before\nuser-after' ]] ||
  fail "removal changed user exclude lines: $(cat "$tracked/.git/info/exclude")"
git -C "$tracked" status --porcelain | grep -Fq '?? docs/' || fail 'removal did not make installed files visible again'

# 5. Not a Git repository root: clear error, nothing installed.
plain="$temp/plain"
mkdir "$plain"
if install --local-only "$plain" >"$temp/plain.out" 2>&1; then
  fail 'local-only install accepted a directory outside Git'
fi
grep -Fq 'requires a Git repository root' "$temp/plain.out" || fail 'non-Git error is unclear'
[[ -z "$(ls -A "$plain")" ]] || fail 'non-Git target received files'
if install --local-only "$clean/docs" >"$temp/subdir.out" 2>&1; then
  fail 'local-only install accepted a repository subdirectory'
fi
if install --local-only --override "$clean" >"$temp/override.out" 2>&1; then
  fail 'local-only install accepted --override'
fi

echo "local-only install, update, doctor, tracked-file, removal, and non-Git modes passed"
