# Jelly-Legs Unsteady Workshop

A visual project management dashboard with an animated jellyfish avatar.

## Visual Workshop

The dashboard features an immersive visual workshop overlay with:
- **Toggle Button**: Switch between simple Kanban and visual workshop views
- **4 Themed Rooms**: Research Lab (🔬), Design Studio (✏️), Build Factory (⚙️), Launch Pad (🚀)
- **Animated Avatar**: 🪼 Jelly-Legs avatar that moves between rooms with bounce animations
- **Interactive Navigation**: Click any room to move the avatar there
- **Responsive Design**: Works on mobile and desktop

Click the toggle button (top-right) to switch between views!

---

# AETHER Blockchain

A high-performance blockchain fork from Agave (Solana's Rust core) featuring modified PoH+PoS consensus with AI-powered transaction priority lanes.

## Overview

AETHER is a next-generation Layer 1 blockchain that combines:

- **Proof of History (PoH)**: Verifiable delay function for high-throughput consensus
- **Proof of Stake (PoS)**: Energy-efficient validator selection
- **AI Priority Lanes**: Machine learning-based transaction prioritization
- **5-Layer Architecture**: Modular design for scalability and maintainability

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                          │
├─────────────────────────────────────────────────────────────┤
│                    AI Priority Lanes                          │
│         (ML-based transaction prioritization)                 │
├─────────────────────────────────────────────────────────────┤
│                    Consensus Layer                          │
│              (PoH + PoS Hybrid)                             │
├─────────────────────────────────────────────────────────────┤
│                    Network Layer                            │
│         (P2P gossip, block propagation)                     │
├─────────────────────────────────────────────────────────────┤
│                    Storage Layer                            │
│         (Block store, state management)                     │
├─────────────────────────────────────────────────────────────┤
│                      Core Layer                             │
│         (Types, crypto, primitives)                         │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
aether/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── .gitignore             # Git ignore rules
│
├── aether-core/           # Core types and primitives
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs       # Block, Transaction, Hash types
│   │   ├── crypto.rs      # Cryptographic utilities
│   │   └── error.rs       # Error types
│   └── Cargo.toml
│
├── aether-consensus/      # PoH + PoS consensus
│   ├── src/
│   │   ├── lib.rs
│   │   ├── poh_pos.rs     # Hybrid consensus
│   │   ├── validator.rs   # Validator management
│   │   └── stake.rs       # Staking operations
│   └── Cargo.toml
│
├── aether-poh/            # Proof of History
│   ├── src/
│   │   ├── lib.rs
│   │   ├── generator.rs   # PoH hash generation
│   │   └── verifier.rs    # PoH verification
│   └── Cargo.toml
│
├── aether-ai-lanes/       # AI transaction priority
│   ├── src/
│   │   ├── lib.rs
│   │   ├── priority.rs    # Priority scoring
│   │   ├── model.rs       # ML model
│   │   └── features.rs    # Feature extraction
│   └── Cargo.toml
│
├── aether-network/        # P2P networking
│   ├── src/
│   │   ├── lib.rs
│   │   ├── gossip.rs      # Gossip protocol
│   │   ├── propagation.rs # Block propagation
│   │   └── peer.rs        # Peer management
│   └── Cargo.toml
│
└── aether-storage/        # Blockchain storage
    ├── src/
    │   ├── lib.rs
    │   ├── blockstore.rs  # Block storage
    │   ├── state.rs       # State management
    │   └── archive.rs     # Archival storage
    └── Cargo.toml
```

## Prerequisites

- **Rust**: 1.75 or later
- **Cargo**: Latest stable
- **Git**: For cloning dependencies

### System Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang
```

#### macOS
```bash
# Install Homebrew if not present
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install openssl pkg-config
```

#### Windows
```powershell
# Install Visual Studio Build Tools
# Install OpenSSL via vcpkg or chocolatey
choco install openssl
```

## Building

### Quick Start

```bash
# Clone the repository
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop.git
cd Jelly-legs-unsteady-workshop

# Build all crates
cargo build --release

# Run tests
cargo test --workspace
```

### Development Build

```bash
# Build in debug mode (faster compilation)
cargo build

# Build specific crate
cargo build -p aether-core
```

### Release Build

```bash
# Optimized release build
cargo build --release

# The release binaries will be in target/release/
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p aether-core

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --workspace
```

## Development

### Code Style

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy lints
cargo clippy --workspace -- -D warnings
```

### Documentation

```bash
# Generate and open docs
cargo doc --workspace --open

# Generate docs for all features
cargo doc --workspace --all-features
```

## Fork Information

This project is a fork from [Agave](https://github.com/anza-xyz/agave), Solana's Rust core client. Key modifications:

1. **AI Priority Lanes**: New crate for ML-based transaction prioritization
2. **Modified Consensus**: Hybrid PoH+PoS with custom validator selection
3. **Modular Architecture**: Clean separation into 5 layers
4. **Extended Types**: Additional metadata for AI scoring

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original Agave/Solana team for the foundational codebase
- Anza for maintaining the Solana ecosystem
- Contributors to the Rust blockchain ecosystem

## Roadmap

- [x] Initial project structure
- [ ] Core types implementation
- [ ] PoH generator and verifier
- [ ] Consensus mechanism
- [ ] AI model integration
- [ ] Network layer
- [ ] Storage layer
- [ ] Testnet deployment
- [ ] Mainnet preparation

## Contact

- **Issues**: [GitHub Issues](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/discussions)

---

**Note**: This is an active development project. APIs and interfaces may change.
