#!/usr/bin/env node
/**
 * aether-cli validators
 *
 * List, filter, and inspect validators on the Aether network.
 * Shows identity, tier, stake, APY, uptime, and performance metrics.
 *
 * Usage:
 *   aether validators list              List all active validators
 *   aether validators list --tier full   Filter by tier (full|lite|observer)
 *   aether validators list --json        JSON output for scripting
 *   aether validators list --rpc <url>   Query a specific RPC endpoint
 *   aether validators list --sort stake  Sort by stake (default: score)
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

const http = require('http');
const https = require('https');

// ANSI colours
const C = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: 8000,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve({ raw: data }); } });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

function httpPost(rpcUrl, path, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 8000,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(bodyStr) },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve(data); } });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.write(bodyStr);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // [node, index.js, validators, list, ...]
  const opts = {
    rpc: DEFAULT_RPC,
    subcmd: 'list',
    tier: null,
    asJson: false,
    sortBy: 'score',
    limit: 100,
    rank: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--tier' || arg === '-t') {
      const tier = (args[++i] || '').toLowerCase();
      if (['full', 'lite', 'observer'].includes(tier)) {
        opts.tier = tier;
      } else {
        console.log(`  ${C.yellow}⚠ Invalid tier "${tier}". Valid: full, lite, observer${C.reset}`);
      }
    } else if (arg === '--json' || arg === '-j') {
      opts.asJson = true;
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--sort' || arg === '-s') {
      const sortVal = (args[++i] || '').toLowerCase();
      if (['stake', 'score', 'apy', 'uptime', 'name'].includes(sortVal)) {
        opts.sortBy = sortVal;
      } else {
        console.log(`  ${C.yellow}⚠ Invalid sort "${sortVal}". Valid: stake, score, apy, uptime, name${C.reset}`);
      }
    } else if (arg === '--limit' || arg === '-l') {
      const limit = parseInt(args[++i], 10);
      if (!isNaN(limit) && limit > 0 && limit <= 500) {
        opts.limit = limit;
      }
    } else if (arg === '--help' || arg === '-h') {
      showHelp();
      process.exit(0);
    } else if (arg === '--rank') {
      opts.rank = true;
      opts.subcmd = 'rank';
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli validators${C.reset} - List and inspect Aether validators

${C.bright}Usage:${C.reset}
  aether validators list [options]
  aether validators rank [options]        Ranked leaderboard (sorted by stake)

${C.bright}Options (list):${C.reset}
  -t, --tier <type>   Filter by tier: full, lite, observer
  -s, --sort <field>  Sort by: stake, score, apy, uptime, name (default: score)
  -l, --limit <n>     Max validators to show (default: 100, max: 500)
  -r, --rpc <url>     RPC endpoint (default: ${DEFAULT_RPC} or $AETHER_RPC)
  -j, --json          Output raw JSON (for scripting)
  -h, --help          Show this help message

${C.bright}Options (rank):${C.reset}
  -t, --tier <type>   Filter by tier: full, lite, observer
  -l, --limit <n>     Max validators to show (default: 50, max: 200)
  -r, --rpc <url>     RPC endpoint (default: ${DEFAULT_RPC} or $AETHER_RPC)
  -j, --json          Output raw JSON (for scripting)
  -h, --help          Show this help message

${C.bright}Examples:${C.reset}
  aether validators list                  # All validators, sorted by score
  aether validators list --tier full       # Full validators only
  aether validators list --sort stake      # Sort by total stake
  aether validators list --sort apy        # Sort by estimated APY
  aether validators list --json           # JSON for scripts
  aether validators rank                  # Top validators by stake (leaderboard)
  aether validators rank --tier full       # Full validators only
  aether validators rank --limit 20       # Top 20 validators
  aether validators list --rpc http://custom-rpc:8899
`.trim());
}

// ---------------------------------------------------------------------------
// Data fetchers
// ---------------------------------------------------------------------------

/** Fetch all validators from the network */
async function fetchValidators(rpc) {
  try {
    // Try /v1/validators first (standard Aether RPC endpoint)
    const res = await httpRequest(rpc, '/v1/validators');
    if (res && !res.error) {
      if (Array.isArray(res)) return res;
      if (res.validators && Array.isArray(res.validators)) return res.validators;
      if (res.accounts && Array.isArray(res.accounts)) return res.accounts;
    }
    // Fallback: POST to a validators query endpoint
    const res2 = await httpPost(rpc, '/v1/validators', {});
    if (res2 && !res2.error) {
      if (Array.isArray(res2)) return res2;
      if (res2.validators && Array.isArray(res2.validators)) return res2.validators;
    }
    return [];
  } catch {
    return [];
  }
}

