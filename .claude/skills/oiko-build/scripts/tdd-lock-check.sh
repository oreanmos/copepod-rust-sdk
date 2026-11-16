#!/usr/bin/env bash
# TDD lock check: the implementer must not change the tests of the red commit.
#
#   tdd-lock-check.sh <worktree> <red-commit> [<head>]
#
# Lists the files the red commit (oiko-test-author) touched. Pure test files
# (tests/, e2e/, *.spec.ts, *_test.rs, tests.rs, test_*.rs, *_tests.rs) must be
# byte-identical at <head> (default HEAD): any change fails the check. Mixed
# files (a stub in src beside inline tests) are printed as a diff for the
# reviewer to judge: replacing a stub body is allowed, changing an assertion
# is not. Exit 0 locked, 1 a pure test file changed, 2 usage.
set -euo pipefail

[[ $# -ge 2 ]] || { sed -n '2,12p' "$0"; exit 2; }
wt="$1"; red="$2"; head="${3:-HEAD}"
git -C "$wt" rev-parse -q --verify "$red^{commit}" >/dev/null || { echo "no such commit: $red" >&2; exit 2; }

is_test_path() {
    [[ "$1" =~ (^|/)(tests|e2e|e2e-desktop)/ || "$1" =~ \.spec\.ts$ || "$1" =~ (_test|_tests)\.rs$ \
        || "$1" =~ (^|/)(tests|test_[^/]*)\.rs$ ]]
}

status=0
mixed=()
echo "red commit: $(git -C "$wt" log -1 --format='%h %s' "$red")"
while read -r f; do
    [[ -n "$f" ]] || continue
    if is_test_path "$f"; then
        if git -C "$wt" diff --quiet "$red" "$head" -- "$f"; then
            echo "locked   $f"
        else
            echo "CHANGED  $f"; status=1
        fi
    else
        mixed+=("$f")
    fi
done < <(git -C "$wt" diff-tree --no-commit-id --name-only -r "$red")

for f in "${mixed[@]}"; do
    if git -C "$wt" diff --quiet "$red" "$head" -- "$f"; then
        echo "mixed    $f (unchanged)"
    else
        echo "mixed    $f (changed since red; review below: stub bodies may change, assertions may not)"
        git -C "$wt" diff "$red" "$head" -- "$f"
    fi
done
[[ $status -eq 0 ]] && echo "TDD-LOCK: ok" || echo "TDD-LOCK: FAILED (a test file of the red commit changed)"
exit "$status"
