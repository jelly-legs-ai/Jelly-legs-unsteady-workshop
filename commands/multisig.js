#!/usr/bin/env node
/**
 * aether-cli multisig
 *
 * Multi-signature wallet management for Aether.
 * Create 2-of-3, 3-of-5, or any M-of-N multisig wallets,
 * add signers, view threshold info, and send transactions.
 *
 * Usage:
 *   aether multisig create --threshold <m> --signers <addr1,addr2,...>
 *   aether multisig list                         List all multisig wallets
 *   aether multisig info --address <addr>        Show threshold, signers, balance
 *   aether multisig add-signer --address <addr> --signer <newAddr>
 *   aether multisig send --address <addr> --to <recipient> --amount <aeth> [--json]
 *
 * Requires AETHER_RPC env var or local node (default: http://127.0.0.1:8899)
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
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

const CLI_VERSION = '1.2.5';
const MULTISIG_VERSION = 1;

// ---------------------------------------------------------------------------
// Paths & config
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getMultisigDir() {
  return path.join(getAetherDir(), 'multisig');
}

function ensureDir(p) {
  if (!fs.existsSync(p)) fs.mkdirSync(p, { recursive: true });
}

function getMultisigFilePath(address) {
  return path.join(getMultisigDir(), `${address}.json`);
}

function loadConfig() {
  const p = path.join(getAetherDir(), 'config.json');
  if (!fs.existsSync(p)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fp = path.join(getAetherDir(), 'wallets', `${address}.json`);
  if (!fs.existsSync(fp)) return null;
  return JSON.parse(fs.readFileSync(fp, 'utf8'));
}

// ---------------------------------------------------------------------------
// Crypto helpers
// ---------------------------------------------------------------------------

function deriveKeypair(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) throw new Error('Invalid mnemonic phrase.');
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return { publicKey: Buffer.from(keyPair.publicKey), secretKey: Buffer.from(keyPair.secretKey) };
}

function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

function shortAddress(addr) {
  if (!addr || addr.length < 16) return addr;
  return addr.slice(0, 8) + '…' + addr.slice(-8);
}

function isValidAddress(addr) {
  return addr && addr.startsWith('ATH') && addr.length >= 36;
}

// ---------------------------------------------------------------------------
// SDK Integration - Real blockchain RPC calls
// ---------------------------------------------------------------------------

const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

/**
 * Fetch account balance via SDK (GET /v1/account/<addr>)
 */
async function fetchAccountBalance(rpcUrl, address) {
  const client = createClient(rpcUrl);
  try {
    const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
    const account = await client.getAccountInfo(rawAddr);
    return account && !account.error ? account.lamports : 0;
  } catch {
    return null;
  }
}

/**
 * Submit transaction via SDK (POST /v1/transaction)
 */
async function submitTransaction(rpcUrl, tx) {
  const client = createClient(rpcUrl);
  return client.sendTransaction(tx);
}

function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

// ---------------------------------------------------------------------------
// Multisig address derivation
// Derived from threshold M and signer list — sorted lexicographically
// ---------------------------------------------------------------------------

/**
 * Derive a deterministic multisig address from signers + threshold.
 * Uses SHA-512 of sorted(signers) + threshold as the seed for a keypair.
 * This gives a deterministic address without requiring on-chain registration.
 */
function deriveMultisigAddress(signers, threshold) {
  // Sort signers lexicographically for deterministic derivation
  const sortedSigners = [...signers].sort();
  const data = JSON.stringify({ signers: sortedSigners, threshold, v: MULTISIG_VERSION });
  const hash = crypto.createHash('sha512').update(data).digest();
  // Use first 32 bytes as seed for nacl keypair
  const seed32 = hash.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return {
    address: formatAddress(Buffer.from(keyPair.publicKey)),
    publicKey: Buffer.from(keyPair.publicKey),
  };
}

// ---------------------------------------------------------------------------
// Multisig storage
// ---------------------------------------------------------------------------

function saveMultisig(ms) {
  ensureDir(getMultisigDir());
  const fp = getMultisigFilePath(ms.address);
  fs.writeFileSync(fp, JSON.stringify(ms, null, 2));
}

