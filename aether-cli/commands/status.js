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

const http = require('http');
const https = require('https');
const os = require('os');

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
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

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
        catch { resolve({ raw: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout after ' + timeoutMs + 'ms')); });
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
        catch { resolve(data); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout after ' + timeoutMs + 'ms')); });
    req.end();
  });
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
  const args = process.argv.slice(3); // skip "aether status"
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

  printDashboard(data, errors, includeValidator);
}

async function fetchEpochInfo(rpc) {
  const [epochResp, slotResp] = await Promise.all([
    httpPost(rpc, '/v1/epoch/info', { jsonrpc: '2.0', id: 1, method: 'getEpochInfo' }),
    httpPost(rpc, '/v1Slot', { jsonrpc: '2.0', id: 1, method: 'getSlot' }),
  ]);

  const epoch = epochResp?.result || {};
  const currentSlot = slotResp?.result || epoch.currentSlot || 0;
  const slotsInEpoch = epoch.slotsInEpoch || 432000;
  const slotIndex = currentSlot % slotsInEpoch;
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
}

async function fetchSupplyInfo(rpc) {
  const resp = await httpPost(rpc, '/v1/supply', { jsonrpc: '2.0', id: 1, method: 'getSupply' });
  const supply = resp?.result?.value || {};
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
}

async function fetchNetworkInfo(rpc) {
  const [slotResp, blockResp, peersResp] = await Promise.all([
    httpPost(rpc, '/v1Slot', { jsonrpc: '2.0', id: 1, method: 'getSlot' }),
    httpPost(rpc, '/v1Block', { jsonrpc: '2.0', id: 1, method: 'getBlockTime', params: [0] }),
    httpPost(rpc, '/v1Peers', { jsonrpc: '2.0', id: 1, method: 'getClusterPeers' }),
  ]);

  const blockHeight = slotResp?.result || 0;
  const blockTime = blockResp?.result || null;
  const peers = Array.isArray(peersResp?.result) ? peersResp.result : [];

  return {
    blockHeight,
    blockTime,
    peerCount: peers.length,
    peers: peers.slice(0, 5), // first 5 for detail
  };
}

async function fetchVersionInfo(rpc) {
  try {
    const resp = await httpPost(rpc, '/v1Version', { jsonrpc: '2.0', id: 1, method: 'getVersion' });
    return resp?.result || {};
  } catch {
    return {};
  }
}

async function fetchRewardsSummary(rpc, address) {
  // Fetch stake accounts for wallet, then fetch rewards for each
  const allAccountsResp = await httpPost(rpc, '/v1Stake/accounts', {
    jsonrpc: '2.0', id: 1, method: 'getStakeAccounts', params: [address],
  }).catch(() => null);

  const stakeAccounts = (allAccountsResp?.result?.value || [])
    .filter(a => a.owner && (!Array.isArray(a.owner) || a.owner.length > 0))
    .map(a => a.pubkey || a);

  if (stakeAccounts.length === 0) return null;

  const rewardsResults = await Promise.all(
    stakeAccounts.slice(0, 10).map(async (sa) => {
      try {
        const resp = await httpPost(rpc, '/v1Stake/rewards', {
          jsonrpc: '2.0', id: 1, method: 'getStakeRewards', params: [sa],
        });
        const rewards = resp?.result?.rewards || [];
        let total = BigInt(0);
        for (const r of rewards) {
          total += BigInt(r.estimatedReward || 0);
        }
        return { stakeAccount: sa, estimatedRewards: total.toString(), estimatedRewardsFormatted: formatAether(total.toString()) };
      } catch {
        return { stakeAccount: sa, estimatedRewards: '0', estimatedRewardsFormatted: '0 AETH' };
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
}

function printDashboard(data, errors, includeValidator) {
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