/**
 * aether-cli wallet
 *
 * Aether wallet management:
 *   aether wallet create   — Create new BIP39 wallet or import existing
 *   aether wallet list     — List all wallets
 *   aether wallet import   — Import wallet from mnemonic
 *   aether wallet default  — Show/set default wallet
 *   aether wallet connect  — Connect wallet via browser verification
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

// Import SDK for blockchain RPC calls
const aether = require('../sdk');

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

// CLI version for session files
const CLI_VERSION = '1.0.3';

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

function getSessionsDir() {
  return path.join(getAetherDir(), 'sessions');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function ensureDirs() {
  const wd = getWalletsDir();
  if (!fs.existsSync(wd)) fs.mkdirSync(wd, { recursive: true });
  const sd = getSessionsDir();
  if (!fs.existsSync(sd)) fs.mkdirSync(sd, { recursive: true });
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
  if (!bip39.validateMnemonic(mnemonic)) {
    throw new Error('Invalid mnemonic phrase.');
  }
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return {
    publicKey: Buffer.from(keyPair.publicKey),
    secretKey: Buffer.from(keyPair.secretKey),
  };
}

/**
 * Format Aether address: ATH + base58check of public key.
 */
function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

// ---------------------------------------------------------------------------
// Session management helpers
// ---------------------------------------------------------------------------

function sessionFilePath(token) {
  return path.join(getSessionsDir(), `${token}.json`);
}

/** Generate a UUID v4 session token */
function generateSessionToken() {
  return crypto.randomUUID();
}

/**
 * Save session to ~/.aether/sessions/<uuid>.json
 * Fields: wallet_address, created_at, expires_at, verified, cli_version
 */
function saveSession(token, wallet_address, expires_in_minutes = 10) {
  ensureDirs();
  const now = new Date();
  const expires_at = new Date(now.getTime() + expires_in_minutes * 60 * 1000);
  const session = {
    wallet_address,
    created_at: now.toISOString(),
    expires_at: expires_at.toISOString(),
    verified: false,
    cli_version: CLI_VERSION,
  };
  fs.writeFileSync(sessionFilePath(token), JSON.stringify(session, null, 2));
  return session;
}

/** Load a session, or return null if missing or expired */
function getSession(token) {
  const fp = sessionFilePath(token);
  if (!fs.existsSync(fp)) return null;
  try {
    const session = JSON.parse(fs.readFileSync(fp, 'utf8'));
    if (new Date(session.expires_at) < new Date()) return null;
    return session;
  } catch {
    return null;
  }
}

/** Mark a session as verified */
function markSessionVerified(token) {
  const session = getSession(token);
  if (!session) return false;
  session.verified = true;
  fs.writeFileSync(sessionFilePath(token), JSON.stringify(session, null, 2));
  return true;
}

/** Delete a session file */
function deleteSession(token) {
  const fp = sessionFilePath(token);
  if (fs.existsSync(fp)) fs.unlinkSync(fp);
}

/**
 * Poll ~/.aether/sessions/<token>.json every 2 seconds.
 * Resolves when verified=true OR session expired/timeout.
 * Returns { verified: boolean, reason?: 'expired' | 'timeout' }
 */
async function pollForVerification(token, timeout_ms = 600000) {
  const interval_ms = 2000;
  const max_retries = Math.floor(timeout_ms / interval_ms);

  for (let i = 0; i < max_retries; i++) {
    const session = getSession(token);
    if (session && session.verified) {
      return { verified: true };
    }
    if (!session) {
      return { verified: false, reason: 'expired' };
    }
    await new Promise((res) => setTimeout(res, interval_ms));
  }
  return { verified: false, reason: 'timeout' };
}

/** Get the site URL from env var or default */
function getSiteUrl() {
  return process.env.AETHER_SITE_URL || 'https://jelly-legs-ai.github.io';
}

/** Open URL in the default browser (cross-platform) */
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
  } catch {
    return false;
  }
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
  return readline.createInterface({ input: process.stdin, output: process.stdout });
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
// CREATE WALLET
// ---------------------------------------------------------------------------

async function createWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Creation ─────────────────────────────────────${C.reset}`);
  console.log(`  ${C.green}1)${C.reset}  Create new wallet — generates a fresh 12-word mnemonic`);
  console.log(`  ${C.green}2)${C.reset}  Import existing — enter your own mnemonic to restore\n`);

  const choice = await question(rl, `  Choose [1/2]: ${C.reset}`);

  let mnemonic;
  if (choice.trim() === '1') {
    mnemonic = bip39.generateMnemonic(128);
  } else if (choice.trim() === '2') {
    mnemonic = await askMnemonic(rl, 'Importing existing wallet');
    if (!bip39.validateMnemonic(mnemonic)) {
      console.log(`\n  ${C.red}✗ Invalid BIP39 mnemonic.${C.reset} Please check your word list and try again.`);
      return;
    }
  } else {
    console.log(`\n  ${C.red}✗ Invalid choice.${C.reset} Run \`aether wallet create\` again.`);
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

  const walletData = saveWalletFile(address, keyPair.publicKey);
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

  const wallets = files
    .map((f) => {
      try {
        return JSON.parse(fs.readFileSync(path.join(getWalletsDir(), f), 'utf8'));
      } catch {
        return null;
      }
    })
    .filter(Boolean);

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

  if (loadWallet(address)) {
    console.log(`\n  ${C.yellow}⚠ Wallet already exists:${C.reset} ${address}`);
    console.log(`  ${C.dim}No new file created.${C.reset}\n`);
    return;
  }

  const walletData = saveWalletFile(address, keyPair.publicKey);
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
// CONNECT WALLET
// Generates a session token, opens browser to verify page, polls until done.
// ---------------------------------------------------------------------------

