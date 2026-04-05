#!/usr/bin/env node
/**
 * aether-cli network - Aether Network Status
 *
 * Queries the Aether network for broad health metrics:
 *   - Network-wide slot, block height, slot production
 *   - Connected peers and their info
 *   - Consensus status and epoch info
 *   - TPS estimates from the network
 *
 * Usage:
 *   aether-cli network             # Interactive summary view
 *   aether-cli network --json      # JSON output for scripting
 *   aether-cli network --rpc <url> # Query a specific RPC endpoint
 *   aether-cli network --peers     # Detailed peer list
 *   aether-cli network --epoch     # Current epoch and consensus info
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
  white: '\x1b[37m',
};

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path, options = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    const reqOptions = {
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: 5000,
      headers: { 'Content-Type': 'application/json' },
    };

    const req = lib.request(reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch {
          resolve({ raw: data });
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });

    req.end();
  });
}

function httpPost(rpcUrl, path, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;
    const bodyStr = JSON.stringify(body);

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 5000,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve(data); }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });

    req.write(bodyStr);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: DEFAULT_RPC,
    showPeers: false,
    showEpoch: false,
    asJson: false,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--rpc' || args[i] === '-r') {
      options.rpc = args[++i];
    } else if (args[i] === '--peers' || args[i] === '-p') {
      options.showPeers = true;
    } else if (args[i] === '--epoch' || args[i] === '-e') {
      options.showEpoch = true;
    } else if (args[i] === '--json' || args[i] === '-j') {
      options.asJson = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return options;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli network${C.reset} - Aether Network Status

${C.bright}Usage:${C.reset}
  aether-cli network [options]

${C.bright}Options:${C.reset}
  -r, --rpc <url>     RPC endpoint (default: ${DEFAULT_RPC} or $AETHER_RPC)
  -p, --peers         Show detailed peer list
  -e, --epoch         Show epoch and consensus information
  -j, --json          Output raw JSON (good for scripting)
  -h, --help          Show this help message

${C.bright}Examples:${C.reset}
  aether-cli network                    # Summary view
  aether-cli network --json             # JSON output
  aether-cli network --rpc http://api.testnet.aether.network
  aether-cli network --peers            # Detailed peer list
  aether-cli network --epoch            # Epoch/consensus info
`.trim());
}

// ---------------------------------------------------------------------------
// Network data fetchers
// ---------------------------------------------------------------------------

/** GET /v1/slot — current network slot */
async function getSlot(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/slot');
    return res.slot ?? res.root_slot ?? null;
  } catch {
    return null;
  }
}

/** GET /v1/block_height — current network block height */
async function getBlockHeight(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/block_height');
    return res.block_height ?? null;
  } catch {
    return null;
  }
}

/** GET /v1/validators — list of validators / peers */
async function getValidators(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/validators');
    if (res.validators && Array.isArray(res.validators)) {
      return res.validators;
    }
    if (Array.isArray(res)) return res;
    return [];
  } catch {
    return [];
  }
}

/** GET /v1/epoch — current epoch and consensus info */
async function getEpoch(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/epoch');
    return res;
  } catch {
    return null;
  }
}

/** POST /v1/slot_production — slot production stats */
async function getSlotProduction(rpc) {
  try {
    // Try slot production endpoint
    const res = await httpPost(rpc, '/v1/slot_production', {});
    return res;
  } catch {
    return null;
  }
}

/** GET /v1/tps — TPS estimate from network */
async function getTPS(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/tps');
    return res.tps ?? res.tps_avg ?? res.transactions_per_second ?? null;
  } catch {
    return null;
  }
}

