#!/usr/bin/env node
/**
 * aether-cli info
 *
 * Display a comprehensive snapshot of the local validator's identity,
 * stake status, node version, sync state, and network connectivity.
 *
 * Usage:
 *   aether info                    Full info dump (all sections)
 *   aether info --identity         Validator identity only
 *   aether info --stake            Stake & delegation info only
 *   aether info --network          Network/peer info only
 *   aether info --json             JSON output for all sections
 *   aether info --rpc <url>        Use custom RPC endpoint
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

// Import SDK for real blockchain RPC calls
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

const CLI_VERSION = '1.1.6';

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function getValidatorIdentityPath() {
  return path.join(getAetherDir(), 'validator-identity.json');
}

function loadConfig() {
  const p = getConfigPath();
  if (!fs.existsSync(p)) return {};
  try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return {}; }
}

// ---------------------------------------------------------------------------
// SDK helpers - Real blockchain RPC calls via Aether SDK
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

function formatAether(lamports) {
  if (lamports === undefined || lamports === null) return '—';
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

// ---------------------------------------------------------------------------
// Parse args
// ---------------------------------------------------------------------------

function parseArgs() {
  return process.argv.slice(2);
}

function hasFlag(args, ...flags) {
  return flags.some(f => args.includes(f));
}

function getFlag(args, ...flags) {
  for (const f of flags) {
    const idx = args.indexOf(f);
    if (idx !== -1 && args[idx + 1] && !args[idx + 1].startsWith('-')) return args[idx + 1];
  }
  return null;
}

// ---------------------------------------------------------------------------
// Section 1: Validator Identity
// ---------------------------------------------------------------------------

async function getIdentity(rpcUrl) {
  const identityPath = getValidatorIdentityPath();
  const identity = fs.existsSync(identityPath)
    ? JSON.parse(fs.readFileSync(identityPath, 'utf8'))
    : null;

  const cfg = loadConfig();
  const defaultWallet = cfg.defaultWallet || null;

  let delegatedStake = 0;
  let stakeStatus = 'unknown';

  if (identity && identity.vote_account) {
    try {
      const client = createClient(rpcUrl);
      const res = await client.getAccountInfo(identity.vote_account);
      if (res && !res.error) {
        delegatedStake = res.lamports || 0;
        stakeStatus = 'active';
      }
    } catch { /* RPC not reachable */ }
  }

  return {
    identity,
    defaultWallet,
    vote_account: identity?.vote_account || null,
    node_id: identity?.node_id || identity?.identity || null,
    stake_account: identity?.stake_account || null,
    delegated_stake: delegatedStake,
    delegated_stake_formatted: formatAether(delegatedStake),
    stake_status: stakeStatus,
    pid: identity?.pid || null,
    uptime_str: identity?.pid ? `PID ${identity.pid}` : null,
    cli_version: CLI_VERSION,
  };
}

// ---------------------------------------------------------------------------
// Section 2: Sync & Network Status
// ---------------------------------------------------------------------------

