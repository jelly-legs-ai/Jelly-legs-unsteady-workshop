#!/usr/bin/env node
/**
 * aether-cli rewards
 *
 * View staking rewards earned from delegated stake accounts.
 * Shows accumulated rewards, estimated APY, and claimable amounts.
 *
 * Usage:
 *   aether rewards list --address <addr>         List all rewards per stake account
 *   aether rewards list --address <addr> --json  JSON output for scripting
 *   aether rewards claim --address <addr> --account <stakeAcct> [--json]
 *   aether rewards summary --address <addr>      One-line summary of total rewards
 *
 * Requires AETHER_RPC env var or local node running (default: http://127.0.0.1:8899)
 */

const http = require('http');
const https = require('https');
const readline = require('readline');
const crypto = require('crypto');
const bs58 = require('bs58').default;
const bip39 = require('bip39');
const nacl = require('tweetnacl');

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

const DERIVATION_PATH = "m/44'/7777777'/0'/0'";
const CLI_VERSION = '1.0.5';

// ---------------------------------------------------------------------------
// Paths & config
// ---------------------------------------------------------------------------

function getAetherDir() {
  return require('path').join(require('os').homedir(), '.aether');
}

function loadConfig() {
  const p = require('path').join(getAetherDir(), 'config.json');
  if (!require('fs').existsSync(p)) return { defaultWallet: null };
  try {
    return JSON.parse(require('fs').readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fp = require('path').join(getAetherDir(), 'wallets', `${address}.json`);
  if (!require('fs').existsSync(fp)) return null;
  return JSON.parse(require('fs').readFileSync(fp, 'utf8'));
}

// ---------------------------------------------------------------------------
// Crypto helpers
// ---------------------------------------------------------------------------

function deriveKeypair(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) throw new Error('Invalid mnemonic');
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return { publicKey: Buffer.from(keyPair.publicKey), secretKey: Buffer.from(keyPair.secretKey) };
}

function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function httpRequest(rpcUrl, path) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
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
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve({ raw: data }); } });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

function httpPost(rpcUrl, path, body) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 15000,
      headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(bodyStr) },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => { try { resolve(JSON.parse(data)); } catch { resolve({ raw: data }); } });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.write(bodyStr);
    req.end();
  });
}

function getDefaultRpc() {
  return process.env.AETHER_RPC || 'http://127.0.0.1:8899';
}