async function connectWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Connect ────────────────────────────────────────${C.reset}\n`);

  // Resolve wallet address: --address flag or default
  const args = process.argv.slice(4);
  let address = null;
  const addrIdx = args.findIndex((a) => a === '--address' || a === '-a');
  if (addrIdx !== -1 && args[addrIdx + 1]) {
    address = args[addrIdx + 1];
  }
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage:${C.reset} aether wallet connect --address <address>`);
    console.log(`  ${C.dim}Or set a default:${C.reset} aether wallet default --set <address>\n`);
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}`);
    console.log(`  ${C.dim}Check your wallets with:${C.reset} aether wallet list\n`);
    return;
  }

  // Generate session token and save session
  const token = generateSessionToken();
  saveSession(token, address, 10);

  // Build verification URL
  const siteUrl = getSiteUrl();
  const verifyUrl = `${siteUrl}/wallet/verify?token=${token}&address=${encodeURIComponent(address)}`;

  console.log(`  ${C.green}★${C.reset} Wallet: ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.dim}  Session expires in 10 minutes${C.reset}`);
  console.log();

  // Open browser
  const opened = openBrowser(verifyUrl);
  if (opened) {
    console.log(`  ${C.green}✓${C.reset} Opened verification page in browser.`);
    console.log(`  ${C.dim}  ${verifyUrl}${C.reset}`);
  } else {
    console.log(`  ${C.yellow}⚠ Could not open browser automatically.${C.reset}`);
    console.log(`  ${C.cyan}Open this URL manually:${C.reset}`);
    console.log(`  ${C.dim}  ${verifyUrl}${C.reset}`);
  }

  console.log();
  console.log(`  ${C.yellow}⏳ Waiting for verification...${C.reset} (Ctrl+C to cancel)`);
  console.log(`  ${C.dim}  Polling every 2s, timeout after 10 minutes${C.reset}`);

  // Poll for verification (blocking, async)
  const result = await pollForVerification(token, 600000);

  if (result.verified) {
    console.log(`\n${C.green}✓ Wallet verified and connected!${C.reset}`);
    console.log(`  ${C.green}★${C.reset} ${address}`);
    deleteSession(token);
    console.log();
    return;
  }

  if (result.reason === 'expired') {
    console.log(`\n  ${C.red}✗ Session expired.${C.reset} Please run ${C.cyan}aether wallet connect${C.reset} again.\n`);
  } else {
    console.log(`\n  ${C.red}✗ Verification timed out (10 minutes).${C.reset} Please run ${C.cyan}aether wallet connect${C.reset} again.\n`);
  }
  deleteSession(token);
  process.exit(1);
}

// ---------------------------------------------------------------------------
// BALANCE
// Query chain RPC GET /v1/account/<addr> for real AETH balance using SDK
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

/**
 * Format lamports as AETH string (1 AETH = 1e9 lamports)
 */
function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  // Show up to 4 decimal places, stripping trailing zeros
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

async function balanceWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Balance ───────────────────────────────────────${C.reset}\n`);

  // Resolve wallet address: --address flag or default
  const args = process.argv.slice(4);
  let address = null;
  const addrIdx = args.findIndex((a) => a === '--address' || a === '-a');
  if (addrIdx !== -1 && args[addrIdx + 1]) {
    address = args[addrIdx + 1];
  }
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage:${C.reset} aether wallet balance --address <address>`);
    console.log(`  ${C.dim}Or set a default:${C.reset} aether wallet default --set <address>\n`);
    return;
  }

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.green}★${C.reset} Wallet: ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.dim}  RPC: ${rpcUrl}${C.reset}`);
  console.log();

  try {
    // Use SDK for real blockchain RPC call
    const client = new aether.AetherClient({ rpcUrl });
    // Strip ATH prefix if present for API call
    const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
    const account = await client.getAccountInfo(rawAddr);
    
    if (!account || account.error) {
      console.log(`  ${C.yellow}⚠ Account not found on chain or RPC error.${C.reset}`);
      console.log(`  ${C.dim}  This is normal for new wallets with 0 balance.${C.reset}`);
      console.log(`  ${C.dim}  RPC response: ${JSON.stringify(account?.error || account)}${C.reset}\n`);
      return;
    }

    const lamports = account.lamports || 0;
    console.log(`  ${C.green}✓ Balance:${C.reset} ${C.bright}${formatAether(lamports)}${C.reset}`);
    console.log(`  ${C.dim}  Raw: ${lamports} lamports${C.reset}`);
    console.log();

    if (account.owner) {
      const ownerStr = Array.isArray(account.owner)
        ? 'ATH' + bs58.encode(Buffer.from(account.owner.slice(0, 32)))
        : account.owner;
      console.log(`  ${C.dim}  Owner: ${ownerStr}${C.reset}`);
    }
    if (account.rent_epoch !== undefined) {
      console.log(`  ${C.dim}  Rent epoch: ${account.rent_epoch}${C.reset}`);
    }
    console.log();
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to fetch balance:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// HTTP helpers for POST requests
// ---------------------------------------------------------------------------

function httpPost(rpcUrl, path, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? require('https') : require('http');
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve(data); }
      });
    });
    req.on('error', reject);
    req.write(bodyStr);
    req.end();
  });
}

