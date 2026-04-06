#!/usr/bin/env node
/**
 * aether-cli validator-info
 *
 * Display detailed information about a specific validator including:
 *   - APY (Annual Percentage Yield)
 *   - Commission rate
 *   - Total stake
 *   - Uptime and performance metrics
 *   - Validator metadata
 *
 * Usage:
 *   aether validator-info --address <validator_addr> [--json] [--rpc <url>]
 *   aether validator-info <validator_addr>
 *
 * Examples:
 *   aether validator-info ATHabc...
 *   aether validator-info --address ATHabc... --json
 *   aether validator-info ATHabc... --rpc http://localhost:8899
 *
 * SDK wired to:
 *   - GET /v1/validator/<address>/apy (via client.getValidatorAPY())
 *   - GET /v1/validators (via client.getValidators() for full list filtering)
 */

const path = require('path');

// Import SDK for blockchain RPC calls
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

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, json: false, rpc: null };

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      result.address = args[++i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      result.json = true;
    } else if (args[i] === '--rpc' && args[i + 1]) {
      result.rpc = args[++i];
    } else if (args[i] === '--help' || args[i] === '-h') {
      result.help = true;
    } else if (!result.address && !args[i].startsWith('-')) {
      // Positional argument for address
      result.address = args[i];
    }
  }

  return result;
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  const aeth = (lamports || 0) / 1e9;
  return aeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 }) + ' AETH';
}

function formatPercent(value) {
  if (value === null || value === undefined) return 'N/A';
  if (typeof value === 'string') value = parseFloat(value);
  if (isNaN(value)) return 'N/A';
  return value.toFixed(2) + '%';
}

