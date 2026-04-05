#!/usr/bin/env node
/**
 * aether-cli broadcast
 *
 * Broadcast a signed transaction to the Aether network.
 * Accepts a base58-encoded transaction signature or a raw JSON payload.
 * Useful for submitting offline-constructed transactions.
 *
 * Usage:
 *   aether broadcast --tx <signature>           Broadcast by tx signature
 *   aether broadcast --json <payload>          Broadcast raw JSON tx payload
 *   aether broadcast --file <path>             Read tx from a JSON file
 *   aether broadcast --rpc <url>                Use a specific RPC endpoint
 *   aether broadcast --json-output            Output result as JSON
 *
 * Examples:
 *   aether broadcast --tx 5abcdef...           # Submit pre-signed tx
 *   aether broadcast --json '{"type":"Transfer",...}'
 *   aether broadcast --file ./unsigned_tx.json
 */

const fs = require('fs');
const path = require('path');
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
};

const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
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
    } else if (args[i] === '--json-output') {
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
  aether-cli broadcast --json-output          JSON output for scripting

${C.bright}Examples:${C.reset}
  aether-cli broadcast --tx 5abcdef123456...  # Submit by signature
  aether-cli broadcast --json '{"type":"Transfer","data":{...}}'
  aether-cli broadcast --file ./my_tx.json    # Read and broadcast from file
`.trim());
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, pathStr, method = 'GET', body = null, timeoutMs = 15000) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathStr, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = body ? JSON.stringify(body) : null;

    const reqOptions = {
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method,
      timeout: timeoutMs,
      headers: {
        'Content-Type': 'application/json',
        ...(bodyStr ? { 'Content-Length': Buffer.byteLength(bodyStr) } : {}),
      },
    };

    const req = lib.request(reqOptions, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ raw: data }); }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    if (bodyStr) req.write(bodyStr);
    req.end();
  });
}

function httpPost(rpcUrl, pathStr, body, timeoutMs = 15000) {
  return httpRequest(rpcUrl, pathStr, 'POST', body, timeoutMs);
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
// Broadcast logic
// ---------------------------------------------------------------------------

async function broadcast({ rpc, signature, jsonPayload, filePath, asJson }) {
  // Build the tx object from inputs (priority: signature > file > inline JSON)
  let tx = null;

  if (signature) {
    // Signature-only broadcast: POST /v1/tx with { signature }
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
    console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
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

  // Submit the transaction
  let result;
  let latencyMs;

  try {
    const start = Date.now();
    result = await httpPost(rpc, '/v1/tx', tx);
    latencyMs = Date.now() - start;
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
      console.log(`  ${C.red}✗ Network error:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  Check that your RPC is accessible: ${rpc}${C.reset}`);
    }
    process.exit(1);
  }

  const success = result && !result.error && result.accepted !== false;

  if (asJson) {
    console.log(JSON.stringify({
      success,
      accepted: result?.accepted ?? null,
      signature: result?.signature ?? result?.tx_signature ?? signature ?? null,
      slot: result?.slot ?? null,
      error: result?.error ?? null,
      rpc,
      latency_ms: latencyMs,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }

  if (success) {
    const sig = result?.signature ?? result?.tx_signature ?? signature ?? 'unknown';
    console.log(`${C.green}✓ Transaction accepted!${C.reset}`);
    console.log(`  ${C.green}★${C.reset} ${C.bright}Signature:${C.reset} ${sig}`);
    if (result?.slot) {
      console.log(`  ${C.dim}  Slot:${C.reset} ${result.slot}`);
    }
    console.log(`  ${C.dim}  Latency:${C.reset} ${latencyMs}ms`);
    console.log(`  ${C.dim}  RPC:${C.reset} ${rpc}`);
    console.log();
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
