#!/usr/bin/env node
/**
 * aether-cli broadcast
 *
 * Broadcast a signed transaction to the Aether network.
 * Fully wired to @jellylegsai/aether-sdk — uses AetherClient for all RPC calls.
 *
 * Accepts a base58-encoded transaction signature or a raw JSON payload.
 * Useful for submitting offline-constructed transactions.
 *
 * Usage:
 *   aether broadcast --tx <signature>           Broadcast by tx signature
 *   aether broadcast --json <payload>          Broadcast raw JSON tx payload
 *   aether broadcast --file <path>             Read tx from a JSON file
 *   aether broadcast --rpc <url>                Use a specific RPC endpoint
 *   aether broadcast --wait                     Wait for confirmation (max 60s)
 *   aether broadcast --json                    JSON output for scripting
 *
 * SDK Methods Used:
 *   - client.sendTransaction(tx)        → POST /v1/transaction
 *   - client.getSlot()                   → GET /v1/slot
 *   - client.getTransaction(signature)   → GET /v1/transaction/<sig>
 *
 * Examples:
 *   aether broadcast --tx 5abcdef...           # Submit pre-signed tx
 *   aether broadcast --json '{"type":"Transfer",...}'
 *   aether broadcast --file ./unsigned_tx.json
 *   aether broadcast --tx <sig> --wait --json
 */

const fs = require('fs');
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
};

const CLI_VERSION = '1.1.0';

// ---------------------------------------------------------------------------
// SDK Import - uses @jellylegsai/aether-sdk for ALL blockchain RPC calls
// ---------------------------------------------------------------------------

