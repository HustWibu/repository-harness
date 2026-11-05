#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
pin="$root/.claude/skills/survey-system/scripts/pin_sources.py"
harness="$root/target/debug/harness"
fail() { echo "survey-system test failed: $*" >&2; exit 1; }

python3 -B "$pin" --self-test >/dev/null || fail "self-test"
[[ -x "$harness" ]] || cargo build --quiet --manifest-path "$root/Cargo.toml"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git_quiet() { git -c user.name=t -c user.email=t@t "$@" >/dev/null; }
mkdir -p "$work/hub/docs/services/orders" "$work/orders/src"
git_quiet -C "$work/hub" init
git_quiet -C "$work/orders" init
printf 'class App {\n  send();\n}\n' > "$work/orders/src/App.java"
git_quiet -C "$work/orders" add -A
git_quiet -C "$work/orders" commit -m c
printf '# Map\n\n## Related Repositories\n\n| Repository | Role | Location |\n| --- | --- | --- |\n| orders | service | ../orders |\n' \
  > "$work/hub/docs/README.md"

marker=$(printf 'orders src/App.java 2 2\n' | python3 -B "$pin" --hub "$work/hub") || fail "pin"
printf '# orders\n\nOrders sends on start.\n%s\n' "$marker" > "$work/hub/docs/services/orders/overview.md"
"$harness" hub check-fresh --directory "$work/hub" | grep -q '^  current docs/services/orders/overview.md:4 ' ||
  fail "a freshly pinned source is not current"

printf 'class App {\n  sendAsync();\n}\n' > "$work/orders/src/App.java"
git_quiet -C "$work/orders" commit -am c
if "$harness" hub check-fresh --directory "$work/hub" >/dev/null; then
  fail "a changed source passed check-fresh"
fi

printf 'payments src/App.java 1 1\n' | python3 -B "$pin" --hub "$work/hub" 2>/dev/null &&
  fail "an unlisted repository was pinned"
printf 'orders src/App.java 9 9\n' | python3 -B "$pin" --hub "$work/hub" 2>/dev/null &&
  fail "a range past the end of the file was pinned"

echo "survey-system pin and freshness contract passed"
