#!/usr/bin/env node

/**
 * AetherChain CLI
 * Commands wired to real SDK with blockchain RPC calls
 */

const { AetherClient } = require('../lib/sdk/client');

const RPC_URL = process.env.AETHER_RPC_URL || 'http://127.0.0.1:8899';
const client = new AetherClient(RPC_URL);

const commands = {
  // Core chain commands
  slot: async () => {
    const info = await client.getSlot();
    console.log(`Current Slot: ${info.slot}`);
    console.log(`Block Hash: ${info.block_hash}`);
    console.log(`Parent Block Hash: ${info.parent_block_hash}`);
    console.log(`Healthy: ${info.healthy}`);
    if (info.error) console.log(`Error: ${info.error}`);
  },

  height: async () => {
    const info = await client.getBlockHeight();
    console.log(`Block Height: ${info.blockHeight}`);
    console.log(`Slot: ${info.slot}`);
  },

  block: async (args) => {
    const slot = args[0] ? parseInt(args[0]) : (await client.getSlot()).slot;
    const block = await client.getBlock(slot);
    if (!block) {
      console.log(`Block not found for slot ${slot}`);
      return;
    }
    console.log(`Slot: ${block.slot}`);
    console.log(`Timestamp: ${block.timestamp}`);
    console.log(`Block Hash: ${block.block_hash}`);
    console.log(`Previous Block Hash: ${block.previous_block_hash}`);
    console.log(`PoH Seed: ${block.poh_seed.slice(0, 30)}...`);
    console.log(`Transaction Count: ${block.transaction_count}`);
  },

  genesis: async () => {
    const info = await client.getGenesis();
    console.log(`Chain ID: ${info.chain_id}`);
    console.log(`Genesis Hash: ${info.genesis_hash}`);
  },

  epoch: async () => {
    const info = await client.getEpoch();
    console.log(`Epoch: ${info.epoch}`);
    console.log(`Slot Index: ${info.slot_index}`);
    console.log(`Slots in Epoch: ${info.slots_in_epoch}`);
    console.log(`Absolute Slot: ${info.absolute_slot}`);
    console.log(`Transaction Count: ${info.transaction_count}`);
  },

  validators: async () => {
    const validators = await client.getValidators();
    console.log(`Connected Validators: ${validators.length}\n`);
    validators.forEach((v, i) => {
      console.log(`[${i + 1}] ${v.identity_pubkey.slice(0, 20)}...`);
      console.log(`    Stake: ${v.activated_stake} lamports`);
      console.log(`    Commission: ${v.commission}%`);
      console.log(`    Active: ${v.active}`);
      console.log();
    });
  },

  'validator-info': async () => {
    const info = await client.getValidatorInfo();
    console.log(`Tier: ${info.tier}`);
    console.log(`Consensus Weight: ${info.consensus_weight}`);
    console.log(`Can Produce Blocks: ${info.can_produce_blocks}`);
    console.log(`Can Vote: ${info.can_vote}`);
  },

  'vote-accounts': async () => {
    const accounts = await client.getVoteAccounts();
    console.log(`Vote Accounts: ${accounts.vote_accounts.length}`);
    console.log(`Total Stake: ${accounts.total_stake} lamports`);
  },

  'block-production': async () => {
    const stats = await client.getBlockProduction();
    console.log(`Blocks Produced: ${stats.blocks_produced}`);
    console.log(`Entries Produced: ${stats.entries_produced}`);
    console.log(`Current Epoch: ${stats.epoch}`);
  },

  account: async (args) => {
    const address = args[0];
    if (!address) {
      console.log('Usage: aether account <address>');
      return;
    }
    const account = await client.getAccount(address);
    console.log(`Address: ${account.address}`);
    console.log(`Balance: ${account.lamports} lamports`);
    console.log(`Owner: ${account.owner}`);
    console.log(`Data Size: ${account.data_size} bytes`);
    console.log(`Rent Epoch: ${account.rent_epoch}`);
  },

  supply: async () => {
    const supply = await client.getTotalSupply();
    console.log(`Total Supply: ${supply.total_supply} ${supply.unit || 'lamports'}`);
  },

  tx: async (args) => {
    const signature = args[0];
    if (!signature) {
      console.log('Usage: aether tx <signature>');
      return;
    }
    const tx = await client.getTransaction(signature);
    if (!tx) {
      console.log(`Transaction not found: ${signature}`);
      return;
    }
    console.log(`Signature: ${tx.signature}`);
    console.log(`Slot: ${tx.slot}`);
    console.log(`Block Hash: ${tx.block_hash}`);
    console.log(`Success: ${tx.success}`);
    console.log(`Timestamp: ${tx.timestamp}`);
    if (tx.error) console.log(`Error: ${tx.error}`);
  },

  ping: async () => {
    const result = await client.ping();
    console.log(`Reachable: ${result.reachable}`);
    if (result.reachable) {
      console.log(`Slot: ${result.slot}`);
      console.log(`Healthy: ${result.healthy}`);
      console.log(`Block Hash: ${result.blockHash}`);
    } else {
      console.log(`Error: ${result.error}`);
    }
  },

  health: async () => {
    const health = await client.health();
    console.log(`Status: ${health.status}`);
  },

  help: () => {
    console.log(`
AetherChain CLI v0.1.0

Commands (all make real RPC calls to ${RPC_URL}):

Chain State:
  slot              GET /v1/slot - Current slot info
  height            GET /v1/blockheight - Current block height
  block [slot]      GET /v1/block?slot=N - Get block by slot (defaults to current)
  genesis           GET /v1/genesis - Genesis configuration
  epoch             GET /v1/epoch - Epoch information
  block-production  GET /v1/block_production - Block production stats

Validators:
  validators        GET /v1/validators - List connected validators
  validator-info    GET /v1/validator/info - Current validator tier
  vote-accounts     GET /v1/voteAccounts - Vote accounts

Accounts & Transactions:
  account <addr>  GET /v1/account/<addr> - Account info
  supply            GET /v1/total_supply - Total token supply
  tx <sig>         GET /v1/tx/<sig> - Transaction status

Health:
  ping              Combined reachable + slot + health check
  health            GET /health - Health check

Environment:
  AETHER_RPC_URL    Override RPC endpoint (default: http://127.0.0.1:8899)

Chain must be running: aether-validator.exe start --genesis genesis.json --no-stake
`);
  },
};

async function main() {
  const [cmd, ...args] = process.argv.slice(2);

  if (!cmd || cmd === 'help' || cmd === '--help' || cmd === '-h') {
    commands.help();
    return;
  }

  const fn = commands[cmd];
  if (!fn) {
    console.error(`Unknown command: ${cmd}`);
    console.log("Run 'aether help' for available commands.");
    process.exit(1);
  }

  try {
    await fn(args);
  } catch (err) {
    console.error(`Error: ${err.message}`);
    process.exit(1);
  }
}

main();
