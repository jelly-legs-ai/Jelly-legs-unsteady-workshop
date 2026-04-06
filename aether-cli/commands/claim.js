#!/usr/bin/env node
/**
 * aether-cli claim
 *
 * Claim accumulated staking rewards for a wallet.
 * Fetches pending rewards from the chain and submits a claim transaction.
 *
 * Usage:
 *   aether claim --address <addr> [--json] [--rpc <url>]
 *
 * Examples:
 *   aether claim --address ATHxxx
 *   aether claim --address ATHxxx --json
 */

const https = require('https');
const http = require('http');

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
// Config
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, pathStr, options = {}, timeoutMs = 8000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const reqOptions = {
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: options.method || 'GET',
      headers: { 'Content-Type': 'application/json', ...options.headers },
    };
    const req = lib.request(reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve(data); }
      });
    });
    req.on('error', reject);
    req.setTimeout(timeoutMs, () => {
      req.destroy();
      reject(new Error(`Request timeout after ${timeoutMs}ms`));
    });
    if (options.body) req.write(options.body);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, json: false };

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      result.address = args[i + 1];
      i++;
    } else if (args[i] === '--json' || args[i] === '--json-output') {
      result.json = true;
    } else if (args[i] === '--rpc' && args[i + 1]) {
      result.rpc = args[i + 1];
      i++;
    } else if (args[i] === '--help' || args[i] === '-h') {
      result.help = true;
    } else if (args[i] === '--dry-run') {
      result.dryRun = true;
    }
  }

  return result;
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  const aeth = (lamports || 0) / 1e9;
  return aeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 }) + ' AETH';
}

function formatFlux(lamports) {
  const flux = (lamports || 0) / 1e6;
  return flux.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) + ' FLUX';
}

