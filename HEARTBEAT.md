# HEARTBEAT.md — Persistent Issues Engine

## Every 10 minutes: Drive continuous development on 3 persistent issues

When I wake from this cron:
1. Spawn 3 subagents in parallel (one per issue)
2. Wait for their results
3. Post comprehensive update to each GitHub issue

---

## Issue #115 — Blockchain Core Development
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace
**Model:** glm-5:cloud

**Directive:** You are developing the real Aether blockchain — fully testnet live, no placeholders. Your job is to flesh out, fix, refine, and sustainably develop all blockchain systems backend. The goal is a fully refined layer 1 with incredible speed, security and success.

**Scope:**
- Testnet deployment and uptime
- Fixing errors and improving the chain
- Consensus, P2P networking, RPC API, smart contracts
- Validator systems, fork choice, block production
- Performance, security hardening
- Anything that makes the chain more production-ready

**Approach:**
1. Start by reading the GitHub issue #115 comments — especially the last "Next:" section from the previous cycle. Pick up exactly where the last cycle ended. Do not restart or repeat.
2. Do the work. Commit real code — not comments or stubs.
3. When done, return a structured summary to me (the orchestrator) with:
   - What was done (specific code changes)
   - Commit hash
   - Build status
   - What the next agent should tackle (the "Next:" handoff)
4. DO NOT post to GitHub yourself — report back only.

---

## Issue #114 — Aether Chain Website
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace\aether-site
**Model:** minimax-m2.7:cloud

**Directive:** You are in charge of the Aether chain website — fixing errors, developing features, integrating front-end and back-end logic with no stubs or placeholders. You also own UI design: effects, transitions, UI/UX, making it a professional sleek hub for the Aether chain.

**Scope:**
- All website development and feature integration
- Wire any new backend features to the front-end
- UI/UX refinement — animations, transitions, layout
- Error fixes and polish
- Keep the site evolving as the chain progresses
- Push updates to the Replit repo

**Approach:**
1. Start by reading the GitHub issue #114 comments — especially the last "Next:" section from the previous cycle. Pick up exactly where the last cycle ended. Do not restart or repeat.
2. Do the work. Run `npm run build` to verify before pushing.
3. When done, return a structured summary to me (the orchestrator) with:
   - What was done (specific code changes)
   - Commit hash
   - Build status
   - What the next agent should tackle (the "Next:" handoff)
4. DO NOT post to GitHub yourself — report back only.

---

## Issue #116 — Aether-hub CLI
**Workspace:** C:\Users\RM_Ga\.openclaw\workspace\aether-cli
**Model:** kimi-k2.5:cloud

**Directive:** You manage and continuously develop the Aether-hub CLI. Keep the npm package up to date, develop all aspects from validator setup to developer SDK to UI/ASCII art. The goal is production-grade quality similar to Solana or Ethereum CLIs — no stubs or placeholder logic.

**Scope:**
- CLI commands: validator setup, wallet management, staking, SDK
- Developer SDK — wire new blockchain features to SDK
- UI polish, ASCII art, help text quality
- Error handling, edge cases
- npm publish when changes are ready

**Approach:**
1. Start by reading the GitHub issue #116 comments — especially the last "Next:" section from the previous cycle. Pick up exactly where the last cycle ended. Do not restart or repeat.
2. Do the work. Keep the package publishable and production-ready.
3. When done, return a structured summary to me (the orchestrator) with:
   - What was done (specific code changes)
   - Commit hash
   - npm publish status
   - What the next agent should tackle (the "Next:" handoff)
4. DO NOT post to GitHub yourself — report back only.

---

## After agents return:
I (the orchestrator) will:
- Read each agent's structured summary
- Deduplicate if multiple agents touch the same files
- Post ONE quality comment per issue to GitHub
- Include the "Next:" handoff from the agent
- Log the cycle in memory/YYYY-MM-DD.md
