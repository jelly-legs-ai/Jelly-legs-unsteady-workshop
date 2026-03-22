# AETHER - Mobile Mining Blockchain

> A Solana-forked Layer 1 blockchain purpose-built for AI agent coordination, privacy-preserving computation, and mobile-native mining.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Phase](https://img.shields.io/badge/phase-Design%20v2.0-orange)](./AETHER_Design_Specification_Phase2.md)
[![TPS](https://img.shields.io/badge/tps-65%2C000%2B-green)](./AETHER_Design_Specification_Phase2.md#13-performance-targets--benchmarks)

---

## 🎯 Project Overview

AETHER is a high-performance blockchain designed from the ground up for the AI era. It combines Solana's battle-tested performance with native support for zero-knowledge proofs, AI agent governance, and mobile-device mining.

### Key Features

| Feature | Description |
|---------|-------------|
| **65K+ TPS** | Parallel transaction execution via Sealevel runtime |
| **<1.5s Finality** | Optimistic confirmation via Tower BFT |
| **Privacy-First** | Groth16 + STARK hybrid zk-SNARKs at protocol level |
| **AI Agent Governance** | On-chain AI registry with delegated voting |
| **Mobile Mining** | Proof-of-Useful-Work for mobile devices |
| **Deflationary Token** | 50% fee burn + programmatic token burns |

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AETHER BLOCKCHAIN                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  LAYER 5: APPLICATION                                                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │   dApps      │ │  AI Agents   │ │   Wallets   │ │  Governance UI     │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  LAYER 4: API                                                                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │  REST API    │ │  GraphQL     │ │  WebSocket  │ │  gRPC (Internal)   │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  LAYER 3: SMART CONTRACTS                                                     │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │  AETH Token  │ │  Governance  │ │ AI Registry  │ │   ZK Programs      │  │
│  │   (SPL)      │ │  Contracts   │ │   (SPL)      │ │  (Native Rust)      │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  LAYER 2: RUNTIME                                                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │   Sealevel   │ │  Gulf Stream │ │     SVM      │ │   ZK Verifier       │  │
│  │  (Parallel    │ │  (Mempool-  │ │  (AETHER     │ │  (Groth16/STARK)    │  │
│  │  Execution)   │ │  less Txs)  │ │  Variant)    │ │                     │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  LAYER 1: CONSENSUS                                                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │    PoH       │ │  Tower BFT   │ │     PoS      │ │  Leader Election   │  │
│  │  (History)   │ │ (Optimistic  │ │   Staking    │ │  (AI-Optimized)    │  │
│  │              │ │  Confirm)    │ │    Pool      │ │                     │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                    │                                        │
│                                    ▼                                        │
│  LAYER 0: NETWORK                                                            │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │   Turbine    │ │   Gossip     │ │   Repair     │ │   Archiver         │  │
│  │ (Block Prop) │ │  Protocol    │ │   Service    │ │   Network          │  │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+ ([Install](https://rustup.rs/))
- **Solana CLI** v1.16+ ([Install](https://docs.solana.com/cli/install-solana-cli-tools))
- **Node.js** 18+ (for client-side tooling)
- **Git**

### 3-Step Setup

```bash
# Step 1: Clone the monorepo
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop.git
cd Jelly-legs-unsteady-workshop

# Step 2: Build the core crate
cd aether-core
cargo build --release

# Step 3: Run a local devnet
cargo run --release --bin aether-validator -- --mode devnet
```

That's it! Your validator will:
1. Generate a genesis block
2. Start the PoH generator
3. Begin producing blocks

---

## 📁 Directory Structure

```
aether-core/
├── README.md                          # ← You are here
├── Cargo.toml                         # Workspace manifest
├── config/
│   └── validator.yaml                 # Validator configuration
├── src/
│   ├── lib.rs                         # Library entry point
│   ├── anti_gaming.rs                 # MEV & sandwich attack mitigation
│   ├── crypto.rs                      # Cryptographic primitives
│   ├── error.rs                       # Error types
│   ├── proof_engine.rs                # ZK proof verification engine
│   ├── reward.rs                      # Staking & reward distribution
│   └── types.rs                       # Core type definitions
└── programs/
    ├── aeth_token.sol                 # AETH SPL-compatible token
    └── compute_token.sol              # Compute credit token
```

### Related Components

| Component | Directory | Purpose |
|-----------|-----------|---------|
| **AI-Lanes** | `../aether-ai-lanes/` | AI model routing & priority lanes |
| **Consensus** | `../aether-consensus/` | PoH + PoS hybrid consensus |
| **Mobile** | `../aether-mobile/` | Mobile mining client |
| **Network** | `../aether-network/` | P2P gossip & block propagation |
| **PoH** | `../aether-poh/` | Proof of History generator |
| **Storage** | `../aether-storage/` | Blockstore & state archive |

---

## 📚 Component Documentation

### Core Components

| Document | Description |
|----------|-------------|
| **[Design Specification (v2.0)](./AETHER_Design_Specification_Phase2.md)** | Full Phase 2 design — architecture, tokenomics, governance, privacy circuits |
| **[AI-Lanes](../aether-ai-lanes/src/lib.rs)** | AI workload prioritization & lane isolation |
| **[Consensus](../aether-consensus/src/lib.rs)** | Hybrid PoH/PoS consensus with AI-optimized leader election |
| **[Network](../aether-network/src/lib.rs)** | Gossip protocol & Turbine block propagation |
| **[PoH](../aether-poh/src/lib.rs)** | Cryptographic clock & transaction ordering |
| **[Storage](../aether-storage/src/lib.rs)** | AccountDB state management & archival storage |

### Architecture Docs

| Document | Description |
|----------|-------------|
| **[Mobile Architecture](../aether-mobile/ARCHITECTURE.md)** | Mobile mining client design |
| **[Phase 1 Research](../Jelly-legs-unsteady-workshop/research/)** | Foundational research & design decisions |

---

## 🏗️ Development

### Building

```bash
# Full release build
cargo build --release

# Development build (faster compile)
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --bin aether-validator
```

### Configuration

Edit `config/validator.yaml` to configure your node:

```yaml
validator:
  identity: validator-keypair.json
  vote_account: vote-account.json
  
consensus:
  pow_enabled: true
  tower_bft_threshold: 32
  
network:
  gossip_port: 8801
  tpu_port: 8802
  
storage:
  account_db_path: /data/accounts
  ledger_path: /data/ledger
```

---

## 🤝 Contributing

We welcome contributions from developers, researchers, and AI enthusiasts!

### Ways to Contribute

1. **Code** — Implement features from the [Phase 3 roadmap](./AETHER_Design_Specification_Phase2.md#11-phase-3-development-specifications)
2. **Research** — Improve consensus, privacy circuits, or tokenomics
3. **Testing** — Audit, fuzz, and load test the network
4. **Documentation** — Improve docs, write tutorials, or translate

### Process

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/your-feature`
3. **Commit** your changes: `git commit -m 'Add some feature'`
4. **Push** to the branch: `git push origin feature/your-feature`
5. **Open** a Pull Request

### Design Decisions

All major changes should reference the [Phase 2 Design Specification](./AETHER_Design_Specification_Phase2.md). For new proposals, please create a design document and open a discussion first.

---

## 📄 License

This project is licensed under the MIT License — see [LICENSE](../LICENSE) for details.

---

## 🦑 Jelly-legs AI Team

Built with autonomy by the [Jelly-legs AI Team](https://github.com/jelly-legs-ai).

**Project Status:** Phase 2 Design Complete — Phase 3 Implementation In Progress

For questions, open an issue or reach out via the repository.
