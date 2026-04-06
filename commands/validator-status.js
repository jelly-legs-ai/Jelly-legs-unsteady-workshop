/**
 * aether-cli validator-status
 * 
 * Queries the validator's RPC endpoint and displays status information.
 * Shows slot height, peer count, block production, and epoch info.
 * 
 * Uses @jellylegsai/aether-sdk for real blockchain RPC calls.
 */

const path = require('path');

// Import SDK for real blockchain RPC calls
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
  dim: '\x1b[2m',
};

/**
 * Create SDK client
 */
function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

/**
 * Parse command line args
 */
function parseArgs() {
  const args = process.argv.slice(3); // Skip 'aether-cli validator status'
  
  const options = {
    rpcUrl: 'http://127.0.0.1:8899',
    details: false,
    json: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--rpc-url':
        options.rpcUrl = args[++i];
        break;
      case '-v':
      case '--verbose':
      case '--details':
        options.details = true;
        break;
      case '--json':
        options.json = true;
        break;
    }
  }

  return options;
}

/**
 * Print the status display
 */
function printStatus(status, options) {
  const epochProgress = options.slotsInEpoch > 0 
    ? ((options.slotIndex / options.slotsInEpoch) * 100).toFixed(1)
    : '0.0';

  console.log();
  console.log(`${colors.cyan}╔═══════════════════════════════════════════════════════════════╗`);
  console.log(`${colors.cyan}║                                                               ║`);
  console.log(`${colors.cyan}║   ${colors.bright}AETHER VALIDATOR STATUS${colors.reset}${colors.cyan}                                  ║`);
  console.log(`${colors.cyan}║                                                               ║`);
  console.log(`${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}`);
  console.log();
  
  console.log(`  ${colors.bright}🌐 RPC Endpoint:${colors.reset}  ${options.rpcUrl}`);
  console.log();
  
  // Check if connected
  if (status.error) {
    console.log(`  ${colors.red}❌ Error:${colors.reset}  ${status.error}`);
    console.log();
    console.log(`  ${colors.yellow}Make sure the validator is running:${colors.reset}`);
    console.log(`    ${colors.bright}aether-cli validator start${colors.reset}`);
    console.log();
    return;
  }
  
  console.log(`  ${colors.green}✅ Connected${colors.reset}`);
  console.log();
  
  console.log(`  ${colors.bright}📊 Chain Status${colors.reset}`);
  console.log(`  ${colors.dim}   ${'─'.repeat(55)}${colors.reset}`);
  console.log(`     Slot Height:        ${colors.bright}${String(status.slot || 0).padStart(12)}${colors.reset}`);
  console.log(`     Block Height:       ${colors.bright}${String(status.blockHeight || 0).padStart(12)}${colors.reset}`);
  console.log(`     Transaction Count:  ${colors.bright}${String(status.transactionCount || 0).padStart(12)}${colors.reset}`);
  console.log();
  
  console.log(`  ${colors.bright}🔗 Network${colors.reset}`);
  console.log(`  ${colors.dim}   ${'─'.repeat(55)}${colors.reset}`);
  console.log(`     Peer Count:         ${colors.bright}${String(status.peerCount || 0).padStart(12)}${colors.reset}`);
  console.log();
  
  console.log(`  ${colors.bright}📈 Epoch ${status.epoch || 0}${colors.reset}`);
  console.log(`  ${colors.dim}   ${'─'.repeat(55)}${colors.reset}`);
  console.log(`     Progress:          ${colors.bright}${String(epochProgress + '%').padStart(12)}${colors.reset}`);
  console.log(`     Slot Index:         ${colors.bright}${String(options.slotIndex || 0).padStart(12)}${colors.reset}`);
  console.log(`     Slots in Epoch:     ${colors.bright}${String(options.slotsInEpoch || 0).padStart(12)}${colors.reset}`);
  console.log();
  
  if (options.details && status.blockProduction) {
    const bp = status.blockProduction;
    console.log(`  ${colors.bright}📦 Block Production${colors.reset}`);
    console.log(`  ${colors.dim}   ${'─'.repeat(55)}${colors.reset}`);
    console.log(`     Blocks Produced:   ${colors.bright}${String(bp.blocksProduced || 0).padStart(12)}${colors.reset}`);
    console.log(`     Entries Produced:   ${colors.bright}${String(bp.entriesProduced || 0).padStart(12)}${colors.reset}`);
    console.log();
  }
  
  console.log(`  ${colors.green}✓ Validator is running normally${colors.reset}`);
  console.log();
}

/**
 * Main status command
 */
async function validatorStatus() {
  const options = parseArgs();
  
  let status = {
    slot: 0,
    blockHeight: 0,
    transactionCount: 0,
    peerCount: 0,
    epoch: 0,
  };
  
  let epochInfo = {};
  let blockProduction = {};

  const client = createClient(options.rpcUrl);

  try {
    // Make parallel RPC calls using SDK
    const [slotResult, blockHeightResult, epochInfoResult, peersResult] = await Promise.all([
      client.getSlot().catch(e => ({ error: e.message })),
      client.getBlockHeight().catch(e => ({ error: e.message })),
      client.getEpochInfo().catch(e => ({})),
      client.getClusterPeers().catch(e => ([])),
    ]);

    if (typeof slotResult === 'object' && slotResult.error) {
      if (options.json) {
        console.log(JSON.stringify({ error: slotResult.error }, null, 2));
        process.exit(1);
      }
      console.log();
      console.log(`  ${colors.red}❌ Cannot connect to validator${colors.reset}`);
      console.log(`     ${colors.yellow}${slotResult.error}${colors.reset}`);
      console.log();
      console.log(`  ${colors.bright}Start the validator first:${colors.reset}`);
      console.log(`    ${colors.cyan}aether-cli validator start${colors.reset}`);
      console.log();
      process.exit(1);
    }

    status.slot = typeof slotResult === 'number' ? slotResult : (slotResult.slot || 0);
    status.blockHeight = typeof blockHeightResult === 'number' ? blockHeightResult : status.slot;
    status.transactionCount = 0; // Transaction count not available via SDK
    status.peerCount = Array.isArray(peersResult) ? peersResult.length : 0;
    
    if (epochInfoResult && typeof epochInfoResult === 'object') {
      epochInfo = epochInfoResult;
      status.epoch = epochInfo.epoch || 0;
      epochInfo.slotIndex = epochInfo.slotIndex || epochInfo.slot_index || 0;
      epochInfo.slotsInEpoch = epochInfo.slotsInEpoch || epochInfo.slots_in_epoch || 432000;
    }
    
    if (options.details) {
      try {
        blockProduction = await client.getSlotProduction();
      } catch { /* Block production not available */ }
    }

    if (options.json) {
      console.log(JSON.stringify({
        slot: status.slot,
        blockHeight: status.blockHeight,
        transactionCount: status.transactionCount,
        peerCount: status.peerCount,
        epoch: status.epoch,
        epochInfo,
        blockProduction,
      }, null, 2));
    } else {
      printStatus(status, { 
        ...options, 
        ...epochInfo, 
        blockProduction,
      });
    }

  } catch (err) {
    if (options.json) {
      console.log(JSON.stringify({ error: err.message }, null, 2));
    } else {
      console.log();
      console.log(`  ${colors.red}❌ Error querying validator${colors.reset}`);
      console.log(`     ${colors.yellow}${err.message}${colors.reset}`);
      console.log();
    }
    process.exit(1);
  }
}

// Export for use as module
module.exports = { validatorStatus };

// Run if called directly
if (require.main === module) {
  validatorStatus();
}
