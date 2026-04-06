#!/usr/bin/env node
/**
 * aether-cli nft
 *
 * Complete NFT management suite - fully wired to @jellylegsai/aether-sdk.
 * All blockchain calls use real HTTP RPC via AetherClient.
 *
 * Commands:
 *   aether nft create    --metadata <url> [--royalties <bps>] [--address <wallet>]
 *   aether nft list      --address <wallet> [--creator <addr>] [--json]
 *   aether nft transfer  --nft <id> --to <addr> [--from <wallet>]
 *   aether nft info      --nft <id> [--json]
 *   aether nft update    --nft <id> --metadata <url> [--address <wallet>]
 *   aether nft burn      --nft <id> [--address <wallet>]
 *
 * SDK Methods:
 *   - client.createNFT()     → Creates new NFT with metadata
 *   - client.transferNFT()   → Transfer NFT ownership
 *   - client.updateMetadata()→ Update NFT metadata URL
 *   - getTokenAccounts()     → List NFTs for address
 *   - getAccountInfo()       → Get NFT details
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
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
  blue: '\x1b[34m',
};

const CLI_VERSION = '1.0.0';
const DERIVATION_PATH = "m/44'/7777777'/0'/0'";

// ============================================================================
// SDK Client Setup
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

function truncate(str, len) {
  if (!str) return '';
  if (str.length <= len) return str;
  return str.slice(0, len - 3) + '...';
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
// NFT Create Command - Uses SDK createNFT()
// ============================================================================

async function nftCreate(args) {
  const rl = createRl();
  const rpc = args.rpc || getDefaultRpc();
  
  // Resolve wallet address
  let address = args.address;
  if (!address) {
    const config = loadConfig();
    address = config.defaultWallet;
  }
  
  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address specified.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether nft create --metadata <url> --address <wallet>${C.reset}\n`);
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
    console.log(`\n${C.bright}${C.cyan}── NFT Creation ─────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Enter the metadata URL for your NFT:${C.reset}`);
    metadataUrl = await question(rl, `  Metadata URL > ${C.reset}`);
    if (!metadataUrl.trim()) {
      console.log(`\n  ${C.red}✗ Metadata URL is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }
  
  // Get royalties (default 5% = 500 bps)
  let royalties = args.royalties;
  if (!royalties) {
    const royaltyAns = await question(rl, `  ${C.dim}Royalties (basis points, default 500 = 5%):${C.reset} `);
    royalties = parseInt(royaltyAns.trim() || '500', 10);
  } else {
    royalties = parseInt(royalties, 10);
  }
  
  if (isNaN(royalties) || royalties < 0 || royalties > 10000) {
    console.log(`\n  ${C.red}✗ Invalid royalties. Must be 0-10000 basis points.${C.reset}\n`);
    rl.close();
    return;
  }
  
  console.log(`\n  ${C.green}★${C.reset} Creator:   ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Metadata:  ${C.bright}${truncate(metadataUrl, 50)}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} Royalties: ${C.bright}${(royalties / 100).toFixed(2)}%${C.reset}`);
  console.log();
  
  // Ask for mnemonic for signing
  console.log(`${C.yellow}  ⚠ Creating NFT requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter passphrase to sign the NFT creation');
  console.log();
  
  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Verify derived address matches
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected: ${address}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Confirm
  const confirm = await question(rl, `  ${C.yellow}Create NFT? [y/N]${C.reset} > `);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();
  
  // Create NFT via SDK
  const client = createClient(rpc);
  
  console.log(`\n  ${C.dim}Submitting NFT creation via SDK...${C.reset}`);
  
  try {
    // SDK call: createNFT with signing function
    const result = await client.createNFT({
      creator: address.startsWith('ATH') ? address.slice(3) : address,
      metadataUrl: metadataUrl.trim(),
      royalties: royalties,
      signFn: async (tx) => signTransaction(tx, keyPair.secretKey),
    });
    
    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }
    
    const nftId = result.nft_id || result.id || result.signature;
    
    console.log(`\n${C.green}✓ NFT created successfully!${C.reset}`);
    console.log(`  ${C.dim}NFT ID:${C.reset}     ${C.cyan}${C.bright}${nftId}${C.reset}`);
    console.log(`  ${C.dim}Creator:${C.reset}    ${address}`);
    console.log(`  ${C.dim}Metadata:${C.reset}   ${truncate(metadataUrl, 45)}`);
    console.log(`  ${C.dim}Royalties:${C.reset}  ${(royalties / 100).toFixed(2)}%`);
    if (result.signature) {
      console.log(`  ${C.dim}Signature:${C.reset}  ${shortAddress(result.signature)}`);
    }
    if (result.slot) {
      console.log(`  ${C.dim}Slot:${C.reset}       ${result.slot}`);
    }
    console.log(`  ${C.dim}SDK:${C.reset}        createNFT() → POST /v1/transaction`);
    console.log();
    console.log(`  ${C.dim}View your NFT:${C.reset}`);
    console.log(`    ${C.cyan}aether nft info --nft ${nftId}${C.reset}\n`);
    
    return result;
  } catch (err) {
    console.log(`\n  ${C.red}✗ NFT creation failed:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Common causes:${C.reset}`);
    console.log(`    • Insufficient balance for NFT creation fee (typically 0.01 AETH)`);
    console.log(`    • RPC endpoint not accepting transactions`);
    console.log(`    • Invalid metadata URL format\n`);
    process.exit(1);
  }
}

// ============================================================================
// NFT List Command - Uses SDK getTokenAccounts()
// ============================================================================

async function nftList(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  
  let address = args.address;
  if (!address) {
    const config = loadConfig();
    address = config.defaultWallet;
  }
  
  if (!address) {
    if (isJson) {
      console.log(JSON.stringify({ error: 'No address provided' }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ No wallet address specified.${C.reset}`);
      console.log(`  ${C.dim}Usage: aether nft list --address <wallet>${C.reset}\n`);
    }
    return;
  }
  
  const client = createClient(rpc);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
  
  if (!isJson) {
    console.log(`\n${C.bright}${C.cyan}── NFT Collection ───────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Wallet:${C.reset} ${address}`);
    console.log(`  ${C.dim}RPC:${C.reset}    ${rpc}\n`);
    console.log(`  ${C.dim}Fetching NFTs via SDK...${C.reset}\n`);
  }
  
  try {
    // SDK call: getTokenAccounts → GET /v1/tokens/<address>
    const tokenAccounts = await client.getTokenAccounts(rawAddr);
    
    // Filter for NFTs (amount = 1, has metadata)
    const nfts = (tokenAccounts || []).filter(t => 
      t.amount === '1' || t.amount === 1 || t.is_nft || t.nft_id
    );
    
    if (isJson) {
      console.log(JSON.stringify({
        address,
        rpc,
        nft_count: nfts.length,
        nfts: nfts.map(n => ({
          id: n.mint || n.nft_id || n.id,
          amount: n.amount,
          metadata: n.metadata_url || n.metadata,
          creator: n.creator,
          royalties: n.royalties,
        })),
        cli_version: CLI_VERSION,
        fetched_at: new Date().toISOString(),
      }, null, 2));
      return;
    }
    
    if (nfts.length === 0) {
      console.log(`  ${C.yellow}⚠ No NFTs found for this wallet.${C.reset}`);
      console.log(`  ${C.dim}Create your first NFT:${C.reset}`);
      console.log(`    ${C.cyan}aether nft create --metadata <url>${C.reset}\n`);
      return;
    }
    
    console.log(`  ${C.bright}Found ${nfts.length} NFT${nfts.length === 1 ? '' : 's'}${C.reset}\n`);
    
    // Display table
    console.log(`  ${C.dim}┌─────────────────────────────────────────────────────────────────┐${C.reset}`);
    console.log(`  ${C.dim}│${C.reset}  ${C.bright}#  NFT ID${C.reset}                    ${C.bright}Metadata${C.reset}               ${C.bright}Royalties${C.reset}  ${C.dim}│${C.reset}`);
    console.log(`  ${C.dim}├─────────────────────────────────────────────────────────────────┤${C.reset}`);
    
    nfts.forEach((nft, i) => {
      const id = shortAddress(nft.mint || nft.nft_id || nft.id, 10).padEnd(24);
      const meta = truncate(nft.metadata_url || nft.metadata || 'N/A', 22).padEnd(22);
      const royalty = nft.royalties !== undefined 
        ? `${(nft.royalties / 100).toFixed(0)}%`.padEnd(9)
        : 'N/A      ';
      console.log(`  ${C.dim}│${C.reset}  ${(i + 1).toString().padStart(2)} ${C.cyan}${id}${C.reset} ${meta} ${royalty} ${C.dim}│${C.reset}`);
    });
    
    console.log(`  ${C.dim}└─────────────────────────────────────────────────────────────────┘${C.reset}`);
    console.log();
    console.log(`  ${C.dim}SDK: getTokenAccounts() → GET /v1/tokens/${shortAddress(rawAddr)}${C.reset}\n`);
    
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({ error: err.message, address }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to fetch NFTs:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}Is your validator running? RPC: ${rpc}${C.reset}\n`);
    }
  }
}

// ============================================================================
// NFT Transfer Command - Uses SDK transferNFT()
// ============================================================================

async function nftTransfer(args) {
  const rl = createRl();
  const rpc = args.rpc || getDefaultRpc();
  
  let fromAddress = args.from;
  if (!fromAddress) {
    const config = loadConfig();
    fromAddress = config.defaultWallet;
  }
  
  if (!fromAddress) {
    console.log(`\n  ${C.red}✗ No wallet address specified.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether nft transfer --nft <id> --to <addr> --from <wallet>${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Get NFT ID
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n  ${C.dim}Enter the NFT ID to transfer:${C.reset}`);
    nftId = await question(rl, `  NFT ID > ${C.reset}`);
    if (!nftId.trim()) {
      console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }
  
  // Get recipient
  let toAddress = args.to;
  if (!toAddress) {
    console.log(`  ${C.dim}Enter recipient address:${C.reset}`);
    toAddress = await question(rl, `  Recipient > ${C.reset}`);
    if (!toAddress.trim()) {
      console.log(`\n  ${C.red}✗ Recipient address is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }
  
  console.log(`\n  ${C.green}★${C.reset} From: ${C.bright}${fromAddress}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} To:   ${C.bright}${toAddress}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} NFT:  ${C.bright}${shortAddress(nftId)}${C.reset}`);
  console.log();
  
  // Ask for mnemonic
  console.log(`${C.yellow}  ⚠ Transferring NFT requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter passphrase to sign the transfer');
  console.log();
  
  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Verify derived address
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== fromAddress) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected: ${fromAddress}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Confirm
  const confirm = await question(rl, `  ${C.yellow}Confirm NFT transfer? [y/N]${C.reset} > `);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();
  
  // Transfer via SDK
  const client = createClient(rpc);
  
  console.log(`\n  ${C.dim}Submitting NFT transfer via SDK...${C.reset}`);
  
  try {
    const result = await client.transferNFT({
      from: fromAddress.startsWith('ATH') ? fromAddress.slice(3) : fromAddress,
      nftId: nftId.trim(),
      to: toAddress.startsWith('ATH') ? toAddress.slice(3) : toAddress,
      signFn: async (tx) => signTransaction(tx, keyPair.secretKey),
    });
    
    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }
    
    console.log(`\n${C.green}✓ NFT transferred successfully!${C.reset}`);
    console.log(`  ${C.dim}NFT ID:${C.reset}      ${C.cyan}${C.bright}${nftId}${C.reset}`);
    console.log(`  ${C.dim}From:${C.reset}        ${fromAddress}`);
    console.log(`  ${C.dim}To:${C.reset}          ${C.green}${toAddress}${C.reset}`);
    if (result.signature) {
      console.log(`  ${C.dim}Signature:${C.reset}   ${shortAddress(result.signature)}`);
    }
    if (result.slot) {
      console.log(`  ${C.dim}Slot:${C.reset}        ${result.slot}`);
    }
    console.log(`  ${C.dim}SDK:${C.reset}         transferNFT() → POST /v1/transaction`);
    console.log();
    console.log(`  ${C.dim}Verify transfer:${C.reset}`);
    console.log(`    ${C.cyan}aether nft list --address ${toAddress}${C.reset}\n`);
    
  } catch (err) {
    console.log(`\n  ${C.red}✗ NFT transfer failed:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Common causes:${C.reset}`);
    console.log(`    • You don't own this NFT`);
    console.log(`    • Invalid NFT ID`);
    console.log(`    • Recipient address is invalid`);
    console.log(`    • Insufficient balance for transaction fee\n`);
    process.exit(1);
  }
}

// ============================================================================
// NFT Info Command - Uses SDK getAccountInfo()
// ============================================================================

async function nftInfo(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether nft info --nft <id>${C.reset}\n`);
    return;
  }
  
  const client = createClient(rpc);
  
  if (!isJson) {
    console.log(`\n${C.bright}${C.cyan}── NFT Details ──────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Fetching NFT info via SDK...${C.reset}\n`);
  }
  
  try {
    // SDK call: getAccountInfo → GET /v1/account/<nft_id>
    const accountInfo = await client.getAccountInfo(nftId);
    
    if (!accountInfo || accountInfo.error) {
      throw new Error(accountInfo?.error || 'NFT not found');
    }
    
    // Extract NFT-specific data
    const nftData = {
      id: nftId,
      owner: accountInfo.owner,
      metadata: accountInfo.data?.metadata || accountInfo.metadata_url,
      creator: accountInfo.data?.creator,
      royalties: accountInfo.data?.royalties,
      lamports: accountInfo.lamports,
      rentEpoch: accountInfo.rent_epoch,
    };
    
    if (isJson) {
      console.log(JSON.stringify({
        nft: nftData,
        rpc,
        cli_version: CLI_VERSION,
        fetched_at: new Date().toISOString(),
      }, null, 2));
      return;
    }
    
    console.log(`  ${C.green}★${C.reset} NFT ID:      ${C.bright}${C.cyan}${nftId}${C.reset}`);
    console.log(`  ${C.green}★${C.reset} Owner:       ${nftData.owner || 'Unknown'}`);
    console.log(`  ${C.green}★${C.reset} Creator:     ${nftData.creator || 'Unknown'}`);
    console.log(`  ${C.green}★${C.reset} Metadata:    ${truncate(nftData.metadata, 50) || 'N/A'}`);
    if (nftData.royalties !== undefined) {
      console.log(`  ${C.green}★${C.reset} Royalties:   ${(nftData.royalties / 100).toFixed(2)}%`);
    }
    if (nftData.lamports !== undefined) {
      console.log(`  ${C.green}★${C.reset} Lamports:    ${nftData.lamports.toLocaleString()}`);
    }
    console.log();
    console.log(`  ${C.dim}SDK: getAccountInfo() → GET /v1/account/${shortAddress(nftId)}${C.reset}\n`);
    
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({ error: err.message, nft_id: nftId }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to fetch NFT info:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}The NFT may not exist or the RPC endpoint is unavailable.${C.reset}\n`);
    }
  }
}

// ============================================================================
// NFT Update Command - Uses SDK updateMetadata()
// ============================================================================

async function nftUpdate(args) {
  const rl = createRl();
  const rpc = args.rpc || getDefaultRpc();
  
  let address = args.address;
  if (!address) {
    const config = loadConfig();
    address = config.defaultWallet;
  }
  
  if (!address) {
    console.log(`\n  ${C.red}✗ No wallet address specified.${C.reset}`);
    console.log(`  ${C.dim}Usage: aether nft update --nft <id> --metadata <url> --address <wallet>${C.reset}\n`);
    rl.close();
    return;
  }
  
  let nftId = args.nft;
  if (!nftId) {
    console.log(`\n  ${C.dim}Enter the NFT ID to update:${C.reset}`);
    nftId = await question(rl, `  NFT ID > ${C.reset}`);
    if (!nftId.trim()) {
      console.log(`\n  ${C.red}✗ NFT ID is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }
  
  let metadataUrl = args.metadata;
  if (!metadataUrl) {
    console.log(`  ${C.dim}Enter new metadata URL:${C.reset}`);
    metadataUrl = await question(rl, `  New Metadata URL > ${C.reset}`);
    if (!metadataUrl.trim()) {
      console.log(`\n  ${C.red}✗ Metadata URL is required.${C.reset}\n`);
      rl.close();
      return;
    }
  }
  
  console.log(`\n  ${C.green}★${C.reset} Updater:   ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} NFT:       ${C.bright}${shortAddress(nftId)}${C.reset}`);
  console.log(`  ${C.green}★${C.reset} New Meta:  ${C.bright}${truncate(metadataUrl, 40)}${C.reset}`);
  console.log();
  
  // Ask for mnemonic
  console.log(`${C.yellow}  ⚠ Updating metadata requires your wallet passphrase.${C.reset}`);
  const mnemonic = await askMnemonic(rl, 'Enter passphrase to sign the update');
  console.log();
  
  let keyPair;
  try {
    keyPair = deriveKeypair(mnemonic);
  } catch (e) {
    console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Verify derived address
  const derivedAddress = formatAddress(keyPair.publicKey);
  if (derivedAddress !== address) {
    console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
    console.log(`  ${C.dim}  Derived:  ${derivedAddress}${C.reset}`);
    console.log(`  ${C.dim}  Expected: ${address}${C.reset}\n`);
    rl.close();
    return;
  }
  
  // Confirm
  const confirm = await question(rl, `  ${C.yellow}Confirm metadata update? [y/N]${C.reset} > `);
  if (!confirm.trim().toLowerCase().startsWith('y')) {
    console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
    rl.close();
    return;
  }
  rl.close();
  
  // Update via SDK
  const client = createClient(rpc);
  
  console.log(`\n  ${C.dim}Submitting metadata update via SDK...${C.reset}`);
  
  try {
    const result = await client.updateMetadata({
      creator: address.startsWith('ATH') ? address.slice(3) : address,
      nftId: nftId.trim(),
      metadataUrl: metadataUrl.trim(),
      signFn: async (tx) => signTransaction(tx, keyPair.secretKey),
    });
    
    if (result.error) {
      throw new Error(result.error.message || JSON.stringify(result.error));
    }
    
    console.log(`\n${C.green}✓ Metadata updated successfully!${C.reset}`);
    console.log(`  ${C.dim}NFT ID:${C.reset}      ${C.cyan}${C.bright}${nftId}${C.reset}`);
    console.log(`  ${C.dim}New Meta:${C.reset}    ${truncate(metadataUrl, 45)}`);
    if (result.signature) {
      console.log(`  ${C.dim}Signature:${C.reset}   ${shortAddress(result.signature)}`);
    }
    if (result.slot) {
      console.log(`  ${C.dim}Slot:${C.reset}        ${result.slot}`);
    }
    console.log(`  ${C.dim}SDK:${C.reset}         updateMetadata() → POST /v1/transaction`);
    console.log();
    console.log(`  ${C.dim}Verify update:${C.reset}`);
    console.log(`    ${C.cyan}aether nft info --nft ${nftId}${C.reset}\n`);
    
  } catch (err) {
    console.log(`\n  ${C.red}✗ Metadata update failed:${C.reset} ${err.message}`);
    console.log(`  ${C.dim}Common causes:${C.reset}`);
    console.log(`    • You are not the NFT creator`);
    console.log(`    • Invalid NFT ID`);
    console.log(`    • Insufficient balance for transaction fee\n`);
    process.exit(1);
  }
}

// ============================================================================
// Help
// ============================================================================

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli nft${C.reset} — NFT management suite

${C.bright}COMMANDS${C.reset}
    create    Create a new NFT with metadata and royalties
    list      List all NFTs owned by a wallet
    transfer  Transfer an NFT to another address
    info      Show detailed info about an NFT
    update    Update NFT metadata (creator only)

${C.bright}USAGE${C.reset}
    aether nft create --metadata <url> [--royalties <bps>] [--address <wallet>]
    aether nft list --address <wallet> [--json]
    aether nft transfer --nft <id> --to <addr> [--from <wallet>]
    aether nft info --nft <id> [--json]
    aether nft update --nft <id> --metadata <url> [--address <wallet>]

${C.bright}SDK METHODS USED${C.reset}
    client.createNFT()      → POST /v1/transaction (CreateNFT)
    client.transferNFT()    → POST /v1/transaction (TransferNFT)
    client.updateMetadata() → POST /v1/transaction (UpdateMetadata)
    client.getTokenAccounts() → GET /v1/tokens/<addr>
    client.getAccountInfo() → GET /v1/account/<addr>

${C.bright}EXAMPLES${C.reset}
    aether nft create --metadata https://example.com/nft.json --royalties 500
    aether nft list --address ATHxxx... --json
    aether nft transfer --nft NFTxxx... --to ATHyyy...
    aether nft info --nft NFTxxx... --json

${C.bright}NOTES${C.reset}
    • Only the creator can update NFT metadata
    • Royalties are specified in basis points (100 = 1%, 500 = 5%)
    • NFT creation fee is typically 0.01 AETH
    • Metadata must be a valid JSON file hosted at the provided URL

${C.green}✓ Fully wired to @jellylegsai/aether-sdk${C.reset}
`);
}

// ============================================================================
// Argument Parser
// ============================================================================

function parseArgs() {
  const rawArgs = process.argv.slice(3);
  const subcmd = rawArgs[0] || 'help';
  const allArgs = rawArgs.slice(1);
  
  const rpcIndex = allArgs.findIndex(a => a === '--rpc' || a === '-r');
  const rpc = rpcIndex !== -1 && allArgs[rpcIndex + 1] ? allArgs[rpcIndex + 1] : getDefaultRpc();
  
  const parsed = {
    subcmd,
    rpc,
    json: allArgs.includes('--json') || allArgs.includes('-j'),
    address: null,
    from: null,
    to: null,
    nft: null,
    metadata: null,
    royalties: null,
  };
  
  const addrIdx = allArgs.findIndex(a => a === '--address' || a === '-a');
  if (addrIdx !== -1 && allArgs[addrIdx + 1]) parsed.address = allArgs[addrIdx + 1];
  
  const fromIdx = allArgs.findIndex(a => a === '--from' || a === '-f');
  if (fromIdx !== -1 && allArgs[fromIdx + 1]) parsed.from = allArgs[fromIdx + 1];
  
  const toIdx = allArgs.findIndex(a => a === '--to' || a === '-t');
  if (toIdx !== -1 && allArgs[toIdx + 1]) parsed.to = allArgs[toIdx + 1];
  
  const nftIdx = allArgs.findIndex(a => a === '--nft' || a === '-n');
  if (nftIdx !== -1 && allArgs[nftIdx + 1]) parsed.nft = allArgs[nftIdx + 1];
  
  const metaIdx = allArgs.findIndex(a => a === '--metadata' || a === '-m');
  if (metaIdx !== -1 && allArgs[metaIdx + 1]) parsed.metadata = allArgs[metaIdx + 1];
  
  const royaltyIdx = allArgs.findIndex(a => a === '--royalties' || a === '-r');
  if (royaltyIdx !== -1 && allArgs[royaltyIdx + 1]) parsed.royalties = allArgs[royaltyIdx + 1];
  
  return parsed;
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function nftCommand() {
  const args = parseArgs();
  
  switch (args.subcmd) {
    case 'create':
      await nftCreate(args);
      break;
    case 'list':
      await nftList(args);
      break;
    case 'transfer':
      await nftTransfer(args);
      break;
    case 'info':
      await nftInfo(args);
      break;
    case 'update':
      await nftUpdate(args);
      break;
    case 'help':
    case '--help':
    case '-h':
    default:
      showHelp();
  }
}

module.exports = { nftCommand };

if (require.main === module) {
  nftCommand().catch(err => {
    console.error(`\n${C.red}✗ NFT command failed:${C.reset}`, err.message, '\n');
    process.exit(1);
  });
}
