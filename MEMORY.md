# MEMORY.md - Long-Term Memory

_Last updated: 2026-04-06_

## Critical System Change (2026-04-05 Evening)

### GitHub Actions DISABLED for AI Work
GitHub Actions cannot do AI — it was only posting placeholder spam comments.
**ALL real AI work now runs through ME via local OpenClaw cron.**

- `agent-orchestrator-v3.yml` → renamed to `.DISABLED`
- `ai-team-real-work.yml` → renamed to `.DISABLED`
- Real comments: `jelly-legs-ai` user ✅
- Spam comments: `github-actions[bot]` ❌ (from disabled workflows)

### My Cron: The Only Working AI System
- **Job:** "AI Team - 7min Persistent Issues Cycle" (ID: `94440797-4584-4f24-8526-e498cdcfb6e1`)
- **Schedule:** Every 7 min (420,000ms)
- **Delivery:** `mode: "none"` (silent work, no announcements)
- **3 subagents in parallel per cycle:**
  - #115 Blockchain: `qwen3.5:397b-cloud`
  - #114 Website: `kimi-k2.5:cloud`
  - #116 CLI: `gemma3:27b-cloud`

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

### ⚠️ GitHub Actions CANNOT run AI — DISABLED
GitHub Actions has no AI capability. The `agent-orchestrator-v3.yml` and `ai-team-real-work.yml` workflows were producing **placeholder spam** (comments from `github-actions[bot]`). They have been renamed to `.DISABLED` files.

**ALL real AI work runs through ME (Jelly-legs) via local OpenClaw cron.**

### My Cron Job: "AI Team - 7min Persistent Issues Cycle"
- **Fires every 7 minutes** (420,000ms interval)
- Spawns 3 subagents in parallel — one per persistent issue
- Models: `qwen3.5:397b-cloud` (blockchain), `kimi-k2.5:cloud` (website), `gemma3:27b-cloud` (CLI)
- Each agent: pulls git, makes ONE real code change, commits, pushes
- Posts GitHub comments as `jelly-legs-ai` user (NOT `github-actions[bot]`)
- Cron ID: `94440797-4584-4f24-8526-e498cdcfb6e1`

### Important Model Constraints
Only these models work as subagents from this OpenClaw setup:
- `minimax-m2.7:cloud` — general purpose
- `qwen3.5:397b-cloud` — best for blockchain/Rust
- `kimi-k2.5:cloud` — great for web dev/TypeScript
- `gemma3:27b-cloud` — good for CLI/tooling
- `gemma3:12b-cloud`, `gemma3:4b-cloud`

### 12 Agent Roles (for reference)
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

## Persistent Issues System (Updated 2026-04-05)

### Three Always-Running Issues
- **#115** Blockchain Core Development — spawns `heartbeat-chain` subagent
- **#114** Website Integration — spawns `heartbeat-web` subagent  
- **#116** CLI Development — spawns `heartbeat-cli` subagent

### How It Works
- I check HEARTBEAT.md on every heartbeat poll (~30 min intervals)
- I spawn 3 parallel subagents, one per persistent issue
- Each agent: pulls latest, implements ONE fix, commits/pushes, posts SPECIFIC GitHub comment
- `heartbeat-state.json` tracks cycle timing
- GitHub Actions `ai-team-real-work.yml` also runs every 5 min (posts cycle comments, can't spawn agents from CI)

### NPM Packages Published
- `@jellylegsai/aether-validator-cli@1.0.5` — wallet stake/transfer, sdk types, CLI commands
- `@jellylegsai/aether-hub@1.0.5` — same as above (aliased)
- Install: `npm install -g @jellylegsai/aether-validator-cli`

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
