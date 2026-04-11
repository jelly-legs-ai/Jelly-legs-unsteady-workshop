#!/usr/bin/env node
/**
 * aether-cli validators
 *
 * List and manage validators on the Aether network.
 * Fully wired to @jellylegsai/aether-sdk for real blockchain RPC calls.
 *
 * Usage:
 *   aether validators list [--tier full|lite|observer] [--json]
 *   aether validators info <address> [--json]
 *   aether validators top [--limit 10] [--json]
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// Import UI framework for consistent branding
const { BRANDING, C, indicators, drawBox, drawTable,
        success, error, warning, info, code, highlight, value,
        startSpinner, stopSpinner, progressBar, progressBarColored,
        formatHelp } = require('../lib/ui');

const CLI_VERSION = '2.0.0';

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
// Config Helpers
// ============================================================================

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function loadConfig() {
  if (!fs.existsSync(getConfigPath())) {
    return { defaultWallet: null };
  }
  try {
    return JSON.parse(fs.readFileSync(getConfigPath(), 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

// ============================================================================
// Format Helpers
// ============================================================================

function formatAether(lamports) {
  if (!lamports || lamports === '0') return '0 AETH';
  const aeth = Number(lamports) / 1e9;
  if (aeth >= 1_000_000) {
    return (aeth / 1_000_000).toFixed(2) + 'M AETH';
  }
  if (aeth >= 1_000) {
    return (aeth / 1_000).toFixed(2) + 'K AETH';
  }
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatNumber(n) {
  if (n === null || n === undefined) return 'N/A';
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function shortAddress(addr) {
  if (!addr || addr.length < 16) return addr || 'unknown';
  return addr.slice(0, 8) + '...' + addr.slice(-8);
}

function formatPercent(val) {
  if (val === undefined || val === null) return 'N/A';
  const pct = Number(val);
  if (isNaN(pct)) return 'N/A';
  return pct.toFixed(2) + '%';
}

function formatAPY(apy) {
  if (!apy && apy !== 0) return `${C.dim}N/A${C.reset}`;
  const val = Number(apy);
  if (isNaN(val)) return `${C.dim}N/A${C.reset}`;
  if (val >= 10) return `${C.green}${val.toFixed(2)}%${C.reset}`;
  if (val >= 5) return `${C.yellow}${val.toFixed(2)}%${C.reset}`;
  return `${C.cyan}${val.toFixed(2)}%${C.reset}`;
}

// ============================================================================
// Fetch Validators via SDK
// ============================================================================

async function fetchValidators(rpcUrl) {
  const client = createClient(rpcUrl);
  
  try {
    const validators = await client.getValidators();
    if (!Array.isArray(validators)) return [];
    
    return validators.map(v => ({
      address: v.vote_account || v.pubkey || v.address || v.identity || 'unknown',
      identity: v.identity || v.node_pubkey || v.address,
      stake: v.stake_lamports || v.activated_stake || v.stake || 0,
      commission: v.commission || v.commission_bps || 0,
      apy: v.apy || v.return_rate || v.estimated_apy || 0,
      name: v.name || v.moniker || v.identity_name || 'Unknown',
      tier: v.tier || 'unknown',
      active: v.active !== false && v.delinquent !== true,
      delinquent: v.delinquent === true,
      skipRate: v.skip_rate || 0,
      uptime: v.uptime || 0,
      lastVote: v.last_vote || v.last_vote_slot || 0,
      version: v.version || 'unknown',
      website: v.website || null,
      details: v.details || null,
    }));
  } catch (err) {
    return [];
  }
}

async function fetchValidatorInfo(rpcUrl, address) {
  const validators = await fetchValidators(rpcUrl);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;
  return validators.find(v => 
    v.address === address || 
    v.address === rawAddr ||
    v.identity === address ||
    v.identity === rawAddr
  ) || null;
}

// ============================================================================
// Validator Status Indicator
// ============================================================================

function getStatusIndicator(validator) {
  if (validator.delinquent) {
    return `${C.red}● DELINQUENT${C.reset}`;
  }
  if (validator.active) {
    return `${C.green}● ACTIVE${C.reset}`;
  }
  return `${C.yellow}● INACTIVE${C.reset}`;
}

function getTierBadge(tier) {
  const tiers = {
    full: `${C.magenta}[FULL]${C.reset}`,
    lite: `${C.cyan}[LITE]${C.reset}`,
    observer: `${C.dim}[OBSERVER]${C.reset}`,
  };
  return tiers[tier?.toLowerCase()] || `${C.dim}[${tier?.toUpperCase() || 'UNKNOWN'}]${C.reset}`;
}

// ============================================================================
// List Command
// ============================================================================

async function listCommand(opts) {
  if (!opts.json) {
    console.log(BRANDING.header(CLI_VERSION));
    console.log();
    console.log(drawBox(
      `${C.bright}VALIDATORS${C.reset} — Active validators on the Aether network`,
      { style: 'double', title: 'NETWORK', titleColor: C.cyan }
    ));
    console.log();
    startSpinner('Fetching validators from ' + shortAddress(opts.rpc));
  }

  const validators = await fetchValidators(opts.rpc);

  if (!opts.json) {
    stopSpinner(true, `Found ${validators.length} validators`);
  }

  // Filter by tier if specified
  let filtered = validators;
  if (opts.tier && opts.tier !== 'all') {
    filtered = validators.filter(v => v.tier?.toLowerCase() === opts.tier.toLowerCase());
  }

  // Sort by stake (descending)
  filtered.sort((a, b) => b.stake - a.stake);

  if (filtered.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'No validators found',
        rpc: opts.rpc,
        tier_filter: opts.tier,
      }, null, 2));
    } else {
      console.log(`\n  ${warning('No validators found.')}`);
      if (opts.tier) {
        console.log(`  ${C.dim}Tier filter: ${opts.tier}${C.reset}`);
      }
      console.log(`  ${C.dim}Check your RPC endpoint: ${opts.rpc}${C.reset}\n`);
    }
    return;
  }

  // Calculate stats
  const totalStake = filtered.reduce((sum, v) => sum + Number(v.stake), 0);
  const activeCount = filtered.filter(v => v.active && !v.delinquent).length;
  const delinquentCount = filtered.filter(v => v.delinquent).length;
  const avgAPY = filtered.length > 0 
    ? filtered.reduce((sum, v) => sum + Number(v.apy || 0), 0) / filtered.length 
    : 0;

  if (opts.json) {
    console.log(JSON.stringify({
      success: true,
      count: filtered.length,
      total_validators: validators.length,
      tier_filter: opts.tier || 'all',
      stats: {
        total_stake_lamports: totalStake,
        total_stake_aeth: totalStake / 1e9,
        active_count: activeCount,
        delinquent_count: delinquentCount,
        average_apy: avgAPY,
      },
      validators: filtered.map((v, i) => ({
        rank: i + 1,
        address: v.address,
        name: v.name,
        tier: v.tier,
        stake_lamports: v.stake,
        stake_aeth: v.stake / 1e9,
        commission: v.commission,
        apy: v.apy,
        active: v.active,
        delinquent: v.delinquent,
        skip_rate: v.skipRate,
        uptime: v.uptime,
      })),
      rpc: opts.rpc,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  // Pretty output
  console.log();
  console.log(drawBox(
    `${C.dim}Network Stats${C.reset}\n\n` +
    `  ${C.cyan}Total Validators:${C.reset} ${C.bright}${formatNumber(filtered.length)}${C.reset}\n` +
    `  ${C.green}Active:${C.reset} ${C.bright}${formatNumber(activeCount)}${C.reset}  ` +
    `${C.red}Delinquent:${C.reset} ${C.bright}${formatNumber(delinquentCount)}${C.reset}\n` +
    `  ${C.yellow}Total Stake:${C.reset} ${C.bright}${formatAether(totalStake)}${C.reset}\n` +
    `  ${C.magenta}Avg APY:${C.reset} ${formatAPY(avgAPY)}`,
    { style: 'single', borderColor: C.dim }
  ));
  console.log();

  // Build table
  const headers = ['#', 'Status', 'Name', 'Address', 'Tier', 'Stake', 'APY', 'Comm'];
  const rows = filtered.slice(0, opts.limit).map((v, i) => {
    const status = v.delinquent ? indicators.error : (v.active ? indicators.success : indicators.warning);
    const name = (v.name || 'Unknown').slice(0, 16).padEnd(16);
    const addr = shortAddress(v.address);
    const tier = getTierBadge(v.tier);
    const stake = formatAether(v.stake);
    const apy = formatAPY(v.apy);
    const comm = formatPercent(v.commission / 100); // Assuming basis points
    return [`${i + 1}`, status, name, addr, tier, stake, apy, comm];
  });

  console.log(drawTable(headers, rows, {
    borderStyle: 'single',
    headerColor: C.cyan + C.bright,
    borderColor: C.dim,
  }));

  if (filtered.length > opts.limit) {
    console.log(`\n  ${C.dim}... and ${filtered.length - opts.limit} more validators${C.reset}`);
  }

  console.log(`\n  ${C.dim}To stake:${C.reset} ${code('aether stake --validator <address> --amount <aeth>')}`);
  console.log(`  ${C.dim}For details:${C.reset} ${code('aether validators info <address>')}`);
  console.log();
}

// ============================================================================
// Info Command
// ============================================================================

async function infoCommand(opts) {
  if (!opts.address) {
    if (!opts.json) {
      console.log(`\n  ${error('Validator address required.')}`);
      console.log(`  ${C.dim}Usage: aether validators info <address>${C.reset}\n`);
    } else {
      console.log(JSON.stringify({ error: 'Validator address required' }, null, 2));
    }
    return;
  }

  if (!opts.json) {
    console.log(BRANDING.header(CLI_VERSION));
    console.log();
    startSpinner('Fetching validator info');
  }

  const validator = await fetchValidatorInfo(opts.rpc, opts.address);

  if (!opts.json) {
    stopSpinner(!!validator, validator ? 'Validator found' : 'Validator not found');
  }

  if (!validator) {
    if (opts.json) {
      console.log(JSON.stringify({
        success: false,
        error: 'Validator not found',
        address: opts.address,
        rpc: opts.rpc,
      }, null, 2));
    } else {
      console.log(`\n  ${error('Validator not found:')} ${opts.address}`);
      console.log(`  ${C.dim}Check the address and try again.${C.reset}\n`);
    }
    return;
  }

  if (opts.json) {
    console.log(JSON.stringify({
      success: true,
      validator: {
        address: validator.address,
        identity: validator.identity,
        name: validator.name,
        tier: validator.tier,
        stake_lamports: validator.stake,
        stake_aeth: validator.stake / 1e9,
        commission: validator.commission,
        apy: validator.apy,
        active: validator.active,
        delinquent: validator.delinquent,
        skip_rate: validator.skipRate,
        uptime: validator.uptime,
        last_vote: validator.lastVote,
        version: validator.version,
        website: validator.website,
        details: validator.details,
      },
      rpc: opts.rpc,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  // Pretty output
  const statusLine = getStatusIndicator(validator);
  const uptimeBar = validator.uptime 
    ? progressBarColored(validator.uptime, 100, 30)
    : `${C.dim}N/A${C.reset}`;

  console.log();
  console.log(drawBox(
    `${C.bright}VALIDATOR DETAILS${C.reset}\n\n` +
    `  ${C.cyan}Status:${C.reset}     ${statusLine}\n` +
    `  ${C.cyan}Name:${C.reset}       ${C.bright}${validator.name || 'Unknown'}${C.reset}\n` +
    `  ${C.cyan}Tier:${C.reset}       ${getTierBadge(validator.tier)}\n\n` +
    `  ${C.cyan}Address:${C.reset}    ${C.bright}${validator.address}${C.reset}\n` +
    `  ${C.cyan}Identity:${C.reset}   ${validator.identity || 'N/A'}\n\n` +
    `  ${C.cyan}Stake:${C.reset}      ${C.bright}${formatAether(validator.stake)}${C.reset}\n` +
    `  ${C.cyan}APY:${C.reset}        ${formatAPY(validator.apy)}\n` +
    `  ${C.cyan}Commission:${C.reset} ${formatPercent(validator.commission / 100)}\n\n` +
    `  ${C.cyan}Uptime:${C.reset}     ${uptimeBar}\n` +
    `  ${C.cyan}Skip Rate:${C.reset}  ${formatPercent(validator.skipRate)}\n` +
    `  ${C.cyan}Version:${C.reset}    ${validator.version || 'unknown'}`,
    { style: 'double', title: validator.name?.toUpperCase() || 'VALIDATOR', titleColor: C.cyan }
  ));

  if (validator.website) {
    console.log(`\n  ${C.dim}Website:${C.reset} ${C.blue}${validator.website}${C.reset}`);
  }
  if (validator.details) {
    console.log(`\n  ${C.dim}Details:${C.reset} ${validator.details}`);
  }

  console.log(`\n  ${C.dim}To stake:${C.reset} ${code(`aether stake --validator ${validator.address} --amount <aeth>`)}`);
  console.log();
}

// ============================================================================
// Top Command
// ============================================================================

async function topCommand(opts) {
  if (!opts.json) {
    console.log(BRANDING.header(CLI_VERSION));
    console.log();
    console.log(drawBox(
      `${C.bright}TOP VALIDATORS${C.reset} — Highest stake validators`,
      { style: 'double', title: 'LEADERBOARD', titleColor: C.yellow }
    ));
    console.log();
    startSpinner('Fetching top validators');
  }

  const validators = await fetchValidators(opts.rpc);

  // Sort by stake and take top N
  validators.sort((a, b) => b.stake - a.stake);
  const topValidators = validators.slice(0, opts.limit);

  if (!opts.json) {
    stopSpinner(topValidators.length > 0, `Found ${topValidators.length} validators`);
  }

  if (topValidators.length === 0) {
    if (opts.json) {
      console.log(JSON.stringify({ error: 'No validators found' }, null, 2));
    } else {
      console.log(`\n  ${warning('No validators found.')}\n`);
    }
    return;
  }

  // Calculate total stake for percentages
  const totalNetworkStake = validators.reduce((sum, v) => sum + Number(v.stake), 0);

  if (opts.json) {
    console.log(JSON.stringify({
      success: true,
      top_count: topValidators.length,
      total_network_stake: totalNetworkStake,
      validators: topValidators.map((v, i) => ({
        rank: i + 1,
        address: v.address,
        name: v.name,
        stake_lamports: v.stake,
        stake_aeth: v.stake / 1e9,
        stake_percentage: totalNetworkStake > 0 ? (v.stake / totalNetworkStake * 100).toFixed(2) : 0,
        apy: v.apy,
        commission: v.commission,
        active: v.active,
      })),
      rpc: opts.rpc,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  console.log();
  console.log(`  ${C.dim}Network:${C.reset} ${C.bright}${formatAether(totalNetworkStake)}${C.reset} total stake`);
  console.log(`  ${C.dim}Showing top ${opts.limit} validators${C.reset}\n`);

  // Build leaderboard table
  const headers = ['Rank', 'Validator', 'Stake', '% Network', 'APY'];
  const rows = topValidators.map((v, i) => {
    const rank = i + 1;
    const rankIcon = rank === 1 ? `${C.yellow}🥇${C.reset}` : 
                     rank === 2 ? `${C.dim}🥈${C.reset}` : 
                     rank === 3 ? `${C.brightYellow}🥉${C.reset}` : 
                     `${C.dim}${rank}${C.reset}`;
    const name = (v.name || shortAddress(v.address)).slice(0, 20);
    const stake = formatAether(v.stake);
    const pct = totalNetworkStake > 0 ? (v.stake / totalNetworkStake * 100).toFixed(2) + '%' : '0%';
    const apy = formatAPY(v.apy);
    return [rankIcon, name, stake, pct, apy];
  });

  console.log(drawTable(headers, rows, {
    borderStyle: 'single',
    headerColor: C.yellow + C.bright,
    borderColor: C.dim,
  }));

  console.log();
}

// ============================================================================
// Show Help
// ============================================================================

function showHelp() {
  console.log(BRANDING.header(CLI_VERSION));
  
  console.log(`\n  ${C.bright}AETHER VALIDATORS${C.reset} — Network validator management\n`);
  
  console.log(`  ${C.cyan}◆ LIST${C.reset}     ${code('aether validators list [--tier <tier>] [--json]')}`);
  console.log(`    ${C.dim}Show all validators, optionally filtered by tier${C.reset}`);
  
  console.log(`\n  ${C.cyan}◆ INFO${C.reset}     ${code('aether validators info <address> [--json]')}`);
  console.log(`    ${C.dim}Detailed information about a specific validator${C.reset}`);
  
  console.log(`\n  ${C.cyan}◆ TOP${C.reset}      ${code('aether validators top [--limit 10] [--json]')}`);
  console.log(`    ${C.dim}Show top validators by stake amount${C.reset}`);
  
  console.log(`\n  ${C.bright}OPTIONS${C.reset}`);
  console.log(`    ${code('--tier <tier>')}    Filter by tier: full, lite, observer`);
  console.log(`    ${code('--limit <n>')}      Show top N validators (default: 15)`);
  console.log(`    ${code('--rpc <url>')}      Custom RPC endpoint`);
  console.log(`    ${code('--json')}           Output JSON for scripting`);
  console.log(`    ${code('--help')}           Show this help message`);
  
  console.log(`\n  ${C.bright}EXAMPLES${C.reset}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether validators list')}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether validators list --tier full --json')}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether validators info ATHxxx...')}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether validators top --limit 20')}`);
  
  console.log(`\n  ${C.bright}SDK METHODS${C.reset}`);
  console.log(`    ${C.dim}client.getValidators()  → GET /v1/validators${C.reset}`);
  console.log();
}

// ============================================================================
// CLI Args Parser
// ============================================================================

function parseArgs() {
  const rawArgs = process.argv.slice(3);
  
  // Determine subcommand - first arg if not a flag
  let subcmd = 'list';
  let allArgs = rawArgs;
  
  if (rawArgs.length > 0 && !rawArgs[0].startsWith('-')) {
    subcmd = rawArgs[0];
    allArgs = rawArgs.slice(1);
  }
  
  const opts = {
    subcmd,
    rpc: getDefaultRpc(),
    json: false,
    tier: null,
    limit: 15,
    address: null,
  };

  // Parse flags
  for (let i = 0; i < allArgs.length; i++) {
    const arg = allArgs[i];
    if (arg === '--rpc' || arg === '-r') {
      opts.rpc = allArgs[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--tier' || arg === '-t') {
      opts.tier = allArgs[++i];
    } else if (arg === '--limit' || arg === '-l') {
      const val = parseInt(allArgs[++i], 10);
      if (!isNaN(val) && val > 0) opts.limit = val;
    } else if (arg === '--help' || arg === '-h') {
      opts.subcmd = 'help';
    } else if (!arg.startsWith('-') && subcmd === 'info') {
      // For info command, non-flag arg is the address
      opts.address = arg;
    }
  }

  // For info command, also check if address was passed directly
  if (subcmd === 'info' && !opts.address) {
    const firstArg = allArgs.find(a => !a.startsWith('-'));
    if (firstArg) opts.address = firstArg;
  }

  return opts;
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function validatorsCommand() {
  const opts = parseArgs();

  switch (opts.subcmd) {
    case 'list':
      await listCommand(opts);
      break;
    case 'info':
      await infoCommand(opts);
      break;
    case 'top':
      await topCommand(opts);
      break;
    case 'help':
    case '--help':
    case '-h':
      showHelp();
      break;
    default:
      console.log(`\n  ${error('Unknown subcommand:')} ${opts.subcmd}`);
      console.log(`  ${C.dim}Run 'aether validators --help' for usage.${C.reset}\n`);
      process.exit(1);
  }
}

module.exports = { validatorsCommand };

if (require.main === module) {
  validatorsCommand().catch(err => {
    console.error(`\n${error('Validators command failed:')} ${err.message}\n`);
    process.exit(1);
  });
}
