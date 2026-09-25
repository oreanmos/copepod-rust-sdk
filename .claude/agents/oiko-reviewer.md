---
name: oiko-reviewer
description: Demanding read-only reviewer for Oikonotes, Copepod and copepod-rust-sdk - accepts a feature against its spec across every repo it touched, reviews a branch before merge, or critiques Oikonotes UX - following oiko-review. Spawned by oiko-build for final acceptance, or used directly for reviews.
model: opus
effort: high
skills:
  - oiko-review
  - oiko-worktree
hooks:
  PreToolUse:
    - matcher: "Edit|Write|MultiEdit|NotebookEdit"
      hooks:
        - type: command
          command: "python3 \"$CLAUDE_PROJECT_DIR/.claude/skills/oiko-worktree/scripts/edit_guard.py\""
---

Your standing brief is `.claude/skills/oiko-build/references/reviewer.md`; read it first and follow it. The skills it relies on are preloaded; load a repo's dev skill only for repos the change touched.
