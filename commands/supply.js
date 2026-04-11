#!/usr/bin/env node
/**
 * aether-cli supply
 *
 * Query Aether token supply information from the blockchain.
 * Shows total supply, circulating supply, non-circulating supply,
 * and supply breakdown with visual indicators.
 *
 * Usage:
 *   aether supply                    Show detailed supply info
 *   aether supply --json             JSON output for scripting
 *   aether supply --rpc <url>        Query specific RPC endpoint
 *   aether supply --watch            Watch mode - updates every 5 seconds
 *   aether supply --compare          Compare with theoretical max
 *
 * SDK wired to: GET /v1/supply
 * SDK Function: sdk.getSupply()
 */

const path = require('path');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// Import UI framework
const {
  C,
  BRANDING,
  indicators,
  success,
  error,
  warning,
  info,
  highlight,
  dim,
  startSpinner,
  stopSpinner,
  drawBox,
  drawTable,
  progressBarColored,
} = require('../lib/ui');

const CLI_VERSION = '1.0.0';
const WATCH_INTERVAL_MS = 5000;

// Supply constants
const MAX_SUPPLY_AETH = 1_000_000_000; // 1 billion AETH theoretical max

// ============================================================================
// SDK Setup
// ============================================================================

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ============================================================================
// ASCII Art & Branding
// ============================================================================

const SUPPLY_LOGO = `
${C.cyan}    ╔══════════════════════════════════════════════════════════╗${C.reset}
${C.cyan}    ║${C.reset}  ${C.bright}${C.yellow}◆${C.reset} ${C.bright}AETHER TOKEN SUPPLY${C.reset}${' '.repeat(30)}${C.dim}v${CLI_VERSION}${C.reset}  ${C.cyan}║${C.reset}
${C.cyan}    ║${C.reset}     ${C.dim}On-chain supply metrics and tokenomics${C.reset}${' '.repeat(20)}${C.cyan}║${C.reset}
${C.cyan}    ╚══════════════════════════════════════════════════════════╝${C.reset}`;

// ============================================================================
// Format Helpers
// ============================================================================

function formatAether(lamports) {
  if (!lamports && lamports !== 0) return 'N/A';
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  if (aeth >= 1_000_000) {
    return aeth.toFixed(2).replace(/\B(?=(\d{3})+(?!\d))/g, ',') + ' AETH';
  }
  return aeth.toFixed(6).replace(/\.?0+$/, '') + ' AETH';
}

function formatNumber(num) {
  if (!num && num !== 0) return 'N/A';
  return Number(num).toLocaleString();
}

function formatPercentage(numerator, denominator) {
  if (!denominator || denominator === 0) return 'N/A';
  const pct = (Number(numerator) / Number(denominator)) * 100;
  return pct.toFixed(2) + '%';
}

function formatCompact(n) {
  if (!n) return 'N/A';
  const num = Number(n);
  if (num >= 1e9) return (num / 1e9).toFixed(2) + 'B';
  if (num >= 1e6) return (num / 1e6).toFixed(2) + 'M';
  if (num >= 1e3) return (num / 1e3).toFixed(2) + 'K';
  return num.toString();
}

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  return {
    rpc: args.includes('--rpc') ? args[args.indexOf('--rpc') + 1] : getDefaultRpc(),
    asJson: args.includes('--json') || args.includes('-j'),
    watch: args.includes('--watch') || args.includes('-w'),
    compare: args.includes('--compare') || args.includes('-c'),
    help: args.includes('--help') || args.includes('-h'),
  };
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli supply${C.reset} — Token Supply Information

${C.bright}USAGE${C.reset}
    aether supply [options]

${C.bright}OPTIONS${C.reset}
    -r, --rpc <url>      RPC endpoint (default: ${getDefaultRpc()})
    -j, --json           Output as JSON
    -w, --watch          Watch mode - updates every 5 seconds
    -c, --compare        Show comparison with theoretical max supply
    -h, --help           Show this help

${C.bright}SDK METHODS USED${C.reset}
    ${C.dim}client.getSupply()     → GET /v1/supply${C.reset}
    ${C.dim}client.getEpochInfo()  → GET /v1/epoch${C.reset}

${C.bright}EXAMPLES${C.reset}
    aether supply                    # Detailed supply view
    aether supply --json             # JSON for scripting
    aether supply --watch            # Live updates
    aether supply --compare          # Compare with max supply
    aether supply --rpc https://api.aether.io

${C.bright}OUTPUT FIELDS${C.reset}
    • Total Supply       — Total minted AETH (lamports)
    • Circulating        — AETH in active circulation
    • Non-Circulating    — Locked, staked, or reserved AETH
    • Staked             — AETH delegated to validators
    • Inflation Rate     — Current epoch inflation