function formatUptime(seconds) {
  if (!seconds) return 'N/A';
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

function shortPubkey(pubkey, len = 8) {
  if (!pubkey || pubkey.length < 16) return pubkey || 'unknown';
  return pubkey.slice(0, len) + '...' + pubkey.slice(-len);
}

function getStatusColor(status) {
  const s = (status || '').toLowerCase();
  if (s === 'active') return C.green;
  if (s === 'delinquent') return C.red;
  if (s === 'inactive') return C.dim;
  if (s === 'jailed') return C.red;
  return C.yellow;
}

// ---------------------------------------------------------------------------
// RPC fetchers - Real blockchain calls via SDK
// ---------------------------------------------------------------------------

/**
 * Fetch validator APY via SDK (GET /v1/validator/<address>/apy)
 */
async function fetchValidatorAPY(client, address) {
  try {
    const result = await client.getValidatorAPY(address);
    return result;
  } catch (err) {
    return { error: err.message };
  }
}

/**
 * Fetch all validators and find the specific one
 * Uses SDK's getValidators() which calls GET /v1/validators
 */
async function fetchValidatorInfo(client, address) {
  try {
    const validators = await client.getValidators();
    const validator = validators.find(v => 
      v.address === address || 
      v.pubkey === address || 
      v.id === address
    );
    return validator || null;
  } catch (err) {
    return { error: err.message };
  }
}

/**
 * Fetch current slot for context
 */
async function fetchCurrentSlot(client) {
  try {
    return await client.getSlot();
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function validatorInfoCommand() {
  const opts = parseArgs();

  if (opts.help) {
    console.log(`
${C.bright}${C.cyan}validator-info${C.reset} — Display detailed information about a validator

${C.bright}USAGE${C.reset}
    aether validator-info <address> [--json] [--rpc <url>]
    aether validator-info --address <addr> [--json] [--rpc <url>]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Validator address (ATH...)
    --json              Output raw JSON
    --rpc <url>         RPC endpoint (default: AETHER_RPC or localhost:8899)
    --help              Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getValidatorAPY()   → GET /v1/validator/<address>/apy
    client.getValidators()     → GET /v1/validators
    client.getSlot()           → GET /v1/slot

${C.bright}EXAMPLES${C.reset}
    aether validator-info ATH3abc...
    aether validator-info ATH3abc... --json
    aether validator-info --address ATH3abc... --rpc http://localhost:8899
`);
    return;
  }

  if (!opts.address) {
    console.log(`  ${C.red}✗ Missing validator address${C.reset}\n`);
    console.log(`  Usage: aether validator-info <address> [--json] [--rpc <url>]\n`);
    process.exit(1);
  }

  const rpcUrl = opts.rpc || getDefaultRpc();
  const client = createClient(rpcUrl);
  const address = opts.address;
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Validator Information ───────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Validator:${C.reset} ${C.bright}${address}${C.reset}`);
    console.log(`  ${C.dim}RPC:      ${C.reset} ${rpcUrl}\n`);
    console.log(`  ${C.dim}Fetching data from chain...${C.reset}\n`);
  }

  try {
    // Fetch data in parallel
    const [apyData, validatorInfo, currentSlot] = await Promise.all([
      fetchValidatorAPY(client, rawAddr),
      fetchValidatorInfo(client, rawAddr),
      fetchCurrentSlot(client),
    ]);

    // Check for errors
    if (apyData.error && !validatorInfo) {
      throw new Error(`Validator not found: ${apyData.error}`);
    }

    // Build response object
    const response = {
      address: address,
      raw_address: rawAddr,
      rpc: rpcUrl,
      slot: currentSlot,
      apy: null,
      commission: null,
      stake: null,
      status: null,
      uptime: null,
      votes: null,
      credits: null,
      last_vote: null,
      fetched_at: new Date().toISOString(),
      cli_version: CLI_VERSION,
    };

    // Extract APY data
    if (!apyData.error) {
      response.apy = apyData.apy ?? apyData.current_apy ?? apyData.estimated_apy ?? null;
      response.commission = apyData.commission ?? null;
    }

    // Extract validator info
    if (validatorInfo && !validatorInfo.error) {
      response.stake = validatorInfo.stake_lamports ?? validatorInfo.stake ?? validatorInfo.activated_stake ?? null;
      response.status = validatorInfo.status ?? validatorInfo.state ?? 'unknown';
      response.uptime = validatorInfo.uptime_seconds ?? validatorInfo.uptime ?? null;
      response.votes = validatorInfo.votes ?? validatorInfo.vote_count ?? null;
      response.credits = validatorInfo.credits ?? validatorInfo.credit_count ?? null;
      response.last_vote = validatorInfo.last_vote ?? validatorInfo.last_vote_slot ?? null;
      
      // Use commission from validator info if not in APY data
      if (response.commission === null) {
        response.commission = validatorInfo.commission ?? null;
      }

      // Additional metadata
      response.identity = validatorInfo.identity ?? validatorInfo.node_id ?? null;
      response.name = validatorInfo.name ?? validatorInfo.moniker ?? null;
      response.website = validatorInfo.website ?? null;
      response.details = validatorInfo.details ?? validatorInfo.description ?? null;
    }

    // JSON output
    if (opts.json) {
      console.log(JSON.stringify(response, null, 2));
      return;
    }

    // Pretty output
    const statusColor = getStatusColor(response.status);
    const statusText = (response.status || 'UNKNOWN').toUpperCase();

    console.log(`  ${C.bright}┌─────────────────────────────────────────────────────────┐${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Validator${C.reset}${' '.repeat(43)}${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}│${C.reset}  ${C.bright}${address.slice(0, 40).padEnd(53)}${C.reset}${C.bright}│${C.reset}`);
    console.log(`  ${C.bright}├${C.reset}${'─'.repeat(58)}${C.bright}│${C.reset}`);
    
    // Status
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Status:${C.reset}  ${statusColor}${statusText.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    
    // APY
    if (response.apy !== null) {
      const apyStr = formatPercent(response.apy);
      const apyColor = response.apy >= 7 ? C.green : response.apy >= 4 ? C.yellow : C.dim;
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}APY:${C.reset}     ${apyColor}${apyStr.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Commission
    if (response.commission !== null) {
      const commStr = formatPercent(response.commission);
      const commColor = response.commission <= 5 ? C.green : response.commission <= 10 ? C.yellow : C.red;
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Commission:${C.reset} ${commColor}${commStr.padEnd(42)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Stake
    if (response.stake !== null) {
      const stakeStr = formatAether(response.stake);
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Stake:${C.reset}   ${C.green}${stakeStr.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Uptime
    if (response.uptime !== null) {
      const uptimeStr = formatUptime(response.uptime);
      const uptimeVal = typeof response.uptime === 'number' ? response.uptime : 0;
      const uptimeColor = uptimeVal > 95 ? C.green : uptimeVal > 80 ? C.yellow : C.red;
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Uptime:${C.reset}  ${uptimeColor}${uptimeStr.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Votes
    if (response.votes !== null) {
      const votesStr = response.votes.toLocaleString();
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Votes:${C.reset}   ${C.cyan}${votesStr.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Credits
    if (response.credits !== null) {
      const creditsStr = response.credits.toLocaleString();
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Credits:${C.reset} ${C.magenta}${creditsStr.padEnd(44)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Last vote
    if (response.last_vote !== null) {
      const lastVoteStr = response.last_vote.toLocaleString();
      const slotDiff = currentSlot ? currentSlot - response.last_vote : null;
      const slotDiffStr = slotDiff !== null ? ` (${slotDiff} slots ago)` : '';
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Last Vote:${C.reset} ${C.blue}${lastVoteStr.padEnd(42 - slotDiffStr.length)}${C.reset}${C.dim}${slotDiffStr}${C.reset}${C.bright}│${C.reset}`);
    }

    // Identity
    if (response.identity) {
      const idStr = shortPubkey(response.identity, 10);
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Identity:${C.reset} ${C.dim}${idStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
    }

    // Name
    if (response.name) {
      const nameStr = response.name.slice(0, 40);
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Name:${C.reset}    ${C.bright}${nameStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
    }

    console.log(`  ${C.bright}└─────────────────────────────────────────────────────────┘${C.reset}`);

    // Additional info
    if (response.website) {
      console.log(`\n  ${C.dim}Website:${C.reset} ${C.cyan}${response.website}${C.reset}`);
    }
    if (response.details) {
      console.log(`  ${C.dim}Details:${C.reset} ${response.details.slice(0, 80)}${response.details.length > 80 ? '...' : ''}`);
    }

    // Staking tip
    console.log(`\n  ${C.dim}Tip: Stake with this validator using${C.reset}`);
    console.log(`      ${C.cyan}aether stake --validator ${shortPubkey(address, 6)} --amount <AETH>${C.reset}\n`);

  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        address: address,
        error: err.message,
        rpc: rpcUrl,
        fetched_at: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to fetch validator info:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    }
    process.exit(1);
  }
}

module.exports = { validatorInfoCommand };

if (require.main === module) {
  validatorInfoCommand().catch(err => {
    console.error(`\n  ${C.red}✗ Error:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