async function getNetworkStatus(rpcUrl) {
  let slot = null, blockHeight = null, epoch = null, slotIndex = null, slotsInEpoch = null;
  let TPS = null, blockTime = null;
  let peers = [];
  let totalPeers = 0;

  try {
    const client = createClient(rpcUrl);
    const results = await Promise.allSettled([
      client.getSlot(),
      client.getEpochInfo(),
      client.getBlockHeight(),
      client.getClusterPeers(),
      { status: 'rejected', reason: new Error('Perf endpoint not in SDK') },
    ]);

    const [slotRes, epochRes, blockHeightRes, peersRes] = results;

    if (slotRes.status === 'fulfilled' && slotRes.value !== null) {
      slot = typeof slotRes.value === 'object' ? slotRes.value.slot : slotRes.value;
    }
    if (epochRes.status === 'fulfilled' && epochRes.value) {
      const ei = epochRes.value;
      epoch = ei.epoch;
      slotIndex = ei.slotIndex ?? ei.slot_index;
      slotsInEpoch = ei.slotsInEpoch ?? ei.slots_in_epoch;
      blockTime = ei.blockTime ?? ei.block_time ?? null;
    }
    if (blockHeightRes.status === 'fulfilled' && blockHeightRes.value !== null) {
      blockHeight = typeof blockHeightRes.value === 'object'
        ? blockHeightRes.value.blockHeight
        : blockHeightRes.value;
    }
    if (peersRes.status === 'fulfilled' && peersRes.value) {
      peers = Array.isArray(peersRes.value) ? peersRes.value : (peersRes.value.peers || []);
      totalPeers = peers.length;
    }
    
    // Get TPS from SDK
    try {
      TPS = await client.getTPS();
    } catch { /* TPS not available */ }
  } catch { /* Network info unavailable */ }

  let syncState = 'unknown';
  if (slot !== null && blockHeight !== null) {
    const lag = slot - blockHeight;
    if (lag <= 1) syncState = 'synced';
    else if (lag <= 10) syncState = 'catching_up';
    else syncState = 'behind';
  } else if (slot !== null) {
    syncState = 'synced';
  }

  let epochProgress = null;
  if (slotIndex !== null && slotsInEpoch !== null && slotsInEpoch > 0) {
    epochProgress = Math.min(100, Math.round((slotIndex / slotsInEpoch) * 100));
  }

  return {
    slot,
    block_height: blockHeight,
    sync_state: syncState,
    epoch,
    slot_index: slotIndex,
    slots_in_epoch: slotsInEpoch,
    epoch_progress: epochProgress,
    tps: TPS,
    block_time_ms: blockTime,
    peers,
    total_peers: totalPeers,
    rpc_url: rpcUrl,
  };
}

// ---------------------------------------------------------------------------
// Section 3: Stake & Delegation Summary
// ---------------------------------------------------------------------------

async function getStakeSummary(rpcUrl) {
  let totalDelegated = BigInt(0);
  let activePositions = 0;
  let deactivatingPositions = 0;
  let positions = [];
  let err = null;

  try {
    const cfg = loadConfig();
    const walletAddr = cfg.defaultWallet;

    if (walletAddr) {
      const client = createClient(rpcUrl);
      const rawAddr = walletAddr.startsWith('ATH') ? walletAddr.slice(3) : walletAddr;
      const res = await client.getStakePositions(rawAddr);
      if (res && !res.error) {
        const accounts = Array.isArray(res) ? res : (res.accounts || res.stakes || res.delegations || []);
        for (const acc of accounts) {
          const lamports = BigInt(acc.stake_lamports || acc.lamports || acc.amount || 0);
          const status = (acc.status || acc.state || 'active').toLowerCase();
          const validator = acc.validator || acc.voter || acc.vote_account || 'unknown';

          if (status === 'active') activePositions++;
          else if (status.includes('deactivating') || status.includes('deactivated')) deactivatingPositions++;

          totalDelegated += lamports;
          positions.push({
            stake_account: acc.pubkey || acc.publicKey || acc.account || acc.stakeAccount || 'unknown',
            validator,
            lamports: lamports.toString(),
            lamports_formatted: formatAether(lamports.toString()),
            status,
          });
        }
      }
    }
  } catch (e) {
    err = e.message;
  }

  return {
    total_delegated: totalDelegated.toString(),
    total_delegated_formatted: formatAether(totalDelegated.toString()),
    active_positions: activePositions,
    deactivating_positions: deactivatingPositions,
    total_positions: positions.length,
    positions,
    error: err,
  };
}

// ---------------------------------------------------------------------------
// Human-readable output
// ---------------------------------------------------------------------------

function printSectionDivider(title) {
  const width = 56;
  const dashes = width - title.length - 4;
  console.log(`\n${C.bright}${C.cyan}── ${title} ${'─'.repeat(Math.max(0, dashes))}${C.reset}`);
}

function printRow(label, value, valueColor) {
  const vc = valueColor || C.reset;
  console.log(`  ${C.dim}${label.padEnd(22)}${C.reset} ${vc}${value}${C.reset}`);
}

