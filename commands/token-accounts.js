#!/usr/bin/env node
/**
 * aether-cli token-accounts
 *
 * Get SPL token accounts for a wallet address.
 * Uses @jellylegsai/aether-sdk for REAL HTTP RPC calls.
 *
 * Usage:
 *   aether token-accounts <address>              List token accounts
 *   aether token-accounts --address <addr>      List token accounts
 *   aether token-accounts --json                JSON output
 *   aether token-accounts --rpc <url>           Custom RPC endpoint
 *
 * RPC Endpoint: GET /v1/tokens/<address>
 * SDK Function: sdk.getTokenAccounts()
 */

const path = require('path');
const fs = require('fs');
const os = require('os');

// Import UI framework for consistent branding
const { BRANDING, C, indicators, startSpinner, stopSpinner, drawBox, drawTable,
        success, error, warning, info, code, highlight, key, value } = require('../lib/ui');

const CLI_VERSION = '1.0.0';

// Import SDK — makes REAL HTTP RPC calls to the blockchain
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function loadConfig() {
  const aetherDir = path.join(os.homedir(), '.aether');
  const cfgPath = path.join(aetherDir, 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function shortenAddress(addr) {
  if (!addr) return 'unknown';
  if (addr.length <= 16) return addr;
  return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
}

function formatTokenAmount(amount, decimals = 9) {
  const val = Number(amount) / Math.pow(10, decimals);
  return val.toFixed(decimals > 6 ? 6 : decimals).replace(/\.?0+$/, '');
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
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--json' || args[i] === '-j') {
      options.asJson = true;
    } else if (args[i] === '--rpc' || args[i] === '-r') {
      options.rpc = args[++i];
    } else if (args[i] === '--address' || args[i] === '-a') {
      options.address = args[++i];
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    } else if (!args[i].startsWith('-') && !options.address) {
      options.address = args[i];
    }
  }

  // Try default wallet if no address
  if (!options.address) {
    const config = loadConfig();
    if (config.defaultWallet) {
      options.address = config.defaultWallet;
    }
  }

  return options;
}

function showHelp() {
  console.log(BRANDING.logoCompact);
  console.log(`
${C.bright}${C.cyan}aether-cli token-accounts${C.reset} — Get SPL token accounts

${C.bright}USAGE${C.reset}
    aether token-accounts [address] [options]
    aether token-accounts --address <addr> [options]

${C.bright}OPTIONS${C.reset}
    -a, --address <addr>  Wallet address (default: configured default)
    -r, --rpc <url>       RPC endpoint (default: ${getDefaultRpc()})
    -j, --json            Output as JSON
    -h, --help            Show this help

${C.bright}DESCRIPTION${C.reset}
    Queries the Aether blockchain for all SPL token accounts
    associated with a wallet address.

${C.bright}SDK METHOD${C.reset}
    client.getTokenAccounts(address) → GET /v1/tokens/<address>

${C.bright}EXAMPLES${C.reset}
    aether token-accounts ATHxxx...         # List token accounts
    aether token-accounts --json            # JSON output
    aether token-accounts --rpc https://custom-rpc:8899
`);
}

// ---------------------------------------------------------------------------
// Token accounts query - REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchTokenAccounts(rpc, address) {
  const client = createClient(rpc);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  // Real RPC call: GET /v1/tokens/<address>
  const tokens = await client.getTokenAccounts(rawAddr);

  return {
    address,
    tokens: tokens || [],
    rpc,
    timestamp: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printTokenAccounts(data) {
  const { address, tokens, rpc } = data;

  console.log();
  console.log(BRANDING.commandBanner('token-accounts', 'SPL Token Accounts'));

  console.log(`\n  ${key('Wallet:')}  ${highlight(address)}`);
  console.log(`  ${key('RPC:')}     ${C.dim}${rpc}${C.reset}\n`);

  if (tokens.length === 0) {
    console.log(`  ${warning('No token accounts found for this wallet.')}\n`);
    return;
  }

  // Build table rows
  const rows = tokens.map((t, i) => {
    const mint = shortenAddress(t.mint || t.token_mint || 'unknown');
    const amount = formatTokenAmount(t.amount || t.balance || 0, t.decimals || 9);
    const decimals = t.decimals || 9;
    const isFrozen = t.is_frozen || t.frozen ? indicators.warning : indicators.success;

    return [
      `${i + 1}`,
      mint,
      `${amount}`,
      `${decimals}`,
      isFrozen,
    ];
  });

  console.log(drawTable(
    ['#', 'Mint', 'Balance', 'Decimals', 'Status'],
    rows,
    { headerColor: C.cyan + C.bright, borderColor: C.dim }
  ));

  console.log(`\n  ${key('Total Accounts:')} ${value(tokens.length)}`);
  console.log(`  ${C.dim}SDK: getTokenAccounts()${C.reset}\n`);
}

function printJson(data) {
  console.log(JSON.stringify({
    address: data.address,
    tokens: data.tokens,
    total_accounts: data.tokens.length,
    rpc: data.rpc,
    timestamp: data.timestamp,
    cli_version: CLI_VERSION,
    sdk: '@jellylegsai/aether-sdk',
    rpc_endpoint: `GET /v1/tokens/${data.address}`,
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function tokenAccountsCommand() {
  const options = parseArgs();
  const { rpc, address, asJson } = options;

  if (!address) {
    console.log(`\n${indicators.error} ${error('No address provided.')}`);
    console.log(`  ${C.dim}Usage: aether token-accounts <address>${C.reset}`);
    console.log(`  ${C.dim}       aether token-accounts --address <address>${C.reset}`);
    console.log(`  ${C.dim}       aether token-accounts --help for more info${C.reset}\n`);
    process.exit(1);
  }

  if (!asJson) {
    startSpinner('Fetching token accounts');
  }

  try {
    // Real blockchain RPC call via SDK
    const data = await fetchTokenAccounts(rpc, address);

    if (!asJson) {
      stopSpinner(true, 'Token accounts retrieved');
    }

    if (asJson) {
      printJson(data);
    } else {
      printTokenAccounts(data);
    }
  } catch (err) {
    if (!asJson) {
      stopSpinner(false, 'Failed');
    }

    if (asJson) {
      console.log(JSON.stringify({
        error: err.message,
        address,
        rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${indicators.error} ${error('Failed to fetch token accounts:')} ${err.message}`);
      console.log(`  ${C.dim}Address: ${address}${C.reset}`);
      console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}Make sure the Aether node is running${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { tokenAccountsCommand: tokenAccountsCommand };

if (require.main === module) {
  tokenAccountsCommand().catch(err => {
    console.error(`${indicators.error} ${error('Token-accounts command failed:')} ${err.message}`);
    process.exit(1);
  });
}
