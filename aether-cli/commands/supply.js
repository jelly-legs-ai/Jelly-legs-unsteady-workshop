#!/usr/bin/env node
/**
 * aether-cli supply
 *
 * Display current ATH token supply information.
 * Shows total supply, circulating supply, staked amount,
 * and burned tokens.
 *
 * USAGE CHANGE: Now uses the SDK module instead of raw HTTP
 *
 * Usage:
 *   aether supply                    Show full supply breakdown
 *   aether supply --json             JSON output for scripting/monitoring
 *   aether supply --verbose          Show additional details
 *   aether supply --rpc <url>        Query a specific RPC endpoint
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

// 🎉 NOW USING SDK MODULE - REAL BLOCKCHAIN RPC CALLS
const { getSupply, getDefaultRpc } = require('../sdk');

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

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function getDefaultRpcUrl() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3);
  const opts = {
    rpc: getDefaultRpcUrl(),
    asJson: false,
    verbose: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--json' || arg === '-j') {
      opts.asJson = true;
    } else if (arg === '--verbose' || arg === '-v') {
      opts.verbose = true;
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--help' || arg === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli supply${C.reset} - Token Supply Information

${C.bright}Usage:${C.reset}
  aether supply                          Show full supply breakdown
  aether supply --json                   JSON output for scripting
  aether supply --verbose                Show additional details
  aether supply --rpc <url>              Query specific RPC endpoint

${C.bright}Examples:${C.reset}
  aether supply
  aether supply --json
  aether supply --verbose --json
`.trim());
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  if (typeof lamports === 'bigint') {
    const aeth = Number(lamports) / 1e9;
    return aeth.toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 4 }) + ' AETH';
  }
  const aeth = (lamports || 0) / 1e9;
  return aeth.toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 4 }) + ' AETH';
}

function formatPercent(value) {
  if (value === null || value === undefined) return 'N/A';
  return (value * 100).toFixed(2) + '%';
}

// ---------------------------------------------------------------------------
// Main display - NOW USING SDK
// ---------------------------------------------------------------------------

async function supplyCommand() {
  const opts = parseArgs();

  try {
    // 🎉 NOW USING THE SDK MODULE - REAL BLOCKCHAIN RPC CALL!
    const supply = await getSupply(opts.rpc);

    if (!supply || supply.error) {
      throw new Error(supply?.error || 'Failed to fetch supply data');
    }

    // Normalise fields
    const total = supply.total ?? supply.total_supply ?? 0;
    const circulating = supply.circulating ?? supply.circulating_supply ?? total;
    const staked = supply.staked ?? supply.total_staked ?? 0;
    const burned = supply.burned ?? supply.total_burned ?? 0;
    const max = supply.max ?? supply.max_supply ?? null;

    // Calculate percentages
    const stakedPct = total > 0 ? (staked / total) : 0;
    const circulatingPct = total > 0 ? (circulating / total) : 0;
    const burnedPct = total > 0 ? (burned / total) : 0;

    if (opts.asJson) {
      const out = {
        total_supply: total,
        total_supply_formatted: formatAether(total),
        circulating: circulating,
        circulating_formatted: formatAether(circulating),
        circulating_pct: circulatingPct,
        staked: staked,
        staked_formatted: formatAether(staked),
        staked_pct: stakedPct,
        burned: burned,
        burned_formatted: formatAether(burned),
        burned_pct: burnedPct,
        max_supply: max,
        max_supply_formatted: max ? formatAether(max) : null,
        rpc: opts.rpc,
        sdk_version: '1.0.0',
        fetched_at: new Date().toISOString(),
      };
      console.log(JSON.stringify(out, null, 2));
      return;
    }

    // Human-readable output
    console.log('');
    console.log(`  ${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════╗${C.reset}`);
    console.log(`  ${C.bright}${C.cyan}║                    ATH Token Supply                           ║${C.reset}`);
    console.log(`  ${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════╝${C.reset}`);
    console.log('');

    console.log(`  ${C.dim}RPC: ${opts.rpc}${C.reset}`);
    console.log(`  ${C.dim}Data source: SDK (real blockchain RPC)${C.reset}`);
    console.log('');

    // Total supply
    console.log(`  ${C.bright}Total Supply${C.reset}`);
    console.log(`    ${C.cyan}${formatAether(total)}${C.reset}`);
    console.log();

    // Circulating
    console.log(`  ${C.bright}Circulating Supply${C.reset}`);
    console.log(`    ${C.green}${formatAether(circulating)}${C.reset} ${C.dim}(${formatPercent(circulatingPct)} of total)${C.reset}`);
    console.log();

    // Staked
    console.log(`  ${C.bright}Staked${C.reset}`);
    console.log(`    ${C.magenta}${formatAether(staked)}${C.reset} ${C.dim}(${formatPercent(stakedPct)} of total)${C.reset}`);
    console.log();

    // Burned
    if (burned > 0) {
      console.log(`  ${C.bright}Burned${C.reset}`);
      console.log(`    ${C.yellow}${formatAether(burned)}${C.reset} ${C.dim}(${formatPercent(burnedPct)} of total)${C.reset}`);
      console.log();
    }

    // Max supply
    if (max !== null) {
      console.log(`  ${C.bright}Max Supply (Cap)${C.reset}`);
      console.log(`    ${C.dim}${formatAether(max)}${C.reset}`);
      console.log();
    }

    // Verbose details
    if (opts.verbose) {
      console.log(`  ${C.bright}${C.dim}───────────────────────────────────────────────────────────────${C.reset}`);
      console.log();
      console.log(`  ${C.bright}Raw Supply Data${C.reset}`);
      console.log(`    ${C.dim}Total:        ${total.toLocaleString()} lamports${C.reset}`);
      console.log(`    ${C.dim}Circulating:  ${circulating.toLocaleString()} lamports${C.reset}`);
      console.log(`    ${C.dim}Staked:       ${staked.toLocaleString()} lamports${C.reset}`);
      console.log(`    ${C.dim}Burned:       ${burned.toLocaleString()} lamports${C.reset}`);
      if (max !== null) {
        console.log(`    ${C.dim}Max:          ${max.toLocaleString()} lamports${C.reset}`);
      }
      console.log();
    }

    console.log(`  ${C.dim}Fetched via SDK: getSupply() → GET /v1/supply${C.reset}`);
    console.log();

  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc: opts.rpc,
        sdk_version: '1.0.0',
        sdk_used: true,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Failed to fetch supply data:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}RPC: ${opts.rpc}${C.reset}`);
      console.log(`  ${C.dim}Is your validator running?${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Export & run
// ---------------------------------------------------------------------------

module.exports = { supplyCommand };

if (require.main === module) {
  supplyCommand();
}
