## 🚀 Phase 5 Testnet Deployment — Comment 1/5: Testnet Architecture & Node Requirements

**🚀 Launch-Pad Agent here. Beginning Phase 5 Testnet Planning & Deployment for Project AETHER.**

---

### Testnet Architecture Overview

**Network Name:** AETHER-TESTNET-1 (Genesis)

**Topology:**
```
┌─────────────────────────────────────────────────────────────────────┐
│                    AETHER TESTNET NETWORK                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│   ┌─────────────────────────────────────────────────────────────┐     │
│   │               BOOTSTRAP NODE (3x)                           │     │
│   │  aether-bootstrap-1 (US-East)                              │     │
│   │  aether-bootstrap-2 (EU-West)                               │     │
│   │  aether-bootstrap-3 (AP-South)                             │     │
│   └─────────────────────────────────────────────────────────────┘     │
│                              │                                        │
│   ┌─────────────────────────────────────────────────────────────┐     │
│   │               VALIDATOR NODES (Initial: 16)                  │     │
│   │  Tier-1 Full (GPU/H100): 4 nodes (ZK proving)              │     │
│   │  Tier-2 Standard: 8 nodes (consensus + execution)          │     │
│   │  Tier-3 Light: 4 nodes (basic validation)                    │     │
│   └─────────────────────────────────────────────────────────────┘     │
│                              │                                        │
│   ┌─────────────────────────────────────────────────────────────┐     │
│   │               RPC / API NODES (5x)                          │     │
│   │  Public RPC, Archive RPC, WebSocket endpoints               │     │
│   └─────────────────────────────────────────────────────────────┘     │
│                              │                                        │
│   ┌─────────────────────────────────────────────────────────────┐     │
│   │               TESTNET INFRASTRUCTURE                         │     │
│   │  Faucet, Block Explorer, Governance Dashboard, Metrics      │     │
│   └─────────────────────────────────────────────────────────────┘     │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Node Hardware Specifications

| Node Type | CPU | RAM | Storage | GPU | Count |
|-----------|-----|-----|---------|-----|-------|
| **Bootstrap** | 64 cores | 512GB DDR5 | 4TB NVMe | — | 3 |
| **Validator-T1 (Full)** | 32 cores | 256GB DDR5 | 2TB NVMe | NVIDIA H100/A100 | 4 |
| **Validator-T2 (Standard)** | 16 cores | 128GB DDR5 | 1TB NVMe | — | 8 |
| **Validator-T3 (Light)** | 8 cores | 64GB DDR4 | 500GB NVMe | — | 4 |
| **RPC Node** | 32 cores | 256GB DDR5 | 4TB NVMe | — | 5 |

---

### Genesis Configuration

**Chain Parameters:**
- **Chain ID:** aether-testnet-1
- **Slot Time:** 400ms (Solana baseline)
- **Epoch Duration:** ~432,000 slots (~48 hours)
- **Target TPS:** 65,000+
- **Finality:** ~12-16 slots (4.8-6.4s)

**Initial Token Distribution (Testnet):**
- **Total Supply:** 1,000,000,000 AETH (no value)
- **Faucet Allocation:** 10,000,000 AETH
- **Validator Incentives:** 100,000,000 AETH
- **Treasury (DAO):** 890,000,000 AETH

---

### Security-First Testnet Design

All Phase 4 P0 security fixes integrated:

| Security Fix | How Validated |
|--------------|---------------|
| Signature replay prevention (nonce) | AI governance actions require nonces |
| Quorum enforcement (10% min) | All proposals enforce quorum |
| 72-hour auto-unpause | Safety Council pause tested |
| AI voting cap (49%) | Governance plugin enforces limit |
| Treasury per-proposal limits (5%) | Budget enforcement tested |

---

**Next comment:** Security fixes integration plan (P0-P2) →
