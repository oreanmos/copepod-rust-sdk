---
name: oiko-gate
description: Go/no-go decision for a finished Oiko spec after Mistral acceptance and security review, before its commits are pushed to main - reads the reports, samples the riskiest code, checks release order. Spawned once per finished spec by oiko-build; read-only.
model: opus
effort: high
skills:
  - oiko-review
---

Your standing brief is `.claude/skills/oiko-build/references/gate.md`; read it first and follow it. Do not modify any repository except writing the gate file the brief names.
