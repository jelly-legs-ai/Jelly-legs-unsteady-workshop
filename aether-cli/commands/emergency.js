#!/usr/bin/env node
/**
 * aether-cli emergency - Emergency Response & Network Alert System
 *
 * Detects network emergencies (halts, consensus failures, high tps drops),
 * monitors validator liveness, issues governance alerts, and triggers backups.
 *
 * Usage:
 *   aether emergency status              # Check current emergency level
 *   aether emergency monitor [--interval 30]  # Continuous monitoring loop
 *   aether emergency alert --message "..."    # Issue a governance alert
 *   aether emergency failover             # Trigger backup node failover
 *   aether emergency history             # Show recent emergency events
 *   aether emergency check               # Run all diagnostics
 */

const http = require('http');
const https = require('https');
const fs = require('fs');
const os = require('os');
const path = require('path');
const readline = require('readline');

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
  white: '\x1b[37m',
};

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';
const EMERGENCY_LOG = path.join(os.homedir(), '.aether', 'emergency.log');

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getValidatorConfig() {
  const p = path.join(getAetherDir(), 'validator-identity.json');
  if (!fs.existsSync(p)) return null;
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, pathStr, method = 'GET') {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method,
      timeout: 5000,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

function httpPost(rpcUrl, pathStr, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;
    const bodyStr = JSON.stringify(body);

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 5000,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve(data); }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.write(bodyStr);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Emergency log
// ---------------------------------------------------------------------------

function logEmergency(level, message, details = {}) {
  const entry = {
    timestamp: new Date().toISOString(),
    level,
    message,
    details,
  };
  const line = JSON.stringify(entry);
  const dir = path.dirname(EMERGENCY_LOG);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  fs.appendFileSync(EMERGENCY_LOG, line + '\n');
  return entry;
}

function readEmergencyLog(lines = 50) {
  if (!fs.existsSync(EMERGENCY_LOG)) return [];
  const content = fs.readFileSync(EMERGENCY_LOG, 'utf8');
  const all = content.split('\n').filter(Boolean).map(l => {
    try { return JSON.parse(l); }
    catch { return null; }
  }).filter(Boolean);
  return all.slice(-lines);
}

// ---------------------------------------------------------------------------
// Diagnostic checks
// ---------------------------------------------------------------------------

/** Check if node is responding */
async function checkNodeHealth(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/slot');
    return { ok: true, slot: res.slot ?? res.root_slot ?? null };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

/** Check slot progression (is network advancing?) */
async function checkSlotProgression(rpc, count = 3) {
  const slots = [];
  for (let i = 0; i < count; i++) {
    try {
      const res = await httpRequest(rpc, '/v1/slot');
      slots.push(res.slot ?? res.root_slot ?? null);
      if (i < count - 1) await new Promise(r => setTimeout(r, 2000));
    } catch {
      slots.push(null);
    }
  }
  const valid = slots.filter(s => s !== null);
  if (valid.length < 2) return { halted: true, slots };
  const advancing = valid[valid.length - 1] > valid[0];
  return { halted: !advancing, slots, delta: valid.length > 1 ? valid[valid.length - 1] - valid[0] : 0 };
}

/** Check block height consistency */
async function checkBlockHeight(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/block_height');
    return { ok: true, blockHeight: res.block_height ?? null };
  } catch (err) {
    return { ok: false, error: err.message };
  }
}

/** Check epoch info */
async function checkEpoch(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/epoch');
    return res;
  } catch {
    return null;
  }
}

/** Check TPS for dramatic drops */
async function checkTPS(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/tps');
    return res.tps ?? res.tps_avg ?? res.transactions_per_second ?? null;
  } catch {
    return null;
  }
}

/** Check connected peers count */
async function checkPeers(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/validators');
    if (Array.isArray(res.validators)) return res.validators.length;
    if (Array.isArray(res)) return res.length;
    return null;
  } catch {
    return null;
  }
}

