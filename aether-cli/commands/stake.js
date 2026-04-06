#!/usr/bin/env node
/**
 * aether-cli stake
 *
 * First-class stake command - stake AETH to a validator.
 * Fully wired to @jellylegsai/aether-sdk for real blockchain RPC calls.
 *
 * Usage:
 *   aether stake --validator <addr> --amount <aeth> [--address <wallet>]
 *   aether stake --validator ATHxxx... --amount 1000
 *   aether stake --validator ATHxxx... --amount 1000 --address ATHxxx...
 *   aether stake --list-validators  # Show available validators to stake to
 *   aether stake --dry-run          # Preview without submitting
 *
 * SDK wired to:
 *   - client.getValidators()        → GET /v1/validators
 *   - client.getAccountInfo()       → GET /v1/account/<addr>
 *   - client.getSlot()              → GET /v1/slot
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

// Minimum stake amounts by tier
const MIN_STAKE = {
  full: 10000,
  lite: 1000,
  observer: 0,
};

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

function formatPercent(val) {
  if (val === undefined || val === null) return 'N/A';
  return val.toFixed(2) + '%';
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    validator: null,
    amount: null,
    address: null,
    rpc: getDefaultRpc(),
    json: false,
    dryRun: false,
    listValidators: false,
    force: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--validator' || arg === '-v') {
      opts.validator = args[++i];
    } else if (arg === '--amount' || arg === '-m') {
      const val = parseFloat(args[++i]);
      if (!isNaN(val) && val > 0) {
        opts.amount = val;
      }
    } else if (arg === '--address' || arg === '-a') {
      opts.address = args[++i];
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--dry-run') {
      opts.dryRun = true;
    } else if (arg === '--list-validators' || arg === '-l') {
      opts.listValidators = true;
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
${C.bright}${C.cyan}aether-cli stake${C.reset} — Stake AETH to a validator

${C.bright}USAGE${C.reset}
    aether stake --validator <addr> --amount <aeth> [--address <wallet>]

${C.bright}REQUIRED${C.reset}
    --validator <addr>   Validator address to stake to
    --amount <aeth>      Amount to stake in AETH

${C.bright}OPTIONS${C.reset}
    --address <addr>     Wallet address (default: configured default)
    --rpc <url>          RPC endpoint (default: AETHER_RPC env or localhost:8899)
    --json               Output JSON for scripting
    --dry-run            Preview stake without submitting
    --list-validators    Show available validators on the network
    --force              Skip confirmation prompts
    --help               Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getValidators()     → GET /v1/validators
    client.getAccountInfo()    → GET /v1/account/<addr>
    client.getSlot()           → GET /v1/slot
    client.sendTransaction()   → POST /v1/transaction

${C.bright}EXAMPLES${C.reset}
    aether stake --validator ATHxxx... --amount 1000
    aether stake --validator ATHxxx... --amount 500 --address ATHxxx...
    aether stake --list-validators
    aether stake --validator ATHxxx... --amount 1000 --dry-run

${C.bright}MINIMUM STAKE AMOUNTS${C.reset}
    Full:     10,000 AETH
    Lite:      1,000 AETH  
    Observer:      0 AETH

${C.bright}NOTES${C.reset}
    • Staked AETH begins earning rewards after one epoch (~2 days)
    • Use 'aether stake-positions' to view your delegations
    • Use 'aether unstake' to withdraw (has cooldown period)
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
// Fetch Validators via SDK
// ============================================================================

async function fetchValidators(rpcUrl) {
  const client = createClient(rpcUrl);
  
  try {
    const validators = await client.getValidators();
    if (!Array.isArray(validators)) return [];
    
    return validators.map(v => ({
      address: v.vote_account || v.pubkey || v.address || v.identity,
      identity: v.identity || v.node_pubkey,
      stake: v.stake_lamports || v.activated_stake || v.stake || 0,
      commission: v.commission || v.commission_bps || 0,
      apy: v.apy || v.return_rate || 0,
      name: v.name || v.moniker || 'Unknown',
      tier: v.tier || 'unknown',
      active: v.active !== false && v.delinquent !== true,
    })).filter(v => v.address);
  } catch (err) {
    return [];
  }
}

// ============================================================================
// List Validators Command
// ============================================================================

async function listValidatorsCommand(opts, rl) {
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Available Validators ───────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Fetching validators from ${opts.rpc}...${C.reset}\n`);
  }

  const validators = await fetchValidators(opts.rpc);

  if (validators.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'No validators found',
        rpc: opts.rpc,
        suggestion: 'Check RPC endpoint or wait for network sync',
      }, null, 2));
    } else {
      console.log(`  ${C.yellow}⚠ No validators found.${C.reset}`);
      console.log(`  ${C.dim}  RPC: ${opts.rpc}${C.reset}`);
      console.log(`  ${C.dim}  Check network status: aether network${C.reset}\n`);
    }
    return;
  }

  // Sort by stake descending
  validators.sort((a, b) => b.stake - a.stake);

  if (opts.json) {
    console.log(JSON.stringify({
      success: true,
      count: validators.length,
      rpc: opts.rpc,
      validators: validators.map(v => ({
        address: v.address,
        identity: v.identity,
        name: v.name,
        stake_lamports: v.stake,
        stake_aeth: v.stake / 1e9,
        commission: v.commission,
        apy: v.apy,
        tier: v.tier,
        active: v.active,
      })),
    }, null, 2));
    return;
  }

  console.log(`  ${C.bright}Found ${validators.length} validators${C.reset}\n`);
  
  console.log(`  ${C.dim}┌${'─'.repeat(78)}┐${C.reset}`);
  console.log(`  ${C.dim}│${C.reset} ${C.bright}#  Name                 Address              Stake(AETH)  APY    Commission${C.reset}${C.dim} │${C.reset}`);
  console.log(`  ${C.dim}├${'─'.repeat(78)}┤${C.reset}`);

  validators.slice(0, 20).forEach((v, i) => {
    const status = v.active ? C.green + '●' : C.yellow + '○';
    const name = (v.name || 'Unknown').slice(0, 18).padEnd(18);
    const addr = shortAddress(v.address).padEnd(20);
    const stake = formatAether(v.stake).padStart(11);
    const apy = formatPercent(v.apy).padStart(6);
    const commission = (v.commission / 100).toFixed(1) + '%';
    
    console.log(`  ${C.dim}│${C.reset} ${status}${C.reset} ${(i + 1).toString().padStart(2)} ${name} ${addr} ${stake} ${apy} ${commission.padStart(10)}${C.dim} │${C.reset}`);
  });

  if (validators.length > 20) {
    console.log(`  ${C.dim}│${C.reset} ${C.dim}... and ${validators.length - 20} more validators${C.reset}${' '.repeat(53)}${C.dim} │${C.reset}`);
  }

  console.log(`  ${C.dim}└${'─'.repeat(78)}┘${C.reset}\n`);
  console.log(`  ${C.dim}Stake to a validator:${C.reset}`);
  console.log(`    ${C.cyan}aether stake --validator <address> --amount <aeth>${C.reset}\n`);
}

// ============================================================================
// Main Stake Logic
// ============================================================================

async function stakeCommand() {
  const opts = parseArgs();
  const rl = createRl();

  if (opts.help) {
    showHelp();
    rl.close();
    return;
  }

  // Handle list-validators mode
  if (opts.listValidators) {
    await listValidatorsCommand(opts, rl);
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
    console.log(`  ${C.dim}Usage: aether stake --validator <addr> --amount <aeth> [--address <wallet>]${C.reset}\n`);
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

  // Check wallet balance via SDK
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Stake AETH ─────────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Fetching wallet balance via SDK...${C.reset}`);
  }

  let balance = 0;
  try {
    const rawAddr = opts.address.startsWith('ATH') ? opts.address.slice(3) : opts.address;
    const account = await client.getAccountInfo(rawAddr);
    balance = account.lamports || 0;
    
    if (!opts.json) {
      console.log(`  ${C.green}✓${C.reset} Balance: ${C.bright}${formatAether(balance)}${C.reset}\n`);
    }
  } catch (err) {
    if (!opts.json) {
      console.log(`  ${C.yellow}⚠${C.reset} Could not fetch balance: ${err.message}\n`);
    }
  }

  // Resolve validator - if not provided, fetch list and prompt
  if (!opts.validator) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Validator address required. Use --validator <addr> or --list-validators',
      }, null, 2));
      rl.close();
      process.exit(1);
    }

    console.log(`  ${C.dim}Fetching validators...${C.reset}\n`);
    const validators = await fetchValidators(opts.rpc);
    
    if (validators.length === 0) {
      console.log(`  ${C.red}✗ No validators found on the network.${C.reset}`);
      console.log(`  ${C.dim}  Check RPC: ${opts.rpc}${C.reset}\n`);
      rl.close();
      process.exit(1);
    }

    // Sort by stake and show top validators
    validators.sort((a, b) => b.stake - a.stake);
    
    console.log(`  ${C.bright}Select a validator to stake to:${C.reset}\n`);
    validators.slice(0, 10).forEach((v, i) => {
      const name = (v.name || 'Unknown').slice(0, 20).padEnd(20);
      const stake = formatAether(v.stake);
      const apy = formatPercent(v.apy);
      console.log(`  ${C.green}${i + 1})${C.reset} ${name} | Stake: ${C.cyan}${stake}${C.reset} | APY: ${apy}`);
      console.log(`     ${C.dim}${v.address}${C.reset}\n`);
    });
    console.log(`  ${C.dim}Enter validator address directly, or select by number [1-10]${C.reset}\n`);

    const choice = await question(rl, `  Validator > ${C.reset}`);
    const choiceNum = parseInt(choice.trim(), 10);
    
    if (!isNaN(choiceNum) && choiceNum >= 1 && choiceNum <= 10) {
      opts.validator = validators[choiceNum - 1].address;
    } else if (choice.trim().startsWith('ATH')) {
      opts.validator = choice.trim();
    } else {
      console.log(`\n  ${C.red}✗ Invalid validator selection.${C.reset}\n`);
      rl.close();
      process.exit(1);
    }
  }

  // Resolve amount - if not provided, prompt
  if (!opts.amount) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Amount required. Use --amount <aeth>',
      }, null, 2));
      rl.close();
      process.exit(1);
    }

    const availableAeth = balance / 1e9;
    console.log(`  ${C.dim}Available balance: ${formatAether(balance)}${C.reset}`);
    console.log(`  ${C.dim}Minimum stakes: Full=10K, Lite=1K, Observer=0${C.reset}\n`);
    
    const amountStr = await question(rl, `  Amount to stake (AETH) > ${C.reset}`);
    const amount = parseFloat(amountStr);
    
    if (isNaN(amount) || amount <= 0) {
      console.log(`\n  ${C.red}✗ Invalid amount: ${amountStr}${C.reset}\n`);
      rl.close();
      process.exit(1);
    }
    
    opts.amount = amount;
  }

  const stakeLamports = Math.round(opts.amount * 1e9);

  // Validate amount against balance (including fee buffer)
  const feeBuffer = 0.01 * 1e9; // 0.01 AETH for fees
  if (stakeLamports + feeBuffer > balance && balance > 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Insufficient balance for stake + fees',
        requested_aeth: opts.amount,
        requested_lamports: stakeLamports,
        balance_aeth: balance / 1e9,
        balance_lamports: balance,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Insufficient balance.${C.reset}`);
      console.log(`  ${C.dim}  Requested: ${formatAether(stakeLamports)}${C.reset}`);
      console.log(`  ${C.dim}  Balance:   ${formatAether(balance)}${C.reset}`);
      console.log(`  ${C.dim}  Need ~0.01 AETH for fees${C.reset}\n`);
    }
    rl.close();
    process.exit(1);
  }

  // Display summary
  if (!opts.json) {
    console.log(`\n  ${C.green}★${C.reset} Wallet:    ${C.bright}${opts.address}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Validator: ${C.bright}${shortAddress(opts.validator)}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Amount:    ${C.bright}${formatAether(stakeLamports)}${C.reset}`);
    console.log(`     ${C.dim}(${stakeLamports.toLocaleString()} lamports)${C.reset}`);
    console.log();
  }

  if (opts.dryRun) {
    if (opts.json) {
      console.log(JSON.stringify({
        dry_run: true,
        wallet: opts.address,
        validator: opts.validator,
        stake_lamports: stakeLamports,
        stake_aeth: opts.amount,
        balance_lamports: balance,
        balance_aeth: balance / 1e9,
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
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the stake');
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
    const confirm = await question(rl, `  ${C.yellow}Confirm stake? [y/N]${C.reset} > `);
    if (!confirm.trim().toLowerCase().startsWith('y')) {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      rl.close();
      return;
    }
    console.log();
  }

  rl.close();

  // Build stake transaction
  const rawWalletAddr = opts.address.startsWith('ATH') ? opts.address.slice(3) : opts.address;
  const rawValidatorAddr = opts.validator.startsWith('ATH') ? opts.validator.slice(3) : opts.validator;

  // Fetch current slot via SDK
  let currentSlot = 0;
  try {
    currentSlot = await client.getSlot();
  } catch (e) {
    // Continue with slot 0
  }

  const tx = {
    signer: rawWalletAddr,
    tx_type: 'Stake',
    payload: {
      type: 'Stake',
      data: {
        validator: rawValidatorAddr,
        amount: stakeLamports,
      },
    },
    fee: 5000,
    slot: currentSlot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  // Sign transaction
  tx.signature = signTransaction(tx, keyPair.secretKey);

  if (!opts.json) {
    console.log(`  ${C.dim}Submitting stake via SDK to ${opts.rpc}...${C.reset}`);
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
        validator: opts.validator,
        stake_lamports: stakeLamports,
        stake_aeth: opts.amount,
        tx_signature: result.signature || result.txid,
        slot: result.slot || currentSlot,
        rpc: opts.rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.green}✓ Stake transaction submitted!${C.reset}\n`);
      console.log(`  ${C.dim}Wallet:    ${C.reset}${shortAddress(opts.address)}`);
      console.log(`  ${C.dim}Validator: ${C.reset}${shortAddress(opts.validator)}`);
      console.log(`  ${C.dim}Amount:    ${C.reset}${C.bright}${formatAether(stakeLamports)}${C.reset}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}Signature: ${C.reset}${C.cyan}${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log(`  ${C.dim}Slot:      ${C.reset}${result.slot || currentSlot}`);
      console.log();
      console.log(`  ${C.green}✓ Stake will activate in the next epoch${C.reset}`);
      console.log(`  ${C.dim}  Check positions: aether stake-positions --address ${opts.address}${C.reset}`);
      console.log(`  ${C.dim}  View rewards:    aether rewards --address ${opts.address}${C.reset}\n`);
    }
  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        wallet: opts.address,
        validator: opts.validator,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Stake failed:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}Common causes:${C.reset}`);
      console.log(`    • Validator address not found on chain`);
      console.log(`    • Insufficient balance for transaction fee`);
      console.log(`    • RPC endpoint not accepting transactions\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// Entry Point
// ============================================================================

module.exports = { stakeCommand };

if (require.main === module) {
  stakeCommand().catch(err => {
    console.error(`\n${C.red}✗ Stake command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
