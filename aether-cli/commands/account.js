#!/usr/bin/env node
/**
 * aether-cli account
 *
 * Query detailed on-chain account data for any address.
 * Shows lamports, owner, executable, rent epoch, and program-owned account keys.
 *
 * USAGE CHANGE: Now uses the SDK module instead of raw HTTP
 * 
 * Usage:
 *   aether account --address <addr>         Full account dump
 *   aether account --address <addr> --json  JSON output for scripting
 *   aether account --address <addr> --data  Show raw account data as base64/hex
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

const { getAccountInfo, getDefaultRpc } = require('../sdk');
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
};

// ---------------------------------------------------------------------------
// HTTP helpers - DEPRECATED: Kept for backwards compatibility only
// Use sdk.getAccountInfo() instead for new code
// ---------------------------------------------------------------------------

function getDefaultRpcUrl() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

// ---------------------------------------------------------------------------
// Parse args
// ---------------------------------------------------------------------------

function parseArgs() {
  return process.argv.slice(3); // [node, index.js, account, ...]
}

function findArg(args, ...flags) {
  for (const flag of flags) {
    const idx = args.indexOf(flag);
    if (idx !== -1 && args[idx + 1] && !args[idx + 1].startsWith('-')) {
      return { value: args[idx + 1], idx };
    }
  }
  return null;
}

function parseBoolArg(args, ...flags) {
  return flags.some(f => args.includes(f));
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatPubkey(key) {
  if (!key) return '—';
  if (Array.isArray(key)) {
    // base64-like or raw bytes → treat as public key bytes
    if (key.length === 32) return 'ATH' + bs58.encode(Buffer.from(key.slice(0, 32)));
    return 'ATH' + bs58.encode(Buffer.from(key));
  }
  if (typeof key === 'string') {
    if (key.startsWith('ATH')) return key;
    // Assume base58
    try { return 'ATH' + bs58.encode(bs58.decode(key)); }
    catch { return key.substring(0, 16) + '...'; }
  }
  return String(key).substring(0, 16);
}

function base64ToHex(b64) {
  try {
    const buf = Buffer.from(b64, 'base64');
    return buf.toString('hex');
  } catch {
    return b64;
  }
}

function formatData(data) {
  if (!data) return null;
  if (typeof data === 'string') return data;
  if (Array.isArray(data)) {
    // raw bytes → hex
    return Buffer.from(data).toString('hex');
  }
  return JSON.stringify(data);
}

// ---------------------------------------------------------------------------
// Main account query - NOW USING SDK
// ---------------------------------------------------------------------------

async function accountCommand() {
  const args = parseArgs();

  // Parse flags
  const addrArg = findArg(args, '--address', '-a');
  const asJson = parseBoolArg(args, '--json', '-j');
  const showData = parseBoolArg(args, '--data', '-d');
  const rpcArg = findArg(args, '--rpc', '-r');
  const rpcUrl = rpcArg ? rpcArg.value : getDefaultRpcUrl();

  if (!addrArg) {
    console.log(`\n  ${C.red}✗ Missing --address flag.${C.reset}`);
    console.log(`\n  ${C.cyan}Usage:${C.reset}`);
    console.log(`    ${C.cyan}aether account --address <addr>${C.reset}         Show account details`);
    console.log(`    ${C.cyan}aether account --address <addr> --json${C.reset}   JSON output`);
    console.log(`    ${C.cyan}aether account --address <addr> --data${C.reset}    Show raw account data`);
    console.log(`    ${C.cyan}aether account --address <addr> --rpc <url>${C.reset}  Use custom RPC\n`);
    process.exit(1);
  }

  const rawAddress = addrArg.value;
  const address = rawAddress.startsWith('ATH') ? rawAddress : rawAddress;

  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}── Account Info ──────────────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.green}★${C.reset} Address: ${C.bright}${address}${C.reset}`);
    console.log(`  ${C.dim}  RPC: ${rpcUrl}${C.reset}`);
    console.log(`  ${C.dim}  (Using SDK module)${C.reset}`);
    console.log();
  }

  try {
    // 🎉 NOW USING THE SDK MODULE - REAL BLOCKCHAIN RPC CALL!
    const account = await getAccountInfo(address, rpcUrl);

    if (!account || account.error) {
      if (asJson) {
        console.log(JSON.stringify({ address, error: account?.error || 'Account not found' }, null, 2));
      } else {
        console.log(`  ${C.yellow}⚠ Account not found on chain or RPC error.${C.reset}`);
        console.log(`  ${C.dim}  This is normal for addresses with no on-chain account.${C.reset}`);
        console.log(`  ${C.dim}  RPC response: ${JSON.stringify(account?.error || account)}${C.reset}\n`);
      }
      process.exit(1);
    }

    const lamports = account.lamports || account.lamports === 0 ? account.lamports : 0;
    const owner = account.owner || null;
    const executable = account.executable || false;
    const rentEpoch = account.rent_epoch;
    const data = account.data;
    const dataLen = data ? (Array.isArray(data) ? data.length : typeof data === 'string' ? data.length : 0) : 0;

    if (asJson) {
      let ownerStr = null;
      if (owner) {
        if (Array.isArray(owner)) {
          ownerStr = 'ATH' + bs58.encode(Buffer.from(owner.slice(0, 32)));
        } else if (typeof owner === 'string') {
          ownerStr = owner.startsWith('ATH') ? owner : 'ATH' + bs58.encode(bs58.decode(owner));
        } else {
          ownerStr = String(owner);
        }
      }

      console.log(JSON.stringify({
        address,
        rpc: rpcUrl,
        lamports,
        lamports_formatted: formatAether(lamports),
        owner: ownerStr,
        executable,
        rent_epoch: rentEpoch,
        data_size: dataLen,
        data: showData ? formatData(data) : null,
        sdk_version: '1.0.0',
        fetched_at: new Date().toISOString(),
      }, null, 2));
      return;
    }

    // Human-readable output
    console.log(`  ${C.green}✓${C.reset} Found on chain (via SDK)`);
    console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);
    console.log(`  ${C.dim}  Balance:${C.reset}     ${C.bright}${formatAether(lamports)}${C.reset}  (${lamports.toLocaleString()} lamports)`);

    if (owner) {
      let ownerStr;
      if (Array.isArray(owner)) {
        ownerStr = 'ATH' + bs58.encode(Buffer.from(owner.slice(0, 32)));
      } else if (typeof owner === 'string') {
        ownerStr = owner.startsWith('ATH') ? owner : 'ATH' + bs58.encode(bs58.decode(owner));
      } else {
        ownerStr = String(owner);
      }
      console.log(`  ${C.dim}  Owner:${C.reset}       ${C.cyan}${ownerStr}${C.reset}`);
    }

    console.log(`  ${C.dim}  Executable:${C.reset} ${C.bright}${(executable ? 'Yes' : 'No')}${C.reset}`);
    if (rentEpoch !== undefined) {
      console.log(`  ${C.dim}  Rent epoch:${C.reset} ${rentEpoch}`);
    }

    if (dataLen > 0) {
      console.log(`  ${C.dim}  Data size:${C.reset}  ${C.bright}${dataLen} bytes${C.reset}`);
    } else {
      console.log(`  ${C.dim}  Data size:${C.reset}  0 bytes`);
    }

    if (showData && data) {
      console.log();
      console.log(`  ${C.dim}  Raw data (hex):${C.reset}`);
      const hex = formatData(data);
      // Pretty-print in 32-byte chunks
      const chunks = hex.match(/.{1,64}/g) || [];
      for (let i = 0; i < chunks.length; i++) {
        const offset = (i * 32).toString(16).padStart(8, '0');
        console.log(`    ${C.dim}${offset}:${C.reset} ${chunks[i]}`);
      }
    }

    console.log();
    console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);

    // Additional context: try to determine account type
    let accountType = 'Unknown';
    if (owner) {
      const ownerStr = Array.isArray(owner)
        ? bs58.encode(Buffer.from(owner.slice(0, 32)))
        : (typeof owner === 'string' && !owner.startsWith('ATH') ? owner : '');
      // Known program IDs (these are common Aether program identifiers)
      const knownPrograms = {
        'STAKE': 'Stake Program',
        'SYSTEM': 'System Program',
        'VOTE': 'Vote Program',
        'TOKEN': 'Token Program',
        'MEMO': 'Memo Program',
      };
      const prog = knownPrograms[ownerStr.toUpperCase()];
      if (prog) accountType = prog;
      else if (executable) accountType = 'Executable Program';
      else if (dataLen === 0) accountType = 'Empty Account (no data)';
      else accountType = 'Data Account';
    }

    console.log(`  ${C.dim}  Account type:${C.reset} ${C.bright}${accountType}${C.reset}`);
    console.log();
    console.log(`  ${C.dim}  Fetched via SDK:${C.reset} getAccountInfo() → RPC`);
    console.log();

  } catch (err) {
    if (asJson) {
      console.log(JSON.stringify({ address, error: err.message, sdk_used: true }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to fetch account:${C.reset} ${err.message}`);
      console.log(`  ${C.dim}  Is your validator running? RPC: ${rpcUrl}${C.reset}`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Export & run
// ---------------------------------------------------------------------------

module.exports = { accountCommand };

if (require.main === module) {
  accountCommand();
}
