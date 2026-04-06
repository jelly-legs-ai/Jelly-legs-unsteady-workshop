#!/usr/bin/env node
/**
 * aether-cli claim
 *
 * Claim accumulated staking rewards for a wallet.
 * Fetches pending rewards from the chain and submits a claim transaction.
 * 
 * FULLY WIRED TO SDK - Uses @jellylegsai/aether-sdk for all blockchain calls.
 * No manual HTTP - all calls go through AetherClient with real RPC.
 *
 * Usage:
 *   aether claim --address <addr> [--json] [--rpc <url>]
 *   aether claim --address <addr> --dry-run
 *
 * SDK wired to: 
 *   - client.getStakePositions(address)  → GET /v1/stake/<addr>
 *   - client.getRewards(address)         → GET /v1/rewards/<addr>  
 *   - client.getSlot()                   → GET /v1/slot
 *   - client.sendTransaction(tx)         → POST /v1/transaction
 */

const path = require('path');
const readline = require('readline');
const crypto = require('crypto');
const bs58 = require('bs58').default;
const bip39 = require('bip39');
const nacl = require('tweetnacl');

// Import SDK for ALL blockchain RPC calls
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

const DERIVATION_PATH = "m/44'/7777777'/0'/0'";
const CLI_VERSION = '1.1.0';

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
// Paths & config
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(require('os').homedir(), '.aether');
}

function loadConfig() {
  const fs = require('fs');
  const p = path.join(getAetherDir(), 'config.json');
  if (!fs.existsSync(p)) return { defaultWallet: null };
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return { defaultWallet: null };
  }
}

function loadWallet(address) {
  const fs = require('fs');
  const fp = path.join(getAetherDir(), 'wallets', `${address}.json`);
  if (!fs.existsSync(fp)) return null;
  return JSON.parse(fs.readFileSync(fp, 'utf8'));
}

// ---------------------------------------------------------------------------
// Crypto helpers
// ---------------------------------------------------------------------------

function deriveKeypair(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) throw new Error('Invalid mnemonic');
  const seedBuffer = bip39.mnemonicToSeedSync(mnemonic, '');
  const seed32 = seedBuffer.slice(0, 32);
  const keyPair = nacl.sign.keyPair.fromSeed(seed32);
  return { 
    publicKey: Buffer.from(keyPair.publicKey), 
    secretKey: Buffer.from(keyPair.secretKey) 
  };
}

function formatAddress(publicKey) {
  return 'ATH' + bs58.encode(publicKey);
}

