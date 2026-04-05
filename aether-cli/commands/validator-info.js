#!/usr/bin/env node
/**
 * aether-cli validator-info
 *
 * Inspect a specific validator by identity address or name.
 * Shows stake, APY, score, commission, tier, uptime, epoch performance,
 * and a breakdown of delegators.
 *
 * Usage:
 *   aether validator info <addressOrName>     Inspect a validator
 *   aether validator info <addressOrName> --json   JSON output for scripting
 *   aether validator info <addressOrName> --rpc <url>   Use specific RPC
 *
 * Examples:
 *   aether validator info ATH3mGH...
 *   aether validator info jellylegs --json
 *   aether validator info --address ATH3mGH... --rpc http://custom:8899
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
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: 8000,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve({ raw: data }); } });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

function httpPost(rpcUrl, path, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 8000,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(bodyStr) },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve(data); } });
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
  const raw = process.argv.slice(3); // [node, index.js, validator, info, ...]
  const opts = {
    rpc: DEFAULT_RPC,
    target: null,
    asJson: false,
  };

  for (let i = 0; i < raw.length; i++) {
    const arg = raw[i];
    if ((arg === '--rpc' || arg === '-r') && raw[i + 1] && !raw[i + 1].startsWith('-')) {
      opts.rpc = raw[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.asJson = true;
    } else if ((arg === '--address' || arg === '-a') && raw[i + 1] && !raw[i + 1].startsWith('-')) {
      opts.target = raw[++i];
    } else if (!arg.startsWith('-') && !opts.target) {
      opts.target = arg;
    }
  }

  return opts;
}

// ---------------------------------------------------------------------------
// Data fetchers
// ---------------------------------------------------------------------------

/** Fetch all validators and find the matching one */
async function fetchValidatorByIdentity(rpc, identity) {
  // identity can be a full address, partial address, or name/moniker
  const validators = await fetchAllValidators(rpc);
  if (!validators || validators.length === 0) return null;

  const isAddress = identity.startsWith('ATH');

  let match = null;

  if (isAddress) {
    // Exact or prefix match on pubkey/identity
    match = validators.find(v =>
      (v.pubkey && (v.pubkey === identity || v.pubkey.startsWith(identity))) ||
      (v.address && (v.address === identity || v.address.startsWith(identity))) ||
      (v.identity && (v.identity === identity || v.identity.startsWith(identity)))
    );
  }

  if (!match) {
    // Try name/moniker match (case-insensitive partial)
    const lower = identity.toLowerCase();
    match = validators.find(v =>
      (v.name && v.name.toLowerCase().includes(lower)) ||
      (v.moniker && v.moniker.toLowerCase().includes(lower))
    );
  }

  if (!match && identity.length >= 8) {
    // Try prefix match on any field
    match = validators.find(v => {
      const pk = v.pubkey || v.address || v.identity || '';
      return pk.startsWith(identity) || pk.endsWith(identity);
    });
  }

  return match;
}

/** Fetch all validators from the network */
async function fetchAllValidators(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/validators');
    if (res && !res.error) {
      if (Array.isArray(res)) return res;
      if (res.validators && Array.isArray(res.validators)) return res.validators;
      if (res.accounts && Array.isArray(res.accounts)) return res.accounts;
    }
    const res2 = await httpPost(rpc, '/v1/validators', {});
    if (res2 && !res2.error) {
      if (Array.isArray(res2)) return res2;
      if (res2.validators && Array.isArray(res2.validators)) return res2.validators;
    }
    return [];
  } catch {
    return [];
  }
}

/** Fetch epoch info for APY and epoch calculations */
async function fetchEpochInfo(rpc) {
  try {
    return await httpRequest(rpc, '/v1/epoch-info');
  } catch {
    return null;
  }
}

/** Fetch supply for APY estimation */
async function fetchSupply(rpc) {
  try {
    return await httpRequest(rpc, '/v1/supply');
  } catch {
    return null;
  }
}

/** Fetch delegators for a specific validator */
async function fetchDelegators(rpc, validatorPubkey) {
  try {
    const res = await httpRequest(rpc, `/v1/validator/${encodeURIComponent(validatorPubkey)}/delegators`);
    if (res && !res.error) {
      return Array.isArray(res) ? res : (res.delegators || []);
    }
    return [];
  } catch {
    return [];
  }
}

