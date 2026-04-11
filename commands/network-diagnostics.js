#!/usr/bin/env node
/**
 * aether-cli network-diagnostics
 *
 * Comprehensive network diagnostics with automatic RPC failover,
 * latency testing, and intelligent error recovery.
 *
 * Features:
 *   - Multi-RPC endpoint health checking with automatic failover
 *   - Latency benchmarking across endpoints
 *   - Connection quality scoring
 *   - Automatic RPC selection with --auto flag
 *   - Detailed error classification and recovery suggestions
 *
 * Usage:
 *   aether network-diagnostics                    # Test default RPC
 *   aether network-diagnostics --auto              # Auto-select best RPC
 *   aether network-diagnostics --rpc <url>       # Test specific RPC
 *   aether network-diagnostics --benchmark         # Latency benchmark mode
 *   aether network-diagnostics --all             # Test all known endpoints
 *
 * SDK wired to:
 *   - client.getHealth()        → GET /v1/health
 *   - client.getSlot()          → GET /v1/slot
 *   - client.getVersion()       → GET /v1/version
 *   - client.getEpochInfo()       → GET /v1/epoch
 *   - client.ping()             → Latency test
 */

const path = require('path');
const readline = require('readline');
const fs = require('fs');
const os = require('os');

// Import SDK
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
  blue: '\x1b[34m',
};

const CLI_VERSION = '1.0.0';

// Known RPC endpoints for failover
const KNOWN_RPC_ENDPOINTS = [
  { url: 'http://127.0.0.1:8899', name: 'Local Node', priority: 1 },
  { url: 'http://localhost:8899', name: 'Local Node (alt)', priority: 2 },
  { url: process.env.AETHER_RPC, name: 'Env AETHER_RPC', priority: 0, conditional: true },
];

// Filter out null/undefined endpoints
function getEffectiveEndpoints() {
  const endpoints = [];
  const seen = new Set();

  for (const ep of KNOWN_RPC_ENDPOINTS) {
    if (ep.conditional && !ep.url) continue;
    if (!ep.url) continue;
    if (seen.has(ep.url)) continue;
    seen.add(ep.url);
    endpoints.push(ep);
  }

  return endpoints.sort((a, b) => a.priority - b.priority);
}

// ============================================================================
// Config & Paths
// ============================================================================

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function loadConfig() {
  if (!fs.existsSync(getConfigPath())) {
    return { preferredRpc: null, rpcHistory: [] };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { preferredRpc: null, rpcHistory: [] };
  }
}

function saveConfig(cfg) {
  if (!fs.existsSync(getAetherDir())) {
    fs.mkdirSync(getAetherDir(), { recursive: true });
  }
  fs.writeFileSync(getConfigPath(), JSON.stringify(cfg, null, 2));
}