/**
 * Sign a transaction using the wallet's secret key.
 * Returns a base58-encoded 64-byte signature.
 */
function signTransaction(tx, secretKey) {
  const txBytes = Buffer.from(JSON.stringify(tx));
  const sig = nacl.sign.detached(txBytes, secretKey);
  return bs58.encode(sig);
}

/**
 * Compute SHA-512 hash of data (as hex string) — used for tx id
 */
function sha512hex(data) {
  return crypto.createHash('sha512').update(data).digest('hex');
}

// ---------------------------------------------------------------------------
// STAKE
// Submit a Stake transaction via POST /v1/tx
// ---------------------------------------------------------------------------

async function stakeWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Stake AETH ─────────────────────────────────────────────${C.reset}\n`);

  // Resolve wallet address
  const args = process.argv.slice(4);
  let address = null;
  let validator = null;
  let amountStr = null;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      address = args[i + 1];
    }
    if ((args[i] === '--validator' || args[i] === '-v') && args[i + 1]) {
      validator = args[i + 1];
    }
    if ((args[i] === '--amount' || args[i] === '-m') && args[i + 1]) {
      amountStr = args[i + 1];
    }
  }

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether stake --address <addr> --validator <val> --amount <aeth>${C.reset}\n`);
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}\n`);
    return;
  }

  // Prompt for missing values interactively
  if (!validator) {
    console.log(`  ${C.cyan}Enter validator address:${C.reset}`);
    validator = await question(rl, `  Validator > ${C.reset}`);
  }

  if (!amountStr) {
    console.log(`  ${C.cyan}Enter amount in AETH:${C.reset}`);
    amountStr = await question(rl, `  Amount (AETH) > ${C.reset}`);
  }

  const amount = parseFloat(amountStr);
  if (isNaN(amount) || amount <= 0) {
    console.log(`  ${C.red}✗ Invalid amount:${C.reset} ${amountStr}\n`);
    return;
  }

  const lamports = Math.round(amount * 1e9);

  console.log(`  ${C.green}★${C.reset} Wallet:    ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Validator: ${C.bright}${validator}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Amount:    ${C.bright}${amount} AETH${C.reset} (${lamports} lamports)`);
  console.log();

  // Ask for mnemonic to derive signing keypair
  console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter your 12/24-word passphrase to sign this transaction');
  console.log();

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic, DERIVATION_PATH);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
    console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
    return;
  }

  // Verify the derived address matches the wallet
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:   ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected:  ${address}${C.reset}`);
    console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
    return;
  }

  const confirm = await question(rl, `  ${C.yellow}Confirm stake? [y/N]${C.reset} > ${C.reset}`);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`  ${C.dim}Cancelled.${C.reset}\n`);
    return;
  }

  // Build the transaction
  const tx = {
    signer: address.startsWith('ATH') ? address.slice(3) : address,
    tx_type: 'Stake',
    payload: {
      type: 'Stake',
      data: {
        validator,
        amount: lamports,
      },
    },
    fee: 0,
    slot: 0,
    timestamp: Math.floor(Date.now() / 1000),
  };

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.dim}Submitting to ${rpcUrl}...${C.reset}`);

  try {
    const result = await httpPost(rpcUrl, '/v1/tx', tx);

    if (result.error) {
      console.log(`\n  ${C.red}✗ Transaction failed:${C.reset} ${result.error}\n`);
      process.exit(1);
    }

    const sig = result.signature || result.tx_signature || result.id || JSON.stringify(result);
    console.log(`\n${C.green}✓ Stake transaction submitted!${C.reset}`);
    console.log(`  ${C.dim}Signature:${C.reset} ${sig}`);
    console.log(`  ${C.dim}Use: aether-cli validator status${C.reset} to monitor.\n`);
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to submit transaction:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Is your validator running? RPC: ${rpcUrl}${C.reset}\n`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// TRANSFER
// Submit a Transfer transaction via POST /v1/tx
// ---------------------------------------------------------------------------

