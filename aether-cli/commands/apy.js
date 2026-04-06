#!/usr/bin/env node
/**
 * aether-cli apy
 *
 * Estimate APY for a validator or wallet's stake positions.
 * FULLY WIRED TO SDK - Uses @jellylegsai/aether-sdk for all blockchain calls.
 * No manual HTTP - all calls go through AetherClient with real RPC.
 *
 * Usage:
 *   aether apy                           Show network-wide average APY
 *   aether apy --validator <addr>        APY for a specific validator
 *   aether apy --address <addr>          APY for a wallet's stake delegations
 *   aether apy --json                    JSON output for scripting/monitoring
 *   aether apy --rpc <url>               Override default RPC
 *
 * Examples:
 *   aether apy --validator ATH3xyz...   # Check validator APY
 *   aether apy --address ATHabc...      # Check your wallet's weighted APY
 *   aether apy --json                   # Machine-readable output
 *
 * SDK Methods Used:
 *   - client.getValidatorAPY(address)  → GET /v1/validator/<address>/apy
 *   - client.getStakePositions(address) → GET /v1/stake/<address>
 *   - client.getRewards(address)       → GET /v1/rewards/<address>
 *   - client.getEpochInfo()            → GET /v1/epoch
 *   - client.getSupply()               → GET /v1/supply
 */

const path = require('path');

// Import SDK — makes REAL HTTP RPC calls to the blockchain
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

const CLI_VERSION = '1.1.0';
const EPOCHS_PER_YEAR = 2190; // Aether epochs are ~4 hours

// ---------------------------------------------------------------------------
// SDK Client Setup
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

