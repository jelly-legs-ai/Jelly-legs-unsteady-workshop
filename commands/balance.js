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
const fs = require('fs');

// Import error handling utilities
const { 
  withErrorHandling, 
  displayError, 
  validateAddress,
  createRpcCaller,
  C 
} = require('../lib/errors');

// Import SDK — REAL blockchain RPC calls to http://127.0.0.1:8899
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

const CLI_VERSION = '1.5.0';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || (aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899');
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function lamportsToAeth(lamports) {
  return (Number(lamports) / 1e9).toFixed(6);
}

function loadConfig() {
  const aetherDir = path.join(os.homedir(), '.aether');
  const cfgPath = path.join(aetherDir, 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try { 
    return JSON.parse(fs.readFileSync(cfgPath, 'utf8')); 
  }
  catch { 
    return { defaultWallet: null }; 
  }
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

  return options;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli balance${C.reset} - Query account balance

${C.bright}Usage:${C.reset}
  aether balance [address] [options]

${C.bright}Options:${C.reset}
  -r, --rpc <url>     RPC endpoint (default: ${getDefaultRpc()})
  -j, --json         Output as JSON
  -l, --lamports     Show balance in lamports instead of AETH
  -h, --help         Show this help message

${C.bright}Examples:${C.reset}
  aether balance 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZvJkVG6PkD
  aether balance --json
  aether balance --lamports 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZvJkVG6PkD
`.trim());
}

// ---------------------------------------------------------------------------
// Balance fetching with retry logic
// ---------------------------------------------------------------------------

async function fetchBalance(client, address, options) {
  const { asJson, showLamports } = options;
  
  // Validate address format
  try {
    validateAddress(address);
  } catch (error) {
    error.isValidationError = true;
    throw error;
  }

  // Create RPC caller with retry logic
  const getBalance = createRpcCaller(client.getBalance.bind(client), {
    maxRetries: 3,
    initialDelay: 500,
    onRetry: (error, attempt, max) => {
      if (!asJson) {
        console.log(
          `${C.yellow}⚠ Retrying balance fetch (attempt ${attempt}/${max})...${C.reset}`
        );
      }
    },
  });

  // Fetch balance with retry
  const lamports = await getBalance(address);
  
  // Handle response
  if (lamports === undefined || lamports === null) {
    throw new Error('Received invalid response from RPC');
  }

  const balanceBigInt = BigInt(lamports);
  const aeth = lamportsToAeth(balanceBigInt);

  if (asJson) {
    console.log(JSON.stringify({
      address,
      lamports: balanceBigInt.toString(),
      aeth: parseFloat(aeth),
      rpc: options.rpc,
      timestamp: new Date().toISOString(),
    }, null, 2));
  } else {
    const shortAddr = shortenAddress(address);
    console.log();
    console.log(`${C.bright}Account Balance${C.reset}`);
    console.log(`${C.dim}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C.reset}`);
    console.log(`${C.cyan}Address:${C.reset}  ${C.bright}${address}${C.reset}`);
    console.log();
    
    if (showLamports) {
      console.log(`${C.green}${C.bright}${balanceBigInt.toString()}${C.reset} ${C.dim}lamports${C.reset}`);
    } else {
      console.log(`${C.green}${C.bright}${aeth}${C.reset} ${C.dim}AETH${C.reset}`);
      console.log(`${C.dim}(${balanceBigInt.toString()} lamports)${C.reset}`);
    }
    
    console.log();
    console.log(`${C.dim}RPC: ${options.rpc}${C.reset}`);
    console.log(`${C.dim}Time: ${new Date().toLocaleString()}${C.reset}`);
    console.log();
  }

  return { address, lamports: balanceBigInt, aeth: parseFloat(aeth) };
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function balanceCommand() {
  const options = parseArgs();
  let address = options.address;

  // If no address provided, try to use default wallet
  if (!address) {
    const config = loadConfig();
    if (config.defaultWallet) {
      address = config.defaultWallet;
      if (!options.asJson) {
        console.log(`${C.dim}Using default wallet: ${shortenAddress(address)}${C.reset}`);
      }
    } else {
      const error = new Error('No address provided and no default wallet configured');
      error.isValidationError = true;
      throw error;
    }
  }

  const client = createClient(options.rpc);
  
  try {
    await fetchBalance(client, address, options);
  } catch (error) {
    // Add context to error
    if (!error.context) {
      error.context = { address, rpc: options.rpc };
    }
    throw error;
  }
}

// Run with error handling
if (require.main === module) {
  withErrorHandling(balanceCommand, { exit: true, verbose: process.env.AETHER_VERBOSE === '1' })();
}

module.exports = { 
  balanceCommand,
  parseArgs,
  fetchBalance,
  loadConfig,
};