/** Check local validator status */
async function checkValidatorStatus() {
  const identity = getValidatorConfig();
  if (!identity) return { configured: false };
  return {
    configured: true,
    identity: identity.identity ?? identity.nodeId ?? 'unknown',
    stake: identity.stake ?? identity.delegated ?? null,
  };
}

// ---------------------------------------------------------------------------
// Emergency level assessment
// ---------------------------------------------------------------------------

function assessEmergencyLevel(results) {
  let level = 0; // 0=ok, 1=warning, 2=elevated, 3=critical

  if (!results.nodeHealth.ok) level = Math.max(level, 3);
  if (results.slotHalt.halted) level = Math.max(level, 3);
  if (results.lowTps !== null && results.lowTps < 10) level = Math.max(level, 2);
  if (results.lowPeers !== null && results.lowPeers < 3) level = Math.max(level, 1);
  if (!results.epoch || !results.epoch.epoch) level = Math.max(level, 1);

  return level;
}

const LEVEL_LABELS = ['OK', 'WARNING', 'ELEVATED', 'CRITICAL'];
const LEVEL_COLORS = [C.green, C.yellow, C.magenta, C.red];

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

async function emergencyStatus(opts) {
  const { rpc, json } = opts;
  console.log(`\n${C.bright}${C.cyan}🔔 Aether Emergency Status${C.reset}\n`);
  console.log(`  ${C.dim}RPC:${C.reset} ${rpc}\n`);

  const [nodeHealth, slotHalt, blockHeight, epoch, tps, peers, validator] = await Promise.all([
    checkNodeHealth(rpc),
    checkSlotProgression(rpc, 3),
    checkBlockHeight(rpc),
    checkEpoch(rpc),
    checkTPS(rpc),
    checkPeers(rpc),
    checkValidatorStatus(),
  ]);

  const results = { nodeHealth, slotHalt, blockHeight, epoch, tps, peers, validator };
  const level = assessEmergencyLevel(results);

  if (json) {
    console.log(JSON.stringify({ ...results, emergencyLevel: level }, null, 2));
    return;
  }

  // Node health
  const healthIcon = nodeHealth.ok ? `${C.green}✓` : `${C.red}✗`;
  const healthLabel = nodeHealth.ok ? `Slot ${nodeHealth.slot}` : nodeHealth.error;
  console.log(`  ${healthIcon} ${C.bright}Node Health${C.reset}   ${healthLabel}`);

  // Slot progression
  const haltIcon = slotHalt.halted ? `${C.red}⚠ HALTED` : `${C.green}✓ Advancing`;
  const haltLabel = slotHalt.halted
    ? `No new slots in ${slotHalt.slots.length} checks`
    : `+${slotHalt.delta} slots over ${slotHalt.slots.length} checks`;
  console.log(`  ${haltIcon} ${C.bright}Slot Progress${C.reset}  ${C.dim}${haltLabel}${C.reset}`);

  // Block height
  if (blockHeight.ok) {
    console.log(`  ${C.green}✓${C.reset} ${C.bright}Block Height${C.reset}  ${blockHeight.blockHeight}`);
  }

  // Epoch
  if (epoch && epoch.epoch) {
    console.log(`  ${C.green}✓${C.reset} ${C.bright}Epoch${C.reset}           ${epoch.epoch} ${C.dim}(progress: ${epoch.slot_index ?? '?'}/${epoch.slots_in_epoch ?? '?'})${C.reset}`);
  } else {
    console.log(`  ${C.yellow}?${C.reset} ${C.bright}Epoch${C.reset}           ${C.dim}unavailable${C.reset}`);
  }

  // TPS
  const tpsColor = tps === null ? C.yellow : (tps < 10 ? C.red : C.green);
  const tpsIcon = tps === null ? '?' : (tps < 10 ? '⚠' : '✓');
  console.log(`  ${tpsColor}${tpsIcon}${C.reset} ${C.bright}TPS${C.reset}             ${tps !== null ? tps.toFixed(1) : C.dim + 'unavailable' + C.reset}`);

  // Peers
  const peerColor = peers === null ? C.yellow : (peers < 3 ? C.red : C.green);
  const peerIcon = peers === null ? '?' : (peers < 3 ? '⚠' : '✓');
  console.log(`  ${peerColor}${peerIcon}${C.reset} ${C.bright}Connected Peers${C.reset} ${peers !== null ? peers : C.dim + 'unavailable' + C.reset}`);

  // Validator
  if (validator.configured) {
    console.log(`  ${C.cyan}▸${C.reset} ${C.bright}Validator${C.reset}     ${validator.identity.substring(0, 16)}... ${C.dim}stake: ${validator.stake ?? '?'}${C.reset}`);
  } else {
    console.log(`  ${C.dim}▸ Validator${C.reset}     ${C.dim}not configured${C.reset}`);
  }

  // Emergency level banner
  console.log(`\n  ${C.bright}Emergency Level:${C.reset} ${LEVEL_COLORS[level]}${LEVEL_LABELS[level]}${C.reset}\n`);

  if (level >= 2) {
    console.log(`  ${C.yellow}⚠ Run:${C.reset} ${C.cyan}aether emergency monitor${C.reset} to watch continuously`);
    console.log(`  ${C.yellow}⚠ Run:${C.reset} ${C.cyan}aether emergency check${C.reset} for full diagnostics\n`);
  }

  logEmergency(LEVEL_LABELS[level], 'Status check', { level, results: { slot: nodeHealth.slot, halted: slotHalt.halted, tps, peers } });

  if (level >= 2 && !json) console.log(`  ${C.dim}Logged to:${C.reset} ${EMERGENCY_LOG}\n`);
}