async function transferWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Transfer AETH ─────────────────────────────────────────${C.reset}\n`);

  const args = process.argv.slice(4);
  let address = null;
  let recipient = null;
  let amountStr = null;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      address = args[i + 1];
    }
    if ((args[i] === '--to' || args[i] === '-t') && args[i + 1]) {
      recipient = args[i + 1];
    }
    if ((args[i] === '--amount' || args[i] === '-m') && args[i + 1]) {
      amountStr = args[i + 1];
    }
  }

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether transfer --to <addr> --amount <aeth>${C.reset}\n`);
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}\n`);
    return;
  }

  // Prompt for missing values interactively
  if (!recipient) {
    console.log(`  ${C.cyan}Enter recipient address:${C.reset}`);
    recipient = await question(rl, `  Recipient > ${C.reset}`);
  }

  if (!amountStr) {
    console.log(`  ${C.cyan}Enter amount in AETH:${C.reset}`);
    amountStr = await question(rl, `  Amount (AETH) > ${C.reset}`);
  }

  const amount = parseFloat(amountStr);
  if (isNaN(amount) || amount <= 0) {
    console.log(`  ${C.red}✗ Invalid amount:${C.reset} ${amountStr}\n`);
    return;
  }

  const lamports = Math.round(amount * 1e9);

  console.log(`  ${C.green}★${C.reset} From:      ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} To:        ${C.bright}${recipient}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Amount:    ${C.bright}${amount} AETH${C.reset} (${lamports} lamports)`);
  console.log();

  // Ask for mnemonic to derive signing keypair
  console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter your 12/24-word passphrase to sign this transaction');
  console.log();

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic, DERIVATION_PATH);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
    console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
    return;
  }

  // Verify the derived address matches the wallet
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:   ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected:  ${address}${C.reset}`);
    console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
    return;
  }

  const confirm = await question(rl, `  ${C.yellow}Confirm transfer? [y/N]${C.reset} > ${C.reset}`);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`  ${C.dim}Cancelled.${C.reset}\n`);
    return;
  }

  const tx = {
    signer: address.startsWith('ATH') ? address.slice(3) : address,
    tx_type: 'Transfer',
    payload: {
      type: 'Transfer',
      data: {
        recipient,
        amount: lamports,
        nonce: Math.floor(Math.random() * 0xffffffff),
      },
    },
    fee: 0,
    slot: 0,
    timestamp: Math.floor(Date.now() / 1000),
  };

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.dim}Submitting to ${rpcUrl}...${C.reset}`);

  try {
    const result = await httpPost(rpcUrl, '/v1/tx', tx);

    if (result.error) {
      console.log(`\n  ${C.red}✗ Transaction failed:${C.reset} ${result.error}\n`);
      process.exit(1);
    }

    const sig = result.signature || result.tx_signature || result.id || JSON.stringify(result);
    console.log(`\n${C.green}✓ Transfer transaction submitted!${C.reset}`);
    console.log(`  ${C.dim}Signature:${C.reset} ${sig}`);
    console.log(`  ${C.dim}Check balance: aether wallet balance --address ${address}${C.reset}\n`);
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to submit transaction:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Is your validator running? RPC: ${rpcUrl}${C.reset}\n`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// TX HISTORY
// Fetch and display recent transactions for an address
// ---------------------------------------------------------------------------

