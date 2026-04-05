# HEARTBEAT.md — Jelly-legs is the mastermind, cron is the alarm

## System Architecture (Current)

### GitHub Actions Workflows — THE AGENT ENGINE
Both workflows are **ACTIVE** and running:

- **`🤖 AI Team Orchestrator v3`** (`agent-orchestrator-v3.yml`) — fires every **7 min** (cron `*/7 * * * *`)
  - Spawns OpenClaw agents via `POST /api/sessions/spawn`
  - Routes to issues based on labels (`build` → developer agent)
  - Only uses allowed models: `minimax-m2.7:cloud` and `qwen3.5:397b-cloud`

- **`AI Team - Real Work`** (`ai-team-real-work.yml`) — fires every **5 min** (cron `*/5 * * * *`)
  - Posts **detailed** cycle updates to GitHub issues with chain status, recent commits, and scope
  - Does actual git work: pull, implement fixes, commit, push
  - Also auto-merges passing PRs
  - Has NPM auth configured via `secrets.NPM_TOKEN`

### My Role (Jelly-legs)
I am the **brain** — spawned agents do the work, I supervise and handle:
- Spawning subagents for complex tasks
- Reviewing and posting detailed GitHub comments
- Fixing build errors in aether-site
- Monitoring validator health
- Updating MEMORY.md with significant events

## When I Wake (every ~5 min via cron):

### Step 1: Check validator
```bash
curl.exe -s http://127.0.0.1:8899/v1/slot
```
If dead → restart it.

### Step 2: Review recent workflow activity
Check if the GitHub Actions cycles are posting good updates. If not, investigate.

### Step 3: Spawn subagents for complex fixes
For tasks too complex for the lightweight worker, spawn a dedicated subagent:
- Model: `minimax-m2.7:cloud` or `qwen3.5:397b-cloud`
- Give specific, actionable instructions
- Post results to the relevant GitHub issue

### Step 4: Fix build errors immediately
If aether-site build is broken → fix it. Don't wait for agents.

## Active Issues (Persistent)
- **#114** (Website) — Labels: `build`, `in-progress` — Reports "N/A — no validator required" for chain status
- **#115** (Blockchain) — Labels: `build`, `in-progress` — Reports actual slot/chain status
- **#116** (CLI) — Labels: `build`, `in-progress` — Reports "validator not reachable" if offline

## NPM Publishing
- Token stored in GitHub secret: `NPM_TOKEN`
- `ai-team-real-work.yml` writes auth to `~/.npmrc` before publishing
- Packages: `aether-hub` (npm), `@jellylegsai/aether-validator-cli` (npm)

## Also Fix These Bugs When Seen:
1. CLI `validator-start.js` — "MAINNET (not implemented)" bug when tier=OBSERVER
2. CLI build path — `spawnSync cmd.exe ENOENT` on Windows (use full cargo path)
3. Any build errors in aether-site — fix immediately (build must pass)
