#!/usr/bin/env node
/**
 * aether-cli validator-info
 *
 * Display detailed information about a specific validator including:
 *   - APY (Annual Percentage Yield)
 *   - Commission rate
 *   - Total stake
 *   - Stake positions and delegations
 *   - Rewards earned
 *   - Uptime and performance metrics
 *   - Validator metadata
 *   - Current epoch participation
 *
 * Usage:
 *   aether validator-info --address <validator_addr> [--json] [--rpc <url>]
 *   aether validator-info <validator_addr>
 *   aether validator-info --address <addr> --delegations
 *   aether validator-info --address <addr> --rewards
 *
 * Examples:
 *   aether validator-info ATHabc...
 *   aether validator-info ATHabc... --json
 *   aether validator-info ATHabc... --rpc http://localhost:8899
 *   aether validator-info ATHabc... --delegations --limit 50
 *
 * SDK wired to:
 *   - GET /v1/validator/<address>/apy (via client.getValidatorAPY())
 *   - GET /v1/validators (via client.getValidators())
 *   - GET /v1/stake/<address> (via client.getStakePositions())
 *   - GET /v1/rewards/<address> (via client.getRewards())
 *   - GET /v1/epoch (via client.getEpochInfo())
 *   - GET /v1/slot (via client.getSlot())
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

const CLI_VERSION = '1.2.0';

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
  const result = { 
    address: null, 
    json: false, 
    rpc: null, 
    showDelegations: false,
    showRewards: false,
    limit: 20,
  };

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      result.address = args[++i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      result.json = true;
    } else if (args[i] === '--rpc' && args[i + 1]) {
      result.rpc = args[++i];
    } else if (args[i] === '--delegations' || args[i] === '-d') {
      result.showDelegations = true;
    } else if (args[i] === '--rewards' || args[i] === '-r') {
      result.showRewards = true;
    } else if (args[i] === '--limit' || args[i] === '-l') {
      const val = parseInt(args[++i], 10);
      if (!isNaN(val) && val > 0) result.limit = val;
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

function formatUptimePercent(value) {
  if (value === null || value === undefined) return 'N/A';
  const pct = typeof value === 'number' ? value : parseFloat(value);
  if (isNaN(pct)) return 'N/A';
  if (pct >= 99) return C.green + pct.toFixed(2) + '%' + C.reset;
  if (pct >= 95) return C.yellow + pct.toFixed(2) + '%' + C.reset;
  return C.red + pct.toFixed(2) + '%' + C.reset;
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

function getStakeStatusColor(status) {
  const s = (status || '').toLowerCase();
  if (s === 'active') return C.green;
  if (s === 'activating') return C.cyan;
  if (s === 'deactivating') return C.yellow;
  if (s === 'inactive') return C.dim;
  return C.reset;
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
      v.id === address ||
      v.vote_account === address
    );
    return validator || null;
  } catch (err) {
    return { error: err.message };
  }
}

/**
 * Fetch stake positions for the validator
 * Uses SDK's getStakePositions() which calls GET /v1/stake/<address>
 */
async function fetchStakePositions(client, address) {
  try {
    const result = await client.getStakePositions(address);
    return Array.isArray(result) ? result : [];
  } catch (err) {
    return { error: err.message };
  }
}

/**
 * Fetch rewards for the validator
 * Uses SDK's getRewards() which calls GET /v1/rewards/<address>
 */
async function fetchRewards(client, address) {
  try {
    const result = await client.getRewards(address);
    return result;
  } catch (err) {
    return { error: err.message };
  }
}

/**
 * Fetch current slot and epoch info
 */
