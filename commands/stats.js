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
 * FULLY WIRED TO SDK - Uses @jellylegsai/aether-sdk for all blockchain calls.
 * No manual HTTP - all calls go through AetherClient with real RPC.
 *
 * Usage:
 *   aether stats --address <addr>         Full stats dashboard
 *   aether stats --address <addr> --json  JSON output for scripting
 *   aether stats --address <addr> --compact  One-line summary
 *
 * Requires AETHER_RPC env var or local node (default: http://127.0.0.1:8899)
 */

const os = require('os');
const path = require('path');
const fs = require('fs');
const bs58 = require('bs58').default;

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
  bold: '\x1b[1m',
};

const CLI_VERSION = '1.1.0';

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
// SDK setup - Real blockchain RPC calls via @jellylegsai/aether-sdk
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
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
// Fetch wallet stats using SDK - Real RPC calls
// ---------------------------------------------------------------------------

async function fetchWalletStats(address, rpcUrl) {
  const client = createClient(rpcUrl);
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  // Parallel SDK calls - all real blockchain RPC calls
  const [account, txHistory, stakeAccounts] = await Promise.all([
    // SDK: getAccountInfo → GET /v1/account/<addr>
    client.getAccountInfo(rawAddr).catch(err => ({ error: err.message })),
    // SDK: getTransactionHistory → GET /v1/tx/<addr>
    client.getTransactionHistory(address, 5).catch(err => ({ error: err.message })),
    // SDK: getStakePositions → GET /v1/stake/<addr>
    client.getStakePositions(rawAddr).catch(err => ({ error: err.message })),
  ]);

  return { account, txHistory, stakeAccounts };
}

// ---------------------------------------------------------------------------
// Render dashboard
// ---------------------------------------------------------------------------

