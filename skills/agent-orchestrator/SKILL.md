---
name: agent-orchestrator
description: Coordinate the 12-agent AI team to complete GitHub issues autonomously. Use when managing multi-agent workflows, assigning tasks to specific agent roles, spawning sub-agents for parallel work, and tracking progress across the factory floor.
---

# Agent Orchestrator Skill

Coordinate the 12-agent factory for autonomous task completion.

## The 12 Agents

| Agent | Emoji | Role | Specialty | Zones |
|-------|-------|------|-----------|-------|
| Jelly-Legs | 🪼 | Marketing Commander | Narrative, community, viral strategy | All |
| Data-Diver | 🤿 | Research Lead | Deep research, data analysis | Research |
| Pattern-Seeker | 🔮 | Trend Analyst | Pattern recognition, viral mechanics | Research |
| Sketch-Bot | 🎨 | Design Architect | UI/UX, visual systems, pixel art | Design |
| Voice-Weaver | 🎭 | Brand Voice | Content, storytelling, tone | Design |
| Hook-Maker | 🪝 | Viral Engineer | Engagement loops, hooks | Design |
| Build-Bot | ⚙️ | System Developer | Infrastructure, CI/CD | Build |
| Pipe-Layer | 🧩 | Pipeline Engineer | Integrations, data flows | Build |
| Code-Crafter | 💻 | Implementation | Features, code, logic | Build |
| Shield-Bot | 🛡️ | Security Guard | Review, audit, guardrails | Security |
| Map-Maker | 🗺️ | Strategy Lead | Planning, roadmaps, milestones | Strategy |
| Launch-Pad | 🚀 | Deployment Chief | Releases, launches, go-live | Deploy |

## Workflow

### 1. Check Open Issues

```bash
gh issue list --repo jelly-legs-ai/Jelly-legs-unsteady-workshop --state open --json number,title,labels,assignees
```

### 2. Assign to Agent

Match issue labels to agent zones:
- `research` → Data-Diver or Pattern-Seeker
- `design` → Sketch-Bot or Voice-Weaver
- `build` → Build-Bot, Pipe-Layer, or Code-Crafter
- `security` → Shield-Bot
- `strategy` → Map-Maker
- `deploy` → Launch-Pad

### 3. Spawn Sub-Agent

Use `sessions_spawn` to create task-specific agents:
- Task description with context
- Specific deliverables
- Time estimate
- Dependencies

### 4. Track Progress

Update kanban board as work progresses:
- Move issues between stages
- Update agent positions in dashboard
- Log activity to memory files

### 5. Complete & Deploy

- Create PR with completed work
- Run checks
- Auto-merge (deployment green light active)
- Close issue
- Update dashboard

## Collaboration Rules

1. **Agents delegate** - If an agent needs help, spawn another
2. **Parallel work** - Multiple agents can work different issues simultaneously
3. **Communication** - Use speech bubbles in dashboard for status updates
4. **No blocking** - If stuck, escalate to different agent or try alternative approach
5. **Ship fast** - Small, frequent commits beat large, delayed ones

## Issue Priority

1. Blockers (other work depends on this)
2. Quick wins (< 30 min tasks)
3. High impact features
4. Research tasks (inform other work)
5. Nice-to-haves
