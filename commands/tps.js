#!/usr/bin/env node
/**
 * aether-cli tps
 *
 * Monitor real-time transactions per second (TPS) on the Aether blockchain.
 * Uses @jellylegsai/aether-sdk for real RPC calls to /v1/tps endpoint.
 *
 * Usage:
 *   aether tps                     Show current TPS
 *   aether tps --monitor           Continuous monitoring (updates every 2s)
 *   aether tps --interval <sec>    Custom interval for monitoring (default: 2)
 *   aether tps --json              JSON output for scripting
 *   aether tps --rpc <url>         Custom RPC endpoint
 *
 * Examples:
 *   aether tps                          # Single TPS reading
 *   aether tps --monitor                # Live monitoring dashboard
 *   aether tps --monitor --interval 1   # Update every second
 *   aether tps --json                   # JSON for alerting/monitoring
 */

const path = require('path');
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

const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function tpsColor(tps) {
  if (tps === null || tps === undefined) return C.red;
  if (tps >= 1000) return C.green;
  if (tps >= 100) return C.cyan;
  if (tps >= 10) return C.yellow;
  return C.red;
}

function tpsLabel(tps) {
  if (tps === null || tps === undefined) return '✗ unreachable';
  if (tps >= 1000) return `● ${tps.toLocaleString()} TPS  (excellent)`;
  if (tps >= 100) return `● ${tps.toLocaleString()} TPS  (good)`;
  if (tps >= 10) return `○ ${tps.toLocaleString()} TPS  (fair)`;
  return `○ ${tps.toLocaleString()} TPS  (low)`;
}

// ---------------------------------------------------------------------------
// Single TPS reading
// ---------------------------------------------------------------------------

async function getTpsOnce(rpcUrl) {
  const client = createClient(rpcUrl);
  const start = Date.now();
  let tps = null;
  let error = null;
  let latencyMs = null;

  try {
    // Real RPC call: GET /v1/tps
    tps = await client.getTPS();
    latencyMs = Date.now() - start;
  } catch (err) {
    latencyMs = Date.now() - start;
    error = err.message;
  }

  return { tps, error, latencyMs, rpcUrl, timestamp: new Date() };
}

// ---------------------------------------------------------------------------
// Continuous monitoring
// ---------------------------------------------------------------------------

