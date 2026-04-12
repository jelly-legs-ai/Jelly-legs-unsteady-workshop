#!/usr/bin/env node
/**
 * aether-cli blockheight
 *
 * Get current block height from the Aether blockchain.
 * Uses @jellylegsai/aether-sdk for REAL HTTP RPC calls.
 *
 * Usage:
 *   aether blockheight                  Show current block height
 *   aether blockheight --json           JSON output for scripting
 *   aether blockheight --rpc <url>      Custom RPC endpoint
 *   aether blockheight --compare          Compare with slot (show lag)
 *
 * RPC Endpoint: GET /v1/blockheight
 * SDK Function: sdk.getBlockHeight()
 */

const path = require('path');

// Import UI framework for consistent branding
const { BRANDING, C, indicators, startSpinner, stopSpinner,
        success, error, code, highlight } = require('../lib/ui');

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

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: getDefaultRpc(),
    asJson: false,
    compare: false,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--json' || args[i] === '-j') {
      options.asJson = true;
    } else if (args[i] === '--rpc' || args[i] === '-r') {
      options.rpc = args[++i];
    } else if (args[i] === '--compare' || args[i] === '-c') {
      options.compare = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return options;
}

function showHelp() {
  console.log(BRANDING.logoCompact);
  console.log(`
${C.bright}${C.cyan}aether-cli blockheight${C.reset} — Get current block height

${C.bright}USAGE${C.reset}
    aether blockheight [options]

${C.bright}OPTIONS${C.reset}
    -r, --rpc <url>     RPC endpoint (default: ${getDefaultRpc()})
    -j, --json          Output as JSON
    -c, --compare       Compare with slot (show lag)
    -h, --help          Show this help

${C.bright}DESCRIPTION${C.reset}
    Queries the Aether blockchain for the current block height.
    Block height represents the number of confirmed blocks in the chain.
    This may differ from slot number due to skipped slots.

${C.bright}SDK METHOD${C.reset}
    client.getBlockHeight() → GET /v1/blockheight

${C.bright}EXAMPLES${C.reset}
    aether blockheight              # Human-readable output
    aether blockheight --json       # JSON for scripting
    aether blockheight --compare    # Show slot vs block height
`);
}

// ---------------------------------------------------------------------------
// Block height query - REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchBlockHeight(rpc, compare = false) {
  const client = createClient(rpc);

  if (compare) {
    // Fetch both slot and block height
    const [blockHeight, slot] = await Promise.all([
      client.getBlockHeight(),
      client.getSlot(),
    ]);

    return {
      blockHeight,
      slot,
      lag: slot - blockHeight,
      rpc,
      timestamp: new Date().toISOString(),
    };
  }

  // Just block height
  const blockHeight = await client.getBlockHeight();

  return {
    blockHeight,
    rpc,
    timestamp: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printBlockHeight(data) {
  const { blockHeight, rpc } = data;

  console.log(`\n${C.bright}${C.cyan}── Aether Block Height ─────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.bright}Block Height:${C.reset}  ${C.green}${blockHeight.toLocaleString()}${C.reset}`);

  if (data.slot !== undefined) {
    console.log(`  ${C.bright}Current Slot:${C.reset}  ${C.cyan}${data.slot.toLocaleString()}${C.reset}`);
    const lagColor = data.lag > 100 ? C.red : data.lag > 10 ? C.yellow : C.green;
    console.log(`  ${C.bright}Slot Lag:${C.reset}      ${lagColor}${data.lag} slots${C.reset}`);
  }

  console.log(`  ${C.dim}RPC:${C.reset}          ${rpc}`);
  console.log(`  ${C.dim}Time:${C.reset}         ${data.timestamp}${C.reset}\n`);

  // Context info
  console.log(`  ${C.dim}RPC Endpoint: GET /v1/blockheight${C.reset}`);
  console.log(`  ${C.dim}SDK Function:  client.getBlockHeight()${C.reset}\n`);
}

function printJson(data) {
  const output = {
    block_height: data.blockHeight,
    rpc: data.rpc,
    timestamp: data.timestamp,
    cli_version: CLI_VERSION,
    sdk: '@jellylegsai/aether-sdk',
    rpc_endpoint: 'GET /v1/blockheight',
  };

  if (data.slot !== undefined) {
    output.slot = data.slot;
    output.slot_lag = data.lag;
  }

  console.log(JSON.stringify(output, null, 2));
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function blockheightCommand() {
  const options = parseArgs();
  const { rpc, asJson, compare } = options;

  if (!asJson) {
    startSpinner('Fetching block height');
  }

  try {
    // Real blockchain RPC call via SDK
    const data = await fetchBlockHeight(rpc, compare);

    if (!asJson) {
      stopSpinner(true, 'Block height retrieved');
    }

    if (asJson) {
      printJson(data);
    } else {
      printBlockHeight(data);
    }
  } catch (err) {
    if (!asJson) {
      stopSpinner(false, 'Failed');
    }

    if (asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${indicators.error} ${error('Failed to fetch block height:')} ${err.message}`);
      console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}Make sure the Aether node is running at ${rpc}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { blockheightCommand };

if (require.main === module) {
  blockheightCommand().catch(err => {
    console.error(`${indicators.error} ${error('Blockheight command failed:')} ${err.message}`);
    process.exit(1);
  });
}
