#!/usr/bin/env node
/**
 * aether-cli stats
 *
 * Comprehensive wallet stats dashboard:
 *   - Token balance (AETH + lamports)
 *   - Active stake positions (validator, amount, status)
 *   - Recent transactions (last 5)
 *   - Estimated rewards accrued
 *
 * Usage:
 *   aether stats --address <addr>         Full stats dashboard
 *   aether stats --address <addr> --json  JSON output for scripting
 *   aether stats --address <addr> --compact  One-line summary
 *
 * Requires AETHER_RPC env var or local node (default: http://127.0.0.1:8899)
 */

const http = require('http');
const https = require('https');
const os = require('os');
const path = require('path');
const fs = require('fs');
const bs58 = require('bs58').default;

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
  bold: '\x1b[1m',
};

// ---------------------------------------------------------------------------
// Paths & config
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function getWalletsDir() {
  return path.join(getAetherDir(), 'wallets');
}

function loadConfig() {
  const p = getConfigPath();
  if (!fs.existsSync(p)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fp = path.join(getWalletsDir(), `${address}.json`);
  if (!fs.existsSync(fp)) return null;
  return JSON.parse(fs.readFileSync(fp, 'utf8'));
}

// ---------------------------------------------------------------------------
// RPC helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

function httpRequest(rpcUrl, pathStr) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
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
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });
    req.on('error', reject);
    req.end();
  });
}

function httpPost(rpcUrl, pathStr, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 8000,
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
    req.write(bodyStr);
    req.end();
  });
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/**
 * Format lamports as AETH string (1 AETH = 1e9 lamports)
 */