const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

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
  const opts = {
    rpc: getDefaultRpc(),
    signature: null,
    jsonPayload: null,
    filePath: null,
    asJson: false,
    wait: false,
    waitTimeoutMs: 60000,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--tx' || args[i] === '-t') {
      opts.signature = args[++i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      opts.jsonPayload = args[++i];
    } else if (args[i] === '--file' || args[i] === '-f') {
      opts.filePath = args[++i];
    } else if (args[i] === '--rpc' || args[i] === '-r') {
      opts.rpc = args[++i];
    } else if (args[i] === '--wait' || args[i] === '-w') {
      opts.wait = true;
    } else if (args[i] === '--json-output') {
      // Backward-compatible alias (used in old code)
      opts.asJson = true;
    } else if (args[i] === '--json') {
      opts.asJson = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli broadcast${C.reset} - Broadcast a Signed Transaction

${C.bright}Usage:${C.reset}
  aether-cli broadcast --tx <signature>         Broadcast by base58 signature
  aether-cli broadcast --json <payload>        Broadcast inline JSON payload
  aether-cli broadcast --file <path>            Read tx from a JSON file
  aether-cli broadcast --rpc <url>             Override default RPC
  aether-cli broadcast --wait                  Poll for confirmation (max 60s)
  aether-cli broadcast --json                  JSON output for scripting

${C.bright}SDK Methods Used:${C.reset}
  client.sendTransaction(tx)      → POST /v1/transaction
  client.getTransaction(sig)      → GET /v1/transaction/<sig>
  client.getSlot()                → GET /v1/slot

${C.bright}Examples:${C.reset}
  aether-cli broadcast --tx 5abcdef123456...  # Submit by signature
  aether-cli broadcast --json '{"type":"Transfer","data":{...}}'
  aether-cli broadcast --file ./my_tx.json
  aether-cli broadcast --tx <sig> --wait --json  # Broadcast and wait for confirm
`.trim());
}

// ---------------------------------------------------------------------------
// Validate transaction payload
// ---------------------------------------------------------------------------

function validateTxPayload(tx) {
  const errors = [];

  if (!tx) {
    errors.push('Transaction payload is null or empty');
    return errors;
  }

  // Must have signer
  if (!tx.signer && !tx.from && !tx.pubkey) {
    errors.push('Missing signer field (signer | from | pubkey)');
  }

  // Must have tx_type or type
  if (!tx.tx_type && !tx.type) {
    errors.push('Missing tx_type or type field');
  }

  // Must have payload
  if (!tx.payload && !tx.data) {
    errors.push('Missing payload or data field');
  }

  return errors;
}

// ---------------------------------------------------------------------------
// Wait for transaction confirmation via SDK
// Polls getTransaction() until the signature appears on-chain or times out.
// ---------------------------------------------------------------------------

async function waitForConfirmation(client, signature, timeoutMs = 60000, pollIntervalMs = 2000) {
  const start = Date.now();
  let lastResult = null;

  while (Date.now() - start < timeoutMs) {
    try {
      // SDK call: getTransaction → GET /v1/transaction/<signature>
      const result = await client.getTransaction(signature);
      lastResult = result;

      // If result has blockTime or slot, tx is confirmed
      if (result && (result.blockTime !== undefined || result.slot !== undefined)) {
        return { confirmed: true, result, waitedMs: Date.now() - start };
      }
    } catch (err) {
      // Transaction not yet visible — expected during confirmation
      lastResult = { error: err.message };
    }

    await new Promise(r => setTimeout(r, pollIntervalMs));
  }

  return { confirmed: false, result: lastResult, waitedMs: Date.now() - start };
}

// ---------------------------------------------------------------------------
// Broadcast logic - uses SDK for all RPC calls
// ---------------------------------------------------------------------------

async function broadcast({ rpc, signature, jsonPayload, filePath, asJson, wait }) {
  const client = createClient(rpc);

  // Build the tx object from inputs (priority: signature > file > inline JSON)
  let tx = null;

  if (signature) {
    // Signature-only broadcast: the SDK's sendTransaction accepts { signature }
    tx = { signature };
  } else if (filePath) {
    // Read from file
    const absPath = path.isAbsolute(filePath) ? filePath : path.resolve(process.cwd(), filePath);
    if (!fs.existsSync(absPath)) {
      throw new Error(`File not found: ${absPath}`);
    }
    try {
      tx = JSON.parse(fs.readFileSync(absPath, 'utf8'));
    } catch {
      throw new Error(`Invalid JSON in file: ${absPath}`);
    }
  } else if (jsonPayload) {
    try {
      tx = JSON.parse(jsonPayload);
    } catch {
      throw new Error('Invalid JSON payload provided with --json');
    }
  } else {
    throw new Error('No transaction provided. Use --tx, --json, or --file');
  }

  // Validate the transaction has required fields
  if (!signature) {
    const validationErrors = validateTxPayload(tx);
    if (validationErrors.length > 0) {
      throw new Error('Invalid transaction payload:\n  ' + validationErrors.join('\n  '));
    }
  }

  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}── Broadcast Transaction ─────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}SDK: AetherClient → ${rpc}${C.reset}`);
    if (signature) {
      console.log(`  ${C.dim}Signature:${C.reset} ${C.cyan}${signature}${C.reset}`);
    } else {
      const txType = tx.tx_type || tx.type || 'Unknown';
      const signer = tx.signer || tx.from || tx.pubkey || 'unknown';
      console.log(`  ${C.dim}Type:${C.reset} ${C.cyan}${txType}${C.reset}`);
      console.log(`  ${C.dim}Signer:${C.reset} ${C.cyan}${signer}${C.reset}`);
    }
    console.log();
  }

  // Submit the transaction via SDK
  let result;
  let latencyMs;
  let submittedSig = signature || tx.signature || null;

  try {
    const start = Date.now();
    // SDK call: sendTransaction → POST /v1/transaction
    result = await client.sendTransaction(tx);
    latencyMs = Date.now() - start;

    // Capture returned signature if different from input
    if (result && result.signature && !submittedSig) {
      submittedSig = result.signature;
    }
  } catch (err) {
    if (asJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`  ${C.red}✗ SDK error:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}  SDK Method: client.sendTransaction() → POST /v1/transaction${C.reset}`);
    }
    process.exit(1);
  }

  const success = result && !result.error && result.accepted !== false;

  if (asJson) {
    console.log(JSON.stringify({
      success,
      accepted: result?.accepted ?? null,
      signature: result?.signature ?? submittedSig ?? null,
      slot: result?.slot ?? null,
      error: result?.error ?? null,
      rpc,
      sdk_method: 'client.sendTransaction()',
      rpc_endpoint: 'POST /v1/transaction',
      latency_ms: latencyMs,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }, null, 2));

    if (wait && submittedSig && success) {
      process.stdout.write(JSON.stringify({ confirming: true, signature: submittedSig }) + '\n');
      const confirmResult = await waitForConfirmation(client, submittedSig, 60000);
      console.log(JSON.stringify({
        confirmed: confirmResult.confirmed,
        signature: submittedSig,
        confirm_waited_ms: confirmResult.waitedMs,
        slot: confirmResult.result?.slot ?? null,
        blocktime: confirmResult.result?.blockTime ?? null,
      }, null, 2));
    }
    return;
  }

  if (success) {
    const sig = result?.signature ?? submittedSig ?? 'unknown';
    console.log(`${C.green}✓ Transaction accepted!${C.reset}`);
    console.log(`  ${C.green}★${C.reset} ${C.bright}Signature:${C.reset} ${sig}`);
    if (result?.slot) {
      console.log(`  ${C.dim}  Slot:${C.reset} ${result.slot}`);
    }
    console.log(`  ${C.dim}  Latency:${C.reset} ${latencyMs}ms`);
    console.log(`  ${C.dim}  SDK Method:${C.reset} client.sendTransaction() → POST /v1/transaction`);
    console.log(`  ${C.dim}  RPC:${C.reset} ${rpc}`);
    console.log();

    // Wait for confirmation if requested
    if (wait && submittedSig) {
      console.log(`  ${C.dim}Waiting for confirmation...${C.reset}`);
      const confirmResult = await waitForConfirmation(client, submittedSig, 60000);

      if (confirmResult.confirmed) {
        console.log(`  ${C.green}✓ Confirmed!${C.reset}`);
        console.log(`  ${C.dim}  Waited:${C.reset} ${confirmResult.waitedMs}ms`);
        if (confirmResult.result?.slot) {
          console.log(`  ${C.dim}  Slot:${C.reset} ${confirmResult.result.slot}`);
        }
        if (confirmResult.result?.blockTime) {
          const confirmedAt = new Date(confirmResult.result.blockTime * 1000).toISOString();
          console.log(`  ${C.dim}  Block time:${C.reset} ${confirmedAt}`);
        }
      } else {
        console.log(`  ${C.yellow}⚠ Transaction submitted but not yet confirmed${C.reset}`);
        console.log(`  ${C.dim}  Signature:${C.reset} ${sig}`);
        console.log(`  ${C.dim}  Check manually: aether tx ${sig}${C.reset}`);
      }
      console.log();
    }
  } else {
    const errMsg = result?.error || 'Transaction rejected by network';
    console.log(`${C.red}✗ Transaction rejected${C.reset}`);
    if (result?.error) {
      console.log(`  ${C.red}  Error:${C.reset} ${result.error}`);
    }
    if (result?.logs) {
      console.log(`  ${C.dim}  Logs:${C.reset}`);
      for (const log of result.logs) {
        console.log(`    ${C.dim}${log}${C.reset}`);
      }
    }
    console.log(`  ${C.dim}  Latency:${C.reset} ${latencyMs}ms`);
    console.log(`  ${C.dim}  RPC:${C.reset} ${rpc}`);
    console.log();
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();

  try {
    await broadcast(opts);
  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({
        success: false,
        error: err.message,
        rpc: opts.rpc,
        cli_version: CLI_VERSION,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ ${err.message}${C.reset}\n`);
    }
    process.exit(1);
  }
}

main();

module.exports = { broadcastCommand: main };
