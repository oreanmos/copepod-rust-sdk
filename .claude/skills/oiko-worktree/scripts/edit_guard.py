#!/usr/bin/env python3
"""PreToolUse guard for the read-only Oiko subagents (reviewer, scout).

They may build and run code from worktrees but must never change oikonotes,
copepod or copepod-rust-sdk. This refuses (exit 2, reason on stderr) any Edit,
Write or NotebookEdit whose target is inside the main checkout or any worktree
of those three repos. Evidence under ~/.cache/oiko-agents/ stays writable.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

REPOS = ("oikonotes", "copepod", "copepod-rust-sdk")


def worktrees(repo: Path) -> list[Path]:
    """The main checkout and every worktree of the repository at `repo`."""
    try:
        out = subprocess.run(
            ["git", "-C", str(repo), "worktree", "list", "--porcelain"],
            capture_output=True, text=True, check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return [repo] if repo.exists() else []
    return [
        Path(os.path.realpath(line[len("worktree "):]))
        for line in out.splitlines()
        if line.startswith("worktree ")
    ]


def protected_roots() -> list[Path]:
    """Worktrees of all three repos, found beside this script's main checkout."""
    own = Path(__file__).resolve().parents[4]
    main_checkout = (worktrees(own) or [own])[0]
    parent = main_checkout.parent
    roots: list[Path] = []
    for name in REPOS:
        roots.extend(worktrees(parent / name))
    return roots or [own]


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0
    tool_input = payload.get("tool_input") or {}
    raw = tool_input.get("file_path") or tool_input.get("notebook_path")
    if not raw:
        return 0
    cwd = Path(payload.get("cwd") or os.getcwd())
    target = Path(os.path.realpath(cwd / os.path.expanduser(raw)))
    for root in protected_roots():
        if target == root or root in target.parents:
            print(
                f"Refused: {target} is inside {root}, and this agent is read-only. "
                "Put evidence under ~/.cache/oiko-agents/<slug>/.",
                file=sys.stderr,
            )
            return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
