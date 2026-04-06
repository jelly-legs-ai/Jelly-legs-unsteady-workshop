# HEARTBEAT.md — Persistent Issues Engine

## Every 10 minutes: Do real work on 3 persistent issues

When I wake from this cron:
1. Spawn 3 subagents in parallel (one per issue)
2. Wait for their results
3. Post comprehensive update to each GitHub issue

## Issue #115 — Blockchain Core
Workspace: C:\Users\RM_Ga\.openclaw\workspace
Task: Check validator health, pick ONE fix, implement it, commit/push, return a summary of what was done.

## Issue #114 — Website Integration  
Workspace: C:\Users\RM_Ga\.openclaw\workspace\aether-site
Task: Pull latest, identify ONE integration gap, implement it, build/verify, commit/push, return summary.

## Issue #116 — CLI Development
Workspace: C:\Users\RM_Ga\.openclaw\workspace\aether-cli
Task: Pull latest, add ONE new command or fix, npm publish if possible, commit/push, return summary.

## After agents return:
Post a DETAILED comment to each GitHub issue with:
- What was done (specific, not placeholder)
- What was committed (include commit hash)
- Validator status if applicable
- What the next cycle should tackle

Use this format for each issue comment:
```
## 🪼/🌐/💻 Cycle Update — [TIMESTAMP]

**What was done:** [specific description of real code changes]
**Commit:** [hash] — [description]
**Validator/Chain:** [status]
**Next:** [what next cycle should work on]
```