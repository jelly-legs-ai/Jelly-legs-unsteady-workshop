# Project AETHER — Security Audit Planning
## Phase 6 Deliverable 2/4

**Status:** PLANNING  
**Author:** 🚀 Launch-Pad (Phase 6 Mainnet Specialist)  
**Date:** 2026-03-22  
**Issue:** [#108](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/108)

---

## 1. Audit Philosophy

The 2025–2026 crypto landscape has cost $3.4B+ to attackers. **Access control exploits drove 59% of losses**, reentrancy remains #1 on OWASP's Smart Contract Top 10, and AI-related exploits surged 1,025% vs 2023. Project AETHER's code is complex — hybrid consensus, mobile mining, AI priority lanes — so a rigorous multi-auditor approach is non-negotiable.

**Guiding principle:** We want auditors to find things. That's the point.

---

## 2. Security Review Scope

### 2.1 Rust Core (Highest Priority)

| Module | Files | Risk Level | Focus |
|--------|-------|-----------|-------|
| `aether-core/src/crypto.rs` | 1 | **P0 — Critical** | Key generation, signatures, Ed25519 impl |
| `aether-core/src/hybrid_consensus.rs` | 1 | **P0 — Critical** | Fork choice, vote handling, finality |
| `aether-core/src/anti_gaming.rs` | 1 | **P0 — Critical** | Sybil resistance, mobile device scoring |
| `aether-core/src/founding_validators.rs` | 1 | **P0 — Critical** | Stake locking, multiplier logic |
| `aether-core/src/reward.rs` | 1 | **P1 — High** | Emission math, rounding exploits |
| `aether-core/src/validator.rs` | 1 | **P1 — High** | Slashing conditions, stake states |
| `aether-core/src/proof_engine.rs` | 1 | **P1 — High** | Mobile PoW validation, difficulty adjustment |

### 2.2 Consensus (High Priority)

| Module | Risk Level | Focus |
|--------|-----------|-------|
| `crates/aether-consensus/src/pos.rs` | **P0 — Critical** | Stake weighting, leader election |
| `crates/aether-consensus/src/poh.rs` | **P1 — High** | VDF soundness, timing attacks |
| `crates/aether-consensus/src/aetherflow.rs` | **P0 — Critical** | Cross-layer interaction, race conditions |
| `crates/aether-consensus/src/tower.rs` | **P1 — High** | Vote locking, rollback limits |
| `crates/aether-consensus/src/fork_choice.rs` | **P0 — Critical** | Long-range attack resistance |
| `crates/aether-consensus/src/validator.rs` | **P1 — High** | Validator set transitions |

### 2.3 AI & Networking (Medium Priority)

| Module | Risk Level | Focus |
|--------|-----------|-------|
| `crates/aether-ai-priority/src/` | **P1 — High** | Lane manipulation, priority flooding |
| `aether-network/src/gossip.rs` | **P1 — High** | Eclipse attacks, gossip amplification |
| `aether-network/src/propagation.rs` | **P2 — Medium** | Block propagation delays, DoS |

### 2.4 Smart Contracts (If deployed on-chain)

| Contract | Risk Level | Focus |
|---------|-----------|-------|
| `aeth_token.sol` | **P0 — Critical** | Standard ERC20, minting authority |
| `flux_token.sol` | **P1 — High** | AI fee token, limited supply |

---

## 3. Recommended Audit Firms

### Tier 1 — Full Consensus Audit

| Firm | Specialty | Estimated Cost | Timeline |
|------|-----------|---------------|---------|
| **Trail of Bits** | Rust + blockchain | $40,000–$80,000 | 3–4 weeks |
| **Zellic** | Rust consensus | $30,000–$60,000 | 2–4 weeks |
| **Sigma Prime** | Distributed systems | $35,000–$70,000 | 3–4 weeks |

### Tier 2 — Smart Contract Audit

| Firm | Specialty | Estimated Cost | Timeline |
|------|-----------|---------------|---------|
| **OpenZeppelin** | Solidity contracts | $15,000–$30,000 | 2–3 weeks |
| **Cyfrin** | Solidity + Rust | $20,000–$40,000 | 2–4 weeks |

### Tier 3 — Specialized Reviews

| Focus | Firm/Resource | Estimated Cost | Timeline |
|-------|--------------|---------------|---------|
| Cryptographic review | NCC Group Crypto | $20,000–$40,000 | 2 weeks |
| Formal verification | Runtime Verification | $50,000+ | 4–8 weeks |
| Mobile PoW analysis | Independent researcher | $5,000–$15,000 | 1–2 weeks |

**Recommended approach:** Do **Trail of Bits** or **Zellic** for Rust core (non-negotiable for consensus-critical code), plus **OpenZeppelin** for smart contracts.

---

## 4. Audit Timeline

### Phase A: Pre-Audit (2 weeks)

| Week | Action | Owner |
|------|--------|-------|
| 1 | Freeze codebase — no new features, bug fixes only | Developer |
| 1 | Prepare public GitHub repo with scope document | Developer |
| 1 | Internal red-team: try to break everything | Cybersecurity |
| 2 | Run automated tools: `cargo audit`, `clippy`, `miri` | Developer |
| 2 | Prepare audit brief: architecture, threat model, prior issues | All |
| 2 | Sign engagement letter with chosen firm | Lead |
| 2 | **Immunefi bug bounty program launched** | Deployment |

### Phase B: Auditor Review (3–4 weeks)

| Week | Action | Owner |
|------|--------|-------|
| 1–2 | Auditor does initial review, asks questions | Auditor |
| 2 | Weekly sync calls — triage findings in real time | Lead + Auditor |
| 3–4 | Draft report received | Auditor |
| 4 | Joint findings triage — P0/P1/P2 classification | All |

### Phase C: Fixes & Verification (2–3 weeks)

| Week | Action | Owner |
|------|--------|-------|
| 1 | Fix all P0 and P1 findings | Developer |
| 1 | Re-audit of fixed code | Cybersecurity |
| 2 | Fix P2 findings | Developer |
| 2 | Auditor re-verifies critical fixes | Auditor |
| 3 | Final report published | Auditor |

### Phase D: Post-Audit (Ongoing)

| Item | When |
|------|------|
| Bug bounty program live on Immunefi | Before genesis |
| Public audit report published | After P0/P1 fixes |
| Continuous fuzzing on public code | Ongoing |
| Quarterly re-audits of new features | Quarterly |

---

## 5. Prior Known Issues (from Phase 4)

Phase 4 applied fixes for P0–P2 issues. These should be re-verified:

| Issue | Severity | Status | Re-verify? |
|-------|----------|--------|-----------|
| Access control on admin functions | P0 | Fixed | Yes |
| Reentrancy in token transfers | P1 | Fixed | Yes |
| Integer overflow in reward calculation | P1 | Fixed | Yes |
| Timing oracle in PoW difficulty | P1 | Fixed | Yes |
| Slashing condition edge cases | P2 | Fixed | Yes |

---

## 6. Immunefi Bug Bounty Program

**Launch immediately** on [Immunefi](https://immunefi.com/) — critical before genesis.

| Tier | Severity | Payout |
|------|----------|--------|
| Critical | Smart contract + consensus failure | $50,000–$250,000 |
| High | Major exploit, fund drain | $10,000–$50,000 |
| Medium | Moderate exploit, DoS | $1,000–$10,000 |
| Low | Minor issues | $100–$1,000 |

**Scope for bug bounty:**
- All Rust core modules
- Smart contracts (`aeth_token.sol`, `flux_token.sol`)
- RPC endpoints
- **NOT in scope:** Frontend, CI/CD, social media

---

## 7. Internal Security Checklist

Before genesis, the following must all pass:

- [ ] All automated tools pass (`cargo audit` clean, no new vulnerabilities)
- [ ] P0 findings from external audit are fixed and verified
- [ ] At least one internal red-team exercise completed
- [ ] Bug bounty program live on Immunefi
- [ ] Key management: multi-sig on all admin functions, timelock 48h+
- [ ] No single point of failure for validator operations
- [ ] Emergency pause mechanism tested (for token contracts)
- [ ] Disaster recovery plan documented and tested
- [ ] Incident response contacts and procedures ready

---

## 8. Security Monitoring Post-Launch

| Tool | Purpose | Cost |
|------|---------|------|
| Tenderly | Transaction monitoring + alerts | ~$500/mo |
| OpenZeppelin Defender | Admin action monitoring | Free tier available |
| Custom Grafana + PagerDuty | Validator uptime + anomaly detection | Self-hosted |
| Chainalysis / Merkle Science | Compliance + fund tracking | ~$2,000/mo |

---

*Document version: 1.0 — 🚀 Launch-Pad | Phase 6*
