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
 * FULLY WIRED TO SDK - Uses @jellylegsai/aether-sdk for all blockchain calls.
 * No manual HTTP - all calls go through AetherClient with real RPC.
 *
 * Usage:
 *   aether network                    # Interactive summary view
 *   aether network --json             # JSON output for scripting
 *   aether network --rpc <url>        # Query a specific RPC endpoint
 *   aether network --peers            # Detailed peer list
 *   aether network --epoch            # Current epoch and consensus info
 *   aether network --wait             # Wait for node to sync
 *   aether network --ping             # Include latency measurements
 *
 * SDK Methods Used:
 *   - client.getSlot()              → GET /v1/slot
 *   - client.getBlockHeight()       → GET /v1/blockheight
 *   - client.getValidators()        → GET /v1/validators
 *   - client.getEpochInfo()         → GET /v1/epoch
 *   - client.getTPS()               → GET /v1/tps
 *   - client.getSupply()            → GET /v1/supply
 *   - client.getHealth()            → GET /v1/health
 *   - client.getVersion()           → GET /v1/version
 *   - client.getSlotProduction()    → POST /v1/slot_production
 *   - client.ping()                 → Health check with latency
 */

const path = require('path');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// Import UI framework
const { C, indicators, startSpinner, stopSpinner, drawBox, drawTable, 
        success, error, warning, info, code, highlight, value,
        formatHealth, formatLatency } = require('../lib/ui');

const DEFAULT_RPC = process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';

// ============================================================================
// SDK Client Setup
// ============================================================================

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: DEFAULT_RPC,
    showPeers: false,
    showEpoch: false,
    asJson: false,
    wait: false,
    doPing: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '-r':
      case '--rpc':
        options.rpc = args[++i];
        break;
      case '-p':
      case '--peers':
        options.showPeers = true;
        break;
      case '-e':
      case '--epoch':
        options.showEpoch = true;
        break;
      case '-j':
      case '--json':
        options.asJson = true;
        break;
      case '-w':
      case '--wait':
        options.wait = true;
        break;
      case '--ping':
        options.doPing = true;
        break;
      case '-h':
      case '--help':
        showHelp();
        process.exit(0);
    }
  }

  return options;
}

function showHelp() {
  console.log(`
${C.cyan}${C.bright}network${C.reset} — Aether Network Status

${C.bright}USAGE${C.reset}
    aether network [options]

${C.bright}OPTIONS${C.reset}
    -r, --rpc <url>      RPC endpoint (default: ${DEFAULT_RPC})
    -p, --peers          Show detailed peer list
    -e, --epoch          Show epoch and consensus information
    -j, --json           Output raw JSON for scripting
    -w, --wait           Wait for node to sync
    --ping               Include latency measurements
    -h, --help           Show this help message

${C.bright}SDK METHODS${C.reset}
    getSlot(), getBlockHeight(), getValidators(), getEpochInfo()
    getTPS(), getSupply(), getHealth(), getVersion(), getSlotProduction()

${C.bright}EXAMPLES${C.reset}
    aether network              # Summary view
    aether network --json       # JSON output
    aether network --peers      # Detailed peer list
    aether network --epoch      # Epoch info
    aether network --rpc http://my-rpc:8899
`);
}

// ============================================================================
// SDK Data Fetchers - REAL RPC CALLS
// ============================================================================

async function fetchNetworkData(rpc, asJson) {
  const client = createClient(rpc);
  
  const startTime = Date.now();
  
  if (!asJson) {
    startSpinner('Querying network via SDK');
  }

  const results = await Promise.allSettled([
    client.getSlot().catch(() => null),
    client.getBlockHeight().catch(() => null),
    client.getValidators().catch(() => []),
    client.getEpochInfo().catch(() => null),
    client.getTPS().catch(() => null),
    client.getSupply().catch(() => null),
    client.getHealth().catch(() => null),
    client.getVersion().catch(() => null),
    client.getSlotProduction().catch(() => null),
    aether.ping(rpc).catch(() => ({ ok: false, latency: null })),
  ]);

  const latency = Date.now() - startTime;

  if (!asJson) {
    stopSpinner(true, 'Network data retrieved');
  }

  const [
    slot,
    blockHeight,
    validators,
    epochInfo,
    tps,
    supply,
    health,
    version,
    slotProduction,
    pingResult,
  ] = results.map(r => r.status === 'fulfilled' ? r.value : null);

  return {
    slot,
    blockHeight,
    validators: Array.isArray(validators) ? validators : [],
    epochInfo,
    tps,
    supply,
    health,
    version,
    slotProduction,
    pingResult,
    latency,
    rpc,
    fetchedAt: new Date().toISOString(),
  };
}