function updateRpcHistory(rpcUrl, latency, success) {
  const cfg = loadConfig();
  cfg.rpcHistory = cfg.rpcHistory || [];

  // Add entry
  cfg.rpcHistory.unshift({
    url: rpcUrl,
    latency,
    success,
    timestamp: new Date().toISOString(),
  });

  // Keep last 50 entries
  cfg.rpcHistory = cfg.rpcHistory.slice(0, 50);

  // Update preferred RPC if this one is good
  if (success && latency < 100) {
    cfg.preferredRpc = rpcUrl;
  }

  saveConfig(cfg);
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    rpc: process.env.AETHER_RPC || 'http://127.0.0.1:8899',
    auto: false,
    benchmark: false,
    all: false,
    json: false,
    timeout: 10000,
    help: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--rpc':
      case '-r':
        opts.rpc = args[++i];
        break;
      case '--auto':
      case '-a':
        opts.auto = true;
        break;
      case '--benchmark':
      case '-b':
        opts.benchmark = true;
        break;
      case '--all':
        opts.all = true;
        break;
      case '--json':
      case '-j':
        opts.json = true;
        break;
      case '--timeout':
      case '-t':
        opts.timeout = parseInt(args[++i], 10) || 10000;
        break;
      case '--help':
      case '-h':
        opts.help = true;
        break;
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli network-diagnostics${C.reset} - Network health & RPC diagnostics

${C.bright}USAGE${C.reset}
    aether network-diagnostics [options]

${C.bright}OPTIONS${C.reset}
    --rpc <url>         Test specific RPC endpoint
    --auto, -a          Auto-select best performing RPC
    --benchmark, -b     Run latency benchmark
    --all               Test all known RPC endpoints
    --json, -j          Output JSON
    --timeout <ms>      Request timeout (default: 10000)
    --help, -h          Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getHealth()      → GET /v1/health
    client.getSlot()        → GET /v1/slot
    client.getVersion()     → GET /v1/version
    client.getEpochInfo()   → GET /v1/epoch
    client.ping()           → Latency measurement

${C.bright}EXAMPLES${C.reset}
    aether network-diagnostics                    # Test default RPC
    aether network-diagnostics --auto             # Find best RPC
    aether network-diagnostics --benchmark        # Latency test
    aether network-diagnostics --all --json       # Test all, JSON output
`);
}

// ============================================================================
// Diagnostics Logic
// ============================================================================

/**
 * Test a single RPC endpoint comprehensively
 */
async function testEndpoint(url, timeout = 10000) {
  const results = {
    url,
    timestamp: new Date().toISOString(),
    tests: {},
    overall: {
      success: false,
      latency: null,
      score: 0,
    },
  };

  const client = new aether.AetherClient({ rpcUrl: url, timeoutMs: timeout });
  const startTime = Date.now();

  try {
    // Test 1: Basic connectivity (ping)
    const pingStart = Date.now();
    const pingResult = await aether.ping(url);
    results.tests.ping = {
      success: pingResult.ok,
      latency: pingResult.latency,
      error: pingResult.ok ? null : pingResult.error,
    };

    if (!pingResult.ok) {
      throw new Error(`Ping failed: ${pingResult.error}`);
    }

    // Test 2: Health check
    const healthStart = Date.now();
    try {
      const health = await client.getHealth();
      results.tests.health = {
        success: true,
        latency: Date.now() - healthStart,
        status: health,
      };
    } catch (err) {
      results.tests.health = {
        success: false,
        latency: Date.now() - healthStart,
        error: err.message,
      };
    }

    // Test 3: Slot query
    const slotStart = Date.now();
    try {
      const slot = await client.getSlot();
      results.tests.slot = {
        success: true,
        latency: Date.now() - slotStart,
        slot,
      };
    } catch (err) {
      results.tests.slot = {
        success: false,
        latency: Date.now() - slotStart,
        error: err.message,
      };
    }

    // Test 4: Version info
    const versionStart = Date.now();
    try {
      const version = await client.getVersion();
      results.tests.version = {
        success: true,
        latency: Date.now() - versionStart,
        version,
      };
    } catch (err) {
      results.tests.version = {
        success: false,
        latency: Date.now() - versionStart,
        error: err.message,
      };
    }

    // Test 5: Epoch info
    const epochStart = Date.now();
    try {
      const epoch = await client.getEpochInfo();
      results.tests.epoch = {
        success: true,
        latency: Date.now() - epochStart,
        epoch,
      };
    } catch (err) {
      results.tests.epoch = {
        success: false,
        latency: Date.now() - epochStart,
        error: err.message,
      };
    }

    // Calculate overall stats
    const totalTime = Date.now() - startTime;
    const successfulTests = Object.values(results.tests).filter(t => t.success).length;
    const totalTests = Object.keys(results.tests).length;

    results.overall = {
      success: successfulTests === totalTests,
      latency: totalTime,
      score: Math.round((successfulTests / totalTests) * 100),
      successfulTests,
      totalTests,
    };

    // Update history
    updateRpcHistory(url, totalTime, results.overall.success);

    return results;

  } catch (err) {
    const totalTime = Date.now() - startTime;
    results.overall = {
      success: false,
      latency: totalTime,
      score: 0,
      error: err.message,
    };
    updateRpcHistory(url, totalTime, false);
    return results;
  } finally {
    client.destroy();
  }
}

/**
 * Run benchmark on an endpoint
 */
async function benchmarkEndpoint(url, iterations = 5) {
  const latencies = [];
  const errors = [];

  for (let i = 0; i < iterations; i++) {
    const start = Date.now();
    try {
      const result = await aether.ping(url);
      if (result.ok) {
        latencies.push(result.latency);
      } else {
        errors.push(result.error);
      }
    } catch (err) {
      errors.push(err.message);
    }
    // Small delay between pings
    if (i < iterations - 1) {
      await new Promise(r => setTimeout(r, 100));
    }
  }

  if (latencies.length === 0) {
    return {
      url,
      success: false,
      errors,
    };
  }

  const sorted = [...latencies].sort((a, b) => a - b);
  const sum = latencies.reduce((a, b) => a + b, 0);

  return {
    url,
    success: true,
    iterations,
    min: sorted[0],
    max: sorted[sorted.length - 1],
    avg: Math.round(sum / latencies.length),
    median: sorted[Math.floor(sorted.length / 2)],
    p95: sorted[Math.floor(sorted.length * 0.95)] || sorted[sorted.length - 1],
    latencies,
    errors: errors.length > 0 ? errors : undefined,
  };
}

/**
 * Find best RPC endpoint automatically
 */
async function findBestEndpoint(endpoints, timeout = 10000) {
  const results = [];

  // Test all endpoints in parallel
  const tests = endpoints.map(ep =>
    testEndpoint(ep.url, timeout).then(result => ({
      ...result,
      name: ep.name,
    }))
  );

  const settled = await Promise.allSettled(tests);

  for (const result of settled) {
    if (result.status === 'fulfilled') {
      results.push(result.value);
    }
  }

  // Sort by score (desc), then by latency (asc)
  results.sort((a, b) => {
    if (b.overall.score !== a.overall.score) {
      return b.overall.score - a.overall.score;
    }
    return a.overall.latency - b.overall.latency;
  });

  return results;
}

// ============================================================================
// Output Formatters
// ============================================================================

function getScoreColor(score) {
  if (score >= 80) return C.green;
  if (score >= 50) return C.yellow;
  return C.red;
}

function getLatencyColor(latency) {
  if (latency < 50) return C.green;
  if (latency < 200) return C.yellow;
  return C.red;
}

function formatLatency(ms) {
  if (ms === null || ms === undefined) return 'N/A';
  if (ms < 1) return '<1ms';
  return `${Math.round(ms)}ms`;
}

function printResult(result, verbose = false) {
  const scoreColor = getScoreColor(result.overall.score);
  const status = result.overall.success ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`;

  console.log(`\n  ${status} ${C.bright}${result.url}${C.reset}`);

  if (result.name) {
    console.log(`     ${C.dim}Name: ${result.name}${C.reset}`);
  }

  console.log(`     ${C.dim}Score:${C.reset} ${scoreColor}${result.overall.score}%${C.reset}`);
  console.log(`     ${C.dim}Total Time:${C.reset} ${getLatencyColor(result.overall.latency)}${formatLatency(result.overall.latency)}${C.reset}`);

  if (result.overall.successfulTests !== undefined) {
    console.log(`     ${C.dim}Tests Passed:${C.reset} ${result.overall.successfulTests}/${result.overall.totalTests}`);
  }

  if (verbose && result.tests) {
    console.log(`\n     ${C.dim}── Test Details ──${C.reset}`);
    for (const [name, test] of Object.entries(result.tests)) {
      const testStatus = test.success ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`;
      const latency = test.latency !== undefined ? ` (${formatLatency(test.latency)})` : '';
      console.log(`       ${testStatus} ${name}${latency}`);
      if (test.error) {
        console.log(`         ${C.red}${test.error}${C.reset}`);
      }
    }
  }

  if (result.overall.error) {
    console.log(`     ${C.red}Error: ${result.overall.error}${C.reset}`);
  }
}

function printBenchmarkResult(result) {
  const status = result.success ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`;
  console.log(`\n  ${status} ${C.bright}${result.url}${C.reset}`);

  if (!result.success) {
    console.log(`     ${C.red}Failed: ${result.errors?.join(', ') || 'Unknown error'}${C.reset}`);
    return;
  }

  const avgColor = getLatencyColor(result.avg);
  console.log(`     ${C.dim}Iterations:${C.reset} ${result.iterations}`);
  console.log(`     ${C.dim}Min:${C.reset} ${getLatencyColor(result.min)}${formatLatency(result.min)}${C.reset}`);
  console.log(`     ${C.dim}Max:${C.reset} ${getLatencyColor(result.max)}${formatLatency(result.max)}${C.reset}`);
  console.log(`     ${C.dim}Avg:${C.reset} ${avgColor}${formatLatency(result.avg)}${C.reset}`);
  console.log(`     ${C.dim}Median:${C.reset} ${getLatencyColor(result.median)}${formatLatency(result.median)}${C.reset}`);
  console.log(`     ${C.dim}P95:${C.reset} ${getLatencyColor(result.p95)}${formatLatency(result.p95)}${C.reset}`);

  if (result.errors && result.errors.length > 0) {
    console.log(`     ${C.yellow}Errors: ${result.errors.length}${C.reset}`);
  }
}

function printRecommendations(results, bestResult) {
  console.log(`\n${C.cyan}── Recommendations ──${C.reset}\n`);

  if (!bestResult || !bestResult.overall.success) {
    console.log(`  ${C.red}⚠ No healthy RPC endpoints found${C.reset}\n`);
    console.log(`  ${C.dim}Suggestions:${C.reset}`);
    console.log(`    1. Check if your local validator is running: ${C.cyan}aether validator start${C.reset}`);
    console.log(`    2. Verify RPC URL is correct`);
    console.log(`    3. Check firewall settings for port 8899`);
    console.log(`    4. Set custom RPC: ${C.cyan}export AETHER_RPC=http://your-rpc:8899${C.reset}`);
    return;
  }

  console.log(`  ${C.green}✓ Best RPC:${C.reset} ${C.bright}${bestResult.url}${C.reset}`);
  console.log(`     ${C.dim}Latency: ${formatLatency(bestResult.overall.latency)} | Score: ${bestResult.overall.score}%${C.reset}\n`);

  // Check for issues
  const failedEndpoints = results.filter(r => !r.overall.success);
  if (failedEndpoints.length > 0) {
    console.log(`  ${C.yellow}⚠ ${failedEndpoints.length} endpoint(s) unavailable${C.reset}`);
    console.log(`    ${C.dim}Run with --verbose for details${C.reset}\n`);
  }

  // Network health
  const avgScore = results.reduce((sum, r) => sum + (r.overall.score || 0), 0) / results.length;
  if (avgScore < 50) {
    console.log(`  ${C.yellow}⚠ Network health is degraded${C.reset}`);
    console.log(`    ${C.dim}Average score: ${avgScore.toFixed(0)}%${C.reset}\n`);
  }
}

// ============================================================================
// Main Command
// ============================================================================

async function networkDiagnosticsCommand() {
  const opts = parseArgs();

  if (opts.help) {
    showHelp();
    return;
  }

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════╗`);
    console.log(`${C.bright}${C.cyan}║         AETHER NETWORK DIAGNOSTICS                           ║`);
    console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════╝${C.reset}\n`);
  }

  let results = [];
  let bestResult = null;

  try {
    if (opts.benchmark) {
      // Benchmark mode
      if (!opts.json) {
        console.log(`${C.dim}Running latency benchmark...${C.reset}\n`);
      }

      const endpoints = opts.all ? getEffectiveEndpoints() : [{ url: opts.rpc, name: 'Custom' }];
      const benchmarks = [];

      for (const ep of endpoints) {
        if (!opts.json) {
          process.stdout.write(`  Testing ${ep.url}... `);
        }
        const result = await benchmarkEndpoint(ep.url);
        benchmarks.push(result);
        if (!opts.json) {
          console.log(result.success ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`);
        }
      }

      if (opts.json) {
        console.log(JSON.stringify({ benchmarks, timestamp: new Date().toISOString() }, null, 2));
      } else {
        console.log(`\n${C.cyan}── Benchmark Results ──${C.reset}`);
        for (const result of benchmarks) {
          printBenchmarkResult(result);
        }

        // Summary
        const successful = benchmarks.filter(b => b.success);
        if (successful.length > 0) {
          const fastest = successful.sort((a, b) => a.avg - b.avg)[0];
          console.log(`\n${C.green}✓ Fastest RPC:${C.reset} ${C.bright}${fastest.url}${C.reset} (${formatLatency(fastest.avg)} avg)`);
        }
      }

      return;
    }

    if (opts.auto) {
      // Auto-select mode
      const endpoints = getEffectiveEndpoints();

      if (!opts.json) {
        console.log(`${C.dim}Testing ${endpoints.length} RPC endpoint(s)...${C.reset}\n`);
      }

      results = await findBestEndpoint(endpoints, opts.timeout);

      if (results.length > 0) {
        bestResult = results[0];
      }

      if (opts.json) {
        console.log(JSON.stringify({
          results,
          recommended: bestResult?.url || null,
          timestamp: new Date().toISOString(),
        }, null, 2));
      } else {
        console.log(`${C.cyan}── Test Results ──${C.reset}`);
        for (const result of results) {
          printResult(result);
        }
        printRecommendations(results, bestResult);
      }

      // Update config with best RPC
      if (bestResult?.overall?.success) {
        const cfg = loadConfig();
        cfg.preferredRpc = bestResult.url;
        saveConfig(cfg);

        if (!opts.json) {
          console.log(`\n  ${C.green}✓ Saved preferred RPC to config${C.reset}`);
          console.log(`    ${C.dim}Set AETHER_RPC=${bestResult.url} to use this endpoint${C.reset}\n`);
        }
      }

      return;
    }

    // Single RPC test mode
    if (!opts.json) {
      console.log(`${C.dim}Testing RPC endpoint...${C.reset}`);
      console.log(`  ${C.dim}URL: ${opts.rpc}${C.reset}`);
      console.log(`  ${C.dim}Timeout: ${opts.timeout}ms${C.reset}\n`);
    }

    const result = await testEndpoint(opts.rpc, opts.timeout);
    results = [result];
    bestResult = result.overall.success ? result : null;

    if (opts.json) {
      console.log(JSON.stringify(result, null, 2));
    } else {
      printResult(result, true);

      // Health summary
      console.log(`\n${C.cyan}── Health Summary ──${C.reset}\n`);
      if (result.overall.success) {
        console.log(`  ${C.green}✓ RPC is healthy${C.reset}`);
        console.log(`  ${C.dim}  Score: ${result.overall.score}%${C.reset}`);
        console.log(`  ${C.dim}  Latency: ${formatLatency(result.overall.latency)}${C.reset}`);

        if (result.tests?.slot?.slot !== undefined) {
          console.log(`  ${C.dim}  Current slot: ${result.tests.slot.slot}${C.reset}`);
        }
      } else {
        console.log(`  ${C.red}✗ RPC is unhealthy${C.reset}`);
        console.log(`  ${C.dim}  Error: ${result.overall.error || 'Unknown error'}${C.reset}`);
        console.log(`\n  ${C.dim}Troubleshooting:${C.reset}`);
        console.log(`    • Check if validator is running: ${C.cyan}aether validator status${C.reset}`);
        console.log(`    • Verify RPC URL: ${opts.rpc}`);
        console.log(`    • Check network connectivity`);
        console.log(`    • Try: ${C.cyan}aether network-diagnostics --auto${C.reset}`);
      }
      console.log();
    }

  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        error: err.message,
        stack: err.stack,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Diagnostics failed:${C.reset} ${err.message}\n`);
    }
    process.exit(1);
  }
}

// Export for module use
module.exports = { networkDiagnosticsCommand };

// Run if called directly
if (require.main === module) {
  networkDiagnosticsCommand().catch(err => {
    console.error(`${C.red}✗ Unexpected error:${C.reset}`, err.message);
    process.exit(1);
  });
}
