#!/usr/bin/env node
/**
 * aether-cli ping
 *
 * Quick RPC health check — measures latency, verifies connectivity,
 * and reports node version and slot info.
 *
 * Usage:
 *   aether ping                   Ping default RPC (AETHER_RPC or localhost:8899)
 *   aether ping --rpc <url>       Ping a specific RPC endpoint
 *   aether ping --count <n>       Run <n> pings and show avg/min/max (default: 1, max 20)
 *   aether ping --json            JSON output for scripting/monitoring
 *
 * Examples:
 *   aether ping                          # Single ping, default RPC
 *   aether ping --rpc https://rpc.example.com  # Ping specific endpoint
 *   aether ping --count 5 --json         # 5 pings, JSON output for alerting
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
  cyan: '\x1b[36m',
};

const CLI_VERSION = '1.0.6';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

function httpRequest(rpcUrl, pathStr, timeoutMs = 8000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: timeoutMs,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
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

function httpPost(rpcUrl, pathStr, body, timeoutMs = 8000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: timeoutMs,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(bodyStr) },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.write(bodyStr);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Single ping: measure latency to /v1/slot
// ---------------------------------------------------------------------------

async function pingOnce(rpcUrl) {
  const start = Date.now();
  let slot = null;
  let error = null;
  let latencyMs = null;

  try {
    // Use POST to /v1/slot (some nodes only support POST)
    const result = await httpPost(rpcUrl, '/v1/slot', {}, 8000);
    latencyMs = Date.now() - start;

    if (result && result.error) {
      error = result.error;
    } else {
      slot = result.slot ?? result;
      if (typeof slot === 'object') slot = slot.slot;
    }
  } catch (err) {
    latencyMs = Date.now() - start;
    error = err.message;
  }

  return { latencyMs, slot, error, rpcUrl };
}

// ---------------------------------------------------------------------------
// Multi-ping: run N pings and aggregate
// ---------------------------------------------------------------------------

async function pingMulti(rpcUrl, count) {
  const results = [];
  for (let i = 0; i < count; i++) {
    results.push(await pingOnce(rpcUrl));
    if (i < count - 1) await new Promise(r => setTimeout(r, 100));
  }
  return results;
}

// ---------------------------------------------------------------------------
// Colour helpers
// ---------------------------------------------------------------------------

function latencyColor(ms) {
  if (ms === null) return C.red;
  if (ms < 50) return C.green;
  if (ms < 200) return C.cyan;
  if (ms < 500) return C.yellow;
  return C.red;
}

function latencyLabel(ms) {
  if (ms === null) return '✗ unreachable';
  if (ms < 50) return `● ${ms}ms  (excellent)`;
  if (ms < 200) return `● ${ms}ms  (good)`;
  if (ms < 500) return `○ ${ms}ms  (fair)`;
  return `○ ${ms}ms  (slow)`;
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printResult(ping, asJson) {
  const { latencyMs, slot, error, rpcUrl } = ping;

  if (asJson) {
    console.log(JSON.stringify({
      rpc: rpcUrl,
      online: error === null,
      latency_ms: latencyMs,
      slot,
      error: error || null,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }));
    return;
  }

  const lc = latencyColor(latencyMs);
  const bar = latencyMs !== null ? '█'.repeat(Math.min(10, Math.floor(latencyMs / 50))) : '▒';

  console.log(`  ${lc}${bar}${C.reset} ${C.bright}${latencyLabel(latencyMs)}${C.reset}`);
  if (slot !== null) {
    console.log(`  ${C.dim}  slot: ${C.reset}${C.cyan}${slot.toLocaleString()}${C.reset}`);
  }
  if (error) {
    console.log(`  ${C.red}  ✗ ${error}${C.reset}`);
  }
  console.log(`  ${C.dim}  rpc: ${rpcUrl}${C.reset}`);
}

function printAggregated(results, rpcUrl, asJson) {
  const online = results.filter(r => r.error === null);
  const failed = results.filter(r => r.error !== null);

  if (asJson) {
    const latencies = online.map(r => r.latencyMs).filter(Boolean);
    const avg = latencies.length > 0 ? Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length) : null;
    const min = latencies.length > 0 ? Math.min(...latencies) : null;
    const max = latencies.length > 0 ? Math.max(...latencies) : null;

    console.log(JSON.stringify({
      rpc: rpcUrl,
      count: results.length,
      online: online.length,
      failed: failed.length,
      latency_ms: { avg, min, max },
      slots: online.map(r => r.slot).filter(Boolean),
      errors: failed.map(r => r.error),
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }));
    return;
  }

  console.log(`\n${C.bright}${C.cyan}── Ping Results: ${rpcUrl} ──${C.reset}\n`);

  const latencies = online.map(r => r.latencyMs).filter(Boolean);

  if (latencies.length > 0) {
    const avg = Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length);
    const min = Math.min(...latencies);
    const max = Math.max(...latencies);

    console.log(`  ${C.green}✓${C.reset} ${online.length}/${results.length} successful\n`);

    // Per-ping bars
    for (let i = 0; i < online.length; i++) {
      const r = online[i];
      const lc = latencyColor(r.latencyMs);
      const bar = '█'.repeat(Math.min(10, Math.floor(r.latencyMs / 50)));
      const slotStr = r.slot !== null ? `  slot=${C.cyan}${r.slot.toLocaleString()}${C.reset}` : '';
      console.log(`    ${lc}${bar}${C.reset}  ${r.latencyMs}ms${slotStr}`);
    }

    console.log();
    console.log(`  ${C.bright}Latency:${C.reset} avg=${latencyColor(avg)}${avg}ms${C.reset}  min=${latencyColor(min)}${min}ms${C.reset}  max=${latencyColor(max)}${max}ms${C.reset}`);
    console.log(`  ${C.dim}  Packets: ${results.length}  Lost: ${failed.length}${C.reset}`);

    // Health assessment
    const healthPct = (online.length / results.length) * 100;
    if (healthPct === 100 && avg < 50) {
      console.log(`  ${C.green}  Health: excellent${C.reset}`);
    } else if (healthPct >= 80 && avg < 200) {
      console.log(`  ${C.cyan}  Health: good${C.reset}`);
    } else if (healthPct >= 60) {
      console.log(`  ${C.yellow}  Health: degraded${C.reset}`);
    } else {
      console.log(`  ${C.red}  Health: poor${C.reset}`);
    }
  } else {
    console.log(`  ${C.red}✗ All pings failed${C.reset}`);
    for (const r of failed) {
      console.log(`    ${C.red}✗ ${r.error}${C.reset}`);
    }
  }

  if (failed.length > 0 && online.length > 0) {
    console.log();
    console.log(`  ${C.yellow}⚠ ${failed.length} pings failed:${C.reset}`);
    for (const r of failed) {
      console.log(`    ${C.red}✗ ${r.error}${C.reset}`);
    }
  }
  console.log();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const args = process.argv.slice(3); // [node, index.js, ping, ...]

  const rpcIndex = args.findIndex(a => a === '--rpc' || a === '-r');
  const rpcUrl = rpcIndex !== -1 && args[rpcIndex + 1] && !args[rpcIndex + 1].startsWith('-')
    ? args[rpcIndex + 1]
    : getDefaultRpc();

  const countIndex = args.findIndex(a => a === '--count' || a === '-c');
  const countRaw = countIndex !== -1 && args[countIndex + 1] && !args[countIndex + 1].startsWith('-')
    ? parseInt(args[countIndex + 1], 10)
    : 1;
  const count = Math.min(Math.max(1, countRaw || 1), 20);

  const asJson = args.includes('--json') || args.includes('-j');

  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}── Aether RPC Ping ──────────────────────────────────────${C.reset}`);
    if (count > 1) {
      console.log(`  ${C.dim}Running ${count} pings against ${rpcUrl}…${C.reset}`);
    } else {
      console.log(`  ${C.dim}RPC: ${rpcUrl}${C.reset}`);
    }
    console.log();
  }

  if (count === 1) {
    const result = await pingOnce(rpcUrl);
    printResult(result, asJson);
    if (!asJson) console.log();
    // Exit 1 if unreachable
    if (result.error) process.exit(1);
  } else {
    const results = await pingMulti(rpcUrl, count);
    printAggregated(results, rpcUrl, asJson);
    // Exit 1 if all failed
    if (results.every(r => r.error)) process.exit(1);
  }
}

main().catch(err => {
  console.error(`\n${C.red}✗ Ping failed:${C.reset} ${err.message}\n`);
  process.exit(1);
});

module.exports = { pingCommand: main };

if (require.main === module) {
  main();
}