function loadMultisig(address) {
  const fp = getMultisigFilePath(address);
  if (!fs.existsSync(fp)) return null;
  return JSON.parse(fs.readFileSync(fp, 'utf8'));
}

function listAllMultisig() {
  ensureDir(getMultisigDir());
  const files = fs.readdirSync(getMultisigDir()).filter(f => f.endsWith('.json'));
  const result = [];
  for (const f of files) {
    try {
      const ms = JSON.parse(fs.readFileSync(path.join(getMultisigDir(), f), 'utf8'));
      result.push(ms);
    } catch {}
  }
  return result.sort((a, b) => (a.created_at || '').localeCompare(b.created_at || ''));
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

function askMnemonic(rl, label = 'wallet passphrase') {
  return new Promise(async (res) => {
    console.log(`\n${C.cyan}Enter your 12/24-word ${label}:${C.reset}`);
    console.log(`${C.dim}One space-separated line:${C.reset}`);
    const raw = await question(rl, `  > ${C.reset}`);
    res(raw.trim().toLowerCase());
  });
}

// ---------------------------------------------------------------------------
// CREATE MULTISIG
// ---------------------------------------------------------------------------

async function createMultisig(rl, args) {
  console.log(`\n${C.bright}${C.cyan}── Create Multi-Signature Wallet ─────────────────────────${C.reset}\n`);

  // Parse --threshold and --signers from args
  let threshold = null;
  let signers = [];

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--threshold' || args[i] === '-t') && args[i + 1]) {
      threshold = parseInt(args[i + 1], 10);
    }
    if ((args[i] === '--signers' || args[i] === '-s') && args[i + 1]) {
      signers = args[i + 1].split(',').map(s => s.trim()).filter(Boolean);
    }
  }

  // Interactive prompts for missing values
  if (signers.length === 0) {
    console.log(`  ${C.cyan}Enter signer addresses (ATH...), separated by commas.${C.reset}`);
    console.log(`  ${C.dim}Example: ATHabc...,ATHdef...,ATHghi...${C.reset}`);
    const rawSigners = await question(rl, `  Signers: ${C.reset}`);
    signers = rawSigners.split(',').map(s => s.trim()).filter(s => s.length > 0);
  }

  if (signers.length < 2) {
    console.log(`  ${C.red}✗ A multisig wallet requires at least 2 signers.${C.reset}\n`);
    return;
  }

  // Validate all signer addresses
  const invalidSigners = signers.filter(s => !isValidAddress(s));
  if (invalidSigners.length > 0) {
    console.log(`  ${C.red}✗ Invalid signer addresses:${C.reset} ${invalidSigners.join(', ')}`);
    console.log(`  ${C.dim}All signers must start with 'ATH' and be at least 36 characters.${C.reset}\n`);
    return;
  }

  // Deduplicate
  const uniqueSigners = [...new Set(signers)];
  if (uniqueSigners.length !== signers.length) {
    console.log(`  ${C.yellow}⚠ Duplicate signers removed.${C.reset}`);
    signers = uniqueSigners;
  }

  if (threshold === null) {
    const defaultThreshold = Math.max(2, Math.ceil(signers.length / 2));
    const rawThresh = await question(rl, `  ${C.cyan}Threshold (M, required signatures)${C.reset} [${defaultThreshold}]: ${C.reset}`);
    threshold = rawThresh.trim() ? parseInt(rawThresh.trim(), 10) : defaultThreshold;
  }

  if (isNaN(threshold) || threshold < 1 || threshold > signers.length) {
    console.log(`  ${C.red}✗ Invalid threshold:${C.reset} ${threshold}. Must be between 1 and ${signers.length}.\n`);
    return;
  }

  console.log(`\n  ${C.green}★${C.reset} Signers (${signers.length}):`);
  for (const s of signers) {
    console.log(`     ${C.cyan}${shortAddress(s)}${C.reset}`);
  }
  console.log(`  ${C.green}★${C.reset} Threshold: ${C.bright}${threshold} of ${signers.length}${C.reset}`);
  console.log();

  const confirm = await question(rl, `  ${C.yellow}Create multisig wallet? [y/N]${C.reset} > ${C.reset}`);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`  ${C.dim}Cancelled.${C.reset}\n`);
    return;
  }

  const { address, publicKey } = deriveMultisigAddress(signers, threshold);

  const ms = {
    version: MULTISIG_VERSION,
    address,
    public_key: bs58.encode(publicKey),
    threshold,
    signers,
    created_at: new Date().toISOString(),
    derivation: 'off-chain deterministic',
    description: '',
  };

  saveMultisig(ms);

  console.log(`\n${C.green}✓ Multi-signature wallet created!${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Address: ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Threshold: ${threshold}/${signers.length}`);
  console.log(`  ${C.dim}  Saved to: ${getMultisigFilePath(address)}${C.reset}`);
  console.log(`  ${C.dim}  Use: aether multisig send --address ${address}${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// LIST MULTISIG
// ---------------------------------------------------------------------------

async function listMultisig(rl, args) {
  console.log(`\n${C.bright}${C.cyan}── Multi-Signature Wallets ────────────────────────────${C.reset}\n`);

  const all = listAllMultisig();

  if (all.length === 0) {
    console.log(`  ${C.dim}No multisig wallets found.${C.reset}`);
    console.log(`  ${C.dim}Create one with:${C.reset} ${C.cyan}aether multisig create --threshold 2 --signers addr1,addr2,addr3${C.reset}\n`);
    return;
  }

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.dim}Location: ${getMultisigDir()}${C.reset}\n`);

  for (const ms of all) {
    const shortAddr = shortAddress(ms.address);
    console.log(`  ${C.bright}${C.cyan}${ms.address}${C.reset}`);
    console.log(`  ${C.dim}  Threshold: ${ms.threshold}/${ms.signers.length}  Signers: ${ms.signers.length}${C.reset}`);
    console.log(`  ${C.dim}  Created:   ${new Date(ms.created_at).toLocaleString()}${C.reset}`);

    // Fetch on-chain balance via SDK (REAL RPC GET /v1/account/<addr>)
    try {
      const balance = await fetchAccountBalance(rpcUrl, ms.address);
      if (balance !== null) {
        console.log(`  ${C.green}✓ Balance:${C.reset} ${C.bright}${formatAether(balance)}${C.reset}`);
      }
    } catch {}

    console.log();
  }
}

