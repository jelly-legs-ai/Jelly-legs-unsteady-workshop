/**
 * AETHER Testnet Genesis Generator
 * 
 * Generates the genesis block for local testnet deployment.
 * Creates bootstrap validator identities and initial chain configuration.
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

// Chain ID for testnet
const CHAIN_ID = 'aether-testnet-1';

// Genesis timestamp (Unix epoch seconds) - defaults to now
const GENESIS_TIMESTAMP = Math.floor(Date.now() / 1000);

// Slot time in milliseconds (400ms = 2.5 slots per second)
const SLOT_TIME_MS = 400;

// Slots per epoch (432,000 slots ≈ 2 days at 400ms per slot)
const SLOTS_PER_EPOCH = 432_000;

// Minimum stake for validators
const MIN_STAKE = 100;

// Bootstrap period rewards multiplier
const BOOTSTRAP_MULTIPLIER = 2.0;

/**
 * Generate Ed25519 keypair
 */
function generateKeypair() {
  const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519');
  return {
    publicKey: publicKey.export({ type: 'spki', format: 'der' }),
    privateKey: privateKey.export({ type: 'pkcs8', format: 'der' }),
  };
}

/**
 * Generate a validator identity
 */
function generateValidatorIdentity() {
  const keypair = generateKeypair();
  const pubkeyBase58 = base58Encode(keypair.publicKey);
  
  return {
    pubkey: pubkeyBase58,
    privkey: base58Encode(keypair.privateKey),
  };
}

/**
 * Simple base58 encoding
 */
function base58Encode(buffer) {
  const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let result = '';
  let num = BigInt('0x' + buffer.toString('hex'));
  
  while (num > 0n) {
    const div = num / 58n;
    const rem = num % 58n;
    num = div;
    result = alphabet[Number(rem)] + result;
  }
  
  // Handle leading zeros
  for (const byte of buffer) {
    if (byte === 0) {
      result = '1' + result;
    } else {
      break;
    }
  }
  
  return result || '1';
}

/**
 * Generate genesis hash from configuration
 */
function generateGenesisHash(timestamp, chainId) {
  const hash = crypto.createHash('sha256');
  hash.update(Buffer.from('aether-genesis-v1'));
  hash.update(Buffer.from(timestamp.toString()));
  hash.update(Buffer.from(chainId));
  return hash.digest('hex');
}

/**
 * Create bootstrap validator entry
 */
function createBootstrapValidator(identity, stake, commission) {
  return {
    identity_pubkey: identity.pubkey,
    activated_stake: stake,
    commission: commission,
    active: true,
  };
}

/**
 * Generate the complete genesis block
 */
function generateGenesis() {
  console.log('Generating AETHER testnet genesis...\n');
  
  // Generate bootstrap validator identities
  console.log('Creating bootstrap validator identities...');
  
  const bootstrapCount = parseInt(process.env.BOOTSTRAP_VALIDATORS || '1');
  const bootstrapValidators = [];
  
  for (let i = 0; i < bootstrapCount; i++) {
    const identity = generateValidatorIdentity();
    
    // Save identity to file
    const identityFile = path.join(__dirname, `bootstrap-validator-${i + 1}-identity.json`);
    fs.writeFileSync(identityFile, JSON.stringify({
      pubkey: identity.pubkey,
      privkey: identity.privkey,
    }, null, 2));
    console.log(`  ✓ Created ${identityFile}`);
    
    // Add to genesis
    bootstrapValidators.push(createBootstrapValidator(
      identity,
      10_000_000, // 10M AETH initial stake
      10          // 10% commission
    ));
  }
  
  // Generate genesis hash
  const genesisHash = generateGenesisHash(GENESIS_TIMESTAMP, CHAIN_ID);
  
  // Build genesis block
  const genesis = {
    chain_id: CHAIN_ID,
    genesis_hash: genesisHash,
    timestamp: GENESIS_TIMESTAMP,
    slot_time_ms: SLOT_TIME_MS,
    slots_per_epoch: SLOTS_PER_EPOCH,
    min_stake: MIN_STAKE,
    bootstrap_multiplier: BOOTSTRAP_MULTIPLIER,
    consensus: {
      mode: 'aetherflow',
      tower_finality: 12,
      poh_target_ticks_per_sec: 2500,
      poh_ticks_per_slot: 64,
    },
    bootstrap_validators: bootstrapValidators,
    rewards: {
      epoch_duration: SLOTS_PER_EPOCH,
      base_reward_rate: 6, // 6% APY
      bootstrap_bonus: 100, // Extra rewards during bootstrap
    },
    network: {
      p2p_port: 8001,
      rpc_port: 8899,
      gossip_port: 8000,
    },
    initial_accounts: [
      // faucet account for testing
      {
        pubkey: 'Faucet1111111111111111111111111111111111111',
        balance: 1_000_000_000_000, // 1M AETH
      },
    ],
  };
  
  // Save genesis block
  const genesisFile = path.join(__dirname, 'genesis.json');
  fs.writeFileSync(genesisFile, JSON.stringify(genesis, null, 2));
  console.log(`\n✓ Created ${genesisFile}`);
  
  // Save genesis hash for easy reference
  const hashFile = path.join(__dirname, 'GENESIS_HASH.txt');
  fs.writeFileSync(hashFile, genesisHash);
  console.log(`✓ Created ${hashFile}`);
  
  console.log('\n' + '='.repeat(60));
  console.log('GENESIS GENERATION COMPLETE');
  console.log('='.repeat(60));
  console.log(`\nChain ID:     ${CHAIN_ID}`);
  console.log(`Genesis Hash: ${genesisHash}`);
  console.log(`Timestamp:    ${GENESIS_TIMESTAMP}`);
  console.log(`Bootstrap:    ${bootstrapCount} validators`);
  console.log('\nTo start local testnet:');
  console.log('  1. Copy genesis.json to your validator\'s ledger directory');
  console.log('  2. Copy bootstrap validator identities to each node');
  console.log('  3. Start bootstrap validator:');
  console.log('     cargo run --bin aether-validator -- start --testnet');
  console.log('\n');
  
  return genesis;
}

// Run if called directly
if (require.main === module) {
  generateGenesis();
}

module.exports = { generateGenesis, CHAIN_ID };
