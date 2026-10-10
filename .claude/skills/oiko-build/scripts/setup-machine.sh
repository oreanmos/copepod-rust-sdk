#!/usr/bin/env bash
# Prepare this machine for the Oiko agent kit. Safe to re-run.
#
#   setup-machine.sh
#
# Expects oikonotes, copepod and copepod-rust-sdk cloned as siblings. It:
#   1. clones (or fast-forwards) the agent state repo into <dev>/agent-state,
#      beside the code repos (moving an old ~/.cache/oiko-agents checkout there
#      and leaving that path as a symlink, which older specs and logs use):
#      build logs, reviews and reports, plus _home/ (Claude memory, Pi config);
#   2. links each repo's Claude memory (also ingrained-2.0's) and the Pi
#      skill-orchestrator into it;
#   3. writes this machine's absolute paths where tools need them, outside
#      tracked files: .claude/settings.local.json in each repo, and a managed
#      block in ~/.codex/config.toml (project trust, sandbox writable roots).
#   4. fast-forwards each repo's main to origin/staging (the other machine's merged
#      work) and lists active claims.
# Set OIKO_STATE_REMOTE to use another remote for the state repo.
set -euo pipefail

STATE_REMOTE="${OIKO_STATE_REMOTE:-https://github.com/oreanmos/agent-state.git}"
LEGACY_STATE="$HOME/.cache/oiko-agents"
REPOS=(oikonotes copepod copepod-rust-sdk)
# Other repos whose agents share the state repo (claims, notes, Claude memory).
SHARED_REPOS=(ingrained-2.0)

here="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
main_checkout="$(git -C "$here" worktree list --porcelain | sed -n '1s/^worktree //p')"
dev="$(dirname "$main_checkout")"
STATE="$dev/agent-state"
stamp="$(date +%Y%m%d%H%M%S)"

say() { printf '==> %s\n' "$*"; }
warn() { printf '!!  %s\n' "$*" >&2; }

