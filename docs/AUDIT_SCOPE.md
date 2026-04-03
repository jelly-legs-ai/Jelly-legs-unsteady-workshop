# Security Audit Scope — Project AETHER

**Version:** 1.0  
**Date:** 2026-04-03  
**Project:** AETHER Blockchain (Solana-fork) — Mainnet Phase 6  
**Budget:** $1-2K (automated tooling + focused freelance review)  

---

## 1. Scope

### In Scope

| Component | Language | Priority | Description |
|-----------|----------|----------|-------------|
| `aether-staking` contract | Rust | **P0** | Staking, tier rewards, auto-compound, slashing |
| `aether-mining` contract | Rust | **P0** | Mining rewards, streak bonuses, geo diversity |
| `aether-token` (FLUX/ATH) | Rust | **P0** | Token transfers, mint, burn, fee mechanisms |
| `aether-governance` contract | Rust | **P1** | Timelock, multi-sig treasury, conviction voting |
| `aether-bridge` contract | Rust | **P1** | Cross-chain, liquidity pools |
| `aether-validator` binary | Rust | **P1** | Consensus, P2P gossip, RPC |
| Consensus mechanism | Rust | **P1** | AetherFlow PoH+PoS hybrid |

### Out of Scope
- Frontend/web UI code
- Testnet infrastructure
- CI/CD tooling (GitHub Actions)
- Third-party dependencies (libp2p, tokio, etc.) — covered by `cargo audit`

---

## 2. Audit Approach

### Tier 1: Automated Analysis (Free)
- `cargo audit` — dependency CVE scan
- `cargo clippy --workspace` — Rust linting (all targets)
- `cargo fmt --check` — style enforcement
- `cargo test --workspace` — unit test suite
- Manual code review of critical paths (staking reward calculation, token minting)

### Tier 2: Community Review (Free — Immunefi)
- Public bug bounty via Immunefi
- Severity matrix: Critical/High/Medium/Low
- Reward range: $500 (Low) → $5,000 (Critical)

### Tier 3: Focused Freelance Review (~$500-1K)
- One senior Rust/security engineer
- 1-week engagement
- Scope: staking + mining contracts only
- Deliverable: written finding report

---

## 3. Critical Code Paths

### Staking Rewards (`aether-staking`)
```
calculate_staking_rewards()
calculate_delegation_rewards()
calculate_early_withdrawal_penalty()
```
Risk: Reward calculation errors → inflation or theft

### Token Minting (`aether-token`)
```
mint()
burn()
transfer()
```
Risk: Unchecked arithmetic → integer overflow/underflow

### Mining Rewards (`aether-mining`)
```
distribute_epoch_rewards()
calculate_bonus_multipliers()
```
Risk: Bonus calculation manipulation

---

## 4. Severity Matrix (Immunefi)

| Severity | Definition | Reward |
|----------|------------|--------|
| Critical | Fund theft, chain consensus break, infinite mint | $5,000 |
| High | Reward calculation error >1%, unauthorized actions | $2,000 |
| Medium | Griefing, inefficient gas paths | $500 |
| Low | Informational, code quality | $100 |

---

## 5. Known Issues

- `lru 0.12.5` soundness advisory (RUSTSEC-2026-0002) — transitive via libp2p, non-critical path
- `paste 1.0.15` unmaintained — transitive via netlink, non-critical

---

## 6. Timeline

| Date | Milestone |
|------|-----------|
| 2026-04-03 | This document + Immunefi setup |
| 2026-04-04 | Freelance reviewer engaged |
| 2026-04-05 | Week 2 deadline: audit commissioned |
| 2026-04-12 | Freelance findings delivered |
| 2026-04-19 | All P0/P1 fixes verified |