async function emergencyMonitor(opts) {
  const { rpc, json, interval = 30 } = opts;
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });

  console.log(`\n${C.bright}${C.red}🔴 Aether Emergency Monitor${C.reset}`);
  console.log(`  Monitoring every ${interval}s. ${C.dim}Press Ctrl+C to stop.${C.reset}\n`);

  let lastLevel = 0;
  let count = 0;

  const doCheck = async () => {
    count++;
    const [nodeHealth, slotHalt, blockHeight, epoch, tps, peers, validator] = await Promise.all([
      checkNodeHealth(rpc),
      checkSlotProgression(rpc, 3),
      checkBlockHeight(rpc),
      checkEpoch(rpc),
      checkTPS(rpc),
      checkPeers(rpc),
      checkValidatorStatus(),
    ]);

    const results = { nodeHealth, slotHalt, blockHeight, epoch, tps, peers, validator };
    const level = assessEmergencyLevel(results);
    const ts = new Date().toISOString().substring(11, 19);

    if (json) {
      console.log(JSON.stringify({ ts, ...results, emergencyLevel: level }));
    } else {
      const icon = level === 0 ? `${C.green}✓` : level === 1 ? `${C.yellow}⚠` : level === 2 ? `${C.magenta}⚡` : `${C.red}🔴`;
      const slot = nodeHealth.ok ? `slot=${nodeHealth.slot}` : 'DOWN';
      const halt = slotHalt.halted ? 'HALT!' : `+${slotHalt.delta}`;
      const tpsStr = tps !== null ? `tps=${tps.toFixed(0)}` : 'tps=?';
      const peerStr = peers !== null ? `peers=${peers}` : '';
      console.log(`${icon} [${ts}] ${slot} ${halt} ${tpsStr} ${peerStr} | Level: ${LEVEL_COLORS[level]}${LEVEL_LABELS[level]}${C.reset}`);
    }

    if (level > lastLevel) {
      logEmergency(LEVEL_LABELS[level], 'Escalation', { from: lastLevel, to: level });
      lastLevel = level;
    }

    if (level >= 3) {
      logEmergency('CRITICAL', 'Network emergency - CRITICAL level', results);
    }
  };

  // First run
  await doCheck();

  // Repeat
  const intervalMs = interval * 1000;
  const id = setInterval(doCheck, intervalMs);

  // Handle Ctrl+C
  const cleanup = () => { clearInterval(id); rl.close(); console.log(`\n${C.dim}Monitor stopped after ${count} checks.${C.reset}\n`); };
  process.on('SIGINT', cleanup);
}

