# @jellylegsai/aether-sdk

Official Aether Blockchain SDK for Node.js. Every function makes **REAL HTTP RPC calls** to the Aether blockchain. No stubs, no mocks.

## Installation

```bash
npm install @jellylegsai/aether-sdk
```

Or use locally from the `sdk/` folder in this repo.

## Quick Start

```javascript
const aether = require('@jellylegsai/aether-sdk');

// Get current slot
const slot = await aether.getSlot();
console.log('Current slot:', slot);

// Get account balance
const balance = await aether.getBalance('ATH...');
console.log('Balance:', balance);

// Use custom RPC endpoint
const customRpc = 'http://127.0.0.1:8899';
const epoch = await aether.getEpoch(customRpc);
```

## API Reference

### Chain Queries

| Function | Description | RPC Endpoint |
|----------|-------------|--------------|
| `getSlot(rpcUrl?)` | Get current network slot | `GET /v1/slot` |
| `getBlockHeight(rpcUrl?)` | Get current block height | `GET /v1/block_height` |
| `getEpoch(rpcUrl?)` | Get epoch information | `GET /v1/epoch` |
| `getAccount(address, rpcUrl?)` | Get account data | `GET /v1/account/:address` |
| `getBalance(address, rpcUrl?)` | Get account balance (lamports) | `GET /v1/account/:address` |
| `getTransaction(signature, rpcUrl?)` | Get transaction by signature | `GET /v1/transaction/:sig` |
| `getRecentTransactions(address, limit?, rpcUrl?)` | Get recent txs for address | `GET /v1/transactions/:address` |
| `getValidators(rpcUrl?)` | Get list of validators | `GET /v1/validators` |
| `getTPS(rpcUrl?)` | Get network TPS | `GET /v1/tps` |
| `getSupply(rpcUrl?)` | Get token supply info | `GET /v1/supply` |
| `getSlotProduction(rpcUrl?)` | Get slot production stats | `POST /v1/slot_production` |
| `getFees(rpcUrl?)` | Get network fee estimates | `GET /v1/fees` |
| `getStakePositions(address, rpcUrl?)` | Get stake delegations | `GET /v1/stake/:address` |
| `getRewards(address, rpcUrl?)` | Get staking rewards | `GET /v1/rewards/:address` |
| `getValidatorAPY(validatorAddr, rpcUrl?)` | Get validator APY | `GET /v1/validator/:addr/apy` |
| `getPeers(rpcUrl?)` | Get network peers | `GET /v1/peers` |
| `getHealth(rpcUrl?)` | Get network health status | `GET /v1/health` |

### Transactions

| Function | Description | RPC Endpoint |
|----------|-------------|--------------|
| `sendTransaction(signedTx, rpcUrl?)` | Submit signed transaction | `POST /v1/transaction` |

### Utilities

| Function | Description |
|----------|-------------|
| `ping(rpcUrl?)` | Ping RPC endpoint, returns latency |
| `rpcGet(rpcUrl, path, timeout?)` | Low-level GET request |
| `rpcPost(rpcUrl, path, body, timeout?)` | Low-level POST request |
| `DEFAULT_RPC` | Default RPC URL (`http://127.0.0.1:8899`) |

## Configuration

Set the `AETHER_RPC` environment variable to use a custom RPC endpoint:

```bash
export AETHER_RPC=http://api.testnet.aether.network:8899
```

Or pass `rpcUrl` to any function:

```javascript
await aether.getSlot('http://custom-rpc:8899');
```

## Example: Full Dashboard

```javascript
const aether = require('@jellylegsai/aether-sdk');

async function dashboard() {
  const [slot, blockHeight, tps, supply] = await Promise.all([
    aether.getSlot(),
    aether.getBlockHeight(),
    aether.getTPS(),
    aether.getSupply(),
  ]);

  console.log('=== Aether Network Dashboard ===');
  console.log(`Slot:        ${slot}`);
  console.log(`Block:       ${blockHeight}`);
  console.log(`TPS:         ${tps}`);
  console.log(`Total Supply: ${supply.total ?? 'N/A'}`);
}

dashboard();
```

## License

MIT © Jelly-legs AI Team
