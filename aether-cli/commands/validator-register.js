#!/usr/bin/env node
/**
 * aether-cli validator-register
 *
 * Register a validator with the Aether network via RPC.
 * This is the FINAL step of validator onboarding - submits identity
 * and stake to the chain, creating an on-chain validator record.
 *
 * Usage:
 *   aether validator register --identity <path> --validator <addr> --amount <aeth>
 *   aether validator register --tier full --stake 10000
 *   aether validator register --json
 *
 * SDK wired to: POST /v1/validator/register, GET /v1/validators, GET /v1/epoch
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');
const crypto = require('crypto');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const bip39 = require('bip39');

// Import SDK for blockchain RPC calls
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

// Tier requirements
const TIER_REQUIREMENTS = {
  full: { minStake: 10000, minCores: 8, minRam: 32, minDisk: 512 },
  lite: { minStake: 1000, minCores: 4, minRam: 8, minDisk: 100 },
  observer: { minStake: 0, minCores: 2, minRam: 4, minDisk: 50 },
};

// ============================================================================
// Config & Paths
// ============================================================================

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function loadConfig() {
  if (!fs.existsSync(getConfigPath())) {
    return { defaultWallet: null, validators: [] };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { defaultWallet: null, validators: [] };
  }
}

function saveConfig(cfg) {
  if (!fs.existsSync(getAetherDir())) {
    fs.mkdirSync(getAetherDir(), { recursive: true });
  }
  fs.writeFileSync(getConfigPath(), JSON.stringify(cfg, null, 2));
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

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  // Handle both direct invocation and subcommand invocation
  // Direct: node validator-register.js --wallet ...
  // Subcommand: aether validator register --wallet ...
  let args = process.argv.slice(2);
  
  // If called as subcommand, strip 'validator' and 'register' from args
  if (args[0] === 'validator' && args[1] === 'register') {
    args = args.slice(2);
  } else if (args[0] === 'register') {
    args = args.slice(1);
  }
  
  const opts = {
    identity: null,
    wallet: null,
    validator: null,
    amount: null,
    tier: 'full',
    commission: 10, // Default 10% commission
    name: null,
    rpc: getDefaultRpc(),
    json: false,
    dryRun: false,
    force: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--identity' || arg === '-i') {
      opts.identity = args[++i];
    } else if (arg === '--wallet' || arg === '-w') {
      opts.wallet = args[++i];
    } else if (arg === '--validator' || arg === '-v') {
      opts.validator = args[++i];
    } else if (arg === '--amount' || arg === '-a') {
      const val = args[++i];
      opts.amount = parseFloat(val);
      if (isNaN(opts.amount)) {
        console.error(`  ${C.red}✗ Invalid amount:${C.reset} ${val}`);
        process.exit(1);
      }
    } else if (arg === '--tier' || arg === '-t') {
      opts.tier = (args[++i] || 'full').toLowerCase();
      if (!['full', 'lite', 'observer'].includes(opts.tier)) {
        console.error(`  ${C.red}✗ Invalid tier:${C.reset} ${opts.tier}. Valid: full, lite, observer`);
        process.exit(1);
      }
    } else if (arg === '--commission' || arg === '-c') {
      const val = parseInt(args[++i], 10);
      if (isNaN(val) || val < 0 || val > 100) {
        console.error(`  ${C.red}✗ Invalid commission:${C.reset} must be 0-100`);
        process.exit(1);
      }
      opts.commission = val;
    } else if (arg === '--name' || arg === '-n') {
      opts.name = args[++i];
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--dry-run') {
      opts.dryRun = true;
    } else if (arg === '--force' || arg === '-f') {
      opts.force = true;
    } else if (arg === '--help' || arg === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli validator register${C.reset} — Register validator with the network

${C.bright}USAGE${C.reset}
    aether validator register --wallet <addr> --amount <aeth> [options]

${C.bright}REQUIRED${C.reset}
    --wallet <addr>      Wallet address to stake from (ATH...)
    --amount <aeth>      Amount of AETH to stake (minimum per tier)

${C.bright}OPTIONS${C.reset}
    --identity <path>    Path to validator-identity.json (default: ./validator-identity.json)
    --validator <addr>   Validator vote account address (default: derive from wallet)
    --tier <type>        Validator tier: full, lite, observer (default: full)
    --commission <n>     Commission percentage 0-100 (default: 10)
    --name <string>      Validator name/moniker
    --rpc <url>          RPC endpoint (default: $AETHER_RPC or localhost:8899)
    --dry-run            Preview registration without submitting
    --json               Output JSON for scripting
    --force              Skip confirmation prompts
    --help               Show this help

${C.bright}TIER REQUIREMENTS${C.reset}
    full:     10,000 AETH stake, 8 cores, 32GB RAM, 512GB SSD
    lite:     1,000 AETH stake, 4 cores, 8GB RAM, 100GB SSD
    observer: 0 AETH stake, 2 cores, 4GB RAM, 50GB disk

${C.bright}SDK METHODS USED${C.reset}
    client.getSlot()           → GET /v1/slot
    client.getEpochInfo()      → GET /v1/epoch
    client.getBalance()        → GET /v1/account/<addr>
    client.sendTransaction()   → POST /v1/validator/register
    client.getValidators()     → GET /v1/validators (verify)

${C.bright}EXAMPLES${C.reset}
    aether validator register --wallet ATHxxx... --amount 10000 --tier full
    aether validator register --wallet ATHxxx... --amount 1000 --tier lite --name "MyLiteNode"
    aether validator register --wallet ATHxxx... --amount 0 --tier observer --json
`);
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
// Validation Helpers
// ============================================================================

async function validatePrerequisites(opts, client) {
  const errors = [];

  // Check identity file
  const identityPath = opts.identity || path.join(process.cwd(), 'validator-identity.json');
  if (!fs.existsSync(identityPath)) {
    errors.push(`Identity file not found: ${identityPath}`);
    errors.push(`Run 'aether init' first to generate validator identity`);
  } else {
    try {
      const identity = JSON.parse(fs.readFileSync(identityPath, 'utf8'));
      if (!identity.pubkey || !identity.secret) {
        errors.push('Invalid identity file: missing pubkey or secret');
      }
    } catch (e) {
      errors.push(`Failed to parse identity file: ${e.message}`);
    }
  }

  // Check wallet exists
  if (!opts.wallet) {
    errors.push('No wallet address provided (--wallet)');
  } else {
    const wallet = loadWallet(opts.wallet);
    if (!wallet) {
      errors.push(`Wallet not found: ${opts.wallet}`);
      errors.push(`Run 'aether wallet import' to add this wallet`);
    }
  }

  // Check minimum stake for tier
  const minStake = TIER_REQUIREMENTS[opts.tier].minStake;
  if (opts.amount === null || opts.amount === undefined) {
    errors.push(`No stake amount provided (--amount)`);
  } else if (opts.amount < minStake) {
    errors.push(`Stake amount ${opts.amount} AETH below minimum ${minStake} AETH for ${opts.tier} tier`);
  }

  // Check RPC connectivity via SDK
  try {
    const slot = await client.getSlot();
    if (typeof slot !== 'number') {
      errors.push('RPC endpoint not responding with valid slot data');
    }
  } catch (e) {
    errors.push(`RPC endpoint unreachable: ${e.message}`);
  }

  // Check wallet balance via SDK
  if (opts.wallet) {
    try {
      const rawAddr = opts.wallet.startsWith('ATH') ? opts.wallet.slice(3) : opts.wallet;
      const balance = await client.getBalance(rawAddr);
      const required = opts.amount * 1e9 + 0.005 * 1e9; // stake + fee buffer
      if (balance < required) {
        errors.push(`Insufficient balance: ${(balance / 1e9).toFixed(4)} AETH, need ${(required / 1e9).toFixed(4)} AETH`);
      }
    } catch (e) {
      errors.push(`Failed to check wallet balance: ${e.message}`);
    }
  }

  return errors;
}

function formatAether(lamports) {
  const aeth = (lamports || 0) / 1e9;
  return aeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 }) + ' AETH';
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
// Core Registration Logic
// ============================================================================

async function registerValidator(opts) {
  const client = createClient(opts.rpc);
  const rl = createRl();

  // Parse identity
  const identityPath = opts.identity || path.join(process.cwd(), 'validator-identity.json');
  let identity;
  try {
    identity = JSON.parse(fs.readFileSync(identityPath, 'utf8'));
  } catch (e) {
    console.error(`\n  ${C.red}✗ Failed to load identity:${C.reset} ${e.message}`);
    process.exit(1);
  }

  // Validate prerequisites
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Validating Prerequisites ─────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Checking identity, wallet, RPC, and balance...${C.reset}`);
  }

  const errors = await validatePrerequisites(opts, client);
  if (errors.length > 0) {
    if (opts.json) {
      console.log(JSON.stringify({ success: false, errors }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Validation failed:${C.reset}\n`);
      errors.forEach(e => console.log(`    ${C.red}•${C.reset} ${e}`));
      console.log();
    }
    rl.close();
    process.exit(1);
  }

  if (!opts.json) {
    console.log(`  ${C.green}✓ All prerequisites validated${C.reset}\n`);
  }

  // Fetch current network state via SDK
  const [slot, epochInfo] = await Promise.all([
    client.getSlot().catch(() => null),
    client.getEpochInfo().catch(() => null),
  ]);

  // Build registration transaction
  const lamports = Math.round(opts.amount * 1e9);
  const validatorAddr = opts.validator || opts.wallet;
  const rawValidatorAddr = validatorAddr.startsWith('ATH') ? validatorAddr.slice(3) : validatorAddr;
  const rawWalletAddr = opts.wallet.startsWith('ATH') ? opts.wallet.slice(3) : opts.wallet;

  const registration = {
    identity_pubkey: identity.pubkey,
    vote_account: rawValidatorAddr,
    stake_account: rawWalletAddr,
    stake_lamports: lamports,
    tier: opts.tier,
    commission_bps: opts.commission * 100, // Convert to basis points
    name: opts.name || `Validator-${identity.pubkey.slice(0, 8)}`,
    registered_at: new Date().toISOString(),
    slot: slot || 0,
    epoch: epochInfo?.epoch || 0,
  };

  if (opts.json) {
    if (opts.dryRun) {
      console.log(JSON.stringify({
        dry_run: true,
        registration,
        identity_path: identityPath,
        rpc: opts.rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
      rl.close();
      return;
    }
  } else {
    // Display registration summary
    console.log(`${C.bright}${C.cyan}── Validator Registration ─────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.green}★${C.reset} Identity:    ${C.bright}${identity.pubkey.slice(0, 20)}...${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Vote Acct:   ${C.bright}${validatorAddr}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Stake From:  ${C.bright}${opts.wallet}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Amount:      ${C.bright}${formatAether(lamports)}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Tier:        ${C.bright}${opts.tier.toUpperCase()}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Commission:  ${C.bright}${opts.commission}%${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Name:        ${C.bright}${registration.name}${C.reset}`);
    console.log(`  ${C.dim}   RPC:        ${opts.rpc}${C.reset}`);
    console.log(`  ${C.dim}   Current Slot: ${slot || 'unknown'}${C.reset}`);
    console.log(`  ${C.dim}   Epoch:      ${epochInfo?.epoch || 'unknown'}${C.reset}\n`);

    if (opts.dryRun) {
      console.log(`  ${C.yellow}⚠ Dry run mode - no transaction submitted${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get mnemonic for signing
  let keyPair;
  if (!opts.json) {
    console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}\n`);
  }

  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the registration');
    keyPair = deriveKeypair(mnemonic);

    // Verify derived address matches wallet
    const derivedAddr = formatAddress(keyPair.publicKey);
    if (derivedAddr !== opts.wallet) {
      if (!opts.json) {
        console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}`);
        console.log(`  ${C.dim}  Derived:  ${derivedAddr}${C.reset}`);
        console.log(`  ${C.dim}  Expected: ${opts.wallet}${C.reset}\n`);
      } else {
        console.log(JSON.stringify({ success: false, error: 'Passphrase mismatch' }, null, 2));
      }
      rl.close();
      process.exit(1);
    }
  } catch (e) {
    if (!opts.json) {
      console.log(`\n  ${C.red}✗ Failed to derive keypair:${C.reset} ${e.message}\n`);
    } else {
      console.log(JSON.stringify({ success: false, error: e.message }, null, 2));
    }
    rl.close();
    process.exit(1);
  }

  // Confirm registration
  if (!opts.json && !opts.force) {
    const confirm = await question(rl, `  ${C.yellow}Confirm registration? [y/N]${C.reset} > `);
    if (!confirm.trim().toLowerCase().startsWith('y')) {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      rl.close();
      return;
    }
    console.log();
  }

  rl.close();

  // Build and sign registration transaction
  const tx = {
    signer: rawWalletAddr,
    tx_type: 'ValidatorRegister',
    payload: {
      type: 'ValidatorRegister',
      data: registration,
    },
    fee: 5000, // Registration fee in lamports
    slot: slot || 0,
    timestamp: Math.floor(Date.now() / 1000),
  };

  // Sign with wallet keypair
  tx.signature = signTransaction(tx, keyPair.secretKey);

  if (!opts.json) {
    console.log(`  ${C.dim}Submitting registration to ${opts.rpc}...${C.reset}`);
  }

  // Submit via SDK
  try {
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    // Save to local config
    const cfg = loadConfig();
    if (!cfg.validators) cfg.validators = [];
    cfg.validators.push({
      identity: identity.pubkey,
      vote_account: validatorAddr,
      tier: opts.tier,
      registered_at: new Date().toISOString(),
      tx_signature: result.signature || result.txid,
    });
    saveConfig(cfg);

    if (opts.json) {
      console.log(JSON.stringify({
        success: true,
        registration: {
          identity: identity.pubkey,
          vote_account: validatorAddr,
          stake_lamports: lamports,
          tier: opts.tier,
          commission_bps: opts.commission,
        },
        tx_signature: result.signature || result.txid,
        slot: result.slot || slot,
        rpc: opts.rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.green}✓ Validator registered successfully!${C.reset}\n`);
      console.log(`  ${C.dim}Identity:${C.reset}    ${identity.pubkey}`);
      console.log(`  ${C.dim}Vote Acct:${C.reset}   ${validatorAddr}`);
      console.log(`  ${C.dim}Tier:${C.reset}        ${opts.tier.toUpperCase()}`);
      console.log(`  ${C.dim}Stake:${C.reset}       ${formatAether(lamports)}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}Signature:${C.reset}   ${(result.signature || result.txid).slice(0, 40)}...`);
      }
      if (result.slot) {
        console.log(`  ${C.dim}Slot:${C.reset}        ${result.slot}`);
      }
      console.log();
      console.log(`  ${C.dim}Next steps:${C.reset}`);
      console.log(`    ${C.cyan}aether validator status${C.reset}     Check validator status`);
      console.log(`    ${C.cyan}aether validators list${C.reset}      View all validators`);
      console.log(`    ${C.cyan}aether delegations list${C.reset}    View your delegations\n`);
    }
  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        registration: { identity: identity.pubkey, vote_account: validatorAddr },
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Registration failed:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}Common causes:${C.reset}`);
      console.log(`    • Validator with this identity already registered`);
      console.log(`    • Insufficient balance for stake + fees`);
      console.log(`    • RPC endpoint not accepting transactions`);
      console.log(`    • Network epoch boundary - retry in a few slots\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function validatorRegisterCommand() {
  const opts = parseArgs();
  await registerValidator(opts);
}

module.exports = { validatorRegisterCommand };

if (require.main === module) {
  validatorRegisterCommand().catch(err => {
    console.error(`\n${C.red}✗ Unexpected error:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