function signTransaction(tx, secretKey) {
  const txBytes = Buffer.from(JSON.stringify(tx));
  const sig = nacl.sign.detached(txBytes, secretKey);
  return bs58.encode(sig);
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

function formatAether(lamports) {
  if (!lamports || lamports === '0') return '0 AETH';
  const aeth = Number(lamports) / 1e9;
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatFlux(lamports) {
  if (!lamports) return '0 FLUX';
  const flux = Number(lamports) / 1e6;
  return flux.toFixed(2) + ' FLUX';
}

function shortPubkey(pubkey) {
  if (!pubkey || pubkey.length < 16) return pubkey || 'unknown';
  return pubkey.slice(0, 8) + '...' + pubkey.slice(-8);
}

// ---------------------------------------------------------------------------
// SDK Reward Fetching (REAL RPC CALLS)
// ---------------------------------------------------------------------------

async function fetchStakeRewards(rpcUrl, stakeAddress) {
  const client = createClient(rpcUrl);
  
  try {
    // Parallel SDK calls
    const [stakePositions, rewards] = await Promise.all([
      client.getStakePositions(stakeAddress).catch(() => []),
      client.getRewards(stakeAddress).catch(() => ({ total: 0, pending: 0 })),
    ]);

    const stakeData = stakePositions.find(s => 
      (s.pubkey || s.publicKey || s.account) === stakeAddress
    ) || stakePositions[0] || {};

    const delegatedStake = BigInt(stakeData.lamports || stakeData.stake_lamports || 0);
    const validator = stakeData.validator || stakeData.delegate || rewards.validator || 'unknown';
    const totalRewards = BigInt(rewards.total || rewards.pending_rewards || rewards.amount || 0);
    const pendingRewards = BigInt(rewards.pending || rewards.pending_rewards || 0);

    return {
      stakeAddress,
      delegatedStake: delegatedStake.toString(),
      delegatedStakeFormatted: formatAether(delegatedStake),
      totalRewards: totalRewards.toString(),
      pendingRewards: pendingRewards.toString(),
      totalRewardsFormatted: formatAether(totalRewards),
      pendingRewardsFormatted: formatFlux(pendingRewards),
      validator,
    };
  } catch (err) {
    return { stakeAddress, error: err.message };
  }
}

async function fetchWalletStakeAccounts(rpcUrl, walletAddress) {
  const client = createClient(rpcUrl);
  
  try {
    const rawAddr = walletAddress.startsWith('ATH') ? walletAddress.slice(3) : walletAddress;
    const stakePositions = await client.getStakePositions(rawAddr);
    
    if (!Array.isArray(stakePositions)) return [];
    
    return stakePositions.map(s => ({
      address: s.pubkey || s.publicKey || s.account,
      validator: s.validator || s.delegate,
      lamports: s.lamports || s.stake_lamports || 0,
      pendingRewards: s.pending_rewards || s.rewards || 0,
    })).filter(s => s.address);
  } catch (err) {
    return [];
  }
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const result = { address: null, json: false, dryRun: false, rpc: getDefaultRpc() };

  for (let i = 0; i < args.length; i++) {
    if ((args[i] === '--address' || args[i] === '-a') && args[i + 1]) {
      result.address = args[i + 1];
      i++;
    } else if (args[i] === '--json' || args[i] === '--json-output') {
      result.json = true;
    } else if (args[i] === '--rpc' && args[i + 1]) {
      result.rpc = args[i + 1];
      i++;
    } else if (args[i] === '--help' || args[i] === '-h') {
      result.help = true;
    } else if (args[i] === '--dry-run') {
      result.dryRun = true;
    }
  }

  return result;
}

function createRl() {
  return readline.createInterface({ input: process.stdin, output: process.stdout });
}

function question(rl, q) {
  return new Promise((res) => rl.question(q, res));
}

async function askMnemonic(rl, promptText) {
  console.log(`\n${C.cyan}${promptText}${C.reset}`);
  console.log(`${C.dim}Enter your 12 or 24-word passphrase, one space-separated line:${C.reset}`);
  const raw = await question(rl, `  > ${C.reset}`);
  return raw.trim().toLowerCase();
}

// ---------------------------------------------------------------------------
// Main Command - FULLY WIRED TO SDK
// ---------------------------------------------------------------------------

async function claimCommand() {
  const opts = parseArgs();
  const rl = createRl();

  if (opts.help) {
    console.log(`
${C.bright}${C.cyan}claim${C.reset} — Claim accumulated staking rewards for a wallet

${C.bright}USAGE${C.reset}
    aether claim --address <addr> [--json] [--rpc <url>] [--dry-run]

${C.bright}OPTIONS${C.reset}
    --address <addr>    Wallet address (ATH...)
    --json               Output raw JSON
    --rpc <url>          RPC endpoint (default: AETHER_RPC or localhost:8899)
    --dry-run            Preview claim without submitting transaction
    --help               Show this help

${C.bright}SDK METHODS USED${C.reset}
    client.getStakePositions(address)  → GET /v1/stake/<addr>
    client.getRewards(address)         → GET /v1/rewards/<addr>
    client.getSlot()                   → GET /v1/slot
    client.sendTransaction(tx)         → POST /v1/transaction

${C.bright}EXAMPLES${C.reset}
    aether claim --address ATH3abc...
    aether claim --address ATH3abc... --dry-run
    aether claim --address ATH3abc... --json
`);
    rl.close();
    return;
  }

  if (!opts.address) {
    // Try default wallet
    const config = loadConfig();
    if (config.defaultWallet) {
      opts.address = config.defaultWallet;
    } else {
      console.log(`  ${C.red}✗ Missing --address${C.reset}\n`);
      console.log(`  Usage: aether claim --address <addr> [--json] [--dry-run]\n`);
      rl.close();
      return;
    }
  }

  const rpcUrl = opts.rpc;
  const address = opts.address;
  const rawAddr = address.startsWith('ATH') ? address.slice(3) : address;

  if (!opts.json) {
    console.log(`\n${C.bright}${C.cyan}── Claim Staking Rewards ────────────────────────────────${C.reset}\n`);
    console.log(`  ${C.dim}Wallet:${C.reset} ${address}`);
    console.log(`  ${C.dim}RPC:   ${C.reset} ${rpcUrl}`);
    if (opts.dryRun) console.log(`  ${C.yellow}(dry-run mode - no transaction will be submitted)${C.reset}`);
    console.log();
  }

  try {
    // SDK call: Fetch stake positions via client.getStakePositions (REAL RPC)
    const client = createClient(rpcUrl);
    
    if (!opts.json) {
      console.log(`  ${C.dim}Fetching stake positions via SDK...${C.reset}`);
    }

    const stakeAccounts = await fetchWalletStakeAccounts(rpcUrl, address);

    if (!stakeAccounts || stakeAccounts.length === 0) {
      if (opts.json) {
        console.log(JSON.stringify({
          address,
          error: 'No active stake positions found',
          suggestion: 'Stake AETH first with: aether stake --validator <addr> --amount <aeth>',
        }, null, 2));
      } else {
        console.log(`  ${C.yellow}⚠ No active stake positions found.${C.reset}`);
        console.log(`  ${C.dim}  Stake AETH with: ${C.cyan}aether stake --validator <addr> --amount <aeth>${C.reset}\n`);
      }
      rl.close();
      return;
    }

    // Calculate total pending rewards using SDK
    let totalPendingRewards = BigInt(0);
    const rewardBreakdown = [];

    for (const acc of stakeAccounts) {
      // SDK call: getRewards for each stake account
      const rewardData = await fetchStakeRewards(rpcUrl, acc.address);
      
      if (!rewardData.error) {
        const pendingRewards = BigInt(rewardData.pendingRewards || 0);
        totalPendingRewards += pendingRewards;
        
        rewardBreakdown.push({
          stakeAcct: acc.address,
          validator: acc.validator || rewardData.validator || 'unknown',
          stakeLamports: acc.lamports || 0,
          pendingRewards: pendingRewards.toString(),
          pendingFormatted: rewardData.pendingRewardsFormatted,
        });
      }
    }

    if (!opts.json) {
      console.log(`  ${C.bright}Stake Positions (${stakeAccounts.length})${C.reset}\n`);

      for (const pos of rewardBreakdown) {
        const shortVal = shortPubkey(pos.validator);
        const shortAcct = shortPubkey(pos.stakeAcct);
        console.log(`  ${C.dim}├─ ${C.reset}${shortAcct} → ${C.cyan}${shortVal}${C.reset}`);
        console.log(`  │   ${C.dim}Staked:${C.reset} ${formatAether(pos.stakeLamports)}`);
        console.log(`  │   ${C.green}Pending:${C.reset} ${pos.pendingFormatted}\n`);
      }

      console.log(`  ${C.dim}────────────────────────────────────────${C.reset}`);
      console.log(`  ${C.bright}Total Pending Rewards:${C.reset} ${C.green}${formatFlux(totalPendingRewards.toString())}${C.reset}\n`);
    }

    // Dry run mode - don't submit
    if (opts.dryRun) {
      if (opts.json) {
        console.log(JSON.stringify({
          wallet_address: address,
          dry_run: true,
          stake_count: stakeAccounts.length,
          total_pending_flux: totalPendingRewards.toString(),
          total_pending_aeth: (Number(totalPendingRewards) / 1e9).toFixed(9),
          breakdown: rewardBreakdown,
          sdk_version: CLI_VERSION,
        }, null, 2));
      } else {
        console.log(`  ${C.yellow}⚠ Dry run - not submitting claim transaction${C.reset}\n`);
      }
      rl.close();
      return;
    }

    // Load wallet for signing
    const wallet = loadWallet(address);
    if (!wallet) {
      console.log(`  ${C.red}✗ Wallet not found locally: ${address}${C.reset}`);
      console.log(`  ${C.dim}Import it: aether wallet import${C.reset}\n`);
      rl.close();
      return;
    }

    // Step: Submit claim transaction
    if (!opts.json) {
      console.log(`  ${C.dim}Preparing claim transaction...${C.reset}`);
    }

    // Ask for mnemonic
    const mnemonic = await askMnemonic(rl, 'Enter your wallet passphrase to sign the claim');
    console.log();

    let keyPair;
    try {
      keyPair = deriveKeypair(mnemonic);
    } catch (e) {
      console.log(`  ${C.red}✗ Failed to derive keypair: ${e.message}${C.reset}\n`);
      rl.close();
      return;
    }

    // Verify derived address matches
    const derivedAddress = formatAddress(keyPair.publicKey);
    if (derivedAddress !== address) {
      console.log(`  ${C.red}✗ Passphrase mismatch.${C.reset}`);
      console.log(`  ${C.dim}  Derived:   ${derivedAddress}${C.reset}`);
      console.log(`  ${C.dim}  Expected:  ${address}${C.reset}`);
      console.log(`  ${C.dim}Check your passphrase and try again.${C.reset}\n`);
      rl.close();
      return;
    }

    // SDK call: get current slot
    const currentSlot = await client.getSlot().catch(() => 0);

    // Build claim transaction for SDK
    const tx = {
      signer: rawAddr,
      tx_type: 'ClaimRewards',
      payload: {
        type: 'ClaimRewards',
        data: {
          stake_accounts: rewardBreakdown.map(r => r.stakeAcct),
          lamports: totalPendingRewards.toString(),
        },
      },
      fee: 5000,
      slot: currentSlot,
      timestamp: Math.floor(Date.now() / 1000),
    };

    // Sign transaction
    tx.signature = signTransaction(tx, keyPair.secretKey);

    if (!opts.json) {
      console.log(`  ${C.dim}Submitting claim via SDK to ${rpcUrl}...${C.reset}`);
    }

    // SDK call: sendTransaction (REAL RPC POST /v1/transaction)
    const result = await client.sendTransaction(tx);

    if (opts.json) {
      console.log(JSON.stringify({
        wallet_address: address,
        success: !result.error,
        total_claimed_flux: totalPendingRewards.toString(),
        total_claimed_aeth: (Number(totalPendingRewards) / 1e9).toFixed(9),
        tx_signature: result.signature || result.txid || null,
        block_height: result.block_height || result.slot || currentSlot,
        slot: result.slot || currentSlot,
        claimed_at: new Date().toISOString(),
        sdk_version: CLI_VERSION,
      }, null, 2));
    } else {
      if (result.error) {
        console.log(`  ${C.red}✗ Claim failed:${C.reset} ${result.error}\n`);
        rl.close();
        process.exit(1);
      }

      console.log(`  ${C.green}✓ Rewards claimed!${C.reset}`);
      console.log(`  ${C.dim}  Amount:${C.reset} ${C.green}${formatFlux(result.claimed || totalPendingRewards.toString())}${C.reset}`);
      if (result.signature || result.txid) {
        console.log(`  ${C.dim}  Tx:${C.reset} ${shortPubkey(result.signature || result.txid)}`);
      }
      console.log(`  ${C.dim}  Slot:${C.reset} ${result.slot || currentSlot}`);
      console.log(`  ${C.dim}  SDK: sendTransaction()${C.reset}\n`);
    }

    rl.close();

  } catch (err) {
    if (opts.json) {
      console.log(JSON.stringify({ address, error: err.message, sdk_version: CLI_VERSION }, null, 2));
    } else {
      console.log(`  ${C.red}✗ Failed to claim rewards:${C.reset} ${err.message}\n`);
      console.log(`  ${C.dim}  Set custom RPC: AETHER_RPC=https://your-rpc-url${C.reset}\n`);
    }
    rl.close();
    process.exit(1);
  }
}

// Export for module use
module.exports = { claimCommand };

// Run if called directly
if (require.main === module) {
  claimCommand();
}
