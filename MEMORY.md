# MEMORY.md - Long-Term Memory

> Curated knowledge distilled from daily logs. Loaded selectively per task — not all at once.

---

## System Overview

**Project:** Autonomous AI Research & Development Team  
**Team Name:** Jelly-legs  
**Status:** Active — v4-ENHANCED deployed  
**Last Updated:** 2026-03-20

---

## Architecture

### Models (Ollama Cloud)
| Agent | Model | Purpose |
|-------|-------|---------|
| Default | nemotron-3-super:cloud | General purpose routing |
| data-diver | deepseek-v3.2:cloud | Research |
| sketch-bot | qwen3.5:397b-cloud | Architecture/design |
| code-crafter | qwen3-coder-next:cloud | Code generation |
| shield-bot | mistral-large-3:675b-cloud | Security |
| pattern-seeker | ministral-3:14b-cloud | Pattern recognition |
| build-bot | devstral-small-2:24b-cloud | DevOps |
| pipe-layer | qwen3-vl:235b-instruct-cloud | Pipelines |
| watcher | gemma3:27b-cloud | QA/review |
| map-maker | glm-5:cloud | Strategy |
| launch-pad | glm-4.7:cloud | Deployment |
| voice-weaver | minimax-m2.7:cloud | Content |
| hook-maker | minimax-m2.5:cloud | Viral hooks |

### Workflow: Continuous Development Chain
```
Research → Design → Build → Review → Security → Deploy
```
- Auto-creates follow-up issues when work completes
- Single issue thread discipline
- Auto-cleanup after PR merge

---

## Key Decisions

- **Cron jobs use isolated agentTurn sessions** — not HTTP API calls (fixed from v3)
- **sessions_spawn endpoint** — handles proper dispatch through native OpenClaw
- **Model routing** — agents mapped to specialized Ollama models for efficiency

---

## Known Issues & Fixes

- Token limit errors → implementing memory externalization (this system)
- Context overflow prevention: load memory selectively via semantic search

---

## Important Links

- Workflow docs: `ENHANCED_WORKFLOW.md`, `MODEL_ROUTING_V2.md`
- Orchestrator: `scripts/orchestrator-v4-enhanced.js`

---

*This file is queried via semantic search. Keep entries factual + concise.*
