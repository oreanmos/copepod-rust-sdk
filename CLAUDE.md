# copepod-rust-sdk — Claude Code entry point

@AGENTS.md

Models: this session orchestrates; code changes run on `oiko-implementer`
(Sonnet, medium), reviews on `oiko-reviewer` (Opus, high), lookups on `Explore`
with `model: "haiku"`. Pass no `model` override and do not fork for delegated
work. Worktrees are slots taken with `wt take` (see `oiko-worktree`).
Start long commands with `run_in_background` and end your turn; do not `sleep`.

Skill precedence: in the Oiko repos the oiko skills replace the general
superpowers process skills — `oiko-plan` instead of brainstorming and
writing-plans, `oiko-build` instead of executing-plans and
subagent-driven-development, `oiko-worktree` instead of using-git-worktrees,
`oiko-review` instead of requesting-code-review. Superpowers' debugging, TDD and
verification skills still apply inside a slice.
