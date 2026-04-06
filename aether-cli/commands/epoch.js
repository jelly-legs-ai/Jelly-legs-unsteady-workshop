#!/usr/bin/env node
/**
 * aether-cli epoch
 *
 * Display current epoch information including timing, schedule,
 * slots per epoch, and estimated staking rewards rate.
 *
 * Usage:
 *   aether epoch                    Show current epoch with timing breakdown
 *   aether epoch --json            JSON output for scripting/monitoring
 *   aether epoch --rpc <url>       Query a specific RPC endpoint
 *   aether epoch --schedule        Show upcoming epoch schedule
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

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

// ---------------------------------------------------------------------------
// SDK Import - Real blockchain RPC calls via @jellylegsai/aether-sdk
// ---------------------------------------------------------------------------

const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // [node, index.js, epoch, ...]
  return {
    rpc: getDefaultRpc(),
    asJson: args.indexOf('--json') !== -1 || args.indexOf('-j') !== -1,
    showSchedule: args.indexOf('--schedule') !== -1 || args.indexOf('-s') !== -1,
    rpcUrl: getDefaultRpc(),
  };
}

// ---------------------------------------------------------------------------
// Fetch epoch info from RPC using SDK
// ---------------------------------------------------------------------------

async function fetchEpochInfo(rpc) {
  // Use SDK for real blockchain RPC calls
  const client = new aether.AetherClient({ rpcUrl: rpc });
  try {
    const epochInfo = await client.getEpochInfo();
    if (epochInfo && (epochInfo.epoch !== undefined || epochInfo.current_epoch)) {
      return { data: epochInfo, source: 'aether-sdk' };
    }
  } catch(e) {
    throw new Error('Failed to fetch epoch info from RPC. Is your validator running?');
  }

  throw new Error('Failed to fetch epoch info from RPC. Is your validator running?');
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatDuration(seconds) {
  if (seconds < 0) return '\u2014';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 24) {
    const d = Math.floor(h / 24);
    return d + 'd ' + (h % 24) + 'h';
  }
  if (h > 0) return h + 'h ' + m + 'm';
  if (m > 0) return m + 'm ' + s + 's';
  return s + 's';
}

function fmtPct(value, decimals) {
  decimals = decimals || 1;
  return (value || 0).toFixed(decimals) + '%';
}

// ---------------------------------------------------------------------------
// Box drawing helpers
// ---------------------------------------------------------------------------

// Box drawing characters
const BOX_H  = '\u2500'; // ─
const BOX_V  = '\u2502'; // │
const BOX_TL = '\u256d'; // ╭
const BOX_TR = '\u256e'; // ╮
const BOX_BL = '\u2570'; // ╰
const BOX_BR = '\u256f'; // ╯
const BOX_Cross = '\u253c'; // ┼

function makeBoxLine(chars, width) {
  let s = '';
  for (let i = 0; i < width; i++) s += chars;
  return s;
}

function makeSectionHeader(label) {
  const total = 62;
  const labelWithSpaces = ' ' + label + ' ';
  const remaining = total - 4 - labelWithSpaces.length; // 4 for ╼ on each side
  const half = Math.floor(remaining / 2);
  const left = makeBoxLine('\u2550', half);
  const right = makeBoxLine('\u2550', remaining - half);
  return C.bright + C.cyan + '\u256e' + left + '\u2554' + labelWithSpaces + '\u2557' + right + '\u256f' + C.reset;
}

// ---------------------------------------------------------------------------
// Main display
// ---------------------------------------------------------------------------

async function showEpochInfo(opts) {
  const rpc = opts.rpcUrl;
  const { data, source } = await fetchEpochInfo(rpc);

  // Normalise fields across different RPC response formats
  const epoch           = data.epoch ?? data.current_epoch ?? 0;
  const slotIndex       = data.slotIndex ?? data.current_slot ?? 0;
  const slotsInEpoch     = data.slotsInEpoch ?? data.slots_per_epoch ?? 8192;
  const epochProgress    = slotsInEpoch > 0 ? (slotIndex / slotsInEpoch) * 100 : 0;
  const absoluteSlot     = data.absoluteSlot ?? data.slot ?? 0;
  const totalStaked      = BigInt(data.totalStaked ?? data.total_staked ?? data.stake ?? 0);
  const rewardsPerEpoch  = BigInt(data.rewardsPerEpoch ?? data.rewards_per_epoch ?? data.rewards ?? 0);

  // Estimate seconds per slot from slot data
  const epochDurationSecs = data.epochDurationSecs ?? (slotsInEpoch * 0.4); // ~400ms/slot default
  const secsPerSlot       = epochDurationSecs / slotsInEpoch;
  const secondsIntoEpoch  = slotIndex * secsPerSlot;
  const secondsRemaining  = (slotsInEpoch - slotIndex) * secsPerSlot;

  // APY estimate: rewards per epoch / total staked * epochs per year
  const epochsPerYear = 365 * 24 * 3600 / epochDurationSecs;
  const apyRate = totalStaked > 0n
    ? (Number(rewardsPerEpoch) / Number(totalStaked)) * epochsPerYear
    : 0;
  const apyBps = Math.round(apyRate * 10000);

  if (opts.asJson) {
    const out = {
      epoch: epoch,
      slotIndex: slotIndex,
      slotsInEpoch: slotsInEpoch,
      absoluteSlot: absoluteSlot,
      epochProgress: epochProgress,
      secondsIntoEpoch: Math.round(secondsIntoEpoch),
      secondsRemaining: Math.round(secondsRemaining),
      totalStaked: totalStaked.toString(),
      totalStakedFormatted: formatAether(totalStaked),
      rewardsPerEpoch: rewardsPerEpoch.toString(),
      rewardsPerEpochFormatted: formatAether(rewardsPerEpoch),
      estimatedApyBps: apyBps,
      estimatedApy: fmtPct(apyRate),
      source: source,
      fetchedAt: new Date().toISOString(),
    };
    console.log(JSON.stringify(out, null, 2));
    return;
  }

  // ASCII art header
  console.log('');
  const line1 = C.bright + C.cyan + BOX_TL + makeBoxLine(BOX_H, 60) + BOX_TR + C.reset;
  console.log(line1);
  const line2 = C.bright + C.cyan + BOX_V + '         AeTHer Epoch ' + epoch + ' Info         ' + BOX_V + C.reset;
  console.log(line2);
  const line3 = C.bright + C.cyan + BOX_BL + makeBoxLine(BOX_H, 60) + BOX_BR + C.reset;
  console.log(line3);
  console.log('');

  console.log('  ' + C.dim + 'RPC: ' + rpc + C.reset);
  console.log('');

  // ── Epoch timing ───────────────────────────────────────────────────────
  console.log(makeSectionHeader('Epoch Timing'));

  const progressBars = 40;
  const filled = Math.round((epochProgress / 100) * progressBars);
  const empty = progressBars - filled;
  const bar = C.green + '#'.repeat(filled) + C.dim + '\u2500'.repeat(empty) + C.reset;

  console.log('  ' + C.dim + '  Progress:  [' + bar + '] ' + C.bright + fmtPct(epochProgress) + C.reset);
  console.log('  ' + C.dim + '  Slot:      ' + C.reset + C.bright + slotIndex.toLocaleString() + C.reset + ' / ' + slotsInEpoch.toLocaleString() + ' slots into epoch');
  console.log('  ' + C.dim + '  Abs slot:   ' + C.reset + absoluteSlot.toLocaleString());
  console.log('  ' + C.dim + '  Elapsed:   ' + C.reset + formatDuration(Math.round(secondsIntoEpoch)));
  console.log('  ' + C.dim + '  Remaining: ' + C.reset + C.yellow + formatDuration(Math.round(secondsRemaining)) + C.reset);
  console.log('  ' + C.dim + '  Duration:  ' + C.reset + '~' + formatDuration(Math.round(epochDurationSecs)) + ' per epoch');
  console.log('');

  // ── Staking rewards ─────────────────────────────────────────────────────
  console.log(makeSectionHeader('Staking Rewards'));

  console.log('  ' + C.dim + '  Network stake:   ' + C.reset + C.bright + formatAether(totalStaked) + C.reset);
  console.log('  ' + C.dim + '  Rewards/epoch:   ' + C.reset + C.green + formatAether(rewardsPerEpoch) + C.reset);
  console.log('  ' + C.dim + '  Estimated APY:  ' + C.reset + C.green + C.bright + fmtPct(apyRate) + C.reset + ' ' + C.dim + '(~' + (apyBps / 100).toFixed(0) + ' bps)' + C.reset);
  console.log('');

  // ── Epoch schedule ──────────────────────────────────────────────────────
  if (opts.showSchedule) {
    console.log(makeSectionHeader('Upcoming Epochs'));
    const startSlotNext = absoluteSlot + (slotsInEpoch - slotIndex);
    for (let i = 0; i < 5; i++) {
      const e = epoch + i;
      const start = startSlotNext + i * slotsInEpoch;
      const end = start + slotsInEpoch - 1;
      const isNext = i === 0 ? ' ' + C.green + '(next)' + C.reset : '';
      console.log('  ' + C.dim + '  Epoch ' + String(e).padStart(4) + ': slots ' + start.toLocaleString() + ' \u2013 ' + end.toLocaleString() + isNext + C.reset);
    }
    console.log('');
  }

  // ── Raw data ─────────────────────────────────────────────────────────────
  console.log(makeSectionHeader('Raw RPC Data'));
  console.log('  ' + C.dim + '  Source: ' + source + C.reset);
  const rawPreview = JSON.stringify(data).substring(0, 80);
  console.log('  ' + C.dim + '  ' + rawPreview + C.reset);
  console.log('');

  console.log('  ' + C.dim + 'Run "aether validators list" to see validator performance for epoch ' + epoch + '.' + C.reset);
  console.log('  ' + C.dim + 'Run "aether rewards list --address <addr>" to check your staking rewards.' + C.reset);
  console.log('');
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

async function epochCommand() {
  const opts = parseArgs();
  try {
    await showEpochInfo(opts);
  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({ error: err.message }, null, 2));
    } else {
      console.log('');
      console.log('  ' + C.red + '\u2514 Error: ' + C.reset + ' ' + err.message);
      console.log('  ' + C.dim + 'Set a custom RPC: AETHER_RPC=https://your-rpc-url' + C.reset);
      console.log('');
    }
    process.exit(1);
  }
}

module.exports = { epochCommand };

if (require.main === module) {
  epochCommand();
}
