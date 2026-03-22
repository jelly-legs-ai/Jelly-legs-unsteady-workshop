# HEARTBEAT.md

## Periodic Checks

### Memory Maintenance (every 3 days, via cron)
- Run `node memory/summarizer.js audit` to check context load
- If context > 15k chars, distill more into MEMORY.md
- Summarize any unlogged recent sessions

### Active Work Tracking
- Check git status for any uncommitted changes
- If work in progress, continue or log to today's memory file

### Context Budget
- If session exceeds ~30 turns, prompt to summarize or start fresh
- Watch for early warning signs (increasing "remind me" pattern)