`);
}

// ============================================================================
// SDK Data Fetching (REAL RPC CALLS)
// ============================================================================

async function fetchSupplyData(rpcUrl) {
  const client = createClient(rpcUrl);

  try {
    // Parallel SDK calls for supply and epoch info
    const [supplyResult, epochInfo] = await Promise.all([
      client.getSupply().catch(() => null),
      client.getEpochInfo().catch(() => null),
    ]);

    if (!supplyResult) {
      throw new Error('No supply data returned from RPC');
    }

    // Normalize supply data from various RPC response formats
    const total = supplyResult.total || supplyResult.total_supply || supplyResult.totalSupply || 0;
    const circulating = supplyResult.circulating || supplyResult.circulating_supply || supplyResult.circulatingSupply || 0;
    const nonCirculating = supplyResult.nonCirculating || supplyResult.non_circulating || supplyResult.nonCirculatingSupply || (total - circulating);
    const staked = supplyResult.staked || supplyResult.total_staked || supplyResult.delegated || 0;
    const rewards = supplyResult.rewards || supplyResult.validator_rewards || 0;

    return {
      total: BigInt(total),
      circulating: BigInt(circulating),
      nonCirculating: BigInt(nonCirculating),
      staked: BigInt(staked),
      rewards: BigInt(rewards),
      epoch: epochInfo?.epoch || 0,
      slot: epochInfo?.absoluteSlot || 0,
      fetchedAt: new Date().toISOString(),
    };
  } catch (err) {
    throw new Error(`Failed to fetch supply: ${err.message}`);
  }
}

// ============================================================================
// Visual Rendering
// ============================================================================

function renderSupplyBox(data, opts = {}) {
  const { total, circulating, nonCirculating, staked, epoch, slot } = data;
  const { compare } = opts;

  const totalAeth = Number(total) / 1e9;
  const circulatingAeth = Number(circulating) / 1e9;
  const nonCirculatingAeth = Number(nonCirculating) / 1e9;
  const stakedAeth = Number(staked) / 1e9;

  // Calculate percentages
  const circulatingPct = total > 0 ? (circulatingAeth / totalAeth) * 100 : 0;
  const nonCircPct = total > 0 ? (nonCirculatingAeth / totalAeth) * 100 : 0;
  const stakedPct = total > 0 ? (stakedAeth / totalAeth) * 100 : 0;

  // Circulation ratio
  const circulationRatio = circulating > 0
    ? ((Number(circulating) / Number(total)) * 100).toFixed(2)
    : '0.00';

  console.log(SUPPLY_LOGO);
  console.log();

  // Main supply box
  const supplyContent = `
${C.bright}Total Supply${C.reset}
${C.cyan}${formatAether(total)}${C.reset} ${C.dim}(${formatNumber(total)} lamports)${C.reset}

${C.bright}Circulating${C.reset}     ${C.green}${formatAether(circulating)}${C.reset}
${C.bright}Non-Circulating${C.reset} ${C.yellow}${formatAether(nonCirculating)}${C.reset}
${C.bright}Staked${C.reset}          ${C.magenta}${formatAether(staked)}${C.reset}