// ---------------------------------------------------------------------------
// INFO
// ---------------------------------------------------------------------------

async function infoMultisig(rl, args) {
  console.log(`\n${C.bright}${C.cyan}── Multi-Signature Wallet Info ─────────────────────────${C.reset}\n`);

  let address = null;
  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      address = args[i + 1];
    }
  }

  if (!address) {
    // Try default wallet
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No address specified.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether multisig info --address <addr>${C.reset}\n`);
    return;
  }

  const ms = loadMultisig(address);
  if (!ms) {
    console.log(`  ${C.red}✗ Multisig wallet not found:${C.reset} ${address}`);
    console.log(`  ${C.dim}Check your wallets: aether multisig list${C.reset}\n`);
    return;
  }

  const rpcUrl = getDefaultRpc();
  console.log(`  ${C.green}★${C.reset} Address:    ${C.bright}${ms.address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Threshold: ${C.bright}${ms.threshold} of ${ms.signers.length}${C.reset}`);
  console.log(`  ${C.dim}  Public key: ${ms.public_key}${C.reset}`);
  console.log(`  ${C.dim}  Created:    ${new Date(ms.created_at).toLocaleString()}${C.reset}`);
  console.log(`  ${C.dim}  Version:    ${ms.version}${C.reset}`);
  console.log();

  console.log(`  ${C.bright}Signers (${ms.signers.length}):${C.reset}`);
  for (let i = 0; i < ms.signers.length; i++) {
    const s = ms.signers[i];
    const isYou = s === loadConfig().defaultWallet;
    const marker = isYou ? ` ${C.green}★ you${C.reset}` : '';
    console.log(`  ${i + 1}. ${C.cyan}${s}${C.reset}${marker}`);
  }
  console.log();

  // On-chain balance via SDK (REAL RPC GET /v1/account/<addr>)
  try {
    const balance = await fetchAccountBalance(rpcUrl, ms.address);
    if (balance !== null) {
      console.log(`  ${C.green}✓ Balance:${C.reset} ${C.bright}${formatAether(balance)}${C.reset}`);
    }
  } catch (err) {
    console.log(`  ${C.yellow}⚠ Could not fetch balance: ${err.message}${C.reset}`);
  }

  console.log();
}

// ---------------------------------------------------------------------------
// ADD SIGNER
// ---------------------------------------------------------------------------

async function addSignerMultisig(rl, args) {
  console.log(`\n${C.bright}${C.cyan}── Add Signer to Multi-Signature Wallet ─────────────────${C.reset}\n`);

  let address = null;
  let newSigner = null;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) address = args[i + 1];
    if ((args[i] === '--signer' || args[i] === '-s') && args[i + 1]) newSigner = args[i + 1];
  }

  if (!address || !newSigner) {
    console.log(`  ${C.red}✗ Missing required arguments.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether multisig add-signer --address <msAddr> --signer <newAddr>${C.reset}\n`);
    return;
  }

  const ms = loadMultisig(address);
  if (!ms) {
    console.log(`  ${C.red}✗ Multisig wallet not found:${C.reset} ${address}\n`);
    return;
  }

  if (!isValidAddress(newSigner)) {
    console.log(`  ${C.red}✗ Invalid signer address:${C.reset} ${newSigner}\n`);
    return;
  }

  if (ms.signers.includes(newSigner)) {
    console.log(`  ${C.yellow}⚠ Signer already in wallet:${C.reset} ${newSigner}\n`);
    return;
  }

  console.log(`  ${C.green}★${C.reset} Multisig:  ${C.bright}${shortAddress(address)}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Adding:    ${C.bright}${shortAddress(newSigner)}${C.reset}`);
  console.log(`  ${C.dim}  Current threshold: ${ms.threshold}/${ms.signers.length}${C.reset}`);
  console.log();

  // Re-derive the address with the new signer appended
  const newSigners = [...ms.signers, newSigner];
  const { address: newAddress } = deriveMultisigAddress(newSigners, ms.threshold);

  const newMs = {
    ...ms,
    address: newAddress,     // new address due to signer change
    signers: newSigners,
    updated_at: new Date().toISOString(),
    note: 'Address changed because signers list changed. Old address no longer valid.',
  };

  saveMultisig(newMs);

  console.log(`${C.green}✓ Signer added.${C.reset}`);
  console.log(`  ${C.yellow}⚠ Important: Changing signers creates a NEW wallet address.${C.reset}`);
  console.log(`  ${C.dim}  Old address: ${ms.address}${C.reset}`);
  console.log(`  ${C.dim}  New address: ${newAddress}${C.reset}`);
  console.log(`  ${C.dim}  Transfer all funds to the new address.${C.reset}`);
  console.log(`  ${C.dim}  Saved to: ${getMultisigFilePath(newAddress)}${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// SEND (multi-sig transaction)
// ---------------------------------------------------------------------------

async function sendMultisig(rl, args) {
  console.log(`\n${C.bright}${C.cyan}── Multi-Signature Send ──────────────────────────────────${C.reset}\n`);

  let address = null;
  let recipient = null;
  let amountStr = null;
  let asJson = false;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) address = args[i + 1];
    else if ((args[i] === '--to' || args[i] === '-t') && args[i + 1]) recipient = args[i + 1];
    else if ((args[i] === '--amount' || args[i] === '-m') && args[i + 1]) amountStr = args[i + 1];
    else if (args[i] === '--json' || args[i] === '-j') asJson = true;
  }

  if (!address || !recipient || !amountStr) {
    console.log(`  ${C.red}✗ Missing required arguments.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether multisig send --address <msAddr> --to <recipient> --amount <aeth>${C.reset}\n`);
    return;
  }

  const ms = loadMultisig(address);
  if (!ms) {
    console.log(`  ${C.red}✗ Multisig wallet not found:${C.reset} ${address}\n`);
    return;
  }

  const amount = parseFloat(amountStr);
  if (isNaN(amount) || amount <= 0) {
    console.log(`  ${C.red}✗ Invalid amount:${C.reset} ${amountStr}\n`);
    return;
  }

  const lamports = Math.round(amount * 1e9);
  const rpcUrl = getDefaultRpc();

  console.log(`  ${C.green}★${C.reset} Multisig: ${C.bright}${shortAddress(address)}${C.reset} (${ms.threshold}/${ms.signers.length})`);
  console.log(`  ${C.green}★${C.reset} To:       ${C.bright}${shortAddress(recipient)}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Amount:   ${C.bright}${amount} AETH${C.reset} (${lamports.toLocaleString()} lamports)`);
  console.log();

  // Fetch balance check via SDK (REAL RPC GET /v1/account/<addr>)
  try {
    const balance = await fetchAccountBalance(rpcUrl, address);
    if (balance !== null) {
      if (balance < lamports) {
        console.log(`  ${C.red}✗ Insufficient balance.${C.reset}`);
        console.log(`  ${C.dim}  Have: ${formatAether(balance)}  Need: ${formatAether(lamports)}${C.reset}\n`);
        return;
      }
      console.log(`  ${C.green}✓ Balance check passed:${C.reset} ${formatAether(balance)}`);
    }
  } catch (err) {
    console.log(`  ${C.yellow}⚠ Could not verify balance: ${err.message}${C.reset}`);
  }
  console.log();

  // Collect M signatures from signers
  console.log(`  ${C.yellow}⚠ This is a multi-signature transaction.${C.reset}`);
  console.log(`  ${C.dim}  Require ${ms.threshold} signature(s) from ${ms.signers.length} signer(s).${C.reset}`);
  console.log(`  ${C.dim}  Signers:${C.reset}`);
  for (const s of ms.signers) {
    console.log(`    ${C.cyan}${shortAddress(s)}${C.reset}`);
  }
  console.log();

  const signatures = [];
  const neededSigs = ms.threshold;

  for (let i = 0; i < ms.signers.length && signatures.length < neededSigs; i++) {
    const signer = ms.signers[i];
    console.log(`  ${C.cyan}[${signatures.length + 1}/${neededSigs}] Requesting signature from:${C.reset} ${C.bright}${shortAddress(signer)}${C.reset}`);

    const isYou = loadConfig().defaultWallet === signer;

    if (isYou) {
      // You are a signer — get your mnemonic to sign
      const mnemonic = await askMnemonic(rl, `your passphrase to sign`);
      let keyPair;
      try {
        keyPair = deriveKeypair(mnemonic);
      } catch (e) {
        console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}`);
        continue;
      }
      const derivedAddr = formatAddress(keyPair.publicKey);
      if (derivedAddr !== signer) {
        console.log(`  ${C.red}✗ Passphrase mismatch for signer ${shortAddress(signer)}.${C.reset}`);
        continue;
      }
      // Sign the transaction digest
      const txDigest = crypto.createHash('sha512')
        .update(JSON.stringify({ to: recipient, amount: lamports, from: address, nonce: Math.floor(Math.random() * 0xffffffff) }))
        .digest();
      const sig = nacl.sign.detached(txDigest, keyPair.secretKey);
      signatures.push({ signer, signature: bs58.encode(sig) });
      console.log(`  ${C.green}✓ Signed.${C.reset}`);
    } else {
      // Not you — simulate a signature request (in real impl, this would prompt via file/network)
      console.log(`  ${C.yellow}⚠ Cannot automatically collect signature for ${shortAddress(signer)}.${C.reset}`);
      console.log(`  ${C.dim}  For remote signers, use: aether multisig sign --signer ${signer} --tx <txId>${C.reset}`);
    }
    console.log();
  }

  if (signatures.length < neededSigs) {
    console.log(`  ${C.red}✗ Not enough signatures.${C.reset} Have ${signatures.length}, need ${neededSigs}.`);
    console.log(`  ${C.dim}  Transaction NOT submitted.${C.reset}\n`);
    return;
  }

  // Build multi-sig transaction
  const tx = {
    type: 'MultisigSend',
    from: address,
    to: recipient.startsWith('ATH') ? recipient.slice(3) : recipient,
    amount_lamports: lamports,
    threshold: ms.threshold,
    signers: ms.signers,
    signatures: signatures.map(s => s.signature),
    timestamp: Math.floor(Date.now() / 1000),
  };

  console.log(`  ${C.green}✓ Collected ${signatures.length} signature(s). Submitting...${C.reset}`);

  // Submit via SDK (REAL RPC POST /v1/transaction)
  try {
    const result = await submitTransaction(rpcUrl, tx);

    if (result.error) {
      console.log(`\n  ${C.red}✗ Transaction failed:${C.reset} ${result.error}\n`);
      process.exit(1);
    }

    const sig = result.signature || result.tx_signature || result.id || JSON.stringify(result);
    console.log(`\n${C.green}✓ Multi-sig transaction submitted!${C.reset}`);
    console.log(`  ${C.dim}Signature: ${sig}${C.reset}`);
    console.log(`  ${C.dim}From: ${address}${C.reset}`);
    console.log(`  ${C.dim}To: ${recipient}${C.reset}`);
    console.log(`  ${C.dim}Amount: ${formatAether(lamports)}${C.reset}`);
    console.log(`  ${C.dim}Signers used: ${signatures.length}/${ms.signers.length}${C.reset}`);
    console.log(`  ${C.dim}SDK: sendTransaction()${C.reset}\n`);
  } catch (err) {
    console.log(`  ${C.red}✗ Failed to submit transaction:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}\n`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Parse CLI args
// ---------------------------------------------------------------------------

function parseArgs() {
  // argv = [node, index.js, multisig, <subcmd>, ...]
  return process.argv.slice(3);
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli multisig${C.reset} — Multi-Signature Wallet Management

${C.bright}Usage:${C.reset}
  aether multisig create       --threshold <m> --signers <addr1,addr2,...>
  aether multisig list
  aether multisig info         --address <addr>
  aether multisig add-signer   --address <msAddr> --signer <newAddr>
  aether multisig send         --address <msAddr> --to <recipient> --amount <aeth>

${C.bright}Examples:${C.reset}
  aether multisig create --threshold 2 --signers ATHabc,ATHdef,ATHghi
  aether multisig list
  aether multisig info --address ATHxxxxx
  aether multisig add-signer --address ATHxxxxx --signer ATHnewww
  aether multisig send --address ATHxxxxx --to ATHdest --amount 10

${C.bright}Notes:${C.reset}
  Multi-sig wallets use off-chain deterministic address derivation.
  Changing signers always produces a new wallet address.
  All M signers must approve a transaction before it can be broadcast.
`.trim());
}

// ---------------------------------------------------------------------------
// Main dispatcher
// ---------------------------------------------------------------------------

async function multisigCommand() {
  const args = parseArgs();
  const subcmd = args[0];

  const rl = createRl();
  try {
    if (!subcmd || subcmd === 'help' || subcmd === '--help' || subcmd === '-h') {
      showHelp();
    } else if (subcmd === 'create') {
      await createMultisig(rl, args.slice(1));
    } else if (subcmd === 'list') {
      await listMultisig(rl, args.slice(1));
    } else if (subcmd === 'info') {
      await infoMultisig(rl, args.slice(1));
    } else if (subcmd === 'add-signer') {
      await addSignerMultisig(rl, args.slice(1));
    } else if (subcmd === 'send') {
      await sendMultisig(rl, args.slice(1));
    } else {
      console.log(`\n  ${C.red}Unknown multisig subcommand:${C.reset} ${subcmd}`);
      showHelp();
      process.exit(1);
    }
  } finally {
    rl.close();
  }
}

module.exports = { multisigCommand };

if (require.main === module) {
  multisigCommand();
}