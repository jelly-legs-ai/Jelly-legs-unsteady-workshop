## 🎨 DESIGN COMMENT 3/5: Validator Network Design

### Full Validators vs Mobile Nodes

| Aspect | Mobile Nodes | Full Validators |
|--------|--------------|-----------------|
| Role | Work submission layer | Consensus + finality |
| Chain data | None (light sync only) | Full copy of ledger |
| Compute | Benchmark proofs | Block validation + ZK verification |
| Storage | Local only (~50MB app) | 500GB+ SSD required |
| Connectivity | Intermittent (mobile) | Always-on (dedicated) |
| Hardware | Consumer device | Server-grade (minimum) |
| Uptime requirement | None (rewards tiered) | 95%+ (consensus duty) |

### Minimum Validator Requirements

**Hardware:**
- CPU: 16+ cores (AMD Ryzen 9 / Intel Xeon)
- RAM: 128GB ECC
- Storage: 2TB+ NVMe SSD (ledger grows ~1TB/year)
- Network: 1Gbps symmetric, unmetered
- Redundancy: UPS + dual ISP recommended

**Economic Stake:**
- Minimum stake: 50,000 $AETH (liquid) + 25,000 $AETH (locked for 6 months)
- Total skin-in-game: 75,000 $AETH
- Projected annual yield: 8-12% on staked $AETH

**Operational:**
- Data center colocation or bare metal
- 24/7 monitoring + on-call response
- Automated failover configurations
- Security: HSM for validator keys, no cloud VPN exposure

### Validator Reward Structure

```
ValidatorEarnings = BlockRewards + TransactionFees + StakingYield

BlockRewards (per slot won):
- Base: 0.01 $AETH per confirmed slot
- Leader bonus: 2x for being block leader
-爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷爷

TransactionFees:
- Priority fees: 80% to validator, 20% to treasury
- Compute unit fees: 90% to validator (execution), 10% to treasury

StakingYield:
- Annual: 8-12% on locked stake (adjusted quarterly by governance)
- Derived from: inflation (5% Year 1) + network activity fees

ValidatorSlashPenalties:
- Downtime: 0.1% of stake per hour offline
- Double sign: 5% of stake + 24h ejection
- Invalid block: 1% of stake per violation
```

### Permissioned vs Permissionless

**Phase 1 (Testnet):** Permissioned validators only
- 20-50 hand-selected validators from founding team
- Controlled environment for debugging
- No slashing risk during testnet

**Phase 2 (Early Mainnet):** Gradual permissionlessness
- Allow permissioned entry (application + KYC for entities)
- Maximum 100 validators initially
- Slow onboarding with 6-month lockup

**Phase 3 (Mature Mainnet):** Full permissionlessness
- Anyone meeting hardware + stake requirements can validate
- Permissionless entry but stake-based priority for leader selection
- Gradual target: 500-1000 validators

**Leader Selection:**
- Based on stake weight + VRF (Verifiable Random Function)
- Rotation prevents any single validator dominating
- Historical performance factored into selection probability

### Validator-Node Communication

- Gossip protocol for block propagation (like Solana's Turbine)
- Mobile nodes submit to nearest validator relay (discoverable via DNS)
- Validators aggregate mobile proofs via ZK-rollup circuit
- Light clients sync via proofs, not full blocks