#!/usr/bin/env node
/**
 * aether-cli nft
 *
 * NFT management commands for the Aether blockchain.
 * Create, mint, transfer, and manage NFTs with real RPC calls.
 *
 * Usage:
 *   aether nft create --metadata <url> [--royalties <bps>] [--json]
 *   aether nft mint --nft <id> --amount <n> [--to <addr>] [--json]
 *   aether nft transfer --nft <id> --to <addr> [--json]
 *   aether nft update --nft <id> --metadata <url> [--json]
 *   aether nft list --address <addr> [--json]
 *   aether nft info --nft <id> [--json]
 *
 * SDK wired to:
 *   - client.sendTransaction(tx)         → POST /v1/transaction
 *   - client.getAccountInfo(addr)      → GET /v1/account/<addr>
 *   - client.getNFT(nftId)              → GET /v1/nft/<id>
 *   - client.getNFTHoldings(address)    → GET /v1/nft-holdings/<addr>
 *   - client.getSlot()                  → GET /v1/slot
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const bip39 = require('bip39');

// Import SDK for ALL blockchain RPC calls
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
  blue: '\x1b[34m',
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

function shortAddress(addr) {
  if (!addr || addr.length < 20) return addr || 'unknown';
  return addr.slice(0, 8) + '...' + addr.slice(-8);
}

function formatTimestamp(ts) {
  if (!ts) return 'unknown';
  const date = new Date(ts * 1000);
  return date.toLocaleString();
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
  console.log(`${C.dim}Enter your 12 or 24-word passphrase, one space-separated line:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

async function askConfirm(rl, text) {
  const ans = await question(rl, `\n${C.yellow}${text} [y/N]${C.reset} > `);
  return ans.trim().toLowerCase().startsWith('y');
}

// ============================================================================
// NFT SDK Fetchers (REAL RPC CALLS)
// ============================================================================

/**
 * Fetch NFT details via SDK
 * REAL RPC: GET /v1/nft/<id>
 */
async function fetchNFT(rpcUrl, nftId) {
  const client = createClient(rpcUrl);
  try {
    const nft = await client.getNFT(nftId);
    if (!nft || nft.error) return null;
    return {
      id: nft.id || nftId,
      creator: nft.creator || nft.mint_authority,
      metadata: nft.metadata_url || nft.metadata,
      royalties: nft.royalties || nft.royalty_bps || 0,
      supply: nft.supply || nft.current_supply || 0,
      maxSupply: nft.max_supply,
      createdAt: nft.created_at,
      updateAuthority: nft.update_authority,
    };
  } catch (err) {
    return null;
  }
}

/**
 * Fetch NFT holdings for an address via SDK
 * REAL RPC: GET /v1/nft-holdings/<addr>
 */
async function fetchNFTHoldings(rpcUrl, address) {
  const client = createClient(rpcUrl);
  try {
    const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
    const holdings = await client.getNFTHoldings(rawAddr);
    if (!Array.isArray(holdings)) return [];
    return holdings.map(h => ({
      nftId: h.nft_id || h.id || h.mint,
      amount: h.amount || h.balance || 1,
      acquiredAt: h.acquired_at,
      metadata: h.metadata_url,
    }));
  } catch (err) {
    return [];
  }
}

// ============================================================================
// NFT Create Command
// ============================================================================

