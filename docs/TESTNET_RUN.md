# Phase 0 Testnet — Live Run Guide

**Goal:** Get a runnable testnet TODAY. Focus only on what's needed to make the chain produce blocks.

---

## Step 1: Build the Validator Binary

```powershell
# Install Rust if needed
irm https://rustup.rs | iex

# Clone and build (release mode)
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop
cd Jelly-legs-unsteady-workshop
cargo build --release --package aether-validator

# Binary will be at:
# target/release/aether-validator.exe
```

---

## Step 2: Generate Genesis Block

```powershell
./target/release/aether-validator.exe create-genesis `
    --chain-id aether-testnet-1 `
    --out genesis.json `
    --bootstrap-validator bootstrap-validator-identity.json
```

This creates:
- `genesis.json` — the genesis block
- `bootstrap-validator-identity.json` — your first validator identity

---

## Step 3: Start First Validator Node

```powershell
./target/release/aether-validator.exe start `
    --testnet `
    --rpc-addr 127.0.0.1:8899 `
    --p2p-addr 0.0.0.0:8001 `
    --identity bootstrap-validator-identity.json
```

You'll see:
```
INFO: Starting AETHER Validator...
INFO: Validator identity: <pubkey>
INFO: RPC server listening on 0.0.0.0:8899
INFO: Gossip: 0.0.0.0:8001
INFO: Validator running. Press Ctrl+C to stop.
```

---

## Step 4: Check Status (from another terminal)

```powershell
./target/release/aether-validator.exe status --rpc-url http://127.0.0.1:8899 --details
```

Expected output:
```
╔══════════════════════════════════════════════╗
║       AETHER VALIDATOR STATUS               ║
╚══════════════════════════════════════════════╝

  🌐 RPC Endpoint:  http://127.0.0.1:8899

  📊 Chain Status
     Slot Height:                100
     Block Height:                99
     Transaction Count:            0

  🔗 Network
     Peer Count:                   1

  📈 Epoch 0
     Progress:             0.02%
     Absolute Slot:       100
```

---

## Step 5: Start Second Validator (2-node network)

On a **different machine** (or same machine, different terminal):

```powershell
# Generate second identity
./target/release/aether-validator.exe create-validator-identity `
    --out validator2-identity.json

# Start second node on different port
./target/release/aether-validator.exe start `
    --testnet `
    --rpc-addr 127.0.0.1:8898 `
    --p2p-addr 0.0.0.0:8002 `
    --identity validator2-identity.json
```

---

## Step 6: Query the Network

```powershell
# Show all validators
./target/release/aether-validator.exe show-validators --rpc-url http://127.0.0.1:8899

# JSON output
./target/release/aether-validator.exe show-validators --rpc-url http://127.0.0.1:8899 --json
```

---

## Architecture (MVP)

```
┌─────────────────────────────────────────────────────┐
│                  TESTNET MVP                        │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────┐         ┌──────────────┐         │
│  │ Validator 1  │◄───────►│ Validator 2  │         │
│  │ RPC :8899    │  P2P    │ RPC :8898    │         │
│  │ Gossip :8001 │  :8002  │ Gossip :8002 │         │
│  └──────┬───────┘         └──────────────┘         │
│         │                                           │
│         ▼                                           │
│  ┌──────────────┐                                  │
│  │ Slot Counter │ ← Increments every 400ms        │
│  │ Block Height │ ← Tracks confirmed blocks        │
│  │ Vote Tracker │ ← Consensus voting               │
│  └──────────────┘                                  │
│                                                      │
│  Consensus: AetherFlow (simplified)                  │
│  - Slots: 400ms each                                │
│  - Block: produced every slot                       │
│  - Finality: Tower BFT (12 block confirmation)       │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## What Works in MVP

| Feature | Status |
|---------|--------|
| Binary builds | ✅ |
| Genesis creation | ✅ |
| Validator start | ✅ |
| Slot/block increment | ✅ |
| RPC server (basic) | ✅ |
| Multi-validator | ⚠️ (P2P stub, simulated) |
| Real consensus | ❌ (needs aether-consensus integration) |
| Actual transaction processing | ❌ (needs contract runtime) |

---

## What's Next (Phase 0B → Phase 1)

**Phase 0B:** Fix workspace compilation, integrate aether-consensus, real P2P gossip  
**Phase 1:** Add FLUX/ATH token contracts, mining rewards, wallet integration  
**Phase 2:** AI priority lanes, governance, mainnet

---

## Troubleshooting

**"Address already in use"**
```powershell
# Find and kill existing process
Get-NetTCPConnection -LocalPort 8899 | Stop-Process -Force
```

**"Identity file not found"**
```powershell
# Create identity first
./target/release/aether-validator.exe create-validator-identity --out my-identity.json
```

**Binary not found after build**
```powershell
# Check the binary exists
Get-ChildItem target/release/*.exe
```
