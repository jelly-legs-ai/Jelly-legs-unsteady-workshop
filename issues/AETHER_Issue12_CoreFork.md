---
title: "[DEV] Sprint 1-2: Core Fork Implementation"
labels: ["development", "sprint1-2", "critical", "consensus", "infrastructure"]
assignees: []
milestone: "Phase 3 - Development"
---

## 📋 Issue #12: Core Fork Implementation

**Parent:** #11 (EPIC: Project AETHER)  
**Priority:** 🔴 Critical  
**Duration:** 4 weeks (Sprint 1-2)  
**Team:** Protocol Team  
**Dependencies:** None (first development sprint)

---

### 🎯 Objective

Fork the Solana Agave validator v1.18.x and implement AETHER-specific modifications to create a functioning devnet with native AETH token support.

---

### 📚 Reference Documentation

- **Phase 1 Research:** `AETHER_Research_Report_Phase1.md`
- **Phase 2 Design:** `AETHER_Design_Specification_Phase2.md` (Section 3.1-3.3, 7.2)
- **Agave Repository:** https://github.com/anza-xyz/agave

---

### 📝 Tasks

#### Week 1: Repository Setup & Fork

- [ ] **1.1** Fork Agave v1.18.x from Anza repository
  - Create `aether-core` repository under jelly-legs-ai org
  - Preserve git history for attribution
  - Tag as `aether-fork-v1.18.0`

- [ ] **1.2** Remove Solana mainnet-specific code
  - Remove mainnet genesis configuration
  - Strip Foundation-specific programs
  - Remove mainnet feature gates
  - Clean up validator bootstrap code

- [ ] **1.3** Setup development environment
  - Rust toolchain setup (v1.75.0+)
  - Cargo workspace configuration
  - Development dependencies
  - Build scripts

#### Week 2: AETH Token Implementation

- [ ] **2.1** Implement AETH native token program
  - Based on SPL Token with modifications
  - Deflationary burn mechanisms (50% of fees)
  - Token metadata initialization

- [ ] **2.2** Configure token parameters
  ```rust
  Initial Supply: 1_000_000_000 AETH
  Decimals: 9
  Burn Rate: 5000 basis points (50% of base fees)
  Min Priority Fee Burn: 100%
  ```

- [ ] **2.3** Create token distribution
  - Community & Ecosystem: 35%
  - Core Contributors: 25%
  - Staking: 20%
  - Treasury: 15%
  - Strategic Partners: 5%

#### Week 3: Genesis & Configuration

- [ ] **3.1** Create genesis block configuration
  - Genesis timestamp
  - Initial validator set (10 validators for devnet)
  - Initial token allocations
  - Genesis programs

- [ ] **3.2** Configure devnet parameters
  ```rust
  Target Slot Time: 400ms
  Epoch Duration: 8192 slots (~54 minutes)
  Inflation Rate: 0% (deflationary)
  Minimum Stake: 1000 AETH
  Consensus Mode: PoH + Tower BFT
  ```

- [ ] **3.3** Setup genesis accounts
  - AETH mint account
  - Stake program accounts
  - Governance program placeholder
  - System program

#### Week 4: CI/CD & Devnet Deployment

- [ ] **4.1** Implement CI/CD pipeline
  - GitHub Actions workflows
  - Build matrix (Linux, macOS)
  - Automated testing
  - Docker image builds

- [ ] **4.2** Create deployment scripts
  - Devnet validator deployment
  - Genesis ceremony automation
  - Faucet setup
  - Monitoring integration

- [ ] **4.3** Deploy devnet
  - Launch 10 validator nodes
  - Bootstrap network
  - Verify consensus
  - Test basic transactions

---

### 🔧 Technical Specifications

#### Hardware Requirements (Devnet)

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 8 cores | 16 cores |
| RAM | 32GB | 64GB |
| Storage | 500GB SSD | 1TB NVMe |
| Network | 100 Mbps | 1 Gbps |

#### Dependencies

```toml
[dependencies]
# Core Agave dependencies
solana-sdk = "1.18"
solana-program = "1.18"
solana-ledger = "1.18"
solana-core = "1.18"

# AETHER-specific
aether-token = { path = "../programs/aether-token" }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
```

---

### ✅ Acceptance Criteria

- [ ] Devnet running with 10+ validators
- [ ] Block production successful (400ms avg block time)
- [ ] Basic AETH token transfers functional
- [ ] Genesis configuration matches specification
- [ ] CI/CD pipeline passing
- [ ] Documentation complete

---

### 🧪 Testing Requirements

- [ ] Unit tests for token program
- [ ] Integration tests for transfers
- [ ] Consensus test: 10 validators, 1000 blocks
- [ ] Genesis validation
- [ ] Faucet functionality test

---

### 📊 Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Block Time | 400ms ± 100ms | Prometheus metrics |
| TPS | 10,000+ | Benchmark suite |
| Validator Uptime | 99% | Monitoring dashboard |
| Build Time | <15 minutes | CI pipeline |

---

### ⚠️ Risks & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Build system complexity | High | Medium | Document thoroughly, pair programming |
| Agave version conflicts | Medium | High | Pin exact versions, lock files |
| Genesis misconfiguration | Low | Critical | Multiple reviews, test on small scale |
| Devnet instability | Medium | Medium | Monitoring, auto-restart |

---

### 🔗 Related Issues

- **Blocks:** #13 (Privacy Integration) - needs devnet for testing
- **Related:** #14 (Governance), #15 (Security Testing)

---

### 📝 Notes

- Keep Solana compatibility where possible for tooling
- Document all deviations from Agave
- Maintain clean git history for upstream merges
- Consider Alpenglow migration path in design

---

/estimate 4w
/label ~"sprint 1-2" ~"core fork" ~devnet