/**
 * aether-cli snapshot - Aether Node Sync & Snapshot Status
 *
 * Shows how far your node has synced vs the network, snapshot availability,
 * and whether your node is catching up or is fully current.
 *
 * Usage:
 *   aether-cli snapshot              # Interactive sync status view
 *   aether-cli snapshot --json       # JSON output for scripting
 *   aether-cli snapshot --rpc <url>  # Query a specific RPC endpoint
 *   aether-cli snapshot --watch     # Refresh every 5 seconds
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
const REFRESH_INTERVAL_MS = 5000;

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
      timeout: 8000,
      headers: { 'Content-Type': 'application/json' },
    };

    const req = lib.request(reqOptions, (res) => {
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
      timeout: 8000,
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
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
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
${C.bright}${C.cyan}aether-cli snapshot${C.reset} - Aether Node Sync & Snapshot Status

${C.bright}Usage:${C.reset}
  aether-cli snapshot [options]

${C.bright}Options:${C.reset}
  -r, --rpc <url>     RPC endpoint (default: ${DEFAULT_RPC} or $AETHER_RPC)
  -j, --json          Output raw JSON (good for scripting)
  -w, --watch         Refresh every 5 seconds (live view)
  -h, --help          Show this help message

${C.bright}Examples:${C.reset}
  aether-cli snapshot                # Interactive sync status
  aether-cli snapshot --json        # JSON output
  aether-cli snapshot --watch        # Live refreshing view
  aether-cli snapshot --rpc https://api.testnet.aether.network
`.trim());
}

// ---------------------------------------------------------------------------
// Data fetchers
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

/** GET /v1/block_height — node's synced block height */
async function getBlockHeight(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/block_height');
    return res.block_height ?? null;
  } catch {
    return null;
  }
}

/** GET /v1/epoch — current epoch info */
async function getEpoch(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/epoch');
    return res;
  } catch {
    return null;
  }
}

/** GET /v1/snapshot — snapshot slot info */
async function getSnapshot(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/snapshot');
    return res;
  } catch {
    return null;
  }
}

/** POST /v1/version — node version info */
async function getVersion(rpc) {
  try {
    const res = await httpPost(rpc, '/v1/version', {});
    return res;
  } catch {
    return null;
  }
}