# Replace $1 with a symlink to $2, keeping any real content as a backup outside
# the tool's directory (Pi would load a backup left in extensions/).
backups="$HOME/.local/state/oiko-agent-kit/backups/$stamp"
link() {
    local path="$1" target="$2"
    if [[ -L "$path" ]]; then
        [[ "$(readlink "$path")" == "$target" ]] && return 0
        rm "$path"
    elif [[ -e "$path" ]]; then
        local backup="$backups/$(printf '%s' "${path#"$HOME"/}" | tr '/' '_')"
        mkdir -p "$backups"
        mv "$path" "$backup"
        warn "moved existing $path to $backup"
    fi
    mkdir -p "$(dirname "$path")"
    ln -s "$target" "$path"
    say "linked $path -> $target"
}

# 1. State repo.
if [[ ! -e "$STATE" && -d "$LEGACY_STATE/.git" && ! -L "$LEGACY_STATE" ]]; then
    mv "$LEGACY_STATE" "$STATE"
    say "moved $LEGACY_STATE to $STATE"
fi
if [[ -d "$STATE/.git" ]]; then
    git -C "$STATE" pull --ff-only --quiet || warn "state repo did not fast-forward; resolve in $STATE"
elif [[ ! -e "$STATE" || -z "$(ls -A "$STATE")" ]]; then
    git clone --quiet "$STATE_REMOTE" "$STATE"
    say "cloned $STATE_REMOTE into $STATE"
else
    warn "$STATE exists and is not a git checkout; move it aside and re-run"
    exit 1
fi
if [[ -L "$LEGACY_STATE" || ! -e "$LEGACY_STATE" ]]; then
    mkdir -p "$(dirname "$LEGACY_STATE")"
    ln -sfn "$STATE" "$LEGACY_STATE"
else
    warn "$LEGACY_STATE is a real directory; move its contents into $STATE and re-run"
fi

# 2. Claude memory per repo (Claude names the project dir after its path) and Pi.
for repo in "${REPOS[@]}" "${SHARED_REPOS[@]}"; do
    path="$dev/$repo"
    [[ -d "$path" ]] || { warn "skip $repo: $path missing"; continue; }
    store="$STATE/_home/claude-memory/$repo"
    project="$HOME/.claude/projects/$(printf '%s' "$path" | sed 's#[/.]#-#g')"
    mem="$project/memory"
    mkdir -p "$store"
    if [[ -d "$mem" && ! -L "$mem" ]]; then
        cp -n "$mem"/* "$store"/ 2>/dev/null || true
    fi
    link "$mem" "$store"
done

if [[ -d "$HOME/.pi/agent" ]]; then
    link "$HOME/.pi/agent/extensions/skill-orchestrator" "$STATE/_home/pi/extensions/skill-orchestrator"
    link "$HOME/.pi/agent/skill-orchestrator.json" "$STATE/_home/pi/skill-orchestrator.json"
fi

# 3. Machine paths, outside tracked files.
present=()
for repo in "${REPOS[@]}"; do [[ -d "$dev/$repo" ]] && present+=("$dev/$repo"); done

for path in "${present[@]}"; do
    python3 - "$path" "$STATE" "${present[@]}" <<'PY'
import json, pathlib, sys
repo, state, repos = sys.argv[1], sys.argv[2], sys.argv[3:]
f = pathlib.Path(repo, ".claude", "settings.local.json")
data = json.loads(f.read_text()) if f.exists() else {}
perms = data.setdefault("permissions", {})
dirs = [d for d in perms.get("additionalDirectories", [])]
for d in [r for r in repos if r != repo] + [state]:
    if d not in dirs:
        dirs.append(d)
perms["additionalDirectories"] = dirs
f.parent.mkdir(exist_ok=True)
f.write_text(json.dumps(data, indent=2) + "\n")
PY
    git -C "$path" check-ignore -q .claude/settings.local.json \
        || warn "$path/.claude/settings.local.json is not gitignored; do not commit it"
done
say "wrote .claude/settings.local.json in ${#present[@]} repos"

if command -v codex >/dev/null 2>&1; then
    python3 - "$HOME/.codex/config.toml" "$STATE" "${present[@]}" <<'PY'
import pathlib, re, sys, tomllib
cfg, state, repos = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3:]
begin, end = "# >>> oiko-agent-kit (setup-machine.sh) >>>", "# <<< oiko-agent-kit <<<"
text = cfg.read_text() if cfg.exists() else ""
text = re.sub(re.escape(begin) + r".*?" + re.escape(end) + r"\n?", "", text, flags=re.S)
outside = tomllib.loads(text)
lines = [begin]
trusted = outside.get("projects", {})
for r in repos:
    if r not in trusted:
        lines += [f'[projects."{r}"]', 'trust_level = "trusted"']
if "sandbox_workspace_write" in outside:
    print("!!  ~/.codex/config.toml already has [sandbox_workspace_write]; add the Oiko writable roots there by hand", file=sys.stderr)
else:
    roots = ", ".join(f'"{p}"' for p in repos + [state])
    lines += ["[sandbox_workspace_write]", f"writable_roots = [{roots}]"]
lines.append(end)
text = text.rstrip("\n") + ("\n\n" if text.strip() else "") + "\n".join(lines) + "\n"
tomllib.loads(text)
cfg.parent.mkdir(exist_ok=True)
cfg.write_text(text)
print("==> updated ~/.codex/config.toml (project trust, writable roots)")
PY
fi

# 4. Bring each repo's main up to origin/staging (merged work from the other
#    machine) when that is a clean fast-forward of a checked-out main.
for path in "${present[@]}"; do
    git -C "$path" fetch --quiet origin || { warn "$(basename "$path"): fetch failed"; continue; }
    git -C "$path" rev-parse --verify --quiet origin/staging >/dev/null \
        || { warn "$(basename "$path"): origin/staging is missing; the carrier branch is staging (oiko-worktree, Two machines)"; continue; }
    # `next` was the carrier until 2026-10-10. A session on the old rules may
    # still push it; its work must reach staging, so flag it loudly.
    if git -C "$path" rev-parse --verify --quiet origin/next >/dev/null \
        && ! git -C "$path" merge-base --is-ancestor origin/next origin/staging; then
        warn "$(basename "$path"): origin/next has work that origin/staging lacks (a session on the old rules pushed it); merge origin/next into main and push main:staging (oiko-worktree, Two machines)"
    fi
    if [[ "$(git -C "$path" branch --show-current)" != main ]]; then
        warn "$(basename "$path"): not on main; integrate origin/staging yourself"
    elif git -C "$path" merge-base --is-ancestor origin/staging main; then
        :
    elif git -C "$path" merge --ff-only --quiet origin/staging; then
        say "$(basename "$path"): main fast-forwarded to origin/staging"
    else
        warn "$(basename "$path"): main and origin/staging diverged; merge per oiko-worktree (Two machines)"
    fi
done

say "active claims:"
"$STATE/bin/claim" list || warn "claim list failed"
say "done; commit and push agent state when you stop: git -C $STATE add -A && git -C $STATE commit -m <msg> && git -C $STATE push"
