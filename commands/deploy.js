#!/usr/bin/env node
/**
 * aether-cli deploy
 *
 * Deploy smart contracts/programs to the Aether blockchain.
 * Fully wired to @jellylegsai/aether-sdk for real blockchain RPC calls.
 *
 * Usage:
 *   aether deploy <contract.wasm>                    Deploy a WASM contract
 *   aether deploy <program.so> --type bpf            Deploy a BPF program
 *   aether deploy <contract.wasm> --name <name>      Deploy with custom name
 *   aether deploy <contract.wasm> --upgradeable       Deploy as upgradeable contract
 *   aether deploy --list-templates                   Show available contract templates
 *   aether deploy --verify <address>                 Verify deployed contract
 *   aether deploy --status <address>                 Check deployment status
 *
 * SDK wired to:
 *   - client.sendTransaction(tx)        → POST /v1/transaction (Deploy)
 *   - client.getAccountInfo(addr)     → GET /v1/account/<addr>
 *   - client.getProgram(programId)    → GET /v1/program/<id>
 *   - client.getSlot()                → GET /v1/slot
 *   - client.getRecentBlockhash()     → GET /v1/recent-blockhash
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');
const crypto = require('crypto');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const bip39 = require('bip39');

// Import SDK for ALL blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// Import UI framework for consistent branding
const { BRANDING, C, indicators, success, error, warning, info, code, key, value, startSpinner, stopSpinner, formatHelp, drawBox } = require('../lib/ui');

const CLI_VERSION = '2.0.0';
const DERIVATION_PATH = "m/44'/7777777'/0'/0'";

// Deployment constants
const DEPLOYMENT_FEE_LAMPORTS = 50000; // 0.00005 AETH base fee
const MIN_RENT_EXEMPTION_LAMPORTS = 890880; // ~0.00089 AETH for rent exemption
const MAX_CONTRACT_SIZE = 10 * 1024 * 1024; // 10MB max

// Contract templates
const CONTRACT_TEMPLATES = {
  token: {
    name: 'SPL Token Contract',
    description: 'Standard token contract with mint/burn/transfer',
    minSize: 10000,
    maxSize: 50000,
  },
  nft: {
    name: 'NFT Collection Contract',
    description: 'NFT minting and management contract',
    minSize: 15000,
    maxSize: 100000,
  },
  staking: {
    name: 'Staking Pool Contract',
    description: 'Stake tokens and earn rewards',
    minSize: 20000,
    maxSize: 150000,
  },
  governance: {
    name: 'Governance Contract',
    description: 'On-chain voting and proposals',
    minSize: 25000,
    maxSize: 200000,
  },
  multisig: {
    name: 'Multi-Signature Wallet',
    description: 'Requires N-of-M signatures for transactions',
    minSize: 18000,
    maxSize: 80000,
  },
  custom: {
    name: 'Custom Contract',
    description: 'Your own compiled WASM/BPF program',
    minSize: 1000,
    maxSize: MAX_CONTRACT_SIZE,
  },
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
    return { defaultWallet: null, deployments: [] };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { defaultWallet: null, deployments: [] };
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

function saveDeployment(deployment) {
  const cfg = loadConfig();
  if (!cfg.deployments) cfg.deployments = [];
  cfg.deployments.push({
    ...deployment,
    deployedAt: new Date().toISOString(),
  });
  saveConfig(cfg);
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

function generateProgramId() {
  const keyPair = nacl.sign.keyPair();
  return bs58.encode(Buffer.from(keyPair.publicKey));
}

// ============================================================================
// Format Helpers
// ============================================================================

function formatAether(lamports) {
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(9).replace(/\.?0+$/, '') + ' AETH';
}

function formatBytes(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

function shortAddress(addr) {
  if (!addr || addr.length < 16) return addr || 'unknown';
  return addr.slice(0, 8) + '…' + addr.slice(-8);
}

function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
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
// Contract Validation
// ============================================================================

function validateContractFile(filePath) {
  const errors = [];
  const warnings = [];

  // Check file exists
  if (!fs.existsSync(filePath)) {
    errors.push(`File not found: ${filePath}`);
    return { valid: false, errors, warnings };
  }

  // Check it's a file
  const stats = fs.statSync(filePath);
  if (!stats.isFile()) {
    errors.push(`Path is not a file: ${filePath}`);
    return { valid: false, errors, warnings };
  }

  // Check size
  const size = stats.size;
  if (size === 0) {
    errors.push('Contract file is empty');
  } else if (size > MAX_CONTRACT_SIZE) {
    errors.push(`Contract size ${formatBytes(size)} exceeds maximum ${formatBytes(MAX_CONTRACT_SIZE)}`);
  }

  // Check extension
  const ext = path.extname(filePath).toLowerCase();
  const validExts = ['.wasm', '.so', '.bin'];
  if (!validExts.includes(ext)) {
    warnings.push(`Unusual extension "${ext}". Expected: .wasm, .so, or .bin`);
  }

  // Try to read and validate basic structure
  try {
    const buffer = fs.readFileSync(filePath);
    
    // Check WASM magic number for .wasm files
    if (ext === '.wasm' || buffer.slice(0, 4).toString('hex') === '0061736d') {
      const wasmMagic = buffer.slice(0, 4);
      if (wasmMagic.toString('hex') !== '0061736d') {
        warnings.push('File does not have standard WASM magic number');
      }
    }

    // Calculate hash
    const hash = crypto.createHash('sha256').update(buffer).digest('hex');

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      size,
      formattedSize: formatBytes(size),
      hash: hash.slice(0, 16) + '...',
      fullHash: hash,
    };
  } catch (e) {
    errors.push(`Failed to read file: ${e.message}`);
    return { valid: false, errors, warnings };
  }
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    filePath: null,
    contractType: 'custom',
    name: null,
    upgradeable: false,
    rpc: getDefaultRpc(),
    json: false,
    dryRun: false,
    force: false,
    wallet: null,
    listTemplates: false,
    verify: null,
    status: null,
    help: false,
    programId: null,
    rentExempt: true,
    computeUnits: 200000,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--type' || arg === '-t') {
      opts.contractType = (args[++i] || 'custom').toLowerCase();
    } else if (arg === '--name' || arg === '-n') {
      opts.name = args[++i];
    } else if (arg === '--upgradeable' || arg === '-u') {
      opts.upgradeable = true;
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--dry-run') {
      opts.dryRun = true;
    } else if (arg === '--force' || arg === '-f') {
      opts.force = true;
    } else if (arg === '--wallet' || arg === '-w') {
      opts.wallet = args[++i];
    } else if (arg === '--list-templates' || arg === '-l') {
      opts.listTemplates = true;
    } else if (arg === '--verify' || arg === '-v') {
      opts.verify = args[++i];
    } else if (arg === '--status' || arg === '-s') {
      opts.status = args[++i];
    } else if (arg === '--program-id' || arg === '-p') {
      opts.programId = args[++i];
    } else if (arg === '--no-rent-exempt') {
      opts.rentExempt = false;
    } else if (arg === '--compute-units' || arg === '-c') {
      const cu = parseInt(args[++i], 10);
      if (!isNaN(cu) && cu > 0) opts.computeUnits = cu;
    } else if (arg === '--help' || arg === '-h') {
      opts.help = true;
    } else if (!arg.startsWith('-') && !opts.filePath) {
      opts.filePath = arg;
    }
  }

  return opts;
}

function showHelp() {
  console.log(BRANDING.header(CLI_VERSION));
  console.log(`
${C.bright}${C.cyan}aether-cli deploy${C.reset} — Deploy smart contracts to Aether blockchain

${C.bright}USAGE${C.reset}
    aether deploy <contract.wasm> [options]
    aether deploy --list-templates
    aether deploy --verify <program-id>
    aether deploy --status <program-id>

${C.bright}COMMANDS${C.reset}
    deploy <file>          Deploy contract from file
    --list-templates       Show available contract templates
    --verify <address>     Verify deployed contract on-chain
    --status <address>     Check deployment/upgrade status

${C.bright}OPTIONS${C.reset}
    -t, --type <type>      Contract type: token, nft, staking, governance, multisig, custom
    -n, --name <name>      Contract name for identification
    -u, --upgradeable      Deploy as upgradeable contract
    -w, --wallet <addr>    Deployer wallet address
    -r, --rpc <url>        RPC endpoint
    -c, --compute-units    Max compute units (default: 200000)
    --no-rent-exempt       Skip rent exemption (not recommended)
    --dry-run              Preview deployment without submitting
    --force                Skip confirmation prompts
    -j, --json             JSON output for scripting
    -h, --help             Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.sendTransaction()      → POST /v1/transaction
    client.getAccountInfo()       → GET /v1/account/<addr>
    client.getSlot()              → GET /v1/slot
    client.getRecentBlockhash()   → GET /v1/recent-blockhash

${C.bright}EXAMPLES${C.reset}
    aether deploy token_contract.wasm --name "MyToken" --type token
    aether deploy program.so --type bpf --upgradeable --wallet ATHxxx...
    aether deploy --list-templates
    aether deploy --verify ATHProgxxx...
    aether deploy --status ATHProgxxx...
`);
}

// ============================================================================
// List Templates
// ============================================================================

function listTemplates(asJson) {
  if (asJson) {
    console.log(JSON.stringify({
      templates: Object.entries(CONTRACT_TEMPLATES).map(([id, t]) => ({
        id,
        name: t.name,
        description: t.description,
        minSize: t.minSize,
        maxSize: t.maxSize,
      })),
    }, null, 2));
    return;
  }

  console.log(`\n${C.bright}${C.cyan}═══ Available Contract Templates ═══${C.reset}\n`);

  for (const [id, template] of Object.entries(CONTRACT_TEMPLATES)) {
    const boxContent = `
${C.bright}${template.name}${C.reset}
${C.dim}${template.description}${C.reset}

${C.cyan}Size:${C.reset} ${formatBytes(template.minSize)} - ${formatBytes(template.maxSize)}
${C.cyan}Usage:${C.reset} aether deploy <file> --type ${id}`;
    
    console.log(drawBox(boxContent, { 
      style: 'rounded', 
      width: 60,
      borderColor: C.dim,
    }));
    console.log();
  }

  console.log(`${C.dim}Tip: Use --type custom for your own compiled contracts${C.reset}\n`);
}

// ============================================================================
// Verify Contract
// ============================================================================

async function verifyContract(opts) {
  const programId = opts.verify;
  const rpcUrl = opts.rpc;
  
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}═══ Verify Contract ═══${C.reset}\n`);
    console.log(`  ${C.dim}Program ID:${C.reset} ${programId}`);
    console.log(`  ${C.dim}RPC:${C.reset} ${rpcUrl}\n`);
    startSpinner('Fetching on-chain data');
  }

  const client = createClient(rpcUrl);
  
  try {
    // Try to get program account info
    const rawProgramId = programId.startsWith('ATH') ? programId.slice(3) : programId;
    const account = await client.getAccountInfo(rawProgramId);
    
    stopSpinner(true);

    if (!account || account.error) {
      if (opts.json) {
        console.log(JSON.stringify({
          programId,
          verified: false,
          error: 'Program not found on-chain',
        }, null, 2));
      } else {
        console.log(`\n  ${indicators.error} ${error('Program not found on-chain')}\n`);
        console.log(`  ${C.dim}The program ID may be incorrect or the deployment failed.${C.reset}\n`);
      }
      return false;
    }

    // Program exists - verify it's executable
    const isExecutable = account.executable === true || account.owner === 'BPFLoader2111111111111111111111111111111111';
    const deployTime = account.rent_epoch ? `Epoch ${account.rent_epoch}` : 'Unknown';
    
    if (opts.json) {
      console.log(JSON.stringify({
        programId,
        verified: true,
        executable: isExecutable,
        owner: account.owner,
        lamports: account.lamports,
        dataSize: account.data ? account.data.length : 0,
        rentEpoch: account.rent_epoch,
        deployTime,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.success} ${success('Contract verified on-chain')}\n`);
      console.log(`  ${C.bright}Program ID:${C.reset}  ${C.cyan}${programId}${C.reset}`);
      console.log(`  ${C.bright}Status:${C.reset}      ${isExecutable ? C.green + 'Executable ✓' : C.yellow + 'Not Executable'}${C.reset}`);
      console.log(`  ${C.bright}Balance:${C.reset}     ${formatAether(account.lamports || 0)}`);
      console.log(`  ${C.bright}Data Size:${C.reset}   ${formatBytes(account.data ? account.data.length : 0)}`);
      console.log(`  ${C.bright}Owner:${C.reset}       ${shortAddress(account.owner)}`);
      console.log(`  ${C.bright}Deployed:${C.reset}    ${deployTime}`);
      console.log();
    }
    
    return true;
  } catch (err) {
    stopSpinner(false);
    if (opts.json) {
      console.log(JSON.stringify({
        programId,
        verified: false,
        error: err.message,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error(`Verification failed: ${err.message}`)}\n`);
    }
    return false;
  }
}

// ============================================================================
// Check Deployment Status
// ============================================================================

async function checkDeploymentStatus(opts) {
  const programId = opts.status;
  const rpcUrl = opts.rpc;
  
  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}═══ Deployment Status ═══${C.reset}\n`);
    console.log(`  ${C.dim}Program ID:${C.reset} ${programId}`);
    console.log(`  ${C.dim}RPC:${C.reset} ${rpcUrl}\n`);
  }

  const client = createClient(rpcUrl);
  
  try {
    const rawProgramId = programId.startsWith('ATH') ? programId.slice(3) : programId;
    
    // Get multiple data points
    const [account, slot, health] = await Promise.all([
      client.getAccountInfo(rawProgramId).catch(() => null),
      client.getSlot().catch(() => null),
      client.getHealth().catch(() => 'unknown'),
    ]);

    const status = {
      programId,
      exists: !!account,
      executable: account?.executable || false,
      currentSlot: slot,
      nodeHealth: health,
      timestamp: new Date().toISOString(),
    };

    if (opts.json) {
      console.log(JSON.stringify(status, null, 2));
    } else {
      console.log(`  ${C.bright}Program ID:${C.reset}    ${C.cyan}${programId}${C.reset}`);
      console.log(`  ${C.bright}Status:${C.reset}        ${status.exists ? C.green + '✓ Deployed' : C.red + '✗ Not Found'}${C.reset}`);
      console.log(`  ${C.bright}Executable:${C.reset}    ${status.executable ? C.green + 'Yes' : C.yellow + 'No'}${C.reset}`);
      console.log(`  ${C.bright}Current Slot:${C.reset}  ${slot || 'N/A'}`);
      console.log(`  ${C.bright}Node Health:${C.reset}   ${health}${C.reset}`);
      console.log();
    }
    
    return status.exists;
  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        programId,
        error: err.message,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error(`Status check failed: ${err.message}`)}\n`);
    }
    return false;
  }
}

// ============================================================================
// Core Deployment Logic
// ============================================================================

async function deployContract(opts) {
  const startTime = Date.now();
  const rl = createRl();

  // Header
  if (!opts.json) {
    console.log(BRANDING.header(CLI_VERSION));
    console.log(`\n${C.bright}${C.cyan}═══ Contract Deployment ═══${C.reset}\n`);
  }

  // Validate contract file
  if (!opts.json) {
    console.log(`  ${C.dim}Validating contract file...${C.reset}`);
  }
  
  const validation = validateContractFile(opts.filePath);
  if (!validation.valid) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        stage: 'validation',
        errors: validation.errors,
        warnings: validation.warnings,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error('Validation failed')}\n`);
      validation.errors.forEach(e => console.log(`    ${C.red}•${C.reset} ${e}`));
      if (validation.warnings.length > 0) {
        console.log();
        validation.warnings.forEach(w => console.log(`    ${C.yellow}⚠${C.reset} ${w}`));
      }
      console.log();
    }
    rl.close();
    process.exit(1);
  }

  if (!opts.json) {
    console.log(`  ${indicators.success} ${success('File validated')}`);
    console.log(`  ${C.dim}  Size: ${validation.formattedSize}${C.reset}`);
    console.log(`  ${C.dim}  Hash: ${validation.hash}${C.reset}\n`);
  }

  // Resolve wallet
  let walletAddress = opts.wallet;
  if (!walletAddress) {
    const cfg = loadConfig();
    walletAddress = cfg.defaultWallet;
  }

  if (!walletAddress) {
    if (opts.json) {
      console.log(JSON.stringify({ success: false, error: 'No wallet specified' }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error('No wallet address')} — Use --wallet or set a default\n`);
    }
    rl.close();
    process.exit(1);
  }

  const wallet = loadWallet(walletAddress);
  if (!wallet) {
    if (opts.json) {
      console.log(JSON.stringify({ success: false, error: 'Wallet not found' }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error(`Wallet not found: ${walletAddress}`)}\n`);
    }
    rl.close();
    process.exit(1);
  }

  // Initialize SDK client
  const client = createClient(opts.rpc);

  // Check balance
  if (!opts.json) {
    console.log(`  ${C.dim}Checking wallet balance...${C.reset}`);
  }

  let balance = 0;
  try {
    const rawAddr = walletAddress.startsWith('ATH') ? walletAddress.slice(3) : walletAddress;
    balance = await client.getBalance(rawAddr);
  } catch (e) {
    // Continue with balance 0
  }

  const minRequired = DEPLOYMENT_FEE_LAMPORTS + (opts.rentExempt ? MIN_RENT_EXEMPTION_LAMPORTS : 0) + validation.size;
  
  if (balance < minRequired) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Insufficient balance',
        required: minRequired,
        available: balance,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error('Insufficient balance')}\n`);
      console.log(`  Required:  ${formatAether(minRequired)}`);
      console.log(`  Available: ${formatAether(balance)}\n`);
    }
    rl.close();
    process.exit(1);
  }

  if (!opts.json) {
    console.log(`  ${indicators.success} ${success('Balance sufficient')}: ${formatAether(balance)}\n`);
  }

  // Get network state
  if (!opts.json) {
    console.log(`  ${C.dim}Fetching network state...${C.reset}`);
  }

  const [slot, blockhash, epoch] = await Promise.all([
    client.getSlot().catch(() => 0),
    client.getRecentBlockhash().catch(() => ({ blockhash: '11111111111111111111111111111111' })),
    client.getEpochInfo().catch(() => ({ epoch: 0 })),
  ]);

  if (!opts.json) {
    console.log(`  ${indicators.success} ${success('Network ready')}`);
    console.log(`  ${C.dim}  Slot: ${slot}${C.reset}`);
    console.log(`  ${C.dim}  Epoch: ${epoch.epoch}${C.reset}\n`);
  }

  // Generate or use provided program ID
  const programId = opts.programId || generateProgramId();
  
  // Read contract bytecode
  const contractBytes = fs.readFileSync(opts.filePath);
  const contractBase64 = contractBytes.toString('base64');

  // Deployment summary
  const deploymentName = opts.name || path.basename(opts.filePath, path.extname(opts.filePath));
  
  if (!opts.json) {
    console.log(`${C.bright}${C.cyan}── Deployment Summary ──${C.reset}\n`);
    console.log(`  ${C.bright}Contract:${C.reset}    ${C.cyan}${deploymentName}${C.reset}`);
    console.log(`  ${C.bright}Type:${C.reset}        ${opts.contractType}`);
    console.log(`  ${C.bright}Size:${C.reset}        ${validation.formattedSize}`);
    console.log(`  ${C.bright}Program ID:${C.reset}  ${shortAddress(programId)}`);
    console.log(`  ${C.bright}Deployer:${C.reset}    ${shortAddress(walletAddress)}`);
    console.log(`  ${C.bright}Upgradeable:${C.reset} ${opts.upgradeable ? C.green + 'Yes' : C.dim + 'No'}${C.reset}`);
    console.log(`  ${C.bright}Rent Exempt:${C.reset} ${opts.rentExempt ? C.green + 'Yes' : C.yellow + 'No'}${C.reset}`);
    console.log(`  ${C.bright}RPC:${C.reset}         ${opts.rpc}`);
    console.log(`  ${C.bright}Fee:${C.reset}         ${formatAether(DEPLOYMENT_FEE_LAMPORTS)}`);
    console.log();
  }

  // Dry run mode
  if (opts.dryRun) {
    if (opts.json) {
      console.log(JSON.stringify({
        dryRun: true,
        name: deploymentName,
        type: opts.contractType,
        size: validation.size,
        programId,
        deployer: walletAddress,
        upgradeable: opts.upgradeable,
        rentExempt: opts.rentExempt,
        fee: DEPLOYMENT_FEE_LAMPORTS,
        slot,
        epoch: epoch.epoch,
      }, null, 2));
    } else {
      console.log(`  ${C.yellow}⚠ Dry run mode — No transaction submitted${C.reset}\n`);
    }
    rl.close();
    return;
  }

  // Get mnemonic for signing
  if (!opts.json) {
    console.log(`${C.yellow}  Signing requires your wallet passphrase${C.reset}\n`);
  }

  let keyPair;
  try {
    const mnemonic = await askMnemonic(rl, 'Enter passphrase to sign deployment');
    keyPair = deriveKeypair(mnemonic);
    
    // Verify address
    const derivedAddr = formatAddress(keyPair.publicKey);
    if (derivedAddr !== walletAddress) {
      if (!opts.json) {
        console.log(`\n  ${indicators.error} ${error('Passphrase mismatch')}\n`);
        console.log(`  Expected: ${walletAddress}`);
        console.log(`  Derived:  ${derivedAddr}\n`);
      } else {
        console.log(JSON.stringify({ success: false, error: 'Passphrase mismatch' }, null, 2));
      }
      rl.close();
      process.exit(1);
    }
  } catch (e) {
    if (!opts.json) {
      console.log(`\n  ${indicators.error} ${error(`Failed: ${e.message}`)}\n`);
    } else {
      console.log(JSON.stringify({ success: false, error: e.message }, null, 2));
    }
    rl.close();
    process.exit(1);
  }

  // Confirm deployment
  if (!opts.json && !opts.force) {
    const confirm = await question(rl, `  ${C.yellow}Confirm deployment? [y/N]${C.reset} > `);
    if (!confirm.trim().toLowerCase().startsWith('y')) {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      rl.close();
      return;
    }
    console.log();
  }

  rl.close();

  // Build deployment transaction
  const rawWalletAddr = walletAddress.startsWith('ATH') ? walletAddress.slice(3) : walletAddress;
  
  const tx = {
    signer: rawWalletAddr,
    tx_type: 'Deploy',
    payload: {
      type: 'Deploy',
      data: {
        program_id: programId,
        bytecode: contractBase64,
        name: deploymentName,
        contract_type: opts.contractType,
        upgradeable: opts.upgradeable,
        rent_exempt: opts.rentExempt,
        compute_units: opts.computeUnits,
        bytecode_hash: validation.fullHash,
      },
    },
    fee: DEPLOYMENT_FEE_LAMPORTS,
    slot: slot,
    timestamp: Math.floor(Date.now() / 1000),
    recent_blockhash: blockhash.blockhash,
  };

  // Sign transaction
  tx.signature = signTransaction(tx, keyPair.secretKey);

  // Submit deployment
  if (!opts.json) {
    startSpinner('Submitting deployment transaction');
  }

  try {
    const result = await client.sendTransaction(tx);
    
    stopSpinner(true);
    
    const deployTime = Date.now() - startTime;
    
    // Save deployment record
    saveDeployment({
      programId,
      name: deploymentName,
      type: opts.contractType,
      size: validation.size,
      deployer: walletAddress,
      transaction: result.signature || result.txid,
      slot: result.slot || slot,
      upgradeable: opts.upgradeable,
    });

    if (opts.json) {
      console.log(JSON.stringify({
        success: true,
        programId,
        name: deploymentName,
        type: opts.contractType,
        size: validation.size,
        deployer: walletAddress,
        signature: result.signature || result.txid,
        slot: result.slot || slot,
        deployTimeMs: deployTime,
        rpc: opts.rpc,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.success} ${C.green}${C.bright}Contract deployed successfully!${C.reset}\n`);
      console.log(`  ${C.bright}Program ID:${C.reset}  ${C.cyan}${programId}${C.reset}`);
      console.log(`  ${C.bright}Name:${C.reset}        ${deploymentName}`);
      console.log(`  ${C.bright}Type:${C.reset}        ${opts.contractType}`);
      console.log(`  ${C.bright}Size:${C.reset}        ${validation.formattedSize}`);
      console.log(`  ${C.bright}Deploy Time:${C.reset} ${formatDuration(deployTime)}`);
      
      if (result.signature || result.txid) {
        console.log(`  ${C.bright}Signature:${C.reset}   ${shortAddress(result.signature || result.txid)}`);
      }
      console.log(`  ${C.bright}Slot:${C.reset}        ${result.slot || slot}`);
      console.log();
      console.log(`  ${C.dim}Verify:   aether deploy --verify ${programId}${C.reset}`);
      console.log(`  ${C.dim}Status:   aether deploy --status ${programId}${C.reset}`);
      console.log(`  ${C.dim}Explorer: https://explorer.aether.network/program/${programId}${C.reset}\n`);
    }

  } catch (err) {
    stopSpinner(false);
    
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        stage: 'deployment',
        error: err.message,
      }, null, 2));
    } else {
      console.log(`\n  ${indicators.error} ${error(`Deployment failed: ${err.message}`)}\n`);
      console.log(`  ${C.dim}Common causes:${C.reset}`);
      console.log(`    • Contract bytecode is invalid or corrupted`);
      console.log(`    • Insufficient balance for deployment fee`);
      console.log(`    • RPC node rejected the transaction`);
      console.log(`    • Network congestion - retry with higher fee\n`);
    }
    process.exit(1);
  }
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function deployCommand() {
  const opts = parseArgs();

  if (opts.help) {
    showHelp();
    return;
  }

  if (opts.listTemplates) {
    listTemplates(opts.json);
    return;
  }

  if (opts.verify) {
    await verifyContract(opts);
    return;
  }

  if (opts.status) {
    await checkDeploymentStatus(opts);
    return;
  }

  if (!opts.filePath) {
    console.log(BRANDING.header(CLI_VERSION));
    console.log(`\n  ${indicators.error} ${error('No contract file specified')}\n`);
    console.log(`  ${C.dim}Usage: aether deploy <contract.wasm> [options]${C.reset}`);
    console.log(`  ${C.dim}       aether deploy --help for more info${C.reset}\n`);
    process.exit(1);
  }

  await deployContract(opts);
}

module.exports = { deployCommand };

if (require.main === module) {
  deployCommand().catch(err => {
    console.error(`\n${C.red}✗ Deploy command failed: ${err.message}${C.reset}\n`);
    process.exit(1);
  });
}