function printIdentitySection(id) {
  printSectionDivider('Validator Identity');

  if (!id.identity && !id.defaultWallet) {
    console.log(`  ${C.yellow}⚠ Validator identity file not found.${C.reset}`);
    console.log(`  ${C.dim}  Run: aether init${C.reset}`);
    console.log(`  ${C.dim}  Or: aether validator start${C.reset}\n`);
    return;
  }

  if (id.node_id) printRow('Node ID', id.node_id, C.bright);
  if (id.vote_account) printRow('Vote account', id.vote_account, C.bright);
  if (id.stake_account) printRow('Stake account', id.stake_account, C.bright);
  if (id.defaultWallet) printRow('Default wallet', id.defaultWallet, C.bright);
  printRow('Delegated stake', id.delegated_stake_formatted, C.green);
  printRow('Stake status', id.stake_status, id.stake_status === 'active' ? C.green : C.yellow);
  if (id.pid) printRow('PID', id.pid.toString(), C.dim);
  printRow('CLI version', id.cli_version, C.dim);
  console.log();
}

function printNetworkSection(net) {
  printSectionDivider('Network & Sync');

  const syncColors = { synced: C.green, catching_up: C.yellow, behind: C.red, unknown: C.dim };
  const syncLabels = { synced: '✓ Synced', catching_up: '⚠ Catching up', behind: '✗ Behind', unknown: '— Unknown' };

  printRow('Sync state', syncLabels[net.sync_state] || net.sync_state, syncColors[net.sync_state] || C.reset);

  if (net.slot !== null) printRow('Current slot', net.slot.toLocaleString(), C.bright);
  if (net.block_height !== null) printRow('Block height', net.block_height.toLocaleString(), C.bright);
  if (net.slot !== null && net.block_height !== null) {
    const lag = net.slot - net.block_height;
    printRow('Slot lag', lag.toLocaleString(), lag <= 1 ? C.green : lag <= 10 ? C.yellow : C.red);
  }

  console.log();

  if (net.epoch !== null) printRow('Epoch', net.epoch.toString(), C.bright);
  if (net.epoch_progress !== null) {
    const filled = Math.floor(net.epoch_progress / 5);
    const bar = '█'.repeat(filled) + '░'.repeat(20 - filled);
    printRow('Epoch progress', `${bar} ${net.epoch_progress}%`, C.cyan);
  }
  if (net.slots_in_epoch !== null) printRow('Slots in epoch', net.slots_in_epoch.toLocaleString(), C.dim);
  if (net.slot_index !== null) {
    printRow('Slot in epoch', `${net.slot_index.toLocaleString()} / ${net.slots_in_epoch?.toLocaleString() || '?'}`, C.dim);
  }

  console.log();

  if (net.tps !== null) printRow('TPS (est.)', net.tps.toLocaleString(), C.green);
  if (net.block_time_ms !== null) printRow('Block time', `${(net.block_time_ms / 1000).toFixed(2)}s`, C.dim);
  printRow('RPC endpoint', net.rpc_url, C.dim);

  console.log();

  if (net.total_peers > 0) {
    printRow('Connected peers', net.total_peers.toString(), C.green);
    const shown = net.peers.slice(0, 10);
    for (const peer of shown) {
      const pubkey = peer.pubkey || peer.identity || peer.id || 'unknown';
      const ip = peer.ip || peer.remote || '—';
      const version = peer.version || peer.agent || '';
      const shortKey = pubkey.length > 16 ? pubkey.slice(0, 8) + '…' + pubkey.slice(-6) : pubkey;
      console.log(`    ${C.dim}  ·${C.reset} ${C.cyan}${shortKey}${C.reset} ${C.dim}${ip} ${version}${C.reset}`);
    }
    if (net.peers.length > 10) {
      console.log(`    ${C.dim}  … and ${net.peers.length - 10} more peers${C.reset}`);
    }
  } else {
    printRow('Connected peers', '0 — not connected', C.yellow);
    console.log(`  ${C.dim}  Validator may still be starting or network is unavailable.${C.reset}`);
  }

  console.log();
}