async function nftCreate(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  const rl = createRl();

  // Resolve wallet address
  let address = args.address;
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address provided.${C.reset}`);
    console.log(`  ${C.dim}Set default: aether wallet default --set <addr>${C.reset}\n`);
    rl.close();
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`\n  ${C.red}✗ Wallet not found locally: ${address}${C.reset}`);
    console.log(`  ${C.dim}Import it: aether wallet import${C.reset}\n`);
    rl.close();
    return;
  }

  // Get metadata URL
  let metadataUrl = args.metadata;
  if (!metadataUrl) {
    console.log(`\n${C.bright}${C.cyan}── Create NFT ────────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Enter metadata URL (IPFS/Arweave/HTTPS):${C.reset}`);
    metadataUrl = await question(rl, `  Metadata URL > ${C.reset}`);
    if (!metadataUrl) {
      console.log(`\n  ${C.red}✗ Metadata URL is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get royalties
  let royalties = args.royalties || 0;
  if (!args.royalties && !isJson) {
    const roy = await question(rl, `  ${C.dim}Royalties (basis points, 0-10000) [0]:${C.reset} `);
    royalties = parseInt(roy, 10) || 0;
  }

  if (royalties < 0 || royalties > 10000) {
    console.log(`\n  ${C.red}✗ Royalties must be between 0 and 10000 basis points.${C.reset}\n`);
    rl.close();
    return;
  }

  // Summary
  console.log(`\n  ${C.green}★${C.reset} Creator:    ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Metadata:   ${C.bright}${metadataUrl}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Royalties:  ${C.bright}${royalties} bps (${(royalties / 100).toFixed(2)}%)${C.reset}`);
  console.log();

  if (args.dryRun) {
    console.log(`  ${C.yellow}⚠ Dry run - no transaction submitted${C.reset}\n`);
    rl.close();
    return;
  }

  // Get mnemonic for signing
  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the NFT creation');
    keyPair = deriveKeypair(mnemonic);
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== address) {
      console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}`);
      console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
      console.log(`  ${C.dim}  Expected: ${address}${C.reset}\n`);
      rl.close();
      return;
    }
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }

  const confirm = await askConfirm(rl, 'Create this NFT?');
  if (!confirm) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();

  // Build and submit transaction via SDK
  const client = createClient(rpc);
  const slot = await client.getSlot().catch(() => 0);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  const tx = {
    signer: rawAddr,
    tx_type: 'CreateNFT',
    payload: {
      type: 'CreateNFT',
      data: {
        metadata_url: metadataUrl,
        royalties: royalties,
      },
    },
    fee: 10000, // Higher fee for NFT creation
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  tx.signature = signTransaction(tx, keyPair.secretKey);

  try {
    if (!isJson) {
      console.log(`\n  ${C.dim}Submitting via SDK to ${rpc}...${C.reset}`);
    }
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    // Generate NFT ID from result
    const nftId = result.nft_id || result.mint || `NFT-${result.signature?.slice(0, 16) || 'UNKNOWN'}`;

    if (isJson) {
      console.log(JSON.stringify({
        success: true,
        nft_id: nftId,
        creator: address,
        metadata_url: metadataUrl,
        royalties_bps: royalties,
        tx_signature: result.signature || result.txid,
        slot: result.slot || slot,
        rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${C.green}✓ NFT created successfully!${C.reset}\n`);
      console.log(`  ${C.green}★${C.reset} NFT ID:      ${C.bright}${C.cyan}${nftId}${C.reset}`);
      console.log(`  ${C.green}★${C.reset} Creator:     ${address}`);
      console.log(`  ${C.green}★${C.reset} Metadata:    ${metadataUrl}`);
      console.log(`  ${C.green}★${C.reset} Royalties:   ${(royalties / 100).toFixed(2)}%`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}  Tx Signature: ${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log(`  ${C.dim}  Slot:         ${result.slot || slot}${C.reset}`);
      console.log();
      console.log(`  ${C.dim}Next steps:${C.reset}`);
      console.log(`    ${C.cyan}aether nft mint --nft ${nftId} --amount 1${C.reset}`);
      console.log(`    ${C.cyan}aether nft info --nft ${nftId}${C.reset}`);
      console.log();
    }
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        creator: address,
        metadata_url: metadataUrl,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ NFT creation failed: ${err.message}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// NFT Mint Command
// ============================================================================

