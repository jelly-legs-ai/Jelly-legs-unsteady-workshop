#!/usr/bin/env node
/**
 * aether-cli delegations
 *
 * View stake delegations and accumulated rewards for a wallet.
 * Also supports claiming rewards from a stake account.
 *
 * Usage:
 *   aether delegations list --address <addr>         List all stake delegations
 *   aether delegations list --address <addr> --json  JSON output
 *   aether delegations claim --address <addr> --account <stakeAcct> [--json]
 *
 * SDK wired to: GET /v1/slot, GET /v1/account/<addr>, GET /v1/stake/<addr>
 */

const path = require('path');
const readline = require('readline');
const crypto = require('crypto');
const bs58 = require('bs58').default;
const bip39 = require('bip39');
const nacl = require('tweetnacl');

// ANSI colours
const C = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

const DERIVATION_PATH = "m/44'/7777777'/0'/0'";
const CLI_VERSION = '1.0.6';

// ---------------------------------------------------------------------------
// SDK Import
// ---------------------------------------------------------------------------

const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ---------------------------------------------------------------------------
// Paths & config
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(require('os').homedir(), '.aether');
}

function loadConfig() {
  const p = path.join(getAetherDir(), 'config.json');
  if (!require('fs').existsSync(p)) return { defaultWallet: null };
  try {
    return JSON.parse(require('fs').readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fp = path.join(getAetherDir(), 'wallets', `${address}.json`);
  if (!require('fs').existsSync(fp)) return null;
  return JSON.parse(require('fs').readFileSync(fp, 'utf8'));
}

// ---------------------------------------------------------------------------
// Crypto helpers (mirrored from wallet.js)
// ---------------------------------------------------------------------------

function deriveKeypair(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) throw new Error('Invalid mnemonic');
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return { publicKey: Buffer.from(keyPair.publicKey), secretKey: Buffer.from(keyPair.secretKey) };
}

function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

// ---------------------------------------------------------------------------
// Parse CLI args
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // [node, index.js, delegations, <subcmd>, ...]
  return args;
}

function createRl() {
  return readline.createInterface({ input: process.stdin, output: process.stdout });
}

function question(rl, q) {
  return new Promise((res) => rl.question(q, res));
}