function getDefaultConfig() {
  const fs = require('fs');
  const cfgPath = path.join(require('os').homedir(), '.aether', 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    rpc: getDefaultRpc(),
    validator: null,
    address: null,
    asJson: false,
    epochs: 14,      // default lookback for APY calc
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--validator' || arg === '-v') {
      opts.validator = args[++i];
    } else if (arg === '--address' || arg === '-a') {
      opts.address = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.asJson = true;
    } else if (arg === '--epochs' || arg === '-e') {
      const v = parseInt(args[++i], 10);
      if (!isNaN(v) && v > 0 && v <= 100) opts.epochs = v;
    } else if (arg === '--help' || arg === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  // Fall back to default wallet if no address provided
  if (!opts.address && !opts.validator) {
    const cfg = getDefaultConfig();
    opts.address = cfg.defaultWallet;
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli apy${C.reset} - Validator APY Estimator

${C.bright}Usage:${C.reset}
  aether apy                              Network-wide average APY
  aether apy --validator <addr>           APY for a specific validator
  aether apy --address <addr>             APY for wallet's stake positions
  aether apy --rpc <url>                  Override default RPC
  aether apy --json                       JSON output for scripting
  aether apy --epochs <n>                 Lookback epochs (default: 14, max: 100)

${C.bright}SDK Methods Used:${C.reset}
  client.getValidatorAPY(address)   → GET /v1/validator/<address>/apy
  client.getStakePositions(address)   → GET /v1/stake/<address>
  client.getRewards(address)          → GET /v1/rewards/<address>
  client.getEpochInfo()               → GET /v1/epoch
  client.getSupply()                  → GET /v1/supply

${C.bright}Examples:${C.reset}
  aether apy --validator ATH3J8...       Check a validator's APY
  aether apy --address ATHabc...           Check your wallet's weighted APY
  aether apy --json                      Machine-readable output
`.trim());
}

// ---------------------------------------------------------------------------
// APY calculation via SDK (REAL RPC CALLS)
// ---------------------------------------------------------------------------

/**
 * Fetch validator APY via SDK (GET /v1/validator/<address>/apy)
 */
async function fetchValidatorApy(client, validatorAddr) {
  try {
    const result = await client.getValidatorAPY(validatorAddr);
    return {
      apy: result.apy ?? result.current_apy ?? result.estimated_apy ?? null,
      commission: result.commission ?? null,
      total_stake: result.total_stake ?? result.stake_lamports ?? null,
      source: 'validator_api',
      raw: result,
    };
  } catch (err) {
    return { error: err.message, source: 'validator_api' };
  }
}

/**
 * Calculate wallet APY from stake positions and rewards via SDK
 */
async function calculateWalletApy(client, address, epochs) {
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  // Parallel SDK calls
  const [stakePositions, rewards, epochInfo, supply] = await Promise.all([
    client.getStakePositions(rawAddr).catch(() => []),
    client.getRewards(rawAddr).catch(() => ({ total: 0, pending: 0, history: [] })),
    client.getEpochInfo().catch(() => ({ epoch: 0, slotsInEpoch: 432000, slotIndex: 0 })),
    client.getSupply().catch(() => null),
  ]);

  const currentEpoch = epochInfo?.epoch ?? 0;

  // No stake positions
  if (!Array.isArray(stakePositions) || stakePositions.length === 0) {
    return {
      apy: null,
      apy_pct: null,
      method: 'no_stake',
      epoch: currentEpoch,
      address,
      note: 'No active stake positions found for this address.',
    };
  }

  // Calculate total staked
  const totalStakedLamports = stakePositions.reduce((sum, s) => {
    return sum + BigInt(s.lamports || s.stake_lamports || 0);
  }, 0n);

  if (totalStakedLamports === 0n) {
    return {
      apy: null,
      apy_pct: null,
      method: 'no_active_stake',
      epoch: currentEpoch,
      address,
      note: 'Stake positions found but no active stake amount.',
    };
  }

  // Get reward history
  const rewardHistory = rewards?.history || rewards?.epochs || [];
  const totalRewardsLamports = BigInt(rewards?.total || rewards?.total_rewards || 0);

  // If we have reward history, calculate from it
  if (rewardHistory.length > 0 && totalRewardsLamports > 0n) {
    const epochsWithData = Math.min(rewardHistory.length, epochs);
    const avgYieldPerEpoch = Number(totalRewardsLamports) / (Number(totalStakedLamports) * epochsWithData);
    const apy = Math.pow(1 + avgYieldPerEpoch, EPOCHS_PER_YEAR) - 1;

    return {
      apy,
      apy_pct: parseFloat((apy * 100).toFixed(2)),
      method: 'reward_history',
      epoch: currentEpoch,
      epochs_used: epochsWithData,
      epochs_with_rewards: rewardHistory.filter(r => (r.rewards || 0) > 0).length,
      total_rewards_lamports: totalRewardsLamports.toString(),
      total_staked_lamports: totalStakedLamports.toString(),
      avg_yield_per_epoch_pct: parseFloat((avgYieldPerEpoch * 100).toFixed(4)),
      address,
      stake_count: stakePositions.length,
    };
  }

  // No reward history yet — use network inflation model
  if (supply && !supply.error) {
    // Aether uses ~7% inflation Year 1, declining
    const inflationRate = 0.07;
    return {
      apy: inflationRate,
      apy_pct: 7.0,
      method: 'inflation_model',
      epoch: currentEpoch,
      epochs_used: 0,
      total_staked_lamports: totalStakedLamports.toString(),
      address,
      stake_count: stakePositions.length,
      note: 'No reward history available yet. APY estimated from network inflation model.',
    };
  }

  // No data available
  return {
    apy: null,
    apy_pct: null,
    method: 'insufficient_data',
    epoch: currentEpoch,
    address,
    stake_count: stakePositions.length,
    note: 'Stake positions exist but no reward data yet. Check back after epoch ends.',
  };
}

/**
 * Estimate APY using SDK
 */
async function estimateApy({ rpc, validator, address, epochs }) {
  const client = createClient(rpc);

  // Validator-specific APY
  if (validator) {
    const rawAddr = validator.startsWith('ATH') ? validator.slice(3) : validator;
    const validatorApy = await fetchValidatorApy(client, rawAddr);

    if (validatorApy.error) {
      // Fall back to wallet calculation method
      return await calculateWalletApy(client, validator, epochs);
    }

    const apy = validatorApy.apy ?? 0;
    return {
      apy,
      apy_pct: apy !== null ? parseFloat((apy * 100).toFixed(2)) : null,
      method: 'validator_api',
      commission: validatorApy.commission,
      total_stake: validatorApy.total_stake,
      validator,
      source: validatorApy.source,
    };
  }

  // Wallet APY (weighted across all stake positions)
  if (address) {
    return await calculateWalletApy(client, address, epochs);
  }

  // Network-wide APY
  try {
    const [epochInfo, supply] = await Promise.all([
      client.getEpochInfo().catch(() => null),
      client.getSupply().catch(() => null),
    ]);

    if (supply && !supply.error) {
      const inflationRate = 0.07; // 7% base inflation
      return {
        apy: inflationRate,
        apy_pct: 7.0,
        method: 'network_inflation_model',
        epoch: epochInfo?.epoch ?? null,
        note: 'Network-wide APY estimate based on inflation schedule.',
      };
    }
  } catch (err) {
    // Fall through
  }

  return {
    apy: null,
    apy_pct: null,
    method: 'none',
    error: 'Unable to fetch network data. Is your validator running?',
  };
}

// ---------------------------------------------------------------------------
// Visual formatters
// ---------------------------------------------------------------------------

function formatApyBar(apyPct, maxPct = 20) {
  const totalBars = 20;
  const filled = Math.min(totalBars, Math.round((apyPct / maxPct) * totalBars));
  return C.green + '█'.repeat(filled) + C.dim + '░'.repeat(totalBars - filled) + C.reset;
}

function getApyColor(apyPct) {
  if (apyPct === null || apyPct === undefined) return C.dim;
  if (apyPct >= 8) return C.green;
  if (apyPct >= 4) return C.cyan;
  if (apyPct >= 2) return C.yellow;
  return C.red;
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function outputHuman(apyData, opts) {
  const { rpc, validator, address } = opts;

  console.log(`\n${C.bright}${C.cyan}── Validator APY Estimate ──────────────────────────────${C.reset}\n`);

  const targetLabel = validator
    ? `Validator: ${C.bright}${validator}${C.reset}`
    : address
    ? `Address: ${C.bright}${address}${C.reset}`
    : `Network: ${C.bright}Aether Chain${C.reset}`;
  console.log(`  ${C.green}★${C.reset} ${targetLabel}`);
  console.log(`  ${C.dim}  RPC: ${rpc}${C.reset}`);
  console.log();

  if (apyData.error) {
    console.log(`  ${C.yellow}⚠ ${apyData.error}${C.reset}\n`);
    return;
  }

  if (apyData.note) {
    console.log(`  ${C.yellow}⚠ ${apyData.note}${C.reset}`);
    console.log();
  }

  if (apyData.apy_pct === null) {
    console.log(`  ${C.yellow}⚠ APY data unavailable.${C.reset}`);
    if (apyData.method === 'inflation_model' || apyData.method === 'network_inflation_model') {
      console.log(`  ${C.dim}  Using network inflation model as estimate.${C.reset}`);
    }
    console.log();
    return;
  }

  // Main APY display
  const apyColor = getApyColor(apyData.apy_pct);
  console.log(`  ${C.bright}${apyColor}${apyData.apy_pct.toFixed(2)}%${C.reset} ${C.dim}APY${C.reset}`);
  console.log();

  // Visual bar
  console.log(`  ${C.dim}Yield:${C.reset} ${formatApyBar(apyData.apy_pct)}`);
  console.log();

  // Stats
  const methodLabel = apyData.method === 'reward_history'
    ? 'Reward history annualised'
    : apyData.method === 'validator_api'
    ? 'Validator API'
    : 'Inflation model';
  console.log(`  ${C.dim}Method:${C.reset}    ${C.bright}${methodLabel}${C.reset}`);

  if (apyData.epoch !== undefined && apyData.epoch !== null) {
    console.log(`  ${C.dim}Epoch:${C.reset}     ${C.bright}#${apyData.epoch}${C.reset}`);
  }

  if (apyData.epochs_used !== undefined) {
    console.log(`  ${C.dim}Epochs used:${C.reset} ${apyData.epochs_used}`);
  }

  if (apyData.avg_yield_per_epoch_pct !== undefined) {
    console.log(`  ${C.dim}Avg/epoch:${C.reset}  ${C.bright}${apyData.avg_yield_per_epoch_pct.toFixed(4)}%${C.reset}`);
  }

  if (apyData.total_rewards_lamports !== undefined && apyData.total_staked_lamports !== undefined) {
    const totalAeth = (Number(apyData.total_rewards_lamports) / 1e9).toFixed(4);
    const stakedAeth = (Number(apyData.total_staked_lamports) / 1e9).toFixed(2);
    console.log(`  ${C.dim}Total rewards:${C.reset} ${C.green}${totalAeth} AETH${C.reset}`);
    console.log(`  ${C.dim}Total staked:${C.reset}  ${stakedAeth} AETH`);
  }

  if (apyData.commission !== undefined && apyData.commission !== null) {
    console.log(`  ${C.dim}Commission:${C.reset}  ${apyData.commission}%`);
  }

  if (apyData.stake_count !== undefined) {
    console.log(`  ${C.dim}Stake positions:${C.reset} ${apyData.stake_count}`);
  }

  console.log();

  // Disclaimer
  console.log(`  ${C.dim}Note: APY is an estimate based on ${apyData.epochs_used || 0} epoch(s) of reward data.${C.reset}`);
  console.log(`  ${C.dim}Actual returns may vary. Check 'aether rewards' for precise figures.${C.reset}`);
  console.log();
}

function outputJson(apyData, opts) {
  console.log(JSON.stringify({
    apy_pct: apyData.apy_pct,
    apy: apyData.apy,
    method: apyData.method,
    epoch: apyData.epoch,
    epochs_used: apyData.epochs_used,
    epochs_with_rewards: apyData.epochs_with_rewards,
    avg_yield_per_epoch_pct: apyData.avg_yield_per_epoch_pct,
    total_rewards_lamports: apyData.total_rewards_lamports,
    total_staked_lamports: apyData.total_staked_lamports,
    commission: apyData.commission,
    validator: apyData.validator,
    address: apyData.address,
    stake_count: apyData.stake_count,
    note: apyData.note || null,
    error: apyData.error || null,
    rpc: opts.rpc,
    cli_version: CLI_VERSION,
    sdk_version: 'SDK via AetherClient',
    timestamp: new Date().toISOString(),
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function apyCommand() {
  const opts = parseArgs();

  try {
    const apyData = await estimateApy(opts);

    if (opts.asJson) {
      outputJson(apyData, opts);
    } else {
      outputHuman(apyData, opts);
    }
  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({
        apy_pct: null,
        apy: null,
        error: err.message,
        validator: opts.validator,
        address: opts.address,
        rpc: opts.rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ APY calculation failed:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  RPC: ${opts.rpc}${C.reset}`);
      console.log(`  ${C.dim}  Is your validator running?${C.reset}\n`);
    }
    process.exit(1);
  }
}

// Export for use in index.js
module.exports = { apyCommand };

// Run if called directly
if (require.main === module) {
  apyCommand().catch(err => {
    console.error(`\n${C.red}APY command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
