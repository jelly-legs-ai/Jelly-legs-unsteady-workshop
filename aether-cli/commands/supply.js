#!/usr/bin/env node
/**
 * aether-cli supply
 *
 * Display Aether network token supply metrics:
 *   - Total supply of AETH (all accounts + locked/escrow)
 *   - Circulating supply (liquid, tradeable tokens)
 *   - Staked supply (locked in stake accounts)
 *   - Burned supply (tokens sent to burn address / invalid addresses)
 *
 * Usage:
 *   aether supply                  Show supply overview
 *   aether supply --json          JSON output for scripting/monitoring
 *   aether supply --rpc <url>     Query a specific RPC endpoint
 *   aether supply --verbose       Show breakdown by account type
 *
 * Requires AETHER_RPC env var (default: http://127.0.0.1:8899)
 */

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
  magenta: '\x1b[35m',
  bold: '\x1b[1m',
};

const CLI_VERSION = '1.0.0';

// ---------------------------------------------------------------------------
// SDK Import - Real blockchain RPC calls via @jellylegsai/aether-sdk
// ---------------------------------------------------------------------------

const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

/** Create SDK client */
function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

function formatAether(lamports) {
  const aeth = Number(lamports) / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatAethFull(lamports) {
  return (Number(lamports) / 1e9).toFixed(6) + ' AETH';
}

function formatLargeNum(n) {
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

// ---------------------------------------------------------------------------
// Core supply fetchers using SDK
// ---------------------------------------------------------------------------

/**
 * Fetch the total supply of AETH from the chain using SDK.
 * Makes real RPC call: GET /v1/supply
 */
async function fetchTotalSupply(rpc) {
  const client = createClient(rpc);
  try {
    // Primary: SDK getSupply() → GET /v1/supply
    const res = await client.getSupply();
    if (res && (res.total !== undefined || res.supply !== undefined)) {
      return {
        total: BigInt(res.total || res.supply?.total || 0),
        circulating: BigInt(res.circulating || res.supply?.circulating || 0),
        nonCirculating: BigInt(res.non_circulating || res.nonCirculating || res.supply?.non_circulating || 0),
        source: 'rpc_v1_supply',
      };
    }
  } catch { /* fall through */ }

  // Fallback: fetch epoch info which contains total token count
  try {
    const epochInfo = await client.getEpochInfo();
    if (epochInfo) {
      const totalStaked = BigInt(epochInfo.total_staked || 0);
      const rewardsPerEpoch = BigInt(epochInfo.rewards_per_epoch || '2000000000');
      const currentEpoch = BigInt(epochInfo.epoch || 0);
      // Rough estimate: total supply ~= minted so far + remaining allocation
      // Aether has ~500M AETH max supply, minted gradually over 100 years
      const maxSupply = BigInt('500000000000000000'); // 500M * 1e9
      const mintedPerEpoch = rewardsPerEpoch;
      const minted = mintedPerEpoch * currentEpoch;
      // Some tokens are locked/vesting; assume ~30% is non-circulating
      const estimatedTotal = minted < maxSupply ? minted : maxSupply;
      const estimatedCirculating = estimatedTotal - BigInt(BigInt(estimatedTotal) / BigInt(3));
      return {
        total: estimatedTotal,
        circulating: estimatedCirculating,
        nonCirculating: estimatedTotal - estimatedCirculating,
        source: 'epoch_info_estimate',
      };
    }
  } catch { /* fall through */ }

  return null;
}

/**
 * Fetch staked supply by querying stake program accounts using SDK.
 * Makes real RPC call: GET /v1/validators
 */
async function fetchStakedSupply(rpc) {
  const client = createClient(rpc);
  try {
    // SDK getValidators() → GET /v1/validators
    const validators = await client.getValidators();
    if (validators && Array.isArray(validators)) {
      let total = BigInt(0);
      for (const v of validators) {
        total += BigInt(v.delegated_stake || v.stake || v.delegatedStake || 0);
      }
      return total;
    }
  } catch { /* fall through */ }

  try {
    // Last resort: epoch info staked amount via SDK
    const epochInfo = await client.getEpochInfo();
    if (epochInfo && epochInfo.total_staked) {
      return BigInt(epochInfo.total_staked);
    }
  } catch { /* fall through */ }

  return BigInt(0);
}

/**
 * Estimate burned supply by querying accounts at known burn/mint addresses using SDK.
 * Makes real RPC calls: GET /v1/account/<address>
 */
async function fetchBurnedSupply(rpc) {
  const client = createClient(rpc);
  const BURN_ADDRESSES = [
    'ATH1111111111111111111111111111111111111',  // mint authority burn
    'ATH2222222222222222222222222222222222222',  // zero authority
    'ATHburn000000000000000000000000000000',     // burn address
  ];

  let totalBurned = BigInt(0);

  for (const addr of BURN_ADDRESSES) {
    try {
      const rawAddr = addr.startsWith('ATH') ? addr.slice(3) : addr;
      // SDK getAccountInfo() → GET /v1/account/<address>
      const account = await client.getAccountInfo(rawAddr);
      if (account && account.lamports !== undefined && Number(account.lamports) > 0) {
        totalBurned += BigInt(account.lamports);
      }
    } catch { /* skip inaccessible addresses */ }
  }

  return totalBurned;
}

/**
 * Fetch circulating supply = total - non-circulating (locked/vesting/burned).
 * Non-circulating includes: burn address, escrow/staking vault, team vesting.
 */
async function fetchNonCirculatingAccounts(rpc) {
  try {
    const res = await httpRequest(rpc, '/v1/supply/non-circulating');
    if (res && !res.error && Array.isArray(res.accounts)) {
      let total = BigInt(0);
      for (const acct of res.accounts) {
        total += BigInt(acct.lamports || 0);
      }
      return total;
    }
  } catch { /* fall through */ }

  return BigInt(0);
}

// ---------------------------------------------------------------------------
// Render output
// ---------------------------------------------------------------------------

function renderSupplyTable(data) {
  const { total, circulating, staked, burned, nonCirculating, rpc, source } = data;

  const circPct = total > 0 ? ((Number(circulating) / Number(total)) * 100).toFixed(1) : '?';
  const stakedPct = total > 0 ? ((Number(staked) / Number(total)) * 100).toFixed(1) : '?';
  const burnedPct = total > 0 ? ((Number(burned) / Number(total)) * 100).toFixed(2) : '?';

  console.log(`\n${C.bold}${C.cyan}╔═══════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bold}${C.cyan}║          AETHER TOKEN SUPPLY                              ║${C.reset}`);
  console.log(`${C.bold}${C.cyan}╚═══════════════════════════════════════════════════════╝${C.reset}\n`);
  console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
  console.log(`  ${C.dim}Source: ${source}${C.reset}\n`);

  console.log(`  ${C.bright}┌─ ${C.cyan}TOTAL SUPPLY${C.reset}`);
  console.log(`  │  ${C.bold}${formatAethFull(total)}${C.reset}`);
  console.log(`  │  ${C.dim}${formatLargeNum(Number(total))} lamports${C.reset}`);
  console.log(`  ${C.dim}└${C.reset}`);
  console.log();

  console.log(`  ${C.bright}┌─ ${C.green}CIRCULATING SUPPLY${C.reset}`);
  console.log(`  │  ${C.green}${formatAethFull(circulating)}${C.reset}`);
  console.log(`  │  ${C.dim}${formatLargeNum(Number(circulating))} lamports${C.reset}`);
  console.log(`  │  ${C.green}${circPct}%${C.reset} of total supply`);
  console.log(`  ${C.dim}└${C.reset}`);
  console.log();

  console.log(`  ${C.bright}┌─ ${C.yellow}STAKED SUPPLY${C.reset}`);
  console.log(`  │  ${C.yellow}${formatAethFull(staked)}${C.reset}`);
  console.log(`  │  ${C.dim}${formatLargeNum(Number(staked))} lamports${C.reset}`);
  console.log(`  │  ${C.yellow}${stakedPct}%${C.reset} of total supply`);
  console.log(`  ${C.dim}└${C.reset}`);
  console.log();

  if (burned > 0) {
    console.log(`  ${C.bright}┌─ ${C.red}BURNED / IRRECOVERABLE${C.reset}`);
    console.log(`  │  ${C.red}${formatAethFull(burned)}${C.reset}`);
    console.log(`  │  ${C.dim}${formatLargeNum(Number(burned))} lamports${C.reset}`);
    console.log(`  │  ${C.red}${burnedPct}%${C.reset} of total supply`);
    console.log(`  ${C.dim}└${C.reset}`);
    console.log();
  }

  if (nonCirculating > 0) {
    console.log(`  ${C.bright}┌─ ${C.magenta}NON-CIRCULATING (LOCKED/ESCROW)${C.reset}`);
    console.log(`  │  ${C.magenta}${formatAethFull(nonCirculating)}${C.reset}`);
    console.log(`  │  ${C.dim}${formatLargeNum(Number(nonCirculating))} lamports${C.reset}`);
    console.log(`  ${C.dim}└${C.reset}`);
    console.log();
  }

  // Visual bar
  const barLen = 40;
  const circBars = Math.round((Number(circulating) / Number(total)) * barLen);
  const stakedBars = Math.round((Number(staked) / Number(total)) * barLen);
  const burnedBars = Math.round((Number(burned) / Number(total)) * barLen);
  const nonCircBars = Math.round((Number(nonCirculating) / Number(total)) * barLen);

  console.log(`  ${C.dim}Supply breakdown bar (per ${barLen} units):${C.reset}`);
  const bar = [
    C.green + '█'.repeat(Math.min(circBars, barLen)) + C.reset,
    C.yellow + '█'.repeat(Math.min(stakedBars, Math.max(0, barLen - circBars))) + C.reset,
    C.red + '█'.repeat(Math.min(burnedBars, Math.max(0, barLen - circBars - stakedBars))) + C.reset,
  ].join('');
  console.log(`  ${bar}`);
  console.log(`  ${C.green}■ circulating${C.reset}  ${C.yellow}■ staked${C.reset}  ${C.red}■ burned${C.reset}`);
  console.log();
}

/**
 * Compute and display supply metrics.
 */
async function showSupply(rpc, opts) {
  const { asJson, verbose } = opts;

  console.error(`${C.dim}Fetching supply data from ${rpc}...${C.reset}`);

  // Fetch all supply components in parallel
  const [totalData, staked, burned, nonCirc] = await Promise.all([
    fetchTotalSupply(rpc),
    fetchStakedSupply(rpc),
    fetchBurnedSupply(rpc),
    fetchNonCirculatingAccounts(rpc),
  ]);

  if (!totalData) {
    const msg = `Failed to fetch supply data from ${rpc}. Ensure your node is running or set AETHER_RPC.`;
    if (asJson) {
      console.log(JSON.stringify({ error: msg, rpc }, null, 2));
    } else {
      console.log(`\n${C.red}✗ ${msg}${C.reset}\n`);
    }
    process.exit(1);
  }

  const { total, circulating, nonCirculating: ncFromSupply, source } = totalData;
  // Use chain non-circulating if available, otherwise fall back to computed value
  const nonCirculating = ncFromSupply > 0 ? ncFromSupply : nonCirc;

  if (asJson) {
    const out = {
      rpc,
      source,
      supply: {
        total: total.toString(),
        total_formatted: formatAethFull(total),
        circulating: circulating.toString(),
        circulating_formatted: formatAethFull(circulating),
        non_circulating: nonCirculating.toString(),
        non_circulating_formatted: formatAethFull(nonCirculating),
        staked: staked.toString(),
        staked_formatted: formatAethFull(staked),
        burned: burned.toString(),
        burned_formatted: formatAethFull(burned),
        percentages: {
          circulating_pct: total > 0 ? ((Number(circulating) / Number(total)) * 100).toFixed(2) : '0',
          staked_pct: total > 0 ? ((Number(staked) / Number(total)) * 100).toFixed(2) : '0',
          burned_pct: total > 0 ? ((Number(burned) / Number(total)) * 100).toFixed(4) : '0',
        },
      },
      fetched_at: new Date().toISOString(),
    };
    console.log(JSON.stringify(out, null, 2));
    return;
  }

  renderSupplyTable({
    total,
    circulating,
    staked,
    burned,
    nonCirculating,
    rpc,
    source,
  });

  if (verbose) {
    console.log(`  ${C.dim}Notes:${C.reset}`);
    console.log(`  ${C.dim}  - Circulating = total - non-circulating (locked/escrow)${C.reset}`);
    console.log(`  ${C.dim}  - Staked supply reflects tokens in active stake accounts${C.reset}`);
    console.log(`  ${C.dim}  - Burned supply reflects tokens sent to irrecoverable addresses${C.reset}`);
    console.log(`  ${C.dim}  - Percentages calculated against total supply${C.reset}`);
    console.log(`  ${C.dim}  - Source: ${source}${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// CLI arg parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  return process.argv.slice(3); // [node, index.js, supply, ...]
}

async function main() {
  const args = parseArgs();

  let rpc = getDefaultRpc();
  let asJson = false;
  let verbose = false;

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--rpc' || args[i] === '-r') && args[i + 1]) {
      rpc = args[++i];
    } else if (args[i] === '--json' || args[i] === '-j') {
      asJson = true;
    } else if (args[i] === '--verbose' || args[i] === '-v') {
      verbose = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      console.log(`
${C.cyan}Usage:${C.reset}
  aether supply              Show Aether token supply overview
  aether supply --json      JSON output for scripting/monitoring
  aether supply --rpc <url> Query a specific RPC endpoint
  aether supply --verbose  Show detailed breakdown and notes

${C.dim}Examples:${C.reset}
  aether supply
  aether supply --json --rpc https://mainnet.aether.io
  AETHER_RPC=https://backup-rpc.example.com aether supply --verbose
`);
      return;
    }
  }

  await showSupply(rpc, { asJson, verbose });
}

main().catch(err => {
  console.error(`${C.red}Error:${C.reset} ${err.message}\n`);
  process.exit(1);
});

module.exports = { supplyCommand: main };