async function fetchNetworkInfo(client) {
  try {
    const [slot, epochInfo] = await Promise.all([
      client.getSlot(),
      client.getEpochInfo().catch(() => null),
    ]);
    return { slot, epoch: epochInfo };
  } catch (err) {
    return { slot: null, epoch: null };
  }
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

function printValidatorHeader(address, rawAddr) {
  console.log(`\n${C.bright}${C.cyan}═══ Validator Information ════════════════════════════════${C.reset}\n`);
  console.log(`  ${C.dim}Address:${C.reset}  ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.dim}Raw:${C.reset}      ${C.dim}${rawAddr}${C.reset}\n`);
}

function printValidatorCard(info, apyData, networkInfo) {
  const statusColor = getStatusColor(info.status);
  const statusText = (info.status || 'UNKNOWN').toUpperCase();

  // Calculate stake percentage
  const totalNetworkStake = apyData?.total_network_stake || info?.total_network_stake;
  const stakePct = totalNetworkStake && info.stake_lamports 
    ? ((info.stake_lamports / totalNetworkStake) * 100).toFixed(4)
    : null;

  console.log(`  ${C.bright}┌─────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}★ Validator Overview${C.reset}${' '.repeat(40)}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├─────────────────────────────────────────────────────────┤${C.reset}`);
  
  // Status row
  console.log(`  ${C.bright}│${C.reset}  ${C.dim}Status:${C.reset}    ${statusColor}${statusText.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  
  // APY row
  const apy = apyData?.apy ?? apyData?.current_apy ?? info?.apy;
  if (apy !== null && apy !== undefined) {
    const apyStr = formatPercent(apy);
    const apyColor = apy >= 7 ? C.green : apy >= 4 ? C.yellow : C.dim;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}APY:${C.reset}       ${apyColor}${apyStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Commission row
  const commission = apyData?.commission ?? info?.commission ?? info?.commission_bps;
  if (commission !== null && commission !== undefined) {
    const commValue = typeof commission === 'number' && commission > 100 
      ? commission / 100  // Handle basis points
      : commission;
    const commStr = formatPercent(commValue);
    const commColor = commValue <= 5 ? C.green : commValue <= 10 ? C.yellow : C.red;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Commission:${C.reset}  ${commColor}${commStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Stake row
  const stakeLamports = info?.stake_lamports ?? info?.stake ?? info?.activated_stake;
  if (stakeLamports !== null && stakeLamports !== undefined) {
    const stakeStr = formatAether(stakeLamports);
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Total Stake:${C.reset} ${C.green}${stakeStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
    if (stakePct) {
      console.log(`  ${C.bright}│${C.reset}  ${C.dim}Network %:${C.reset}   ${C.cyan}${(stakePct + '%').padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
    }
  }

  // Uptime row
  const uptime = info?.uptime_seconds ?? info?.uptime;
  if (uptime !== null && uptime !== undefined) {
    const uptimeStr = formatUptime(uptime);
    const uptimeVal = typeof uptime === 'number' ? uptime : parseFloat(uptime);
    const uptimeColor = uptimeVal > 95 ? C.green : uptimeVal > 80 ? C.yellow : C.red;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Uptime:${C.reset}    ${uptimeColor}${uptimeStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Votes row
  const votes = info?.votes ?? info?.vote_count;
  if (votes !== null && votes !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Votes:${C.reset}     ${C.cyan}${votes.toLocaleString().padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Credits row
  const credits = info?.credits ?? info?.credit_count;
  if (credits !== null && credits !== undefined) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Credits:${C.reset}   ${C.magenta}${credits.toLocaleString().padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Skip rate
  const skipRate = info?.skip_rate;
  if (skipRate !== null && skipRate !== undefined) {
    const skipStr = formatPercent(skipRate);
    const skipColor = skipRate < 2 ? C.green : skipRate < 10 ? C.yellow : C.red;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Skip Rate:${C.reset} ${skipColor}${skipStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Last vote
  const lastVote = info?.last_vote ?? info?.last_vote_slot;
  if (lastVote !== null && lastVote !== undefined && networkInfo?.slot) {
    const slotDiff = networkInfo.slot - lastVote;
    const lastVoteStr = `${lastVote.toLocaleString()} (${slotDiff} slots ago)`;
    const lvColor = slotDiff < 10 ? C.green : slotDiff < 100 ? C.yellow : C.red;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Last Vote:${C.reset} ${lvColor}${lastVoteStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Epoch info
  if (networkInfo?.epoch) {
    const epochStr = `Epoch ${networkInfo.epoch.epoch} (${networkInfo.epoch.slotIndex}/${networkInfo.epoch.slotsInEpoch} slots)`;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Epoch:${C.reset}     ${C.dim}${epochStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  // Identity
  const identity = info?.identity ?? info?.node_id;
  if (identity) {
    const idStr = shortPubkey(identity, 12);
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Identity:${C.reset}  ${C.dim}${idStr.padEnd(43)}${C.reset}${C.bright}│${C.reset}`);
  }

  console.log(`  ${C.bright}└─────────────────────────────────────────────────────────┘${C.reset}`);

  // Name and description
  const name = info?.name ?? info?.moniker;
  if (name) {
    console.log(`\n  ${C.bright}Name:${C.reset} ${name}`);
  }
  
  const details = info?.details ?? info?.description;
  if (details) {
    console.log(`  ${C.dim}${details.slice(0, 80)}${details.length > 80 ? '...' : ''}${C.reset}`);
  }

  // Website
  if (info?.website) {
    console.log(`  ${C.dim}Website:${C.reset} ${C.cyan}${info.website}${C.reset}`);
  }
}

function printStakePositions(stakePositions, limit) {
  if (!stakePositions || stakePositions.length === 0) return;

  const positions = stakePositions.slice(0, limit);

  console.log(`\n  ${C.bright}${C.cyan}── Stake Positions (${positions.length}/${stakePositions.length}) ─────────────────────${C.reset}\n`);

  let totalStaked = 0;
  
  positions.forEach((pos, i) => {
    const stakeAcct = pos.pubkey || pos.publicKey || pos.account || 'unknown';
    const validator = pos.validator || pos.delegate || pos.vote_account || 'unknown';
    const lamports = pos.lamports || pos.stake_lamports || 0;
    const status = pos.status || pos.state || 'unknown';
    const activationEpoch = pos.activation_epoch || pos.activationEpoch;
    const deactivationEpoch = pos.deactivation_epoch || pos.deactivationEpoch;
    const rewards = pos.rewards_earned || 0;

    totalStaked += lamports;

    const statusColor = getStakeStatusColor(status);
    const statusText = status.toUpperCase();

    console.log(`  ${C.dim}┌─${'─'.repeat(58)}┐${C.reset}`);
    console.log(`  ${C.dim}│${C.reset}  ${C.bright}#${i + 1}${C.reset} ${statusColor}[${statusText}]${C.reset}${' '.repeat(50 - statusText.length)}${C.dim}│${C.reset}`);
    console.log(`  ${C.dim}│${C.reset}  ${C.dim}Account:${C.reset}  ${shortPubkey(stakeAcct, 14).padEnd(40)}${C.dim}│${C.reset}`);
    console.log(`  ${C.dim}│${C.reset}  ${C.dim}Amount:${C.reset}   ${C.green}${formatAether(lamports).padEnd(40)}${C.reset}${C.dim}│${C.reset}`);
    
    if (validator !== 'unknown') {
      console.log(`  ${C.dim}│${C.reset}  ${C.dim}Delegate:${C.reset} ${shortPubkey(validator, 14).padEnd(40)}${C.dim}│${C.reset}`);
    }

    if (activationEpoch !== undefined) {
      console.log(`  ${C.dim}│${C.reset}  ${C.dim}Activated:${C.reset} epoch ${activationEpoch.toString().padEnd(33)}${C.dim}│${C.reset}`);
    }

    if (deactivationEpoch !== undefined) {
      console.log(`  ${C.dim}│${C.reset}  ${C.dim}Deactivates:${C.reset} epoch ${deactivationEpoch.toString().padEnd(31)}${C.dim}│${C.reset}`);
    }

    if (rewards > 0) {
      console.log(`  ${C.dim}│${C.reset}  ${C.dim}Rewards:${C.reset}  ${C.magenta}+${formatAether(rewards).padEnd(39)}${C.reset}${C.dim}│${C.reset}`);
    }

    console.log(`  ${C.dim}└${'─'.repeat(59)}┘${C.reset}`);
    console.log();
  });

  console.log(`  ${C.bright}Total Staked:${C.reset} ${C.green}${formatAether(totalStaked)}${C.reset}\n`);
}

function printRewards(rewardsData) {
  if (!rewardsData || rewardsData.error) return;

  console.log(`\n  ${C.bright}${C.cyan}── Rewards Summary ────────────────────────────────────────${C.reset}\n`);

  const total = rewardsData.total ?? rewardsData.total_rewards ?? rewardsData.lifetime_rewards ?? 0;
  const pending = rewardsData.pending ?? rewardsData.pending_rewards ?? rewardsData.claimable ?? 0;
  const claimed = rewardsData.claimed ?? rewardsData.claimed_rewards ?? (total - pending);
  const rewardsPerEpoch = rewardsData.rewards_per_epoch ?? rewardsData.epoch_rate ?? 0;

  console.log(`  ${C.bright}┌─────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}★ Rewards${C.reset}${' '.repeat(46)}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}├─────────────────────────────────────────────────────────┤${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.dim}Total Earned:${C.reset}   ${C.green}${formatAether(total).padEnd(37)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.dim}Claimed:${C.reset}        ${C.cyan}${formatAether(claimed).padEnd(37)}${C.reset}${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.dim}Pending Claim:${C.reset}  ${C.yellow}${formatAether(pending).padEnd(37)}${C.reset}${C.bright}│${C.reset}`);
  
  if (rewardsPerEpoch > 0) {
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Per Epoch:${C.reset}      ${C.magenta}${formatAether(rewardsPerEpoch).padEnd(37)}${C.reset}${C.bright}│${C.reset}`);
  }
  console.log(`  ${C.bright}└─────────────────────────────────────────────────────────┘${C.reset}`);

  if (pending > 0) {
    console.log(`\n  ${C.dim}💡 Tip: Claim pending rewards with:${C.reset}`);
    console.log(`      ${C.cyan}aether claim --address <your_wallet>${C.reset}\n`);
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
    aether validator-info <address> [options]
    aether validator-info --address <addr> [options]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Validator address (ATH...)
    -a <addr>
    --delegations, -d   Show stake delegations/positions
    --rewards, -r       Show rewards summary
    --limit <n>         Max delegations to show (default: 20)
    --json              Output raw JSON
    --rpc <url>         RPC endpoint (default: AETHER_RPC or localhost:8899)
    --help              Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getValidatorAPY()    → GET /v1/validator/<address>/apy
    client.getValidators()      → GET /v1/validators
    client.getStakePositions()  → GET /v1/stake/<address>
    client.getRewards()         → GET /v1/rewards/<address>
    client.getEpochInfo()       → GET /v1/epoch
    client.getSlot()            → GET /v1/slot

${C.bright}EXAMPLES${C.reset}
    aether validator-info ATH3abc...
    aether validator-info ATH3abc... --json
    aether validator-info ATH3abc... --delegations --limit 50
    aether validator-info --address ATH3abc... --rewards --rpc http://localhost:8899
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
    printValidatorHeader(address, rawAddr);
  }

  try {
    // Fetch data in parallel for efficiency
    const fetchPromises = [
      fetchValidatorAPY(client, rawAddr),
      fetchValidatorInfo(client, rawAddr),
      fetchNetworkInfo(client),
    ];

    // Conditionally fetch additional data
    if (opts.showDelegations) {
      fetchPromises.push(fetchStakePositions(client, rawAddr));
    }
    if (opts.showRewards) {
      fetchPromises.push(fetchRewards(client, rawAddr));
    }

    const results = await Promise.all(fetchPromises);
    const [apyData, validatorInfo, networkInfo] = results;
    
    // Extract optional results
    let stakePositions = null;
    let rewardsData = null;
    let resultIdx = 3;
    if (opts.showDelegations) {
      stakePositions = results[resultIdx++];
    }
    if (opts.showRewards) {
      rewardsData = results[resultIdx++];
    }

    // Check for critical errors
    if (apyData.error && !validatorInfo) {
      throw new Error(`Validator not found or RPC error: ${apyData.error}`);
    }

    // Build comprehensive response object for JSON output
    const response = {
      address: address,
      raw_address: rawAddr,
      rpc: rpcUrl,
      slot: networkInfo?.slot,
      epoch: networkInfo?.epoch,
      apy: null,
      commission: null,
      stake: null,
      status: null,
      uptime: null,
      votes: null,
      credits: null,
      skip_rate: null,
      last_vote: null,
      identity: null,
      name: null,
      website: null,
      details: null,
      stake_positions: stakePositions && !stakePositions.error ? stakePositions : null,
      rewards: rewardsData && !rewardsData.error ? {
        total: rewardsData.total ?? rewardsData.total_rewards ?? 0,
        pending: rewardsData.pending ?? rewardsData.pending_rewards ?? 0,
        claimed: rewardsData.claimed ?? rewardsData.claimed_rewards ?? 0,
        per_epoch: rewardsData.rewards_per_epoch ?? 0,
      } : null,
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
      response.skip_rate = validatorInfo.skip_rate ?? null;
      response.last_vote = validatorInfo.last_vote ?? validatorInfo.last_vote_slot ?? null;
      response.identity = validatorInfo.identity ?? validatorInfo.node_id ?? null;
      response.name = validatorInfo.name ?? validatorInfo.moniker ?? null;
      response.website = validatorInfo.website ?? null;
      response.details = validatorInfo.details ?? validatorInfo.description ?? null;
      
      // Use commission from validator info if not in APY data
      if (response.commission === null) {
        response.commission = validatorInfo.commission ?? validatorInfo.commission_bps ?? null;
      }
    }

    // JSON output
    if (opts.json) {
      console.log(JSON.stringify(response, (key, value) => {
        // Convert BigInt to string for JSON serialization
        if (typeof value === 'bigint') return value.toString();
        return value;
      }, 2));
      return;
    }

    // Pretty output
    printValidatorCard(validatorInfo, apyData, networkInfo);

    // Show delegations if requested or if showing all
    if (opts.showDelegations && stakePositions && !stakePositions.error) {
      printStakePositions(stakePositions, opts.limit);
    }

    // Show rewards if requested
    if (opts.showRewards && rewardsData && !rewardsData.error) {
      printRewards(rewardsData);
    }

    // Quick actions
    console.log(`  ${C.dim}── Quick Actions ─────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}• Stake with this validator:${C.reset}`);
    console.log(`      ${C.cyan}aether stake --validator ${shortPubkey(address, 8)} --amount <AETH>${C.reset}`);
    console.log(`  ${C.dim}• View all validators:${C.reset}`);
    console.log(`      ${C.cyan}aether validators list${C.reset}`);
    console.log(`  ${C.dim}• Check your delegations:${C.reset}`);
    console.log(`      ${C.cyan}aether stake-positions --address <your_wallet>${C.reset}`);
    console.log();

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
      console.log(`  ${C.dim}  Troubleshooting:${C.reset}`);
      console.log(`    • Is your validator running? Check with: ${C.cyan}aether ping${C.reset}`);
      console.log(`    • Verify RPC endpoint: ${C.dim}${rpcUrl}${C.reset}`);
      console.log(`    • Set custom RPC: ${C.dim}AETHER_RPC=https://your-rpc-url${C.reset}`);
      console.log(`    • Check network status: ${C.cyan}aether network${C.reset}\n`);
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
