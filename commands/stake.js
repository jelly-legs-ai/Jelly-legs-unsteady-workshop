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

// Import UI framework for consistent branding
const { BRANDING, C, indicators, startSpinner, stopSpinner, drawBox, drawTable,
        success, error, warning, info, code, highlight, value } = require('../lib/ui');

const CLI_VERSION = '2.0.0';
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
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
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
    }));
  } catch (err) {
    return [];
  }
}

// ============================================================================
// Show Help
// ============================================================================

function showHelp() {
  console.log(BRANDING.commandBanner('stake', 'Stake AETH to a validator'));
  
  console.log(`\n  ${C.bright}USAGE${C.reset}`);
  console.log(`      ${code('aether stake --validator <addr> --amount <aeth> [--address <wallet>]')}`);
  
  console.log(`\n  ${C.bright}REQUIRED${C.reset}`);
  console.log(`      ${code('--validator <addr>')}   Validator address to stake to`);
  console.log(`      ${code('--amount <aeth>')}      Amount to stake in AETH`);
  
  console.log(`\n  ${C.bright}OPTIONS${C.reset}`);
  console.log(`      ${code('--address <addr>')}     Wallet address (default: configured default)`);
  console.log(`      ${code('--rpc <url>')}          RPC endpoint (default: AETHER_RPC env or localhost:8899)`);
  console.log(`      ${code('--json')}               Output JSON for scripting`);
  console.log(`      ${code('--dry-run')}            Preview stake without submitting`);
  console.log(`      ${code('--list-validators')}    Show available validators on the network`);
  console.log(`      ${code('--force')}              Skip confirmation prompts`);
  
  console.log(`\n  ${C.bright}SDK METHODS USED${C.reset}`);
  console.log(`      ${C.dim}client.getValidators()     → GET /v1/validators${C.reset}`);
  console.log(`      ${C.dim}client.getAccountInfo()    → GET /v1/account/<addr>${C.reset}`);
  console.log(`      ${C.dim}client.getSlot()           → GET /v1/slot${C.reset}`);
  console.log(`      ${C.dim}client.sendTransaction()   → POST /v1/transaction${C.reset}`);
  
  console.log(`\n  ${C.bright}EXAMPLES${C.reset}`);
  console.log(`      ${C.dim}$${C.reset} ${code('aether stake --validator ATHxxx... --amount 1000')}`);
  console.log(`      ${C.dim}$${C.reset} ${code('aether stake --validator ATHxxx... --amount 1000 --address ATHxxx...')}`);
  console.log(`      ${C.dim}$${C.reset} ${code('aether stake --list-validators')}`);
  console.log(`      ${C.dim}$${C.reset} ${code('aether stake --validator ATHxxx... --amount 1000 --dry-run')}`);
  
  console.log(`\n  ${C.bright}MINIMUM STAKE AMOUNTS${C.reset}`);
  console.log(`      Full:     10,000 AETH`);
  console.log(`      Lite:      1,000 AETH`);
  console.log(`      Observer:      0 AETH`);
  
  console.log(`\n  ${C.bright}NOTES${C.reset}`);
  console.log(`      ${C.dim}◆ Staked AETH begins earning rewards after one epoch (~2 days)${C.reset}`);
  console.log(`      ${C.dim}◆ Use 'aether stake-positions' to view your delegations${C.reset}`);
  console.log(`      ${C.dim}◆ Use 'aether unstake' to withdraw (has cooldown period)${C.reset}`);
  console.log();
}

// ============================================================================
// List Validators Command
// ============================================================================

async function listValidatorsCommand(opts) {
  if (!opts.json) {
    console.log(BRANDING.commandBanner('stake --list-validators', 'Available Validators'));
    console.log(`\n  ${C.dim}Fetching validators from ${opts.rpc}...${C.reset}\n`);
    startSpinner('Querying validators');
  }

  const validators = await fetchValidators(opts.rpc);

  if (!opts.json) {
    stopSpinner(true, 'Validators retrieved');
  }

  if (validators.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'No validators found',
        rpc: opts.rpc,
      }, null, 2));
    } else {
      console.log(`\n  ${warning('No validators found.')}`);
      console.log(`  ${C.dim}Check your RPC endpoint: ${opts.rpc}${C.reset}\n`);
    }
    return;
  }

  validators.sort((a, b) => b.stake - a.stake);

  if (opts.json) {
    console.log(JSON.stringify({
      success: true,
      count: validators.length,
      validators: validators.map(v => ({
        address: v.address,
        name: v.name,
        stake_aeth: v.stake / 1e9,
        apy: v.apy,
        tier: v.tier,
      })),
    }, null, 2));
    return;
  }

  // Build table rows
  const rows = validators.slice(0, 15).map((v, i) => {
    const status = v.active ? indicators.success : indicators.warning;
    const name = (v.name || 'Unknown').slice(0, 20);
    const addr = shortAddress(v.address);
    const stake = formatAether(v.stake);
    const apy = formatPercent(v.apy);
    return [status, `${i + 1}`, name, addr, stake, apy];
  });

  console.log(`\n  ${C.bright}Found ${C.cyan}${validators.length}${C.reset} validators\n`);
  
  console.log(drawTable(
    ['', '#', 'Name', 'Address', 'Stake', 'APY'],
    rows,
    { borderStyle: 'single', headerColor: C.cyan + C.bright }
  ));

  if (validators.length > 15) {
    console.log(`\n  ${C.dim}... and ${validators.length - 15} more validators (use --json for full list)${C.reset}`);
  }

  console.log(`\n  ${C.dim}To stake: ${code('aether stake --validator <addr> --amount <aeth>')}${C.reset}\n`);
}