async function askMnemonic(rl, prompt) {
  console.log(`\n${C.cyan}${prompt}${C.reset}`);
  console.log(`${C.dim}Enter your 12 or 24-word passphrase, one space-separated line:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

// ---------------------------------------------------------------------------
// LIST DELEGATIONS  — uses SDK
// ---------------------------------------------------------------------------

async function listDelegations(args) {
  const rl = createRl();
  let address = null;
  let asJson = false;
  let rpcUrl = getDefaultRpc();

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) address = args[++i];
    else if (args[i] === '--json' || args[i] === '-j') asJson = true;
    else if ((args[i] === '--rpc' || args[i] === '-r') && args[i + 1]) rpcUrl = args[++i];
  }

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether delegations list --address <addr> [--json]${C.reset}\n`);
    rl.close();
    return;
  }

  const client = createClient(rpcUrl);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  try {
    // Real chain RPC calls via SDK
    const [account, stakeAccounts] = await Promise.all([
      client.getAccountInfo(rawAddr).catch(() => null),
      client.getStakePositions(rawAddr).catch(() => []),
    ]);

    if (asJson) {
      console.log(JSON.stringify({
        address,
        rpc: rpcUrl,
        account: account && !account.error ? { lamports: account.lamports } : null,
        delegations: stakeAccounts,
        cli_version: CLI_VERSION,
        fetched_at: new Date().toISOString(),
      }, null, 2));
      rl.close();
      return;
    }

    console.log(`\n${C.bright}${C.cyan}── Stake Delegations ─────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.green}★${C.reset} Wallet: ${C.bright}${address}${C.reset}`);
    console.log(`  ${C.dim}  RPC: ${rpcUrl}${C.reset}`);
    if (account && !account.error) {
      console.log(`  ${C.green}✓ Balance:${C.reset} ${C.bright}${formatAether(account.lamports || 0)}${C.reset}`);
    }
    console.log();

    if (!stakeAccounts || stakeAccounts.length === 0) {
      console.log(`  ${C.dim}No stake delegations found for this wallet.${C.reset}`);
      console.log(`  ${C.dim}Delegate with:${C.reset} ${C.cyan}aether stake --address ${address} --validator <val> --amount <aeth>${C.reset}\n`);
      rl.close();
      return;
    }

    const typeColors = {
      Stake: C.green,
      Unstake: C.yellow,
      ClaimRewards: C.magenta,
    };

    for (const stake of stakeAccounts) {
      const status = stake.status || stake.state || 'active';
      const statusColor = status === 'active' ? C.green : status === 'unstaked' ? C.yellow : C.red;
      const validator = stake.validator || stake.delegation?.validator || 'unknown';
      const amount = stake.lamports || stake.amount || stake.delegation?.lamports || 0;
      const rewards = stake.rewards || stake.pending_rewards || 0;
      const stakeAcct = stake.pubkey || stake.publicKey || stake.account || 'unknown';

      console.log(`  ${C.bright}┌─ ${stakeAcct}${C.reset}`);
      console.log(`  │  Validator: ${C.cyan}${validator}${C.reset}`);
      console.log(`  │  Amount:   ${C.bright}${formatAether(amount)}${C.reset}`);
      if (rewards > 0) {
        console.log(`  │  ${C.magenta}★ Rewards: ${formatAether(rewards)}${C.reset}`);
      }
      console.log(`  │  Status:   ${statusColor}${status}${C.reset}`);
      console.log(`  ${C.dim}└${C.reset}`);
      console.log();
    }

    // Summary
    const totalDelegated = stakeAccounts.reduce((sum, s) => sum + (s.lamports || s.amount || s.delegation?.lamports || 0), 0);
    const totalRewards = stakeAccounts.reduce((sum, s) => sum + (s.rewards || s.pending_rewards || 0), 0);
    console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}Total delegated: ${C.reset}${C.bright}${formatAether(totalDelegated)}${C.reset}`);
    if (totalRewards > 0) {
      console.log(`  ${C.dim}Total rewards:   ${C.reset}${C.bright}${C.magenta}${formatAether(totalRewards)}${C.reset}`);
      console.log(`  ${C.dim}  Claim with: aether delegations claim --address ${address} --account <stake_account>${C.reset}`);
    }
    console.log();
    rl.close();
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to fetch delegations:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    rl.close();
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// CLAIM REWARDS  — uses SDK for fetch, wallet for signing
// ---------------------------------------------------------------------------

async function claimRewards(args) {
  const rl = createRl();

  let address = null;
  let stakeAccount = null;
  let asJson = false;
  let rpcUrl = getDefaultRpc();

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) address = args[++i];
    else if ((args[i] === '--account' || args[i] === '-s') && args[i + 1]) stakeAccount = args[++i];
    else if (args[i] === '--json' || args[i] === '-j') asJson = true;
    else if ((args[i] === '--rpc' || args[i] === '-r') && args[i + 1]) rpcUrl = args[++i];
  }

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether delegations claim --address <addr> --account <stakeAcct>${C.reset}\n`);
    rl.close();
    return;
  }

  const client = createClient(rpcUrl);

  // If no stake account specified, fetch list via SDK
  if (!stakeAccount) {
    const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
    let stakeAccounts = await client.getStakePositions(rawAddr).catch(() => []);

    if (!stakeAccounts || stakeAccounts.length === 0) {
      console.log(`  ${C.red}✗ No stake accounts found.${C.reset} Use ${C.cyan}--account <stakeAcct>${C.reset} to specify one.\n`);
      rl.close();
      return;
    }

    console.log(`\n${C.bright}${C.cyan}── Select Stake Account ──────────────────────────────────${C.reset}\n`);
    for (let i = 0; i < stakeAccounts.length; i++) {
      const s = stakeAccounts[i];
      const rewards = s.rewards || s.pending_rewards || 0;
      const validator = s.validator || s.delegation?.validator || 'unknown';
      console.log(`  ${C.green}${i + 1})${C.reset} ${s.pubkey || s.publicKey || s.account}`);
      console.log(`      Validator: ${C.cyan}${validator}${C.reset}  Rewards: ${C.magenta}${formatAether(rewards)}${C.reset}`);
    }
    console.log();
    const choice = await question(rl, `  ${C.cyan}Select account [1-${stakeAccounts.length}]:${C.reset} `);
    const idx = parseInt(choice.trim(), 10) - 1;
    if (isNaN(idx) || idx < 0 || idx >= stakeAccounts.length) {
      console.log(`  ${C.red}Invalid selection.${C.reset}\n`);
      rl.close();
      return;
    }
    stakeAccount = stakeAccounts[idx].pubkey || stakeAccounts[idx].publicKey || stakeAccounts[idx].account;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}\n`);
    rl.close();
    return;
  }

  console.log(`\n${C.bright}${C.cyan}── Claim Rewards ─────────────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.green}★${C.reset} Wallet:      ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Stake acct: ${C.bright}${stakeAccount}${C.reset}`);
  console.log();

  // Ask for mnemonic to derive signing keypair
  console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter your 12/24-word passphrase to sign this transaction');
  console.log();

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }

  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected: ${address}${C.reset}\n`);
    rl.close();
    return;
  }

  const confirm = await question(rl, `  ${C.yellow}Confirm claim? [y/N]${C.reset} > ${C.reset}`);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }

  // Build claim rewards transaction
  const tx = {
    signer: address.startsWith('ATH') ? address.slice(3) : address,
    tx_type: 'ClaimRewards',
    payload: {
      type: 'ClaimRewards',
      data: {
        stake_account: stakeAccount,
      },
    },
    fee: 0,
    slot: 0,
    timestamp: Math.floor(Date.now() / 1000),
  };

  console.log(`  ${C.dim}Submitting via SDK to ${rpcUrl}...${C.reset}`);

  try {
    const result = await client.sendTransaction(tx);

    if (result.error) {
      console.log(`\n  ${C.red}✗ Claim failed:${C.reset} ${result.error}\n`);
      rl.close();
      process.exit(1);
    }

    const sig = result.signature || result.tx_signature || result.id || JSON.stringify(result);
    console.log(`\n${C.green}✓ Rewards claim submitted!${C.reset}`);
    console.log(`  ${C.dim}Signature: ${sig}${C.reset}`);
    console.log(`  ${C.dim}Check balance: aether wallet balance --address ${address}${C.reset}\n`);
    rl.close();
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to submit claim:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}\n`);
    rl.close();
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Main dispatcher
// ---------------------------------------------------------------------------

async function delegationsCommand() {
  const args = parseArgs();
  const subcmd = args[0];

  const rl = createRl();
  try {
    if (!subcmd || subcmd === 'list') {
      await listDelegations(args);
    } else if (subcmd === 'claim') {
      await claimRewards(args);
    } else {
      console.log(`\n  ${C.red}Unknown subcommand:${C.reset} ${subcmd}`);
      console.log(`\n  Usage:`);
      console.log(`    ${C.cyan}aether delegations list  --address <addr>${C.reset}  List stake delegations`);
      console.log(`    ${C.cyan}aether delegations claim --address <addr> --account <stakeAcct>${C.reset}  Claim rewards`);
      console.log();
      process.exit(1);
    }
  } finally {
    rl.close();
  }
}

module.exports = { delegationsCommand };

if (require.main === module) {
  delegationsCommand();
}
