#!/usr/bin/env node
/**
 * aether-cli tx
 *
 * Look up a specific transaction by its signature and show confirmation status,
 * raw JSON data, fee paid, slot, block time, and logs.
 * This is the go-to command after submitting any transaction to check if it landed.
 *
 * FULLY WIRED TO SDK — Uses @jellylegsai/aether-sdk for real HTTP RPC calls.
 * No stubs, no mocks. Every call hits the actual blockchain RPC endpoint.
 *
 * Usage:
 *   aether tx <signature>                    Show human-readable transaction details
 *   aether tx <signature> --json             Raw JSON output for scripting
 *   aether tx <signature> --rpc <url>        Query a specific RPC endpoint
 *   aether tx <signature> --wait             Poll until confirmed (max 60s)
 *   aether tx <signature> --wait 120          Poll with custom timeout (seconds)
 *   aether tx <signature> --logs              Show parsed instruction logs if available
 *
 * SDK Methods Used:
 *   - client.getTransaction(sig)   → GET /v1/transaction/<signature>
 *   - client.getSlot()              → GET /v1/slot
 *   - client.getBlockHeight()       → GET /v1/blockheight
 *
 * Examples:
 *   aether tx 5abc123...def           # Look up a transaction
 *   aether tx 5abc123...def --json    # JSON for scripting
 *   aether tx 5abc123...def --wait    # Wait up to 60s for confirmation
 *   aether tx 5abc123...def --logs    # Show transaction logs
 *
 * Note: Transactions are final once confirmed. Use --wait when submitting new
 * transactions to get immediate confirmation without a separate poll step.
 */

const path = require('path');

