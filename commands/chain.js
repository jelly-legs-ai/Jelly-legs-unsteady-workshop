#!/usr/bin/env node
/**
 * aether-cli chain
 *
 * Display chain-level blockchain data: current slot, epoch, validator count,
 * and chain ID. Uses @jellylegsai/aether-sdk for REAL HTTP RPC calls.
 *
 * Usage:
 *   aether chain                    Show chain overview
 *   aether chain --json            JSON output for scripting
 *   aether chain --rpc <url>       Custom RPC endpoint
 *   aether chain --validators      Include validator count
 *   aether chain --compact         Single-line summary
 *
 * SDK Methods Used:
 *   - client.getSlot()            → GET /v1/slot
 *   - client.getEpochInfo()        → GET /v1/epoch
 *   - client.getValidators()       → GET /v1/validators
 *   - client.getVersion()          → GET /v1/version
 *
 * RPC Endpoints Called:
 *   - GET /v1/slot              → current slot
 *   - GET /v1/epoch              → epoch info (epoch, slotIndex, slotsInEpoch)
 *   - GET /v1/validators         → validator list (for count)
 *   - GET /v1/version            → chain ID / version info
 */

const path = require('path');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ANSI colours (same palette as other commands)
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
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

function formatNumber(n) {
  if (n === null || n === undefined) return `${C.dim}N/A${C.reset}`;
  return n.toLocaleString();
}