async function nftMint(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  const rl = createRl();

  // Resolve wallet address
  let address = args.address;
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address provided.${C.reset}\n`);
    rl.close();
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`\n  ${C.red}✗ Wallet not found locally: ${address}${C.reset}\n`);
    rl.close();
    return;
  }

  // Get NFT ID
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n${C.bright}${C.cyan}── Mint NFT ─────────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Enter NFT ID to mint:${C.reset}`);
    nftId = await question(rl, `  NFT ID > ${C.reset}`);
    if (!nftId) {
      console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get amount
  let amount = args.amount;
  if (!amount) {
    const amt = await question(rl, `  ${C.dim}Amount to mint [1]:${C.reset} `);
    amount = parseInt(amt, 10) || 1;
  } else {
    amount = parseInt(amount, 10);
  }

  if (isNaN(amount) || amount < 1) {
    console.log(`\n  ${C.red}✗ Invalid amount.${C.reset}\n`);
    rl.close();
    return;
  }

  // Get recipient (optional, defaults to self)
  let recipient = args.to || address;

  // Verify NFT exists
  const nftInfo = await fetchNFT(rpc, nftId);
  if (!nftInfo && !isJson) {
    console.log(`\n  ${C.yellow}⚠ Warning: Could not verify NFT exists.${C.reset}`);
    console.log(`  ${C.dim}  Continuing anyway...${C.reset}\n`);
  }

  // Summary
  console.log(`\n  ${C.green}★${C.reset} NFT:        ${C.bright}${nftId}${C.reset}`);
  if (nftInfo) {
    console.log(`  ${C.green}★${C.reset} Creator:    ${shortAddress(nftInfo.creator)}`);
    console.log(`  ${C.green}★${C.reset} Current Supply: ${nftInfo.supply}`);
  }
  console.log(`  ${C.green}★${C.reset} Amount:     ${C.bright}${amount}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Recipient:  ${C.bright}${recipient}${C.reset}`);
  console.log();

  if (args.dryRun) {
    console.log(`  ${C.yellow}⚠ Dry run - no transaction submitted${C.reset}\n`);
    rl.close();
    return;
  }

  // Get mnemonic for signing
  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the mint');
    keyPair = deriveKeypair(mnemonic);
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== address) {
      console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}\n`);
      rl.close();
      return;
    }
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }

  const confirm = await askConfirm(rl, 'Mint these NFTs?');
  if (!confirm) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();

  // Build and submit transaction via SDK
  const client = createClient(rpc);
  const slot = await client.getSlot().catch(() => 0);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
  const rawRecipient = recipient.startsWith('ATH') ? recipient.slice(3) : recipient;

  const tx = {
    signer: rawAddr,
    tx_type: 'MintNFT',
    payload: {
      type: 'MintNFT',
      data: {
        nft_id: nftId,
        amount: amount,
        recipient: rawRecipient,
      },
    },
    fee: 5000,
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  tx.signature = signTransaction(tx, keyPair.secretKey);

  try {
    if (!isJson) {
      console.log(`\n  ${C.dim}Submitting via SDK...${C.reset}`);
    }
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    if (isJson) {
      console.log(JSON.stringify({
        success: true,
        nft_id: nftId,
        amount: amount,
        recipient: recipient,
        minter: address,
        tx_signature: result.signature || result.txid,
        slot: result.slot || slot,
        rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${C.green}✓ NFT minted successfully!${C.reset}\n`);
      console.log(`  ${C.green}★${C.reset} NFT ID:     ${C.cyan}${nftId}${C.reset}`);
      console.log(`  ${C.green}★${C.reset} Amount:     ${amount}`);
      console.log(`  ${C.green}★${C.reset} Recipient:  ${recipient}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}  Tx: ${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log(`  ${C.dim}  Slot: ${result.slot || slot}${C.reset}`);
      console.log();
    }
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        nft_id: nftId,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ NFT mint failed: ${err.message}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// NFT Transfer Command
// ============================================================================

async function nftTransfer(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  const rl = createRl();

  // Resolve wallet address
  let address = args.address;
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address provided.${C.reset}\n`);
    rl.close();
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`\n  ${C.red}✗ Wallet not found: ${address}${C.reset}\n`);
    rl.close();
    return;
  }

  // Get NFT ID
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n${C.bright}${C.cyan}── Transfer NFT ─────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Enter NFT ID to transfer:${C.reset}`);
    nftId = await question(rl, `  NFT ID > ${C.reset}`);
    if (!nftId) {
      console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get recipient
  let recipient = args.to;
  if (!recipient) {
    console.log(`  ${C.dim}Enter recipient address:${C.reset}`);
    recipient = await question(rl, `  Recipient > ${C.reset}`);
    if (!recipient) {
      console.log(`\n  ${C.red}✗ Recipient address is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get amount (for semi-fungible NFTs)
  let amount = args.amount || 1;

  // Summary
  console.log(`\n  ${C.green}★${C.reset} NFT:        ${C.bright}${nftId}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} From:       ${address}`);
  console.log(`  ${C.green}★${C.reset} To:         ${C.bright}${C.cyan}${recipient}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Amount:     ${amount}`);
  console.log();

  if (args.dryRun) {
    console.log(`  ${C.yellow}⚠ Dry run - no transaction submitted${C.reset}\n`);
    rl.close();
    return;
  }

  // Get mnemonic for signing
  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the transfer');
    keyPair = deriveKeypair(mnemonic);
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== address) {
      console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}\n`);
      rl.close();
      return;
    }
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }

  const confirm = await askConfirm(rl, `Transfer ${amount} of ${nftId} to ${shortAddress(recipient)}?`);
  if (!confirm) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();

  // Build and submit transaction via SDK
  const client = createClient(rpc);
  const slot = await client.getSlot().catch(() => 0);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
  const rawRecipient = recipient.startsWith('ATH') ? recipient.slice(3) : recipient;

  const tx = {
    signer: rawAddr,
    tx_type: 'TransferNFT',
    payload: {
      type: 'TransferNFT',
      data: {
        nft_id: nftId,
        recipient: rawRecipient,
        amount: amount,
      },
    },
    fee: 5000,
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  tx.signature = signTransaction(tx, keyPair.secretKey);

  try {
    if (!isJson) {
      console.log(`\n  ${C.dim}Submitting via SDK...${C.reset}`);
    }
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    if (isJson) {
      console.log(JSON.stringify({
        success: true,
        nft_id: nftId,
        amount: amount,
        from: address,
        to: recipient,
        tx_signature: result.signature || result.txid,
        slot: result.slot || slot,
        rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${C.green}✓ NFT transferred successfully!${C.reset}\n`);
      console.log(`  ${C.green}★${C.reset} NFT ID:    ${C.cyan}${nftId}${C.reset}`);
      console.log(`  ${C.green}★${C.reset} From:      ${shortAddress(address)}`);
      console.log(`  ${C.green}★${C.reset} To:        ${C.cyan}${shortAddress(recipient)}${C.reset}`);
      console.log(`  ${C.green}★${C.reset} Amount:    ${amount}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}  Tx: ${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log();
    }
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        nft_id: nftId,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ NFT transfer failed: ${err.message}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// NFT Update Metadata Command
// ============================================================================

async function nftUpdate(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  const rl = createRl();

  // Resolve wallet address
  let address = args.address;
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address provided.${C.reset}\n`);
    rl.close();
    return;
  }

  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`\n  ${C.red}✗ Wallet not found: ${address}${C.reset}\n`);
    rl.close();
    return;
  }

  // Get NFT ID
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n${C.bright}${C.cyan}── Update NFT Metadata ──────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Enter NFT ID to update:${C.reset}`);
    nftId = await question(rl, `  NFT ID > ${C.reset}`);
    if (!nftId) {
      console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Get new metadata URL
  let metadataUrl = args.metadata;
  if (!metadataUrl) {
    console.log(`  ${C.dim}Enter new metadata URL:${C.reset}`);
    metadataUrl = await question(rl, `  New Metadata URL > ${C.reset}`);
    if (!metadataUrl) {
      console.log(`\n  ${C.red}✗ Metadata URL is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }

  // Verify NFT exists
  const nftInfo = await fetchNFT(rpc, nftId);
  if (!nftInfo) {
    console.log(`\n  ${C.red}✗ NFT not found: ${nftId}${C.reset}\n`);
    rl.close();
    return;
  }

  // Check if user is update authority
  if (nftInfo.updateAuthority && nftInfo.updateAuthority !== address) {
    console.log(`\n  ${C.yellow}⚠ Warning: You may not be the update authority.${C.reset}`);
    console.log(`  ${C.dim}  Update Authority: ${nftInfo.updateAuthority}${C.reset}`);
    console.log(`  ${C.dim}  Your Address:     ${address}${C.reset}\n`);
  }

  // Summary
  console.log(`\n  ${C.green}★${C.reset} NFT:           ${C.bright}${nftId}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Current:       ${C.dim}${nftInfo.metadata}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} New Metadata:  ${C.bright}${metadataUrl}${C.reset}`);
  console.log();

  if (args.dryRun) {
    console.log(`  ${C.yellow}⚠ Dry run - no transaction submitted${C.reset}\n`);
    rl.close();
    return;
  }

  // Get mnemonic for signing
  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the update');
    keyPair = deriveKeypair(mnemonic);
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== address) {
      console.log(`\n  ${C.red}✗ Passphrase mismatch!${C.reset}\n`);
      rl.close();
      return;
    }
  } catch (e) {
    console.log(`\n  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }

  const confirm = await askConfirm(rl, 'Update metadata?');
  if (!confirm) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();

  // Build and submit transaction via SDK
  const client = createClient(rpc);
  const slot = await client.getSlot().catch(() => 0);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  const tx = {
    signer: rawAddr,
    tx_type: 'UpdateMetadata',
    payload: {
      type: 'UpdateMetadata',
      data: {
        nft_id: nftId,
        metadata_url: metadataUrl,
      },
    },
    fee: 5000,
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
  };

  tx.signature = signTransaction(tx, keyPair.secretKey);

  try {
    if (!isJson) {
      console.log(`\n  ${C.dim}Submitting via SDK...${C.reset}`);
    }
    const result = await client.sendTransaction(tx);

    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }

    if (isJson) {
      console.log(JSON.stringify({
        success: true,
        nft_id: nftId,
        new_metadata: metadataUrl,
        updater: address,
        tx_signature: result.signature || result.txid,
        slot: result.slot || slot,
        rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${C.green}✓ NFT metadata updated!${C.reset}\n`);
      console.log(`  ${C.green}★${C.reset} NFT ID:       ${C.cyan}${nftId}${C.reset}`);
      console.log(`  ${C.green}★${C.reset} New Metadata: ${metadataUrl}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}  Tx: ${(result.signature || result.txid).slice(0, 40)}...${C.reset}`);
      }
      console.log();
    }
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        nft_id: nftId,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Metadata update failed: ${err.message}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// NFT List Command
// ============================================================================