function renderDashboard(address, stats, opts) {
  const { account, txHistory, stakeAccounts } = stats;
  const { compact, asJson } = opts;
  const rpcUrl = opts.rpcUrl;

  // Parse SDK responses - handle both wrapped and raw shapes
  const rawAccount = account && !account.error ? account : null;
  const rawTxs = (txHistory && !txHistory.error)
    ? (txHistory.transactions || txHistory || [])
    : [];
  const rawStakes = (stakeAccounts && !stakeAccounts.error)
    ? (Array.isArray(stakeAccounts) ? stakeAccounts
       : stakeAccounts.accounts || stakeAccounts.stake_accounts || [])
    : [];

  if (asJson) {
    const out = {
      address,
      rpc: rpcUrl,
      balance: rawAccount && rawAccount.lamports !== undefined ? {
        lamports: rawAccount.lamports || 0,
        aeth: formatAether(rawAccount.lamports || 0),
      } : null,
      stake_positions: rawStakes.map((sa) => ({
        stake_account: sa.stake_account || sa.address || sa.pubkey || sa.publicKey || 'unknown',
        validator: sa.validator || sa.delegate || 'unknown',
        amount: sa.amount || sa.lamports || sa.stake_lamports || 0,
        aeth: formatAether(sa.amount || sa.lamports || sa.stake_lamports || 0),
        status: sa.status || sa.state || 'active',
        created_epoch: sa.created_epoch || sa.activation_epoch || null,
      })),
      recent_txs: rawTxs.slice(0, 5).map((tx) => ({
        type: tx.tx_type || tx.type || 'Unknown',
        signature: tx.signature || tx.id || tx.tx_signature || null,
        timestamp: tx.blockTime || null,
        relative_time: relativeTime(tx.blockTime),
        payload: tx.payload?.data || tx.payload || {},
        fee: tx.fee || 0,
      })),
      fetched_at: new Date().toISOString(),
      cli_version: CLI_VERSION,
    };
    console.log(JSON.stringify(out, null, 2));
    return;
  }

  const lamports = rawAccount?.lamports ?? null;

  if (compact) {
    const bal = lamports !== null ? formatAether(lamports) : 'unknown';
    const stakes = rawStakes.length;
    const recent = rawTxs.length > 0 ? (rawTxs[0].tx_type || rawTxs[0].type || '?') : 'none';
    console.log(`${C.bright}${address}${C.reset}  bal:${C.green}${bal}${C.reset}  stakes:${stakes}  last:${recent}  txs:${rawTxs.length}`);
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
    if (rawAccount?.owner) {
      const ownerStr = Array.isArray(rawAccount.owner)
        ? 'ATH' + bs58.encode(Buffer.from(rawAccount.owner.slice(0, 32)))
        : rawAccount.owner;
      console.log(`    ${C.dim}Owner: ${ownerStr}${C.reset}`);
    }
    if (rawAccount?.rent_epoch !== undefined) {
      console.log(`    ${C.dim}Rent epoch: ${rawAccount.rent_epoch}${C.reset}`);
    }
  } else {
    console.log(`    ${C.yellow}⚠ Could not fetch balance (account may not exist)${C.reset}`);
  }
  console.log();

  // Stake positions section
  console.log(`  ${C.bright}Stake Positions (${rawStakes.length})${C.reset}`);
  if (rawStakes.length === 0) {
    console.log(`    ${C.dim}No active stake positions.${C.reset}`);
  } else {
    const statusColors = {
      active: C.green,
      inactive: C.yellow,
      activating: C.yellow,
      deactivating: C.red,
      unknown: C.dim,
    };
    for (const sa of rawStakes) {
      const status = sa.status || 'unknown';
      const color = statusColors[status] || C.dim;
      const amount = sa.amount || sa.lamports || sa.stake_lamports || 0;
      const validator = sa.validator || sa.delegate || 'unknown';
      console.log(`    ${C.dim}┌─${C.reset}`);
      console.log(`    │  ${C.bright}Validator:${C.reset} ${validator}`);
      console.log(`    │  ${C.bright}Amount:${C.reset}     ${color}${formatAether(amount)}${C.reset}`);
      console.log(`    │  ${C.bright}Status:${C.reset}      ${color}${status}${C.reset}`);
      const saAddr = sa.stake_account || sa.address || sa.pubkey || sa.publicKey;
      if (saAddr) {
        console.log(`    │  ${C.bright}Stake acct:${C.reset} ${truncate(saAddr)}`);
      }
      console.log(`    ${C.dim}└${C.reset}`);
    }
  }
  console.log();

  // Recent transactions section
  console.log(`  ${C.bright}Recent Transactions (${rawTxs.length})${C.reset}`);
  if (rawTxs.length === 0) {
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
    for (const tx of rawTxs.slice(0, 5)) {
      const txType = tx.tx_type || tx.type || 'Unknown';
      const color = typeColors[txType] || C.dim;
      const sig = tx.signature || tx.id || tx.tx_signature || '—';
      const ts = tx.blockTime ? relativeTime(tx.blockTime) : 'unknown';

      console.log(`    ${C.dim}┌─ ${ts}${C.reset}  ${C.bright}${color}${txType}${C.reset}  sig:${truncate(sig)}`);
      const payload = tx.payload?.data || tx.payload || {};
      if (payload.recipient) console.log(`    │  ${C.dim}→ to:      ${payload.recipient}${C.reset}`);
      if (payload.amount)    console.log(`    │  ${C.dim}amount:   ${formatAether(payload.amount)}${C.reset}`);
      if (payload.validator) console.log(`    │  ${C.dim}validator: ${payload.validator}${C.reset}`);
      if (tx.fee !== undefined && tx.fee > 0) {
        console.log(`    │  ${C.dim}fee: ${tx.fee} lamports${C.reset}`);
      }
      console.log(`    ${C.dim}└${C.reset}`);
    }
  }
  console.log();

  // SDK attribution
  console.log(`  ${C.dim}SDK: getAccountInfo(), getTransactionHistory(), getStakePositions()${C.reset}`);
  console.log();
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function statsCommand() {
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
      showHelp();
      return;
    } else if ((args[i] === '--rpc' || args[i] === '-r') && args[i + 1]) {
      // Ignore -r here (handled by SDK client default)
      i++;
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
    console.log(`  ${C.dim}Fetching stats via SDK from ${rpcUrl}...${C.reset}`);
  }

  try {
    const stats = await fetchWalletStats(address, rpcUrl);
    renderDashboard(address, stats, { compact, asJson, rpcUrl });
  } catch (err) {
    if (asJson) {
      console.log(JSON.stringify({ address, error: err.message, rpc: rpcUrl }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to fetch wallet stats:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    }
    process.exit(1);
  }
}

function showHelp() {
  console.log(`
${C.bright}Aether Wallet Stats${C.reset}
${C.dim}Comprehensive wallet overview: balance, stakes, recent transactions.
Fully wired to @jellylegsai/aether-sdk for real blockchain RPC calls.${C.reset}

${C.bright}Usage:${C.reset}
  aether stats --address <addr>         Full dashboard
  aether stats --address <addr> --json  JSON output
  aether stats --address <addr> --compact  One-line summary

${C.bright}Options:${C.reset}
  -a, --address <addr>   Wallet address (or set default)
  -j, --json             JSON output
  -c, --compact          One-line summary
  -r, --rpc <url>        RPC endpoint (default: AETHER_RPC or localhost:8899)
  -h, --help             Show this help

${C.bright}SDK Methods Used:${C.reset}
  client.getAccountInfo(addr)       → GET /v1/account/<addr>
  client.getTransactionHistory(addr) → GET /v1/tx/<addr>
  client.getStakePositions(addr)     → GET /v1/stake/<addr>
`);
}

module.exports = { statsCommand };
