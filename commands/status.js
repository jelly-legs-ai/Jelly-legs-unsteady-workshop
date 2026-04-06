#!/usr/bin/env node
/**
 * aether-cli status
 *
 * Single-command dashboard: epoch + network + supply + validator + rewards
 * Gives a full node/network overview in one shot — no need to run multiple commands.
 *
 * Usage:
 *   aether status                 Show full status dashboard
 *   aether status --json          JSON output for scripting/monitoring
 *   aether status --rpc <url>     Query a specific RPC endpoint
 *   aether status --validator    Include local validator info
 *   aether status --compact      One-line summary
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

const os = require('os');
const path = require('path');

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

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function formatAether(lamports) {
  const aeth = (Number(lamports) / 1e9).toFixed(4);
  return aeth + ' AETH';
}

function loadConfig() {
  const fs = require('fs');
  const path = require('path');
  const aetherDir = path.join(os.homedir(), '.aether');
  const cfgPath = path.join(aetherDir, 'config.json');
  if (!fs.existsSync(cfgPath)) return { defaultWallet: null };
  try { return JSON.parse(fs.readFileSync(cfgPath, 'utf8')); }
  catch { return { defaultWallet: null }; }
}

function loadIdentity() {
  const fs = require('fs');
  const path = require('path');
  const idPath = path.join(os.homedir(), '.aether', 'validator-identity.json');
  if (!fs.existsSync(idPath)) return null;
  try { return JSON.parse(fs.readFileSync(idPath, 'utf8')); }
  catch { return null; }
}

// ---------------------------------------------------------------------------
// Status command
// ---------------------------------------------------------------------------

async function statusCommand() {
  const args = process.argv.slice(2); // [node, status.js, ...]
  const isJson = args.includes('--json') || args.includes('-j');
  const isCompact = args.includes('--compact');
  const includeValidator = args.includes('--validator');
  const rpcIdx = args.findIndex(a => a === '--rpc');
  const rpc = rpcIdx !== -1 && args[rpcIdx + 1] ? args[rpcIdx + 1] : getDefaultRpc();

  const errors = {};
  const data = {};

  // Fetch all data in parallel
  const promises = [
    fetchEpochInfo(rpc).then(d => { data.epoch = d; }).catch(e => { errors.epoch = e.message; }),
    fetchSupplyInfo(rpc).then(d => { data.supply = d; }).catch(e => { errors.supply = e.message; }),
    fetchNetworkInfo(rpc).then(d => { data.network = d; }).catch(e => { errors.network = e.message; }),
    fetchVersionInfo(rpc).then(d => { data.version = d; }).catch(e => { errors.version = e.message; }),
  ];

  await Promise.all(promises);

  // Validator identity (local file)
  data.validator = loadIdentity();

  // Rewards for default wallet (optional)
  const config = loadConfig();
  if (config.defaultWallet) {
    data.defaultWallet = config.defaultWallet;
    try {
      data.rewards = await fetchRewardsSummary(rpc, config.defaultWallet);
    } catch (e) {
      errors.rewards = e.message;
    }
  }

  if (isJson) {
    console.log(JSON.stringify({ rpc, errors: Object.keys(errors).length ? errors : undefined, ...data }, null, 2));
    return;
  }

  if (isCompact) {
    printCompact(data, errors);
    return;
  }

  printDashboard(data, errors, includeValidator, rpc);
}

async function fetchEpochInfo(rpc) {
  const client = createClient(rpc);
  try {
    const epoch = await client.getEpochInfo();
    const currentSlot = epoch.absoluteSlot || epoch.slot || 0;
    const slotsInEpoch = epoch.slotsInEpoch || 432000;
    const slotIndex = epoch.slotIndex || (currentSlot % slotsInEpoch);
    const epochProgress = slotsInEpoch > 0 ? (slotIndex / slotsInEpoch * 100).toFixed(1) : '0';

    // Estimate time remaining (assuming 400ms slots)
    const slotsRemaining = slotsInEpoch - slotIndex;
    const secsRemaining = Math.round(slotsRemaining * 0.4);
    const minsRemaining = Math.round(secsRemaining / 60);
    const timeStr = minsRemaining >= 60
      ? `${Math.floor(minsRemaining / 60)}h ${minsRemaining % 60}m`
      : `${minsRemaining}m`;

    return {
      epoch: epoch.epoch || 0,
      absoluteSlot: currentSlot,
      slotIndex,
      slotsInEpoch,
      progress: epochProgress,
      timeRemaining: timeStr,
      totalSlots: epoch.totalSlots || 0,
    };
  } catch (e) {
    throw new Error('Epoch info fetch failed: ' + e.message);
  }
}

async function fetchSupplyInfo(rpc) {
  const client = createClient(rpc);
  try {
    const supply = await client.getSupply();
    const total = BigInt(supply.total || 0);
    const circulating = BigInt(supply.circulating || 0);
    const nonCirculating = BigInt(supply.nonCirculating || 0);

    return {
      total: total.toString(),
      totalFormatted: formatAether(total.toString()),
      circulating: circulating.toString(),
      circulatingFormatted: formatAether(circulating.toString()),
      nonCirculating: nonCirculating.toString(),
      nonCirculatingFormatted: formatAether(nonCirculating.toString()),
    };
  } catch (e) {
    throw new Error('Supply info fetch failed: ' + e.message);
  }
}

async function fetchNetworkInfo(rpc) {
  const client = createClient(rpc);
  try {
    const [slot, blockHeight, peers] = await Promise.all([
      client.getSlot(),
      client.getBlockHeight(),
      client.getClusterPeers(),
    ]);

    return {
      blockHeight: blockHeight || slot || 0,
      blockTime: null,
      peerCount: Array.isArray(peers) ? peers.length : 0,
      peers: (Array.isArray(peers) ? peers : []).slice(0, 5),
    };
  } catch (e) {
    throw new Error('Network info fetch failed: ' + e.message);
  }
}

async function fetchVersionInfo(rpc) {
  const client = createClient(rpc);
  try {
    return await client.getVersion();
  } catch {
    return {};
  }
}

async function fetchRewardsSummary(rpc, address) {
  const client = createClient(rpc);
  try {
    // Get stake positions for the address (real RPC call via SDK)
    const stakePositions = await client.getStakePositions(address);
    
    if (!stakePositions || stakePositions.length === 0) return null;

    const rewardsResults = await Promise.all(
      stakePositions.slice(0, 10).map(async (stake) => {
        try {
          const stakeAccount = stake.stakeAccount || stake.account || stake.pubkey || stake;
          const rewards = await client.getRewards(stakeAccount);
          const total = BigInt(rewards.total || rewards.amount || 0);
          return { 
            stakeAccount, 
            estimatedRewards: total.toString(), 
            estimatedRewardsFormatted: formatAether(total.toString()) 
          };
        } catch {
          return { stakeAccount: stake.stakeAccount || stake, estimatedRewards: '0', estimatedRewardsFormatted: '0 AETH' };
        }
      })
    );

    let totalRewards = BigInt(0);
    for (const r of rewardsResults) {
      totalRewards += BigInt(r.estimatedRewards);
    }

    return {
      address,
      totalRewards: totalRewards.toString(),
      totalRewardsFormatted: formatAether(totalRewards.toString()),
      activeAccounts: rewardsResults.filter(r => BigInt(r.estimatedRewards) > 0n).length,
      totalAccounts: rewardsResults.length,
    };
  } catch (e) {
    throw new Error('Rewards fetch failed: ' + e.message);
  }
}

function printDashboard(data, errors, includeValidator, rpc) {
  const { epoch, supply, network, version, validator, rewards, defaultWallet } = data;

  console.log(`\n${C.bright}${C.cyan}  ╔══════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}  ║              AETHER STATUS DASHBOARD                   ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}  ╚══════════════════════════════════════════════════════════╝${C.reset}\n`);

  // Epoch row
  if (epoch) {
    console.log(`  ${C.bright}Epoch${C.reset}  ${C.cyan}E${epoch.epoch}${C.reset}  │  Slot ${C.bright}${epoch.absoluteSlot.toLocaleString()}${C.reset} (${epoch.progress}%)  │  ${epoch.timeRemaining} remaining`);
  } else {
    console.log(`  ${C.red}✗ Epoch info unavailable${errors.epoch ? ': ' + errors.epoch : ''}${C.reset}`);
  }

  // Network row
  if (network) {
    const peerStr = network.peerCount > 0 ? `${C.green}${network.peerCount} peers${C.reset}` : `${C.yellow}no peers${C.reset}`;
    console.log(`  ${C.bright}Network${C.reset} │  Block ${C.bright}${network.blockHeight.toLocaleString()}${C.reset}  │  ${peerStr}`);
  } else {
    console.log(`  ${C.red}✗ Network info unavailable${errors.network ? ': ' + errors.network : ''}${C.reset}`);
  }

  // Supply row
  if (supply) {
    console.log(`  ${C.bright}Supply${C.reset}  │  Total ${C.cyan}${supply.totalFormatted}${C.reset}  │  Circulating ${C.green}${supply.circulatingFormatted}${C.reset}`);
    console.log(`         │  Staked (non-circulating) ${C.yellow}${supply.nonCirculatingFormatted}${C.reset}`);
  } else {
    console.log(`  ${C.red}✗ Supply info unavailable${errors.supply ? ': ' + errors.supply : ''}${C.reset}`);
  }

  // Version row
  if (version && Object.keys(version).length > 0) {
    console.log(`  ${C.bright}Version${C.reset} │  ${C.dim}${JSON.stringify(version)}${C.reset}`);
  }

  // Validator row
  if (includeValidator && validator) {
    const identity = validator.identity || validator.nodeKey || 'unknown';
    const shortId = identity.length > 16 ? identity.substring(0, 16) + '...' : identity;
    const stake = validator.delegatedStake ? formatAether(validator.delegatedStake) : 'unknown';
    console.log(`  ${C.bright}Validator${C.reset} │  ${C.magenta}${shortId}${C.reset}  │  Stake: ${stake}`);
  } else if (includeValidator && !validator) {
    console.log(`  ${C.bright}Validator${C.reset} │  ${C.yellow}No validator identity found (run aether init)${C.reset}`);
  }

  // Rewards row
  if (rewards && defaultWallet) {
    const shortAddr = defaultWallet.length > 16 ? defaultWallet.substring(0, 16) + '...' : defaultWallet;
    console.log(`  ${C.bright}Rewards${C.reset}  │  ${C.green}${rewards.totalRewardsFormatted}${C.reset} est.  │  Wallet: ${C.dim}${shortAddr}${C.reset}`);
  }

  console.log(`  ${C.dim}RPC: ${rpc || getDefaultRpc()}${C.reset}\n`);
}

function printCompact(data, errors) {
  const parts = [];
  if (data.epoch) parts.push(`E${data.epoch.epoch}`);
  if (data.network) parts.push(`blk ${data.network.blockHeight}`);
  if (data.network) parts.push(`p${data.network.peerCount}`);
  if (data.supply) parts.push(`total ${data.supply.totalFormatted}`);
  if (data.rewards) parts.push(`rwd ${data.rewards.totalRewardsFormatted}`);
  if (Object.keys(errors).length > 0) parts.push(`err:${Object.keys(errors).join(',')}`);
  console.log(parts.join(' │ '));
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

module.exports = { statusCommand };

// Run directly
if (require.main === module) {
  statusCommand().catch(err => {
    console.error(`${C.red}✗ Status command failed:${C.reset} ${err.message}`);
    process.exit(1);
  });
}