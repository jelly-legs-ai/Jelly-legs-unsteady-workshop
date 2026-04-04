/**
 * aether-cli wallet
 *
 * Aether wallet management:
 *   aether wallet create   — Create new BIP39 wallet or import existing
 *   aether wallet list     — List all wallets
 *   aether wallet import   — Import wallet from mnemonic
 *   aether wallet default  — Show/set default wallet
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');
const crypto = require('crypto');
const { execSync } = require('child_process');
const bip39 = require('bip39');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;

// ANSI colours
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

// Derivation path for Aether wallets
const DERIVATION_PATH = "m/44'/7777777'/0'/0'";

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

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
  const wd = getWalletsDir();
  if (!fs.existsSync(wd)) fs.mkdirSync(wd, { recursive: true });
}

function loadConfig() {
  const p = getConfigPath();
  if (!fs.existsSync(p)) return { defaultWallet: null, version: 1 };
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null, version: 1 };
  }
}

function saveConfig(cfg) {
  ensureDirs();
  fs.writeFileSync(getConfigPath(), JSON.stringify(cfg, null, 2));
}

// ---------------------------------------------------------------------------
// Crypto helpers
// ---------------------------------------------------------------------------

/**
 * Derive an Ed25519 keypair from a BIP39 seed.
 * BIP39 seed → 64-byte seed → TweetNaCl keypair
 */
function deriveKeypair(mnemonic, derivationPath) {
  // Validate mnemonic first
  if (!bip39.validateMnemonic(mnemonic)) {
    throw new Error('Invalid mnemonic phrase.');
  }

  // BIP39 seed (512 bits = 64 bytes)
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, ''); // passphrase empty for now

  // TweetNaCl keypair from 32-byte seed
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);

  return {
    publicKey: Buffer.from(keyPair.publicKey),
    secretKey: Buffer.from(keyPair.secretKey),
  };
}

/**
 * Format Aether address: ATH + base58check of public key.
 * base58check = bs58.encode(publicKey) — TweetNaCl pubkeys are 32 bytes
 * so bs58 encoding itself acts as the check.
 */
function formatAddress(publicKey) {
  const encoded = bs58.encode(publicKey);
  return 'ATH' + encoded;
}

// ---------------------------------------------------------------------------
// Session management helpers
// ---------------------------------------------------------------------------

const CLI_VERSION = '1.0.3';

function getSessionsDir() {
  return path.join(getAetherDir(), 'sessions');
}

function ensureSessionsDir() {
  const sd = getSessionsDir();
  if (!fs.existsSync(sd)) fs.mkdirSync(sd, { recursive: true });
}

/** Generate a UUID v4 session token */
function generateSessionToken() {
  return crypto.randomUUID();
}

/** Save a new session file to ~/.aether/sessions/<token>.json */
function saveSession(token, wallet_address, expires_in_minutes = 10) {
  ensureSessionsDir();
  const now = new Date();
  const expires_at = new Date(now.getTime() + expires_in_minutes * 60 * 1000);
  const session = {
    wallet_address,
    created_at: now.toISOString(),
    expires_at: expires_at.toISOString(),
    verified: false,
    cli_version: CLI_VERSION,
  };
  fs.writeFileSync(
    path.join(getSessionsDir(), `${token}.json`),
    JSON.stringify(session, null, 2)
  );
  return session;
}

/** Load a session file, or return null if missing/expired */
function getSession(token) {
  const fp = path.join(getSessionsDir(), `${token}.json`);
  if (!fs.existsSync(fp)) return null;
  try {
    const session = JSON.parse(fs.readFileSync(fp, 'utf8'));
    // Check expiry
    if (new Date(session.expires_at) < new Date()) {
      return null;
    }
    return session;
  } catch {
    return null;
  }
}

/** Mark a session as verified (called by external verification flow) */
function markSessionVerified(token) {
  const fp = path.join(getSessionsDir(), `${token}.json`);
  if (!fs.existsSync(fp)) return false;
  try {
    const session = JSON.parse(fs.readFileSync(fp, 'utf8'));
    session.verified = true;
    fs.writeFileSync(fp, JSON.stringify(session, null, 2));
    return true;
  } catch {
    return false;
  }
}

