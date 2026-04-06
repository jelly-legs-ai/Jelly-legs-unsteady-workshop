#!/usr/bin/env node
/**
 * aether-cli blockhash
 *
 * Fetches the latest recent blockhash from the Aether blockchain — required
 * as a prerequisite for signing and submitting any transaction.
 *
 * Usage:
 *   aether-cli blockhash                  Show latest blockhash
 *   aether-cli blockhash --json           JSON output for scripting
 *   aether-cli blockhash --rpc <url>      Query a specific RPC endpoint
 *   aether-cli blockhash --watch          Poll every 5 seconds
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 * SDK: @jellylegsai/aether-sdk — makes REAL HTTP RPC calls to the chain
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

// Import SDK — REAL blockchain RPC calls to http://127.0.0.1:8899
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

const DEFAULT_RPC = process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: DEFAULT_RPC,
    asJson: false,
    watch: false,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--rpc' || args[i] === '-r') {
      options.rpc = args[++i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      options.asJson = true;
    } else if (args[i] === '--watch' || args[i] === '-w') {
      options.watch = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return options;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli blockhash${C.reset} - Get Latest Blockhash

${C.bright}Usage:${C.reset}
  aether-cli blockhash [options]

${C.bright}Options:${C.reset}
  --rpc <url>     Query a specific RPC endpoint (default: http://127.0.0.1:8899)
  --json, -j      Output raw JSON for scripting
  --watch, -w     Poll every 5 seconds
  --help, -h      Show this help message

${C.bright}Description:${C.reset}
  Fetches the most recent blockhash from the Aether blockchain.
  Blockhashes are required as a recent-ref entry when building
  signed transactions (Transfer, Stake, Unstake, etc.).

  Every call makes a REAL HTTP RPC request to the configured
  Aether node — no caching, no stubs.

${C.bright}Examples:${C.reset}
  aether blockhash
  aether blockhash --json
  aether blockhash --watch
  AETHER_RPC=https://my-node:8899 aether blockhash
`);
}

// ---------------------------------------------------------------------------
// Fetch and display blockhash from REAL chain via SDK
// ---------------------------------------------------------------------------

async function fetchBlockhash(rpc) {
  const client = new aether.AetherClient({ rpcUrl: rpc });
  const result = await client.getRecentBlockhash();
  return result;
}

function printBlockhash(result, rpc) {
  const { blockhash, lastValidBlockHeight } = result;

  console.log(`\n${C.bright}${C.cyan}── Aether Blockhash ─────────────────────────────────────${C.reset}\n`);
  console.log(`  ${C.bright}Blockhash:${C.reset}              ${C.magenta}${blockhash}${C.reset}`);
  if (lastValidBlockHeight !== undefined) {
    console.log(`  ${C.bright}Last Valid Block Height:${C.reset}  ${C.cyan}${lastValidBlockHeight.toLocaleString()}${C.reset}`);
  }
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}\n`);
}

function printJson(result, rpc) {
  console.log(JSON.stringify({
    blockhash: result.blockhash,
    lastValidBlockHeight: result.lastValidBlockHeight,
    rpc,
    cli_version: '1.0.0',
    timestamp: new Date().toISOString(),
  }));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();
  const { rpc, asJson, watch } = opts;

  if (watch && !asJson) {
    console.log(`${C.dim}Watching for new blockhashes every 5s (Ctrl+C to stop)…${C.reset}`);
    console.log();
  }

  async function tick() {
    try {
      const result = await fetchBlockhash(rpc);
      if (asJson) {
        printJson(result, rpc);
      } else if (!watch) {
        printBlockhash(result, rpc);
      } else {
        process.stdout.write(`\r${C.dim}[${new Date().toISOString()}]${C.reset} blockhash: ${C.magenta}${result.blockhash}${C.reset}  `);
      }
    } catch (err) {
      if (asJson) {
        console.log(JSON.stringify({ error: err.message, rpc }));
      } else if (!watch) {
        console.log(`${C.red}✗ Failed to fetch blockhash: ${err.message}${C.reset}`);
      } else {
        process.stdout.write(`\r${C.red}✗ ${err.message}${C.reset}  `);
      }
    }
  }

  if (watch) {
    await tick(); // immediate first run
    // eslint-disable-next-line no-unmodified-loop-condition
    while (true) {
      await new Promise(r => setTimeout(r, 5000));
      await tick();
    }
  } else {
    await tick();
  }
}

main().catch(err => {
  console.error(`${C.red}✗ Blockhash command failed:${C.reset} ${err.message}`);
  process.exit(1);
});

module.exports = { blockhashCommand: main };

if (require.main === module) {
  main();
}