${C.dim}Circulation Ratio: ${C.bright}${circulationRatio}%${C.reset}
${C.dim}Current Epoch:     ${C.bright}${epoch}${C.reset}
${C.dim}Current Slot:      ${C.bright}${formatNumber(slot)}${C.reset}
`.trim();

  console.log(drawBox(supplyContent, {
    style: 'double',
    title: 'SUPPLY OVERVIEW',
    titleColor: C.cyan + C.bright,
    borderColor: C.cyan,
    width: 60,
  }));

  console.log();

  // Distribution bars
  console.log(`  ${C.bright}Supply Distribution:${C.reset}\n`);

  const barWidth = 40;

  // Circulating bar
  const circFill = Math.round((circulatingPct / 100) * barWidth);
  const circBar = `${C.green}${'█'.repeat(circFill)}${C.dim}${'░'.repeat(barWidth - circFill)}${C.reset}`;
  console.log(`  ${C.green}●${C.reset} Circulating      ${circBar} ${C.green}${circulatingPct.toFixed(1)}%${C.reset}`);

  // Non-circulating bar
  const ncFill = Math.round((nonCircPct / 100) * barWidth);
  const ncBar = `${C.yellow}${'█'.repeat(ncFill)}${C.dim}${'░'.repeat(barWidth - ncFill)}${C.reset}`;
  console.log(`  ${C.yellow}●${C.reset} Non-Circulating  ${ncBar} ${C.yellow}${nonCircPct.toFixed(1)}%${C.reset}`);

  // Staked bar (subset of circulating or total)
  const stakedOfTotal = stakedAeth / totalAeth * 100;
  const stakedFill = Math.round((stakedOfTotal / 100) * barWidth);
  const stakedBar = `${C.magenta}${'█'.repeat(stakedFill)}${C.dim}${'░'.repeat(barWidth - stakedFill)}${C.reset}`;
  console.log(`  ${C.magenta}●${C.reset} Staked           ${stakedBar} ${C.magenta}${stakedOfTotal.toFixed(1)}%${C.reset}`);

  console.log();

  // Comparison with max supply if requested
  if (compare) {
    const pctOfMax = (totalAeth / MAX_SUPPLY_AETH) * 100;
    const remaining = MAX_SUPPLY_AETH - totalAeth;

    console.log(`  ${C.bright}Comparison with Theoretical Max:${C.reset}\n`);
    console.log(`  ${C.dim}Max Supply:${C.reset}      ${C.bright}${formatCompact(MAX_SUPPLY_AETH * 1e9)}${C.reset}`);
    console.log(`  ${C.dim}Current:${C.reset}         ${C.cyan}${formatCompact(Number(total))}${C.reset}`);
    console.log(`  ${C.dim}Remaining:${C.reset}       ${C.green}${formatCompact(remaining * 1e9)}${C.reset}`);
    console.log(`  ${C.dim}% of Max:${C.reset}        ${C.bright}${pctOfMax.toFixed(4)}%${C.reset}`);

    const maxFill = Math.round((pctOfMax / 100) * barWidth);
    const maxBar = `${C.cyan}${'█'.repeat(maxFill)}${C.dim}${'░'.repeat(barWidth - maxFill)}${C.reset}`;
    console.log(`\n  ${C.dim}Supply Cap:${C.reset} ${maxBar} ${C.cyan}${pctOfMax.toFixed(2)}%${C.reset}`);
    console.log();
  }

  // Tokenomics stats table
  const statsRows = [
    ['Metric', 'Value', 'Percentage'],
    ['─'.repeat(20), '─'.repeat(25), '─'.repeat(12)],
    ['Total Supply', formatAether(total), '100%'],
    ['Circulating', formatAether(circulating), `${circulatingPct.toFixed(2)}%`],
    ['Non-Circulating', formatAether(nonCirculating), `${nonCircPct.toFixed(2)}%`],
    ['Staked', formatAether(staked), `${stakedPct.toFixed(2)}%`],
  ];

  console.log(`  ${C.bright}Tokenomics Breakdown:${C.reset}\n`);
  console.log(drawTable(['', '', ''], [
    [`${C.cyan}Total Supply${C.reset}`, C.bright + formatAether(total) + C.reset, '100%'],
    [`${C.green}Circulating${C.reset}`, formatAether(circulating), `${circulatingPct.toFixed(2)}%`],
    [`${C.yellow}Non-Circulating${C.reset}`, formatAether(nonCirculating), `${nonCircPct.toFixed(2)}%`],
    [`${C.magenta}Staked${C.reset}`, formatAether(staked), `${stakedPct.toFixed(2)}%`],
  ], {
    borderStyle: 'single',
    headerColor: C.bright,
  }));

  console.log();

  // Network health indicator
  const healthStatus = stakedPct > 50
    ? `${C.green}✓ Healthy${C.reset} - High stake ratio indicates network security`
    : stakedPct > 30
      ? `${C.yellow}⚠ Moderate${C.reset} - Adequate stake ratio`
      : `${C.red}✗ Low${C.reset} - Low stake ratio may indicate risk`;

  console.log(`  ${C.bright}Network Health:${C.reset} ${healthStatus}`);
  console.log();

  // Footer
  console.log(`  ${C.dim}Data fetched: ${data.fetchedAt}${C.reset}`);
  console.log(`  ${C.dim}RPC: ${opts.rpc}${C.reset}`);
  console.log(`  ${C.dim}SDK: @jellylegsai/aether-sdk${C.reset}`);
  console.log();
}

function renderJson(data, rpc) {
  const output = {
    total: {
      lamports: data.total.toString(),
      aeth: (Number(data.total) / 1e9).toFixed(9),
      formatted: formatAether(data.total),
    },
    circulating: {
      lamports: data.circulating.toString(),
      aeth: (Number(data.circulating) / 1e9).toFixed(9),
      formatted: formatAether(data.circulating),
      percentage: ((Number(data.circulating) / Number(data.total)) * 100).toFixed(2),
    },
    nonCirculating: {
      lamports: data.nonCirculating.toString(),
      aeth: (Number(data.nonCirculating) / 1e9).toFixed(9),
      formatted: formatAether(data.nonCirculating),
      percentage: ((Number(data.nonCirculating) / Number(data.total)) * 100).toFixed(2),
    },
    staked: {
      lamports: data.staked.toString(),
      aeth: (Number(data.staked) / 1e9).toFixed(9),
      formatted: formatAether(data.staked),
      percentage: ((Number(data.staked) / Number(data.total)) * 100).toFixed(2),
    },
    epoch: data.epoch,
    slot: data.slot,
    rpc,
    fetched_at: data.fetchedAt,
    cli_version: CLI_VERSION,
    sdk: '@jellylegsai/aether-sdk',
  };
  console.log(JSON.stringify(output, null, 2));
}

// ============================================================================
// Watch Mode
// ============================================================================

async function watchMode(rpc, compare) {
  const clearScreen = () => {
    process.stdout.write('\x1Bc');
  };

  let iteration = 0;
  const spinnerFrames = ['◐', '◓', '◑', '◒'];

  while (true) {
    try {
      clearScreen();
      const data = await fetchSupplyData(rpc);

      console.log(SUPPLY_LOGO);
      console.log();
      console.log(`  ${C.dim}Watch mode enabled | Update ${iteration + 1} | Press Ctrl+C to exit${C.reset}`);
      console.log();

      // Simple inline display for watch mode
      const totalAeth = Number(data.total) / 1e9;
      const circAeth = Number(data.circulating) / 1e9;
      const ncAeth = Number(data.nonCirculating) / 1e9;
      const stakedAeth = Number(data.staked) / 1e9;

      console.log(`  ${C.bright}Total:${C.reset}        ${C.cyan}${formatAether(data.total)}${C.reset}`);
      console.log(`  ${C.bright}Circulating:${C.reset}  ${C.green}${formatAether(data.circulating)}${C.reset} (${((circAeth/totalAeth)*100).toFixed(2)}%)`);
      console.log(`  ${C.bright}Non-Circ:${C.reset}     ${C.yellow}${formatAether(data.nonCirculating)}${C.reset} (${((ncAeth/totalAeth)*100).toFixed(2)}%)`);
      console.log(`  ${C.bright}Staked:${C.reset}       ${C.magenta}${formatAether(data.staked)}${C.reset} (${((stakedAeth/totalAeth)*100).toFixed(2)}%)`);
      console.log(`  ${C.bright}Epoch:${C.reset}        ${C.bright}${data.epoch}${C.reset}`);
      console.log(`  ${C.dim}Last update: ${data.fetchedAt}${C.reset}`);

      if (compare) {
        const pctOfMax = (totalAeth / MAX_SUPPLY_AETH) * 100;
        console.log();
        console.log(`  ${C.dim}% of Max Supply: ${C.bright}${pctOfMax.toFixed(4)}%${C.reset}`);
      }

      console.log();
      console.log(`  ${C.dim}${spinnerFrames[iteration % 4]} Waiting ${WATCH_INTERVAL_MS/1000}s for next update...${C.reset}`);

      iteration++;
      await new Promise(r => setTimeout(r, WATCH_INTERVAL_MS));
    } catch (err) {
      console.log(`\n  ${error('Watch mode error:')} ${err.message}`);
      console.log(`  ${dim('Retrying in 5s...')}`);
      await new Promise(r => setTimeout(r, WATCH_INTERVAL_MS));
    }
  }
}

// ============================================================================
// Main Command
// ============================================================================

async function supplyCommand() {
  const opts = parseArgs();

  if (opts.help) {
    showHelp();
    return;
  }

  // Handle watch mode
  if (opts.watch) {
    console.log(`${info('Starting watch mode... Press Ctrl+C to exit')}`);
    await watchMode(opts.rpc, opts.compare);
    return;
  }

  if (!opts.asJson) {
    startSpinner('Fetching supply data via SDK');
  }

  try {
    const data = await fetchSupplyData(opts.rpc);

    if (!opts.asJson) {
      stopSpinner(true, 'Supply data retrieved');
    }

    if (opts.asJson) {
      renderJson(data, opts.rpc);
    } else {
      renderSupplyBox(data, opts);
    }
  } catch (err) {
    if (!opts.asJson) {
      stopSpinner(false, 'Failed');
    }

    if (opts.asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc: opts.rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${error('Supply query failed:')} ${err.message}\n`);
      console.log(`  ${dim('Troubleshooting:')}`);
      console.log(`    • Is your validator running? ${C.cyan}aether ping${C.reset}`);
      console.log(`    • Check RPC endpoint: ${C.dim}${opts.rpc}${C.reset}`);
      console.log(`    • Set custom RPC: ${C.dim}AETHER_RPC=https://your-rpc-url${C.reset}`);
      console.log();
    }
    process.exit(1);
  }
}

// ============================================================================
// Exports
// ============================================================================

module.exports = { supplyCommand };

if (require.main === module) {
  supplyCommand().catch(err => {
    console.error(`\n${C.red}✗ Supply command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
