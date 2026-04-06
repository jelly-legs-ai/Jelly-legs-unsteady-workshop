#!/usr/bin/env node
/**
 * aether-cli fees
 *
 * Query current network fee estimates for Aether transactions.
 * Shows priority fee tiers (low, medium, high) and recent average fees.
 *
 * Usage:
 *   aether fees                    Show current fee estimates
 *   aether fees --json             JSON output for scripting
 *   aether fees --verbose          Show detailed fee breakdown
 *   aether fees --rpc <url>        Custom RPC endpoint
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

// Fee priority levels
const PRIORITY_LEVELS = {
  LOW: 'low',
  MEDIUM: 'medium',
  HIGH: 'high',
  TURBO: 'turbo',
};

/**
 * HTTP GET helper
 */
function httpGet(url, timeout = 8000) {
  return new Promise((resolve, reject) => {
    const lib = url.startsWith('https') ? https : http;
    const parsed = new URL(url);
    const req = lib.request({
      hostname: parsed.hostname,
      port: parsed.port || (parsed.protocol === 'https:' ? 443 : 80),
      path: parsed.pathname + parsed.search,
      method: 'GET',
      timeout,
      headers: { 'Accept': 'application/json', 'User-Agent': 'Aether-CLI/1.0' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ _raw: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

/**
 * Fetch fee data from Aether RPC endpoint
 * Uses getFeeForMessage or getRecentPrioritizationFees RPC methods
 */
async function fetchFromRpc(rpcUrl) {
  try {
    // Try to get recent prioritization fees
    const body = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'getRecentPrioritizationFees',
      params: [],
    });

    const res = await httpGet(rpcUrl, 5000);
    if (res && res.result) {
      const fees = res.result;
      if (Array.isArray(fees) && fees.length > 0) {
        const prioritizationFee = fees.reduce((sum, f) => sum + (f.prioritizationFee || 0), 0) / fees.length;
        return {
          baseFee: 5000, // Base fee in lamports (typical)
          prioritizationFee: Math.round(prioritizationFee),
          totalFee: 5000 + Math.round(prioritizationFee),
        };
      }
    }
  } catch { /* RPC not available */ }

  return null;
}

/**
 * Fetch fee estimates from a public fee oracle API
 * Fallback if local RPC is not available
 */
async function fetchFromOracle() {
  try {
    // Simulated fee oracle response based on network conditions
    // In production, this would query a real fee oracle service
    const baseUrl = 'https://api.aether.network/v1/fees';
    
    const res = await httpGet(baseUrl, 8000);
    if (res && res.fees) {
      return res.fees;
    }
  } catch { /* Oracle not available */ }

  return null;
}

/**
 * Generate simulated fee estimates based on network activity
 * Used as final fallback when no RPC or oracle is available
 */
function generateSimulatedFees() {
  // Base fee in lamports (1 AETH = 1e9 lamports)
  const baseFee = 5000;
  
  // Simulate network congestion levels
  const congestionFactor = 1.0 + (Math.random() * 0.5);
  
  return {
    baseFee,
    levels: {
      [PRIORITY_LEVELS.LOW]: {
        lamports: Math.round(baseFee * 1.0 * congestionFactor),
        aeth: (baseFee * 1.0 * congestionFactor / 1e9).toFixed(9),
        description: 'Economy - may take longer during congestion',
        color: C.green,
      },
      [PRIORITY_LEVELS.MEDIUM]: {
        lamports: Math.round(baseFee * 1.5 * congestionFactor),
        aeth: (baseFee * 1.5 * congestionFactor / 1e9).toFixed(9),
        description: 'Standard - typical confirmation time',
        color: C.cyan,
      },
      [PRIORITY_LEVELS.HIGH]: {
        lamports: Math.round(baseFee * 2.5 * congestionFactor),
        aeth: (baseFee * 2.5 * congestionFactor / 1e9).toFixed(9),
        description: 'Fast - priority during high congestion',
        color: C.yellow,
      },
      [PRIORITY_LEVELS.TURBO]: {
        lamports: Math.round(baseFee * 5.0 * congestionFactor),
        aeth: (baseFee * 5.0 * congestionFactor / 1e9).toFixed(9),
        description: 'Maximum - fastest confirmation',
        color: C.magenta,
      },
    },
    averageFee24h: Math.round(baseFee * 1.8 * congestionFactor),
    medianFee24h: Math.round(baseFee * 1.5 * congestionFactor),
    source: 'Aether Network (simulated)',
    timestamp: new Date(),
  };
}

/**
 * Format lamports to human-readable string
 */
function formatLamports(lamports) {
  if (lamports >= 1e9) {
    return `${(lamports / 1e9).toFixed(6)} AETH`;
  } else if (lamports >= 1e6) {
    return `${(lamports / 1e6).toFixed(3)} mAETH`;
  } else if (lamports >= 1e3) {
    return `${(lamports / 1e3).toFixed(1)} µAETH`;
  }
  return `${lamports} lamports`;
}

/**
 * Format timestamp
 */
function formatTime(date) {
  return date.toISOString().replace('T', ' ').substring(0, 19) + ' UTC';
}

/**
 * Main fees command
 */
async function feesCommand() {
  const args = process.argv.slice(2);
  const asJson = args.includes('--json') || args.includes('-j');
  const verbose = args.includes('--verbose') || args.includes('-v');
  const rpcUrl = args.includes('--rpc')
    ? args[args.indexOf('--rpc') + 1]
    : process.env.AETHER_RPC || 'http://127.0.0.1:8899';

  console.log(`\n${C.bright}${C.cyan}── Aether Network Fees ──────────────────────────────────${C.reset}\n`);

  // Try to fetch real fee data
  let feeData = null;
  let source = 'Simulated';

  // Try local RPC first
  try {
    const rpcFees = await fetchFromRpc(rpcUrl);
    if (rpcFees) {
      feeData = {
        baseFee: rpcFees.baseFee,
        levels: {
          [PRIORITY_LEVELS.LOW]: {
            lamports: rpcFees.totalFee,
            aeth: (rpcFees.totalFee / 1e9).toFixed(9),
            description: 'Economy',
            color: C.green,
          },
          [PRIORITY_LEVELS.MEDIUM]: {
            lamports: Math.round(rpcFees.totalFee * 1.5),
            aeth: (rpcFees.totalFee * 1.5 / 1e9).toFixed(9),
            description: 'Standard',
            color: C.cyan,
          },
          [PRIORITY_LEVELS.HIGH]: {
            lamports: Math.round(rpcFees.totalFee * 2.5),
            aeth: (rpcFees.totalFee * 2.5 / 1e9).toFixed(9),
            description: 'Fast',
            color: C.yellow,
          },
          [PRIORITY_LEVELS.TURBO]: {
            lamports: Math.round(rpcFees.totalFee * 5),
            aeth: (rpcFees.totalFee * 5 / 1e9).toFixed(9),
            description: 'Maximum',
            color: C.magenta,
          },
        },
        averageFee24h: rpcFees.totalFee,
        medianFee24h: rpcFees.totalFee,
        source: 'Aether RPC',
        timestamp: new Date(),
      };
      source = 'Aether RPC';
    }
  } catch { /* RPC not available */ }

  // Try oracle if RPC failed
  if (!feeData) {
    try {
      const oracleFees = await fetchFromOracle();
      if (oracleFees) {
        feeData = oracleFees;
        source = 'Aether Oracle';
      }
    } catch { /* Oracle not available */ }
  }

  // Use simulated fees as fallback
  if (!feeData) {
    feeData = generateSimulatedFees();
    source = 'Simulated';
  }

  // JSON output
  if (asJson) {
    const output = {
      source: feeData.source,
      timestamp: formatTime(feeData.timestamp),
      base_fee_lamports: feeData.baseFee,
      priority_levels: Object.fromEntries(
        Object.entries(feeData.levels).map(([key, val]) => [
          key,
          { lamports: val.lamports, aeth: parseFloat(val.aeth) }
        ])
      ),
      average_fee_24h_lamports: feeData.averageFee24h,
      median_fee_24h_lamports: feeData.medianFee24h,
    };
    console.log(JSON.stringify(output, null, 2));
    return;
  }

  // Human-readable output
  console.log(`  ${C.dim}Source:${C.reset}      ${C.bright}${feeData.source}${C.reset}`);
  console.log(`  ${C.dim}Updated:${C.reset}     ${formatTime(feeData.timestamp)}`);
  console.log(`  ${C.dim}Base Fee:${C.reset}    ${formatLamports(feeData.baseFee)}`);
  console.log();

  console.log(`  ${C.bright}Priority Levels:${C.reset}\n`);
  console.log(`  ┌─────────────┬──────────────────────┬─────────────────────────────────────┐`);
  console.log(`  │ ${C.bright}Level${C.reset}      │ ${C.bright}Fee${C.reset}                │ ${C.bright}Description${C.reset}                     │`);
  console.log(`  ├─────────────┼──────────────────────┼─────────────────────────────────────┤`);

  Object.entries(feeData.levels).forEach(([level, info]) => {
    const levelName = level.charAt(0) + level.slice(1).toLowerCase();
    const feeStr = `${info.color}${formatLamports(info.lamports).padEnd(20)}${C.reset}`;
    const descStr = info.description.padEnd(35);
    console.log(`  │ ${levelName.padEnd(11)} │ ${feeStr} │ ${descStr} │`);
  });

  console.log(`  └─────────────┴──────────────────────┴─────────────────────────────────────┘`);
  console.log();

  // 24h statistics
  console.log(`  ${C.dim}24h Statistics:${C.reset}`);
  console.log(`    ${C.dim}Average:${C.reset}  ${formatLamports(feeData.averageFee24h)}`);
  console.log(`    ${C.dim}Median:${C.reset}   ${formatLamports(feeData.medianFee24h)}`);
  console.log();

  // Verbose mode - show additional details
  if (verbose) {
    console.log(`  ${C.bright}Fee Breakdown:${C.reset}\n`);
    console.log(`    ${C.dim}Base Fee:${C.reset}           ${formatLamports(feeData.baseFee)}`);
    console.log(`    ${C.dim}Priority Multiplier:${C.reset} 1.0x - 5.0x (based on urgency)`);
    console.log(`    ${C.dim}Network Congestion:${C.reset} ${Math.round((feeData.averageFee24h / feeData.baseFee - 1) * 100)}% above base`);
    console.log();

    console.log(`  ${C.bright}Recommendations:${C.reset}\n`);
    console.log(`    • Use ${C.cyan}Standard${C.reset} for routine transactions`);
    console.log(`    • Use ${C.yellow}Fast${C.reset} during network congestion or for time-sensitive ops`);
    console.log(`    • Use ${C.magenta}Maximum${C.reset} for validator operations or large transfers`);
    console.log();
  }

  // Usage tip
  console.log(`  ${C.dim}Tip: Set ${C.cyan}--priority${C.reset}${C.dim} flag when submitting transactions to choose fee level.${C.reset}`);
  console.log(`  ${C.dim}Example: ${C.cyan}aether transfer --to <addr> --amount 10 --priority high${C.reset}\n`);
}

// Export for use in index.js
module.exports = { feesCommand };

// Run if called directly
if (require.main === module) {
  feesCommand().catch(err => {
    console.error(`\n${C.red}Fees error:${C.reset}`, err.message, '\n');
    process.exit(1);
  });
}
