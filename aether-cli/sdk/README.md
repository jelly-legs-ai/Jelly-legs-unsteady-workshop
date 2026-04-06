# @jellylegsai/aether-sdk

Official Aether Blockchain SDK for Node.js. Every function makes **REAL HTTP RPC calls** to the Aether blockchain at `http://127.0.0.1:8899`. No stubs, no mocks.

## Installation

```bash
npm install @jellylegsai/aether-sdk
```

Or use locally from the `sdk/` folder in this repo:

```javascript
const aether = require('./sdk');
```

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
const client = new aether.AetherClient({ rpcUrl: 'http://127.0.0.1:8899' });
const epoch = await client.getEpochInfo();
```

## SDK Architecture

- **`AetherClient`** — Full-featured blockchain client class. All methods make real HTTP calls.
- **Convenience functions** — Top-level exports for one-off queries using the default RPC.
- **Low-level RPC helpers** — `rpcGet` / `rpcPost` for custom requests.

Default RPC: `http://127.0.0.1:8899` (override via `AETHER_RPC` env or constructor option).

## AetherClient

```javascript
const aether = require('@jellylegsai/aether-sdk');

const client = new aether.AetherClient({
  rpcUrl: 'http://127.0.0.1:8899',  // default
  timeoutMs: 10000,                  // default: 10s
});
```

## API Reference

### Chain Queries

| Method | Returns | RPC Endpoint |
|--------|---------|--------------|
| `getSlot()` | `number` — current slot | `GET /v1/slot` |
| `getBlockHeight()` | `number` — block height | `GET /v1/blockheight` |
| `getEpochInfo()` | `object` — epoch + slot info | `GET /v1/epoch` |
| `getAccountInfo(address)` | `object` — lamports, owner, data | `GET /v1/account/:address` |
| `getBalance(address)` | `number` — lamports | `GET /v1/account/:address` |
| `getTransaction(signature)` | `object` — tx details | `GET /v1/transaction/:sig` |
| `getRecentTransactions(address, limit?)` | `array` — recent txs | `GET /v1/transactions/:address` |
| `getTokenAccounts(address)` | `array` — SPL token accounts | `GET /v1/tokens/:address` |
| `getStakeAccounts(address)` | `array` — stake accounts | `GET /v1/stake-accounts/:address` |
| `getStakePositions(address)` | `array` — delegations | `GET /v1/stake/:address` |
| `getRewards(address)` | `object` — staking rewards | `GET /v1/rewards/:address` |
| `getValidators()` | `array` — validator list | `GET /v1/validators` |
| `getValidatorAPY(validatorAddr)` | `object` — APY data | `GET /v1/validator/:addr/apy` |
| `getTPS()` | `number` — TPS | `GET /v1/tps` |
| `getSupply()` | `object` — total/circulating/non-circulating | `GET /v1/supply` |
| `getFees()` | `object` — fee estimates | `GET /v1/fees` |
| `getSlotProduction()` | `object` — slot production | `POST /v1/slot_production` |
| `getClusterPeers()` | `array` — peer list | `GET /v1/peers` |
| `getHealth()` | `string` — "ok" if healthy | `GET /v1/health` |
| `getVersion()` | `object` — node version | `GET /v1/version` |

### Blockhash (required for signing)

| Method | Returns | RPC Endpoint |
|--------|---------|--------------|
| `getRecentBlockhash()` | `{ blockhash, lastValidBlockHeight }` | `GET /v1/recent-blockhash` |

### Transaction Submission

| Method | Returns | RPC Endpoint |
|--------|---------|--------------|
| `sendTransaction(tx)` | `object` — receipt | `POST /v1/transaction` |

### Transaction Builders (build + submit)

| Method | Description |
|--------|-------------|
| `transfer({ from, to, amount, nonce, signFn })` | Build + send a Transfer TX |
| `stake({ staker, validator, amount, signFn })` | Build + send a Stake TX |
| `unstake({ stakeAccount, amount, signFn })` | Build + send an Unstake TX |
| `claimRewards({ stakeAccount, signFn })` | Build + send a ClaimRewards TX |
| `createNFT({ creator, metadataUrl, royalties, signFn })` | Build + send a CreateNFT TX |
| `transferNFT({ from, nftId, to, signFn })` | Build + send a TransferNFT TX |
| `updateMetadata({ creator, nftId, metadataUrl, signFn })` | Build + send an UpdateMetadata TX |

