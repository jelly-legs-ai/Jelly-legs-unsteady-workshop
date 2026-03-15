# Autonomous AI Team System v2.0

## Architecture

### Components

1. **GitHub Actions Workflows** (Server-side)
   - `ai-team-worker.yml` - Runs every 10 minutes
   - `auto-merge.yml` - Auto-merges passing PRs
   - `dashboard-refresh.yml` - Updates dashboard data every 30 seconds

2. **Worker Scripts** (`scripts/`)
   - `ai-team-worker.js` - Assigns agents to issues, creates branches/PRs
   - `generate-dashboard-data.js` - Fetches GitHub data for dashboard

3. **Dashboard** (`dashboard/`)
   - Real-time factory view with 12 agents
   - Zone modals, work request system
   - Pixel art sprites

### Workflow

```
Every 10 minutes:
  ├─ GitHub Actions triggers
  ├─ ai-team-worker.js runs
  │   ├─ Fetches open issues
  │   ├─ Assigns agent based on labels
  │   ├─ Creates work branch
  │   ├─ Implements feature (or creates plan)
 │   ├─ Commits changes
  │   └─ Opens PR
  └─ auto-merge.yml
      └─ Merges PR if checks pass
```

### Agent Assignment Logic

| Label/Keyword | Assigned Agent |
|--------------|----------------|
| bug, security, error | 🛡️ Shield-Bot |
| design, ui, pixel | 🎨 Sketch-Bot |
| research, analyze | 🤿 Data-Diver |
| deploy, workflow | 🚀 Launch-Pad |
| build, implement | 💻 Code-Crafter |

### Security

- Tokens stored in GitHub Secrets (GITHUB_TOKEN)
- Never hardcode tokens in source
- .gitignore excludes .env files
- GitHub Push Protection enabled

### Rate Limiting

- 10-minute cycles (not 5) to avoid API limits
- Max 2 issues processed per run
- GitHub Actions has its own rate limits (higher than personal tokens)

## Deployment

Auto-deploy is active. All PRs created by the AI team auto-merge after checks pass.

## Monitoring

- Dashboard: https://jelly-legs-ai.github.io/Jelly-legs-unsteady-workshop/dashboard/
- GitHub Actions: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/actions
- Issues: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues
