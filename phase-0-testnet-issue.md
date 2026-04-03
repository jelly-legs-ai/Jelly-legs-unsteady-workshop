## Context

We have ~90K lines of Rust code across 8 crates + 30 smart contracts. It compiles cleanly but **nothing is integrated into a runnable system**. This issue tracks making the testnet actually runnable — where "runnable" means:

1. A 2-3 node testnet that processes real transactions
2. Validators that do actual work (not just simulate counters)
3. Contracts that execute (staking, mining, token)
4. A CLI that a user can run to join the testnet

After this is done: we iterate on security, economics, frontend, SDK — in that order.

---

## Current State

### ✅ What We Have

**Binary:** `aether-validator.exe` — compiles, CLI works, but only simulates block production (in-memory counter). Does not execute contracts or do real P2P.

**Consensus module** (`aether-consensus`): Well-structured PoH + PoS + Tower BFT code with:
- `PoHGenerator` with Blake3-based verifiable delay function
- `StakePool` and `LeaderSchedule` for PoS leader election  
- `AIPriorityQueue` (Critical/High/Standard lanes)
- `AetherFlow` engine with block production
- `Tower` fork choice implementation
- Tests for priority ordering, genesis, AI metadata

**Smart contracts** (in `contracts/`): ~30 .rs files, ~400KB+ of code:
- `staking_contract.rs` (109KB) — tiered staking, auto-compound, slashing
- `mining_contract.rs` (90KB) — streak bonuses, geo diversity, reputation
- `flux_token.rs`, `ath_token.rs`, `aeth_token.rs` — token contracts
- `liquidity_pool.rs` — AMM with IL calculation
- `governance_contract.rs`, `bridge_enhanced.rs`, `cross_chain_bridge.rs`
- Plus full API route definitions

**Other crates:** `aether-common`, `aether-ai-priority`, `aether-mobile`, `aether-governance`, `aether-privacy` — some have real code, some are stubs.

### ❌ What's Missing (Critical Gaps)

1. **Contracts not compiled** — all `contracts/*.rs` are loose files, not in any crate or build
2. **Workspace incomplete** — `Cargo.toml` only lists `aether-validator` as a workspace member; 7 other crates exist but ignored
3. **No contract runtime** — no VM, no state machine, no transaction execution engine
4. **No persistent state** — no RocksDB or similar; all state is in-memory
5. **Simulated consensus** — `aether-validator` just increments a counter, doesn't use the `aether-consensus` module
6. **No P2P block propagation** — gossip just logs, doesn't actually sync blocks
7. **No transaction pool** — transactions submitted via CLI don't go through the contract layer

---

## Scope

### Phase 0A — Fix the Workspace (Foundation)

- [ ] Add all 8 crates to workspace `Cargo.toml`
- [ ] Fix missing `aether-core` (empty crate — needs actual content or remove)
- [ ] Fix missing `aether-governance`, `aether-privacy` (empty, stub or remove)
- [ ] Ensure all crates compile: `cargo build --workspace`
- [ ] Run `cargo clippy --workspace -D warnings` — target 0 warnings

**Deliverable:** `cargo build --workspace && cargo clippy --workspace -D warnings` passes clean.

### Phase 0B — Build the Contract Runtime

- [ ] Create `aether-runtime` crate — the transaction execution engine
  - Loads contracts from `contracts/`
  - Executes transactions against a state database
  - Connects to `aether-consensus` for block production
- [ ] Integrate `aether-common` types throughout (AIPriorityLane, AITransactionMeta, etc.)
- [ ] Add RocksDB for persistent state (`aether-rocksdb` or use existing rocksdb-sys`)
- [ ] Wire `AetherFlow::produce_block()` to actually execute transactions via runtime

**Deliverable:** A block produced by `aether-validator` executes real contract transactions.

### Phase 0C — Build the Testnet CLI (User Onboarding)

- [ ] `aether validator --join <network>` — connect to testnet
- [ ] `aether miner --start` — start mining (PoW/PoH work on device)
- [ ] `aether stake --delegate <amount>` — stake tokens
- [ ] `aether wallet --create` / `--balance` / `--send`
- [ ] `aether status` — show node health, rewards, connected peers
- [ ] Genesis CLI: `aether genesis --init --validators <n>`

**Deliverable:** User can download binary, run `aether genesis`, spin up 3 nodes, see them produce blocks.

### Phase 0D — Real P2P Networking

- [ ] Replace simulated gossip with real libp2p block sync
- [ ] Implement `aether-consensus` `run_gossip()` with actual block propagation
- [ ] Node discovery (Kademlia or DNS-based bootstrap)
- [ ] Multi-node test: 2-3 laptops on same network producing blocks

**Deliverable:** Blocks propagate between real nodes on different machines.

---

## Technical Notes

### Workspace Target Structure

```
crates/
├── aether-common/      ✅ exists, needs completion
├── aether-consensus/    ✅ exists, good code
├── aether-core/         ❌ empty, needs content or removal
├── aether-governance/   ❌ empty
├── aether-privacy/      ❌ empty
├── aether-ai-priority/ ✅ stub with module declarations
├── aether-mobile/       ✅ stub
├── aether-validator/    ✅ binary, needs integration
└── aether-runtime/      🆕 new — contract execution engine
```

### Key Dependencies to Add to Workspace

```toml
# For persistent state
rocksdb = "0.22"

# For RPC
jsonrpsee = "0.23"

# For serialization  
borsh = "1.6"
```

### Consensus Integration Path

1. `aether-validator main.rs` calls `AetherFlow::new()`
2. `AetherFlow::produce_block()` calls `aether-runtime` to execute txs
3. Runtime reads/writes state via RocksDB
4. P2P layer gossip propagates blocks using `libp2p::gossipsub`

---

## Definition of Done

A testnet is "done" for Phase 0 when:

1. `cargo build --release` produces a binary that a non-technical user can run
2. `aether genesis --validators 3` creates a 3-node testnet
3. All 3 nodes produce and propagate real blocks (verifiable on each node's RPC)
4. A transaction submitted via CLI (e.g. token transfer) is included in a block and persists
5. `cargo clippy --workspace -D warnings` passes clean
6. `cargo test --workspace` passes clean

---

## Priority

**Critical path for all subsequent work.** Nothing else matters if the chain doesn't run.

---

*This issue exists to give the autonomous agent team clear, scoped, testable deliverables. Progress is tracked in the Testnet milestone.*