/** Fetch recent performance history for a validator */
async function fetchValidatorPerformance(rpc, validatorPubkey) {
  try {
    const res = await httpRequest(rpc, `/v1/validator/${encodeURIComponent(validatorPubkey)}/performance`);
    if (res && !res.error) {
      return res;
    }
    return null;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Normalise a validator record
// ---------------------------------------------------------------------------

function normalise(v) {
  const pubkey = v.pubkey || v.address || v.identity || v.id || null;
  const name = v.name || v.moniker || v.label || null;
  const tier = (v.tier || v.node_type || v.type || 'full').toLowerCase();
  const stake = typeof v.stake === 'bigint' ? v.stake
    : typeof v.stake === 'object' ? BigInt(v.stake.toString())
    : BigInt(v.stake || v.delegatedStake || v.stake_lamports || v.lamports || 0);
  const score = v.score !== undefined ? v.score
    : v.uptime !== undefined ? Math.round(v.uptime * 100)
    : null;
  const apy = v.apy !== undefined ? v.apy
    : v.apy_bps !== undefined ? v.apy_bps / 100
    : null;
  const commission = v.commission !== undefined ? v.commission
    : v.commission_bps !== undefined ? v.commission_bps / 100
    : null;
  const version = v.version || v.agent || null;
  const ip = v.ip || v.remote || null;
  const lastVote = v.last_vote || v.lastVote || null;
  const stakeAccounts = v.stake_accounts || v.stakeAccounts || v.delegators || null;
  const activatedStake = v.activated_stake || null;
  const lastEpochStake = v.last_epoch_stake || v.stake_last_epoch || null;
  const credits = v.credits || v.epoch_credits || null;
  const rootSlot = v.root_slot || v.rootSlot || null;
  const delinquent = v.delinquent || false;

  return {
    pubkey, name, tier, stake, score, apy, commission,
    version, ip, lastVote, stakeAccounts, activatedStake,
    lastEpochStake, credits, rootSlot, delinquent,
    _raw: v,
  };
}

function formatAether(lamports) {
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatAethFull(lamports) {
  return (Number(lamports) / 1e9).toFixed(6) + ' AETH';
}

function formatNumber(n) {
  if (n === null || n === undefined) return '—';
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function tierBadge(tier) {
  if (tier === 'full') return `${C.cyan}◆ FULL${C.reset}`;
  if (tier === 'lite') return `${C.yellow}◇ LITE${C.reset}`;
  if (tier === 'observer') return `${C.green}○ OBSERVER${C.reset}`;
  return `${C.dim}[${tier}]${C.reset}`;
}

function tierColor(tier) {
  if (tier === 'full') return C.cyan;
  if (tier === 'lite') return C.yellow;
  if (tier === 'observer') return C.green;
  return C.reset;
}

function scoreColor(score) {
  if (score === null || score === undefined) return C.dim;
  if (score >= 80) return C.green;
  if (score >= 50) return C.yellow;
  return C.red;
}

function apyColor(apy) {
  if (apy === null || apy === undefined) return C.dim;
  if (apy >= 8) return C.green;
  if (apy >= 5) return C.cyan;
  if (apy >= 2) return C.yellow;
  return C.red;
}

// ---------------------------------------------------------------------------
// Render output
// ---------------------------------------------------------------------------

function renderHeader(v) {
  const shortAddr = v.pubkey
    ? v.pubkey.slice(0, 12) + '...' + v.pubkey.slice(-8)
    : 'unknown';

  console.log();
  console.log(`${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║${C.reset}              ${C.bright}VALIDATOR DETAILS${C.reset}  ${C.dim}${shortAddr}${C.reset}                  ${C.bright}║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════════════════════╝${C.reset}`);
  console.log();

  if (v.name) {
    console.log(`  ${C.cyan}${C.bright}${v.name}${C.reset}`);
  }
  if (v.pubkey) {
    console.log(`  ${C.dim}${v.pubkey}${C.reset}`);
  }
  console.log();

  // Tier + delinquent banner
  const tierStr = tierBadge(v.tier);
  const deliqStr = v.delinquent ? `  ${C.red}⚠ DELINQUENT${C.reset}` : '';
  console.log(`  ${tierStr}${deliqStr}`);
  console.log();
}

function renderStats(v, epochInfo) {
  const stakeFormatted = formatAether(v.stake);
  const stakeAeth = Number(v.stake) / 1e9;
  const score = v.score;
  const apy = v.apy;
  const commission = v.commission;

  // APY bar (0-15% range for display)
  const apyBarLen = 12;
  const apyDisplay = apy !== null && apy !== undefined ? apy : 0;
  const apyFillLen = Math.min(apyBarLen, Math.round((apyDisplay / 15) * apyBarLen));

  // Commission bar (0-100%)
  const commBarLen = 10;
  const commDisplay = commission !== null && commission !== undefined ? commission : 0;
  const commFillLen = Math.round((commDisplay / 100) * commBarLen);

  console.log(`  ${C.bright}┌───────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Stake${C.reset}          ${formatAether(v.stake).padEnd(15)}   ${C.cyan}${formatAethFull(v.stake).padEnd(14)}  ${C.bright}│${C.reset}`);

  if (v.lastEpochStake !== null && v.lastEpochStake !== undefined) {
    const lastAeth = Number(v.lastEpochStake) / 1e9;
    const change = stakeAeth - lastAeth;
    const changeStr = change >= 0 ? `+${change.toFixed(2)}` : change.toFixed(2);
    const changeColor = change >= 0 ? C.green : C.red;
    console.log(`  ${C.bright}│${C.reset}  ${C.dim}Last epoch${C.reset}    ${formatAethFull(v.lastEpochStake).padEnd(15)}   ${changeColor}${changeStr} AETH${C.reset}`.padEnd(75) + `  ${C.bright}│${C.reset}`);
  }

  if (score !== null && score !== undefined) {
    const sc = scoreColor(score);
    const scoreLabel = score >= 80 ? 'excellent' : score >= 50 ? 'good' : score >= 20 ? 'fair' : 'poor';
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Score${C.reset}          ${sc}${score}${C.reset}% (${scoreLabel})`.padEnd(75) + `  ${C.bright}│${C.reset}`);
  }

  if (apy !== null && apy !== undefined) {
    const ac = apyColor(apy);
    const apyBar = `${ac}${'█'.repeat(apyFillLen)}${C.dim}${'░'.repeat(apyBarLen - apyFillLen)}${C.reset}`;
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Est. APY${C.reset}       ${ac}${apy.toFixed(2)}%${C.reset}  ${apyBar}`.padEnd(75) + `  ${C.bright}│${C.reset}`);
  }

  if (commission !== null && commission !== undefined) {
    const commBar = `${C.yellow}${'█'.repeat(commFillLen)}${C.dim}${'░'.repeat(commBarLen - commFillLen)}${C.reset}`;
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Commission${C.reset}     ${commission.toFixed(1)}%  ${commBar}`.padEnd(75) + `  ${C.bright}│${C.reset}`);
  }

  console.log(`  ${C.bright}└───────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();
}

function renderNodeInfo(v) {
  const items = [];
  if (v.version) items.push({ label: 'Version', value: v.version, color: C.cyan });
  if (v.ip) items.push({ label: 'IP', value: v.ip, color: C.dim });
  if (v.lastVote !== null && v.lastVote !== undefined) items.push({ label: 'Last vote', value: `slot ${formatNumber(v.lastVote)}`, color: C.cyan });
  if (v.rootSlot !== null && v.rootSlot !== undefined) items.push({ label: 'Root slot', value: formatNumber(v.rootSlot), color: C.cyan });

  if (items.length === 0) return;

  console.log(`  ${C.bright}── Node Info ──────────────────────────────────────────────${C.reset}`);
  for (const item of items) {
    console.log(`  ${C.dim}${item.label}:${C.reset}  ${item.color}${item.value}${C.reset}`);
  }
  console.log();
}

function renderEpochPerformance(v, epochInfo, performance) {
  console.log(`  ${C.bright}── Epoch Performance ─────────────────────────────────────${C.reset}`);

  if (epochInfo) {
    const ep = epochInfo.epoch;
    const slotIdx = epochInfo.slot_index;
    const slotsInEp = epochInfo.slots_in_epoch;
    const progress = slotsInEp > 0 ? ((slotIdx / slotsInEp) * 100).toFixed(1) : '?';
    console.log(`  ${C.dim}Current epoch:${C.reset}  ${C.bright}${ep}${C.reset}  ${C.dim}progress: ${C.reset}${progress}%`);
    if (slotsInEp) console.log(`  ${C.dim}Slots in epoch:${C.reset} ${formatNumber(slotsInEp)}`);
    if (slotIdx !== undefined) console.log(`  ${C.dim}Current slot:${C.reset}  ${formatNumber(slotIdx)}`);
    console.log();
  }

  if (v.credits !== null && v.credits !== undefined) {
    if (Array.isArray(v.credits) && v.credits.length > 0) {
      const [ep, cr, pr] = v.credits.length >= 3
        ? [v.credits[v.credits.length - 1], v.credits[v.credits.length - 2], v.credits[v.credits.length - 3]]
        : [null, null, null];
      if (ep !== null) console.log(`  ${C.dim}Epoch credits:${C.reset}  ${formatNumber(ep)}  ${C.dim}(prev: ${formatNumber(pr)})${C.reset}`);
    } else if (typeof v.credits === 'number') {
      console.log(`  ${C.dim}Epoch credits:${C.reset}  ${formatNumber(v.credits)}`);
    }
  }

  if (performance) {
    if (performance.slots_in_epoch !== undefined) {
      console.log(`  ${C.dim}Epoch slots:${C.reset}  ${formatNumber(performance.slots_in_epoch)}`);
    }
    if (performance.slots_produced !== undefined) {
      const pct = performance.slots_in_epoch > 0
        ? ((performance.slots_produced / performance.slots_in_epoch) * 100).toFixed(1)
        : '?';
      console.log(`  ${C.dim}Slots produced:${C.reset} ${performance.slots_produced} / ${formatNumber(performance.slots_in_epoch)} ${C.dim}(${pct}%)${C.reset}`);
    }
    if (performance.credits !== undefined) {
      console.log(`  ${C.dim}Credits earned:${C.reset} ${formatNumber(performance.credits)}`);
    }
  }

  console.log();
}

function renderDelegators(delegators) {
  if (!delegators || delegators.length === 0) {
    console.log(`  ${C.bright}── Delegators ────────────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}No delegator data available for this validator.${C.reset}`);
    console.log();
    return;
  }

  console.log(`  ${C.bright}── Delegators (${delegators.length}) ──────────────────────────────────${C.reset}`);
  console.log(`  ${C.dim}┌─────────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.dim}│${C.reset}  ${C.cyan}Delegator${C.reset.padEnd(44)} ${C.cyan}Stake${C.reset.padEnd(18)} ${C.cyan}Status${C.reset.padEnd(12)} ${C.dim}│${C.reset}`);
  console.log(`  ${C.dim}├─────────────────────────────────────────────────────────────────────────┤${C.reset}`);

  const sorted = [...delegators].sort((a, b) => {
    const aLamports = Number(a.lamports || a.stake || 0);
    const bLamports = Number(b.lamports || b.stake || 0);
    return bLamports - aLamports;
  }).slice(0, 20);

  let totalDelegated = BigInt(0);

  for (const d of sorted) {
    const addr = d.address || d.pubkey || d.delegator || 'unknown';
    const shortAddr = addr.slice(0, 20) + '...' + addr.slice(-12);
    const lamports = BigInt(d.lamports || d.stake || 0);
    totalDelegated += lamports;
    const stakeFormatted = formatAether(lamports);
    const status = d.status || d.activation_status || 'active';
    const statusColor = status === 'active' ? C.green : status === 'pending' ? C.yellow : C.dim;

    console.log(
      `  ${C.dim}│${C.reset}  ${shortAddr.padEnd(46)} ${stakeFormatted.padEnd(18)} ${statusColor}${(status + '').padEnd(12)}${C.reset} ${C.dim}│${C.reset}`
    );
  }

  if (delegators.length > 20) {
    console.log(`  ${C.dim}│${C.reset}  ${C.dim}... and ${delegators.length - 20} more delegators (use --json for full list)${C.reset}`.padEnd(77) + `${C.dim}│${C.reset}`);
  }

  console.log(`  ${C.dim}└─────────────────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log(`  ${C.dim}Total delegated (shown): ${formatAether(totalDelegated)}${C.reset}`);
  console.log();
}

function renderRank(validators, v) {
  if (!validators || !v) return;

  const sorted = [...validators].sort((a, b) => {
    const aStake = typeof a.stake === 'bigint' ? Number(a.stake) : Number(a.stake || 0);
    const bStake = typeof b.stake === 'bigint' ? Number(b.stake) : Number(b.stake || 0);
    return bStake - aStake;
  });

  const rank = sorted.findIndex(m => {
    const mPub = m.pubkey || m.address || '';
    const vPub = v.pubkey || '';
    return mPub === vPub;
  });

  if (rank >= 0) {
    const pct = ((rank + 1) / sorted.length * 100).toFixed(1);
    console.log(`  ${C.bright}── Network Rank ───────────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}Stake rank:${C.reset}  ${C.bright}#${rank + 1}${C.reset} of ${sorted.length} validators  ${C.dim}(top ${pct}%)${C.reset}`);
    console.log();
  }
}

function renderJson(v, delegators, performance, epochInfo, validators) {
  const out = {
    rpc: opts.rpc,
    pubkey: v.pubkey,
    name: v.name,
    tier: v.tier,
    stake: v.stake.toString(),
    stake_aeth: Number(v.stake) / 1e9,
    stake_formatted: formatAether(v.stake),
    score: v.score,
    apy: v.apy,
    commission: v.commission,
    version: v.version,
    ip: v.ip,
    last_vote: v.lastVote,
    root_slot: v.rootSlot,
    delinquent: v.delinquent,
    activated_stake: v.activatedStake?.toString(),
    last_epoch_stake: v.lastEpochStake?.toString(),
    credits: v.credits,
    delegators: delegators ? delegators.map(d => ({
      address: d.address || d.pubkey || d.delegator,
      lamports: String(d.lamports || d.stake || 0),
      stake_aeth: Number(d.lamports || d.stake || 0) / 1e9,
      status: d.status || d.activation_status || 'unknown',
    })) : [],
    performance,
    epoch_info: epochInfo,
    fetched_at: new Date().toISOString(),
  };

  // Add rank
  if (validators && v.pubkey) {
    const sorted = [...validators].sort((a, b) => {
      const aS = typeof a.stake === 'bigint' ? Number(a.stake) : Number(a.stake || 0);
      const bS = typeof b.stake === 'bigint' ? Number(b.stake) : Number(b.stake || 0);
      return bS - aS;
    });
    const rank = sorted.findIndex(m => (m.pubkey || m.address || '') === v.pubkey);
    if (rank >= 0) out.rank = { position: rank + 1, total: sorted.length };
  }

  console.log(JSON.stringify(out, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();

  if (!opts.target) {
    console.log(`\n  ${C.red}✗ Missing validator address or name.${C.reset}`);
    console.log(`\n  ${C.cyan}Usage:${C.reset}`);
    console.log(`    ${C.cyan}aether validator info <addressOrName>${C.reset}    Inspect a validator`);
    console.log(`    ${C.cyan}aether validator info <addressOrName> --json${C.reset}   JSON output`);
    console.log(`    ${C.cyan}aether validator info --address <addr>${C.reset}       Use --address flag`);
    console.log();
    process.exit(1);
  }

  const rpc = opts.rpc;

  if (!opts.asJson) {
    console.log(`\n${C.dim}Looking up "${opts.target}" on ${rpc}...${C.reset}`);
  }

  // Fetch everything in parallel
  const [rawValidator, allValidators, epochInfo, supply] = await Promise.all([
    fetchValidatorByIdentity(rpc, opts.target),
    fetchAllValidators(rpc),
    fetchEpochInfo(rpc),
    fetchSupply(rpc),
  ]);

  if (!rawValidator) {
    if (opts.asJson) {
      console.log(JSON.stringify({ error: 'Validator not found', query: opts.target, rpc }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Validator not found:${C.reset} "${opts.target}"`);
      console.log(`  ${C.dim}Try a full address (ATH...), partial address (first 8+ chars), or name.${C.reset}`);
      console.log(`  ${C.dim}List all validators: aether validators list${C.reset}\n`);
    }
    process.exit(1);
  }

  const v = normalise(rawValidator);

  // Fetch delegators and performance in parallel
  const [delegators, performance] = await Promise.all([
    v.pubkey ? fetchDelegators(rpc, v.pubkey) : Promise.resolve([]),
    v.pubkey ? fetchValidatorPerformance(rpc, v.pubkey) : Promise.resolve(null),
  ]);

  // Estimate APY if not available
  let apyEstimated = v.apy;
  if ((apyEstimated === null || apyEstimated === undefined) && supply && !supply.error && epochInfo) {
    const totalStake = Number(supply.total_staked || supply.total || 0);
    const rewardsPerEpoch = Number(epochInfo.rewards_per_epoch || '2000000000');
    if (totalStake > 0 && rewardsPerEpoch > 0) {
      const validatorStake = Number(v.stake);
      const networkApy = (rewardsPerEpoch / totalStake) * 73;
      if (validatorStake > 0) {
        apyEstimated = networkApy;
      }
    }
  }

  const finalV = { ...v, apy: apyEstimated !== undefined ? apyEstimated : v.apy };

  if (opts.asJson) {
    renderJson(finalV, delegators, performance, epochInfo, allValidators);
    return;
  }

  renderHeader(finalV);
  renderStats(finalV, epochInfo);
  renderNodeInfo(finalV);
  renderEpochPerformance(finalV, epochInfo, performance);
  renderDelegators(delegators);
  renderRank(allValidators, finalV);

  console.log(`  ${C.dim}Fetched from: ${rpc}${C.reset}`);
  console.log();
}

main().catch(err => {
  console.error(`\n${C.red}✗ Validator info failed:${C.reset} ${err.message}\n`);
  process.exit(1);
});