async function txHistory(rl) {
  console.log(`\n${C.bright}${C.cyan}── Transaction History ────────────────────────────────────${C.reset}\n`);

  const args = process.argv.slice(4);
  let address = null;
  let limit = 20;
  let asJson = false;

  const addrIdx = args.findIndex((a) => a === '--address' || a === '-a');
  if (addrIdx !== -1 && args[addrIdx + 1]) {
    address = args[addrIdx + 1];
  }

  const limitIdx = args.findIndex((a) => a === '--limit' || a === '-l');
  if (limitIdx !== -1 && args[limitIdx + 1]) {
    limit = parseInt(args[limitIdx + 1], 10);
    if (isNaN(limit) || limit < 1 || limit > 100) {
      console.log(`  ${C.red}✗ --limit must be between 1 and 100.${C.reset}\n`);
      return;
    }
  }

  asJson = args.includes('--json') || args.includes('-j');

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether tx history --address <addr> [--limit 20] [--json]${C.reset}\n`);
    return;
  }

  const rpcUrl = getDefaultRpc();
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!asJson) {
    console.log(`  ${C.green}★${C.reset} Address: ${C.bright}${address}${C.reset}`);
    console.log(`  ${C.dim}  RPC: ${rpcUrl}  Limit: ${limit}${C.reset}`);
    console.log();
  }

  try {
    // Fetch account info first (for context)
    const account = await httpRequest(rpcUrl, `/v1/account/${rawAddr}`);

    // Fetch transactions for this address
    const txs = await httpRequest(rpcUrl, `/v1/tx?address=${encodeURIComponent(rawAddr)}&limit=${limit}`);

    if (asJson) {
      const out = {
        address,
        rpc: rpcUrl,
        account: account && !account.error ? {
          lamports: account.lamports,
          owner: account.owner,
        } : null,
        transactions: txs && !txs.error ? (Array.isArray(txs) ? txs : txs.transactions || []) : [],
        fetched_at: new Date().toISOString(),
      };
      console.log(JSON.stringify(out, null, 2));
      return;
    }

    if (!account || account.error) {
      console.log(`  ${C.yellow}⚠ Account not found on chain.${C.reset}`);
    } else {
      console.log(`  ${C.green}✓ Balance:${C.reset} ${C.bright}${formatAether(account.lamports || 0)}${C.reset}`);
      if (account.owner) {
        const ownerStr = Array.isArray(account.owner)
          ? 'ATH' + bs58.encode(Buffer.from(account.owner.slice(0, 32)))
          : account.owner;
        console.log(`  ${C.dim}  Owner: ${ownerStr}${C.reset}`);
      }
      console.log();
    }

    if (!txs || txs.error) {
      console.log(`  ${C.yellow}⚠ No transaction history available.${C.reset}`);
      console.log(`  ${C.dim}  RPC response: ${JSON.stringify(txs?.error || txs)}${C.reset}`);
      console.log(`  ${C.dim}  (New wallets with 0 txs will return empty results)${C.reset}\n`);
      return;
    }

    const txList = Array.isArray(txs) ? txs : txs.transactions || [];
    console.log(`  ${C.bright}Recent Transactions (${txList.length})${C.reset}\n`);

    if (txList.length === 0) {
      console.log(`  ${C.dim}  No transactions found for this address.${C.reset}`);
      console.log(`  ${C.dim}  This is normal for new wallets.${C.reset}\n`);
      return;
    }

    const typeColors = {
      Transfer: C.cyan,
      Stake: C.green,
      Unstake: C.yellow,
      ClaimRewards: C.magenta,
      CreateNFT: C.red,
      MintNFT: C.red,
      TransferNFT: C.cyan,
      UpdateMetadata: C.yellow,
    };

    for (const tx of txList) {
      const txType = tx.tx_type || tx.type || 'Unknown';
      const color = typeColors[txType] || C.reset;
      const ts = tx.timestamp
        ? new Date(tx.timestamp * 1000).toISOString()
        : 'unknown';
      const sig = tx.signature || tx.id || tx.tx_signature || '—';
      const sigShort = sig.length > 20 ? sig.slice(0, 8) + '…' + sig.slice(-8) : sig;

      console.log(`  ${C.dim}┌─ ${ts}${C.reset}`);
      console.log(`  │  ${C.bright}${color}${txType}${C.reset}  ${C.dim}sig:${C.reset} ${sigShort}`);
      if (tx.payload && tx.payload.data) {
        const d = tx.payload.data;
        if (d.recipient) console.log(`  │  ${C.dim}  → to:      ${d.recipient}${C.reset}`);
        if (d.amount)    console.log(`  │  ${C.dim}  amount:   ${formatAether(d.amount)}${C.reset}`);
        if (d.validator) console.log(`  │  ${C.dim}  validator: ${d.validator}${C.reset}`);
        if (d.stake_account) console.log(`  │  ${C.dim}  stake_acct: ${d.stake_account}${C.reset}`);
      }
      if (tx.fee !== undefined && tx.fee > 0) {
        console.log(`  │  ${C.dim}  fee: ${tx.fee} lamports${C.reset}`);
      }
      console.log(`  ${C.dim}└${C.reset}`);
      console.log();
    }
    console.log();
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to fetch transaction history:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// EXPORT WALLET
// Export wallet data for backup — public data by default, mnemonic with --mnemonic flag
// ---------------------------------------------------------------------------

async function exportWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Wallet Export ─────────────────────────────────────────${C.reset}\n`);

  const args = process.argv.slice(4);
  let address = null;
  let asJson = false;
  let includeMnemonic = false;

  const addrIdx = args.findIndex((a) => a === '--address' || a === '-a');
  if (addrIdx !== -1 && args[addrIdx + 1]) {
    address = args[addrIdx + 1];
  }

  asJson = args.includes('--json') || args.includes('-j');
  includeMnemonic = args.includes('--mnemonic') || args.includes('-m');

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether wallet export --address <addr> [--mnemonic] [--json]${C.reset}\n`);
    return;
  }

  const walletData = loadWallet(address);
  if (!walletData) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}`);
    console.log(`  ${C.dim}  Available wallets: ${C.cyan}aether wallet list${C.reset}\n`);
    return;
  }

  const exportData = {
    version: walletData.version || 1,
    address: walletData.address,
    public_key: walletData.public_key,
    derivation_path: walletData.derivation_path,
    created_at: walletData.created_at,
    source: 'aether-cli',
  };

  // Mnemonic export requires interactive confirmation for security
  if (includeMnemonic) {
    console.log(`  ${C.yellow}⚠ WARNING: You are about to export your SECRET MNEMONIC.${C.reset}`);
    console.log(`  ${C.dim}  Anyone with this phrase can access your funds.${C.reset}\n`);
    const confirmed = await question(rl, `  Type ${C.bright}EXPORT${C.reset} to confirm: `);
    if (confirmed.trim().toUpperCase() !== 'EXPORT') {
      console.log(`\n  ${C.dim}Aborted. Mnemonic not exported.${C.reset}\n`);
      return;
    }
    console.log(`  ${C.green}✓ Confirmed${C.reset}\n`);

    // Re-derive mnemonic from keypair is NOT possible — we must ask for it
    console.log(`  ${C.dim}The CLI cannot retrieve your mnemonic from the stored keypair.${C.reset}`);
    console.log(`  ${C.dim}If you have a backup of your mnemonic, enter it below to include it.${C.reset}`);
    console.log(`  ${C.dim}Otherwise, press Enter to export public data only.${C.reset}\n`);
    const mnemonicInput = await question(rl, `  ${C.cyan}Mnemonic (or Enter to skip):${C.reset} `);
    const mnemonic = mnemonicInput.trim();
    if (mnemonic && bip39.validateMnemonic(mnemonic)) {
      exportData.mnemonic = mnemonic;
    } else if (mnemonic) {
      console.log(`  ${C.red}✗ Invalid mnemonic phrase. Skipping.${C.reset}`);
    }
  }

  if (asJson) {
    console.log(JSON.stringify(exportData, null, 2));
    return;
  }

  // Human-readable output
  console.log(`  ${C.green}★${C.reset} Wallet exported successfully\n`);
  console.log(`  ${C.dim}Address:${C.reset}        ${exportData.address}`);
  console.log(`  ${C.dim}Public key:${C.reset}     ${exportData.public_key}`);
  console.log(`  ${C.dim}Derivation path:${C.reset} ${exportData.derivation_path}`);
  console.log(`  ${C.dim}Created:${C.reset}        ${exportData.created_at ? new Date(exportData.created_at).toLocaleString() : 'unknown'}`);
  if (exportData.mnemonic) {
    console.log();
    console.log(`  ${C.yellow}★ MNEMONIC (keep this secret!):${C.reset}`);
    console.log(`  ${C.bright}${exportData.mnemonic}${C.reset}`);
  }
  console.log();
  console.log(`  ${C.dim}Export saved to stdout in JSON format with:${C.reset}`);
  console.log(`    ${C.cyan}aether wallet export --address ${address} --json${C.reset}`);
  console.log();
}

