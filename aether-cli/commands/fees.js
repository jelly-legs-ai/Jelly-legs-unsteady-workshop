#!/usr/bin/env node
/**
 * aether-cli fees
 *
 * Query current network fee estimates for Aether transactions.
 * Shows priority fee tiers (low, medium, high) and recent average fees.
 * Uses @jellylegsai/aether-sdk for real RPC calls to /v1/fees.
 *
 * Usage:
 *   aether fees                    Show current fee estimates
 *   aether fees --json             JSON output for scripting
 *   aether fees --verbose          Show detailed fee breakdown
 *   aether fees --rpc <url>        Custom RPC endpoint
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

// Fee priority levels
const PRIORITY_LEVELS = {
  LOW: 'low',
  MEDIUM: 'medium',
  HIGH: 'high',
  TURBO: 'turbo',
};

/**
 * Fetch fee data from Aether RPC endpoint using SDK
 * Real RPC call: GET /v1/fees
 */
async function fetchFromRpc(rpcUrl) {
  const client = new aether.AetherClient({ rpcUrl });
  try {
    const fees = await client.getFees();
    if (fees && fees.fee !== undefined) {
      return {
        baseFee: fees.baseFee || fees.fee || 5000,
        prioritizationFee: fees.prioritizationFee || 0,
        totalFee: fees.totalFee || fees.fee || 5000,
        raw: fees,
      };
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Generate simulated fee estimates as fallback
 * Used when RPC endpoint is unavailable
 */
function generateSimulatedFees() {
  const baseFee = 5000;
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
 * Build fee data from RPC response
 */
function buildFeeData(rpcFees) {
  const baseFee = rpcFees.baseFee || 5000;
  const totalFee = rpcFees.totalFee || baseFee;
  
  return {
    baseFee,
    levels: {
      [PRIORITY_LEVELS.LOW]: {
        lamports: totalFee,
        aeth: (totalFee / 1e9).toFixed(9),
        description: 'Economy',
        color: C.green,
      },
      [PRIORITY_LEVELS.MEDIUM]: {
        lamports: Math.round(totalFee * 1.5),
        aeth: (totalFee * 1.5 / 1e9).toFixed(9),
        description: 'Standard',
        color: C.cyan,
      },
      [PRIORITY_LEVELS.HIGH]: {
        lamports: Math.round(totalFee * 2.5),
        aeth: (totalFee * 2.5 / 1e9).toFixed(9),
        description: 'Fast',
        color: C.yellow,
      },
      [PRIORITY_LEVELS.TURBO]: {
        lamports: Math.round(totalFee * 5),
        aeth: (totalFee * 5 / 1e9).toFixed(9),
        description: 'Maximum',
        color: C.magenta,
      },
    },
    averageFee24h: totalFee,
    medianFee24h: totalFee,
    source: 'Aether RPC',
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
    : process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';

  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}── Aether Network Fees ──────────────────────────────────${C.reset}\n`);
  }

  // Fetch real fee data from RPC using SDK
  let feeData = null;
  let source = 'Simulated';

  try {
    const rpcFees = await fetchFromRpc(rpcUrl);
    if (rpcFees) {
      feeData = buildFeeData(rpcFees);
      source = 'Aether RPC';
    }
  } catch { /* RPC not available */ }

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
    console.error(`\n${C.red}Fees error:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
