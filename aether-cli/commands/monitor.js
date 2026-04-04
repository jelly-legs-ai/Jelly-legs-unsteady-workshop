#!/usr/bin/env node
/**
 * aether-cli monitor - Real-time Validator Dashboard
 * 
 * Polls the validator RPC endpoints and displays:
 * - Current slot number
 * - Block height
 * - Peer count
 * - Transactions per second (TPS)
 * - Validator health status
 * 
 * Updates every 2 seconds with rich terminal output.
 */

const http = require('http');
const https = require('https');
const os = require('os');

// ANSI colors and control codes
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  bgBlue: '\x1b[44m',
  bgGreen: '\x1b[42m',
  bgRed: '\x1b[41m',
};

const cursor = {
  up: '\x1b[A',
  down: '\x1b[B',
  right: '\x1b[C',
  left: '\x1b[D',
  hide: '\x1b[?25l',
  show: '\x1b[?25h',
  clear: '\x1b[2J',
  clearLine: '\x1b[2K',
};

// Default RPC endpoint
const DEFAULT_RPC = 'http://127.0.0.1:8899';
const POLL_INTERVAL_MS = 2000;

// State tracking for TPS calculation
let previousSlot = null;
let previousTimestamp = null;
let tpsHistory = [];

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(3);
  const options = {
    rpc: DEFAULT_RPC,
    interval: POLL_INTERVAL_MS,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--rpc':
      case '-r':
        options.rpc = args[++i];
        break;
      case '--interval':
      case '-i':
        options.interval = parseInt(args[++i], 10) || POLL_INTERVAL_MS;
        break;
      case '--help':
      case '-h':
        showHelp();
        process.exit(0);
    }
  }

  return options;
}

/**
 * Show help message
 */
function showHelp() {
  console.log(`
${colors.bright}${colors.cyan}aether-cli monitor${colors.reset} - Real-time Validator Dashboard

${colors.bright}Usage:${colors.reset}
  aether-cli monitor [options]

${colors.bright}Options:${colors.reset}
  -r, --rpc <url>       RPC endpoint (default: ${DEFAULT_RPC})
  -i, --interval <ms>   Poll interval in milliseconds (default: ${POLL_INTERVAL_MS})
  -h, --help            Show this help message

${colors.bright}Examples:${colors.reset}
  aether-cli monitor                    # Monitor local validator
  aether-cli monitor --rpc http://api.testnet.aether.network
  aether-cli monitor -i 1000            # Poll every second
`.trim());
}

/**
 * Make HTTP request to RPC endpoint
 */
function rpcRequest(method, params = []) {
  return new Promise((resolve, reject) => {
    const url = new URL(options.rpc);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    const postData = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method,
      params,
    });

    const reqOptions = {
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: '/',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData),
      },
      timeout: 5000,
    };

    const req = lib.request(reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          const result = JSON.parse(data);
          if (result.error) {
            reject(new Error(result.error.message));
          } else {
            resolve(result.result);
          }
        } catch (e) {
          reject(new Error(`Failed to parse response: ${e.message}`));
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });

    req.write(postData);
    req.end();
  });
}

/**
 * Fetch slot information
 */
async function getSlot() {
  return await rpcRequest('getSlot');
}

/**
 * Fetch block height
 */
async function getBlockHeight() {
  return await rpcRequest('getBlockHeight');
}

/**
 * Fetch vote accounts (for peer count approximation)
 */
async function getVoteAccounts() {
  return await rpcRequest('getVoteAccounts');
}

/**
 * Fetch recent performance samples (for TPS)
 */
async function getRecentPerformanceSamples() {
  return await rpcRequest('getRecentPerformanceSamples', [1]);
}

/**
 * Calculate TPS from slot progression
 */
function calculateTPS(currentSlot, currentTimestamp) {
  if (previousSlot === null || previousTimestamp === null) {
    return null;
  }

  const slotDiff = currentSlot - previousSlot;
  const timeDiff = (currentTimestamp - previousTimestamp) / 1000; // seconds

  if (timeDiff <= 0) return 0;

  const instantTPS = slotDiff / timeDiff;
  
  // Smooth with history
  tpsHistory.push(instantTPS);
  if (tpsHistory.length > 5) {
    tpsHistory.shift();
  }

  const avgTPS = tpsHistory.reduce((a, b) => a + b, 0) / tpsHistory.length;
  return avgTPS;
}

/**
 * Get status color based on value
 */
function getStatusColor(healthy) {
  return healthy ? colors.green : colors.red;
}

/**
 * Format number with commas
 */