function formatAether(lamports) {
  if (!lamports && lamports !== 0) return `${C.dim}N/A${C.reset}`;
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  return {
    rpc: getDefaultRpc(),
    asJson: args.includes('--json') || args.includes('-j'),
    compact: args.includes('--compact'),
    showValidators: args.includes('--validators') || args.includes('-v'),
    help: args.includes('--help') || args.includes('-h'),
  };
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli chain${C.reset} — Blockchain Chain Data

${C.bright}USAGE${C.reset}
    aether chain [options]

${C.bright}OPTIONS${C.reset}
    -r, --rpc <url>       RPC endpoint (default: ${getDefaultRpc()})
    -j, --json           Output raw JSON for scripting
    -c, --compact        Single-line summary
    -v, --validators     Include validator count
    -h, --help           Show this help message

${C.bright}SDK METHODS${C.reset}
    getSlot()         → GET /v1/slot
    getEpochInfo()    → GET /v1/epoch
    getValidators()   → GET /v1/validators
    getVersion()       → GET /v1/version

${C.bright}EXAMPLES${C.reset}
    aether chain              # Full chain overview
    aether chain --json       # JSON output
    aether chain --compact    # One-line summary
    aether chain --validators # Include validator count
    aether chain --rpc http://localhost:8899
`);
}

// ---------------------------------------------------------------------------
// Fetch chain data via SDK
// ---------------------------------------------------------------------------

async function fetchChainData(rpc) {
  const client = createClient(rpc);

  const [slotResult, epochResult, validatorsResult, versionResult] = await Promise.allSettled([
    client.getSlot().catch(err => ({ error: err.message, code: err.code })),
    client.getEpochInfo().catch(err => ({ error: err.message, code: err.code })),
    client.getValidators().catch(err => ({ error: err.message, code: err.code })),
    client.getVersion().catch(err => ({ error: err.message, code: err.code })),
  ]);

  const slot = slotResult.status === 'fulfilled' ? slotResult.value : null;
  const epoch = epochResult.status === 'fulfilled' ? epochResult.value : null;
  const validators = validatorsResult.status === 'fulfilled' ? validatorsResult.value : [];
  const version = versionResult.status === 'fulfilled' ? versionResult.value : {};

  // Extract chain ID from version response
  const chainId = version.chainId || version.network || version['aether-core'] ||
                  version.version || `unknown-${rpc}`;

  // Compute derived values
  let currentSlot = null;
  let currentEpoch = null;
  let slotIndex = null;
  let slotsInEpoch = null;
  let epochProgress = null;
  let absoluteSlot = null;

  if (slot !== null && typeof slot === 'object') {
    currentSlot = slot.slot ?? slot.current_slot ?? null;
  } else if (typeof slot === 'number') {
    currentSlot = slot;
  }

  if (epoch !== null && typeof epoch === 'object') {
    currentEpoch = epoch.epoch ?? epoch.current_epoch ?? null;
    slotIndex = epoch.slotIndex ?? epoch.slot_index ?? epoch.current_slot ?? null;
    slotsInEpoch = epoch.slotsInEpoch ?? epoch.slots_in_epoch ?? null;
    absoluteSlot = epoch.absoluteSlot ?? epoch.absolute_slot ?? currentSlot;
    if (slotIndex !== null && slotsInEpoch !== null && slotsInEpoch > 0) {
      epochProgress = Math.min(100, Math.round((slotIndex / slotsInEpoch) * 100));
    }
  } else if (typeof epoch === 'number') {
    currentEpoch = epoch;
  }

  const validatorCount = Array.isArray(validators) ? validators.length : 0;

  return {
    currentSlot,
    currentEpoch,
    slotIndex,
    slotsInEpoch,
    epochProgress,
    absoluteSlot,
    validatorCount,
    chainId,
    version,
    raw: { slot, epoch, validators, version },
    rpc,
    fetchedAt: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printChainSummary(data) {
  const { currentSlot, currentEpoch, slotIndex, slotsInEpoch, epochProgress,
          validatorCount, chainId, rpc } = data;

  // Progress bar for epoch
  const barWidth = 40;
  const filled = Math.floor((epochProgress || 0) / 100 * barWidth);
  const empty = barWidth - filled;
  const bar = C.green + '█'.repeat(filled) + C.dim + '░'.repeat(empty) + C.reset;

  console.log(`\n${C.bright}${C.cyan}── Aether Chain ────────────────────────────────────────${C.reset}\n`);

  if (currentSlot !== null) {
    console.log(`  ${C.bright}Current Slot${C.reset}   ${C.green}${formatNumber(currentSlot)}${C.reset}`);
  } else {
    console.log(`  ${C.bright}Current Slot${C.reset}   ${C.red}unreachable${C.reset}`);
  }

  if (currentEpoch !== null) {
    console.log(`  ${C.bright}Current Epoch${C.reset}   ${C.cyan}${currentEpoch}${C.reset}`);
    if (slotIndex !== null && slotsInEpoch !== null) {
      console.log(`  ${C.bright}Slot in Epoch${C.reset}  ${C.dim}${formatNumber(slotIndex)} / ${formatNumber(slotsInEpoch)}${C.reset}`);
    }
    if (epochProgress !== null) {
      console.log(`  ${C.bright}Epoch Progress${C.reset} ${bar} ${C.bright}${epochProgress}%${C.reset}`);
    }
  } else {
    console.log(`  ${C.bright}Current Epoch${C.reset}   ${C.dim}N/A${C.reset}`);
  }

  if (validatorCount > 0) {
    console.log(`  ${C.bright}Validators${C.reset}      ${C.magenta}${formatNumber(validatorCount)}${C.reset}`);
  }

  console.log(`  ${C.bright}Chain ID${C.reset}        ${C.yellow}${chainId}${C.reset}`);

  console.log(`\n  ${C.dim}RPC: ${rpc}${C.reset}\n`);
}

function printChainCompact(data) {
  const { currentSlot, currentEpoch, validatorCount, chainId, rpc } = data;
  const parts = [];
  if (currentSlot !== null) parts.push(`slot=${formatNumber(currentSlot)}`);
  if (currentEpoch !== null) parts.push(`epoch=${currentEpoch}`);
  if (validatorCount > 0) parts.push(`validators=${validatorCount}`);
  parts.push(`chain=${chainId}`);
  console.log(parts.join('  '));
}

function printChainJson(data) {
  const { currentSlot, currentEpoch, slotIndex, slotsInEpoch, epochProgress,
          validatorCount, chainId, version, rpc, fetchedAt } = data;

  console.log(JSON.stringify({
    chain: {
      current_slot: currentSlot,
      current_epoch: currentEpoch,
      slot_index: slotIndex,
      slots_in_epoch: slotsInEpoch,
      epoch_progress: epochProgress,
      absolute_slot: data.absoluteSlot,
      validator_count: validatorCount,
      chain_id: chainId,
    },
    version,
    rpc,
    fetched_at: fetchedAt,
    sdk_methods: ['getSlot', 'getEpochInfo', 'getValidators', 'getVersion'],
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function chainCommand() {
  const opts = parseArgs();

  if (opts.help) {
    showHelp();
    return;
  }

  try {
    const data = await fetchChainData(opts.rpc);

    if (opts.compact) {
      printChainCompact(data);
    } else if (opts.asJson) {
      printChainJson(data);
    } else {
      printChainSummary(data);
    }
  } catch (err) {
    if (opts.asJson) {
      console.log(JSON.stringify({
        error: err.message,
        rpc: opts.rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${C.red}✗ Chain command failed: ${err.message}${C.reset}`);
      console.log(`  ${C.dim}RPC: ${opts.rpc}${C.reset}`);
      console.log(`  ${C.dim}Set AETHER_RPC to your validator's RPC endpoint.${C.reset}\n`);
    }
    process.exit(1);
  }
}

module.exports = { chainCommand };

if (require.main === module) {
  chainCommand().catch(err => {
    console.error(`${C.red}✗ Chain command failed: ${err.message}${C.reset}`);
    process.exit(1);
  });
}