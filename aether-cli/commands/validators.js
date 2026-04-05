/**
 * aether-cli validators - Aether Validator Registry
 *
 * List and explore validators available for staking.
 *   aether validators list         — Show all active validators
 *   aether validators list --json  — JSON output for scripting
 *   aether validators list --tier <full|lite|observer> — Filter by tier
 *
 * @see docs/MINING_VALIDATOR_TOOLS.md for spec
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
// HTTP helpers (mirrors network.js patterns)
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: 8000,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // skip 'validators' and subcommand
  const opts = {
    rpc: DEFAULT_RPC,
    tier: null,
    asJson: false,
    sort: 'stake', // 'stake' | 'name' | 'uptime'
    limit: 50,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--rpc' || args[i] === '-r') {
      opts.rpc = args[++i];
    } else if (args[i] === '--tier' || args[i] === '-t') {
      opts.tier = args[++i]?.toLowerCase();
    } else if (args[i] === '--json' || args[i] === '-j') {
      opts.asJson = true;
    } else if (args[i] === '--sort' || args[i] === '-s') {
      opts.sort = args[++i]?.toLowerCase();
    } else if (args[i] === '--limit' || args[i] === '-l') {
      opts.limit = parseInt(args[++i], 10) || 50;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli validators${C.reset} - Aether Validator Registry

${C.bright}Usage:${C.reset}
  aether validators list [options]

${C.bright}Options:${C.reset}
  -r, --rpc <url>      RPC endpoint (default: ${DEFAULT_RPC} or $AETHER_RPC)
  -t, --tier <tier>    Filter by tier: full, lite, observer
  -s, --sort <field>   Sort by: stake (default), name, uptime
  -l, --limit <n>      Max validators to show (default: 50)
  -j, --json           Output raw JSON
  -h, --help           Show this help

${C.bright}Examples:${C.reset}
  aether validators list
  aether validators list --tier full
  aether validators list --sort stake --limit 20
  aether validators list --json
`.trim());
}

// ---------------------------------------------------------------------------
// Data fetching
// ---------------------------------------------------------------------------

/** Fetch validators list from RPC */
async function fetchValidators(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/validators');
    if (Array.isArray(res)) return res;
    if (res.validators && Array.isArray(res.validators)) return res.validators;
    return [];
  } catch (err) {
    return [];
  }
}

/** Fetch epoch info for context */
async function fetchEpoch(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/epoch');
    return res;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  if (!lamports && lamports !== 0) return '?';
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0';
  return aeth.toFixed(2).replace(/\.?0+$/, '');
}

function formatPct(n) {
  if (n === undefined || n === null) return '?';
  return n.toFixed(1) + '%';
}

function tierBadge(tier) {
  const map = {
    full: `${C.cyan}FULL${C.reset}`,
    lite: `${C.yellow}LITE${C.reset}`,
    observer: `${C.green}OBS${C.reset}`,
  };
  return map[tier?.toLowerCase()] || `${C.dim}?${C.reset}`;
}

function scoreColor(score) {
  if (score === undefined || score === null) return C.dim;
  if (score >= 90) return C.green;
  if (score >= 70) return C.yellow;
  return C.red;
}

function shortenAddr(addr, len = 16) {
  if (!addr) return '?';
  if (addr.length <= len) return addr;
  return addr.slice(0, 8) + '…' + addr.slice(-6);
}

// ---------------------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------------------