/** Fetch epoch info for APY calculations */
async function fetchEpochInfo(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/epoch-info');
    return res;
  } catch {
    return null;
  }
}

/** Fetch network-wide stake totals for APY estimation */
async function fetchSupply(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/supply');
    return res;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Normalise validator record from various RPC response shapes
// ---------------------------------------------------------------------------

function normaliseValidator(v) {
  // Handle different response shapes from different RPC implementations
  const pubkey = v.pubkey || v.address || v.identity || v.id || null;
  const name = v.name || v.moniker || v.label || v.identity_name || null;
  const tier = (v.tier || v.node_type || v.type || 'full').toLowerCase();
  const stake = BigInt(v.stake || v.delegatedStake || v.stake_lamports || v.lamports || 0);
  const score = v.score !== undefined ? v.score : (v.uptime !== undefined ? Math.round(v.uptime * 100) : null);
  const apy = v.apy !== undefined ? v.apy : (v.apy_bps !== undefined ? v.apy_bps / 100 : null);
  const commission = v.commission !== undefined ? v.commission : (v.commission_bps !== undefined ? v.commission_bps / 100 : null);
  const version = v.version || v.agent || v.app_version || null;
  const ip = v.ip || v.remote || null;
  const lastVote = v.last_vote || v.lastVote || null;
  const epoch = v.epoch || null;
  const voteAccount = v.vote_account || v.voteAccount || null;

  return {
    pubkey,
    name,
    tier,
    stake: stake.toString(),
    stakeFormatted: formatAether(stake),
    stakeAeth: Number(stake) / 1e9,
    score,
    apy,
    commission,
    version,
    ip,
    lastVote,
    epoch,
    voteAccount,
    // Raw for JSON export
    _raw: v,
  };
}

function formatAether(lamports) {
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(2).replace(/\.?0+$/, '') + ' AETH';
}

function formatScore(score) {
  if (score === null || score === undefined) return `${C.dim}—${C.reset}`;
  if (score >= 80) return `${C.green}${score}${C.reset}`;
  if (score >= 50) return `${C.yellow}${score}${C.reset}`;
  return `${C.red}${score}${C.reset}`;
}

// ---------------------------------------------------------------------------
// Render outputs
// ---------------------------------------------------------------------------

function tierColor(tier) {
  if (tier === 'full') return `${C.cyan}FULL${C.reset}`;
  if (tier === 'lite') return `${C.yellow}LITE${C.reset}`;
  if (tier === 'observer') return `${C.green}OBS${C.reset}`;
  return `${C.dim}${tier.toUpperCase()}${C.reset}`;
}

function tierBadge(tier) {
  if (tier === 'full') return `${C.cyan}◆ FULL${C.reset}`;
  if (tier === 'lite') return `${C.yellow}◇ LITE${C.reset}`;
  if (tier === 'observer') return `${C.green}○ OBS${C.reset}`;
  return `${C.dim}[${tier}]${C.reset}`;
}

function renderTable(validators, opts) {
  const sortBy = opts.sortBy;
  const tier = opts.tier;

  // Sort validators
  const sorted = [...validators].sort((a, b) => {
    if (sortBy === 'stake') return b.stakeAeth - a.stakeAeth;
    if (sortBy === 'score') return (b.score || 0) - (a.score || 0);
    if (sortBy === 'apy') return (b.apy || 0) - (a.apy || 0);
    if (sortBy === 'name') return (a.name || '').localeCompare(b.name || '');
    return 0;
  });

  // Filter by tier
  const filtered = tier
    ? sorted.filter(v => v.tier === tier)
    : sorted;

  const shown = filtered.slice(0, opts.limit);
  const total = filtered.length;

  // Header
  console.log();
  console.log(`${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║${C.reset}              ${C.bright}AETHER VALIDATORS${C.reset}  ${C.dim}(total: ${total})${C.reset}                      ${C.bright}║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════════════════╝${C.reset}`);
  if (tier) console.log(`  ${C.dim}Tier filter: ${tier.toUpperCase()}   Sort: ${sortBy}   RPC: ${opts.rpc}${C.reset}`);
  else console.log(`  ${C.dim}Sort: ${sortBy}   RPC: ${opts.rpc}${C.reset}`);
  console.log();

  if (shown.length === 0) {
    console.log(`  ${C.yellow}⚠ No validators found${C.reset}${tier ? ` for tier "${tier}"` : ''}.`);
    console.log(`  ${C.dim}  Check your RPC endpoint: ${opts.rpc}${C.reset}\n`);
    return;
  }

  // Table header
  console.log(`  ${C.bright}┌──────────────────────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(
    `  ${C.bright}│${C.reset}` +
    ` ${C.cyan}#${C.reset}`.padEnd(4) +
    `${C.cyan}Validator${C.reset}`.padEnd(36) +
    `${C.cyan}Tier${C.reset}`.padEnd(8) +
    `${C.cyan}Stake${C.reset}`.padEnd(14) +
    `${C.cyan}Score${C.reset}`.padEnd(8) +
    `${C.cyan}APY${C.reset}`.padEnd(8) +
    `${C.cyan}Version${C.reset}`.padEnd(10) +
    `${C.bright}│${C.reset}`
  );
  console.log(`  ${C.bright}├${'─'.repeat(90)}${C.bright}│${C.reset}`);

  for (let i = 0; i < shown.length; i++) {
    const v = shown[i];
    const num = (i + 1).toString().padStart(3);
    const nameOrKey = v.name
      ? v.name.substring(0, 20).padEnd(20)
      : (v.pubkey ? v.pubkey.substring(0, 20).padEnd(20) : 'unknown'.padEnd(20));
    const tierStr = tierBadge(v.tier);
    const stakeStr = v.stakeFormatted.padEnd(12);
    const scoreStr = v.score !== null && v.score !== undefined
      ? `${v.score}%`.padEnd(6)
      : '—'.padEnd(6);
    const apyStr = v.apy !== null && v.apy !== undefined
      ? `${v.apy.toFixed(1)}%`.padEnd(6)
      : '—'.padEnd(6);
    const versionStr = v.version ? v.version.substring(0, 10).padEnd(10) : '—'.padEnd(10);

    const scoreColor = v.score === null || v.score === undefined ? C.dim
      : v.score >= 80 ? C.green
      : v.score >= 50 ? C.yellow
      : C.red;

    console.log(
      `  ${C.bright}│${C.reset}` +
      ` ${C.dim}${num}${C.reset} `.substring(0, 5) +
      `${C.cyan}${nameOrKey}${C.reset} ` +
      `${tierStr} `.substring(0, 9) +
      `${C.green}${stakeStr}${C.reset} ` +
      `${scoreColor}${scoreStr}${C.reset} ` +
      `${C.green}${apyStr}${C.reset} ` +
      `${C.dim}${versionStr}${C.reset} ` +
      `${C.bright}│${C.reset}`
    );
  }

  console.log(`  ${C.bright}└${'─'.repeat(90)}${C.bright}│${C.reset}`);
  console.log();

  // Summary row
  const totalStake = shown.reduce((sum, v) => sum + v.stakeAeth, 0);
  const avgScore = shown.reduce((sum, v) => sum + (v.score || 0), 0) / shown.filter(v => v.score !== null).length;
  const fullCount = shown.filter(v => v.tier === 'full').length;
  const liteCount = shown.filter(v => v.tier === 'lite').length;
  const obsCount = shown.filter(v => v.tier === 'observer').length;

  console.log(`  ${C.dim}Showing ${shown.length} of ${total} validators${total !== shown.length ? ` (limit ${opts.limit})` : ''}${C.reset}`);
  console.log(`  ${C.dim}Total stake shown: ${C.reset}${C.green}${totalStake.toFixed(2)} AETH${C.reset}  ${C.dim}Avg score: ${C.reset}${avgScore ? `${avgScore.toFixed(1)}%` : '—'}${C.reset}`);
  if (!tier) {
    console.log(`  ${C.cyan}◆${C.reset} ${C.cyan}Full${C.reset}: ${fullCount}   ${C.yellow}◇${C.reset} ${C.yellow}Lite${C.reset}: ${liteCount}   ${C.green}○${C.reset} ${C.green}Observer${C.reset}: ${obsCount}`);
  }
  console.log();
  console.log(`  ${C.dim}Tip: --tier full|lite|observer  |  --sort stake|score|apy|name  |  --json for data${C.reset}`);
  console.log();
}

function renderJson(validators, opts) {
  const tier = opts.tier;
  const filtered = tier
    ? validators.filter(v => v.tier === tier)
    : validators;

  const out = {
    rpc: opts.rpc,
    total: filtered.length,
    sort: opts.sortBy,
    tier_filter: tier,
    fetched_at: new Date().toISOString(),
    validators: filtered.map(v => ({
      pubkey: v.pubkey,
      name: v.name,
      tier: v.tier,
      stake: v.stake,
      stake_aeth: v.stakeAeth,
      stake_formatted: v.stakeFormatted,
      score: v.score,
      apy: v.apy,
      commission: v.commission,
      version: v.version,
      ip: v.ip,
      vote_account: v.voteAccount,
      last_vote: v.lastVote,
      epoch: v.epoch,
    })),
  };

  console.log(JSON.stringify(out, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function validatorsList(opts) {
  const rpc = opts.rpc;

  if (!opts.asJson) {
    console.log(`${C.dim}Fetching validators from ${rpc}...${C.reset}`);
  }

  const [rawValidators, epochInfo, supply] = await Promise.all([
    fetchValidators(rpc),
    fetchEpochInfo(rpc),
    fetchSupply(rpc),
  ]);

  if (rawValidators.length === 0) {
    if (opts.asJson) {
      console.log(JSON.stringify({ rpc, validators: [], total: 0, error: 'No validator data returned from RPC' }, null, 2));
    } else {
      console.log(`\n  ${C.yellow}⚠ No validator data returned from RPC.${C.reset}`);
      console.log(`  ${C.dim}  RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}  Check that your validator is running and the RPC endpoint is accessible.${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=http://your-rpc-url${C.reset}\n`);
    }
    return;
  }

  // Normalise all validators
  let validators = rawValidators.map(normaliseValidator);

  // Estimate APY if not provided by RPC (rough approximation)
  if (supply && !supply.error) {
    const totalStake = Number(supply.total_staked || supply.total || 0);
    const rewardsPerEpoch = Number(epochInfo?.rewards_per_epoch || '2000000000');
    if (totalStake > 0 && rewardsPerEpoch > 0) {
      const apyEstimate = (rewardsPerEpoch / totalStake) * 73; // ~73 epochs/year
      validators = validators.map(v => {
        if (v.apy === null || v.apy === undefined) {
          return { ...v, apy: apyEstimate };
        }
        return v;
      });
    }
  }

  if (opts.asJson) {
    renderJson(validators, opts);
  } else {
    renderTable(validators, opts);
  }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();

  if (opts.subcmd === 'list') {
    await validatorsList(opts);
  } else {
    console.log(`\n  ${C.red}Unknown subcommand:${C.reset} ${opts.subcmd}`);
    console.log(`  ${C.dim}Usage: aether validators list [--tier full] [--sort stake] [--json]${C.reset}\n`);
    process.exit(1);
  }
}

main().catch(err => {
  console.error(`\n${C.red}✗ Validators command failed:${C.reset} ${err.message}`);
  console.error(`  ${C.dim}Set custom RPC: AETHER_RPC=http://your-rpc-url${C.reset}\n`);
  process.exit(1);
});

module.exports = { validatorsListCommand: main };

if (require.main === module) {
  main();
}
