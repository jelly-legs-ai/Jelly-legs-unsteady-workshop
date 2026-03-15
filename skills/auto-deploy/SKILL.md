---
name: auto-deploy
description: Autonomous deployment skill for the Jelly-legs AI team. Use when agents need to deploy code to GitHub Pages, trigger GitHub Actions workflows, or release updates without human approval. Handles git operations, workflow triggers, and deployment verification.
---

# Auto-Deploy Skill

Autonomous deployment system for the 12-agent factory. No human approval required.

## Capabilities

- Deploy to GitHub Pages
- Trigger and monitor GitHub Actions workflows
- Auto-merge PRs that pass checks
- Verify deployment success
- Rollback on failure

## Usage

### Deploy Dashboard to GitHub Pages

```bash
# Stage and commit changes
git add .
git commit -m "🚀 Deploy: <description>"
git push origin main

# Verify GitHub Pages deployment
gh api repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/pages --jq '.html_url, .status'
```

### Trigger GitHub Actions Workflow

```bash
# Trigger specific workflow
gh workflow run <workflow-name> --repo jelly-legs-ai/Jelly-legs-unsteady-workshop

# Monitor workflow run
gh run list --repo jelly-legs-ai/Jelly-legs-unsteady-workshop --limit 5
gh run watch <run-id> --repo jelly-legs-ai/Jelly-legs-unsteady-workshop
```

### Auto-Merge PR

```bash
# Check PR status
gh pr checks <pr-number> --repo jelly-legs-ai/Jelly-legs-unsteady-workshop

# Merge if checks pass
gh pr merge <pr-number> --squash --delete-branch --repo jelly-legs-ai/Jelly-legs-unsteady-workshop
```

## Deployment Checklist

Before deploying:
- [ ] All tests pass (if applicable)
- [ ] Code reviewed by at least one agent
- [ ] Changes committed and pushed
- [ ] No breaking changes to existing functionality

After deploying:
- [ ] Verify live URL works
- [ ] Check GitHub Actions for errors
- [ ] Update issue/PR status

## Rollback

If deployment fails:
```bash
git revert HEAD
git push origin main
```
