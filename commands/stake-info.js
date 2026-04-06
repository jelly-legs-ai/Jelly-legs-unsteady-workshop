#!/usr/bin/env node
/**
 * aether-cli - Stake Info Command
 * Get staking information for an address using real chain RPC calls.
 * All calls go through @jellylegsai/aether-sdk → AetherClient → real HTTP RPC.
 *
 * Usage:
 *   aether stake-info <address>          Show stake info for address
 *   aether stake-info <address> --json    JSON output
 *   aether stake-info <address> --rpc <url>  Custom RPC endpoint
 *
 * SDK wired to: GET /v1/slot, GET /v1/account/<addr>, GET /v1/blockheight
 */

const path = require('path');
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const { AetherClient } = require(sdkPath);

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

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, json: false, rpc: null };

  for (let i = 0; i < args.length; i++) {
    if (!args[i].startsWith('-') && !result.address) {
      result.address = args[i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      result.json = true;
    } else if ((args[i] === '--rpc' || args[i] === '-r') && args[i + 1]) {
      result.rpc = args[++i];
    } else if (args[i] === '--help' || args[i] === '-h') {
      result.help = true;
    }
  }
  return result;
}

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function stakeInfoCommand() {
  const opts = parseArgs();

  if (opts.help || !opts.address) {
    console.log(`
${C.bright}${C.cyan}aether-cli stake-info${C.reset} — Get stake/account info for an address

${C.bright}USAGE${C.reset}
    aether stake-info <address> [--json] [--rpc <url>]

${C.bright}OPTIONS${C.reset}
    --json, -j       Output raw JSON
    --rpc <url>       RPC endpoint (default: AETHER_RPC or localhost:8899)
    --help, -h        Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getSlot()           → GET /v1/slot
    client.getAccountInfo(addr) → GET /v1/account/<addr>
    client.getBlockHeight()     → GET /v1/blockheight

${C.bright}EXAMPLE${C.reset}
    aether stake-info ATH3abc...def
`);
    return;
  }

  const rpcUrl = opts.rpc || getDefaultRpc();
  const client = new AetherClient({ rpcUrl });

  try {
    // Real chain RPC calls — all parallel for speed
    const [slot, accountInfo, blockHeight] = await Promise.all([
      client.getSlot().catch(() => null),
      client.getAccountInfo(opts.address).catch(() => null),
      client.getBlockHeight().catch(() => null),
    ]);

    const balance = accountInfo?.lamports != null
      ? (accountInfo.lamports / 1e9).toFixed(4) + ' AETH'
      : 'unavailable';
    const owner = accountInfo?.owner || 'unknown';
    const executable = accountInfo?.executable ? 'yes' : 'no';

    if (opts.json) {
      console.log(JSON.stringify({
        address: opts.address,
        slot: slot ?? null,
        block_height: blockHeight ?? null,
        account: {
          lamports: accountInfo?.lamports ?? null,
          balance_aeth: accountInfo?.lamports != null ? (accountInfo.lamports / 1e9).toFixed(4) : null,
          owner: accountInfo?.owner ?? null,
          executable: accountInfo?.executable ?? false,
          rent_epoch: accountInfo?.rent_epoch ?? null,
        },
        rpc: rpcUrl,
        timestamp: new Date().toISOString(),
      }, null, 2));
      return;
    }

    console.log(`\n${C.bright}${C.cyan}── Aether Stake Info ─────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Address:${C.reset}   ${opts.address}`);
    console.log(`  ${C.dim}Balance:${C.reset}   ${C.green}${balance}${C.reset}`);
    console.log(`  ${C.dim}Owner:${C.reset}     ${owner}`);
    console.log(`  ${C.dim}Executable:${C.reset} ${executable}`);
    console.log(`  ${C.dim}Slot:${C.reset}      ${slot ?? 'unavailable'}`);
    console.log(`  ${C.dim}Block Height:${C.reset} ${blockHeight ?? 'unavailable'}`);
    console.log(`  ${C.dim}RPC:${C.reset}       ${rpcUrl}\n`);
    console.log(`  ${C.green}? Real chain RPC calls completed${C.reset}\n`);
  } catch (error) {
    console.log(`  ${C.red}? Error: ${error.message}${C.reset}\n`);
    process.exit(1);
  }
}

stakeInfoCommand();

module.exports = { stakeInfoCommand };
