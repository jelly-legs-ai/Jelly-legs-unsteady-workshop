/**
 * aether-cli sdk
 * 
 * Provides download links and install instructions for the Aether SDK,
 * Aether JS client, and FLUX/ATH token libraries.
 * 
 * Usage:
 *   aether-cli sdk              # Show all SDK options
 *   aether-cli sdk js           # Aether JS client
 *   aether-cli sdk rust         # Aether Rust SDK
 *   aether-cli sdk tokens       # FLUX/ATH token libraries
 *   aether-cli sdk types        # TypeScript/Rust type definitions for TX payloads
 */

const os = require('os');

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
  dim: '\x1b[2m',
  magenta: '\x1b[35m',
  blue: '\x1b[34m',
};

/**
 * Print the SDK banner
 */
function printBanner() {
  console.log(`
${colors.cyan}╔═══════════════════════════════════════════════════════════════╗
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}AETHER SDK${colors.reset}${colors.cyan}                                              ║
${colors.cyan}║   ${colors.bright}Developer Tools & Libraries${colors.reset}${colors.cyan}                            ║
${colors.cyan}║                                                               ║
${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
}

/**
 * Print a section header
 */
function printSection(title, icon = '📦') {
  console.log();
  console.log(`${colors.bright}${colors.cyan}${'═'.repeat(60)}${colors.reset}`);
  console.log(`${colors.bright}  ${icon} ${title}${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}${'═'.repeat(60)}${colors.reset}`);
  console.log();
}

/**
 * Print a code block
 */
function printCode(code, lang = 'bash') {
  console.log(`  ${colors.dim}[ ${lang} ]${colors.reset}`);
  console.log(`  ${colors.bright}${code}${colors.reset}`);
  console.log();
}

/**
 * Print a link
 */
function printLink(label, url) {
  console.log(`  ${colors.cyan}🔗 ${label}:${colors.reset}`);
  console.log(`     ${colors.blue}${url}${colors.reset}`);
  console.log();
}

/**
 * Show all SDK options
 */
function showAllSdks() {
  printBanner();
  
  console.log(`  ${colors.bright}Available SDKs and Libraries:${colors.reset}\n`);
  
  console.log(`  ${colors.green}npm${colors.reset}   aether-cli sdk install - Install @jellylegsai/aether-sdk  ← NEW!`);
  console.log(`  ${colors.yellow}npm${colors.reset}   aether-cli sdk js      - JavaScript/TypeScript client`);
  console.log(`  ${colors.yellow}npm${colors.reset}   aether-cli sdk rust    - Rust SDK for native development`);
  console.log(`  ${colors.yellow}npm${colors.reset}   aether-cli sdk tokens  - FLUX/ATH token libraries`);
  console.log(`  ${colors.yellow}npm${colors.reset}   aether-cli sdk docs    - Documentation portal`);
  console.log(`  ${colors.yellow}npm${colors.reset}   aether-cli sdk types   - TypeScript/Rust type definitions`);
  console.log();
  
  // Quick start
  printSection('⚡ Quick Start', '🚀');
  console.log('  Get started with Aether development in 3 steps:\n');
  console.log(`  1. ${colors.bright}Install the SDK:${colors.reset}`);
  printCode('npx aether-cli sdk install');
  console.log(`  2. ${colors.bright}Initialize your connection:${colors.reset}`);
  printCode("const aether = require('@jellylegsai/aether-sdk');\nconst client = new aether.AetherClient({ rpcUrl: 'http://localhost:8899' });");
  console.log(`  3. ${colors.bright}Start building!${colors.reset}`);
  console.log(`     ${colors.dim}See docs for full API reference${colors.reset}`);
  console.log();
  
  printLink('Documentation', 'https://docs.aether.network');
  printLink('GitHub Organization', 'https://github.com/aether-network');
  printLink('Discord Community', 'https://discord.gg/aether');
}

/**
 * Show JavaScript SDK info
 */
function showJsSdk() {
  printSection('Aether JavaScript Client', '📜');
  
  console.log(`  ${colors.bright}The official Aether JavaScript/TypeScript client library.${colors.reset}`);
  console.log(`  Provides a simple API for interacting with the Aether blockchain.${colors.reset}\n`);
  
  console.log(`  ${colors.green}✓ Stable Release${colors.reset}`);
  console.log(`  ${colors.dim}Version: 1.2.0${colors.reset}`);
  console.log();
  
  printSection('Transaction Types');
  console.log(`  ${colors.cyan}Transfer${colors.reset}      — ${colors.dim}Send AETH to another address${colors.reset}`);
  console.log(`    { recipient: string, amount: u64, nonce: u64 }`);
  console.log();
  console.log(`  ${colors.cyan}Stake${colors.reset}         — ${colors.dim}Delegate tokens to a validator${colors.reset}`);
  console.log(`    { validator: string, amount: u64 }`);
  console.log();
  console.log(`  ${colors.cyan}Unstake${colors.reset}       — ${colors.dim}Request withdrawal of staked tokens${colors.reset}`);
  console.log(`    { stake_account: string, amount: u64 }`);
  console.log();
  console.log(`  ${colors.cyan}ClaimRewards${colors.reset} — ${colors.dim}Claim accumulated staking rewards${colors.reset}`);
  console.log(`    { stake_account: string }`);
  console.log();
  console.log(`  ${colors.cyan}CreateNFT${colors.reset}    — ${colors.dim}Create a new NFT on-chain${colors.reset}`);
  console.log(`    { metadata_url: string, royalties: u16 }`);
  console.log();
  console.log(`  ${colors.cyan}MintNFT${colors.reset}      — ${colors.dim}Mint additional supply of an existing NFT${colors.reset}`);
  console.log(`    { nft_id: string, amount: u64 }`);
  console.log();
  console.log(`  ${colors.cyan}TransferNFT${colors.reset}  — ${colors.dim}Transfer an NFT to another address${colors.reset}`);
  console.log(`    { nft_id: string, recipient: string }`);
  console.log();
  console.log(`  ${colors.cyan}UpdateMetadata${colors.reset} — ${colors.dim}Update NFT metadata URL${colors.reset}`);
  console.log(`    { nft_id: string, metadata_url: string }`);
  console.log();
  
  printSection('Installation');
  printCode('npm install @aether-network/client');
  console.log('  or');
  printCode('yarn add @aether-network/client');
  console.log('  or');
  printCode('pnpm add @aether-network/client');
  
  printSection('Usage Example');
  const example = `const aether = require('@aether-network/client');

// Initialize client
const client = new aether.Client({
  rpcUrl: 'http://localhost:8899',
  wsUrl: 'ws://localhost:8900',
});

// Get slot info
const slot = await client.getSlot();
console.log('Current slot:', slot);

// Get account info (includes balance)
const account = await client.getAccountInfo(pubkey);
console.log('Balance:', account.lamports, 'lamports');

// Send Transfer transaction
const tx = await client.sendTransaction({
  type: 'Transfer',
  payload: {
    recipient: 'ATH...',
    amount: 1000000000, // 1 AETH in lamports
    nonce: 0,
  },
});
console.log('Transaction signature:', tx.signature);`;
  
  console.log(`  ${colors.dim}[ javascript ]${colors.reset}`);
  example.split('\n').forEach(line => {
    console.log(`  ${colors.bright}${line}${colors.reset}`);
  });
  console.log();
  
  printSection('RPC API Reference');
  console.log(`  ${colors.cyan}GET /v1/account/<addr>${colors.reset}  — ${colors.dim}Fetch account info + lamports balance${colors.reset}`);
  console.log(`  ${colors.cyan}GET /v1/slot${colors.reset}             — ${colors.dim}Get current slot number${colors.reset}`);
  console.log(`  ${colors.cyan}GET /v1/validators${colors.reset}       — ${colors.dim}List active validators${colors.reset}`);
  console.log(`  ${colors.cyan}POST /v1/tx${colors.reset}             — ${colors.dim}Submit signed transaction${colors.reset}`);
  console.log(`  ${colors.cyan}GET /v1/tx/<signature>${colors.reset} — ${colors.dim}Get transaction receipt${colors.reset}`);
  console.log();
  
  printLink('NPM Package', 'https://www.npmjs.com/package/@aether-network/client');
  printLink('TypeScript Docs', 'https://docs.aether.network/sdk/js');
  printLink('GitHub Repo', 'https://github.com/aether-network/aether-js');
}

/**
 * Show Rust SDK info
 */
function showRustSdk() {
  printSection('Aether Rust SDK', '🦀');
  
  console.log(`  ${colors.bright}Native Rust SDK for building Aether programs and clients.${colors.reset}`);
  console.log(`  Use this for validator plugins, custom programs, and high-performance tools.${colors.reset}\n`);
  
  console.log(`  ${colors.green}✓ Stable Release${colors.reset}`);
  console.log(`  ${colors.dim}Version: 1.2.0${colors.reset}`);
  console.log();
  
  printSection('Installation');
  printCode('cargo add aether-sdk');
  console.log('  or add to your Cargo.toml:');
  console.log();
  console.log(`  ${colors.dim}${colors.bgRed}toml${colors.reset}`);
  console.log(`  ${colors.bright}[dependencies]${colors.reset}`);
  console.log(`  ${colors.bright}aether-sdk = "1.2"${colors.reset}`);
  console.log();
  
  printSection('Usage Example');
  const rustExample = `use aether_sdk::{client::Client, pubkey::Pubkey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = Client::new("http://localhost:8899");
    
    // Get slot
    let slot = client.get_slot().await?;
    println!("Current slot: {}", slot);
    
    // Get balance
    let pubkey = Pubkey::from_str("...")?;
    let balance = client.get_balance(&pubkey).await?;
    println!("Balance: {} lamports", balance);
    
    Ok(())
}`;
  
  console.log(`  ${colors.dim}${colors.bgRed}rust${colors.reset}`);
  rustExample.split('\n').forEach(line => {
    console.log(`  ${colors.bright}${line}${colors.reset}`);
  });
  console.log();
  
  printSection('Features');
  console.log(`  ${colors.cyan}• Full RPC client${colors.reset}`);
  console.log(`  ${colors.cyan}• Program development framework${colors.reset}`);
  console.log(`  ${colors.cyan}• Account serialization/deserialization${colors.reset}`);
  console.log(`  ${colors.cyan}• Transaction building and signing${colors.reset}`);
  console.log(`  ${colors.cyan}• Async runtime support (tokio)${colors.reset}`);
  console.log();
  
  printLink('Crates.io', 'https://crates.io/crates/aether-sdk');
  printLink('API Docs', 'https://docs.rs/aether-sdk');
  printLink('GitHub Repo', 'https://github.com/aether-network/aether-rust');
}

/**
 * Show token libraries info
 */
function showTokensSdk() {
  printSection('FLUX / ATH Token Libraries', '🪙');
  
  console.log(`  ${colors.bright}Token libraries for FLUX (utility) and ATH (governance) tokens.${colors.reset}`);
  console.log(`  Use these to integrate Aether tokens into your applications.${colors.reset}\n`);
  
  console.log(`  ${colors.yellow}⚠ Beta Release${colors.reset}`);
  console.log(`  ${colors.dim}Version: 0.9.0 (testnet only)${colors.reset}`);
  console.log();
  
  printSection('Installation');
  console.log(`  ${colors.bright}JavaScript:${colors.reset}`);
  printCode('npm install @aether-network/tokens');
  console.log();
  console.log(`  ${colors.bright}Rust:${colors.reset}`);
  printCode('cargo add aether-tokens');
  
  printSection('Supported Tokens');
  console.log();
  console.log(`  ${colors.magenta}FLUX${colors.reset} - Utility Token`);
  console.log(`    • Purpose: Transaction fees, staking rewards`);
  console.log(`    • Decimals: 9`);
  console.log(`    • Mint: ${colors.dim}flux7x... (testnet)${colors.reset}`);
  console.log();
  console.log(`  ${colors.blue}ATH${colors.reset} - Governance Token`);
  console.log(`    • Purpose: Voting, protocol upgrades`);
  console.log(`    • Decimals: 6`);
  console.log(`    • Mint: ${colors.dim}athgov... (testnet)${colors.reset}`);
  console.log();
  
  printSection('Usage Example (JavaScript)');
  const tokenExample = `const { TokenClient, TOKENS } = require('@aether-network/tokens');

const client = new TokenClient(rpcUrl);

// Get FLUX balance
const fluxBalance = await client.getTokenBalance(pubkey, TOKENS.FLUX);
console.log('FLUX:', fluxBalance);

// Transfer FLUX
const tx = await client.transfer({
  mint: TOKENS.FLUX,
  from: senderPubkey,
  to: recipientPubkey,
  amount: 1000,
});`;
  
  console.log(`  ${colors.dim}${colors.bgRed}javascript${colors.reset}`);
  tokenExample.split('\n').forEach(line => {
    console.log(`  ${colors.bright}${line}${colors.reset}`);
  });
  console.log();
  
  printLink('Token Documentation', 'https://docs.aether.network/tokens');
  printLink('Token Registry', 'https://github.com/aether-network/token-registry');
  printLink('Testnet Faucet', 'https://faucet.aether.network');
}

/**
 * Show install instructions + run npm install for the user
 */
function showInstall() {
  printSection('Install Aether SDK', '📦');
  
  console.log(`  ${colors.bright}The @jellylegsai/aether-sdk package lets you:${colors.reset}`);
  console.log(`    ${colors.cyan}•${colors.reset} Query the Aether blockchain (balances, accounts, validators)`);
  console.log(`    ${colors.cyan}•${colors.reset} Submit real transactions (transfer, stake, unstake, claim)`);
  console.log(`    ${colors.cyan}•${colors.reset} Build DApps and automation scripts on top of Aether`);
  console.log();
  
  console.log(`  ${colors.bright}Installation options:${colors.reset}\n`);
  console.log(`  ${colors.green}1)${colors.reset} ${colors.cyan}Install all SDK packages (recommended):${colors.reset}`);
  console.log(`     ${colors.dim}npm install @jellylegsai/aether-sdk${colors.reset}`);
  console.log();
  
  printSection('Quick Install');
  
  console.log(`  ${colors.dim}The CLI can run the install command for you in your project directory.${colors.reset}`);
  console.log();
  
  const installCmd = 'npm install @jellylegsai/aether-sdk';
  console.log(`  ${colors.bright}Running:${colors.reset} ${colors.cyan}${installCmd}${colors.reset}`);
  console.log();
}

/**
 * Execute `npm install @jellylegsai/aether-sdk` in the user's project directory.
 * Detects the target directory from the nearest package.json, or uses cwd.
 */
async function runInstall(args) {
  const { exec } = require('child_process');
  const fs = require('fs');
  const path = require('path');
  
  // Detect install target: nearest package.json up from cwd, else cwd
  let targetDir = process.cwd();
  let searchDir = targetDir;
  for (let i = 0; i < 10; i++) {
    if (fs.existsSync(path.join(searchDir, 'package.json'))) {
      targetDir = searchDir;
      break;
    }
    const parent = path.dirname(searchDir);
    if (parent === searchDir) break;
    searchDir = parent;
  }
  
  const isCoreOnly = args.includes('--core');
  
  let pkgToInstall = '@jellylegsai/aether-sdk';
  
  const installCmd = `npm install ${pkgToInstall}`;
  
  printBanner();
  printSection('Installing Aether SDK', '📦');
  
  console.log(`  ${colors.bright}Target directory:${colors.reset} ${colors.cyan}${targetDir}${colors.reset}`);
  console.log(`  ${colors.bright}Package:${colors.reset}         ${colors.cyan}${pkgToInstall}${colors.reset}`);
  console.log(`  ${colors.dim}Registry:${colors.reset}         ${colors.blue}https://registry.npmjs.org${colors.reset}`);
  console.log();
  console.log(`  ${colors.dim}Running: ${installCmd}${colors.reset}`);
  console.log();
  console.log(`  ${colors.yellow}This may take a moment...${colors.reset}\n`);
  
  return new Promise((resolve) => {
    const child = exec(installCmd, { cwd: targetDir }, (err, stdout, stderr) => {
      if (err) {
        console.log(`  ${colors.red}✗ Install failed:${colors.reset} ${err.message}`);
        if (stderr) console.log(`  ${colors.dim}${stderr}${colors.reset}`);
        console.log();
        console.log(`  ${colors.dim}You can try manually:${colors.reset}`);
        console.log(`    ${colors.cyan}cd ${targetDir} && npm install @jellylegsai/aether-sdk${colors.reset}`);
        resolve({ success: false, error: err.message });
      } else {
        console.log(`  ${colors.green}✓ Install succeeded!${colors.reset}`);
        if (stdout) {
          const lines = stdout.split('\n').filter(l => l.trim());
          for (const line of lines.slice(-5)) {
            console.log(`  ${colors.dim}${line}${colors.reset}`);
          }
        }
        console.log();
        console.log(`  ${colors.bright}Next steps:${colors.reset}`);
        console.log(`    ${colors.dim}1. Import in your code:${colors.reset}`);
        console.log(`       ${colors.cyan}const aether = require('@jellylegsai/aether-sdk');${colors.reset}`);
        console.log();
        console.log(`    ${colors.dim}2. Initialize the client:${colors.reset}`);
        console.log(`       ${colors.cyan}const client = new aether.AetherClient({ rpcUrl: 'http://localhost:8899' });${colors.reset}`);
        console.log();
        console.log(`    ${colors.dim}3. Query the chain:${colors.reset}`);
        console.log(`       ${colors.cyan}const slot = await client.getSlot();${colors.reset}`);
        console.log(`       ${colors.cyan}console.log('Current slot:', slot);${colors.reset}`);
        console.log();
        console.log(`  ${colors.bright}Docs:${colors.reset} ${colors.blue}https://docs.aether.network/sdk/js${colors.reset}`);
        console.log();
        resolve({ success: true, targetDir, pkg: pkgToInstall });
      }
    });
    
    child.stdout?.on('data', (chunk) => {
      const line = chunk.toString().trim();
      if (line) process.stdout.write(`  ${colors.dim}${line}${colors.reset}\n`);
    });
    child.stderr?.on('data', (chunk) => {
      const line = chunk.toString().trim();
      if (line && !line.includes('npm warn')) process.stdout.write(`  ${colors.dim}${line}${colors.reset}\n`);
    });
  });
}

