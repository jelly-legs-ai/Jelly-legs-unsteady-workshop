# Project AETHER — Mainnet Architecture Document
## Phase 6 Deliverable 1/4

**Status:** DRAFT — Pending Validator Feedback  
**Author:** 🚀 Launch-Pad (Phase 6 Mainnet Specialist)  
**Date:** 2026-03-22  
**Issue:** [#108](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/108)

---

## 1. Overview

Project AETHER mainnet is a Solana-derived blockchain optimized for mobile mining and AI workloads. It uses a hybrid Proof of History + Proof of Stake consensus mechanism branded **AetherFlow**.

**Codename:** AETHER (Autonomous Ecosystem for Transparent, High-Efficiency, Encrypted, Resilient operations)  
**Native Token:** AETH  
**Secondary Token:** FLUX (AI operation fee token)

---

## 2. Chain Identity

| Parameter | Value | Notes |
|-----------|-------|-------|
| Chain ID | `aether-mainnet-1` | Human-readable identifier |
| Network Magic | `0x41455448` ("AETH") | Fork ID discrimination |
| RPC Port | 8899 | Primary RPC |
| P2P Port | 8001 | Validator gossip |
| WebSocket | 8900 | Subscription-based events |
| Genesis Hash | TBD (computed at genesis) | Pre-generate and hardcode |
| AETH Decimals | 9 | Same as SOL |
| FLUX Decimals | 6 | AI operations |

---

## 3. Consensus Parameters — AetherFlow

### 3.1 Proof of History (PoH)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Slot Time | 400ms | Target slot duration |
| PoH Tick Duration | 10ms | Hash computation quantum |
| Hash Function | Blake3 | Fast, parallelizable |
| PoH Records | Transactions + timestamps | Verifiable ordering |
| VDF Difficulty | Adaptive | Auto-tuned for 400ms target |

### 3.2 Proof of Stake (PoS)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Minimum Stake | 100 AETH | To be a active validator |
| Minimum Delegation | 10 AETH | For non-validator delegators |
| Epoch Length | 432,000 slots (~48 hours) | ~1 epoch per 2 days |
| Consensus Threshold | 67% stake + 1 | Fork choice finality |
| Validator Set Refresh | Every epoch | At epoch boundary |

### 3.3 Tower BFT (Fork Choice)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Finality | 12 confirmed votes | ~4.8s with 400ms slots |
| Tower Height | Locked stake × confirmations | Optimistic rollup style |
| Rollback Limit | 12 slots | Beyond this, finality is economic |
| Expected Finality | 4.8–6.4s | 12–16 slots |

---

## 4. Tokenomics

### 4.1 AETH Token

| Parameter | Value | Notes |
|-----------|-------|-------|
| Total Supply | 500,000,000 AETH (500M) | Fixed, non-inflationary |
| Initial Circulation | ~30% at TGE | Community, validators, treasury |
| Decimals | 9 | 1 AETH = 10⁹ lamports |
| Fee Burn | 50% of tx fees burned | Deflationary pressure |
| Staking Emissions | From 50% tx fees + treasury top-up | ~5–8% APY initially |

### 4.2 FLUX Token

| Parameter | Value | Notes |
|-----------|-------|-------|
| Total Supply | 10,000,000,000 FLUX (10B) | AI operations only |
| Max AI Ops Fee | 1 FLUX per inference | At genesis |
| Mint Authority | Protocol-controlled | Adjustable via governance |
| Decimals | 6 | FLUX = 10⁶ units |

### 4.3 Token Distribution at TGE

| Allocation | Amount | % | Vesting |
|-----------|--------|---|---------|
| Founding Validators | 50,000,000 AETH | 10% | 12mo cliff, 24mo linear |
| Early Supporters | 25,000,000 AETH | 5% | 6mo cliff, 18mo linear |
| Treasury (DAO) | 150,000,000 AETH | 30% | Governance-controlled |
| Ecosystem/LP | 75,000,000 AETH | 15% | 24mo linear |
| Team | 75,000,000 AETH | 15% | 12mo cliff, 36mo linear |
| Public Sale | 75,000,000 AETH | 15% | TGE + 12mo unlock |
| Airdrop | 50,000,000 AETH | 10% | TGE + 3mo linear |

---

## 5. Founding Validator Program

Details from `founding_validators.rs`:

| Parameter | Value |
|-----------|-------|
| Required Stake | 10,000 AETH |
| Multiplier | 2.0x (double rewards during bootstrap) |
| Bootstrap Period | 12 months from genesis |
| Minimum Uptime | 95% required for multiplier |
| Count | Open (no hard cap, minimum viable: 10) |

**Genesis Validators:** Minimum 10 confirmed validators required before genesis.

---

## 6. Transaction Lanes & Fees

### 6.1 AI Priority Lanes (from `aether-ai-lanes`)

| Lane | Priority | Fee Multiplier | Target |
|------|----------|----------------|--------|
| Lane 0 — Critical | Highest | 10x base | AI agent settlement, oracle |
| Lane 1 — Standard | Normal | 1x base | Regular transfers, staking |
| Lane 2 — Background | Lowest | 0.1x base | Non-urgent, batchable |

### 6.2 Base Fees

| Transaction Type | Estimated Fee |
|-----------------|---------------|
| Transfer (AETH) | 0.00001 AETH |
| Stake/Delegate | 0.0001 AETH |
| AI Inference Request | 0.001 AETH + FLUX |
| Smart Contract Deploy | 0.01 AETH |

---

## 7. Genesis Configuration

```
Genesis Timestamp:  TBD — target Q2 2026
Genesis Hash:       Pre-generated, hardcoded in source
Initial Validators: ≥10 founding validators
Initial Supply:     500,000,000 AETH (all pre-minted)
PoH Generator:      Leader initializes with timestamp 0
```

**Genesis Block Contents:**
- Pre-mint all AETH tokens per distribution table
- Initialize staking accounts for founding validators
- Set initial epoch = 0
- Initialize AI priority lane weights
- Set FLUX token metadata

---

## 8. Network Peering

| Parameter | Value |
|-----------|-------|
| P2P Protocol | QUIC (Solana's net板块) |
| UDP for PoH gossip | Enabled |
| Min peers | 5 |
| Max peers | 200 |
| Bootstrap nodes | Hardcoded relay list |
| Gossip interval | 100ms |

---

## 9. Important Hardforks / Known Gaps

These items are **out of scope for v1.0** but planned:

| Feature | Target |
|---------|--------|
| zk-SNARKs privacy | v2.0 |
| Cross-chain bridges | v2.0 |
| AI inference on-chain | v1.5 |
| Governance DAO activation | 30 days post-genesis |
| Quantum-resistant crypto | v3.0 |

---

## 10. RPC & Explorer Infrastructure

| Service | Requirement |
|---------|------------|
| Public RPC | At least 3 geo-distributed nodes |
| Archive RPC | Full historical state (for explorers) |
| Block Explorer | Custom AETHER explorer or fork of Solscan |
| Indexer | AETHER-specific token indexing |

---

## 11. Next Steps

- [ ] Finalize genesis timestamp with founding validators
- [ ] Generate and distribute genesis config JSON
- [ ] Verify all token distribution math
- [ ] Confirm chain ID with community
- [ ] Set FLUX initial supply and mint

---

*Document version: 1.0 — 🚀 Launch-Pad | Phase 6*