function printStakeSection(stake) {
  printSectionDivider('Stake & Delegations');

  if (stake.error) {
    console.log(`  ${C.yellow}⚠ Could not fetch stake info: ${stake.error}${C.reset}`);
    console.log(`  ${C.dim}  Set AETHER_RPC to your validator's RPC endpoint.${C.reset}\n`);
    return;
  }

  printRow('Total delegated', stake.total_delegated_formatted, C.green);
  printRow('Active positions', stake.active_positions.toString(), stake.active_positions > 0 ? C.green : C.dim);
  printRow('Deactivating', stake.deactivating_positions.toString(), stake.deactivating_positions > 0 ? C.yellow : C.dim);

  console.log();

  if (stake.positions.length === 0) {
    console.log(`  ${C.dim}  No stake delegations found.${C.reset}`);
    console.log(`  ${C.dim}  Delegate: aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
    return;
  }

  for (const pos of stake.positions) {
    const shortAcct = pos.stake_account.length > 20
      ? pos.stake_account.slice(0, 8) + '…' + pos.stake_account.slice(-8)
      : pos.stake_account;
    const shortVal = pos.validator.length > 20
      ? pos.validator.slice(0, 8) + '…' + pos.validator.slice(-8)
      : pos.validator;
    const statusColor = pos.status === 'active' ? C.green : C.yellow;

    console.log(`  ${C.dim}┌─ ${shortAcct}${C.reset}`);
    console.log(`  │  ${C.dim}Validator:${C.reset} ${C.cyan}${shortVal}${C.reset}`);
    console.log(`  │  ${C.dim}Amount:${C.reset}     ${C.bright}${pos.lamports_formatted}${C.reset}`);
    console.log(`  │  ${C.dim}Status:${C.reset}     ${statusColor}${pos.status}${C.reset}`);
    console.log(`  ${C.dim}└${C.reset}`);
  }
  console.log();
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

function printJson(id, net, stake) {
  console.log(JSON.stringify({
    identity: {
      node_id: id.node_id,
      vote_account: id.vote_account,
      stake_account: id.stake_account,
      default_wallet: id.defaultWallet,
      delegated_stake: id.delegated_stake,
      delegated_stake_formatted: id.delegated_stake_formatted,
      stake_status: id.stake_status,
      pid: id.pid,
      cli_version: id.cli_version,
    },
    network: {
      slot: net.slot,
      block_height: net.block_height,
      sync_state: net.sync_state,
      epoch: net.epoch,
      slot_index: net.slot_index,
      slots_in_epoch: net.slots_in_epoch,
      epoch_progress: net.epoch_progress,
      tps: net.tps,
      block_time_ms: net.block_time_ms,
      total_peers: net.total_peers,
      peers: net.peers.map(p => ({ pubkey: p.pubkey || p.identity || p.id, ip: p.ip })),
      rpc_url: net.rpc_url,
    },
    stake: {
      total_delegated: stake.total_delegated,
      total_delegated_formatted: stake.total_delegated_formatted,
      active_positions: stake.active_positions,
      deactivating_positions: stake.deactivating_positions,
      total_positions: stake.total_positions,
      positions: stake.positions,
      error: stake.error,
    },
    fetched_at: new Date().toISOString(),
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const args = parseArgs();
  const asJson = hasFlag(args, '--json', '-j');
  const showIdentity = hasFlag(args, '--identity', '-i');
  const showNetwork = hasFlag(args, '--network', '-n');
  const showStake = hasFlag(args, '--stake', '-s');
  const rpcUrl = getFlag(args, '--rpc', '-r') || getDefaultRpc();
  const showAll = !showIdentity && !showNetwork && !showStake;

  const [identityData, networkData, stakeData] = await Promise.all([
    getIdentity(rpcUrl),
    getNetworkStatus(rpcUrl),
    getStakeSummary(rpcUrl),
  ]);

  if (asJson) {
    printJson(identityData, networkData, stakeData);
    return;
  }

  console.log(`\n${C.bright}${C.cyan}╔════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║              AETHER VALIDATOR — Info                    ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚════════════════════════════════════════════════════════════╝${C.reset}`);
  console.log(`  ${C.dim}AETHER_RPC: ${rpcUrl}${C.reset}`);

  if (showAll || showIdentity) printIdentitySection(identityData);
  if (showAll || showNetwork) printNetworkSection(networkData);
  if (showAll || showStake) printStakeSection(stakeData);

  console.log(`  ${C.dim}Run with --json for scripted output.${C.reset}\n`);
}

main().catch(err => {
  console.error(`\n${C.red}✗ Info command failed:${C.reset} ${err.message}\n`);
  process.exit(1);
});

module.exports = { infoCommand: main };