// Import SDK — REAL HTTP RPC calls to the blockchain
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
// SDK Client Setup
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ---------------------------------------------------------------------------
// Argument Parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    signature: null,
    rpc: getDefaultRpc(),
    asJson: false,
    showLogs: false,
    wait: false,
    waitTimeoutS: 60,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (!arg.startsWith('-') && !opts.signature) {
      // First non-flag is the signature
      opts.signature = arg;
    } else if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.asJson = true;
    } else if (arg === '--logs' || arg === '-l') {
      opts.showLogs = true;
    } else if (arg === '--wait' || arg === '-w') {
      opts.wait = true;
      // Check for custom timeout: --wait 30
      const next = args[i + 1];
      if (next && /^\d+$/.test(next)) {
        opts.waitTimeoutS = parseInt(next, 10);
        i++;
      }
    } else if (arg === '--help' || arg === '-h') {
      opts.help = true;
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli tx${C.reset} — Look up a transaction by signature

${C.bright}USAGE${C.reset}
    aether tx <signature> [options]

${C.bright}ARGUMENTS${C.reset}
    <signature>    Transaction signature (base58, required)

${C.bright}OPTIONS${C.reset}
    --rpc <url>      RPC endpoint (default: AETHER_RPC or localhost:8899)
    --json, -j       Output raw JSON for scripting
    --logs, -l       Show parsed transaction logs
    --wait, -w [s]   Poll until confirmed (default: 60s, max: 300s)
    --help, -h       Show this help

${C.bright}SDK METHODS${C.reset}
    client.getTransaction(sig)  → GET /v1/transaction/<signature>
    client.getSlot()            → GET /v1/slot
    client.getBlockHeight()      → GET /v1/blockheight

${C.bright}DESCRIPTION${C.reset}
    Queries the Aether blockchain for a specific transaction by its
    base58-encoded signature. Returns:
      • Confirmation status (confirmed / pending / not found)
      • Slot number (when the transaction landed)
      • Block time (unix timestamp)
      • Fee paid
      • Transaction type and data
      • Error message if failed
      • Program logs if available

    This is the primary command to verify a submitted transaction
    has been accepted by the network.

${C.bright}EXAMPLES${C.reset}
    aether tx 5abc123def...              # Human-readable lookup
    aether tx 5abc123def... --json        # JSON output
    aether tx 5abc123def... --wait        # Poll until confirmed (max 60s)
    aether tx 5abc123def... --logs        # Show program logs
    aether tx 5abc123def... --wait 120    # Poll with 120s timeout
    aether tx 5abc123def... --rpc https://my-node:8899

${C.green}✓ Fully wired to @jellylegsai/aether-sdk${C.reset}
`);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function shortSig(sig) {
  if (!sig || sig.length < 16) return sig || 'unknown';
  return sig.slice(0, 8) + '...' + sig.slice(-8);
}

function formatTime(unixTs) {
  if (!unixTs) return '—';
  try {
    return new Date(unixTs * 1000).toISOString().replace('T', ' ').substring(0, 19) + ' UTC';
  } catch {
    return String(unixTs);
  }
}

function formatAether(lamports) {
  if (lamports === undefined || lamports === null) return 'N/A';
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(6).replace(/\.?0+$/, '') + ' AETH';
}

function statusColorAndLabel(tx) {
  if (!tx) return { color: C.red, label: 'NOT FOUND' };
  if (tx.confirmed !== false && (tx.slot !== undefined || tx.blockTime !== undefined)) {
    return { color: C.green, label: 'CONFIRMED' };
  }
  if (tx.pending || tx.confirmations === null) {
    return { color: C.yellow, label: 'PENDING' };
  }
  if (tx.error || tx.err) {
    return { color: C.red, label: 'FAILED' };
  }
  return { color: C.yellow, label: 'UNKNOWN' };
}

// ---------------------------------------------------------------------------
// Transaction Fetch — REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchTransaction(rpcUrl, signature) {
  const client = createClient(rpcUrl);

  // Real RPC call: GET /v1/transaction/<signature>
  const tx = await client.getTransaction(signature);

  // Also get current slot for context
  let currentSlot = null;
  try {
    currentSlot = await client.getSlot();
  } catch {
    // Non-critical — don't fail the whole lookup
  }

  return { tx, currentSlot };
}

// ---------------------------------------------------------------------------
// Wait for Confirmation — polls getTransaction until confirmed or timeout
// ---------------------------------------------------------------------------

async function waitForConfirmation(rpcUrl, signature, timeoutS = 60) {
  const client = createClient(rpcUrl);
  const start = Date.now();
  const deadline = start + timeoutS * 1000;
  const pollIntervalMs = 2000;

  process.stdout.write(`  ${C.dim}Polling every ${pollIntervalMs / 1000}s (max ${timeoutS}s)...${C.reset}\n`);

  while (Date.now() < deadline) {
    try {
      const tx = await client.getTransaction(signature);

      if (tx && (tx.blockTime !== undefined || tx.slot !== undefined)) {
        const elapsed = ((Date.now() - start) / 1000).toFixed(1);
        return { confirmed: true, tx, waitedS: parseFloat(elapsed) };
      }
    } catch {
      // Not found yet — keep polling
    }

    // Progress indicator
    const elapsed = ((Date.now() - start) / 1000).toFixed(0);
    process.stdout.write(`\r  ${C.dim}[${elapsed}s] waiting for confirmation... ${C.reset}`);

    await new Promise(r => setTimeout(r, pollIntervalMs));
  }

  process.stdout.write('\r' + ' '.repeat(70) + '\r');
  return { confirmed: false, tx: null, waitedS: timeoutS };
}

// ---------------------------------------------------------------------------
// Output Formatters
// ---------------------------------------------------------------------------

function printTxHuman(tx, signature, rpcUrl, currentSlot) {
  const status = statusColorAndLabel(tx);

  console.log(`\n${C.bright}${C.cyan}╔════════════════════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║${C.reset}                   ${C.bright}AETHER TRANSACTION${C.reset}                            ${C.bright}║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚════════════════════════════════════════════════════════════════════╝${C.reset}`);

  // Status banner
  const statusBg = status.color === C.green ? C.green : status.color === C.red ? C.red : C.yellow;
  console.log(`\n  ${C.bright}┌────────────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.bright}│${C.reset}  ${C.bright}Status:${C.reset}  ${status.color}${C.bright}${status.label.padEnd(52)}${C.reset} ${C.bright}│${C.reset}`);
  console.log(`  ${C.bright}└────────────────────────────────────────────────────────────────────┘${C.reset}`);

  // Signature
  console.log(`\n  ${C.bright}Signature:${C.reset}  ${C.magenta}${signature}${C.reset}`);

  // Transaction details table
  console.log(`\n  ${C.bright}┌─────────────────────┬──────────────────────────────────────────────┐${C.reset}`);

  if (tx) {
    // Slot
    const slotStr = tx.slot !== undefined ? tx.slot.toLocaleString() : '—';
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Slot${C.reset}               ${C.bright}│${C.reset} ${slotStr.padEnd(44)} ${C.bright}│${C.reset}`);

    // Block time
    const timeStr = formatTime(tx.blockTime);
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Block Time${C.reset}           ${C.bright}│${C.reset} ${timeStr.padEnd(44)} ${C.bright}│${C.reset}`);

    // Fee
    const feeStr = tx.fee !== undefined ? formatAether(tx.fee) : '—';
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Fee${C.reset}                 ${C.bright}│${C.reset} ${feeStr.padEnd(44)} ${C.bright}│${C.reset}`);

    // Transaction type
    const typeStr = tx.type || tx.tx_type || tx.txType || '—';
    console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Type${C.reset}                ${C.bright}│${C.reset} ${typeStr.padEnd(44)} ${C.bright}│${C.reset}`);

    // Signer
    const signerStr = tx.signer || tx.from || tx.pubkey || '—';
    if (signerStr !== '—') {
      console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Signer${C.reset}              ${C.bright}│${C.reset} ${signerStr.padEnd(44)} ${C.bright}│${C.reset}`);
    }

    // Status details
    if (tx.confirmations !== undefined && tx.confirmations !== null) {
      console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Confirmations${C.reset}       ${C.bright}│${C.reset} ${String(tx.confirmations).padEnd(44)} ${C.bright}│${C.reset}`);
    }

    // Error
    if (tx.error || tx.err) {
      const errStr = (tx.error || tx.err).toString().substring(0, 44);
      console.log(`  ${C.bright}│${C.reset}  ${C.red}Error${C.reset}               ${C.bright}│${C.reset} ${errStr.padEnd(44)} ${C.bright}│${C.reset}`);
    }

    // Memo
    if (tx.memo) {
      const memoStr = tx.memo.toString().substring(0, 44);
      console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Memo${C.reset}               ${C.bright}│${C.reset} ${memoStr.padEnd(44)} ${C.bright}│${C.reset}`);
    }

    // Network context
    if (currentSlot !== null) {
      const slotsAgo = currentSlot !== null && tx.slot !== undefined ? currentSlot - tx.slot : null;
      const agoStr = slotsAgo !== null ? `${slotsAgo} slots ago` : '—';
      console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Current Slot${C.reset}        ${C.bright}│${C.reset} ${currentSlot.toLocaleString().padEnd(44)} ${C.bright}│${C.reset}`);
      console.log(`  ${C.bright}│${C.reset}  ${C.cyan}Confirmed${C.reset}           ${C.bright}│${C.reset} ${agoStr.padEnd(44)} ${C.bright}│${C.reset}`);
    }

  } else {
    console.log(`  ${C.bright}│${C.reset}  ${C.yellow}Transaction not found on chain${C.reset}                      ${C.bright}│${C.reset}`);
  }

  console.log(`  ${C.bright}└─────────────────────┴──────────────────────────────────────────────┘${C.reset}`);

  // Logs section
  if (tx && tx.logs && tx.logs.length > 0) {
    console.log(`\n  ${C.bright}── Program Logs ─────────────────────────────────────────────${C.reset}\n`);
    tx.logs.forEach((log, i) => {
      const prefix = log.includes('Error') || log.includes('failed') ? C.red : C.dim;
      console.log(`    ${prefix}${log}${C.reset}`);
    });
  }

  // RPC info
  console.log(`\n  ${C.dim}RPC: ${rpcUrl}${C.reset}`);
  console.log(`  ${C.dim}SDK: getTransaction() → GET /v1/transaction/${shortSig(signature)}${C.reset}\n`);
}

function printTxJson(tx, signature, rpcUrl) {
  const status = statusColorAndLabel(tx);

  const output = {
    signature,
    rpc: rpcUrl,
    status: status.label,
    confirmed: tx !== null && (tx.blockTime !== undefined || tx.slot !== undefined),
    slot: tx?.slot ?? null,
    block_time: tx?.blockTime ?? null,
    block_time_human: formatTime(tx?.blockTime),
    fee: tx?.fee ?? null,
    type: tx?.type ?? tx?.tx_type ?? tx?.txType ?? null,
    signer: tx?.signer ?? tx?.from ?? tx?.pubkey ?? null,
    error: tx?.error ?? tx?.err ?? null,
    logs: tx?.logs ?? null,
    memo: tx?.memo ?? null,
    confirmations: tx?.confirmations ?? null,
    raw: tx ?? null,
    cli_version: CLI_VERSION,
    timestamp: new Date().toISOString(),
    sdk_method: 'client.getTransaction()',
    rpc_endpoint: `GET /v1/transaction/${shortSig(signature)}`,
  };

  console.log(JSON.stringify(output, null, 2));
}

// ---------------------------------------------------------------------------
// Main Command
// ---------------------------------------------------------------------------

async function txCommand() {
  const opts = parseArgs();

  if (opts.help || !opts.signature) {
    showHelp();
    return;
  }

  const { signature, rpc, asJson, showLogs, wait, waitTimeoutS } = opts;

  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}── Transaction Lookup ───────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}Signature: ${C.magenta}${shortSig(signature)}${C.reset}`);
    console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);

    if (wait) {
      console.log(`  ${C.dim}Mode: polling until confirmed (timeout: ${waitTimeoutS}s)${C.reset}`);
    } else {
      console.log(`  ${C.dim}Mode: one-shot lookup${C.reset}`);
    }
    console.log();
  }

  if (wait) {
    // Wait for confirmation mode
    const effectiveTimeout = Math.min(waitTimeoutS, 300);
    const result = await waitForConfirmation(rpc, signature, effectiveTimeout);

    if (result.confirmed) {
      const tx = result.tx;

      if (asJson) {
        printTxJson(tx, signature, rpc);
      } else {
        const currentSlot = null; // Already waited, no need to fetch again
        printTxHuman(tx, signature, rpc, currentSlot);
        console.log(`  ${C.green}✓ Confirmed after ${result.waitedS}s${C.reset}\n`);
      }
    } else {
      if (asJson) {
        console.log(JSON.stringify({
          signature,
          rpc,
          status: 'TIMEOUT',
          confirmed: false,
          waited_s: result.waitedS,
          message: `Transaction not confirmed after ${result.waitedS}s`,
          cli_version: CLI_VERSION,
          timestamp: new Date().toISOString(),
        }, null, 2));
      } else {
        console.log(`\n  ${C.yellow}⚠ Timed out — transaction not confirmed after ${result.waitedS}s${C.reset}`);
        console.log(`  ${C.dim}It may still be pending. Check again with:${C.reset}`);
        console.log(`    ${C.cyan}aether tx ${signature}${C.reset}\n`);
      }
      process.exit(1);
    }
  } else {
    // One-shot lookup
    let tx;
    let currentSlot;

    if (!asJson) {
      process.stdout.write(`  ${C.dim}Fetching from ${rpc}...${C.reset}\n`);
    }

    try {
      const result = await fetchTransaction(rpc, signature);
      tx = result.tx;
      currentSlot = result.currentSlot;
    } catch (err) {
      if (asJson) {
        console.log(JSON.stringify({
          signature,
          rpc,
          error: err.message,
          status: 'ERROR',
          confirmed: false,
          cli_version: CLI_VERSION,
          timestamp: new Date().toISOString(),
        }, null, 2));
      } else {
        console.log(`\n  ${C.red}✗ Failed to fetch transaction:${C.reset} ${err.message}`);
        console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
        console.log(`  ${C.dim}The RPC endpoint may be down or the signature may be invalid.${C.reset}\n`);
      }
      process.exit(1);
    }

    if (asJson) {
      printTxJson(tx, signature, rpc);
    } else {
      // Suppress logs in human output unless requested
      if (!showLogs && tx) {
        tx = { ...tx, logs: undefined };
      }
      printTxHuman(tx, signature, rpc, currentSlot);
    }
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { txCommand };

if (require.main === module) {
  txCommand().catch(err => {
    console.error(`\n${C.red}✗ Tx command failed:${C.reset} ${err.message}\n`);
    process.exit(1);
  });
}
