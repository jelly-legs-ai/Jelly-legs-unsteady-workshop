#!/usr/bin/env node
/**
 * aether-cli unstake
 *
 * Unstake AETH from a validator — deactivate a stake account and begin cooldown.
 * Fully wired to @jellylegsai/aether-sdk for real blockchain RPC calls.
 *
 * Usage:
 *   aether unstake --address <wallet> [--account <stakeAcct>] [--amount <aeth>]
 *   aether unstake --address ATHxxx... --account Stakexxx... --amount 100
 *   aether unstake --address ATHxxx... --json
 *
 * SDK wired to:
 *   - client.getSlot()              → GET /v1/slot
 *   - client.getStakePositions()    → GET /v1/stake/<addr>
 *   - client.getAccountInfo()       → GET /v1/account/<addr>
 *   - client.sendTransaction()      → POST /v1/transaction
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const bip39 = require('bip39');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

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

const CLI_VERSION = '1.0.0';
const DERIVATION_PATH = "m/44'/7777777'/0'/0'";

// ============================================================================
// SDK Setup
// ============================================================================

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ============================================================================
// Config & Wallet
// ============================================================================

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function loadConfig() {
  if (!fs.existsSync(getConfigPath())) {
    return { defaultWallet: null };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fp = path.join(getAetherDir(), 'wallets', `${address}.json`);
  if (!fs.existsSync(fp)) return null;
  try {
    return JSON.parse(fs.readFileSync(fp, 'utf8'));
  } catch {
    return null;
  }
}

// ============================================================================
// Crypto Helpers
// ============================================================================

function deriveKeypair(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) {
    throw new Error('Invalid mnemonic phrase');
  }
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return {
    publicKey: Buffer.from(keyPair.publicKey),
    secretKey: Buffer.from(keyPair.secretKey),
  };
}

function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

function signTransaction(tx, secretKey) {
  const txBytes = Buffer.from(JSON.stringify(tx));
  const sig = nacl.sign.detached(txBytes, secretKey);
  return bs58.encode(sig);
}

// ============================================================================
// Format Helpers
// ============================================================================

function formatAether(lamports) {
  if (!lamports || lamports === '0') return '0 AETH';
  const aeth = Number(lamports) / 1e9;
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function shortAddress(addr) {
  if (!addr || addr.length < 16) return addr || 'unknown';
  return addr.slice(0, 8) + '...' + addr.slice(-8);
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    address: null,
    stakeAccount: null,
    amount: null,
    rpc: getDefaultRpc(),
    json: false,
    dryRun: false,
    force: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--address' || arg === '-a') {
      opts.address = args[++i];
    } else if (arg === '--account' || arg === '-s') {
      opts.stakeAccount = args[++i];
    } else if (arg === '--amount' || arg === '-m') {
      const val = parseFloat(args[++i]);
      if (!isNaN(val) && val > 0) {
        opts.amount = val;
      }
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--dry-run') {
      opts.dryRun = true;
    } else if (arg === '--force' || arg === '-f') {
      opts.force = true;
    } else if (arg === '--help' || arg === '-h') {
      opts.help = true;
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli unstake${C.reset} — Unstake AETH from a validator

${C.bright}USAGE${C.reset}
    aether unstake --address <wallet> [--account <stakeAcct>] [--amount <aeth>]

${C.bright}REQUIRED${C.reset}
    --address <wallet>   Wallet address with the stake account

${C.bright}OPTIONS${C.reset}
    --account <addr>     Specific stake account to deactivate
    --amount <aeth>      Amount to unstake (default: full stake)
    --rpc <url>          RPC endpoint (default: $AETHER_RPC or localhost:8899)
    --json               Output JSON for scripting
    --dry-run            Preview unstake without submitting
    --force              Skip confirmation prompts
    --help               Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getSlot()            → GET /v1/slot
    client.getStakePositions()  → GET /v1/stake/<addr>
    client.getAccountInfo()     → GET /v1/account/<addr>
    client.sendTransaction()    → POST /v1/transaction

${C.bright}EXAMPLES${C.reset}
    aether unstake --address ATHxxx...
    aether unstake --address ATHxxx... --account Stakexxx... --amount 500
    aether unstake --address ATHxxx... --json --dry-run

${C.bright}NOTES${C.reset}
    • Unstaking begins a cooldown period (typically 1-2 epochs)
    • During cooldown, stake is "deactivating" and earns reduced rewards
    • Once cooldown completes, use 'aether claim' to withdraw to wallet
`);
}

// ============================================================================
// Readline Helpers
// ============================================================================

function createRl() {
  return readline.createInterface({ input: process.stdin, output: process.stdout });
}

function question(rl, q) {
  return new Promise((res) => rl.question(q, res));
}

async function askMnemonic(rl, promptText) {
  console.log(`\n${C.cyan}${promptText}${C.reset}`);
  console.log(`${C.dim}Enter your 12 or 24-word passphrase:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

// ============================================================================
// Fetch Stake Accounts via SDK
// ============================================================================

async function fetchStakeAccounts(rpcUrl, walletAddress) {
  const client = createClient(rpcUrl);
  const rawAddr = walletAddress.startsWith('ATH') ? walletAddress.slice(3) : walletAddress;

  try {
    const stakePositions = await client.getStakePositions(rawAddr);
    if (!Array.isArray(stakePositions)) return [];

    return stakePositions.map(s => ({
      address: s.pubkey || s.publicKey || s.account || s.stake_account,
      validator: s.validator || s.delegate || s.vote_account,
      lamports: s.lamports || s.stake_lamports || s.amount || 0,
      status: s.status || s.state || 'active',
      activationEpoch: s.activation_epoch || s.activationEpoch,
      deactivationEpoch: s.deactivation_epoch || s.deactivationEpoch,
    })).filter(s => s.address);
  } catch (err) {
    return [];
  }
}

// ============================================================================
// Main Unstake Logic
// ============================================================================

async function unstakeCommand() {
  const opts = parseArgs();
  const rl = createRl();

  if (opts.help) {
    showHelp();
    rl.close();
    return;
  }

  // Resolve wallet address
  if (!opts.address) {
    const cfg = loadConfig();
    opts.address = cfg.defaultWallet;
  }

  if (!opts.address) {
    console.log(`\n  ${C.red}✗ No wallet address provided.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether unstake --address <addr> [--account <stakeAcct>]${C.reset}\n`);
    rl.close();
    process.exit(1);
  }

  const wallet = loadWallet(opts.address);
  if (!wallet) {
    console.log(`\n  ${C.red}✗ Wallet not found locally:${C.reset} ${opts.address}`);
    console.log(`  ${C.dim}Import it: aether wallet import${C.reset}\n`);
    rl.close();
    process.exit(1);
  }

  const client = createClient(opts.rpc);

  // Fetch stake accounts via SDK
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Unstake AETH ──────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Fetching stake accounts via SDK...${C.reset}`);
  }

  const stakeAccounts = await fetchStakeAccounts(opts.rpc, opts.address);

  if (stakeAccounts.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'No active stake accounts found',
        address: opts.address,
        suggestion: 'Stake AETH first with: aether stake --validator <addr> --amount <aeth>',
      }, null, 2));
    } else {
      console.log(`\n  ${C.yellow}⚠ No active stake accounts found.${C.reset}`);
      console.log(`  ${C.dim}  Stake AETH first: aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
    }
    rl.close();
    process.exit(1);
  }

  // Filter for active stakes only (can't unstake already deactivating)
  const activeStakes = stakeAccounts.filter(s =>
    s.status === 'active' && !s.deactivationEpoch
  );

  if (activeStakes.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'No active stake positions to unstake',
        address: opts.address,
        stake_accounts: stakeAccounts,
      }, null, 2));
    } else {
      console.log(`\n  ${C.yellow}⚠ No active stake positions to unstake.${C.reset}`);
      console.log(`  ${C.dim}  Current status:${C.reset}`);
      stakeAccounts.forEach(s => {
        const status = s.deactivationEpoch ? 'deactivating' : s.status;
        console.log(`    ${shortAddress(s.address)} → ${status}`);
      });
      console.log();
    }
    rl.close();
    process.exit(1);
  }

  // Select stake account
  let selectedStake = null;

  if (opts.stakeAccount) {
    // User specified a stake account
    selectedStake = activeStakes.find(s =>
      s.address === opts.stakeAccount ||
      s.address.endsWith(opts.stakeAccount)
    );
    if (!selectedStake) {
      console.log(`\n  ${C.red}✗ Stake account not found or not active:${C.reset} ${opts.stakeAccount}`);
      console.log(`  ${C.dim}Active stake accounts:${C.reset}`);
      activeStakes.forEach((s, i) => {
        console.log(`    ${i + 1}) ${shortAddress(s.address)} → ${formatAether(s.lamports)}`);
      });
      console.log();
      rl.close();
      process.exit(1);
    }
  } else if (activeStakes.length === 1) {
    // Only one active stake, use it
    selectedStake = activeStakes[0];
  } else {
    // Multiple active stakes - prompt user to select
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Multiple active stake accounts found. Use --account to specify.',
        address: opts.address,
        active_stakes: activeStakes.map(s => ({
          address: s.address,
          validator: s.validator,
          lamports: s.lamports,
          aeth: formatAether(s.lamports),
        })),
      }, null, 2));
      rl.close();
      process.exit(1);
    }

    console.log(`\n  ${C.bright}Multiple active stake accounts found:${C.reset}\n`);
    activeStakes.forEach((s, i) => {
      const val = s.validator ? shortAddress(s.validator) : 'unknown';
      console.log(`  ${C.green}${i + 1})${C.reset} ${shortAddress(s.address)}`);
      console.log(`      Validator: ${C.cyan}${val}${C.reset}  Amount: ${C.bright}${formatAether(s.lamports)}${C.reset}`);
    });
    console.log();

    const choice = await question(rl, `  ${C.cyan}Select account [1-${activeStakes.length}]:${C.reset} `);
    const idx = parseInt(choice.trim(), 10) - 1;

    if (isNaN(idx) || idx < 0 || idx >= activeStakes.length) {
      console.log(`\n  ${C.red}✗ Invalid selection.${C.reset}\n`);
      rl.close();
      process.exit(1);
    }

    selectedStake = activeStakes[idx];
  }

  // Determine unstake amount
  let unstakeLamports = selectedStake.lamports;
  if (opts.amount) {
    const requestedLamports = Math.round(opts.amount * 1e9);
    if (requestedLamports > selectedStake.lamports) {
      console.log(`\n  ${C.red}✗ Requested amount exceeds staked balance.${C.reset}`);
      console.log(`  ${C.dim}  Requested: ${formatAether(requestedLamports)}${C.reset}`);
      console.log(`  ${C.dim}  Staked:    ${formatAether(selectedStake.lamports)}${C.reset}\n`);
      rl.close();
      process.exit(1);
    }
    unstakeLamports = requestedLamports;
  }

  // Display summary
  if (!opts.json) {
    console.log(`\n  ${C.green}★${C.reset} Wallet:       ${C.bright}${opts.address}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Stake acct:   ${C.bright}${shortAddress(selectedStake.address)}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Validator:    ${C.bright}${shortAddress(selectedStake.validator || 'unknown')}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Amount:       ${C.bright}${opts.amount ? formatAether(unstakeLamports) : 'FULL STAKE'}${C.reset}`);
    console.log(`     ${C.dim}(${unstakeLamports.toLocaleString()} lamports)${C.reset}`);
    console.log();
  }

  if (opts.dryRun) {
    if (opts.json) {
      console.log(JSON.stringify({
        dry_run: true,
        wallet: opts.address,
        stake_account: selectedStake.address,
        validator: selectedStake.validator,
        unstake_lamports: unstakeLamports,
        unstake_aeth: formatAether(unstakeLamports),
        current_stake_lamports: selectedStake.lamports,
        current_stake_aeth: formatAether(selectedStake.lamports),
        rpc: opts.rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`  ${C.yellow}⚠ Dry run mode - no transaction submitted${C.reset}\n`);
    }
    rl.close();
    return;
  }

  // Get mnemonic for signing
  if (!opts.json) {
    console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}\n`);
  }

  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the unstake');
    keyPair = deriveKeypair(mnemonic);

    // Verify derived address matches
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== opts.address) {
      if (opts.json) {
        console.log(JSON.stringify({ success: false, error: 'Passphrase mismatch' }, null, 2));
      } else {
        console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}`);
        console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
        console.log(`  ${C.dim}  Expected: ${opts.address}${C.reset}\n`);
      }
      rl.close();
      process.exit(1);
    }
  } catch (e) {
    if (opts.json) {
      console.log(JSON.stringify({ success: false, error: e.message }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Failed to derive keypair:${C.reset} ${e.message}\n`);
    }
    rl.close();
    process.exit(1);
  }

  // Confirm transaction
  if (!opts.json && !opts.force) {
    const confirm = await question(rl, `  ${C.yellow}Confirm unstake? [y/N]${C.reset} > `);
    if (!confirm.trim().toLowerCase().startsWith('y')) {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      rl.close();
      return;
    }
    console.log();
  }

  rl.close();

  // Build unstake transaction
  const rawWalletAddr = opts.address.startsWith('ATH') ? opts.address.slice(3) : opts.address;

  // Fetch current slot via SDK
  let currentSlot = 0;
  try {
    currentSlot = await client.getSlot();
  } catch (e) {
    // Continue with slot 0
  }

  const tx = {
    signer: rawWalletAddr,
    tx_type: 'Unstake',
    payload: {
      type: 'Unstake',
      data: {
        stake_account: selectedStake.address,
        amount: unstakeLamports,
        validator: selectedStake.validator,
      },
    },
    fee: 5000,
    slot: currentSlot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  // Sign transaction
  tx.signature = signTransaction(tx, keyPair.secretKey);

  if (!opts.json) {
    console.log(`  ${C.dim}Submitting unstake via SDK to ${opts.rpc}...${C.reset}`);
  }

  // Submit via SDK
  try {
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    if (opts.json) {
      console.log(JSON.stringify({
        success: true,
        wallet: opts.address,
        stake_account: selectedStake.address,
        validator: selectedStake.validator,
        unstake_lamports: unstakeLamports,
        unstake_aeth: formatAether(unstakeLamports),
        tx_signature: result.signature || result.txid,
        slot: result.slot || currentSlot,
        rpc: opts.rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.green}✓ Unstake transaction submitted!${C.reset}\n`);
      console.log(`  ${C.dim}Stake Account:${C.reset} ${shortAddress(selectedStake.address)}`);
      console.log(`  ${C.dim}Amount:       ${C.reset}${C.bright}${formatAether(unstakeLamports)}${C.reset}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}Signature:    ${C.reset}${C.cyan}${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log(`  ${C.dim}Slot:         ${C.reset}${result.slot || currentSlot}`);
      console.log();
      console.log(`  ${C.yellow}⚠ Cooldown period started${C.reset}`);
      console.log(`  ${C.dim}  Your stake is now deactivating. Rewards will be reduced during cooldown.${C.reset}`);
      console.log(`  ${C.dim}  Check status: aether stake-positions --address ${opts.address}${C.reset}`);
      console.log(`  ${C.dim}  After cooldown: aether claim --address ${opts.address}${C.reset}\n`);
    }
  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        wallet: opts.address,
        stake_account: selectedStake.address,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Unstake failed:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}Common causes:${C.reset}`);
      console.log(`    • Stake account already deactivating`);
      console.log(`    • Insufficient balance for transaction fee`);
      console.log(`    • RPC endpoint not accepting transactions\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// Entry Point
// ============================================================================

module.exports = { unstakeCommand };

if (require.main === module) {
  unstakeCommand().catch(err => {
    console.error(`\n${C.red}✗ Unstake command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
