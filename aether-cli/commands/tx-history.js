#!/usr/bin/env node
/**
 * aether-cli tx-history
 *
 * Fetch and display transaction history for an Aether wallet address.
 * Shows recent transactions with type, amount, timestamp, fee, and status.
 *
 * Usage:
 *   aether tx history --address <addr> [--limit <n>] [--json] [--rpc <url>]
 *   aether history     --address <addr> [--limit <n>] [--json]
 *
 * Examples:
 *   aether tx history --address ATHxxx
 *   aether tx history --address ATHxxx --limit 50 --json
 *   aether history --address ATHxxx --rpc https://rpc.aether.io
 */

const path = require('path');

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

const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// SDK helpers - Real blockchain RPC calls via Aether SDK
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, limit: 20, json: false, rpc: null };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if ((arg === '--address' || arg === '-a') && args[i + 1] && !args[i + 1].startsWith('-')) {
      result.address = args[i + 1];
      i++;
    } else if ((arg === '--limit' || arg === '-l') && args[i + 1] && !args[i + 1].startsWith('-')) {
      result.limit = parseInt(args[i + 1], 10);
      i++;
    } else if (arg === '--json' || arg === '--json-output') {
      result.json = true;
    } else if (arg === '--rpc' && args[i + 1] && !args[i + 1].startsWith('-')) {
      result.rpc = args[i + 1];
      i++;
    } else if (arg === '--help' || arg === '-h') {
      result.help = true;
    }
  }

  return result;
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  if (lamports == null) return '—';
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatTimestamp(slot, blockTime) {
  if (!blockTime) return '—';
  try {
    const d = new Date(blockTime * 1000);
    return d.toISOString().replace('T', ' ').slice(0, 19);
  } catch {
    return String(blockTime);
  }
}

function shortAddress(addr) {
  if (!addr) return '—';
  if (addr.length <= 16) return addr;
  return addr.slice(0, 8) + '…' + addr.slice(-6);
}

function formatTxType(type) {
  const t = (type || 'unknown').toLowerCase();
  if (t.includes('transfer') || t.includes('send') || t.includes('payment')) return { label: 'TRANSFER', color: C.cyan };
  if (t.includes('stake') || t.includes('delegate')) return { label: 'STAKE', color: C.green };
  if (t.includes('unstake') || t.includes('deactivate')) return { label: 'UNSTAKE', color: C.yellow };
  if (t.includes('reward') || t.includes('mint')) return { label: 'REWARD', color: C.magenta };
  if (t.includes('vote')) return { label: 'VOTE', color: C.bright };
  if (t.includes('create') || t.includes('initialize')) return { label: 'CREATE', color: C.bright };
  if (t.includes('burn')) return { label: 'BURN', color: C.red };
  return { label: type ? type.toUpperCase().slice(0, 10) : 'TX', color: C.dim };
}

function formatStatus(status) {
  if (!status) return { label: '—', color: C.dim };
  const s = status.toLowerCase();
  if (s === 'success' || s === 'finalized' || s === 'confirmed') {
    return { label: '✓ OK', color: C.green };
  }
  if (s === 'pending' || s === 'processing') {
    return { label: '⏳', color: C.yellow };
  }
  if (s === 'failed') {
    return { label: '✗ FAIL', color: C.red };
  }
  return { label: status.slice(0, 8), color: C.dim };
}

// ---------------------------------------------------------------------------
// Parse a transaction result into a normalized display object
// ---------------------------------------------------------------------------