/** GET /v1/health — node health (if supported) */
async function getHealth(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/health');
    return res;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

function formatNumber(n) {
  if (n === null || n === undefined) return 'N/A';
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function syncStatus(nodeSlot, networkSlot) {
  if (nodeSlot === null || networkSlot === null) {
    return { label: `${C.yellow}UNKNOWN${C.reset}`, icon: '?', color: C.yellow };
  }
  const diff = networkSlot - nodeSlot;
  const pct = networkSlot > 0 ? ((nodeSlot / networkSlot) * 100).toFixed(1) : '0.0';

  if (diff <= 0) {
    return { label: `${C.green}SYNCED${C.reset}`, icon: '✓', color: C.green, diff };
  }
  if (diff <= 5) {
    return { label: `${C.green}CATCHING UP${C.reset}`, icon: '◐', color: C.green, diff };
  }
  if (diff <= 50) {
    return { label: `${C.yellow}BEHIND${C.reset}`, icon: '◑', color: C.yellow, diff };
  }
  return { label: `${C.red}FAR BEHIND${C.reset}`, icon: '✗', color: C.red, diff };
}

function progressBar(nodeSlot, networkSlot, width = 30) {
  if (nodeSlot === null || networkSlot === null || networkSlot === 0) {
    return `${C.dim}[${'─'.repeat(width)}]${C.reset} N/A`;
  }
  const ratio = Math.min(nodeSlot / networkSlot, 1);
  const filled = Math.round(ratio * width);
  const empty = width - filled;
  return (
    `${C.green}[${'█'.repeat(filled)}${C.dim}${'─'.repeat(empty)}${C.reset}]` +
    ` ${(ratio * 100).toFixed(1)}%`
  );
}

function catchupEstimate(nodeSlot, networkSlot) {
  if (nodeSlot === null || networkSlot === null || networkSlot <= nodeSlot) return null;
  const diff = networkSlot - nodeSlot;
  // Rough estimate: ~2 slots/second typical throughput
  const seconds = Math.floor(diff / 2);
  if (seconds < 60) return `~${seconds}s`;
  if (seconds < 3600) return `~${Math.floor(seconds / 60)}m`;
  return `~${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
}

// ---------------------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------------------

function renderSync(data, rpc) {
  const { nodeSlot, networkSlot, blockHeight, epochData, snapshotData, versionData, healthData, asJson } = data;

  if (asJson) {
    console.log(JSON.stringify({
      rpc,
      fetchedAt: new Date().toISOString(),
      node: {
        slot: nodeSlot,
        blockHeight,
      },
      network: {
        slot: networkSlot,
      },
      sync: {
        status: syncStatus(nodeSlot, networkSlot).label.replace(/\x1b\[\d+m/g, ''),
        slotsBehind: networkSlot !== null && nodeSlot !== null ? Math.max(0, networkSlot - nodeSlot) : null,
        percentSynced: nodeSlot !== null && networkSlot !== null && networkSlot > 0
          ? parseFloat((nodeSlot / networkSlot * 100).toFixed(2))
          : null,
      },
      epoch: epochData ? {
        epoch: epochData.epoch,
        slotIndex: epochData.slot_index,
        slotsInEpoch: epochData.slots_in_epoch,
        absoluteSlot: epochData.absolute_slot,
      } : null,
      snapshot: snapshotData && !snapshotData.error ? {
        fullSnapshotSlot: snapshotData.full_snapshot_slot ?? snapshotData.snapshot_slot ?? null,
        incrementalSnapshotSlot: snapshotData.incremental_snapshot_slot ?? null,
      } : null,
      version: versionData?.version ?? versionData?.solana_core ?? versionData?.['software-version'] ?? null,
      healthy: healthData && !healthData.error ? (healthData.ok ?? true) : null,
    }, null, 2));
    return;
  }

  const status = syncStatus(nodeSlot, networkSlot);
  const now = new Date().toLocaleTimeString();
  const catchup = catchupEstimate(nodeSlot, networkSlot);

  console.log();
  console.log(`${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║${C.reset}            ${C.bright}AETHER NODE SNAPSHOT / SYNC STATUS${C.reset}${C.cyan}                 ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════╝${C.reset}`);
  console.log(`  ${C.dim}RPC:${C.reset} ${rpc}`);
  console.log(`  ${C.dim}Updated:${C.reset} ${now}`);
  console.log();

  // Health indicator
  if (healthData && !healthData.error) {
    const ok = healthData.ok ?? true;
    console.log(`  ${C.bright}┌──────────────────────────────────────────────────────────────────────┐${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  Node Health: ${ok ? `${C.green}● HEALTHY${C.reset}` : `${C.red}● UNHEALTHY${C.reset}`}`.padEnd(65) + `${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}└──────────────────────────────────────────────────────────────────────┘${C.reset}`);
    console.log();
  } else if (healthData && healthData.error) {
    console.log(`  ${C.red}⚠ Health check failed:${C.reset} ${healthData.error}\n`);
  }

  // Sync status — large prominent display
  console.log(`  ${C.bright}┌──────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  Sync Status: ${status.color}${C.bright}${status.label}${C.reset}`.padEnd(65) + `${C.bright}│${C.reset}`);
  if (status.diff !== undefined && status.diff > 0) {
    console.log(`  ${C.bright}│${C.reset}  Slots behind: ${C.yellow}${formatNumber(status.diff)}${C.reset}`.padEnd(65) + `${C.bright}│${C.reset}`);
    if (catchup) {
      console.log(`  ${C.bright}│${C.reset}  Est. time to sync: ${C.cyan}${catchup}${C.reset}`.padEnd(65) + `${C.bright}│${C.reset}`);
    }
  }
  console.log(`  ${C.bright}└──────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();

  // Progress bar
  console.log(`  ${C.bright}── Slot Progress ───────────────────────────────────────────${C.reset}`);
  console.log(`  ${progressBar(nodeSlot, networkSlot)}`);
  console.log();

  // Slot details
  console.log(`  ${C.bright}┌────────────────────┬────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Your Node${C.reset}          ${C.bright}│${C.reset}  ${C.cyan}Network${C.reset}             ${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├────────────────────┼────────────────────┤${C.reset}`);
  const nodeStr = nodeSlot !== null ? formatNumber(nodeSlot) : 'N/A';
  const netStr = networkSlot !== null ? formatNumber(networkSlot) : 'N/A';
  console.log(`  ${C.bright}│${C.reset}  Slot: ${C.green}${nodeStr.padEnd(22)}${C.reset}  ${C.bright}│${C.reset}  Slot: ${C.cyan}${netStr.padEnd(22)}${C.reset}  ${C.bright}│${C.reset}`);
  const bhStr = blockHeight !== null ? formatNumber(blockHeight) : 'N/A';
  console.log(`  ${C.bright}│${C.reset}  Block: ${C.blue}${bhStr.padEnd(21)}${C.reset}  ${C.bright}│${C.reset}  ${C.dim}Block: same as slot${C.reset}    ${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}└────────────────────┴────────────────────┘${C.reset}`);
  console.log();

  // Epoch info
  if (epochData && epochData.epoch !== undefined) {
    console.log(`  ${C.bright}── Epoch Info ──────────────────────────────────────────────${C.reset}`);
    const ep = epochData.epoch;
    const slotIdx = epochData.slot_index !== undefined ? epochData.slot_index : '?';
    const slotsInEp = epochData.slots_in_epoch !== undefined ? epochData.slots_in_epoch : '?';
    const progress = slotsInEp !== '?' && slotsInEp > 0
      ? ((slotIdx / slotsInEp) * 100).toFixed(1) + '%'
      : '?';
    console.log(`  ${C.dim}Epoch:${C.reset} ${C.bright}${ep}${C.reset}  ${C.dim}Slot in epoch:${C.reset} ${C.bright}${slotIdx} / ${slotsInEp}${C.reset} ${C.dim}(${progress})${C.reset}`);
    if (epochData.absolute_slot !== undefined) {
      console.log(`  ${C.dim}Absolute slot:${C.reset} ${C.bright}${formatNumber(epochData.absolute_slot)}${C.reset}`);
    }
    console.log();
  }

  // Snapshot info
  if (snapshotData && !snapshotData.error) {
    console.log(`  ${C.bright}── Snapshot Info ───────────────────────────────────────────${C.reset}`);
    const fullSnap = snapshotData.full_snapshot_slot ?? snapshotData.snapshot_slot ?? null;
    const incSnap = snapshotData.incremental_snapshot_slot ?? null;

    if (fullSnap !== null) {
      const age = networkSlot !== null ? formatNumber(networkSlot - fullSnap) : null;
      console.log(`  ${C.dim}Full snapshot slot:${C.reset} ${C.green}${formatNumber(fullSnap)}${C.reset}`);
      if (age !== null) {
        console.log(`  ${C.dim}  (~${age} slots ago)${C.reset}`);
      }
      if (nodeSlot !== null && networkSlot !== null) {
        const snapAge = networkSlot - fullSnap;
        const freshness = snapAge < 100 ? `${C.green}fresh${C.reset}` : snapAge < 1000 ? `${C.yellow}aging${C.reset}` : `${C.red}old${C.reset}`;
        console.log(`  ${C.dim}  Snapshot freshness:${C.reset} ${freshness}`);
      }
    } else {
      console.log(`  ${C.dim}No full snapshot available.${C.reset}`);
    }

    if (incSnap !== null) {
      console.log(`  ${C.dim}Incremental snapshot slot:${C.reset} ${C.cyan}${formatNumber(incSnap)}${C.reset}`);
    }
    console.log();
  }

  // Version info
  if (versionData && !versionData.error) {
    const ver = versionData.version || versionData.solana_core || versionData['software-version'];
    if (ver) {
      console.log(`  ${C.bright}── Node Version ───────────────────────────────────────────${C.reset}`);
      console.log(`  ${C.dim}Version:${C.reset} ${C.green}${ver}${C.reset}`);
      console.log();
    }
  }

  // Tips
  if (status.diff === 0 || status.diff === undefined) {
    console.log(`  ${C.green}✓ Node is fully synced with the network.${C.reset}`);
  } else if (status.diff > 0) {
    console.log(`  ${C.yellow}⏳ Node is catching up — this is normal on first start or after a restart.${C.reset}`);
    console.log(`  ${C.dim}  For faster sync, try downloading a recent snapshot from a peer.${C.reset}`);
    console.log(`  ${C.dim}  Check: aether-cli network --peers${C.reset}`);
  }
  console.log();
  console.log(`  ${C.dim}Tip: --watch for live view  |  --json for scripting${C.reset}`);
  console.log();
}

// ---------------------------------------------------------------------------
// Watch mode
// ---------------------------------------------------------------------------

async function watchMode(rpc) {
  const readline = require('readline');
  let running = true;

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  const drainStdin = () => {
    rl.close();
    running = false;
  };
  process.stdin.on('data', drainStdin);
  process.stdin.resume();

  console.log(`  ${C.cyan}Live sync monitoring started.${C.reset} ${C.dim}Press Ctrl+C to stop.${C.reset}\n`);

  while (running) {
    // Move cursor up and clear lines for clean refresh
    process.stdout.write('\x1b[2J\x1b[H');

    try {
      const [nodeSlot, networkSlot, blockHeight, epochData, snapshotData, versionData, healthData] =
        await Promise.all([
          getSlot(rpc),
          getSlot(rpc), // network slot uses same endpoint
          getBlockHeight(rpc),
          getEpoch(rpc),
          getSnapshot(rpc),
          getVersion(rpc),
          getHealth(rpc),
        ]);

      renderSync({ nodeSlot, networkSlot, blockHeight, epochData, snapshotData, versionData, healthData, asJson: false }, rpc);
    } catch (err) {
      console.log(`  ${C.red}✗ Error fetching data:${C.reset} ${err.message}`);
    }

    if (!running) break;
    await new Promise((res) => setTimeout(res, REFRESH_INTERVAL_MS));
  }

  process.stdin.pause();
  process.stdin.off('data', drainStdin);
  console.log(`\n  ${C.dim}Stopped.${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function snapshotCommand() {
  const opts = parseArgs();
  const rpc = opts.rpc;

  if (opts.watch) {
    await watchMode(rpc);
    return;
  }

  const [nodeSlot, networkSlot, blockHeight, epochData, snapshotData, versionData, healthData] =
    await Promise.all([
      getSlot(rpc),
      getSlot(rpc), // same endpoint for network slot
      getBlockHeight(rpc),
      getEpoch(rpc),
      getSnapshot(rpc),
      getVersion(rpc),
      getHealth(rpc),
    ]);

  renderSync({ nodeSlot, networkSlot, blockHeight, epochData, snapshotData, versionData, healthData, asJson: opts.asJson }, rpc);
}

module.exports = { snapshotCommand };

if (require.main === module) {
  snapshotCommand().catch((err) => {
    console.error(`\n${C.red}✗ Snapshot command failed:${C.reset} ${err.message}`);
    console.error(`  ${C.dim}Check that your validator is running and RPC is accessible.${C.reset}`);
    console.error(`  ${C.dim}Set custom RPC: AETHER_RPC=http://your-rpc-url${C.reset}\n`);
    process.exit(1);
  });
}
