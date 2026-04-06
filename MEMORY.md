# MEMORY.md — Jelly-legs Long-Term Memory

## About Reggie (My Human)

- Runs the AI R&D team
- Timezone: Europe/London
- Has 3 active repos: blockchain (Jelly-legs-unsteady-workshop), website (Aether-Chain), CLI (aether-cli)
- Wants autonomous operation — I should just do the work, not ask

## Project Context

### Active Repos & Workspaces
- **Blockchain:** `C:\Users\RM_Ga\.openclaw\workspace\Jelly-legs-unsteady-workshop`
  - Rust blockchain with validator, consensus, core crates
  - Local RPC server on `http://127.0.0.1:8899`
  - Genesis: `genesis.json` with epoch_duration=432000, slot_time_ms=400
  - 1 bootstrap validator (CSQLoeZ9FwGpsZr94MvnQGbufuR9ux4ZiXikMGFL8asK, 10M stake)
- **Website:** `C:\Users\RM_Ga\.openclaw\workspace\aether-site` (repo: jelly-legs-ai/Aether-Chain)
  - Next.js app with staking, bridge, dashboard, SDK pages
  - API routes under `src/app/api/`
- **CLI:** `C:\Users\RM_Ga\.openclaw\workspace\aether-cli`
  - Node.js CLI with `@aetherai/sdk` package
  - SDK at `lib/sdk/client.js` with 14+ real RPC functions

### Persistent Issues (AI Team 10-min Cycles)
- **#115** — Blockchain core fixes (in `Jelly-legs-unsteady-workshop`)
- **#114** — Website integration (in `Jelly-legs-unsteady-workshop` issue tracker)
- **#116** — CLI + SDK buildout (in `Jelly-legs-unsteady-workshop` issue tracker)

### Important Notes
- PowerShell uses `;` separator, NOT `&&` for chained commands
- gemma3:27b-cloud model is UNRELIABLE (500 errors) — use kimi-k2.5:cloud for CLI tasks
- Chain RPC: `http://127.0.0.1:8899` with endpoints: /v1/slot, /v1/epoch, /v1/validators, /v1/tx, /v1/account/<addr>
- All 3 repos need commits + pushes + GitHub issue comments each cycle

## What I Know How To Do

### Blockchain (Rust)
- Fix RPC endpoints in `crates/aether-validator/src/rpc_server.rs`
- Fix state management in `crates/aether-validator/src/state.rs`
- Fix block production in `crates/aether-validator/src/block_producer.rs`
- Run `cargo build --release` and handle "Access denied" (kill aether-validator.exe first)
- Check chain: `Invoke-WebRequest -Uri "http://127.0.0.1:8899/v1/slot" -UseBasicParsing | Select-Object -ExpandProperty Content`

### Website (Next.js)
- Find integration gaps between API routes and frontend components
- Implement fixes in `src/app/` pages and `src/app/api/` routes
- Run `npm run build` and verify success

### CLI/SDK (Node.js)
- SDK at `lib/sdk/client.js` — all functions must call real `http://127.0.0.1:8899` endpoints
- CLI commands should wire to SDK functions
- Use `kimi-k2.5:cloud` model (NOT gemma3)

## Patterns That Work
1. Spawn 3 agents in parallel, each with specific workspace + model
2. Wait for completion, check git log for new commits
3. Post GitHub comments to `jelly-legs-ai/Jelly-legs-unsteady-workshop` issues
4. Log cycle in `memory/YYYY-MM-DD.md`

## Status
- Last cycle: 2026-04-06 09:37 UTC — All 3 issues fixed ✅
  - #115: epoch_duration from genesis (a936505)
  - #114: staking-rewards API integration (6aa8000)
  - #116: @aetherai/sdk package created (4168e82)
