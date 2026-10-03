#!/usr/bin/env bash
# Keep the shared Oiko agent kit identical in oikonotes, copepod and
# copepod-rust-sdk. Run it from the repo whose copy you just edited.
#
#   sync-agent-kit.sh          copy this repo's kit into the two sibling repos
#   sync-agent-kit.sh --check  report differences only; exit 1 if any
#
# Per-repo files (AGENTS.md, CLAUDE.md, .gitignore) are not part of the kit.
set -euo pipefail

SKILLS=(oiko-plan oiko-build oiko-implement oiko-review oiko-worktree oiko-contract
        oikonotes-dev copepod-dev copepod-sdk-dev)
AGENTS=(oiko-implementer oiko-reviewer oiko-scout)
FILES=(.claude/settings.json .codex/config.toml .pi/skill-orchestrator.json)
REPOS=(oikonotes copepod copepod-rust-sdk)

mode="${1:-sync}"
here="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
# Resolve siblings from the main checkout, so this also works from a worktree.
main_checkout="$(git -C "$here" worktree list --porcelain | sed -n '1s/^worktree //p')"
parent="$(dirname "$main_checkout")"
self="$(basename "$main_checkout")"

kit_paths() {
    for s in "${SKILLS[@]}"; do echo ".claude/skills/$s"; done
    for a in "${AGENTS[@]}"; do
        echo ".claude/agents/$a.md"; echo ".codex/agents/$a.toml"; echo ".opencode/agents/$a.md"
    done
    printf '%s\n' "${FILES[@]}"
}

status=0
for repo in "${REPOS[@]}"; do
    [[ "$repo" == "$self" ]] && continue
    dest="$parent/$repo"
    if [[ ! -d "$dest/.git" && ! -f "$dest/.git" ]]; then
        echo "skip: $dest is not a git checkout" >&2; continue
    fi
    while read -r p; do
        src="$here/$p"
        [[ -e "$src" ]] || { echo "missing in source: $p" >&2; status=1; continue; }
        if [[ "$mode" == "--check" ]]; then
            if ! diff -rq "$src" "$dest/$p" >/dev/null 2>&1; then
                echo "differs: $repo/$p"; status=1
            fi
        else
            mkdir -p "$(dirname "$dest/$p")"
            if [[ -d "$src" ]]; then
                rm -rf "${dest:?}/$p"; cp -a "$src" "$dest/$p"
            else
                cp -a "$src" "$dest/$p"
            fi
        fi
    done < <(kit_paths)
    if [[ "$mode" != "--check" ]]; then
        mkdir -p "$dest/.agents/skills"
        for s in "${SKILLS[@]}"; do ln -sfn "../../.claude/skills/$s" "$dest/.agents/skills/$s"; done
        echo "synced: $repo"
    else
        for s in "${SKILLS[@]}"; do
            [[ "$(readlink "$dest/.agents/skills/$s" 2>/dev/null)" == "../../.claude/skills/$s" ]] \
                || { echo "missing link: $repo/.agents/skills/$s"; status=1; }
        done
    fi
done
exit "$status"
