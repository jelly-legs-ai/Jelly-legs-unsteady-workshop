# HEARTBEAT.md — Jelly-legs is the mastermind, cron is the alarm

## System Architecture
- **GitHub Actions `ai-team-real-work.yml`** — DISABLED (was posting placeholder comments)
- **My cron** — Fires every 5 minutes, wakes ME up
- **I am the brain** — Spawn agents, review their work, post detailed updates to GitHub

## When cron fires (every 5 minutes):

### Step 1: Check validator
```bash
curl.exe -s http://127.0.0.1:8899/v1/slot
```
If dead, restart it.

### Step 2: Spawn 4 agents in parallel
- **`glm-5:cloud`** → Issue #115 (blockchain core — complex systems specialist)
- **`minimax-m2.7:cloud`** → Issue #114 (website integration)
- **`qwen3.5:397b-cloud`** → Issue #116 (CLI/npm development)
- **`gemma4:31b-cloud`** → Orchestrator (summarize all results, post to GitHub issues)

Each agent gets this instruction:
```
You are working on Issue #[NUM]. 
Workspace: [path]
Task: [specific work]
1. Pull latest code
2. Implement ONE specific fix
3. Commit and push
4. Return: what was done, commit hash, next steps
```

### Step 3: After agents return
Post detailed GitHub comment to each issue with:
- What was done (specific, real)
- Commit hash + description  
- Validator status
- What next cycle should tackle

Use THIS format exactly:
```markdown
## Cycle Update — YYYY-MM-DD HH:MM UTC

**What was done:** [specific description of real code changes]
**Commit:** [hash]
**Chain status:** [slot / healthy / down]
**Next:** [what next cycle should do]
```

## Also fix these bugs when you see them:
1. CLI `validator-start.js` — "MAINNET (not implemented)" bug when tier=OBSERVER
2. CLI build path — `spawnSync cmd.exe ENOENT` on Windows (use full cargo path)
3. Any build errors in aether-site — fix them immediately