async function nftList(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  let address = args.address;

  // Resolve address
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    if (isJson) {
      console.log(JSON.stringify({ error: 'No address provided and no default wallet' }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ No wallet address specified.${C.reset}\n`);
    }
    return;
  }

  // Fetch NFT holdings via SDK (REAL RPC)
  const holdings = await fetchNFTHoldings(rpc, address);

  if (isJson) {
    console.log(JSON.stringify({
      address,
      rpc,
      holdings: holdings.map(h => ({
        nft_id: h.nftId,
        amount: h.amount,
        metadata: h.metadata,
        acquired_at: h.acquiredAt,
      })),
      total_nfts: holdings.length,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  console.log(`\n${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║              NFT HOLDINGS — ${shortAddress(address).padEnd(30)}║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════╝${C.reset}\n`);
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}\n`);

  if (holdings.length === 0) {
    console.log(`  ${C.yellow}⚠ No NFTs found for this wallet.${C.reset}`);
    console.log(`  ${C.dim}Create an NFT:${C.reset}`);
    console.log(`    ${C.cyan}aether nft create --metadata <url>${C.reset}\n`);
    return;
  }

  console.log(`  ${C.dim}┌─────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.dim}│${C.reset} ${C.bright}#  NFT ID                          Amount  Metadata${C.reset}            ${C.dim}│${C.reset}`);
  console.log(`  ${C.dim}├─────────────────────────────────────────────────────────────────────┤${C.reset}`);

  holdings.forEach((h, i) => {
    const num = (i + 1).toString().padStart(2);
    const shortId = shortAddress(h.nftId).padEnd(30);
    const amt = h.amount.toString().padStart(6);
    const meta = h.metadata ? h.metadata.substring(0, 20) + '...' : 'N/A';
    console.log(`  ${C.dim}│${C.reset} ${num} ${shortId} ${amt}    ${C.dim}${meta}${C.reset} ${C.dim}│${C.reset}`);
  });

  console.log(`  ${C.dim}└─────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log(`\n  ${C.bright}Total NFTs:${C.reset} ${holdings.length}`);
  console.log(`  ${C.dim}SDK: getNFTHoldings()${C.reset}\n`);
}

// ============================================================================
// NFT Info Command
// ============================================================================

async function nftInfo(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  const nftId = args.nft;

  if (!nftId) {
    if (isJson) {
      console.log(JSON.stringify({ error: 'NFT ID required (--nft <id>)' }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ NFT ID required.${C.reset}`);
      console.log(`  ${C.dim}Usage: aether nft info --nft <id>${C.reset}\n`);
    }
    return;
  }

  // Fetch NFT info via SDK (REAL RPC)
  const nft = await fetchNFT(rpc, nftId);

  if (!nft) {
    if (isJson) {
      console.log(JSON.stringify({ error: 'NFT not found', nft_id: nftId }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ NFT not found: ${nftId}${C.reset}\n`);
    }
    return;
  }

  if (isJson) {
    console.log(JSON.stringify({
      nft_id: nft.id,
      creator: nft.creator,
      metadata_url: nft.metadata,
      royalties_bps: nft.royalties,
      supply: nft.supply,
      max_supply: nft.maxSupply,
      update_authority: nft.updateAuthority,
      created_at: nft.createdAt,
      rpc,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  console.log(`\n${C.bright}${C.cyan}── NFT Details ────────────────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.green}★${C.reset} NFT ID:          ${C.bright}${C.cyan}${nft.id}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Creator:         ${shortAddress(nft.creator)}`);
  console.log(`  ${C.green}★${C.reset} Metadata:        ${C.blue}${nft.metadata}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Royalties:       ${(nft.royalties / 100).toFixed(2)}%`);
  console.log(`  ${C.green}★${C.reset} Supply:          ${nft.supply}${nft.maxSupply ? ' / ' + nft.maxSupply : ''}`);
  if (nft.updateAuthority) {
    console.log(`  ${C.green}★${C.reset} Update Authority: ${shortAddress(nft.updateAuthority)}`);
  }
  if (nft.createdAt) {
    console.log(`  ${C.dim}  Created:         ${formatTimestamp(nft.createdAt)}${C.reset}`);
  }
  console.log();
  console.log(`  ${C.dim}SDK: getNFT()${C.reset}\n`);
}

// ============================================================================
// CLI Args Parser
// ============================================================================

function parseArgs() {
  const rawArgs = process.argv.slice(3);
  const subcmd = rawArgs[0] || 'list';
  const allArgs = rawArgs.slice(1);

  const rpcIndex = allArgs.findIndex(a => a === '--rpc' || a === '-r');
  const rpc = rpcIndex !== -1 && allArgs[rpcIndex + 1] ? allArgs[rpcIndex + 1] : getDefaultRpc();

  const parsed = {
    subcmd,
    rpc,
    json: allArgs.includes('--json') || allArgs.includes('-j'),
    dryRun: allArgs.includes('--dry-run'),
    address: null,
    nft: null,
    metadata: null,
    to: null,
    amount: null,
    royalties: null,
  };

  const addrIdx = allArgs.findIndex(a => a === '--address' || a === '-a');
  if (addrIdx !== -1 && allArgs[addrIdx + 1]) parsed.address = allArgs[addrIdx + 1];

  const nftIdx = allArgs.findIndex(a => a === '--nft' || a === '-n');
  if (nftIdx !== -1 && allArgs[nftIdx + 1]) parsed.nft = allArgs[nftIdx + 1];

  const metaIdx = allArgs.findIndex(a => a === '--metadata' || a === '-m');
  if (metaIdx !== -1 && allArgs[metaIdx + 1]) parsed.metadata = allArgs[metaIdx + 1];

  const toIdx = allArgs.findIndex(a => a === '--to' || a === '-t');
  if (toIdx !== -1 && allArgs[toIdx + 1]) parsed.to = allArgs[toIdx + 1];

  const amtIdx = allArgs.findIndex(a => a === '--amount' || a === '-x');
  if (amtIdx !== -1 && allArgs[amtIdx + 1]) parsed.amount = allArgs[amtIdx + 1];

  const royIdx = allArgs.findIndex(a => a === '--royalties' || a === '-r');
  if (royIdx !== -1 && allArgs[royIdx + 1]) parsed.royalties = parseInt(allArgs[royIdx + 1], 10);

  return parsed;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli nft${C.reset} — NFT Management

${C.bright}USAGE${C.reset}
    aether nft create --metadata <url> [--royalties <bps>] [--json]
    aether nft mint --nft <id> --amount <n> [--to <addr>] [--json]
    aether nft transfer --nft <id> --to <addr> [--amount <n>] [--json]
    aether nft update --nft <id> --metadata <url> [--json]
    aether nft list [--address <addr>] [--json]
    aether nft info --nft <id> [--json]

${C.bright}COMMANDS${C.reset}
    create    Create a new NFT
    mint      Mint additional supply of an existing NFT
    transfer  Transfer NFT to another address
    update    Update NFT metadata URL
    list      Show all NFTs held by a wallet
    info      Show detailed info about a specific NFT

${C.bright}OPTIONS${C.reset}
    --metadata <url>   Metadata URL (IPFS/Arweave/HTTPS)
    --royalties <bps>   Royalties in basis points (0-10000)
    --nft <id>          NFT identifier
    --amount <n>        Amount to mint/transfer
    --to <addr>         Recipient address
    --address <addr>    Wallet address (default: configured default)
    --rpc <url>         RPC endpoint
    --json              Output JSON
    --dry-run           Preview without submitting

${C.bright}EXAMPLES${C.reset}
    aether nft create --metadata ipfs://Qm... --royalties 500
    aether nft mint --nft NFTabc... --amount 10 --to ATH...
    aether nft transfer --nft NFTabc... --to ATH... --amount 1
    aether nft update --nft NFTabc... --metadata ipfs://QmNew...
    aether nft list --address ATH...
    aether nft info --nft NFTabc...

${C.green}✓ Fully wired to @jellylegsai/aether-sdk${C.reset}
`);
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function nftCommand() {
  const args = parseArgs();

  if (args.subcmd === '--help' || args.subcmd === '-h' || args.subcmd === 'help') {
    showHelp();
    return;
  }

  switch (args.subcmd) {
    case 'create':
      await nftCreate(args);
      break;
    case 'mint':
      await nftMint(args);
      break;
    case 'transfer':
      await nftTransfer(args);
      break;
    case 'update':
      await nftUpdate(args);
      break;
    case 'list':
      await nftList(args);
      break;
    case 'info':
      await nftInfo(args);
      break;
    default:
      console.log(`\n  ${C.red}✗ Unknown subcommand: ${args.subcmd}${C.reset}`);
      showHelp();
      process.exit(1);
  }
}

// Export for module use
module.exports = { nftCommand };

// Run if called directly
if (require.main === module) {
  nftCommand().catch(err => {
    console.error(`\n${C.red}✗ NFT command failed:${C.reset}`, err.message, '\n');
    process.exit(1);
  });
}