// ============================================================================
// Format Helpers
// ============================================================================

function formatNumber(n) {
  if (n === null || n === undefined) return `${C.dim}N/A${C.reset}`;
  return n.toLocaleString();
}

function formatAether(lamports) {
  if (!lamports && lamports !== 0) return `${C.dim}N/A${C.reset}`;
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
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

function statusColor(status) {
  const s = (status || '').toLowerCase();
  if (s === 'active' || s === 'ok' || s === 'healthy') return C.green;
  if (s === 'delinquent' || s === 'error') return C.red;
  if (s === 'inactive' || s === 'syncing') return C.yellow;
  return C.dim;
}

// ============================================================================
// Output Renderers
// ============================================================================

function renderSummary(data) {
  const { slot, blockHeight, validators, epochInfo, tps, supply, health, version, pingResult, latency } = data;
  const peerCount = validators.length;
  
  // Health status
  const isHealthy = health === 'ok' || health === 'healthy';
  const healthStatus = isHealthy ? 
    `${C.green}${indicators.success} Healthy${C.reset}` : 
    health ? `${C.yellow}${indicators.warning} ${health}${C.reset}` : 
    `${C.red}${indicators.error} Unknown${C.reset}`;

  // Version string
  const versionStr = version ? 
    (version.aetherCore || version.featureSet || JSON.stringify(version)) : 
    `${C.dim}unknown${C.reset}`;

  console.log();
  console.log(drawBox(
    `
${C.bright}AETHER NETWORK STATUS${C.reset}    ${C.dim}${data.fetchedAt}${C.reset}

${C.cyan}Health:${C.reset}      ${healthStatus}
${C.cyan}RPC:${C.reset}         ${data.rpc}
${C.cyan}Version:${C.reset}     ${versionStr}
${C.cyan}Latency:${C.reset}     ${formatLatency(pingResult?.latency || latency)}

${C.cyan}Current Slot:${C.reset}     ${highlight(formatNumber(slot))}
${C.cyan}Block Height:${C.reset}    ${C.green}${formatNumber(blockHeight)}${C.reset}
${C.cyan}Active Peers:${C.reset}     ${C.magenta}${formatNumber(peerCount)}${C.reset}
${C.cyan}TPS:${C.reset}              ${tps !== null ? `${C.cyan}${tps.toFixed(2)}${C.reset}` : `${C.dim}N/A${C.reset}`}

${epochInfo ? `${C.cyan}Epoch:${C.reset}            ${C.bright}${epochInfo.epoch}${C.reset} (${formatNumber(epochInfo.slotIndex)}/${formatNumber(epochInfo.slotsInEpoch)} slots)` : ''}
${supply ? `${C.cyan}Total Supply:${C.reset}     ${C.green}${formatAether(supply.total)}${C.reset}` : ''}

${C.dim}SDK: @jellylegsai/aether-sdk${C.reset}
`.trim(),
    { style: 'double', title: 'AETHER NETWORK', titleColor: C.cyan + C.bright }
  ));

  console.log();
}

function renderPeers(validators, rpc) {
  if (!validators || validators.length === 0) {
    console.log(`\n  ${warning('No peer information available')}`);
    console.log(`  ${C.dim}Peers may not be exposed by your validator's RPC configuration.${C.reset}\n`);
    return;
  }

  const rows = validators.slice(0, 50).map((v, i) => {
    const addr = (v.address || v.pubkey || v.id || v.vote_account || 'unknown').slice(0, 32);
    const tier = (v.tier || v.node_type || 'unknown').toUpperCase();
    const stake = formatAether(v.stake_lamports || v.stake || v.activated_stake || 0);
    const status = v.status || 'active';
    const statusCol = statusColor(status);
    
    return [
      `${statusCol}●${C.reset}`,
      `${i + 1}`,
      addr,
      tier,
      stake,
    ];
  });

  console.log();
  console.log(drawTable(
    ['', '#', 'Validator', 'Tier', 'Stake'],
    rows,
    { borderStyle: 'single', headerColor: C.cyan + C.bright }
  ));

  if (validators.length > 50) {
    console.log(`\n  ${C.dim}... and ${validators.length - 50} more validators (use --json for full list)${C.reset}`);
  }

  console.log();
  console.log(`  ${C.bright}Total validators:${C.reset} ${C.magenta}${validators.length}${C.reset}`);
  console.log();
}

function renderEpoch(epochInfo) {
  console.log();
  
  if (!epochInfo) {
    console.log(`\n  ${warning('Epoch information not available')}`);
    console.log(`  ${C.dim}Is your validator fully synced?${C.reset}\n`);
    return;
  }

  const progress = epochInfo.slotsInEpoch > 0 ?
    ((epochInfo.slotIndex / epochInfo.slotsInEpoch) * 100).toFixed(2) : '0.00';

  console.log(drawBox(
    `
${C.bright}Epoch ${epochInfo.epoch}${C.reset}

${C.cyan}Slots in Epoch:${C.reset}     ${formatNumber(epochInfo.slotsInEpoch)}
${C.cyan}Current Slot:${C.reset}       ${formatNumber(epochInfo.slotIndex)}
${C.cyan}Progress:${C.reset}          ${highlight(progress + '%')}
${C.cyan}Blocks Remaining:${C.reset}  ${formatNumber(epochInfo.slotsInEpoch - epochInfo.slotIndex)}

${epochInfo.absoluteSlot ? `${C.cyan}Absolute Slot:${C.reset}     ${formatNumber(epochInfo.absoluteSlot)}` : ''}
${epochInfo.blockHeight ? `${C.cyan}Block Height:${C.reset}      ${formatNumber(epochInfo.blockHeight)}` : ''}
    `.trim(),
    { style: 'single', title: 'EPOCH INFO', titleColor: C.cyan }
  ));

  // Progress bar
  const barWidth = 40;
  const filled = Math.floor((progress / 100) * barWidth);
  const empty = barWidth - filled;
  const progressBar = C.green + '█'.repeat(filled) + C.dim + '░'.repeat(empty) + C.reset;
  
  console.log(`\n  ${progressBar} ${C.bright}${progress}%${C.reset}\n`);
}

function renderJson(data) {
  console.log(JSON.stringify(data, (key, value) => {
    if (typeof value === 'bigint') return value.toString();
    return value;
  }, 2));
}

// ============================================================================
// Main Command
// ============================================================================

async function networkCommand() {
  const opts = parseArgs();
  
  try {
    const data = await fetchNetworkData(opts.rpc, opts.asJson);

    if (opts.asJson) {
      renderJson(data);
      return;
    }

    if (opts.showPeers) {
      renderPeers(data.validators, opts.rpc);
    } else if (opts.showEpoch) {
      renderEpoch(data.epochInfo);
    } else {
      renderSummary(data);
    }

  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc: opts.rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${error('Network query failed')}`);
      console.log(`  ${C.dim}${err.message}${C.reset}\n`);
      console.log(`  ${C.bright}Troubleshooting:${C.reset}`);
      console.log(`    • Is your validator running? ${code('aether ping')}`);
      console.log(`    • Check RPC endpoint: ${C.dim}${opts.rpc}${C.reset}`);
      console.log(`    • Set custom RPC: ${C.dim}AETHER_RPC=https://your-rpc-url${C.reset}`);
      console.log();
    }
    process.exit(1);
  }
}

module.exports = { networkCommand, main: networkCommand };

if (require.main === module) {
  networkCommand().catch(err => {
    console.error(`\n  ${error('Network command failed')}: ${err.message}\n`);
    process.exit(1);
  });
}
