# 🤖 Agent Persona System

## Overview
9 specialized subagent personas that form the autonomous AI team, controlled by Jelly.

## Stage Workflow

```
🔬 Researcher → 🎨 Designer → 💻 Developer → 👁️ Watcher → ⚙️ Engineer → 🛡️ Cybersecurity → 🚀 Deployment
```

## On-Demand Agents
- 🧠 **Skiller** - Creates/updates skills
- 🩹 **Doc** - Fixes errors and emergencies

## Label System

| Label | Agent | Color | Priority |
|-------|-------|-------|----------|
| `research` | Researcher | `#0E8A16` | 1 |
| `design` | Designer | `#5319E7` | 2 |
| `build` | Developer | `#1D76DB` | 3 |
| `review` | Watcher | `#D93F0B` | 4 |
| `engineer` | Engineer | `#28A745` | 5 |
| `security` | Cybersecurity | `#B60205` | 6 |
| `deploy` | Deployment | `#FBCA04` | 7 |
| `skill` | Skiller | `#6F42C1` | On-demand |
| `fix` | Doc | `#D876E3` | On-demand |

## Usage
Each agent file contains:
- Core identity and directive
- Capabilities and workflow
- Handoff triggers
- Checklists (where applicable)

## GitHub Integration
Cron job spawns appropriate agent based on issue labels every 3 minutes.

---
*Created: 2026-02-22 | Issue #7*