async function monitorTps(rpcUrl, intervalSec) {
  console.log(`\n${C.bright}${C.cyan}── Aether TPS Monitor ───────────────────────────────────${C.reset}`);
  console.log(`  ${C.dim}RPC: ${rpcUrl}${C.reset}`);
  console.log(`  ${C.dim}Interval: ${intervalSec}s${C.reset}`);
  console.log(`  ${C.dim}Press Ctrl+C to stop${C.reset}\n`);

  const history = [];
  const maxHistory = 20;

  process.on('SIGINT', () => {
    console.log(`\n${C.dim}Monitoring stopped.${C.reset}`);
    if (history.length > 1) {
      const tpsValues = history.map(h => h.tps).filter(t => t !== null);
      if (tpsValues.length > 0) {
        const avg = Math.round(tpsValues.reduce((a, b) => a + b, 0) / tpsValues.length);
        const min = Math.min(...tpsValues);
        const max = Math.max(...tpsValues);
        console.log(`  ${C.bright}Summary:${C.reset} avg=${avg}  min=${min}  max=${max}  samples=${history.length}`);
      }
    }
    console.log();
    process.exit(0);
  });

  while (true) {
    const result = await getTpsOnce(rpcUrl);
    history.push(result);
    if (history.length > maxHistory) history.shift();

    // Clear line and print
    process.stdout.write('\x1b[2K\r');
    
    const tc = tpsColor(result.tps);
    const barLen = result.tps !== null ? Math.min(30, Math.floor(result.tps / 50)) : 0;
    const bar = tc + '█'.repeat(barLen) + C.dim + '░'.repeat(30 - barLen) + C.reset;
    
    const slotInfo = result.tps !== null ? `  ${C.dim}RPC latency: ${result.latencyMs}ms${C.reset}` : '';
    
    console.log(`  ${tc}${bar}${C.reset}  ${tpsLabel(result.tps)}${C.reset}${slotInfo}`);
    
    // Show trend
    if (history.length >= 3) {
      const recent = history.slice(-3).map(h => h.tps).filter(t => t !== null);
      if (recent.length >= 2) {
        const trend = recent[recent.length - 1] - recent[0];
        const arrow = trend > 0 ? C.green + '▲' : trend < 0 ? C.red + '▼' : C.dim + '─';
        console.log(`  ${C.dim}Trend: ${arrow} ${Math.abs(trend)} TPS${C.reset}`);
      }
    }
    
    await new Promise(resolve => setTimeout(resolve, intervalSec * 1000));
  }
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function tpsCommand() {
  const args = process.argv.slice(2);
  const asJson = args.includes('--json') || args.includes('-j');
  const isMonitor = args.includes('--monitor') || args.includes('-m');
  const rpcIdx = args.findIndex(a => a === '--rpc' || a === '-r');
  const rpcUrl = rpcIdx !== -1 && args[rpcIdx + 1] ? args[rpcIdx + 1] : getDefaultRpc();
  
  const intervalIdx = args.findIndex(a => a === '--interval' || a === '-i');
  const intervalSec = intervalIdx !== -1 && args[intervalIdx + 1]
    ? Math.max(1, parseInt(args[intervalIdx + 1], 10) || 2)
    : 2;

  if (isMonitor) {
    await monitorTps(rpcUrl, intervalSec);
    return;
  }

  // Single reading
  const result = await getTpsOnce(rpcUrl);

  if (asJson) {
    console.log(JSON.stringify({
      rpc: rpcUrl,
      tps: result.tps,
      online: result.tps !== null,
      latency_ms: result.latencyMs,
      error: result.error || null,
      timestamp: result.timestamp.toISOString(),
      cli_version: CLI_VERSION,
    }, null, 2));
    return;
  }

  console.log(`\n${C.bright}${C.cyan}── Aether Network TPS ─────────────────────────────────${C.reset}\n`);

  const tc = tpsColor(result.tps);
  const barLen = result.tps !== null ? Math.min(40, Math.floor(result.tps / 50)) : 0;
  const bar = tc + '█'.repeat(barLen) + C.dim + '░'.repeat(40 - barLen) + C.reset;

  console.log(`  ${C.dim}RPC:${C.reset}       ${rpcUrl}`);
  console.log(`  ${C.dim}Latency:${C.reset}   ${result.latencyMs}ms`);
  console.log();
  console.log(`  ${C.bright}${tc}${tpsLabel(result.tps)}${C.reset}`);
  console.log();
  console.log(`  ${bar}`);
  console.log();

  if (result.error) {
    console.log(`  ${C.red}✗ ${result.error}${C.reset}`);
    console.log();
  }

  // Context info
  if (result.tps !== null) {
    console.log(`  ${C.dim}Network Health:${C.reset}`);
    if (result.tps >= 1000) {
      console.log(`    ${C.green}● Network is handling high throughput${C.reset}`);
    } else if (result.tps >= 100) {
      console.log(`    ${C.cyan}● Network operating normally${C.reset}`);
    } else if (result.tps >= 10) {
      console.log(`    ${C.yellow}○ Network has low activity${C.reset}`);
    } else {
      console.log(`    ${C.red}○ Network is idle or experiencing issues${C.reset}`);
    }
  }

  console.log();
  console.log(`  ${C.dim}Run ${C.cyan}aether tps --monitor${C.reset}${C.dim} for live tracking.${C.reset}\n`);

  if (result.tps === null) {
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

module.exports = { tpsCommand };

if (require.main === module) {
  tpsCommand().catch(err => {
    console.error(`\n${C.red}TPS command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