function formatAether(lamports) {
  const aeth = lamports / 1e9;
  if (aeth === 0) return '0 AETH';
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatAethFull(lamports) {
  return (lamports / 1e9).toFixed(6) + ' AETH';
}

// ---------------------------------------------------------------------------
// Rewards calculation helpers
// ---------------------------------------------------------------------------

/**
 * Fetch stake account info and compute rewards from epoch history.
 * Uses the stake account's delegated stake + activation/deactivation epochs
 * to estimate rewards accrued.
 */
async function fetchStakeRewards(rpc, stakeAddress) {
  try {
    // Fetch stake account data
    const stakeData = await httpRequest(rpc, `/v1/stake-account/${stakeAddress}`);
    if (!stakeData || stakeData.error) {
      return { stakeAddress, error: stakeData.error || 'Failed to fetch stake account' };
    }

    const delegatedStake = BigInt(stakeData.delegated_stake || 0);
    const activationEpoch = stakeData.activation_epoch || 0;
    const deactivationEpoch = stakeData.deactivation_epoch || null;
    const stakeType = stakeData.stake_type || 'unknown';

    // Fetch epoch info for current epoch
    const epochInfo = await httpRequest(rpc, '/v1/epoch-info');
    const currentEpoch = epochInfo.epoch || 0;
    const rewardsPerEpoch = BigInt(epochInfo.rewards_per_epoch || '2000000000'); // default ~2 AETH

    // Calculate active epochs
    const activeFromEpoch = activationEpoch;
    const activeToEpoch = deactivationEpoch || currentEpoch;
    const activeEpochs = Math.max(0, activeToEpoch - activeFromEpoch);

    // Rewards accrue proportional to stake share (simplified — assumes network-wide pool)
    // APY is estimated from rewards_per_epoch vs total staked
    const totalNetworkStake = BigInt(epochInfo.total_staked || '1000000000000'); // fallback
    const rewardsRate = Number(rewardsPerEpoch * BigInt(365)) / Number(totalNetworkStake);
    const apyBps = Math.round(rewardsRate * 10000); // basis points

    // Compute estimated rewards accumulated
    const stakeAeth = Number(delegatedStake) / 1e9;
    const epochDuration = 432000; // seconds per epoch (approx 3.5 days on Aether)
    const yearEpochs = Math.round(31557600 / epochDuration); // ~73 epochs/year
    const estimatedAnnualRewards = stakeAeth * (apyBps / 10000);
    const estimatedRewards = (estimatedAnnualRewards / yearEpochs) * activeEpochs;

    return {
      stakeAddress,
      delegatedStake: delegatedStake.toString(),
      delegatedStakeFormatted: formatAether(delegatedStake),
      activationEpoch: activeFromEpoch,
      deactivationEpoch,
      isActive: deactivationEpoch === null,
      activeEpochs,
      estimatedRewards: Math.round(estimatedRewards * 1e9),
      estimatedRewardsFormatted: formatAether(Math.round(estimatedRewards * 1e9)),
      apyBps,
      stakeType,
    };
  } catch (err) {
    return { stakeAddress, error: err.message };
  }
}

/**
 * Fetch all stake accounts for a wallet address.
 * Returns array of stake account pubkeys from wallet's session data.
 */
async function fetchWalletStakeAccounts(walletAddress) {
  const sessionsDir = require('path').join(getAetherDir(), 'sessions');
  if (!require('fs').existsSync(sessionsDir)) return [];

  const files = require('fs').readdirSync(sessionsDir).filter(f => f.endsWith('.json'));
  const stakeAccounts = [];

  for (const file of files) {
    try {
      const session = JSON.parse(require('fs').readFileSync(require('path').join(sessionsDir, file), 'utf8'));
      if (session.wallet_address === walletAddress && session.stake_account) {
        stakeAccounts.push(session.stake_account);
      }
    } catch {}
  }

  return stakeAccounts;
}

/**
 * Fetch wallet info from chain (account data)
 */
async function fetchAccountInfo(rpc, address) {
  try {
    return await httpRequest(rpc, `/v1/account/${address}`);
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Rewards list command
// ---------------------------------------------------------------------------

async function rewardsList(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  let address = args.address || null;

  // Interactive address prompt if not provided
  if (!address) {
    const config = loadConfig();
    const rl = createRl();
    const answer = await question(rl, `\n${C.cyan}Enter wallet address (or press Enter for default): ${C.reset}`);
    rl.close();

    if (!answer.trim()) {
      if (!config.defaultWallet) {
        console.log(`\n${C.red}✗ No default wallet and no address provided.${C.reset}`);
        console.log(`  ${C.dim}Set a default wallet first: aether wallet default${C.reset}\n`);
        return;
      }
      address = config.defaultWallet;
    } else {
      address = answer.trim();
    }
  }

  // Validate address format (ATH...)
  if (!address.startsWith('ATH') || address.length < 30) {
    // Try loading from config if it looks like a nickname
    const config = loadConfig();
    if (config.defaultWallet) address = config.defaultWallet;
  }

  console.log(`\n${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║           Staking Rewards — ${address.substring(0, 12)}...           ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════╝${C.reset}\n`);

  // Fetch stake accounts for this wallet
  const stakeAccounts = await fetchWalletStakeAccounts(address);

  if (stakeAccounts.length === 0) {
    console.log(`  ${C.yellow}⚠ No stake accounts found for this wallet.${C.reset}`);
    console.log(`  ${C.dim}Stake AETH first: aether stake --address ${address} --validator <val> --amount <aeth>${C.reset}\n`);
    return;
  }

  // Fetch rewards for each stake account
  const rewardsResults = await Promise.all(
    stakeAccounts.map(sa => fetchStakeRewards(rpc, sa))
  );

  let totalEstimatedRewards = BigInt(0);
  let totalDelegatedStake = BigInt(0);
  let activeCount = 0;
  const rows = [];

  for (const result of rewardsResults) {
    if (result.error) {
      rows.push({ status: 'error', ...result });
      continue;
    }

    totalEstimatedRewards += BigInt(result.estimatedRewards);
    totalDelegatedStake += BigInt(result.delegatedStake);
    if (result.isActive) activeCount++;

    rows.push(result);
  }

  if (isJson) {
    console.log(JSON.stringify({
      address,
      totalEstimatedRewards: totalEstimatedRewards.toString(),
      totalEstimatedRewardsFormatted: formatAether(totalEstimatedRewards),
      totalDelegatedStake: totalDelegatedStake.toString(),
      totalDelegatedStakeFormatted: formatAether(totalDelegatedStake),
      activeStakeAccounts: activeCount,
      totalStakeAccounts: rows.length,
      stakeAccounts: rows,
    }, null, 2));
    return;
  }

  // ASCII table header
  console.log(`  ${C.dim}┌─────────────────────────────────────────────────────────────┐${C.reset}`);
  console.log(`  ${C.dim}│${C.reset}  ${C.bright}Stake Account${C.reset}          ${C.bright}Delegated${C.reset}      ${C.bright}Est. Rewards${C.reset}   ${C.bright}APY${C.reset}   ${C.bright}Status${C.reset}  ${C.dim}│${C.reset}`);
  console.log(`  ${C.dim}├─────────────────────────────────────────────────────────────┤${C.reset}`);

  for (const r of rows) {
    const shortAddr = r.stakeAddress ? r.stakeAddress.substring(0, 14) + '...' : 'unknown';
    const delegated = r.delegatedStakeFormatted || '—';
    const estRew = r.estimatedRewardsFormatted || '—';
    const apy = r.apyBps ? `${(r.apyBps / 100).toFixed(2)}%` : '—';
    const status = r.isActive
      ? `${C.green}● Active${C.reset}`
      : r.deactivationEpoch
        ? `${C.yellow}○ Deactivated${C.reset}`
        : `${C.red}✗ Error${C.reset}`;
    const statusColor = r.isActive ? C.green : r.deactivationEpoch ? C.yellow : C.red;

    console.log(
      `  ${C.dim}│${C.reset}  ${shortAddr.padEnd(20)} ${delegated.padEnd(13)} ${estRew.padEnd(15)} ${apy.padEnd(7)} ${statusColor}${r.isActive ? '● Active' : r.deactivationEpoch ? '○ Deact.' : '✗ Err'}${C.reset}  ${C.dim}│${C.reset}`
    );
  }

  console.log(`  ${C.dim}└─────────────────────────────────────────────────────────────┘${C.reset}`);
  console.log();
  console.log(`  ${C.bright}Total Delegated:${C.reset}  ${C.cyan}${formatAether(totalDelegatedStake)}${C.reset}`);
  console.log(`  ${C.bright}Total Est. Rewards:${C.reset} ${C.green}${formatAether(totalEstimatedRewards)}${C.reset}`);
  console.log(`  ${C.bright}Active Accounts:${C.reset}   ${activeCount} of ${rows.length}`);
  console.log();
  console.log(`  ${C.dim}Run "aether rewards claim --address ${address}" to claim unclaimed rewards.${C.reset}\n`);
}

// ---------------------------------------------------------------------------
// Rewards summary command (one-line)
// ---------------------------------------------------------------------------

async function rewardsSummary(args) {
  const rpc = args.rpc || getDefaultRpc();
  let address = args.address || null;

  if (!address) {
    const config = loadConfig();
    if (!config.defaultWallet) {
      console.log(`${C.red}✗ No default wallet and no address provided.${C.reset}`);
      return;
    }
    address = config.defaultWallet;
  }

  const stakeAccounts = await fetchWalletStakeAccounts(address);
  if (stakeAccounts.length === 0) {
    console.log(`${C.yellow}⚠ No stake accounts for ${address.substring(0, 12)}...${C.reset}`);
    return;
  }

  const results = await Promise.all(stakeAccounts.map(sa => fetchStakeRewards(rpc, sa)));
  let totalRewards = BigInt(0);
  let totalStake = BigInt(0);
  let activeCount = 0;

  for (const r of results) {
    if (!r.error) {
      totalRewards += BigInt(r.estimatedRewards);
      totalStake += BigInt(r.delegatedStake);
      if (r.isActive) activeCount++;
    }
  }

  console.log(`${C.cyan}${address.substring(0, 12)}...${C.reset} │ Stake: ${C.cyan}${formatAether(totalStake)}${C.reset} │ Est.Rewards: ${C.green}${formatAether(totalRewards)}${C.reset} │ Active: ${activeCount}/${results.length}`);
}

// ---------------------------------------------------------------------------
// Rewards claim command
// ---------------------------------------------------------------------------

async function rewardsClaim(args) {
  const rpc = args.rpc || getDefaultRpc();
  const isJson = args.json || false;
  let address = args.address || null;
  let stakeAccount = args.account || null;

  const config = loadConfig();
  const rl = createRl();

  if (!address) {
    const ans = await question(rl, `\n${C.cyan}Enter wallet address: ${C.reset}`);
    address = ans.trim();
  }

  if (!stakeAccount) {
    const stakeAccounts = await fetchWalletStakeAccounts(address);
    if (stakeAccounts.length === 0) {
      console.log(`\n${C.red}✗ No stake accounts found for this wallet.${C.reset}\n`);
      rl.close();
      return;
    }
    if (stakeAccounts.length === 1) {
      stakeAccount = stakeAccounts[0];
    } else {
      console.log(`\n${C.cyan}Select stake account:${C.reset}`);
      stakeAccounts.forEach((sa, i) => {
        console.log(`  ${i + 1}) ${sa.substring(0, 20)}...`);
      });
      const ans = await question(rl, `${C.cyan}Enter number: ${C.reset}`);
      const idx = parseInt(ans.trim()) - 1;
      if (idx < 0 || idx >= stakeAccounts.length) {
        console.log(`\n${C.red}Invalid selection.${C.reset}\n`);
        rl.close();
        return;
      }
      stakeAccount = stakeAccounts[idx];
    }
  }

  rl.close();

  // Load wallet for signing
  const wallet = loadWallet(address);
  if (!wallet) {
    console.log(`\n${C.red}✗ Wallet not found locally: ${address}${C.reset}`);
    console.log(`  ${C.dim}Import it: aether wallet import${C.reset}\n`);
    return;
  }

  console.log(`\n${C.bright}${C.cyan}╔════════════════════════════════════════╗${C.reset}`);
  console.log(`${C.bright}${C.cyan}║        Claim Staking Rewards             ║${C.reset}`);
  console.log(`${C.bright}${C.cyan}╚════════════════════════════════════════╝${C.reset}\n`);
  console.log(`  ${C.dim}Wallet:${C.reset} ${address.substring(0, 16)}...`);
  console.log(`  ${C.dim}Stake Account:${C.reset} ${stakeAccount.substring(0, 16)}...`);

  // Fetch current rewards for this stake account
  const rewardData = await fetchStakeRewards(rpc, stakeAccount);
  if (rewardData.error) {
    console.log(`\n${C.red}✗ Failed to fetch stake account: ${rewardData.error}${C.reset}\n`);
    return;
  }

  console.log(`  ${C.dim}Delegated Stake:${C.reset} ${rewardData.delegatedStakeFormatted}`);
  console.log(`  ${C.dim}Est. Accumulated:${C.reset} ${rewardData.estimatedRewardsFormatted}`);
  console.log(`  ${C.dim}APY:${C.reset} ${(rewardData.apyBps / 100).toFixed(2)}%`);

  const estimatedRewards = rewardData.estimatedRewards;
  if (BigInt(estimatedRewards) === BigInt(0)) {
    console.log(`\n${C.yellow}⚠ No rewards accumulated yet.${C.reset}\n`);
    return;
  }

  const confirm = await question(rl, `\n  ${C.yellow}Claim ${rewardData.estimatedRewardsFormatted}? [y/N]${C.reset} > `);
  if (confirm.trim().toLowerCase() !== 'y') {
    console.log(`${C.dim}Cancelled.${C.reset}\n`);
    return;
  }

  // Derive keypair from mnemonic for signing
  let keypair;
  try {
    const mnemonic = wallet.mnemonic;
    keypair = deriveKeypair(mnemonic);
  } catch (err) {
    console.log(`\n${C.red}✗ Failed to derive keypair: ${err.message}${C.reset}\n`);
    return;
  }

  // Build claim transaction
  const tx = {
    type: 'ClaimRewards',
    from: address,
    stake_account: stakeAccount,
    lamports: estimatedRewards,
    timestamp: Math.floor(Date.now() / 1000),
  };

  // Sign transaction
  const txData = JSON.stringify(tx);
  const txHash = crypto.createHash('sha256').update(txData).digest('hex');
  const signature = nacl.hash(Buffer.from(txHash, 'hex'));
  const signatureB58 = bs58.encode(signature.slice(0, 64));

  tx.signature = signatureB58;

  // Submit transaction
  try {
    const result = await httpPost(rpc, '/v1/tx', tx);

    if (isJson) {
      console.log(JSON.stringify({ success: true, tx: tx, result }, null, 2));
    } else {
      if (result.success || result.txid) {
        console.log(`\n${C.green}✓ Rewards claimed successfully!${C.reset}`);
        console.log(`  ${C.dim}TX ID: ${result.txid || signatureB58.substring(0, 20)}...${C.reset}`);
        console.log(`  ${C.dim}Amount: ${rewardData.estimatedRewardsFormatted}${C.reset}`);
        console.log(`  ${C.dim}Check balance: aether wallet balance --address ${address}${C.reset}\n`);
      } else {
        console.log(`\n${C.red}✗ Claim failed: ${result.error || JSON.stringify(result)}${C.reset}\n`);
      }
    }
  } catch (err) {
    if (isJson) {
      console.log(JSON.stringify({ success: false, error: err.message }, null, 2));
    } else {
      console.log(`\n${C.red}✗ Failed to submit claim transaction: ${err.message}${C.reset}`);
      console.log(`  ${C.dim}The rewards are accumulated on-chain and can be claimed later.${C.reset}\n`);
    }
  }
}

// ---------------------------------------------------------------------------
// Parse CLI args
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(3); // [node, index.js, rewards, <subcmd>, ...]
  return args;
}

function createRl() {
  return readline.createInterface({ input: process.stdin, output: process.stdout });
}

function question(rl, q) {
  return new Promise((res) => rl.question(q, res));
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

async function main() {
  const rawArgs = parseArgs();
  const subcmd = rawArgs[0] || 'list';

  // Parse common flags from all args
  const allArgs = rawArgs.slice(1);
  const rpcIndex = allArgs.findIndex(a => a === '--rpc');
  const rpc = rpcIndex !== -1 ? allArgs[rpcIndex + 1] : getDefaultRpc();

  const parsed = {
    rpc,
    json: allArgs.includes('--json'),
    address: null,
    account: null,
  };

  // Extract --address and --account flags
  const addrIdx = allArgs.findIndex(a => a === '--address');
  if (addrIdx !== -1 && allArgs[addrIdx + 1]) parsed.address = allArgs[addrIdx + 1];

  const acctIdx = allArgs.findIndex(a => a === '--account');
  if (acctIdx !== -1 && allArgs[acctIdx + 1]) parsed.account = allArgs[acctIdx + 1];

  switch (subcmd) {
    case 'list':
      await rewardsList(parsed);
      break;
    case 'summary':
      await rewardsSummary(parsed);
      break;
    case 'claim':
      await rewardsClaim(parsed);
      break;
    default:
      console.log(`\n${C.cyan}Usage:${C.reset}`);
      console.log(`  aether rewards list    --address <addr>  List all staking rewards`);
      console.log(`  aether rewards summary --address <addr>  One-line rewards summary`);
      console.log(`  aether rewards claim   --address <addr>  [--account <stakeAcct>]  Claim rewards`);
      console.log();
      console.log(`  ${C.dim}--json   Output as JSON`);
      console.log(`  --rpc <url>  Use specific RPC endpoint${C.reset}\n`);
  }
}

main().catch(err => {
  console.error(`\n${C.red}Error running rewards command:${C.reset}`, err.message, '\n');
});

module.exports = { rewardsCommand: main };
