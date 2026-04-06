# AetherChain SDK (@jellylegsai/aether-sdk)

Real blockchain RPC client for the Aether testnet.

**All SDK functions make actual HTTP calls to the blockchain. No stubs, no mocks.**

## Installation

```bash
npm install @jellylegsai/aether-sdk
```

Or from source:
```bash
cd aether-cli
npm install
```

## Quick Start

```javascript
const { AetherClient } = require('@jellylegsai/aether-sdk');

const client = new AetherClient('http://127.0.0.1:8899');

// Check chain status
const { reachable, slot, healthy } = await client.ping();
console.log(`Chain: reachable=${reachable}, slot=${slot}, healthy=${healthy}`);

// Get current slot
const slotInfo = await client.getSlot();
console.log(`Slot: ${slotInfo.slot}, Block hash: ${slotInfo.block_hash}`);

// Get validators
const validators = await client.getValidators();
console.log(`Validators: ${validators.length}`);

// Get account info
const account = await client.getAccount('GdGs8zKhYnAYmcLqwX3oGEnptoBB2khX6yxqDqqo9RFZ');
console.log(`Balance: ${account.lamports} lamports`);

// Submit a transaction
const tx = await client.sendTransaction({
  tx_type: 'transfer',
  signer: 'GdGs8zKhYnAYmcLqwX3oGEnptoBB2khX6yxqDqqo9RFZ',
  signature: '...',
  payload: { recipient: '...', amount: 1000 },
  fee: 5000,
});
```

## CLI Usage

The SDK includes a CLI that wires directly to the SDK:

```bash
# Install globally
npm install -g @jellylegsai/aether-sdk

# Or run directly
node bin/aether.js help
```

### CLI Commands

```bash
# Chain State
aether slot              # GET /v1/slot
aether height            # GET /v1/blockheight
aether block [slot]      # GET /v1/block?slot=N
aether genesis           # GET /v1/genesis
aether epoch             # GET /v1/epoch
aether block-production  # GET /v1/block_production

# Validators
aether validators        # GET /v1/validators
aether validator-info    # GET /v1/validator/info
aether vote-accounts     # GET /v1/voteAccounts

# Accounts & Transactions
aether account <addr>    # GET /v1/account/<addr>
aether supply            # GET /v1/total_supply
aether tx <sig>          # GET /v1/tx/<sig>

# Health
aether ping              # Combined reachability check
aether health            # GET /health
```

## API Reference

### Chain State

| Method | Endpoint | Description |
|--------|----------|-------------|
| `getSlot()` | `GET /v1/slot` | Current slot, block hash, health |
| `getBlockHeight()` | `GET /v1/blockheight` | Current block height |
| `getBlock(slot)` | `GET /v1/block?slot=N` | Block by slot number |
| `getGenesis()` | `GET /v1/genesis` | Genesis chain ID and hash |
| `getEpoch()` | `GET /v1/epoch` | Epoch info |
| `getBlockProduction()` | `GET /v1/block_production` | Block production stats |

### Validators

| Method | Endpoint | Description |
|--------|----------|-------------|
| `getValidators()` | `GET /v1/validators` | All connected validators |
| `getValidatorInfo()` | `GET /v1/validator/info` | Current validator tier |
| `getVoteAccounts()` | `GET /v1/voteAccounts` | Vote accounts |

### Accounts & Transactions

| Method | Endpoint | Description |
|--------|----------|-------------|
| `getAccount(addr)` | `GET /v1/account/<addr>` | Account info |
| `getTotalSupply()` | `GET /v1/total_supply` | Total token supply |
| `sendTransaction(tx)` | `POST /v1/tx` | Submit transaction |
| `getTransaction(sig)` | `GET /v1/v1/tx/<sig>` | TX status |

### Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| `health()` | `GET /health` | Health check |
| `ping()` | - | Combined reachable + slot + healthy |

## Environment Variables

```bash
AETHER_RPC_URL=http://127.0.0.1:8899  # Override RPC endpoint
```

## Chain Setup

Chain must be running:
```bash
aether-validator.exe start --genesis genesis.json --no-stake
```

## License

MIT
