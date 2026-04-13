#!/usr/bin/env node
/**
 * aether-cli slot
 *
 * Get current slot number from the Aether blockchain.
 * Uses @jellylegsai/aether-sdk for REAL HTTP RPC calls.
 *
 * Usage:
 *   aether slot                     Show current slot
 *   aether slot --json              JSON output for scripting
 *   aether slot --rpc <url>         Custom RPC endpoint
 *
 * RPC Endpoint: GET /v1/slot
 * SDK Function: sdk.getSlot()
 */

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
  return {
    rpc: getDefaultRpc(),
    asJson: args.includes('--json') || args.includes('-j'),
  };
}

// ---------------------------------------------------------------------------
// Slot query - REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchSlot(rpc) {
  const client = createClient(rpc);
  
  // Real RPC call: GET /v1/slot
  // SDK function: client.getSlot()
  const slot = await client.getSlot();
  
  return {
    slot,
    rpc,
    timestamp: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printSlot(data) {
  const { slot, rpc } = data;

  console.log(`\n${C.bright}${C.cyan}── Aether Current Slot ──────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.bright}Slot:${C.reset}       ${C.green}${slot.toLocaleString()}${C.reset}`);
  console.log(`  ${C.dim}RPC:${C.reset}       ${rpc}`);
  console.log(`  ${C.dim}Time:${C.reset}      ${data.timestamp}${C.reset}\n`);
  
  // Context info
  console.log(`  ${C.dim}RPC Endpoint: GET /v1/slot${C.reset}`);
  console.log(`  ${C.dim}SDK Function:  sdk.getSlot()${C.reset}\n`);
}

function printJson(data) {
  console.log(JSON.stringify({
    slot: data.slot,
    rpc: data.rpc,
    timestamp: data.timestamp,
    cli_version: CLI_VERSION,
    sdk: '@jellylegsai/aether-sdk',
    rpc_endpoint: 'GET /v1/slot',
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function slotCommand() {
  const options = parseArgs();
  const { rpc, asJson } = options;

  if (!asJson) {
    console.log(`${C.dim}Fetching current slot from ${C.cyan}${rpc}${C.dim}...${C.reset}`);
  }

  try {
    // Real blockchain RPC call via SDK
    const data = await fetchSlot(rpc);
    
    if (asJson) {
      printJson(data);
    } else {
      printSlot(data);
    }
  } catch (err) {
    if (asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc,
        timestamp: new Date().toISOString(),
      }));
    } else {
      console.log(`\n${C.red}✗ Failed to fetch slot: ${err.message}${C.reset}`);
      console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}Make sure the Aether node is running at ${rpc}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { slotCommand };

if (require.main === module) {
  slotCommand().catch(err => {
    console.error(`${C.red}Slot command failed:${C.reset} ${err.message}`);
    process.exit(1);
  });
}