Each builder fetches a fresh `blockhash` + current `slot` from the chain, then submits the signed transaction via `sendTransaction()`. All steps are real RPC calls.

## Convenience Functions

```javascript
// All of these use the default RPC (http://127.0.0.1:8899)
const slot = await aether.getSlot();
const balance = await aether.getBalance('ATH...');
const epoch = await aether.getEpoch();
const tps = await aether.getTPS();
const supply = await aether.getSupply();
const fees = await aether.getFees();
const validators = await aether.getValidators();
const peers = await aether.getPeers();
const health = await aether.getHealth();
const { blockhash } = await aether.getRecentBlockhash();
const tx = await aether.getTransaction('sig...');
const txs = await aether.getRecentTransactions('ATH...', 20);
const stakePos = await aether.getStakePositions('ATH...');
const rewards = await aether.getRewards('ATH...');
const tokenAccts = await aether.getTokenAccounts('ATH...');
const stakeAccts = await aether.getStakeAccounts('ATH...');
const apy = await aether.getValidatorAPY('validatorAddr');
const pingResult = await aether.ping(); // { ok, latency, rpc }
```

## Utilities

| Function | Description |
|----------|-------------|
| `createClient(options)` | Factory: create an `AetherClient` instance |
| `ping(rpcUrl?)` | Ping RPC, returns `{ ok, latency, rpc }` |
| `rpcGet(rpcUrl, path, timeout?)` | Low-level GET |
| `rpcPost(rpcUrl, path, body, timeout?)` | Low-level POST |
| `DEFAULT_RPC_URL` | `"http://127.0.0.1:8899"` |
| `DEFAULT_TIMEOUT_MS` | `10000` |

## CLI Integration

The SDK is wired into these CLI commands in `aether-cli`:

| CLI Command | SDK Method Used |
|-------------|----------------|
| `aether status` | `getEpochInfo`, `getSupply`, `getSlot`, `getBlockHeight`, `getClusterPeers`, `getVersion`, `getStakePositions`, `getRewards` |
| `aether network` | `getSlot`, `getBlockHeight`, `getClusterPeers`, `getValidators`, `getEpochInfo`, `getSupply`, `getTPS` |
| `aether blockhash` | `getRecentBlockhash` |
| `aether fees` | `getFees`, `getRecentBlockhash` |
| `aether tps` | `getTPS`, `getSlot` |
| `aether account` | `getAccountInfo`, `getBalance`, `getRecentTransactions`, `getStakePositions`, `getTokenAccounts`, `getStakeAccounts` |

## Configuration

```bash
# Environment variable
export AETHER_RPC=http://127.0.0.1:8899

# Or per-call
const client = new aether.AetherClient({ rpcUrl: 'http://custom:8899' });
```

## Example: Full Dashboard

```javascript
const aether = require('@jellylegsai/aether-sdk');

async function dashboard() {
  const [slot, blockHeight, tps, supply, { blockhash }] = await Promise.all([
    aether.getSlot(),
    aether.getBlockHeight(),
    aether.getTPS(),
    aether.getSupply(),
    aether.getRecentBlockhash(),
  ]);

  console.log('=== Aether Network Dashboard ===');
  console.log(`Slot:              ${slot}`);
  console.log(`Block:             ${blockHeight}`);
  console.log(`TPS:               ${tps}`);
  console.log(`Total Supply:      ${supply.total}`);
  console.log(`Blockhash:         ${blockhash}`);
}

dashboard();
```

## Example: Check Staking Rewards

```javascript
const aether = require('@jellylegsai/aether-sdk');

async function checkRewards(walletAddress) {
  const [stakePositions, rewards] = await Promise.all([
    aether.getStakePositions(walletAddress),
    aether.getRewards(walletAddress),
  ]);

  console.log(`Stake positions: ${stakePositions.length}`);
  console.log(`Total rewards:   ${rewards.total || rewards.amount} lamports`);
}

checkRewards('ATH...');
```

## License

MIT © Jelly-legs AI Team
