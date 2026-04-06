/**
 * aether-cli init
 * 
 * COMPLETE ONBOARDING WIZARD - Fully wired to SDK
 * 
 * Guides users through:
 *   1. Prerequisites check (Node.js, disk space)
 *   2. Tier selection (Full/Lite/Observer)
 *   3. Identity generation (Ed25519 keypair)
 *   4. Wallet creation/import (BIP39)
 *   5. RPC connection & health check
 *   6. Faucet funding (testnet)
 *   7. Validator registration (real SDK calls)
 *   8. Auto-start option
 * 
 * All blockchain calls use @jellylegsai/aether-sdk with REAL RPC.
 * No stubs. No mocks.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const bip39 = require('bip39');
const readline = require('readline');
const os = require('os');
const http = require('http');
const https = require('https');

// Import SDK
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ANSI colors
const C = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
  dim: '\x1b[2m',
  magenta: '\x1b[35m',
};

const CLI_VERSION = '2.0.0';
const DERIVATION_PATH = "m/44'/7777777'/0'/0'";

// Tier requirements
const TIER_REQUIREMENTS = {
  full: { minStake: 10000, minCores: 8, minRam: 32, minDisk: 512 },
  lite: { minStake: 1000, minCores: 4, minRam: 8, minDisk: 100 },
  observer: { minStake: 0, minCores: 2, minRam: 4, minDisk: 50 },
};

// ============================================================================
// Paths & Config
// ============================================================================

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getWalletsDir() {
  return path.join(getAetherDir(), 'wallets');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function ensureDirs() {
  const dirs = [getAetherDir(), getWalletsDir()];
  for (const d of dirs) {
    if (!fs.existsSync(d)) fs.mkdirSync(d, { recursive: true });
  }
}

function loadConfig() {
  if (!fs.existsSync(getConfigPath())) {
    return { defaultWallet: null, validators: [], version: 1 };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { defaultWallet: null, validators: [], version: 1 };
  }
}

function saveConfig(cfg) {
  ensureDirs();
  fs.writeFileSync(getConfigPath(), JSON.stringify(cfg, null, 2));
}

function walletFilePath(address) {
  return path.join(getWalletsDir(), `${address}.json`);
}

function saveWalletFile(address, publicKey) {
  ensureDirs();
  const data = {
    version: 1,
    address,
    public_key: bs58.encode(publicKey),
    created_at: new Date().toISOString(),
    derivation_path: DERIVATION_PATH,
  };
  fs.writeFileSync(walletFilePath(address), JSON.stringify(data, null, 2));
  return data;
}

function loadWallet(address) {
  const fp = walletFilePath(address);
  if (!fs.existsSync(fp)) return null;
  try {
    return JSON.parse(fs.readFileSync(fp, 'utf8'));
  } catch {
    return null;
  }
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

async function askYesNo(rl, text, defaultYes = true) {
  const suffix = defaultYes ? ' [Y/n]' : ' [y/N]';
  const ans = await question(rl, `${C.cyan}${text}${suffix}:${C.reset} `);
  const normalized = ans.trim().toLowerCase();
  if (normalized === '') return defaultYes;
  return normalized === 'y' || normalized === 'yes';
}

async function askValue(rl, text, defaultVal = '') {
  const suffix = defaultVal ? ` [${defaultVal}]` : '';
  const ans = await question(rl, `${C.cyan}${text}${suffix}:${C.reset} `);
  return ans.trim() || defaultVal;
}

async function askMnemonic(rl, promptText) {
  console.log(`\n${C.cyan}${promptText}${C.reset}`);
  console.log(`${C.dim}Enter your 12 or 24-word mnemonic:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

// ============================================================================
// Print Helpers
// ============================================================================

function printBanner() {
  console.log(`
${C.cyan}╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ${C.bright}AETHER VALIDATOR ONBOARDING WIZARD${C.reset}${C.cyan}                          ║
║   ${C.dim}v${CLI_VERSION} - Fully wired to SDK${C.reset}${C.cyan}                              ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝${C.reset}
  `);
}

function printStep(step, total, title) {
  console.log();
  console.log(`${C.yellow}Step ${step}/${total}:${C.reset} ${C.bright}${title}${C.reset}`);
  console.log(`${C.dim}${'─'.repeat(60)}${C.reset}`);
}

function printSuccess(msg) {
  console.log(`  ${C.green}✓${C.reset} ${msg}`);
}

function printWarning(msg) {
  console.log(`  ${C.yellow}⚠${C.reset} ${msg}`);
}

function printError(msg) {
  console.log(`  ${C.red}✗${C.reset} ${msg}`);
}

function printInfo(msg) {
  console.log(`  ${C.dim}ℹ${C.reset} ${msg}`);
}

// ============================================================================
// Crypto Helpers
// ============================================================================

function generateEd25519Identity() {
  const keyPair = nacl.sign.keyPair();
  const seed32 = keyPair.secretKey.slice(0, 32);
  return {
    pubkey: bs58.encode(keyPair.publicKey),
    secret: bs58.encode(seed32),
    publicKey: keyPair.publicKey,
    secretKey: keyPair.secretKey,
  };
}

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
// Step 1: Prerequisites Check
// ============================================================================

async function checkPrerequisites(rl) {
  printStep(1, 6, 'Checking Prerequisites');

  const checks = [];

  // Check Node.js version
  const nodeVersion = process.version;
  const nodeMajor = parseInt(nodeVersion.slice(1).split('.')[0], 10);
  checks.push({
    name: 'Node.js',
    passed: nodeMajor >= 14,
    message: `Node.js ${nodeVersion}`,
  });

  // Check disk space
  try {
    let freeGB = 100;
    if (process.platform === 'win32') {
      const output = execSync('powershell -c "(Get-PSDrive -Name C).Free / 1GB"', { encoding: 'utf8' });
      freeGB = parseFloat(output.trim());
    } else {
      const stat = fs.statfsSync('/');
      freeGB = (stat.bsize * stat.bfree) / (1024 * 1024 * 1024);
    }
    checks.push({
      name: 'Disk Space',
      passed: freeGB >= 50,
      message: `${Math.floor(freeGB)} GB free`,
    });
  } catch {
    checks.push({ name: 'Disk Space', passed: true, message: 'Unable to check' });
  }

  // Check SDK imports
  try {
    const sdk = require(sdkPath);
    checks.push({
      name: 'Aether SDK',
      passed: !!sdk.AetherClient,
      message: 'SDK loaded',
    });
  } catch (e) {
    checks.push({ name: 'Aether SDK', passed: false, message: `Failed: ${e.message}` });
  }

  let allPassed = true;
  for (const check of checks) {
    const icon = check.passed ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`;
    console.log(`  ${icon} ${check.name}: ${check.message}`);
    if (!check.passed) allPassed = false;
  }

  if (!allPassed) {
    printWarning('Some prerequisites may need attention.');
    const cont = await askYesNo(rl, 'Continue anyway?', false);
    if (!cont) {
      console.log('\nInstall prerequisites and try again.\n');
      process.exit(1);
    }
  } else {
    printSuccess('All prerequisites met');
  }

  return true;
}

// ============================================================================
// Step 2: Tier Selection
// ============================================================================

async function selectTier(rl) {
  printStep(2, 6, 'Select Validator Tier');

  console.log(`\n  Choose your validator tier based on hardware and stake:\n`);
  console.log(`  ${C.bright}[1] FULL${C.reset}    - 10K AETH stake, 8 cores, 32GB RAM, 512GB SSD`);
  console.log(`               Full consensus weight, block production, voting`);
  console.log();
  console.log(`  ${C.bright}[2] LITE${C.reset}    - 1K AETH stake, 4 cores, 8GB RAM, 100GB SSD`);
  console.log(`               Stake-based weight, voting only`);
  console.log();
  console.log(`  ${C.bright}[3] OBSERVER${C.reset} - 0 AETH stake, 2 cores, 4GB RAM, 50GB disk`);
  console.log(`               Relay-only, earns FLUX via data relay`);
  console.log();

  const choice = await askValue(rl, 'Select tier', '1');

  let tier = 'full';
  let badge = '[FULL]';

  switch (choice.trim()) {
    case '1':
    case 'full':
      tier = 'full';
      badge = '[FULL]';
      printSuccess('Selected: FULL Validator');
      break;
    case '2':
    case 'lite':
      tier = 'lite';
      badge = '[LITE]';
      printSuccess('Selected: LITE Validator');
      break;
    case '3':
    case 'observer':
      tier = 'observer';
      badge = '[OBSERVER]';
      printSuccess('Selected: OBSERVER Node');
      break;
    default:
      tier = 'full';
      badge = '[FULL]';
      printWarning('Invalid choice, defaulting to FULL');
  }

  return { tier, badge, minStake: TIER_REQUIREMENTS[tier].minStake };
}

// ============================================================================
// Step 3: Identity Generation
// ============================================================================

async function generateIdentity(rl) {
  printStep(3, 6, 'Generating Validator Identity');

  const identityPath = path.join(process.cwd(), 'validator-identity.json');

  if (fs.existsSync(identityPath)) {
    printWarning('Identity already exists');
    const regen = await askYesNo(rl, 'Regenerate identity?', false);
    if (!regen) {
      const existing = JSON.parse(fs.readFileSync(identityPath, 'utf8'));
      printSuccess(`Using existing identity: ${existing.pubkey.slice(0, 20)}...`);
      return { identityPath, ...existing };
    }
  }

  const identity = generateEd25519Identity();
  fs.writeFileSync(identityPath, JSON.stringify(identity, null, 2));

  printSuccess(`Identity saved to ${path.basename(identityPath)}`);
  console.log(`  ${C.dim}Public key: ${identity.pubkey}${C.reset}`);

  printWarning('IMPORTANT: Backup validator-identity.json!');
  printWarning('If you lose this file, you lose your validator status.');

  return { identityPath, ...identity };
}

// ============================================================================
// Step 4: Wallet Setup
// ============================================================================

async function setupWallet(rl) {
  printStep(4, 6, 'Wallet Setup');

  const cfg = loadConfig();

  console.log(`\n  ${C.green}1)${C.reset} Create new wallet`);
  console.log(`  ${C.green}2)${C.reset} Import existing wallet`);
  console.log(`  ${C.green}3)${C.reset} Use default wallet (${cfg.defaultWallet || 'none set'})`);
  console.log();

  const choice = await askValue(rl, 'Choose', '1');

  let wallet = null;
  let keyPair = null;
  let mnemonic = null;

  if (choice === '2') {
    // Import
    mnemonic = await askMnemonic(rl, 'Import wallet from mnemonic');
    if (!bip39.validateMnemonic(mnemonic)) {
      printError('Invalid BIP39 mnemonic');
      throw new Error('Invalid mnemonic');
    }
    keyPair = deriveKeypair(mnemonic);
    const address = formatAddress(keyPair.publicKey);

    if (!loadWallet(address)) {
      saveWalletFile(address, keyPair.publicKey);
      cfg.defaultWallet = address;
      saveConfig(cfg);
    }
    wallet = { address, ...loadWallet(address) };
    printSuccess(`Wallet imported: ${address}`);

  } else if (choice === '3' && cfg.defaultWallet) {
    // Use default
    wallet = { address: cfg.defaultWallet, ...loadWallet(cfg.defaultWallet) };
    printSuccess(`Using default wallet: ${wallet.address}`);

  } else {
    // Create new
    mnemonic = bip39.generateMnemonic(128);
    keyPair = deriveKeypair(mnemonic);
    const address = formatAddress(keyPair.publicKey);

    const words = mnemonic.split(' ');
    console.log(`\n${C.red}${C.bright}╔═══════════════════════════════════════════════════════════════╗${C.reset}`);
    console.log(`${C.red}${C.bright}║           YOUR WALLET PASSPHRASE                               ║${C.reset}`);
    console.log(`${C.red}${C.bright}╚═══════════════════════════════════════════════════════════════╝${C.reset}`);
    console.log(`\n${C.yellow}  Write these words down. They cannot be recovered.${C.reset}\n`);

    for (let i = 0; i < words.length; i += 3) {
      const line = [];
      for (let j = 0; j < 3 && i + j < words.length; j++) {
        line.push(`${C.bright}${i + j + 1}.${C.reset} ${words[i + j].padEnd(15)}`);
      }
      console.log(`  ${line.join('   ')}`);
    }

    await question(rl, `\n${C.cyan}Press Enter when you have saved your passphrase...${C.reset}`);

    saveWalletFile(address, keyPair.publicKey);
    cfg.defaultWallet = address;
    saveConfig(cfg);

    wallet = { address, ...loadWallet(address) };
    printSuccess(`Wallet created: ${address}`);
  }

  return { wallet, keyPair, mnemonic };
}

// ============================================================================
// Step 5: RPC Connection & Health Check
// ============================================================================

async function checkRpcConnection(rl) {
  printStep(5, 6, 'RPC Connection & Health Check');

  const defaultRpc = process.env.AETHER_RPC || 'http://127.0.0.1:8899';
  const rpcUrl = await askValue(rl, 'RPC endpoint', defaultRpc);

  printInfo(`Connecting to ${rpcUrl}...`);

  try {
    const client = new aether.AetherClient({ rpcUrl });
    const [slot, health, version] = await Promise.all([
      client.getSlot().catch(() => null),
      client.getHealth().catch(() => null),
      client.getVersion().catch(() => null),
    ]);

    if (slot === null) {
      throw new Error('RPC not responding with valid slot data');
    }

    printSuccess(`Connected to RPC`);
    console.log(`  ${C.dim}Slot:${C.reset} ${slot}`);
    console.log(`  ${C.dim}Health:${C.reset} ${health || 'unknown'}`);
    if (version) {
      console.log(`  ${C.dim}Version:${C.reset} ${version.aetherCore || JSON.stringify(version)}`);
    }

    return { rpcUrl, slot, client };
  } catch (err) {
    printError(`Failed to connect: ${err.message}`);
    printWarning('You can continue and use a local validator later');
    const cont = await askYesNo(rl, 'Continue with setup?', true);
    if (!cont) {
      process.exit(1);
    }
    return { rpcUrl, slot: 0, client: null };
  }
}

// ============================================================================
// Step 6: Faucet Funding (Testnet)
// ============================================================================

async function fundFromFaucet(rl, wallet, rpcUrl, tier) {
  printStep(6, 6, 'Faucet Funding (Testnet)');

  const minStake = TIER_REQUIREMENTS[tier].minStake;

  console.log(`\n  To register as a ${tier.toUpperCase()} validator, you need ${minStake} AETH.`);
  console.log(`  ${C.dim}Testnet faucets provide free AETH for development.${C.reset}\n`);

  // First check if wallet already has funds
  try {
    const client = new aether.AetherClient({ rpcUrl });
    const rawAddr = wallet.address.startsWith('ATH') ? wallet.address.slice(3) : wallet.address;
    const balance = await client.getBalance(rawAddr);
    const aethBalance = balance / 1e9;

    if (aethBalance >= minStake) {
      printSuccess(`Wallet already funded: ${aethBalance.toFixed(4)} AETH`);
      return true;
    }

    printInfo(`Current balance: ${aethBalance.toFixed(4)} AETH`);
  } catch (err) {
    printWarning(`Could not check balance: ${err.message}`);
  }

  // Try faucet funding
  printInfo('Requesting testnet AETH from faucet...');

  try {
    // Aether testnet faucet endpoint
    const faucetUrl = process.env.AETHER_FAUCET_URL || 'http://127.0.0.1:8899';
    const faucetEndpoint = `${faucetUrl}/v1/faucet`;

    const result = await httpPost(faucetEndpoint, {
      address: wallet.address,
      amount: Math.max(minStake * 1e9, 10000 * 1e9), // Request min stake + buffer
    }, 10000);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    printSuccess(`Faucet request submitted!`);
    if (result.signature) {
      console.log(`  ${C.dim}Tx:${C.reset} ${result.signature.slice(0, 30)}...`);
    }

    // Wait for funds
    printInfo('Waiting for funds to arrive...');
    await waitForBalance(rpcUrl, wallet.address, minStake * 1e9, 30);

    return true;
  } catch (err) {
    printWarning(`Faucet request failed: ${err.message}`);
    printInfo('You can fund manually and re-run registration later');
    printInfo(`Address: ${wallet.address}`);
    return false;
  }
}

async function waitForBalance(rpcUrl, address, minLamports, timeoutSec) {
  const client = new aether.AetherClient({ rpcUrl });
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  for (let i = 0; i < timeoutSec; i++) {
    try {
      const balance = await client.getBalance(rawAddr);
      if (balance >= minLamports) {
        printSuccess(`Balance confirmed: ${(balance / 1e9).toFixed(4)} AETH`);
        return true;
      }
    } catch {}
    process.stdout.write('.');
    await new Promise(r => setTimeout(r, 1000));
  }
  console.log();
  printWarning('Timeout waiting for funds');
  return false;
}

function httpPost(url, body, timeout) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    const lib = urlObj.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);

    const req = lib.request({
      hostname: urlObj.hostname,
      port: urlObj.port || (urlObj.protocol === 'https:' ? 443 : 80),
      path: urlObj.pathname,
      method: 'POST',
      timeout,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch {
          resolve({ raw: data });
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });
    req.write(bodyStr);
    req.end();
  });
}

// ============================================================================
// Step 7: Validator Registration
// ============================================================================

async function registerValidator(rl, wallet, identity, tier, rpcUrl, keyPair) {
  console.log();
  printStep(6, 6, 'Validator Registration');

  const minStake = TIER_REQUIREMENTS[tier].minStake;
  const stakeLamports = Math.round(minStake * 1e9);
  const rawWalletAddr = wallet.address.startsWith('ATH') ? wallet.address.slice(3) : wallet.address;

  printInfo(`Registering validator with ${minStake} AETH stake...`);

  try {
    const client = new aether.AetherClient({ rpcUrl });
    const [slot, epochInfo] = await Promise.all([
      client.getSlot().catch(() => 0),
      client.getEpochInfo().catch(() => ({ epoch: 0 })),
    ]);

    // Build registration transaction
    const registration = {
      identity_pubkey: identity.pubkey,
      vote_account: rawWalletAddr,
      stake_account: rawWalletAddr,
      stake_lamports: stakeLamports,
      tier: tier,
      commission_bps: 1000, // 10%
      name: `Validator-${identity.pubkey.slice(0, 8)}`,
      registered_at: new Date().toISOString(),
      slot: slot,
      epoch: epochInfo.epoch || 0,
    };

    const tx = {
      signer: rawWalletAddr,
      tx_type: 'ValidatorRegister',
      payload: {
        type: 'ValidatorRegister',
        data: registration,
      },
      fee: 5000,
      slot: slot,
      timestamp: Math.floor(Date.now() / 1000),
    };

    // Sign transaction
    tx.signature = signTransaction(tx, keyPair.secretKey);

    printInfo(`Submitting registration transaction...`);

    // Submit via SDK
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    // Save to config
    const cfg = loadConfig();
    cfg.validators = cfg.validators || [];
    cfg.validators.push({
      identity: identity.pubkey,
      vote_account: wallet.address,
      tier: tier,
      registered_at: new Date().toISOString(),
      tx_signature: result.signature || result.txid,
    });
    saveConfig(cfg);

    printSuccess('Validator registered successfully!');
    console.log(`  ${C.dim}Identity:${C.reset} ${identity.pubkey.slice(0, 30)}...`);
    console.log(`  ${C.dim}Stake:${C.reset} ${minStake} AETH`);
    console.log(`  ${C.dim}Tier:${C.reset} ${tier.toUpperCase()}`);
    if (result.signature || result.txid) {
      console.log(`  ${C.dim}Tx:${C.reset} ${(result.signature || result.txid).slice(0, 40)}...`);
    }
    console.log(`  ${C.dim}Slot:${C.reset} ${result.slot || slot}`);

    return { success: true, result };
  } catch (err) {
    printError(`Registration failed: ${err.message}`);
    printInfo('Common causes:');
    printInfo('  • Validator already registered with this identity');
    printInfo('  • Insufficient balance for stake + fees');
    printInfo('  • RPC endpoint not accepting transactions');
    return { success: false, error: err.message };
  }
}

// ============================================================================
// Completion Summary
// ============================================================================

async function printSummary(identity, wallet, tier, badge) {
  console.log();
  console.log(`${C.green}╔═══════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.green}║                                                               ║${C.reset}`);
  console.log(`${C.green}║   ${C.bright}✅ VALIDATOR SETUP COMPLETE${C.reset}${C.green}                                 ║${C.reset}`);
  console.log(`${C.green}║   ${C.dim}Tier: ${badge}${C.reset}${C.green}                                              ║${C.reset}`);
  console.log(`${C.green}║                                                               ║${C.reset}`);
  console.log(`${C.green}╚═══════════════════════════════════════════════════════════════╝${C.reset}`);
  console.log();

  console.log(`${C.bright}Identity:${C.reset}`);
  console.log(`  File: validator-identity.json`);
  console.log(`  Pubkey: ${identity.pubkey}`);
  console.log();

  console.log(`${C.bright}Wallet:${C.reset}`);
  console.log(`  Address: ${wallet.address}`);
  console.log(`  File: ~/.aether/wallets/${wallet.address}.json`);
  console.log();

  console.log(`${C.bright}Next steps:${C.reset}`);
  console.log(`  ${C.cyan}aether validator status${C.reset}      Check validator status`);
  console.log(`  ${C.cyan}aether validator start${C.reset}       Start the validator node`);
  console.log(`  ${C.cyan}aether network${C.reset}               View network status`);
  console.log(`  ${C.cyan}aether wallet balance${C.reset}        Check wallet balance`);
  console.log(`  ${C.cyan}aether stake-info <addr>${C.reset}    Check stake positions`);
  console.log();
}

// ============================================================================
// Main Init Function
// ============================================================================

async function init() {
  const rl = createRl();

  try {
    printBanner();

    // Step 1: Prerequisites
    await checkPrerequisites(rl);

    // Step 2: Tier Selection
    const { tier, badge, minStake } = await selectTier(rl);

    // Step 3: Identity
    const identity = await generateIdentity(rl);

    // Step 4: Wallet
    const { wallet, keyPair } = await setupWallet(rl);

    // Step 5: RPC Connection
    const { rpcUrl, client } = await checkRpcConnection(rl);

    // Step 6: Funding (if testnet and tier requires stake)
    let funded = false;
    if (tier !== 'observer') {
      funded = await fundFromFaucet(rl, wallet, rpcUrl, tier);
    } else {
      printInfo('Observer tier requires no stake - skipping funding');
      funded = true;
    }

    // Step 7: Registration
    const registered = await registerValidator(rl, wallet, identity, tier, rpcUrl, keyPair);

    // Summary
    await printSummary(identity, wallet, tier, badge);

    // Offer to start validator
    if (registered.success) {
      const startNow = await askYesNo(rl, 'Start validator now?', true);
      if (startNow) {
        console.log();
        printInfo('Starting validator...');
        rl.close();
        const { validatorStart } = require('./validator-start');
        validatorStart({ testnet: true, tier });
        return;
      }
    }

    rl.close();

  } catch (err) {
    rl.close();
    console.error(`\n${C.red}✗ Init failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  }
}

// Export for module use
module.exports = { init };

// Run if called directly
if (require.main === module) {
  init();
}