function formatAether(lamports) {
  if (lamports === undefined || lamports === null) return '0 AETH';
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

/**
 * Format a timestamp as relative time ("2h ago", "3d ago")
 */
function relativeTime(timestamp) {
  if (!timestamp) return 'unknown';
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

/**
 * Truncate a signature or hash for display
 */
function truncate(str, chars = 8) {
  if (!str || typeof str !== 'string') return '—';
  if (str.length <= chars * 2 + 3) return str;
  return str.slice(0, chars) + '…' + str.slice(-chars);
}

// ---------------------------------------------------------------------------
// Fetch wallet stats from RPC
// ---------------------------------------------------------------------------

async function fetchWalletStats(address, rpcUrl) {
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  // Fetch account, transactions, and stake positions in parallel
  const [account, txs, stakeAccounts] = await Promise.all([
    httpRequest(rpcUrl, `/v1/account/${rawAddr}`).catch(() => null),
    httpRequest(rpcUrl, `/v1/tx?address=${encodeURIComponent(rawAddr)}&limit=5`).catch(() => null),
    httpRequest(rpcUrl, `/v1/stake?address=${encodeURIComponent(rawAddr)}`).catch(() => null),
  ]);

  return { account, txs, stakeAccounts };
}

// ---------------------------------------------------------------------------
// Render dashboard
// ---------------------------------------------------------------------------

function renderDashboard(address, stats, opts) {
  const { account, txs, stakeAccounts } = stats;
  const { compact, asJson } = opts;
  const rpcUrl = opts.rpcUrl;

  if (asJson) {
    const stakeList = stakeAccounts && !stakeAccounts.error
      ? (Array.isArray(stakeAccounts) ? stakeAccounts : stakeAccounts.accounts || stakeAccounts.stake_accounts || [])
      : [];
    const txList = txs && !txs.error
      ? (Array.isArray(txs) ? txs : txs.transactions || [])
      : [];

    const out = {
      address,
      rpc: rpcUrl,
      balance: account && !account.error ? {
        lamports: account.lamports || 0,
        aeth: formatAether(account.lamports || 0),
      } : null,
      stake_positions: stakeList.map((sa) => ({
        stake_account: sa.stake_account || sa.address || 'unknown',
        validator: sa.validator || 'unknown',
        amount: sa.amount || 0,
        aeth: formatAether(sa.amount || 0),
        status: sa.status || 'active',
        created_epoch: sa.created_epoch || null,
      })),
      recent_txs: txList.map((tx) => ({
        type: tx.tx_type || tx.type || 'Unknown',
        signature: tx.signature || tx.id || tx.tx_signature || null,
        timestamp: tx.timestamp || null,
        relative_time: relativeTime(tx.timestamp),
        payload: tx.payload?.data || {},
        fee: tx.fee || 0,
      })),
      fetched_at: new Date().toISOString(),
    };
    console.log(JSON.stringify(out, null, 2));
    return;
  }

  const lamports = (account && !account.error) ? (account.lamports || 0) : null;
  const stakeList = stakeAccounts && !stakeAccounts.error
    ? (Array.isArray(stakeAccounts) ? stakeAccounts : stakeAccounts.accounts || stakeAccounts.stake_accounts || [])
    : [];
  const txList = txs && !txs.error
    ? (Array.isArray(txs) ? txs : txs.transactions || [])
    : [];

  if (compact) {
    // One-line summary
    const bal = lamports !== null ? formatAether(lamports) : 'unknown';
    const stakes = stakeList.length;
    const recent = txList.length > 0 ? (txList[0].tx_type || txList[0].type || '?') : 'none';
    console.log(`${C.bright}${address}${C.reset}  bal:${C.green}${bal}${C.reset}  stakes:${stakes}  last:${recent}  txs:${txList.length}`);
    return;
  }

  // Full dashboard
  console.log(`\n${C.bright}${C.cyan}── Wallet Stats ─────────────────────────────────────────${C.reset}`);
  console.log(`  ${C.green}★${C.reset} ${C.bright}${address}${C.reset}`);
  console.log(`  ${C.dim}RPC: ${rpcUrl}${C.reset}`);
  console.log();

  // Balance section
  console.log(`  ${C.bright}Balance${C.reset}`);
  if (lamports !== null) {
    console.log(`    ${C.green}${formatAether(lamports)}${C.reset}  ${C.dim}(${lamports} lamports)${C.reset}`);
    if (account.owner) {
      const ownerStr = Array.isArray(account.owner)
        ? 'ATH' + bs58.encode(Buffer.from(account.owner.slice(0, 32)))
        : account.owner;
      console.log(`    ${C.dim}Owner: ${ownerStr}${C.reset}`);
    }
    if (account.rent_epoch !== undefined) {
      console.log(`    ${C.dim}Rent epoch: ${account.rent_epoch}${C.reset}`);
    }
  } else {
    console.log(`    ${C.yellow}⚠ Could not fetch balance (account may not exist)${C.reset}`);
  }
  console.log();

  // Stake positions section
  console.log(`  ${C.bright}Stake Positions (${stakeList.length})${C.reset}`);
  if (stakeList.length === 0) {
    console.log(`    ${C.dim}No active stake positions.${C.reset}`);
  } else {
    const statusColors = {
      active: C.green,
      inactive: C.yellow,
      activating: C.yellow,
      deactivating: C.red,
      unknown: C.dim,
    };
    for (const sa of stakeList) {
      const status = sa.status || 'unknown';
      const color = statusColors[status] || C.dim;
      const amount = sa.amount ? formatAether(sa.amount) : '0 AETH';
      const validator = sa.validator || 'unknown';
      console.log(`    ${C.dim}┌─${C.reset}`);
      console.log(`    │  ${C.bright}Validator:${C.reset} ${validator}`);
      console.log(`    │  ${C.bright}Amount:${C.reset}     ${color}${amount}${C.reset}`);
      console.log(`    │  ${C.bright}Status:${C.reset}      ${color}${status}${C.reset}`);
      if (sa.stake_account) {
        console.log(`    │  ${C.bright}Stake acct:${C.reset} ${truncate(sa.stake_account)}`);
      }
      console.log(`    ${C.dim}└${C.reset}`);
    }
  }
  console.log();

  // Recent transactions section
  console.log(`  ${C.bright}Recent Transactions (${txList.length})${C.reset}`);
  if (txList.length === 0) {
    console.log(`    ${C.dim}No transactions yet.${C.reset}`);
  } else {
    const typeColors = {
      Transfer: C.cyan,
      Stake: C.green,
      Unstake: C.yellow,
      ClaimRewards: C.magenta,
      CreateNFT: C.red,
      MintNFT: C.red,
      TransferNFT: C.cyan,
      UpdateMetadata: C.yellow,
      Unknown: C.dim,
    };
    for (const tx of txList) {
      const txType = tx.tx_type || tx.type || 'Unknown';
      const color = typeColors[txType] || C.dim;
      const sig = tx.signature || tx.id || tx.tx_signature || '—';
      const ts = tx.timestamp ? relativeTime(tx.timestamp) : 'unknown';

      console.log(`    ${C.dim}┌─ ${ts}${C.reset}  ${C.bright}${color}${txType}${C.reset}  sig:${truncate(sig)}`);
      if (tx.payload && tx.payload.data) {
        const d = tx.payload.data;
        if (d.recipient) console.log(`    │  ${C.dim}→ to:      ${d.recipient}${C.reset}`);
        if (d.amount)    console.log(`    │  ${C.dim}amount:   ${formatAether(d.amount)}${C.reset}`);
        if (d.validator) console.log(`    │  ${C.dim}validator: ${d.validator}${C.reset}`);
      }
      if (tx.fee !== undefined && tx.fee > 0) {
        console.log(`    │  ${C.dim}fee: ${tx.fee} lamports${C.reset}`);
      }
      console.log(`    ${C.dim}└${C.reset}`);
    }
  }
  console.log();
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

function statsCommand() {
  const args = process.argv.slice(3);

  // Parse flags
  let address = null;
  let compact = false;
  let asJson = false;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      address = args[i + 1];
      i++;
    } else if (args[i] === '--compact' || args[i] === '-c') {
      compact = true;
    } else if (args[i] === '--json' || args[i] === '-j') {
      asJson = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      console.log(`
${C.bright}Aether Wallet Stats${C.reset}
${C.dim}Comprehensive wallet overview: balance, stakes, recent transactions.${C.reset}

${C.bright}Usage:${C.reset}
  aether stats --address <addr>         Full dashboard
  aether stats --address <addr> --json  JSON output
  aether stats --address <addr> --compact  One-line summary

${C.bright}Options:${C.reset}
  -a, --address <addr>   Wallet address (or set default)
  -j, --json             JSON output
  -c, --compact          One-line summary
  -h, --help             Show this help
`);
      return;
    }
  }

  // Resolve address
  if (!address) {
    const cfg = loadConfig();
    address = cfg.defaultWallet;
  }

  if (!address) {
    console.log(`  ${C.red}✗ No wallet address.${C.reset} Use ${C.cyan}--address <addr>${C.reset} or set a default.`);
    console.log(`  ${C.dim}Usage: aether stats --address <address> [--compact] [--json]${C.reset}\n`);
    process.exit(1);
  }

  // Verify wallet file exists
  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`  ${C.red}✗ Wallet not found locally:${C.reset} ${address}`);
    console.log(`  ${C.dim}Check your wallets: ${C.cyan}aether wallet list${C.reset}\n`);
    process.exit(1);
  }

  const rpcUrl = getDefaultRpc();

  if (!asJson) {
    console.log(`  ${C.dim}Fetching stats from ${rpcUrl}...${C.reset}`);
  }

  // Fetch and render
  (async () => {
    try {
      const stats = await fetchWalletStats(address, rpcUrl);
      renderDashboard(address, stats, { compact, asJson, rpcUrl });
    } catch (err) {
      console.log(`  ${C.red}✗ Failed to fetch wallet stats:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
      process.exit(1);
    }
  })();
}

module.exports = { statsCommand };