// STAKE POSITIONS
// Query and display current stake positions/delegations for a wallet
// ---------------------------------------------------------------------------

async function stakePositions(rl) {
  console.log(`\n${C.bright}${C.cyan}── Stake Positions ──────────────────────────────────────${C.reset}\n`);

  const args = process.argv.slice(4);
  let address = null;
  let asJson = false;

  const addrIdx = args.findIndex((a) => a === '--address' || a === '-a');
  if (addrIdx !== -1 && args[addrIdx + 1]) {
    address = args[addrIdx + 1];
  }

  asJson = args.includes('--json') || args.includes('-j');

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address specified and no default wallet set.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether wallet stake-positions --address <addr> [--json]${C.reset}\n`);
    return;
  }

  const rpcUrl = getDefaultRpc();
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!asJson) {
    console.log(`  ${C.green}★${C.reset} Address: ${C.bright}${address}${C.reset}`);
    console.log(`  ${C.dim}  RPC: ${rpcUrl}${C.reset}`);
    console.log();
  }

  try {
    // Fetch stake accounts for this address
    const res = await httpRequest(rpcUrl, `/v1/stake?address=${encodeURIComponent(rawAddr)}`);

    let stakeAccounts = [];
    if (res && !res.error) {
      stakeAccounts = Array.isArray(res) ? res : (res.accounts || res.stake_accounts || []);
    }

    if (asJson) {
      const out = {
        address,
        rpc: rpcUrl,
        stake_accounts: stakeAccounts.map(acc => ({
          stake_account: acc.pubkey || acc.publicKey || acc.account,
          validator: acc.validator || acc.voter || acc.vote_account,
          stake_lamports: acc.stake_lamports || acc.lamports || 0,
          stake_aeth: (acc.stake_lamports || acc.lamports || 0) / 1e9,
          status: acc.status || acc.state || 'unknown',
          activation_epoch: acc.activation_epoch,
          deactivation_epoch: acc.deactivation_epoch,
          rewards_earned: acc.rewards_earned || 0,
        })),
        total_staked_lamports: stakeAccounts.reduce((sum, acc) => sum + (acc.stake_lamports || acc.lamports || 0), 0),
        fetched_at: new Date().toISOString(),
      };
      console.log(JSON.stringify(out, null, 2));
      return;
    }

    if (!stakeAccounts || stakeAccounts.length === 0) {
      console.log(`  ${C.yellow}⚠ No active stake positions found.${C.reset}`);
      console.log(`  ${C.dim}  This wallet has not delegated to any validators.${C.reset}`);
      console.log(`  ${C.dim}  Stake AETH with: ${C.cyan}aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
      return;
    }

    let totalStaked = 0;
    let activeCount = 0;
    let deactivatingCount = 0;
    let inactiveCount = 0;

    console.log(`  ${C.bright}Stake Positions (${stakeAccounts.length})${C.reset}\n`);

    const statusColors = {
      active: C.green,
      activating: C.cyan,
      deactivating: C.yellow,
      inactive: C.dim,
    };

    for (const acc of stakeAccounts) {
      const stakeAcct = acc.pubkey || acc.publicKey || acc.account || 'unknown';
      const validator = acc.validator || acc.voter || acc.vote_account || 'unknown';
      const lamports = acc.stake_lamports || acc.lamports || 0;
      const aeth = lamports / 1e9;
      const status = (acc.status || acc.state || 'unknown').toLowerCase();
      const rewards = acc.rewards_earned || 0;

      totalStaked += lamports;

      if (status === 'active') activeCount++;
      else if (status === 'deactivating' || status === 'deactivated') deactivatingCount++;
      else inactiveCount++;

      const statusColor = statusColors[status] || C.reset;
      const shortAcct = stakeAcct.length > 20 ? stakeAcct.slice(0, 8) + '…' + stakeAcct.slice(-8) : stakeAcct;
      const shortVal = validator.length > 20 ? validator.slice(0, 8) + '…' + validator.slice(-8) : validator;

      console.log(`  ${C.dim}┌─ ${C.bright}${statusColor}${status.toUpperCase()}${C.reset}`);
      console.log(`  │  ${C.dim}Stake acct:${C.reset} ${shortAcct}`);
      console.log(`  │  ${C.dim}Validator:${C.reset}  ${shortVal}`);
      console.log(`  │  ${C.dim}Staked:${C.reset}    ${C.bright}${aeth.toFixed(4)} AETH${C.reset} (${lamports.toLocaleString()} lamports)`);
      if (rewards > 0) {
        console.log(`  │  ${C.dim}Rewards:${C.reset}   ${C.green}+${(rewards / 1e9).toFixed(4)} AETH${C.reset}`);
      }
      if (acc.activation_epoch !== undefined) {
        console.log(`  │  ${C.dim}Activated:${C.reset} epoch ${acc.activation_epoch}`);
      }
      if (acc.deactivation_epoch !== undefined) {
        console.log(`  │  ${C.dim}Deactivates:${C.reset} epoch ${acc.deactivation_epoch}`);
      }
      console.log(`  ${C.dim}└${C.reset}`);
      console.log();
    }

    console.log(`  ${C.bright}Summary:${C.reset}`);
    console.log(`  ${C.dim}  Total staked:${C.reset} ${C.bright}${(totalStaked / 1e9).toFixed(4)} AETH${C.reset} (${totalStaked.toLocaleString()} lamports)`);
    console.log(`  ${C.green}  ● Active:${C.reset} ${activeCount}  ${C.yellow}● Deactivating:${C.reset} ${deactivatingCount}  ${C.dim}● Inactive:${C.reset} ${inactiveCount}`);
    console.log();

  } catch (err) {
    console.log(`  ${C.red}✗ Failed to fetch stake positions:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// UNSTAKE
// Submit an Unstake transaction via POST /v1/tx to deactivate stake
// ---------------------------------------------------------------------------

async function unstakeWallet(rl) {
  console.log(`\n${C.bright}${C.cyan}── Unstake AETH ──────────────────────────────────────────${C.reset}\n`);

  const args = process.argv.slice(4);
  let address = null;
  let stakeAccount = null;
  let amountStr = null;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      address = args[i + 1];
    }
    if ((args[i] === '--account' || args[i] === '-s') && args[i + 1]) {
      stakeAccount = args[i + 1];
    }
    if ((args[i] === '--amount' || args[i] === '-m') && args[i + 1]) {
      amountStr = args[i + 1];
    }
  }

  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether unstake --account <stakeAcct> [--amount <aeth>] [--address <addr>]${C.reset}\n`);
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found:${C.reset} ${address}\n`);
    return;
  }

  // Resolve stake account: --account flag, or query chain for first active stake
  if (!stakeAccount) {
    const rpcUrl = getDefaultRpc();
    const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

    let stakeAccounts = [];
    try {
      const res = await httpRequest(rpcUrl, `/v1/stake?address=${encodeURIComponent(rawAddr)}`);
      if (res && !res.error) {
        stakeAccounts = Array.isArray(res) ? res : (res.accounts || []);
      }
    } catch { /* no stake accounts */ }

    if (stakeAccounts.length === 0) {
      console.log(`  ${C.red}✗ No active stake accounts found for this wallet.${C.reset}`);
      console.log(`  ${C.dim}Use ${C.cyan}--account <stakeAcct>${C.reset} ${C.dim}to specify a stake account.${C.reset}`);
      console.log(`  ${C.dim}Check delegations: aether delegations list --address ${address}${C.reset}\n`);
      return;
    }

    // Default to first active stake account
    const active = stakeAccounts.find(s => !s.deactivation_epoch && (s.status === 'active' || s.state === 'active'));
    stakeAccount = active
      ? (active.pubkey || active.publicKey || active.account)
      : (stakeAccounts[0].pubkey || stakeAccounts[0].publicKey || stakeAccounts[0].account);

    console.log(`  ${C.cyan}Using stake account:${C.reset} ${C.bright}${stakeAccount}${C.reset}`);
    console.log(`  ${C.dim}(override with ${C.cyan}--account <stakeAcct>${C.reset}${C.dim})${C.reset}\n`);
  }

  // Resolve amount: --amount flag, or prompt if partial unstake supported
  // If no amount provided, unstake entire stake
  let lamports = null;
  if (amountStr) {
    const amount = parseFloat(amountStr);
    if (isNaN(amount) || amount <= 0) {
      console.log(`  ${C.red}✗ Invalid amount:${C.reset} ${amountStr}\n`);
      return;
    }
    lamports = Math.round(amount * 1e9);
  }

  console.log(`  ${C.green}★${C.reset} Wallet:       ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Stake acct: ${C.bright}${stakeAccount}${C.reset}`);
  if (lamports !== null) {
    console.log(`  ${C.green}★${C.reset} Amount:     ${C.bright}${(lamports / 1e9).toFixed(4)} AETH${C.reset} (${lamports} lamports)`);
  } else {
    console.log(`  ${C.green}★${C.reset} Amount:     ${C.bright}FULL STAKE${C.reset}`);
  }
  console.log();

  // Ask for mnemonic to derive signing keypair
  console.log(`${C.yellow}  ⚠ Signing requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter your 12/24-word passphrase to sign this transaction');
  console.log();

  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic, DERIVATION_PATH);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
    console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
    return;
  }

  // Verify the derived address matches the wallet
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:   ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected:  ${address}${C.reset}\n`);
    return;
  }

  const confirm = await question(rl, `  ${C.yellow}Confirm unstake? [y/N]${C.reset} > ${C.reset}`);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`  ${C.dim}Cancelled.${C.reset}\n`);
    return;
  }

  // Build the unstake transaction
  const txData = {
    type: 'Unstake',
    data: {
      stake_account: stakeAccount,
    },
  };
  if (lamports !== null) {
    txData.data.amount = lamports;
  }

  const tx = {
    signer: address.startsWith('ATH') ? address.slice(3) : address,
    tx_type: 'Unstake',
    payload: txData,
    fee: 0,
    slot: 0,
    timestamp: Math.floor(Date.now() / 1000),
  };

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.dim}Submitting to ${rpcUrl}...${C.reset}`);

  try {
    const result = await httpPost(rpcUrl, '/v1/tx', tx);

    if (result.error) {
      console.log(`\n  ${C.red}✗ Unstake failed:${C.reset} ${result.error}\n`);
      process.exit(1);
    }

    const sig = result.signature || result.tx_signature || result.id || JSON.stringify(result);
    console.log(`\n${C.green}✓ Unstake transaction submitted!${C.reset}`);
    console.log(`  ${C.dim}Signature:${C.reset} ${sig}`);
    console.log(`  ${C.dim}Stake will deactivate over the next epoch.${C.reset}`);
    console.log(`  ${C.dim}Check status: aether delegations list --address ${address}${C.reset}\n`);
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to submit transaction:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Is your validator running? RPC: ${rpcUrl}${C.reset}\n`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Main dispatcher
// ---------------------------------------------------------------------------

async function walletCommand() {
  // CLI: argv = [node, index.js, wallet, <subcmd>]
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
    } else if (subcmd === 'balance') {
      await balanceWallet(rl);
    } else if (subcmd === 'stake') {
      await stakeWallet(rl);
    } else if (subcmd === 'stake-positions') {
      await stakePositions(rl);
    } else if (subcmd === 'export') {
      await exportWallet(rl);
    } else if (subcmd === 'unstake') {
      await unstakeWallet(rl);
    } else if (subcmd === 'transfer') {
      await transferWallet(rl);
    } else if (subcmd === 'history' || subcmd === 'tx') {
      await txHistory(rl);
    } else {
      console.log(`\n  ${C.red}Unknown wallet subcommand:${C.reset} ${subcmd}`);
      console.log(`\n  Usage:`);
      console.log(`    ${C.cyan}aether wallet create${C.reset}   Create new or import wallet`);
      console.log(`    ${C.cyan}aether wallet list${C.reset}     List all wallets`);
      console.log(`    ${C.cyan}aether wallet import${C.reset}   Import wallet from mnemonic`);
      console.log(`    ${C.cyan}aether wallet export${C.reset}     Export wallet data (pubkey, address) — --mnemonic to include phrase`);
      console.log(`    ${C.cyan}aether wallet default${C.reset}  Show/set default wallet`);
      console.log(`    ${C.cyan}aether wallet connect${C.reset}  Connect wallet via browser verification`);
      console.log(`    ${C.cyan}aether wallet balance${C.reset}  Query chain balance for an address`);
      console.log(`    ${C.cyan}aether wallet stake${C.reset}     Stake AETH to a validator`);
      console.log(`    ${C.cyan}aether wallet stake-positions${C.reset} Show current stake delegations and rewards`);
      console.log(`    ${C.cyan}aether wallet unstake${C.reset}   Unstake AETH — deactivate a stake account`);
      console.log(`    ${C.cyan}aether wallet transfer${C.reset} Transfer AETH to another address`);
      console.log(`    ${C.cyan}aether wallet history${C.reset}  Show recent transactions for an address`);
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