// ============================================================================
// Main Stake Logic
// ============================================================================

async function stakeCommand() {
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

  // Parse args
  const args = process.argv.slice(3);
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--validator' || arg === '-v') opts.validator = args[++i];
    else if (arg === '--amount' || arg === '-m') {
      const val = parseFloat(args[++i]);
      if (!isNaN(val) && val > 0) opts.amount = val;
    }
    else if (arg === '--address' || arg === '-a') opts.address = args[++i];
    else if (arg === '--rpc' || arg === '-r') opts.rpc = args[++i];
    else if (arg === '--json' || arg === '-j') opts.json = true;
    else if (arg === '--dry-run') opts.dryRun = true;
    else if (arg === '--list-validators' || arg === '-l') opts.listValidators = true;
    else if (arg === '--force' || arg === '-f') opts.force = true;
    else if (arg === '--help' || arg === '-h') {
      showHelp();
      return;
    }
  }

  // List validators mode
  if (opts.listValidators) {
    await listValidatorsCommand(opts);
    return;
  }

  const rl = createRl();

  // Resolve wallet address
  if (!opts.address) {
    const cfg = loadConfig();
    opts.address = cfg.defaultWallet;
  }

  if (!opts.address) {
    console.log(`\n  ${error('No wallet address.')} Use --address <addr> or set a default.`);
    console.log(`  ${C.dim}Usage: aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
    rl.close();
    return;
  }

  // Check wallet exists
  const wallet = loadWallet(opts.address);
  if (!wallet) {
    console.log(`\n  ${error('Wallet not found:')} ${opts.address}`);
    console.log(`  ${C.dim}Import it: ${code('aether wallet import')}${C.reset}\n`);
    rl.close();
    return;
  }

  // Fetch balance via SDK
  let balance = 0;
  const client = createClient(opts.rpc);
  const rawAddr = opts.address.startsWith('ATH') ? opts.address.slice(3) : opts.address;
  
  if (!opts.json) {
    startSpinner('Fetching wallet balance');
  }
  
  try {
    const account = await client.getAccountInfo(rawAddr);
    balance = account.lamports || 0;
  } catch (err) {
    if (!opts.json) console.log(`  ${warning('Could not fetch balance:')} ${err.message}`);
  }

  if (!opts.json) {
    stopSpinner(true, 'Balance retrieved');
  }

  // Interactive validator selection
  let validator = opts.validator;
  if (!validator) {
    if (!opts.json) {
      console.log(`\n  ${C.dim}Fetching validators...${C.reset}`);
      startSpinner('Querying network');
    }
    
    const validators = await fetchValidators(opts.rpc);
    
    if (!opts.json) {
      stopSpinner(true, 'Validators retrieved');
    }
    
    if (validators.length === 0) {
      console.log(`\n  ${error('No validators found.')}\n`);
      rl.close();
      return;
    }
    
    validators.sort((a, b) => b.stake - a.stake);
    console.log(`\n  ${C.bright}Select a validator:${C.reset}\n`);
    
    validators.slice(0, 10).forEach((v, i) => {
      const name = (v.name || 'Unknown').slice(0, 18).padEnd(18);
      const stake = formatAether(v.stake);
      const apy = formatPercent(v.apy);
      console.log(`  ${C.green}${i + 1})${C.reset} ${name} | ${stake} | ${apy}`);
    });
    
    console.log(`\n  ${C.dim}Enter number [1-10] or validator address${C.reset}`);
    const choice = await question(rl, `  Validator > ${C.reset}`);
    const choiceNum = parseInt(choice.trim(), 10);
    
    if (!isNaN(choiceNum) && choiceNum >= 1 && choiceNum <= 10) {
      validator = validators[choiceNum - 1].address;
    } else {
      validator = choice.trim();
    }
  }

  // Resolve amount
  let amount = opts.amount;
  if (!amount) {
    console.log(`\n  ${C.cyan}Available:${C.reset} ${formatAether(balance)}`);
    console.log(`  ${C.dim}Minimum: Full=10K, Lite=1K, Observer=0${C.reset}`);
    const amt = await question(rl, `  Amount (AETH) > ${C.reset}`);
    amount = parseFloat(amt);
    if (isNaN(amount) || amount <= 0) {
      console.log(`\n  ${error('Invalid amount.')}\n`);
      rl.close();
      return;
    }
  }

  const stakeLamports = Math.round(amount * 1e9);
  const feeBuffer = 0.005 * 1e9;

  if (stakeLamports + feeBuffer > balance) {
    console.log(`\n  ${error('Insufficient balance.')}`);
    console.log(`  Requested: ${formatAether(stakeLamports)}`);
    console.log(`  Balance:   ${formatAether(balance)}\n`);
    rl.close();
    return;
  }

  // Display summary in a box
  if (!opts.json) {
    console.log();
    console.log(drawBox(
      `
${C.bright}STAKE SUMMARY${C.reset}

${C.cyan}Wallet:${C.reset}     ${C.bright}${opts.address}${C.reset}
${C.cyan}Validator:${C.reset}  ${C.bright}${shortAddress(validator)}${C.reset}
${C.cyan}Amount:${C.reset}     ${C.green}${formatAether(stakeLamports)}${C.reset}
${C.cyan}Balance:${C.reset}    ${formatAether(balance)}
      `.trim(),
      { style: 'single', title: 'TRANSACTION', titleColor: C.cyan }
    ));
    console.log();
  }

  // Dry run
  if (opts.dryRun) {
    console.log(JSON.stringify({
      dry_run: true,
      wallet: opts.address,
      validator: validator,
      stake_lamports: stakeLamports,
      stake_aeth: amount,
      balance_aeth: balance / 1e9,
      rpc: opts.rpc,
      cli_version: CLI_VERSION,
    }, null, 2));
    rl.close();
    return;
  }

  // Sign
  console.log(`${warning('Signing requires your wallet passphrase.')}`);
  const mnemonic = await askMnemonic(rl, 'Enter passphrase to sign this transaction');

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic);
  } catch (e) {
    console.log(`\n  ${error('Failed to derive keypair:')} ${e.message}\n`);
    rl.close();
    return;
  }

  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== opts.address) {
    console.log(`\n  ${error('Passphrase mismatch!')}`);
    console.log(`  Expected: ${opts.address}`);
    console.log(`  Derived:  ${derivedAddress}\n`);
    rl.close();
    return;
  }

  // Confirm
  if (!opts.force) {
    const confirm = await question(rl, `\n  ${warning('Confirm stake? [y/N]')} > `);
    if (!confirm.trim().toLowerCase().startsWith('y')) {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get slot and build transaction
  let slot = 0;
  try { slot = await client.getSlot(); } catch (e) {}

  const tx = {
    signer: rawAddr,
    tx_type: 'Stake',
    payload: {
      type: 'Stake',
      data: { validator: validator, amount: stakeLamports },
    },
    fee: 5000,
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  tx.signature = signTransaction(tx, keyPair.secretKey);

  if (!opts.json) {
    console.log(`\n  ${C.dim}Submitting to ${opts.rpc}...${C.reset}`);
    startSpinner('Submitting transaction');
  }

  try {
    const result = await client.sendTransaction(tx);
    if (result.error) throw new Error(result.error);

    if (!opts.json) {
      stopSpinner(true, 'Transaction submitted');
      
      console.log();
      console.log(BRANDING.successBanner('Stake transaction submitted!'));
      console.log(`\n  ${C.cyan}Wallet:${C.reset}    ${opts.address}`);
      console.log(`  ${C.cyan}Validator:${C.reset} ${validator}`);
      console.log(`  ${C.cyan}Amount:${C.reset}    ${C.bright}${formatAether(stakeLamports)}${C.reset}`);
      if (result.signature) console.log(`  ${C.cyan}Signature:${C.reset} ${result.signature.slice(0, 40)}...`);
      console.log(`  ${C.cyan}Slot:${C.reset}      ${result.slot || slot}`);
      console.log();
      console.log(`  ${info('Stake will activate in the next epoch')}`);
      console.log(`  ${C.dim}Check: ${code(`aether stake-positions --address ${opts.address}`)}${C.reset}\n`);
    } else {
      console.log(JSON.stringify({
        success: true,
        wallet: opts.address,
        validator: validator,
        amount: stakeLamports,
        signature: result.signature,
        slot: result.slot || slot,
      }, null, 2));
    }
  } catch (err) {
    if (!opts.json) {
      stopSpinner(false, 'Transaction failed');
      console.log(`\n  ${error('Stake failed:')} ${err.message}\n`);
    } else {
      console.log(JSON.stringify({ success: false, error: err.message }, null, 2));
    }
  }

  rl.close();
}

// ============================================================================
// Entry Point
// ============================================================================

module.exports = { stakeCommand };

if (require.main === module) {
  stakeCommand().catch(err => {
    console.error(`\n${error('Stake command failed:')} ${err.message}`);
    process.exit(1);
  });
}