function parseTransaction(txResult) {
  const blockTime = txResult.blockTime;
  const slot = txResult.slot;
  const status = txResult.status || 'confirmed';

  let txType = txResult.tx_type || txResult.type || 'unknown';
  let amount = 0;
  let fee = txResult.fee || 0;
  let fromAddr = txResult.signer || null;
  let toAddr = null;
  let memo = txResult.memo || null;

  // Parse payload for details
  if (txResult.payload) {
    const payload = txResult.payload;
    if (payload.amount !== undefined) {
      amount = Number(payload.amount);
    }
    if (payload.recipient) {
      toAddr = payload.recipient;
    }
    if (payload.validator) {
      toAddr = payload.validator;
      txType = 'stake';
    }
    if (payload.stake_account) {
      fromAddr = payload.stake_account;
      txType = 'unstake';
    }
  }

  return {
    signature: txResult.signature,
    slot,
    blockTime,
    status,
    type: txType,
    amount,
    fee,
    from: fromAddr,
    to: toAddr,
    memo,
  };
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

function displayTxTable(txs) {
  // Header
  console.log(
    `\n${C.bright}${C.cyan}  ╔══════════════════════════════════════════════════════════════════════════════════════════╗${C.reset}\n` +
    `  ║${C.bright}                                   ${C.cyan}Transaction History${C.reset}${C.bright}                                              ║${C.reset}\n` +
    `  ${C.cyan}╚══════════════════════════════════════════════════════════════════════════════════════════${C.reset}`
  );

  if (txs.length === 0) {
    console.log(`\n  ${C.yellow}No transactions found for this address.${C.reset}\n`);
    return;
  }

  console.log(
    `  ${C.dim}┌────┬──────────────────────┬──────────┬───────────────┬────────────┬────────┬───────────┐${C.reset}\n` +
    `  ${C.dim}│ #  │ ${C.reset}Timestamp           ${C.dim}│ ${C.reset}Type       ${C.dim}│ ${C.reset}Amount       ${C.dim}│ ${C.reset}From         ${C.dim}│ ${C.reset}To           ${C.dim}│ ${C.reset}Status  ${C.dim}│${C.reset}`
  );

  for (let i = 0; i < txs.length; i++) {
    const tx = txs[i];
    const num = String(i + 1).padStart(3);
    const time = formatTimestamp(tx.blockTime).slice(0, 19).padEnd(19);
    const typeInfo = formatTxType(tx.type);
    const type = typeInfo.label.padEnd(10);
    const amt = tx.amount > 0 ? formatAether(tx.amount).padEnd(13) : '—'.padEnd(13);
    const from = shortAddress(tx.from || '').padEnd(11);
    const to = shortAddress(tx.to || '').padEnd(11);
    const statusInfo = formatStatus(tx.status);
    const status = statusInfo.label;

    const bgAlt = (i % 2 === 0) ? '' : C.dim;
    const reset = C.reset + bgAlt;

    console.log(
      `  ${bgAlt}${C.dim}├────┼──────────────────────┼──────────┼───────────────┼────────────┼────────┼───────────┤${reset}`
    );
    console.log(
      `  ${bgAlt}${C.dim}│${reset} ${num} ${bgAlt}${C.dim}│ ${reset}${time} ${bgAlt}${C.dim}│ ${reset}${typeInfo.color}${type}${reset} ${bgAlt}${C.dim}│ ${reset}${amt} ${bgAlt}${C.dim}│ ${reset}${from} ${bgAlt}${C.dim}│ ${reset}${to} ${bgAlt}${C.dim}│ ${reset}${statusInfo.color}${status}${reset}${bgAlt}  │${reset}`
    );
  }

  console.log(
    `  ${C.dim}└────┴──────────────────────┴──────────┴───────────────┴────────────┴────────┴───────────┘${C.reset}`
  );

  // Summary
  const totalVolume = txs.reduce((sum, tx) => sum + tx.amount, 0);
  const successCount = txs.filter(tx => tx.status !== 'failed').length;
  console.log(`\n  ${C.dim}  ${txs.length} transactions  ·  Total volume: ${C.reset}${C.bright}${formatAether(totalVolume)}${C.reset} ${C.dim}·  Success: ${C.reset}${C.green}${successCount}/${txs.length}${C.reset} ${C.dim}·  Failed: ${C.reset}${C.red}${(txs.length - successCount)}${C.reset}\n`);
}

function displayJson(txs, meta) {
  console.log(JSON.stringify({
    address: meta.address,
    rpc: meta.rpc,
    limit: meta.limit,
    transaction_count: txs.length,
    total_volume_lamports: txs.reduce((sum, tx) => sum + tx.amount, 0),
    transactions: txs.map(tx => ({
      signature: tx.signature,
      slot: tx.slot,
      timestamp: tx.blockTime ? new Date(tx.blockTime * 1000).toISOString() : null,
      type: tx.type,
      amount_lamports: tx.amount,
      amount_aeth: (tx.amount / 1e9).toFixed(9),
      fee_lamports: tx.fee,
      from: tx.from,
      to: tx.to,
      memo: tx.memo,
      status: tx.status,
    })),
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();
  // Shift args if this module was called via index.js (extra argument in argv)
  // Detect by checking if first non-option arg after 'tx' is the command name
  const args = process.argv.slice(2);
  const txIdx = args.indexOf('tx');
  const historyIdx = args.indexOf('history');
  if (txIdx !== -1 && historyIdx !== -1) {
    // Already parsed correctly above - no action needed
  }

  if (opts.help) {
    console.log(`
${C.bright}${C.cyan}tx-history${C.reset} — Fetch and display transaction history for an Aether address

${C.bright}USAGE${C.reset}
    aether tx history --address <addr> [--limit <n>] [--json] [--rpc <url>]
    aether history     --address <addr> [--limit <n>] [--json]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Aether wallet address (ATH...)
    --limit <n>          Max transactions to fetch (default: 20, max: 100)
    --json               Output raw JSON for scripting
    --rpc <url>          RPC endpoint (default: AETHER_RPC or http://127.0.0.1:8899)
    --help               Show this help

${C.bright}EXAMPLES${C.reset}
    aether tx history --address ATH3abc... --limit 50
    aether tx history --address ATH3abc... --json
    aether history --address ATH3abc... --rpc https://mainnet.aether.io
`);
    return;
  }

  if (!opts.address) {
    console.log(`  ${C.red}✗ Missing --address${C.reset}\n`);
    console.log(`  Usage: aether tx history --address <addr> [--limit <n>] [--json]\n`);
    process.exit(1);
  }

  const rpcUrl = opts.rpc || getDefaultRpc();
  const limit = Math.min(Math.max(1, opts.limit || 20), 100);

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}  Tx History${C.reset}  ·  ${C.dim}address:${C.reset} ${opts.address}  ${C.dim}·  ${C.dim}limit:${C.reset} ${limit}  ${C.dim}·  ${C.dim}rpc:${C.reset} ${rpcUrl}\n`);
  }

  try {
    // Use SDK for real blockchain RPC calls
    const client = createClient(rpcUrl);
    const history = await client.getTransactionHistory(opts.address, limit);
    
    const txs = (history.transactions || []).map(tx => parseTransaction(tx));

    if (opts.json) {
      displayJson(txs, { address: opts.address, rpc: rpcUrl, limit });
    } else {
      displayTxTable(txs);
    }

  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({
        error: err.message,
        address: opts.address,
        rpc: rpcUrl,
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ Failed to fetch transaction history:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    }
    process.exit(1);
  }
}

// Export for CLI integration
module.exports = { txHistoryCommand: main };

// Run if called directly
if (require.main === module) {
  main();
}