function formatNumber(num) {
  if (num === null || num === undefined) return 'N/A';
  return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

/**
 * Render the dashboard
 */
function renderDashboard(data, error = null) {
  const now = new Date();
  const timestamp = now.toLocaleTimeString();

  // Clear screen and hide cursor
  process.stdout.write(cursor.clear);
  process.stdout.write(cursor.hide);

  // Header
  const header = `
${colors.bright}${colors.cyan}
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ${colors.bright}AETHER NETWORK MONITOR${colors.reset}${colors.cyan}                                  ║
║   ${colors.dim}Real-time Validator Dashboard${colors.reset}${colors.cyan}                             ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `.trim();

  console.log(header);
  console.log();

  if (error) {
    console.log(`  ${colors.bgRed}${colors.bright} ERROR ${colors.reset} ${colors.red}${error.message}${colors.reset}`);
    console.log();
    console.log(`  ${colors.dim}Retrying in ${options.interval / 1000}s...${colors.reset}`);
    console.log();
    console.log(`  ${colors.dim}RPC: ${options.rpc}${colors.reset}`);
    console.log(`  ${colors.dim}Last update: ${timestamp}${colors.reset}`);
    return;
  }

  const { slot, blockHeight, peerCount, tps, health } = data;

  // Status indicator
  const statusIcon = health ? `${colors.green}●${colors.reset}` : `${colors.red}●${colors.reset}`;
  const statusText = health ? `${colors.green}HEALTHY${colors.reset}` : `${colors.red}UNHEALTHY${colors.reset}`;

  console.log(`  ${statusIcon} Status: ${statusText}`);
  console.log();

  // Metrics grid
  console.log(`  ${colors.bright}┌─────────────────────────────────────────────────────────────┐${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.cyan}Current Slot${colors.reset}${' '.repeat(44)}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.bright}${colors.green}${formatNumber(slot).padEnd(52)}${colors.reset}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}                                                               ${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.cyan}Block Height${colors.reset}${' '.repeat(44)}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.bright}${colors.blue}${formatNumber(blockHeight).padEnd(52)}${colors.reset}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}                                                               ${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.cyan}Active Peers${colors.reset}${' '.repeat(44)}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.bright}${colors.magenta}${formatNumber(peerCount).padEnd(52)}${colors.reset}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}                                                               ${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.cyan}Transactions/sec${colors.reset}${' '.repeat(40)}${colors.bright}│${colors.reset}`);
  
  const tpsDisplay = tps !== null ? `${tps.toFixed(2)} TPS` : 'Calculating...';
  const tpsColor = tps !== null && tps > 0 ? colors.green : colors.yellow;
  console.log(`  ${colors.bright}│${colors.reset}  ${colors.bright}${tpsColor}${tpsDisplay.padEnd(52)}${colors.reset}${colors.bright}│${colors.reset}`);
  console.log(`  ${colors.bright}└─────────────────────────────────────────────────────────────┘${colors.reset}`);

  console.log();
  console.log(`  ${colors.dim}RPC: ${options.rpc}${colors.reset}`);
  console.log(`  ${colors.dim}Last update: ${timestamp}${colors.reset}`);
  console.log();
  console.log(`  ${colors.dim}Press Ctrl+C to exit${colors.reset}`);
}

/**
 * Main monitor loop
 */
async function monitorLoop() {
  let iteration = 0;

  while (true) {
    try {
      const startTime = Date.now();

      // Fetch all metrics in parallel
      const [slot, blockHeight, voteAccounts, performanceSamples] = await Promise.all([
        getSlot().catch(() => null),
        getBlockHeight().catch(() => null),
        getVoteAccounts().catch(() => null),
        getRecentPerformanceSamples().catch(() => null),
      ]);

      const currentTime = Date.now();

      // Calculate TPS
      let tps = null;
      if (slot !== null) {
        tps = calculateTPS(slot, currentTime);
        previousSlot = slot;
        previousTimestamp = currentTime;
      }

      // Derive peer count from vote accounts
      const peerCount = voteAccounts 
        ? voteAccounts.current.length + (voteAccounts.delinquent?.length || 0)
        : 0;

      // Health check: validator is healthy if we got valid slot data
      const health = slot !== null && blockHeight !== null;

      renderDashboard({
        slot: slot || 0,
        blockHeight: blockHeight || 0,
        peerCount,
        tps,
        health,
      });

      iteration++;

    } catch (error) {
      renderDashboard(null, error);
    }

    // Wait for next poll
    await new Promise(resolve => setTimeout(resolve, options.interval));
  }
}

/**
 * Handle graceful shutdown
 */
function setupShutdownHandlers() {
  const cleanup = () => {
    // Show cursor
    process.stdout.write(cursor.show);
    console.log(`\n${colors.yellow}Monitor stopped.${colors.reset}\n`);
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('exit', () => {
    process.stdout.write(cursor.show);
  });
}

// Global options (parsed from args)
let options;

/**
 * Main entry point
 */
function main() {
  options = parseArgs();

  // Show cursor on exit
  setupShutdownHandlers();

  // Print startup message
  console.log(`\n${colors.cyan}Starting Aether Network Monitor...${colors.reset}`);
  console.log(`${colors.dim}RPC Endpoint: ${options.rpc}${colors.reset}`);
  console.log(`${colors.dim}Poll Interval: ${options.interval}ms${colors.reset}\n`);

  // Start monitoring
  monitorLoop().catch(err => {
    console.error(`${colors.red}Fatal error: ${err.message}${colors.reset}`);
    process.exit(1);
  });
}

// Run if called directly
if (require.main === module) {
  main();
}

// Export for testing and CLI integration
module.exports = { 
  monitorLoop, 
  getSlot, 
  getBlockHeight, 
  getVoteAccounts,
  calculateTPS,
  renderDashboard,
  main,
};