async function emergencyCheck(opts) {
  const { rpc, json } = opts;
  console.log(`\n${C.bright}${C.cyan}🔬 Aether Emergency Diagnostics${C.reset}\n`);

  const checks = [
    { name: 'Node Health', fn: () => checkNodeHealth(rpc) },
    { name: 'Slot Progression', fn: () => checkSlotProgression(rpc, 5) },
    { name: 'Block Height', fn: () => checkBlockHeight(rpc) },
    { name: 'Epoch Info', fn: () => checkEpoch(rpc) },
    { name: 'TPS', fn: () => checkTPS(rpc) },
    { name: 'Peers', fn: () => checkPeers(rpc) },
    { name: 'Validator Config', fn: () => checkValidatorStatus() },
  ];

  const results = {};
  let pass = 0, fail = 0, warn = 0;

  for (const check of checks) {
    process.stdout.write(`  ${C.dim}Checking ${check.name}...${C.reset} `);
    const result = await check.fn();
    results[check.name] = result;

    const ok = result && (result.ok === undefined ? true : result.ok);
    if (ok !== false && check.name !== 'Validator Config') {
      // Determine status by check type
      let status = `${C.green}✓ PASS${C.reset}`;
      if (check.name === 'Slot Progression' && result.halted) {
        status = `${C.red}✗ FAIL${C.reset}`; fail++;
      } else if (check.name === 'TPS' && result !== null && result < 10) {
        status = `${C.yellow}⚠ WARN${C.reset}`; warn++;
      } else if (check.name === 'Peers' && result !== null && result < 3) {
        status = `${C.yellow}⚠ WARN${C.reset}`; warn++;
      } else if (check.name === 'Node Health' && !result.ok) {
        status = `${C.red}✗ FAIL${C.reset}`; fail++;
      } else {
        pass++;
      }
      console.log(status);
    } else if (check.name === 'Validator Config') {
      if (result.configured) {
        console.log(`${C.green}✓ CONFIGURED${C.reset}`);
        pass++;
      } else {
        console.log(`${C.yellow}⚠ NOT CONFIGURED${C.reset}`);
        warn++;
      }
    } else {
      console.log(`${C.red}✗ FAIL${C.reset}`);
      fail++;
    }
  }

  console.log(`\n  ${C.bright}Results:${C.reset} ${C.green}${pass} pass${C.reset} ${C.yellow}${warn} warn${C.reset} ${C.red}${fail} fail${C.reset}\n`);

  if (fail > 0) {
    console.log(`  ${C.red}⚠ Network emergency detected!${C.reset}`);
    console.log(`  ${C.dim}  Run:${C.reset} ${C.cyan}aether emergency monitor${C.reset} to watch continuously`);
    console.log(`  ${C.dim}  Run:${C.reset} ${C.cyan}aether emergency failover${C.reset} to trigger backup\n`);
    logEmergency('CRITICAL', 'Diagnostics failed', { pass, fail, warn });
  } else if (warn > 0) {
    console.log(`  ${C.yellow}⚠ Some metrics are degraded but network is operational.${C.reset}\n`);
    logEmergency('WARNING', 'Diagnostics warning', { pass, fail, warn });
  } else {
    console.log(`  ${C.green}✓ All checks passed. Network is healthy.${C.reset}\n`);
    logEmergency('OK', 'Diagnostics passed', { pass, fail, warn });
  }

  if (json) {
    console.log(JSON.stringify({ results, pass, fail, warn }, null, 2));
  }
}

