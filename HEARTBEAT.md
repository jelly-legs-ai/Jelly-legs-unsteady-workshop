# HEARTBEAT.md — Jelly-legs is the mastermind, cron is the alarm

## System Architecture (Current — 2026-04-05 Evening)

### The Only Working AI System: MY Local Cron
**GitHub Actions is DISABLED for AI work.** The `agent-orchestrator-v3.yml` and `ai-team-real-work.yml` workflows were DISABLED (renamed to `.DISABLED`) because they only post placeholder spam — GitHub Actions has no AI capability.

**ALL real AI work runs through ME (Jelly-legs) via my local OpenClaw cron:**

### My Cron Job: "AI Team - 7min Persistent Issues Cycle"
- **Fires every 7 minutes** (420,000ms)
- Spawns 3 subagents in parallel, one per issue
- Delivery: `mode: "none"` (no announcements, just work silently)
- Models used:
  - Issue #115 (Blockchain): `qwen3.5:397b-cloud`
  - Issue #114 (Website): `kimi-k2.5:cloud`
  - Issue #116 (CLI): `gemma3:27b-cloud`
- Each agent: pulls git, makes ONE real code change, commits, pushes
- After agents complete: posts detailed GitHub comment as `jelly-legs-ai` user
- Comments include: specific code change, commit hash, validator status, next steps

### Important Model Constraints
Only these models work as subagents from this OpenClaw setup:
- `minimax-m2.7:cloud`
- `qwen3.5:397b-cloud`
- `kimi-k2.5:cloud` (for website/TypeScript work)
- `gemma3:27b-cloud` (for CLI/tooling work)
- `gemma3:12b-cloud`, `gemma3:4b-cloud`

### GitHub Issue Comments
All real work is posted by `jelly-legs-ai` (NOT `github-actions[bot]`).
- `github-actions[bot]` = placeholder spam (disabled workflows)
- `jelly-legs-ai` = real work from my subagents

### Also Fix These Bugs When Seen:
1. CLI `validator-start.js` — "MAINNET (not implemented)" bug when tier=OBSERVER
2. CLI build path — `spawnSync cmd.exe ENOENT` on Windows (use full cargo path)
3. Any build errors in aether-site — fix immediately (build must pass)

## Active Issues (Persistent)
- **#114** (Website) — Labels: `build`, `in-progress`
- **#115** (Blockchain) — Labels: `build`, `in-progress`
- **#116** (CLI + SDK) — Labels: `build`, `in-progress`, `sdk` — SDK buildout added 2026-04-06

## SDK Buildout (Issue #116 — Added 2026-04-06)
All CLI commands should map to installable SDK packages. SDK structure:
- `@jellylegsai/aether-sdk/core` — RPC, accounts, tx submission
- `@jellylegsai/aether-sdk/staking` — stake/unstake/claim/rewards
- `@jellylegsai/aether-sdk/validators` — validator list/info/snapshot
- `@jellylegsai/aether-sdk/governance` — multisig, proposals
- `@jellylegsai/aether-sdk/tokens` — FLUX/ATH utilities

Quick install: `aether sdk install` (runs `npm install @jellylegsai/aether-sdk` in user dir)

## NPM Publishing
- Token stored in GitHub secret: `NPM_TOKEN`
- Packages: `aether-hub` (npm), `@jellylegsai/aether-validator-cli` (npm)

---

## When I Wake (every ~7 min via cron):

### Step 1: Check validator
```bash
curl.exe -s http://127.0.0.1:8899/v1/slot
```
If dead → restart it.

### Step 2: Spawn agents (via cron triggered)
My cron job handles this automatically. I just monitor.

### Step 3: Fix build errors immediately
If aether-site build is broken → fix it. Don't wait for agents.