/**
 * Show TypeScript/Rust type definitions for Aether transactions
 */
function showTypes() {
  printSection('Aether Transaction Type Definitions', '🏷️');

  console.log(`  ${colors.bright}Exported from ${colors.cyan}@aether-network/client${colors.reset} and ${colors.cyan}aether-sdk${colors.reset}\n`);

  printSection('TransactionPayload (Rust enum — serde JSON tag)');

  const tsTypes = `// TypeScript / JavaScript
// Import from @aether-network/client

// TransactionPayload — discriminated union via 'type' field
type TransferPayload    = { type: 'Transfer';    data: { recipient: string; amount: u64; nonce: u64 } };
type StakePayload       = { type: 'Stake';       data: { validator: string; amount: u64 } };
type UnstakePayload     = { type: 'Unstake';    data: { stake_account: string; amount: u64 } };
type ClaimRewardsPayload = { type: 'ClaimRewards'; data: { stake_account: string } };
type CreateNFTPayload   = { type: 'CreateNFT';  data: { metadata_url: string; royalties: u16 } };
type MintNFTPayload     = { type: 'MintNFT';    data: { nft_id: string; amount: u64 } };
type TransferNFTPayload = { type: 'TransferNFT'; data: { nft_id: string; recipient: string } };
type UpdateMetadataPayload = { type: 'UpdateMetadata'; data: { nft_id: string; metadata_url: string } };

type TransactionPayload =
  | TransferPayload | StakePayload | UnstakePayload | ClaimRewardsPayload
  | CreateNFTPayload | MintNFTPayload | TransferNFTPayload | UpdateMetadataPayload;

// Full AetherTransaction
interface AetherTransaction {
  signature: string;   // base58 of [u8; 64]
  signer: string;       // base58 of [u8; 32]
  tx_type: string;     // e.g. "Transfer", "Stake"
  payload: TransactionPayload;
  fee: u64;
  slot: u64;
  timestamp: u64;
}

// Account response from GET /v1/account/<addr>
interface Account {
  lamports: u64;
  owner: string;        // base58 of [u8; 32]
  data: Uint8Array;
  rent_epoch: u64;
}`;

  console.log(`  ${colors.dim}[ typescript ]${colors.reset}`);
  tsTypes.split('\n').forEach(line => {
    console.log(`  ${line}`);
  });
  console.log();

  printSection('Rust struct definitions (from crates/aether-core/src/types.rs)');

  const rustTypes = `// Rust — use aether_sdk::types;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionPayload {
    #[serde(tag = "type", content = "data")]
    Transfer { recipient: String, amount: u64, nonce: u64 },
    #[serde(tag = "type", content = "data")]
    Stake { validator: String, amount: u64 },
    #[serde(tag = "type", content = "data")]
    Unstake { stake_account: String, amount: u64 },
    #[serde(tag = "type", content = "data")]
    ClaimRewards { stake_account: String },
    #[serde(tag = "type", content = "data")]
    CreateNFT { metadata_url: String, royalties: u16 },
    #[serde(tag = "type", content = "data")]
    MintNFT { nft_id: String, amount: u64 },
    #[serde(tag = "type", content = "data")]
    TransferNFT { nft_id: String, recipient: String },
    #[serde(tag = "type", content = "data")]
    UpdateMetadata { nft_id: String, metadata_url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherTransaction {
    #[serde(with = "serde_bytes_64")]
    pub signature: [u8; 64],
    #[serde(with = "serde_bytes_32")]
    pub signer: [u8; 32],
    pub tx_type: TransactionType,
    pub payload: TransactionPayload,
    pub fee: u64,
    pub slot: u64,
    pub timestamp: u64,
}

pub type Address = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub lamports: u64,
    pub owner: [u8; 32],
    pub data: Vec<u8>,
    pub rent_epoch: u64,
}`;

  console.log(`  ${colors.dim}[ rust ]${colors.reset}`);
  rustTypes.split('\n').forEach(line => {
    console.log(`  ${line}`);
  });
  console.log();

  printSection('TransactionType enum');
  console.log(`  ${colors.cyan}Transfer${colors.reset}       — Send AETH`);
  console.log(`  ${colors.cyan}Stake${colors.reset}          — Delegate to validator`);
  console.log(`  ${colors.cyan}Unstake${colors.reset}        — Request withdrawal`);
  console.log(`  ${colors.cyan}ClaimRewards${colors.reset}  — Claim staking rewards`);
  console.log(`  ${colors.cyan}CreateNFT${colors.reset}     — Create on-chain NFT`);
  console.log(`  ${colors.cyan}MintNFT${colors.reset}       — Mint additional NFT supply`);
  console.log(`  ${colors.cyan}TransferNFT${colors.reset}   — Transfer NFT to another address`);
  console.log(`  ${colors.cyan}UpdateMetadata${colors.reset} — Update NFT metadata URL`);
  console.log();
}

