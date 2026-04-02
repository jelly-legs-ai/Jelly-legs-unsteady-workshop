# AETHER Testnet Genesis

This directory contains tools for generating and managing local testnet genesis blocks.

## Quick Start

```bash
# Generate genesis block and bootstrap validator identities
node generate.js

# This creates:
#   genesis.json                          - The genesis block
#   bootstrap-validator-1-identity.json   - Bootstrap validator keypair
#   GENESIS_HASH.txt                     - Genesis hash for reference
```

## Files

- `generate.js` - Genesis block generator script
- `genesis.json` - Generated genesis block (after running generator)
- `bootstrap-validator-*-identity.json` - Validator keypairs for bootstrap nodes
- `GENESIS_HASH.txt` - Hash of the genesis block

## Chain Configuration

| Parameter | Value |
|-----------|-------|
| Chain ID | `aether-testnet-1` |
| Slot Time | 400ms |
| Slots/Epoch | 432,000 (~2 days) |
| Min Stake | 100 AETH |
| Tower Finality | 12 slots |

## Starting a Local Testnet

### Terminal 1: Bootstrap Validator

```bash
# From the project root
cargo run --bin aether-validator -- start --testnet
```

### Terminal 2: Check Status

```bash
# Using aether-cli
aether-cli validator status

# Or using curl directly
curl -X POST http://127.0.0.1:8899 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

### Terminal 3: View Validators

```bash
cargo run --bin aether-validator -- show-validators
```

## Genesis Block Structure

```json
{
  "chain_id": "aether-testnet-1",
  "genesis_hash": "<sha256-hash>",
  "timestamp": 1234567890,
  "slot_time_ms": 400,
  "slots_per_epoch": 432000,
  "min_stake": 100,
  "consensus": {
    "mode": "aetherflow",
    "tower_finality": 12
  },
  "bootstrap_validators": [
    {
      "identity_pubkey": "<base58-encoded>",
      "activated_stake": 10000000,
      "commission": 10,
      "active": true
    }
  ],
  "rewards": {
    "epoch_duration": 432000,
    "base_reward_rate": 6
  }
}
```

## Bootstrap Validators

The genesis generator creates bootstrap validator identities with:
- 10,000,000 AETH initial stake
- 10% commission rate
- Active status at genesis

**IMPORTANT**: These keypairs control your validator identity. Backup and protect them!

## Adding More Validators

To add validators to a running testnet:

```bash
# Create new identity
cargo run --bin aether-validator -- create-validator-identity --out new-validator-identity.json

# The new validator will join via P2P gossip
# They need to have stake delegated to their vote account
```

## Troubleshooting

### "Connection refused" on RPC
The validator may not be running. Start it first:
```bash
cargo run --bin aether-validator -- start --testnet
```

### "Slot stuck" 
Check that your clock is synchronized. Use `ntpdate` or similar to sync time.

### "Peer count is 0"
Gossip takes time to propagate. Wait ~30 seconds and check again.