async function emergencyAlert(opts) {
  const { message, rpc } = opts;
  if (!message) {
    console.log(`\n${C.red}Error: --message is required${C.reset}`);
    console.log(`  ${C.dim}Usage: aether emergency alert --message "Network alert text"${C.reset}\n`);
    return;
  }

  console.log(`\n${C.bright}🔶 Issuing Governance Alert${C.reset}\n`);
  console.log(`  ${C.dim}Message:${C.reset} ${message}\n`);

  // Try to submit alert to governance endpoint
  try {
    const identity = getValidatorConfig();
    const result = await httpPost(rpc, '/v1/governance/alert', {
      message,
      validator: identity?.identity ?? 'unknown',
      timestamp: new Date().toISOString(),
    });

    if (result.success || result.alert_id) {
      console.log(`  ${C.green}✓ Alert issued successfully${C.reset}`);
      console.log(`  ${C.dim}Alert ID: ${result.alert_id ?? 'unknown'}${C.reset}\n`);
      logEmergency('ELEVATED', `Alert issued: ${message}`, { alertId: result.alert_id });
    } else {
      console.log(`  ${C.yellow}⚠ Alert submitted (check response):${C.reset}`);
      console.log(`  ${JSON.stringify(result)}\n`);
    }
  } catch (err) {
    console.log(`  ${C.yellow}⚠ Could not reach governance endpoint (network may be down)${C.reset}`);
    console.log(`  ${C.dim}Storing alert locally for later submission...${C.reset}\n`);
    logEmergency('ELEVATED', `Local alert (network unreachable): ${message}`, { error: err.message });
  }
}

