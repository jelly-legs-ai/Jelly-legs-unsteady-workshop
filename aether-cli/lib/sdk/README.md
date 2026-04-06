# AetherChain SDK

Real blockchain RPC client for the Aether testnet.

## Installation

```bash
npm install  # from aether-cli directory
```

## Quick Start

```javascript
const { AetherClient } = require('./lib/sdk/client');

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
| `getTransaction(sig)` | `GET /v1/tx/<sig>` | TX status |

### Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| `health()` | `GET /health` | Health check |
| `ping()` | - | Combined reachable + slot + healthy |