/** Delete a session file */
function deleteSession(token) {
  const fp = path.join(getSessionsDir(), `${token}.json`);
  if (fs.existsSync(fp)) {
    fs.unlinkSync(fp);
  }
}

/** Poll until session is verified or timeout expires */
function pollForVerification(token, timeout_ms = 600000) {
  const interval_ms = 2000;
  const max_retries = Math.floor(timeout_ms / interval_ms);

  for (let i = 0; i < max_retries; i++) {
    const session = getSession(token);
    if (session && session.verified) {
      return { verified: true, session };
    }
    // Check if session expired
    if (!session) {
      return { verified: false, reason: 'expired' };
    }
    // Sleep for interval
    const sleep = (ms) => new Promise((res) => setTimeout(res, ms));
    sleep(interval_ms);
  }
  return { verified: false, reason: 'timeout' };
}

// ---------------------------------------------------------------------------
// Wallet file helpers
// ---------------------------------------------------------------------------

function walletFilePath(address) {
  return path.join(getWalletsDir(), `${address}.json`);
}

function loadWallet(address) {
  const fp = walletFilePath(address);
  if (!fs.existsSync(fp)) return null;
  return JSON.parse(fs.readFileSync(fp, 'utf8'));
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

// ---------------------------------------------------------------------------
// Readline helpers
// ---------------------------------------------------------------------------

function createRl() {
  return readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
}

function question(rl, q) {
  return new Promise((res) => rl.question(q, res));
}

async function askMnemonic(rl, questionText) {
  console.log(`\n${C.cyan}${questionText}${C.reset}`);
  console.log(`${C.dim}Enter your ${C.bright}12 or 24${C.reset}${C.dim}-word mnemonic phrase, one space-separated line:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

// ---------------------------------------------------------------------------
// Session management helpers
// ---------------------------------------------------------------------------

function getSessionsDir() {
  return path.join(getAetherDir(), 'sessions');
}

function sessionFilePath(token) {
  return path.join(getSessionsDir(), `${token}.json`);
}

/**
 * Generate a UUID v4 session token
 */
function generateSessionToken() {
  return crypto.randomUUID();
}

/**
 * Save session to ~/.aether/sessions/<uuid>.json
 */
function saveSession(token, wallet_address, expires_in_minutes = 10) {
  const sessionsDir = getSessionsDir();
  if (!fs.existsSync(sessionsDir)) {
    fs.mkdirSync(sessionsDir, { recursive: true });
  }
  
  const now = new Date();
  const expiresAt = new Date(now.getTime() + expires_in_minutes * 60 * 1000);
  
  const sessionData = {
    wallet_address: wallet_address,
    created_at: now.toISOString(),
    expires_at: expiresAt.toISOString(),
    verified: false,
    cli_version: '1.0.3',
  };
  
  fs.writeFileSync(sessionFilePath(token), JSON.stringify(sessionData, null, 2));
  return sessionData;
}

/**
 * Load session from file, returns null if not found
 */
function getSession(token) {
  const fp = sessionFilePath(token);
  if (!fs.existsSync(fp)) return null;
  try {
    return JSON.parse(fs.readFileSync(fp, 'utf8'));
  } catch {
    return null;
  }
}

/**
 * Mark session as verified
 */
function markSessionVerified(token) {
  const session = getSession(token);
  if (!session) return false;
  session.verified = true;
  fs.writeFileSync(sessionFilePath(token), JSON.stringify(session, null, 2));
  return true;
}

/**
 * Delete session file
 */
function deleteSession(token) {
  const fp = sessionFilePath(token);
  if (fs.existsSync(fp)) {
    fs.unlinkSync(fp);
    return true;
  }
  return false;
}

/**
 * Check if session has expired
 */
function isSessionExpired(session) {
  if (!session || !session.expires_at) return true;
  return new Date(session.expires_at) < new Date();
}

/**
 * Poll for verification until verified=true or expired
 * Returns { verified: boolean, session: object|null }
 */
async function pollForVerification(token, timeout_ms = 600000) {
  const startTime = Date.now();
  const pollInterval = 2000; // 2 seconds
  
  while (Date.now() - startTime < timeout_ms) {
    const session = getSession(token);
    
    if (!session) {
      // Session file was deleted (possibly by website after verification)
      return { verified: false, session: null };
    }
    
    if (isSessionExpired(session)) {
      return { verified: false, session };
    }
    
    if (session.verified === true) {
      return { verified: true, session };
    }
    
    // Wait before next poll
    await new Promise(resolve => setTimeout(resolve, pollInterval));
  }
  
  // Timeout reached
  return { verified: false, session: getSession(token) };
}

/**
 * Get the site URL from env var or default
 */
function getSiteUrl() {
  return process.env.AETHER_SITE_URL || 'https://jelly-legs-ai.github.io';
}

/**
 * Open URL in default browser (cross-platform)
 */
function openBrowser(url) {
  const platform = os.platform();
  try {
    if (platform === 'win32') {
      execSync(`start "" "${url}"`, { shell: 'cmd' });
    } else if (platform === 'darwin') {
      execSync(`open "${url}"`);
    } else {
      execSync(`xdg-open "${url}"`);
    }
    return true;
  } catch (e) {
    console.log(`${C.yellow}⚠ Could not open browser automatically.${C.reset}`);
    console.log(`${C.dim}  Please open this URL manually:${C.reset}`);
    console.log(`  ${C.cyan}${url}${C.reset}\n`);
    return false;
  }
}

// ---------------------------------------------------------------------------
// CREATE WALLET
// ---------------------------------------------------------------------------

async function createWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Creation ─────────────────────────────────────${C.reset}`);
  console.log(`  ${C.green}1)${C.reset}  Create new wallet — generates a fresh 12-word mnemonic`);
  console.log(`  ${C.green}2)${C.reset}  Import existing — enter your own mnemonic to restore\n`);

  const choice = await question(rl, `  Choose [1/2]: ${C.reset}`);

  let mnemonic;
  if (choice.trim() === '1') {
    // Generate new 12-word BIP39 mnemonic
    mnemonic = bip39.generateMnemonic(128); // 128 bits = 12 words
  } else if (choice.trim() === '2') {
    // Ask for existing mnemonic
    mnemonic = await askMnemonic(rl, 'Importing existing wallet');
    if (!bip39.validateMnemonic(mnemonic)) {
      // Also try without lowercasing
      if (!bip39.validateMnemonic(mnemonic)) {
        console.log(`\n  ${C.red}✗ Invalid BIP39 mnemonic.${C.reset} Please check your word list and try again.`);
        return;
      }
    }
  } else {
    console.log(`\n  ${C.red}✗ Invalid choice.${C.reset} Run \`aether wallet create\` again.`);
    return;
  }

  // Derive keypair
  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic, DERIVATION_PATH);
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
    return;
  }

  const address = formatAddress(keyPair.publicKey);

  // -------------------------------------------------------------------
  // IMPORTANT WARNING — show mnemonic word-by-word (create only)
  // -------------------------------------------------------------------
  if (choice.trim() === '1') {
    const words = mnemonic.split(' ');
    console.log(`\n`);
    console.log(`${C.red}${C.bright}╔═══════════════════════════════════════════════════════════════╗${C.reset}`);
    console.log(`${C.red}${C.bright}║           YOUR WALLET PASSPHRASE                               ║${C.reset}`);
    console.log(`${C.red}${C.bright}╚═══════════════════════════════════════════════════════════════╝${C.reset}`);
    console.log(`\n${C.yellow}  Write these words down. They cannot be recovered.${C.reset}`);
    console.log(`${C.yellow}  No copy is stored. If you lose them, your wallet is UNRECOVERABLE.${C.reset}\n`);
    console.log(`  ${C.bright}1.${C.reset} ${words[0].padEnd(15)}   ${C.bright}5.${C.reset} ${words[4].padEnd(15)}   ${C.bright}9.${C.reset} ${words[8]}`);
    console.log(`  ${C.bright}2.${C.reset} ${words[1].padEnd(15)}   ${C.bright}6.${C.reset} ${words[5].padEnd(15)}   ${C.bright}10.${C.reset} ${words[9]}`);
    console.log(`  ${C.bright}3.${C.reset} ${words[2].padEnd(15)}   ${C.bright}7.${C.reset} ${words[6].padEnd(15)}   ${C.bright}11.${C.reset} ${words[10]}`);
    console.log(`  ${C.bright}4.${C.reset} ${words[3].padEnd(15)}   ${C.bright}8.${C.reset} ${words[7].padEnd(15)}   ${C.bright}12.${C.reset} ${words[11]}`);
    console.log(`\n`);
    await question(rl, `  ${C.cyan}Press Enter when you have saved your passphrase.${C.reset}\n`);
  }

  // Save wallet
  const walletData = saveWalletFile(address, keyPair.publicKey);

  // Set as default
  const cfg = loadConfig();
  cfg.defaultWallet = address;
  saveConfig(cfg);

  console.log(`${C.green}✓ Wallet created:${C.reset} ${C.bright}${address}${C.reset}`);
  console.log(`${C.dim}  Saved to:${C.reset} ${walletFilePath(address)}`);
  console.log(`${C.green}✓ Set as default wallet.${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// LIST WALLETS
// ---------------------------------------------------------------------------

async function listWallets(rl) {
  ensureDirs();
  const cfg = loadConfig();
  const defaultWallet = cfg.defaultWallet;

  let files;
  try {
    files = fs.readdirSync(getWalletsDir()).filter((f) => f.endsWith('.json'));
  } catch {
    files = [];
  }

  if (files.length === 0) {
    console.log(`\n  ${C.dim}No wallets found. Create one with:${C.reset}`);
    console.log(`    ${C.cyan}aether wallet create${C.reset}\n`);
    return;
  }

  console.log(`\n${C.bright}${C.cyan}── Aether Wallets ─────────────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.dim}Location: ${getWalletsDir()}${C.reset}\n`);

  const wallets = files.map((f) => {
    try {
      return JSON.parse(fs.readFileSync(path.join(getWalletsDir(), f), 'utf8'));
    } catch {
      return null;
    }
  }).filter(Boolean);

  wallets.sort((a, b) => (a.created_at || '').localeCompare(b.created_at || ''));

  for (const w of wallets) {
    const isDefault = w.address === defaultWallet;
    const marker = isDefault ? ` ${C.green}★ default${C.reset}` : '';
    const date = w.created_at ? new Date(w.created_at).toLocaleDateString() : 'unknown';
    console.log(`  ${C.bright}${w.address}${C.reset}${marker}`);
    console.log(`  ${C.dim}  Created: ${date}  |  ${w.derivation_path}${C.reset}`);
    console.log();
  }

  if (defaultWallet) {
    console.log(`  ${C.green}★${C.reset} = default wallet (used for signing transactions)\n`);
  }
}

// ---------------------------------------------------------------------------
// IMPORT WALLET
// ---------------------------------------------------------------------------

async function importWallet(rl) {
  const mnemonic = await askMnemonic(rl, 'Importing wallet from mnemonic');

  if (!bip39.validateMnemonic(mnemonic)) {
    // Try splitting by spaces and checking word count
    const words = mnemonic.split(/\s+/);
    if (words.length !== 12 && words.length !== 24) {
      console.log(`\n  ${C.red}✗ Invalid word count:${C.reset} got ${words.length}, expected 12 or 24.`);
      return;
    }
    console.log(`\n  ${C.red}✗ Invalid BIP39 mnemonic.${C.reset} Please check your word list and try again.`);
    return;
  }

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic, DERIVATION_PATH);
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
    return;
  }

  const address = formatAddress(keyPair.publicKey);

  // Check if wallet already exists
  if (loadWallet(address)) {
    console.log(`\n  ${C.yellow}⚠ Wallet already exists:${C.reset} ${address}`);
    console.log(`  ${C.dim}No new file created.${C.reset}\n`);
    return;
  }

  const walletData = saveWalletFile(address, keyPair.publicKey);

  // Set as default
  const cfg = loadConfig();
  cfg.defaultWallet = address;
  saveConfig(cfg);

  console.log(`\n${C.green}✓ Wallet imported:${C.reset} ${C.bright}${address}${C.reset}`);
  console.log(`${C.dim}  Saved to:${C.reset} ${walletFilePath(address)}`);
  console.log(`${C.green}✓ Set as default wallet.${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// DEFAULT WALLET
// ---------------------------------------------------------------------------

async function defaultWallet(rl) {
  const cfg = loadConfig();
  const defaultAddr = cfg.defaultWallet;

  // Check for --set flag
  const args = process.argv.slice(4);
  if (args.includes('--set') || args.includes('-s')) {
    const setIdx = args.indexOf('--set') !== -1 ? args.indexOf('--set') : args.indexOf('-s');
    const address = args[setIdx + 1];
    if (!address) {
      console.log(`\n  ${C.red}Usage:${C.reset} aether wallet default --set <address>\n`);
      return;
    }
    const w = loadWallet(address);
    if (!w) {
      console.log(`\n  ${C.red}✗ Wallet not found:${C.reset} ${address}`);
      return;
    }
    cfg.defaultWallet = address;
    saveConfig(cfg);
    console.log(`\n${C.green}✓ Default wallet set to:${C.reset} ${address}\n`);
    return;
  }

  // Show current default
  console.log(`\n${C.bright}${C.cyan}── Default Wallet ─────────────────────────────────────────${C.reset}\n`);
  if (!defaultAddr) {
    console.log(`  ${C.dim}No default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether wallet default --set <address>${C.reset}\n`);
    return;
  }

  const w = loadWallet(defaultAddr);
  if (w) {
    console.log(`  ${C.green}★${C.reset} ${C.bright}${defaultAddr}${C.reset}`);
    console.log(`  ${C.dim}  Created: ${new Date(w.created_at).toLocaleString()}${C.reset}`);
    console.log(`  ${C.dim}  Derivation: ${w.derivation_path}${C.reset}\n`);
  } else {
    console.log(`  ${C.yellow}⚠ Default wallet file missing, but config references:${C.reset}`);
    console.log(`    ${defaultAddr}\n`);
    console.log(`  ${C.dim}Run:${C.reset} aether wallet default --set <address> ${C.dim}to update.${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// CONNECT WALLET (session token + browser verification)
// ---------------------------------------------------------------------------

async function connectWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Connection ─────────────────────────────────────${C.reset}\n`);

  // Get wallet address (default or specified via --address)
  const args = process.argv.slice(4);
  let targetAddress = null;
  
  const addrIndex = args.indexOf('--address');
  if (addrIndex !== -1 && args[addrIndex + 1]) {
    targetAddress = args[addrIndex + 1];
  }
  
  // If no address specified, use default wallet
  if (!targetAddress) {
    const cfg = loadConfig();
    targetAddress = cfg.defaultWallet;
  }
  
  if (!targetAddress) {
    console.log(`  ${C.red}✗ No wallet specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Create a wallet first:${C.reset} ${C.cyan}aether wallet create${C.reset}`);
    console.log(`  ${C.dim}Or specify:${C.reset} ${C.cyan}aether wallet connect --address ATH...${C.reset}\n`);
    return;
  }
  
  // Verify wallet exists
  const wallet = loadWallet(targetAddress);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${targetAddress}`);
    console.log(`  ${C.dim}Run:${C.reset} ${C.cyan}aether wallet list${C.reset} ${C.dim}to see available wallets.${C.reset}\n`);
    return;
  }
  
  console.log(`  ${C.green}✓ Using wallet:${C.reset} ${C.bright}${targetAddress}${C.reset}\n`);
  
  // Generate session token
  const token = generateSessionToken();
  console.log(`  ${C.dim}Session token:${C.reset} ${token}`);
  
  // Save session
  saveSession(token, targetAddress, 10);
  console.log(`  ${C.green}✓ Session created (expires in 10 minutes)${C.reset}\n`);
  
  // Build verification URL
  const siteUrl = getSiteUrl();
  const verifyUrl = `${siteUrl}/wallet/verify?token=${token}&address=${encodeURIComponent(targetAddress)}`;
  
  console.log(`  ${C.dim}Opening verification page:${C.reset}`);
  console.log(`  ${C.cyan}${verifyUrl}${C.reset}\n`);
  
  // Open browser
  openBrowser(verifyUrl);
  
  // Start polling for verification
  console.log(`  ${C.dim}Waiting for verification...${C.reset}`);
  console.log(`  ${C.dim}(Press Ctrl+C to cancel)${C.reset}\n`);
  
  // Show progress dots while polling
  const pollPromise = pollForVerification(token, 600000);
  
  // Animated dots
  const dotInterval = setInterval(() => {
    process.stdout.write('.');
  }, 1000);
  
  const result = await pollPromise;
  clearInterval(dotInterval);
  console.log('\n');
  
  if (result.verified) {
    console.log(`  ${C.green}${C.bright}✓ Wallet verified successfully!${C.reset}\n`);
    console.log(`  ${C.green}★${C.reset} ${C.bright}${targetAddress}${C.reset}`);
    console.log(`  ${C.dim}Session complete. You can now use this wallet for transactions.${C.reset}\n`);
    
    // Clean up session file
    deleteSession(token);
    process.exit(0);
  } else {
    console.log(`  ${C.red}✗ Session expired or verification failed.${C.reset}\n`);
    console.log(`  ${C.dim}The verification page must be completed within 10 minutes.${C.reset}`);
    console.log(`  ${C.dim}Run again to create a new session:${C.reset}\n`);
    console.log(`    ${C.cyan}aether wallet connect${C.reset}\n`);
    
    // Clean up expired session
    deleteSession(token);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Main dispatcher
// ---------------------------------------------------------------------------

async function walletCommand() {
  // Handle both direct execution (node commands/wallet.js list) and CLI (aether wallet list)
  // Direct: argv = [node, wallet.js, list] → subcmd at argv[2]
  // CLI: argv = [node, index.js, wallet, list] → subcmd at argv[3]
  let subcmd = process.argv[2];
  if (subcmd === 'wallet.js' || subcmd === 'wallet') {
    subcmd = process.argv[3];
  }

  const rl = createRl();

  try {
    if (!subcmd || subcmd === 'create') {
      await createWallet(rl);
    } else if (subcmd === 'list') {
      await listWallets(rl);
    } else if (subcmd === 'import') {
      await importWallet(rl);
    } else if (subcmd === 'default') {
      await defaultWallet(rl);
    } else if (subcmd === 'connect') {
      await connectWallet(rl);
    } else {
      console.log(`\n  ${C.red}Unknown wallet subcommand:${C.reset} ${subcmd}`);
      console.log(`\n  Usage:`);
      console.log(`    ${C.cyan}aether wallet create${C.reset}   Create new or import wallet`);
      console.log(`    ${C.cyan}aether wallet list${C.reset}     List all wallets`);
      console.log(`    ${C.cyan}aether wallet import${C.reset}   Import wallet from mnemonic`);
      console.log(`    ${C.cyan}aether wallet default${C.reset}  Show/set default wallet`);
      console.log(`    ${C.cyan}aether wallet connect${C.reset}  Connect wallet to website (session token)`);
      console.log();
      process.exit(1);
    }
  } finally {
    rl.close();
  }
}

module.exports = { walletCommand };

if (require.main === module) {
  walletCommand();
}
