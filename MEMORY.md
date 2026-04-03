# MEMORY.md - Long-Term Memory

_Last updated: 2026-04-01_

## Identity
- **Name:** Jelly-legs 🦑
- **Role:** Autonomous AI research lead for "Jelly-legs AI Team"
- **Mission:** Build AI systems that evolve, adapt, and ship

## Project Overview

### Core Project: Project AETHER
**Status:** 🔴 CRITICAL PRIORITY  
**Type:** Solana-forked blockchain optimized for AI workloads  
**Native Token:** AETH  
**Secondary Token:** FLUX (AI operations)  
**Target Mainnet:** June 14, 2026  

### Key Architecture
- Hybrid PoH + PoS consensus ("AetherFlow")
- 400ms slot time, 65,000+ TPS target
- AI Priority Lanes (Critical/Standard/Background)
- Founding validators needed: ≥10 (10K AETH each, 2x bootstrap rewards)
- Total supply: 500M AETH, 10B FLUX

### Development Pipeline
- Phase 1-4: Research → Design → Development → Security
- Phase 5: Testnet
- Phase 6: Mainnet (in progress, started 2026-03-22)

### Reference Documents
- Full architecture: `docs/MAINNET_ARCHITECTURE.md`
- Launch timeline: `docs/LAUNCH_TIMELINE.md`
- Epic details: `epics/PROJECT_AETHER.md`

---

## Autonomous Agent System

### Components
- **GitHub Actions Workflows:** `ai-team-worker.yml` (10min cycles), `auto-merge.yml`, `dashboard-refresh.yml` (30sec)
- **Worker Scripts:** `scripts/ai-team-worker.js`, `scripts/generate-dashboard-data.js`
- **Dashboard:** GitHub Pages - real-time factory view with 12 agents

### 12 Agent Roles
| Agent | Role | Labels |
|-------|------|--------|
| 🛡️ Shield-Bot | Security, bugs | bug, security, error |
| 🎨 Sketch-Bot | Design, UI, pixel | design, ui, pixel |
| 🤿 Data-Diver | Research, analysis | research, analyze |
| 🚀 Launch-Pad | Deploy, workflow | deploy, workflow |
| 💻 Code-Crafter | Build, implement | build, implement |
| 🔮 Pattern-Seeker | Research, trends | research |
| 🎭 Voice-Weaver | Content, comms | voice, content |
| 🧩 Pipe-Layer | Infrastructure | infra, pipe |
| ⚙️ Build-Bot | CI/CD, tooling | build, ci |
| 🗺️ Map-Maker | Docs, diagrams | docs, map |
| 🪼 Jelly-Legs | Orchestration | jelly-legs |
| ⚡ Volt-Runner | Performance | performance |

### Security Rules
- Tokens in GitHub Secrets only
- Never hardcode tokens
- .gitignore excludes .env files

---

## Sprint History

### Sprint 22 (Last - 2026-03-28)
**Focus:** Enhanced mining rewards system  
**Status:** ✅ Complete, local commits made  
**Features Added:**
1. Streak Bonus (up to 1.5x for 24+ consecutive epochs)
2. Peak Hours Bonus (1.2x during 09:00-12:00 and 19:00-23:00 UTC)
3. Early Adopter Bonus (2x for first 10K miners)
4. Geo Diversity Bonus (1.3x for underrepresented regions)
5. Reputation System (up to 1.2x based on activity)
6. Network Bonus Pool (top 10% share 10 FLUX/epoch)

**Blockers:** GitHub push failed (account suspended), missing GITHUB_TOKEN for comments  
**Files:** `contracts/mining_rewards.rs`, `SPRINT-22-MINING-REWARDS.md`

### Sprint 23 (Next - Planned)
- Replit DB integration for subscriptions
- API routes for user/agent management
- FLUX/ATH token contracts
- Staking contract structure

---

## Technical Stack

- **Language:** Rust (blockchain), JavaScript (automation)
- **Framework:** Solana codebase fork, GitHub Actions
- **Deploy:** GitHub Pages (dashboard), GitHub Actions (automation)
- **AI Models:** Ollama (local), OpenRouter (cloud failover)
- **Blockchain:** Solana fork + custom AetherFlow consensus

---

## Open Issues / Blockers

1. **GitHub Push Blocked** - account suspended, need to resolve before deploying
2. **Missing GITHUB_TOKEN** - can't post PR comments automatically
3. **Sprint 23 Not Started** - next priorities: Replit DB, API routes, FLUX/ATH tokens
4. **Week 2 Audit Commissioning** - due ~2026-04-05 (needs budget $40-80K)
5. **Validator Commitments** - need ≥10 founding validators confirmed

---

## Workspace Structure

```
C:\Users\RM_Ga\.openclaw\workspace\
├── Jelly-legs-unsteady-workshop/  # Main GitHub repo
├── aether*/                       # Blockchain components
├── agents/                         # Agent configs
├── contracts/                      # Smart contracts (Rust)
├── crates/                         # Rust crates
├── dashboard/                      # Web dashboard
├── data/                          # Data files
├── docs/                          # Launch docs
├── epics/                         # Project epics
├── research/                      # Research docs
├── scripts/                       # Automation scripts
└── memory/                        # Daily memory logs
```

---

## People & Context
- **Human:** Running the AI R&D team, timezone Europe/London
- **GitHub:** jelly-legs-ai organization, Jelly-legs-unsteady-workshop repo
- **Dashboard:** https://jelly-legs-ai.github.io/Jelly-legs-unsteady-workshop/dashboard/

---

*Memory system established 2026-03-28, updated 2026-04-01*
