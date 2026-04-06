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

const https = require('https');
const http = require('http');

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

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
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
    req.on('timeout', () => { req.destroy(); reject(new Error(`Request timeout after ${timeoutMs}ms`)); });
    req.end();
  });
}

function httpPost(rpcUrl, path, body, timeoutMs = 15000) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: timeoutMs,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(bodyStr) },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error(`Request timeout after ${timeoutMs}ms`)); });
    req.write(bodyStr);
    req.end();
  });
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
// Core RPC calls
// ---------------------------------------------------------------------------

/**
 * Fetch confirmed transaction signatures for an address using getSignaturesForAddress.
 */
async function fetchTxSignatures(rpcUrl, address, limit) {
  const body = {
    jsonrpc: '2.0',
    id: 1,
    method: 'getSignaturesForAddress',
    params: [
      address,
      { limit },
    ],
  };
  return httpPost(rpcUrl, '/', body);
}

/**
 * Fetch a specific confirmed transaction by signature.
 */
async function fetchTx(rpcUrl, signature) {
  const body = {
    jsonrpc: '2.0',
    id: 1,
    method: 'getTransaction',
    params: [
      signature,
      { encoding: 'jsonParsed', maxSupportedTransactionVersion: 0 },
    ],
  };
  return httpPost(rpcUrl, '/', body);
}

/**
 * Parse a transaction result into a normalized display object.
 */
function parseTransaction(txResult, sigInfo) {
  const blockTime = sigInfo.blockTime || txResult.blockTime;
  const slot = sigInfo.slot;
  const status = sigInfo.err ? 'failed' : (sigInfo.confirmationStatus || 'confirmed');

  let txType = 'unknown';
  let amount = 0;
  let fee = txResult.meta?.fee || 0;
  let fromAddr = null;
  let toAddr = null;
  let memo = null;

  try {
    const msg = txResult.transaction?.message;
    if (msg) {
      // Parse instructions for transfer/stake types
      const instructions = msg.instructions || [];
      for (const ix of instructions) {
        const programId = ix.programId || (ix.parsed && ix.parsed.info && ix.parsed.type);
        // Native transfer
        if (ix.parsed && ix.parsed.type === 'transfer') {
          txType = 'transfer';
          const info = ix.parsed.info;
          fromAddr = info.source || info.from;
          toAddr = info.destination || info.to;
          amount = info.lamports || info.amount || 0;
        } else if (ix.parsed && ix.parsed.type === 'stake') {
          txType = 'stake';
          const info = ix.parsed.info;
          fromAddr = info.from || info.funder;
          toAddr = info.validator;
          amount = info.lamports || info.amount || 0;
        } else if (ix.parsed && ix.parsed.type === 'withdrawStake') {
          txType = 'unstake';
          const info = ix.parsed.info;
          toAddr = info.destination || info.withdrawer;
          amount = info.lamports || info.amount || 0;
        } else if (ix.parsed && ix.parsed.type === 'vote') {
          txType = 'vote';
        } else if (ix.parsed && ix.parsed.type === 'initialize') {
          txType = 'initialize';
        } else if (ix.parsed && ix.parsed.type === 'createAccount') {
          txType = 'create';
        } else if (ix.parsed && ix.parsed.type === 'approve') {
          txType = 'stake';
          const info = ix.parsed.info || {};
          fromAddr = info.from || info.owner;
          toAddr = info.stake;
          amount = info.amount || info.lamports || 0;
        } else if (ix.parsed && ix.parsed.type === 'delegate') {
          txType = 'stake';
          const info = ix.parsed.info || {};
          fromAddr = info.stake || info.from;
          toAddr = info.validator;
          amount = info.lamports || 0;
        } else if (ix.parsed && ix.parsed.type === 'withdraw') {
          txType = 'unstake';
          const info = ix.parsed.info || {};
          toAddr = info.destination;
          amount = info.lamports || 0;
        }
        // Check memo
        if (ix.memo) memo = ix.memo;
      }

      // Fallback: try legacy instructions if no parsed instructions
      if (!instructions.length || instructions.every(ix => !ix.parsed)) {
        for (const ix of instructions) {
          if (ix.data === 'AAAA' || ix.data === '2ugJ4ELK3wW9qNXH' || !ix.data) {
            txType = 'transfer';
          }
        }
      }

      // Compute fee
      if (txResult.meta) {
        fee = txResult.meta.fee || 0;
        if (txResult.meta.postBalances && txResult.meta.preBalances) {
          // Try to detect native transfer from balance changes
          for (let i = 0; i < txResult.meta.postBalances.length; i++) {
            const diff = txResult.meta.postBalances[i] - txResult.meta.preBalances[i];
            if (diff < 0) {
              amount = Math.abs(diff);
              if (!fromAddr) fromAddr = msg.accountKeys?.[i];
            } else if (diff > 0 && amount === 0) {
              if (!toAddr) toAddr = msg.accountKeys?.[i];
            }
          }
        }
      }
    }
  } catch (e) {
    // Parsing failed — use defaults
  }

  return {
    signature: sigInfo.signature || sigInfo.signatures?.[0],
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
    // Step 1: Get transaction signatures
    const sigsResult = await fetchTxSignatures(rpcUrl, opts.address, limit);

    if (sigsResult.error) {
      throw new Error(sigsResult.error.message || JSON.stringify(sigsResult.error));
    }

    const signatures = Array.isArray(sigsResult.result) ? sigsResult.result : [];

    if (signatures.length === 0) {
      if (!opts.json) {
        displayTxTable([]);
      } else {
        displayJson([], { address: opts.address, rpc: rpcUrl, limit });
      }
      return;
    }

    // Step 2: Fetch each transaction in parallel (up to 10 at a time)
    const txResults = [];
    const BATCH = 10;

    for (let i = 0; i < signatures.length; i += BATCH) {
      const batch = signatures.slice(i, i + BATCH);
      const batchPromises = batch.map(sig => fetchTx(rpcUrl, sig.signature).catch(err => ({ error: err.message })));
      const batchResults = await Promise.all(batchPromises);
      txResults.push(...batchResults);
    }

    // Step 3: Parse and normalize
    const txs = txResults
      .map((res, idx) => {
        if (res.error) return null;
        try {
          return parseTransaction(res.result || {}, signatures[idx] || {});
        } catch {
          return null;
        }
      })
      .filter(Boolean);

    if (opts.json) {
      displayJson(txs, { address: opts.address, rpc: rpcUrl, limit });
    } else {
      displayTxTable(txs);
    }

  } catch (err) {
    console.log(`\n  ${C.red}✗ Failed to fetch transaction history:${C.reset} ${err.message}\n`);
    if (err.stack && !opts.json) {
      console.log(`  ${C.dim}${err.stack.split('\n').slice(0, 3).join('\n  ')}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}