function shortPubkey(pubkey) {
  if (!pubkey || pubkey.length < 16) return pubkey || 'unknown';
  return pubkey.slice(0, 8) + '...' + pubkey.slice(-8);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function claimCommand() {
  const opts = parseArgs();

  if (opts.help) {
    console.log(`
${C.bright}${C.cyan}claim${C.reset} — Claim accumulated staking rewards for a wallet

${C.bright}USAGE${C.reset}
    aether claim --address <addr> [--json] [--rpc <url>] [--dry-run]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Wallet address (ATH...)
    --json               Output raw JSON
    --rpc <url>          RPC endpoint (default: AETHER_RPC or localhost:8899)
    --dry-run            Preview claim without submitting transaction
    --help               Show this help

${C.bright}EXAMPLES${C.reset}
    aether claim --address ATH3abc...
    aether claim --address ATH3abc... --dry-run
    aether claim --address ATH3abc... --json
`);
    return;
  }

  if (!opts.address) {
    console.log(`  ${C.red}✗ Missing --address${C.reset}\n`);
    console.log(`  Usage: aether claim --address <addr> [--json] [--dry-run]\n`);
    return;
  }

  const rpcUrl = opts.rpc || getDefaultRpc();
  const address = opts.address;
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Claim Staking Rewards ────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Wallet:${C.reset} ${address}`);
    console.log(`  ${C.dim}RPC:   ${C.reset} ${rpcUrl}`);
    if (opts.dryRun) console.log(`  ${C.yellow}(dry-run mode - no transaction will be submitted)${C.reset}`);
    console.log();
  }

  try {
    // Step 1: Fetch stake positions to calculate pending rewards
    const stakeRes = await httpRequest(rpcUrl, `/v1/stake?address=${encodeURIComponent(rawAddr)}`);

    let stakeAccounts = [];
    if (Array.isArray(stakeRes)) {
      stakeAccounts = stakeRes;
    } else if (stakeRes && typeof stakeRes === 'object') {
      stakeAccounts = stakeRes.accounts || stakeRes.stake_accounts || stakeRes.data || [];
    }

    if (!opts.json) {
      console.log(`  ${C.dim}Fetching stake positions...${C.reset}`);
    }

    if (!stakeAccounts || stakeAccounts.length === 0) {
      console.log(`  ${C.yellow}? No active stake positions found.${C.reset}`);
      console.log(`  ${C.dim}  Stake AETH with: ${C.cyan}aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
      return;
    }

    // Calculate total pending rewards
    let totalPendingRewards = 0;
    const rewardBreakdown = [];

    for (const acc of stakeAccounts) {
      const pendingRewards = acc.pending_rewards || acc.pendingRewards || acc.rewards || 0;
      const stakeLamports = acc.stake_lamports || acc.lamports || 0;
      const validator = acc.validator || acc.delegate || acc.validator_address || 'unknown';

      totalPendingRewards += pendingRewards;
      rewardBreakdown.push({
        stakeAcct: acc.pubkey || acc.publicKey || acc.account || 'unknown',
        validator,
        stakeLamports,
        pendingRewards,
      });
    }

    if (!opts.json) {
      console.log(`  ${C.bright}Stake Positions (${stakeAccounts.length})${C.reset}\n`);

      for (const pos of rewardBreakdown) {
        const shortVal = shortPubkey(pos.validator);
        const shortAcct = shortPubkey(pos.stakeAcct);
        console.log(`  ${C.dim}├─ ${C.reset}${shortAcct} → ${C.cyan}${shortVal}${C.reset}`);
        console.log(`  │   ${C.dim}Staked:${C.reset} ${formatAether(pos.stakeLamports)}`);
        console.log(`  │   ${C.green}Pending:${C.reset} ${formatFlux(pos.pendingRewards)}\n`);
      }

      console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);
      console.log(`  ${C.bright}Total Pending Rewards:${C.reset} ${C.green}${formatFlux(totalPendingRewards)}${C.reset}\n`);
    }

    // Step 2: If not dry-run, submit claim transaction
    if (opts.dryRun) {
      console.log(`  ${C.yellow}⚠ Dry run - not submitting claim transaction${C.reset}\n`);
      if (opts.json) {
        console.log(JSON.stringify({
          wallet_address: address,
          dry_run: true,
          stake_count: stakeAccounts.length,
          total_pending_flux: totalPendingRewards,
          total_pending_aeth: (totalPendingRewards / 1e6).toFixed(2),
          breakdown: rewardBreakdown.map(r => ({
            stake_account: r.stakeAcct,
            validator: r.validator,
            pending_flux: r.pendingRewards,
          })),
        }, null, 2));
      }
      return;
    }

    // Step 3: Submit claim transaction
    if (!opts.json) {
      console.log(`  ${C.dim}Submitting claim transaction...${C.reset}`);
    }

    const claimBody = JSON.stringify({
      address: rawAddr,
      stake_accounts: rewardBreakdown.map(r => r.stakeAcct),
    });

    const claimRes = await httpRequest(rpcUrl, '/v1/claim', {
      method: 'POST',
      body: claimBody,
    });

    if (opts.json) {
      console.log(JSON.stringify({
        wallet_address: address,
        success: !claimRes.error,
        total_claimed_flux: claimRes.claimed || totalPendingRewards,
        tx_signature: claimRes.signature || claimRes.txid || null,
        block_height: claimRes.block_height || null,
        claimed_at: new Date().toISOString(),
      }, null, 2));
      return;
    }

    if (claimRes.error) {
      console.log(`  ${C.red}✗ Claim failed:${C.reset} ${claimRes.error}\n`);
      process.exit(1);
    }

    console.log(`  ${C.green}✓ Rewards claimed!${C.reset}`);
    console.log(`  ${C.dim}  Amount:${C.reset} ${C.green}${formatFlux(claimRes.claimed || totalPendingRewards)}${C.reset}`);
    if (claimRes.signature || claimRes.txid) {
      console.log(`  ${C.dim}  Tx:${C.reset} ${shortPubkey(claimRes.signature || claimRes.txid)}`);
    }
    console.log();

  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({ address, error: err.message }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to claim rewards:${C.reset} ${err.message}\n`);
    }
    process.exit(1);
  }
}

module.exports = { claimCommand };

if (require.main === module) {
  claimCommand();
}