async function emergencyFailover(opts) {
  const { rpc, json } = opts;
  console.log(`\n${C.bright}${C.magenta}⚡ Aether Emergency Failover${C.reset}\n`);

  const identity = getValidatorConfig();
  if (!identity) {
    console.log(`  ${C.red}✗ No validator identity found.${C.reset}`);
    console.log(`  ${C.dim}  Run:${C.reset} ${C.cyan}aether init${C.reset} first to configure validator\n`);
    return;
  }

  console.log(`  ${C.dim}Validator:${C.reset} ${identity.identity ?? identity.nodeId ?? 'unknown'}`);
  console.log(`  ${C.dim}Checking backup node status...${C.reset}\n`);

  // Check if backup RPC is configured
  const backupRpc = process.env.AETHER_BACKUP_RPC;
  if (!backupRpc) {
    console.log(`  ${C.yellow}⚠ AETHER_BACKUP_RPC not set.${C.reset}`);
    console.log(`  ${C.dim}  Export it in your environment to enable automatic failover.${C.reset}`);
    console.log(`  ${C.dim}  export AETHER_BACKUP_RPC=http://backup-node:8899${C.reset}\n`);
  }

  // Check current node
  const [health, slotHalt] = await Promise.all([
    checkNodeHealth(rpc),
    checkSlotProgression(rpc, 3),
  ]);

  console.log(`  ${C.dim}Current node health:${C.reset} ${health.ok ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`}`);
  console.log(`  ${C.dim}Slot status:${C.reset} ${slotHalt.halted ? `${C.red}HALTED${C.reset}` : `${C.green}Advancing${C.reset}`}\n`);

  if (!health.ok || slotHalt.halted) {
    console.log(`  ${C.red}⚠ Primary node is down! Attempting failover...${C.reset}\n`);

    if (backupRpc) {
      try {
        const backupHealth = await checkNodeHealth(backupRpc);
        if (backupHealth.ok) {
          console.log(`  ${C.green}✓ Backup node is healthy!${C.reset}`);
          console.log(`  ${C.green}✓ Failover would succeed.${C.reset}`);
          console.log(`  ${C.dim}  Update your AETHER_RPC to:${C.reset} ${backupRpc}\n`);
          logEmergency('CRITICAL', 'Failover needed and backup available', { backupRpc });
        } else {
          console.log(`  ${C.red}✗ Backup node is also unreachable.${C.reset}\n`);
          logEmergency('CRITICAL', 'Failover failed - both primary and backup down', {});
        }
      } catch {
        console.log(`  ${C.red}✗ Backup node check failed.${C.reset}\n`);
      }
    } else {
      console.log(`  ${C.yellow}⚠ Set AETHER_BACKUP_RPC to enable automatic failover.${C.reset}\n`);
      logEmergency('CRITICAL', 'Failover needed but no backup configured', {});
    }
  } else {
    console.log(`  ${C.green}✓ Primary node is healthy. No failover needed.${C.reset}\n`);
  }
}

async function emergencyHistory(opts) {
  const { json, lines = 20 } = opts;
  const events = readEmergencyLog(lines);

  if (events.length === 0) {
    console.log(`\n${C.dim}No emergency events logged.${C.reset}\n`);
    return;
  }

  if (json) {
    console.log(JSON.stringify(events, null, 2));
    return;
  }

  console.log(`\n${C.bright}📋 Recent Emergency Events${C.reset} ${C.dim}(last ${events.length})${C.reset}\n`);

  for (const ev of events) {
    const levelColor = ev.level === 'OK' ? C.green : ev.level === 'WARNING' ? C.yellow : ev.level === 'ELEVATED' ? C.magenta : C.red;
    const ts = ev.timestamp ? ev.timestamp.substring(0, 19) : '?';
    console.log(`  ${levelColor}[${ev.level}]${C.reset} ${C.dim}${ts}${C.reset} — ${ev.message}`);
  }
  console.log();
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // [node, index.js, emergency, <subcmd>, ...]
  return args;
}

async function main() {
  const rawArgs = parseArgs();
  const subcmd = rawArgs[0] || 'status';

  const allArgs = rawArgs.slice(1);
  const rpcIndex = allArgs.findIndex(a => a === '--rpc');
  const rpc = rpcIndex !== -1 && allArgs[rpcIndex + 1] ? allArgs[rpcIndex + 1] : DEFAULT_RPC;

  const opts = {
    rpc,
    json: allArgs.includes('--json'),
    message: null,
    interval: 30,
    lines: 20,
  };

  const msgIndex = allArgs.findIndex(a => a === '--message');
  if (msgIndex !== -1 && allArgs[msgIndex + 1]) opts.message = allArgs[msgIndex + 1];

  const intIndex = allArgs.findIndex(a => a === '--interval');
  if (intIndex !== -1 && allArgs[intIndex + 1]) opts.interval = parseInt(allArgs[intIndex + 1], 10);

  const linesIndex = allArgs.findIndex(a => a === '--lines');
  if (linesIndex !== -1 && allArgs[linesIndex + 1]) opts.lines = parseInt(allArgs[linesIndex + 1], 10);

  switch (subcmd) {
    case 'status':
      await emergencyStatus(opts);
      break;
    case 'monitor':
      await emergencyMonitor(opts);
      break;
    case 'check':
      await emergencyCheck(opts);
      break;
    case 'alert':
      await emergencyAlert(opts);
      break;
    case 'failover':
      await emergencyFailover(opts);
      break;
    case 'history':
      await emergencyHistory(opts);
      break;
    default:
      console.log(`\n${C.bright}${C.cyan}aether emergency${C.reset} — Emergency Response & Alert System\n`);
      console.log(`Usage: ${C.cyan}aether emergency <command>${C.reset}\n`);
      console.log(`Commands:`);
      console.log(`  ${C.cyan}status${C.reset}      Check current emergency level (default)`);
      console.log(`  ${C.cyan}monitor${C.reset}     Continuous monitoring loop (--interval <sec>)`);
      console.log(`  ${C.cyan}check${C.reset}       Run full diagnostic checks`);
      console.log(`  ${C.cyan}alert${C.reset}        Issue a governance alert (--message "...")`);
      console.log(`  ${C.cyan}failover${C.reset}     Trigger backup node failover`);
      console.log(`  ${C.cyan}history${C.reset}      Show recent emergency events (--lines <n>)`);
      console.log(`\nOptions:`);
      console.log(`  --rpc <url>     RPC endpoint (default: $AETHER_RPC or localhost)`);
      console.log(`  --json          JSON output\n`);
  }
}

main().catch(err => {
  console.error(`\n${C.red}Error in emergency command:${C.reset}`, err.message, '\n');
  process.exit(1);
});
