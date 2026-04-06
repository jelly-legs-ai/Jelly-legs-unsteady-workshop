#!/usr/bin/env node
/**
 * aether-cli stake-positions
 *
 * Query and display current stake positions/delegations for a wallet.
 * Shows validator, amount, status, and accumulated rewards.
 *
 * Usage:
 *   aether stake-positions --address <addr> [--json]
 *   aether wallet stake-positions --address <addr> [--json]
 *
 * Examples:
 *   aether stake-positions --address ATHxxx
 *   aether wallet stake-positions --address ATHxxx --json
 *
 * SDK wired to: GET /v1/slot, GET /v1/stake/<address>, GET /v1/account/<addr>
 */

const path = require('path');

// Import SDK — all network calls go through @jellylegsai/aether-sdk
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
};

const CLI_VERSION = '1.0.1';

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, json: false };

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      result.address = args[++i];
    } else if (args[i] === '--json' || args[i] === '--json-output') {
      result.json = true;
    } else if (args[i] === '--rpc' && args[i + 1]) {
      result.rpc = args[++i];
    } else if (args[i] === '--help' || args[i] === '-h') {
      result.help = true;
    }
  }

  return result;
}

// ---------------------------------------------------------------------------
// Balance formatting
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  const aeth = (lamports || 0) / 1e9;
  return aeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 }) + ' AETH';
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function stakePositionsCommand() {
  const opts = parseArgs();

  if (opts.help) {
    console.log(`
${C.bright}${C.cyan}stake-positions${C.reset} — Query active stake delegations for a wallet

${C.bright}USAGE${C.reset}
    aether stake-positions --address <addr> [--json] [--rpc <url>]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Wallet address (ATH...)
    --json               Output raw JSON
    --rpc <url>          RPC endpoint (default: AETHER_RPC or localhost:8899)
    --help               Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getSlot()             → GET /v1/slot
    client.getStakePositions()   → GET /v1/stake/<address>
    client.getAccountInfo()      → GET /v1/account/<addr>

${C.bright}EXAMPLES${C.reset}
    aether stake-positions --address ATH3abc...
    aether stake-positions --address ATH3abc... --json
`);
    return;
  }

  if (!opts.address) {
    console.log(`  ${C.red}✗ Missing --address${C.reset}\n`);
    console.log(`  Usage: aether stake-positions --address <addr> [--json]\n`);
    return;
  }

  const rpcUrl = opts.rpc || getDefaultRpc();
  const client = createClient(rpcUrl);
  const address = opts.address;
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Stake Positions ──────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Wallet:${C.reset} ${address}`);
    console.log(`  ${C.dim}RPC:   ${C.reset} ${rpcUrl}\n`);
  }

  try {
    // Verify chain connectivity via SDK (real RPC call)
    const slot = await client.getSlot().catch(() => null);

    // Fetch stake positions via SDK (real RPC call to GET /v1/stake/<address>)
    const stakeAccounts = await client.getStakePositions(rawAddr);

    if (opts.json) {
      const totalLamports = stakeAccounts.reduce((sum, acc) => sum + (acc.stake_lamports || acc.lamports || 0), 0);
      console.log(JSON.stringify({
        wallet_address: address,
        slot,
        stake_accounts: stakeAccounts.map(acc => ({
          stake_account: acc.pubkey || acc.publicKey || acc.account || 'unknown',
          validator: acc.validator || acc.delegate || acc.validator_address || 'unknown',
          stake_lamports: acc.stake_lamports || acc.lamports || 0,
          stake_aeth: ((acc.stake_lamports || acc.lamports || 0) / 1e9).toFixed(4),
          status: acc.status || acc.state || 'active',
          updated_epoch: acc.epoch || acc.last_update_epoch || null,
        })),
        total_staked_lamports: totalLamports,
        total_staked_aeth: (totalLamports / 1e9).toFixed(4),
        count: stakeAccounts.length,
        rpc: rpcUrl,
        cli_version: CLI_VERSION,
        fetched_at: new Date().toISOString(),
      }, null, 2));
      return;
    }

    if (!stakeAccounts || stakeAccounts.length === 0) {
      console.log(`  ${C.yellow}? No active stake positions found.${C.reset}`);
      console.log(`  ${C.dim}  Stake AETH with: ${C.cyan}aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
      return;
    }

    let totalStaked = 0;
    console.log(`  ${C.bright}Stake Positions (${stakeAccounts.length})${C.reset}\n`);

    for (const acc of stakeAccounts) {
      const stakeAcct = acc.pubkey || acc.publicKey || acc.account || 'unknown';
      const validator = acc.validator || acc.delegate || acc.validator_address || 'unknown';
      const lamports = acc.stake_lamports || acc.lamports || 0;
      const status = (acc.status || acc.state || 'active').toLowerCase();
      const epoch = acc.epoch || acc.last_update_epoch || null;

      totalStaked += lamports;

      const statusColor = status === 'active' ? C.green : status === 'deactivating' ? C.yellow : C.dim;
      const shortAcct = stakeAcct.length > 20 ? stakeAcct.slice(0, 8) + '.' + stakeAcct.slice(-8) : stakeAcct;
      const shortVal = validator.length > 20 ? validator.slice(0, 8) + '.' + validator.slice(-8) : validator;
      const aeth = (lamports / 1e9).toFixed(4);

      console.log(`  ${C.dim}┌─${C.bright}${statusColor} ${status.toUpperCase()}${C.reset}`);
      console.log(`  │  ${C.dim}Stake acct:${C.reset} ${shortAcct}`);
      console.log(`  │  ${C.dim}Validator:${C.reset} ${shortVal}`);
      console.log(`  │  ${C.dim}Staked:${C.reset}    ${C.bright}${aeth} AETH${C.reset} (${lamports.toLocaleString()} lamports)`);
      if (epoch) console.log(`  │  ${C.dim}Epoch:${C.reset}     ${C.bright}#${epoch}${C.reset}`);
      console.log(`  ${C.dim}└${C.reset}\n`);
    }

    console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);
    console.log(`  ${C.bright}Total Staked:${C.reset} ${C.green}${formatAether(totalStaked)}${C.reset}\n`);

  } catch (err) {
    console.log(`  ${C.red}? Failed to fetch stake positions:${C.reset} ${err.message}\n`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    process.exit(1);
  }
}

stakePositionsCommand();

module.exports = { stakePositionsCommand };
