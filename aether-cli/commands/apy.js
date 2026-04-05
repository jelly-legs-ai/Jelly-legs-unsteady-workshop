#!/usr/bin/env node
/**
 * aether-cli apy
 *
 * Estimate APY for a validator or wallet's stake positions.
 * Fetches recent reward history, computes average yield over the current
 * epoch, and annualises it to an APY figure.
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
 */

const http = require('http');
const https = require('https');

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

const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

function getDefaultConfig() {
  const fs = require('fs');
  const path = require('path');
  const os = require('os');
  const cfgPath = path.join(os.homedir(), '.aether', 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, pathStr, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: timeoutMs,
      headers: { 'Content-Type': 'application/json' },
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

function httpPost(rpcUrl, pathStr, body, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: timeoutMs,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
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
    req.write(bodyStr);
    req.end();
  });
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

${C.bright}Examples:${C.reset}
  aether apy --validator ATH3J8...       Check a validator's APY
  aether apy --address ATHabc...          Check your wallet's weighted APY
  aether apy --json                       Machine-readable output
`.trim());
}

// ---------------------------------------------------------------------------
// APY calculation
// ---------------------------------------------------------------------------

/**
 * Estimate APY from reward history.
 *
 * Strategy:
 *   1. Fetch last N epochs of reward history for the target
 *   2. Compute per-epoch yield: rewards_staked
 *   3. Average the yield, then annualise: APY = ((1 + avg_yield) ^ epochs_per_year) - 1
 *
 * Aether epochs are ~4 hours → ~2190 epochs/year
 */
async function estimateApy({ rpc, validator, address, epochs }) {
  const EPOCHS_PER_YEAR = 2190;

  // Normalise address: strip ATH prefix for API calls
  const rawAddr = (validator || address || '').startsWith('ATH')
    ? (validator || address || '').slice(3)
    : (validator || address || '');

  // Step 1: fetch epoch info to get current epoch
  let currentEpoch = null;
  let epochLength = null;
  try {
    const epochInfo = await httpRequest(rpc, '/v1/epoch/info');
    if (!epochInfo.error) {
      currentEpoch = epochInfo.epoch;
      epochLength = epochInfo.slots_in_epoch || epochInfo.epoch_length;
    }
  } catch { /* use defaults */ }

  // Step 2: fetch reward history
  // Try /v1/rewards?address=<addr>&epochs=<N> first
  let rewardHistory = [];
  try {
    const fromEpoch = Math.max(0, (currentEpoch || epochs) - epochs);
    const rewardsRes = await httpRequest(
      rpc,
      `/v1/rewards?address=${encodeURIComponent(rawAddr)}&from_epoch=${fromEpoch}&limit=${epochs}`
    );
    if (!rewardsRes.error) {
      rewardHistory = Array.isArray(rewardsRes)
        ? rewardsRes
        : (rewardsRes.rewards || []);
    }
  } catch { /* try alternate endpoint */ }

  // Fallback: /v1/stake?address=<addr> (contains accumulated rewards)
  if (rewardHistory.length === 0) {
    try {
      const stakeRes = await httpRequest(
        rpc,
        `/v1/stake?address=${encodeURIComponent(rawAddr)}`
      );
      if (!stakeRes.error) {
        const accounts = Array.isArray(stakeRes) ? stakeRes : (stakeRes.accounts || []);
        for (const acc of accounts) {
          if (acc.rewards !== undefined) {
            rewardHistory.push({
              epoch: acc.epoch || currentEpoch,
              rewards: acc.rewards,
              lamports: acc.stake_lamports || acc.lamports || 0,
            });
          }
        }
      }
    } catch { /* no stake data */ }
  }

  // Step 3: calculate APY
  if (rewardHistory.length === 0) {
    // No reward data — try fetching network-wide stats for a ballpark
    try {
      const supplyRes = await httpRequest(rpc, '/v1/supply');
      const epochRes = await httpRequest(rpc, '/v1/epoch/info');
      if (!supplyRes.error && !epochRes.error) {
        // Rough estimate: total_reward_rate from inflation schedule
        // Aether uses ~7% inflation Year 1, declining. Use 7% as starting point.
        const inflationRate = 0.07; // 7% base, would need real data for precision
        return {
          apy: inflationRate,
          apy_pct: inflationRate * 100,
          method: 'inflation_model',
          epoch: currentEpoch,
          epochs_used: 0,
          epochs_available: 0,
          total_staked: supplyRes.total || 0,
          validator,
          address,
          note: 'No reward history available. APY estimated from network inflation model.',
        };
      }
    } catch { /* no network data either */ }

    return {
      apy: null,
      apy_pct: null,
      method: 'none',
      epoch: currentEpoch,
      epochs_used: 0,
      error: 'No reward history or network data available. Ensure your validator is running and has reward data.',
    };
  }

  // Compute weighted average yield per epoch
  let totalRewards = 0;
  let totalStaked = 0;
  let epochsWithRewards = 0;

  for (const entry of rewardHistory) {
    const rewards = entry.rewards || 0;
    const staked = entry.lamports || entry.stake_lamports || 0;
    totalRewards += rewards;
    if (staked > 0) totalStaked += staked;
    if (rewards > 0) epochsWithRewards++;
  }

  if (totalStaked === 0 || totalRewards === 0) {
    return {
      apy: null,
      apy_pct: null,
      method: 'insufficient_data',
      epoch: currentEpoch,
      epochs_used: rewardHistory.length,
      epochs_with_rewards: epochsWithRewards,
      validator,
      address,
      note: 'Stake positions exist but no reward data yet. Check back after epoch ends.',
    };
  }

  // Average epoch yield = total rewards / (total staked * num epochs)
  const avgYieldPerEpoch = totalRewards / (totalStaked * rewardHistory.length);

  // Annualise: APY = (1 + yield_per_epoch) ^ epochs_per_year - 1
  const apy = Math.pow(1 + avgYieldPerEpoch, EPOCHS_PER_YEAR) - 1;
  const apyPct = apy * 100;

  return {
    apy,
    apy_pct: parseFloat(apyPct.toFixed(2)),
    method: 'reward_history',
    epoch: currentEpoch,
    epochs_used: rewardHistory.length,
    epochs_with_rewards: epochsWithRewards,
    total_rewards_lamports: totalRewards,
    total_staked_lamports: totalStaked,
    avg_yield_per_epoch_pct: parseFloat((avgYieldPerEpoch * 100).toFixed(4)),
    validator,
    address,
  };
}

/**
 * Format APY bar: ████░░░░░░
 */
function formatApyBar(apyPct, maxPct = 20) {
  const totalBars = 20;
  const filled = Math.min(totalBars, Math.round((apyPct / maxPct) * totalBars));
  return C.green + '█'.repeat(filled) + C.dim + '░'.repeat(totalBars - filled) + C.reset;
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function outputHuman(apyData, opts) {
  const { rpc, validator, address, asJson } = opts;

  console.log(`\n${C.bright}${C.cyan}── Validator APY Estimate ──────────────────────────────${C.reset}\n`);

  const targetLabel = validator
    ? `Validator: ${C.bright}${validator}${C.reset}`
    : `Address: ${C.bright}${address}${C.reset}`;
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
    if (apyData.method === 'inflation_model') {
      console.log(`  ${C.dim}  Using network inflation model as estimate.${C.reset}`);
    }
    console.log();
    return;
  }

  // Main APY display
  const apyColor = apyData.apy_pct >= 8
    ? C.green
    : apyData.apy_pct >= 4
    ? C.cyan
    : apyData.apy_pct >= 2
    ? C.yellow
    : C.red;

  console.log(`  ${C.bright}${apyColor}${apyData.apy_pct.toFixed(2)}%${C.reset} ${C.dim}APY${C.reset}`);
  console.log();

  // Visual bar
  console.log(`  ${C.dim}Yield:${C.reset} ${formatApyBar(apyData.apy_pct)}`);
  console.log();

  // Stats
  console.log(`  ${C.dim}Method:${C.reset}    ${C.bright}${apyData.method === 'reward_history' ? 'Reward history annualised' : 'Inflation model'}${C.reset}`);
  if (apyData.epoch !== undefined && apyData.epoch !== null) {
    console.log(`  ${C.dim}Epoch:${C.reset}     ${C.bright}#${apyData.epoch}${C.reset}`);
  }
  console.log(`  ${C.dim}Epochs used:${C.reset} ${apyData.epochs_used}`);

  if (apyData.avg_yield_per_epoch_pct !== undefined) {
    console.log(`  ${C.dim}Avg/epoch:${C.reset}  ${C.bright}${apyData.avg_yield_per_epoch_pct.toFixed(4)}%${C.reset}`);
  }

  if (apyData.total_rewards_lamports !== undefined && apyData.total_staked_lamports !== undefined) {
    const totalAeth = (apyData.total_rewards_lamports / 1e9).toFixed(4);
    const stakedAeth = (apyData.total_staked_lamports / 1e9).toFixed(2);
    console.log(`  ${C.dim}Total rewards:${C.reset} ${C.green}${totalAeth} AETH${C.reset}`);
    console.log(`  ${C.dim}Total staked:${C.reset}  ${stakedAeth} AETH`);
  }

  console.log();

  // Disclaimer
  console.log(`  ${C.dim}Note: APY is an estimate based on ${apyData.epochs_used} epoch(s) of reward data.${C.reset}`);
  console.log(`  ${C.dim}Actual returns may vary. Check aether rewards for precise figures.${C.reset}`);
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
    validator: apyData.validator,
    address: apyData.address,
    note: apyData.note || null,
    error: apyData.error || null,
    rpc: opts.rpc,
    cli_version: CLI_VERSION,
    timestamp: new Date().toISOString(),
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
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

main();

module.exports = { apyCommand: main };
