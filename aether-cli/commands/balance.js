#!/usr/bin/env node
/**
 * aether-cli balance
 *
 * Query account balance from the Aether blockchain — real HTTP RPC calls,
 * real data. No stubs, no mocks.
 *
 * Usage:
 *   aether balance <address>            Query balance for an address
 *   aether balance                      Query default wallet balance
 *   aether balance --json               JSON output for scripting
 *   aether balance --rpc <url>          Query a specific RPC endpoint
 *   aether balance --lamports           Show balance in lamports (not AETH)
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 * SDK: @jellylegsai/aether-sdk — makes REAL HTTP RPC calls to the chain
 */

const os = require('os');
const path = require('path');

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

// Import SDK — REAL blockchain RPC calls to http://127.0.0.1:8899
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

const DEFAULT_RPC = process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function lamportsToAeth(lamports) {
  return (Number(lamports) / 1e9).toFixed(6);
}

function loadConfig() {
  const fs = require('fs');
  const aetherDir = path.join(os.homedir(), '.aether');
  const cfgPath = path.join(aetherDir, 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try { return JSON.parse(fs.readFileSync(cfgPath, 'utf8')); }
  catch { return { defaultWallet: null }; }
}

function shortenAddress(addr) {
  if (!addr) return 'unknown';
  if (addr.length <= 10) return addr;
  return `${addr.substring(0, 6)}...${addr.substring(addr.length - 4)}`;
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: getDefaultRpc(),
    address: null,
    asJson: false,
    showLamports: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--rpc' || arg === '-r') {
      options.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      options.asJson = true;
    } else if (arg === '--lamports' || arg === '-l') {
      options.showLamports = true;
    } else if (arg === '--help' || arg === '-h') {
      showHelp();
      process.exit(0);
    } else if (!arg.startsWith('-') && !options.address) {
      options.address = arg;
    }
  }

  // If no address provided, try to load default wallet
  if (!options.address) {
    const config = loadConfig();
    if (config.defaultWallet) {
      options.address = config.defaultWallet;
    }
  }

  return options;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli balance${C.reset} - Query Account Balance

${C.bright}Usage:${C.reset}
  aether balance [address] [options]

${C.bright}Arguments:${C.reset}
  address         Account public key (base58). If omitted, uses default wallet.

${C.bright}Options:${C.reset}
  --rpc <url>     Query a specific RPC endpoint (default: ${DEFAULT_RPC})
  --json, -j      Output raw JSON for scripting
  --lamports, -l  Show balance in lamports (not AETH)
  --help, -h      Show this help message

${C.bright}Description:${C.reset}
  Queries the Aether blockchain for an account's balance.
  Every call makes a REAL HTTP RPC request to the configured
  Aether node — no caching, no stubs, no mocks.

  RPC Endpoint: GET /v1/account/<address>

${C.bright}Examples:${C.reset}
  aether balance                           # Default wallet
  aether balance 8xPt...3nQ                  # Specific address
  aether balance --json                    # JSON output
  aether balance --lamports                # Show lamports
  AETHER_RPC=https://my-node:8899 aether balance
`);
}

// ---------------------------------------------------------------------------
// Balance query - REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchBalance(rpc, address) {
  const client = createClient(rpc);
  
  // Real RPC call: GET /v1/account/<address>
  const account = await client.getAccountInfo(address);
  
  return {
    address,
    lamports: account.lamports !== undefined ? account.lamports : 0,
    owner: account.owner || null,
    executable: account.executable || false,
    rentEpoch: account.rent_epoch !== undefined ? account.rent_epoch : null,
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printBalance(data, options) {
  const { address, lamports, owner, executable, rentEpoch } = data;
  const { showLamports, rpc } = options;

  const aeth = lamportsToAeth(lamports);

  console.log(`\n${C.bright}${C.cyan}── Aether Account Balance ───────────────────────────────${C.reset}\n`);
  console.log(`  ${C.bright}Address:${C.reset}    ${C.magenta}${address}${C.reset}`);
  
  if (showLamports) {
    console.log(`  ${C.bright}Balance:${C.reset}    ${C.green}${lamports.toLocaleString()} lamports${C.reset}`);
  } else {
    console.log(`  ${C.bright}Balance:${C.reset}    ${C.green}${aeth} AETH${C.reset}`);
    console.log(`  ${C.dim}             (${lamports.toLocaleString()} lamports)${C.reset}`);
  }
  
  if (owner) {
    console.log(`  ${C.bright}Owner:${C.reset}      ${C.dim}${shortenAddress(owner)}${C.reset}`);
  }
  
  if (executable !== undefined) {
    const execStr = executable ? `${C.yellow}yes${C.reset}` : `${C.dim}no${C.reset}`;
    console.log(`  ${C.bright}Executable:${C.reset} ${execStr}`);
  }
  
  if (rentEpoch !== null) {
    console.log(`  ${C.bright}Rent Epoch:${C.reset} ${C.dim}${rentEpoch}${C.reset}`);
  }
  
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}\n`);
}

function printJson(data, rpc) {
  console.log(JSON.stringify({
    address: data.address,
    lamports: data.lamports,
    aeth: lamportsToAeth(data.lamports),
    owner: data.owner,
    executable: data.executable,
    rentEpoch: data.rentEpoch,
    rpc,
    cli_version: CLI_VERSION,
    timestamp: new Date().toISOString(),
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function balanceCommand() {
  const options = parseArgs();
  const { rpc, address, asJson } = options;

  // Validate address
  if (!address) {
    if (asJson) {
      console.log(JSON.stringify({
        error: 'No address provided and no default wallet configured',
        hint: 'Run "aether init" to set up a wallet, or provide an address',
      }));
    } else {
      console.log(`\n${C.red}✗ No address provided${C.reset}`);
      console.log(`  ${C.dim}Provide an address or run "aether init" to set up a default wallet.${C.reset}`);
      console.log(`  ${C.dim}Usage: aether balance <address>${C.reset}\n`);
    }
    process.exit(1);
  }

  if (!asJson) {
    console.log(`${C.dim}Querying balance for ${C.cyan}${shortenAddress(address)}${C.dim}...${C.reset}`);
  }

  try {
    // Real blockchain RPC call
    const data = await fetchBalance(rpc, address);
    
    if (asJson) {
      printJson(data, rpc);
    } else {
      printBalance(data, options);
    }
  } catch (err) {
    if (asJson) {
      console.log(JSON.stringify({
        error: err.message,
        address,
        rpc,
      }));
    } else {
      console.log(`\n${C.red}✗ Failed to fetch balance: ${err.message}${C.reset}`);
      console.log(`  ${C.dim}Address: ${address}${C.reset}`);
      console.log(`  ${C.dim}RPC: ${rpc}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { balanceCommand };

if (require.main === module) {
  balanceCommand().catch(err => {
    console.error(`${C.red}✗ Balance command failed: ${err.message}${C.reset}`);
    process.exit(1);
  });
}