/**
 * Show documentation portal info
 */
function showDocs() {
  printSection('Aether Documentation', '📚');
  
  console.log(`  ${colors.bright}Comprehensive documentation for Aether developers.${colors.reset}\n`);
  
  console.log(`  ${colors.cyan}📖 Documentation Portal:${colors.reset}`);
  console.log(`     ${colors.blue}https://docs.aether.network${colors.reset}`);
  console.log();
  
  console.log(`  ${colors.cyan}Sections:${colors.reset}`);
  console.log(`    • Getting Started      - Quick start guides`);
  console.log(`    • Core Concepts        - Accounts, programs, transactions`);
  console.log(`    • SDK Reference        - Full API docs for JS and Rust`);
  console.log(`    • Tutorials            - Step-by-step projects`);
  console.log(`    • Validator Guide      - Running and maintaining validators`);
  console.log(`    • Economics            - Staking, rewards, fees`);
  console.log();
  
  printLink('Main Docs', 'https://docs.aether.network');
  printLink('API Reference', 'https://docs.aether.network/api');
  printLink('Tutorials', 'https://docs.aether.network/tutorials');
  printLink('Validator Docs', 'https://docs.aether.network/validators');
}

/**
 * Parse command line args
 */
function parseArgs() {
  const args = process.argv.slice(3); // Skip 'aether-cli sdk'
  
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    return 'all';
  }
  
  const subcmd = args[0].toLowerCase();
  
  switch (subcmd) {
    case 'js':
    case 'javascript':
    case 'node':
      return 'js';
    case 'rust':
    case 'rs':
      return 'rust';
    case 'tokens':
    case 'token':
    case 'flux':
    case 'ath':
      return 'tokens';
    case 'docs':
    case 'doc':
    case 'documentation':
      return 'docs';
    case 'types':
    case 'type':
    case 'typedef':
    case 'typedefs':
      return 'types';
    case 'install':
    case 'i':
      return 'install';
    default:
      return 'all';
  }
}

/**
 * Main SDK command
 */
async function sdkCommand() {
  const subcmd = parseArgs();
  
  switch (subcmd) {
    case 'js':
      showJsSdk();
      break;
    case 'rust':
      showRustSdk();
      break;
    case 'tokens':
      showTokensSdk();
      break;
    case 'docs':
      showDocs();
      break;
    case 'types':
      showTypes();
      break;
    case 'install':
      await runInstall(process.argv.slice(4));
      break;
    default:
      showAllSdks();
  }
}

// Export for use as module
module.exports = { sdkCommand };

// Run if called directly
if (require.main === module) {
  sdkCommand();
}