/** GET /v1/supply — token supply info */
async function getSupply(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/supply');
    return res;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function formatNumber(n) {
  if (n === null || n === undefined) return 'N/A';
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function peerHealthIcon(score) {
  if (score === undefined || score === null) return `${C.dim}●${C.reset}`;
  if (score >= 80) return `${C.green}●${C.reset}`;
  if (score >= 50) return `${C.yellow}●${C.reset}`;
  return `${C.red}●${C.reset}`;
}

function uptimeString(seconds) {
  if (!seconds) return `${C.dim}unknown${C.reset}`;
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const parts = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  return parts.length > 0 ? parts.join(' ') : `${seconds}s`;
}

// ---------------------------------------------------------------------------
// Main renderers
// ---------------------------------------------------------------------------

function renderSummary(data, rpc) {
  const { slot, blockHeight, peerCount, tps, supply, epochData } = data;
  const now = new Date().toLocaleTimeString();

  console.log(`
${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════╗${C.reset}
${C.bright}${C.cyan}║${C.reset}              ${C.bright}AETHER NETWORK STATUS${C.reset}${C.cyan}                            ║${C.reset}
${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════╝${C.reset}
`);
  console.log(`  ${C.dim}RPC:${C.reset} ${rpc}`);
  console.log(`  ${C.dim}Updated:${C.reset} ${now}`);
  console.log();

  // Network health
  const isHealthy = slot !== null && blockHeight !== null;
  const healthIcon = isHealthy ? `${C.green}● HEALTHY${C.reset}` : `${C.red}● UNHEALTHY${C.reset}`;
  console.log(`  Network ${healthIcon}`);
  console.log();

  // Key metrics
  console.log(`  ${C.bright}┌─────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Current Slot${C.reset}          ${C.bright}│${C.reset}  ${C.green}${formatNumber(slot).padEnd(20)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Block Height${C.reset}         ${C.bright}│${C.reset}  ${C.blue}${formatNumber(blockHeight).padEnd(20)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Active Peers${C.reset}         ${C.bright}│${C.reset}  ${C.magenta}${formatNumber(peerCount).padEnd(20)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Network TPS${C.reset}          ${C.bright}│${C.reset}  ${tps !== null ? (tps > 0 ? `${C.green}` : `${C.yellow}`) + tps.toFixed(2).padEnd(20) : `${C.dim}N/A`.padEnd(20)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}└─────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();

  // Epoch info if available
  if (epochData && (epochData.epoch !== undefined || epochData.absolute_slot !== undefined)) {
    console.log(`  ${C.bright}── Epoch / Consensus ──────────────────────────────────────────${C.reset}`);
    const ep = epochData.epoch !== undefined ? epochData.epoch : '?';
    const slotIndex = epochData.slot_index !== undefined ? epochData.slot_index : '?';
    const slotsInEpoch = epochData.slots_in_epoch !== undefined ? epochData.slots_in_epoch : '?';
    const progress = slotsInEpoch !== '?' && slotsInEpoch > 0
      ? ((slotIndex / slotsInEpoch) * 100).toFixed(1) + '%'
      : '?';

    console.log(`  ${C.dim}Epoch:${C.reset} ${C.bright}${ep}${C.reset}  ${C.dim}Slot in epoch:${C.reset} ${slotIndex}/${slotsInEpoch} ${C.dim}(${progress})${C.reset}`);
    if (epochData.absolute_slot !== undefined) {
      console.log(`  ${C.dim}Absolute slot:${C.reset} ${formatNumber(epochData.absolute_slot)}`);
    }
    if (epochData.block_height !== undefined) {
      console.log(`  ${C.dim}Block height:${C.reset} ${formatNumber(epochData.block_height)}`);
    }
    console.log();
  }

  // Supply if available
  if (supply && !supply.error) {
    console.log(`  ${C.bright}── Token Supply ───────────────────────────────────────────────${C.reset}`);
    if (supply.total !== undefined) {
      const totalAETH = (supply.total / 1e9).toFixed(2);
      console.log(`  ${C.dim}Total supply:${C.reset}  ${C.green}${formatNumber(supply.total)} lamports${C.reset} ${C.dim}(${totalAETH} AETH)${C.reset}`);
    }
    if (supply.circulating !== undefined) {
      const circAETH = (supply.circulating / 1e9).toFixed(2);
      console.log(`  ${C.dim}Circulating:${C.reset} ${formatNumber(supply.circulating)} lamports ${C.dim}(${circAETH} AETH)${C.reset}`);
    }
    console.log();
  }

  console.log(`  ${C.dim}Tip: --peers for peer list  |  --epoch for consensus  |  --json for raw data${C.reset}`);
  console.log();
}

function renderPeers(peers, rpc) {
  console.log();
  console.log(`${C.bright}${C.cyan}── Peer List ─────────────────────────────────────────────────${C.reset}`);
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
  console.log();

  if (!peers || peers.length === 0) {
    console.log(`  ${C.yellow}⚠ No peer information available from this RPC.${C.reset}`);
    console.log(`  ${C.dim}  Peers may not be exposed by your validator's RPC configuration.${C.reset}`);
    console.log();
    return;
  }

  console.log(`  ${C.bright}┌────────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset} ${C.cyan}#${C.reset}  ${C.cyan}Validator Address${C.reset}                        ${C.cyan}Tier${C.reset}   ${C.cyan}Score${C.reset}  ${C.cyan}Uptime${C.reset} ${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├${C.reset}${'-'.repeat(78)}${C.bright}│${C.reset}`);

  peers.slice(0, 50).forEach((peer, i) => {
    const num = (i + 1).toString().padStart(2);
    const addr = (peer.address || peer.pubkey || peer.id || 'unknown').slice(0, 32).padEnd(34);
    const tier = (peer.tier || peer.node_type || '?').toUpperCase().padEnd(6).slice(0, 6);
    const score = peer.score !== undefined ? peer.score : (peer.uptime !== undefined ? Math.round(peer.uptime * 100) : null);
    const scoreStr = score !== null ? `${score}%` : '?';
    const uptime = uptimeString(peer.uptime_seconds || peer.uptime);
    const scoreColor = score === null ? C.dim : score >= 80 ? C.green : score >= 50 ? C.yellow : C.red;

    console.log(
      `  ${C.bright}│${C.reset} ${C.dim}${num}${C.reset}  ${addr} ${tier.padEnd(6)} ${scoreColor}${(scoreStr + '%').padEnd(7)}${C.reset} ${C.dim}${uptime}${C.reset} ${C.bright}│${C.reset}`
    );
  });

  if (peers.length > 50) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}... and ${peers.length - 50} more peers (use --json for full list)${C.reset}`.padEnd(80) + `${C.bright}│${C.reset}`);
  }

  console.log(`  ${C.bright}└────────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();
  console.log(`  ${C.dim}Total peers: ${peers.length}${C.reset}`);
  console.log();
}

function renderEpoch(epochData, rpc) {
  console.log();
  console.log(`${C.bright}${C.cyan}── Epoch / Consensus ────────────────────────────────────────${C.reset}`);
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
  console.log();

  if (!epochData) {
    console.log(`  ${C.yellow}⚠ Epoch information not available.${C.reset}`);
    console.log(`  ${C.dim}  Is your validator fully synced?${C.reset}`);
    console.log();
    return;
  }

  const t = (label, val) => console.log(`  ${C.dim}${label}:${C.reset}  ${val !== undefined && val !== null ? C.bright + val : C.dim + 'N/A'}${C.reset}`);

  console.log(`  ${C.bright}┌─────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Epoch${C.reset}${' '.repeat(45)}${C.bright}│${C.reset}`);
  const ep = epochData.epoch !== undefined ? epochData.epoch : '?';
  console.log(`  ${C.bright}│${C.reset}  ${C.green}${(ep + '').padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├${C.reset}${'─'.repeat(58)}${C.bright}│${C.reset}`);

  if (epochData.slots_in_epoch !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Slots in epoch${C.reset}`.padEnd(53) + `${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  ${C.bright}${formatNumber(epochData.slots_in_epoch).padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
  }

  if (epochData.slot_index !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Current slot in epoch${C.reset}`.padEnd(53) + `${C.bright}│${C.reset}`);
    const progress = epochData.slots_in_epoch > 0
      ? `${epochData.slot_index} / ${epochData.slots_in_epoch} (${((epochData.slot_index / epochData.slots_in_epoch) * 100).toFixed(1)}%)`
      : `${epochData.slot_index}`;
    console.log(`  ${C.bright}│${C.reset}  ${C.bright}${progress.padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
  }

  if (epochData.absolute_slot !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Absolute slot${C.reset}`.padEnd(53) + `${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  ${C.bright}${formatNumber(epochData.absolute_slot).padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
  }

  if (epochData.block_height !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Block height${C.reset}`.padEnd(53) + `${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  ${C.bright}${formatNumber(epochData.block_height).padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
  }

  if (epochData.epoch_schedule) {
    const es = epochData.epoch_schedule;
    if (es.first_normal_epoch !== undefined) {
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}First normal epoch${C.reset}`.padEnd(53) + `${C.bright}│${C.reset}`);
      console.log(`  ${C.bright}│${C.reset}  ${C.bright}${es.first_normal_epoch.padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
    }
  }

  console.log(`  ${C.bright}└─────────────────────────────────────────────────────┘${C.reset}`);
  console.log();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();
  const rpc = opts.rpc;

  if (!opts.asJson) {
    console.log(`\n${C.cyan}Querying Aether network...${C.reset} ${C.dim}(${rpc})${C.reset}\n`);
  }

  // Fetch all data in parallel
  const [slot, blockHeight, validators, epochData, tps, supply] = await Promise.all([
    getSlot(rpc),
    getBlockHeight(rpc),
    getValidators(rpc),
    getEpoch(rpc),
    getTPS(rpc),
    getSupply(rpc),
  ]);

  const peerCount = Array.isArray(validators) ? validators.length : 0;

  const data = {
    slot,
    blockHeight,
    peerCount,
    tps,
    supply,
    epochData,
    validators,
    rpc,
    fetchedAt: new Date().toISOString(),
  };

  if (opts.asJson) {
    console.log(JSON.stringify(data, null, 2));
    return;
  }

  if (opts.showPeers) {
    renderPeers(validators, rpc);
  } else if (opts.showEpoch) {
    renderEpoch(epochData, rpc);
  } else {
    renderSummary(data, rpc);
  }
}

module.exports = { main, networkCommand: main };

if (require.main === module) {
  main().catch((err) => {
    console.error(`\n${C.red}✗ Network command failed:${C.reset} ${err.message}`);
    console.error(`  ${C.dim}Check that your validator is running and RPC is accessible.${C.reset}`);
    console.error(`  ${C.dim}Set custom RPC: AETHER_RPC=http://your-rpc-url${C.reset}\n`);
    process.exit(1);
  });
}
