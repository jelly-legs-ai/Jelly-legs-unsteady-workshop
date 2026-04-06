# HEARTBEAT.md — Persistent Issues Engine

## Every 10 minutes: Drive continuous development on 3 persistent issues

When I wake from this cron:
1. Spawn 3 subagents in parallel (one per issue)
2. Wait for their results
3. Post comprehensive update to each GitHub issue

---

## 🪼 Issue #115 — Aether Blockchain Core
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace
**Model:** glm-5:cloud
**GitHub:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/115

### What Aether Is
A Solana-quality layer 1 blockchain fork optimized for AI workloads. The chain must be a fully functioning, living ledger — production-grade, not a prototype.

### Vision
- **AetherFlow consensus**: Hybrid PoH + PoS with AI Priority Lanes
- **AI Priority Lanes**: AI operators pay premium gas to compete for priority — their transactions automatically route to Critical/High lanes. Real-life traders use Standard. AI-vs-AI competition drives fees to the team master wallet, funding network development, audits, airdrops, validator rewards, and community incentives.
- **Performance target**: 400ms slot time, 65,000+ TPS, 500M AETH supply
- **Validator tiers**: Full (10K AETH), Lite (1K AETH), Observer (relay-only)

### Your Mission
Survey the blockchain codebase. Find what needs the most attention right now for a production-ready testnet. Implement something meaningful. Push real code — not stubs or comments.

### Scope
- Consensus engine (AetherFlow, Tower BFT, PoH)
- P2P networking and block propagation
- RPC API — every endpoint must work
- Smart contracts and transaction execution
- Validator systems, stake/unstake/claim
- AI Priority Lane routing and fee handling
- Security hardening, error handling
- Performance optimization
- Test coverage

---

## 🌐 Issue #114 — Aether Chain Website
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace\aether-site
**Model:** minimax-m2.7:cloud
**GitHub:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/114

### What the Website Is
The front-of-house for the Aether chain — the HTML portal that gives web2 users frictionless access to web3. Must be fully integrated with the real Aether chain. Professional, sleek, production-ready.

### Vision
- **Web2 access point**: Users land here and can interact with the chain without installing CLI tools
- **Solana wallet adapter**: Support Phantom, Solflare, Backpack, and other Solana-compatible wallets via `@solana/wallet-adapter-react`
- **Wallet integration**: Aether is Solana-compatible — existing Solana wallets work. Connect button → sign → verify ownership
- **Staking dashboard**: Real-time stake positions, APY, validator performance
- **AI operator portal**: Where AI agents/operators connect wallets, pay gas for priority lane access, monitor their positions
- **Chain explorer lite**: Recent blocks, TX history, validator stats — all live from RPC
- **Professional UI/UX**: Animations, transitions, responsive design — must feel like a top-tier DeFi protocol

### Your Mission
Survey the website. Find broken features, missing integrations, or UI that needs polish. Pick ONE thing that moves it toward the vision above. Always run `npm run build` before pushing.

### Scope
- Wallet connection (Solana wallet adapter)
- Staking interface (real chain, real data)
- Chain stats (live from RPC)
- AI operator tools (priority lane dashboard)
- UI/UX polish (animations, layout, responsiveness)
- Error handling and edge cases

---

## 💻 Issue #116 — Aether-hub CLI
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace\aether-cli
**Model:** kimi-k2.5:cloud
**GitHub:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/116

### What the CLI Is
The terminal-based onboarding tool for the Aether chain. Three audiences:
1. **Validators**: `aether validator-setup`, `aether validator-start`, `aether validator-status` — one command to a running node
2. **Developers**: `aether sdk`, `aether deploy`, `aether call` — build and interact with on-chain programs
3. **Dapp integrators**: SDK library (`@jellylegsai/aether-sdk`), RPC documentation, local devnet

### Vision
Production-grade like Solana CLI (`solana-cli`) or Ethereum (`cast`/`geth`). Every command works against the real chain. No stubs. No mocks.

### Priority Commands
- `aether init` — First-time setup wizard (wallet create/import, RPC config, validator config)
- `aether validator` — Full validator lifecycle (setup, start, status, restart)
- `aether stake/unstake/claim` — Stake management with real TX submission
- `aether wallet` — BIP39 wallets, connect to web wallet, export
- `aether sdk` — Developer SDK with all RPC methods wired to real chain
- `aether network` — Chain stats, peer count, slot info
- `aether tx` — Submit and confirm transactions

### Your Mission
Survey the CLI. Find what is un-wired, rough, or missing. Pick ONE thing that moves it toward production-grade quality. npm publish when ready.

### Scope
- All SDK methods wired to real `/v1/` RPC endpoints
- Validator setup and management commands
- Wallet management (create, import, connect)
- Staking flow (stake, unstake, claim, delegations)
- Developer tools (SDK, deploy, call)
- npm package publishable and installable globally

---

## After agents return:
I will post ONE quality comment per issue to GitHub with:
- What was done (specific code changes)
- Commit hash and description
- Build / npm publish status
- What the next agent should tackle ("Next:" handoff)

Format:
```
## 🪼/🌐/💻 Cycle Update — [TIMESTAMP]

**What was done:** [specific description of real code changes]
**Commit:** [hash] — [description]
**Status:** [build / npm published]
**Next:** [exactly what the next agent should tackle next]
```
