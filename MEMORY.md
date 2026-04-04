# MEMORY.md - Long-Term Memory

_Last updated: 2026-04-04_

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
- **Tiered validators: Full (10K AETH), Lite (1K AETH), Observer (no stake, relay-only)**
- Total supply: 500M AETH, 10B FLUX

### Development Pipeline
- Phase 1-4: Research → Design → Development → Security
- Phase 5: Testnet ✅ (2-node network live)
- Phase 6: Mainnet (in progress)

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

### Sprint 22 (2026-03-28)
**Focus:** Enhanced mining rewards system
**Status:** ✅ Complete
**Features Added:** Streak Bonus, Peak Hours Bonus, Early Adopter Bonus, Geo Diversity Bonus, Reputation System, Network Bonus Pool

### Sprint 23 (2026-04-04)
**Focus:** Unified staking + testnet hardening
**Status:** ✅ Complete
**Work:** unified_staking.rs, get_pool accessors, calculate_pending_rewards, 2-node TCP P2P, slot sync on handshake

### Sprint 24 (2026-04-04 afternoon)
**Focus:** Aether-Onboard-CLI P0/P1/P2
**Status:** ✅ Complete
**Work:** validator-start.js binary detection, init.js wiring, doctor auto-fix, monitor (REST polling), logs, sdk, npm global install

### Sprint 25 (2026-04-04 late afternoon)
**Focus:** Tiered Validator System (Issue #113)
**Status:** ✅ Complete — Issue #113 fully implemented
**Rust:** ValidatorTier enum, TierConfig, stake enforcement (10K/1K/0), --tier CLI flag, relay rewards
**CLI:** doctor --tier, init tier selection, validator-start --tier, monitor tier display

## NPM Packages Published
- `@jellylegsai/aether-validator-cli@1.0.2` — tiered validators, interactive menu, postinstall guide
- Install: `npm install -g @jellylegsai/aether-validator-cli`
- Bin aliases: `aether`, `aether-validator`, `jelly-aether`

---

## Technical Stack

- **Language:** Rust (blockchain), JavaScript (automation)
- **Framework:** Solana codebase fork, GitHub Actions
- **Deploy:** GitHub Pages (dashboard), GitHub Actions (automation)
- **AI Models:** Ollama (local), OpenRouter (cloud failover)
- **Blockchain:** Solana fork + custom AetherFlow consensus
- **Rust toolchain:** C:\Users\RM_Ga\.cargo\bin (rustc 1.94.1) — cargo not in exec PATH by default

---

## Open Issues / Blockers

1. **Validator Commitments** — Need ≥10 founding validators confirmed (10K AETH each, 2x bootstrap rewards)
2. **NPM automation token** — Stored in `C:\Users\RM_Ga\AppData\Roaming\npm\etc\npmrc` — write-only, cannot read metadata from this machine (IP-restricted). Package publishes fine, users can install normally.
3. **Week 2 Audit** — Was due ~2026-04-05. Status unknown — verify if completed.
4. **Mainnet launch** — June 14, 2026 target. ~10 weeks away.

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

*Memory system established 2026-03-28, last full review 2026-04-04*