function renderList(validators, epochData, opts, rpc) {
  const filtered = opts.tier
    ? validators.filter(v => (v.tier || v.node_type || '').toLowerCase() === opts.tier)
    : validators;

  // Sort
  if (opts.sort === 'name') {
    filtered.sort((a, b) => (a.name || a.address || '').localeCompare(b.name || b.address || ''));
  } else if (opts.sort === 'uptime') {
    filtered.sort((a, b) => (b.uptime || b.score || 0) - (a.uptime || a.score || 0));
  } else {
    // Default: by stake (descending)
    filtered.sort((a, b) => (b.stake || b.stake_amount || 0) - (a.stake || a.stake_amount || 0));
  }

  const shown = filtered.slice(0, opts.limit);
  const totalStake = validators.reduce((sum, v) => sum + (v.stake || v.stake_amount || 0), 0);
  const networkScore = validators.reduce((sum, v) => sum + (v.score || v.uptime || 0) * (v.stake || 1), 0) / (totalStake || 1);

  console.log();
  console.log(`${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║${C.reset}              ${C.bright}AETHER VALIDATOR REGISTRY${C.reset}${C.cyan}                           ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════════╝${C.reset}`);
  console.log(`  ${C.dim}RPC:${C.reset} ${rpc}`);
  console.log(`  ${C.dim}Total validators:${C.reset} ${C.bright}${validators.length}${C.reset}`);
  if (opts.tier) console.log(`  ${C.dim}Filtered by tier:${C.reset} ${C.bright}${opts.tier}${C.reset}`);
  console.log();

  // Summary stats
  console.log(`  ${C.bright}┌──────────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Total Stake${C.reset}         ${C.bright}│${C.reset}  ${C.green}${formatAether(totalStake).padEnd(20)} AETH${C.reset}`.padEnd(80) + `${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Network Uptime${C.reset}     ${C.bright}│${C.reset}  ${scoreColor(networkScore)}${formatPct(networkScore).padEnd(20)}${C.reset}`.padEnd(80) + `${C.bright}│${C.reset}`);
  if (epochData && epochData.epoch !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Current Epoch${C.reset}      ${C.bright}│${C.reset}  ${C.green}${epochData.epoch}${C.reset}`.padEnd(80) + `${C.bright}│${C.reset}`);
  }
  console.log(`  ${C.bright}└──────────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();

  if (shown.length === 0) {
    console.log(`  ${C.yellow}⚠ No validators found${C.reset}${opts.tier ? ` for tier "${opts.tier}"` : ''}.`);
    console.log(`  ${C.dim}Try without --tier filter or check RPC connectivity.${C.reset}`);
    console.log();
    return;
  }

  // Table header
  console.log(`  ${C.bright}┌────┬────────────────────────┬────────┬─────────┬────────┬─────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset} ${C.cyan}#${C.reset}  ${C.cyan}Validator${C.reset}                    ${C.cyan}Tier${C.reset}     ${C.cyan}Stake${C.reset}    ${C.cyan}Uptime${C.reset} ${C.cyan}Commission${C.reset} ${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├────┼────────────────────────┼────────┼─────────┼────────┼─────────────┤${C.reset}`);

  for (let i = 0; i < shown.length; i++) {
    const v = shown[i];
    const num = (i + 1).toString().padStart(3);
    const addr = shortenAddr(v.address || v.pubkey || v.id);
    const tierStr = tierBadge(v.tier || v.node_type);
    const stake = formatAether(v.stake || v.stake_amount || 0);
    const uptime = formatPct(v.uptime || v.score);
    const commission = formatPct(v.commission || v.fee);
    const row = `  ${C.bright}│${C.reset} ${C.dim}${num}${C.reset}  ${addr.padEnd(24)} ${tierStr.padEnd(8)} ${stake.padEnd(9)} ${scoreColor(v.uptime || v.score)}${uptime.padEnd(9)}${C.reset} ${C.dim}${commission}${C.reset} ${C.bright}│${C.reset}`;
    console.log(row);
  }

  console.log(`  ${C.bright}└────┴────────────────────────┴────────┴─────────┴────────┴─────────────┘${C.reset}`);
  console.log();

  if (filtered.length > opts.limit) {
    console.log(`  ${C.dim}Showing ${opts.limit} of ${filtered.length} validators. Use --limit to see more.${C.reset}`);
  }

  console.log();
  console.log(`  ${C.dim}Stake to a validator:${C.reset}`);
  console.log(`    ${C.cyan}aether stake --validator <address> --amount <aeth>${C.reset}`);
  console.log(`    ${C.cyan}aether stake --validator <address> --amount <aeth> --dry-run${C.reset}  ${C.dim}(preview)${C.reset}`);
  console.log();
}

function renderJson(validators, epochData, opts, rpc) {
  const out = {
    rpc,
    fetchedAt: new Date().toISOString(),
    total: validators.length,
    epoch: epochData?.epoch ?? null,
    validators: validators.map(v => ({
      address: v.address || v.pubkey || v.id,
      name: v.name || null,
      tier: v.tier || v.node_type || null,
      stake: v.stake || v.stake_amount || 0,
      stakeAETH: formatAether(v.stake || v.stake_amount || 0),
      uptime: v.uptime ?? v.score ?? null,
      uptimePct: formatPct(v.uptime ?? v.score),
      commission: v.commission ?? v.fee ?? null,
      commissionPct: formatPct(v.commission ?? v.fee),
      lastSeen: v.last_seen || v.lastActive || null,
      version: v.version || v.clientVersion || null,
    })),
  };
  console.log(JSON.stringify(out, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function validatorsListCommand() {
  const opts = parseArgs();
  const rpc = opts.rpc;

  if (!opts.asJson) {
    console.log(`\n${C.cyan}Fetching validators...${C.reset} ${C.dim}(${rpc})${C.reset}`);
  }

  const [validators, epochData] = await Promise.all([
    fetchValidators(rpc),
    fetchEpoch(rpc),
  ]);

  if (validators.length === 0 && !opts.asJson) {
    console.log(`\n  ${C.yellow}⚠ No validator data returned from RPC.${C.reset}`);
    console.log(`  ${C.dim}  Is your validator running and fully synced?${C.reset}`);
    console.log(`  ${C.dim}  Check: aether-cli network${C.reset}`);
    console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=http://your-rpc-url${C.reset}\n`);
    return;
  }

  if (opts.asJson) {
    renderJson(validators, epochData, opts, rpc);
  } else {
    renderList(validators, epochData, opts, rpc);
  }
}

module.exports = { validatorsListCommand };

if (require.main === module) {
  validatorsListCommand().catch((err) => {
    console.error(`\n${C.red}✗ Validators command failed:${C.reset} ${err.message}`);
    console.error(`  ${C.dim}Check that your validator is running and RPC is accessible.${C.reset}\n`);
    process.exit(1);
  });
}