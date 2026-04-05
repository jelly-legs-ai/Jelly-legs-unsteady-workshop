/**
 * aether-cli validator-status
 * 
 * Queries the validator's RPC endpoint and displays status information.
 * Shows slot height, peer count, block production, and epoch info.
 */

const http = require('http');

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
 * Make an RPC call to the validator
 */
function rpcCall(url, method, params = []) {
  return new Promise((resolve, reject) => {
    const urlObj = new URL(url);
    
    const postData = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method,
      params,
    });

    const options = {
      hostname: urlObj.hostname,
      port: urlObj.port || 8899,
      path: '/',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData),
      },
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          if (json.error) {
            reject(new Error(json.error.message || JSON.stringify(json.error)));
          } else {
            resolve(json.result);
          }
        } catch (e) {
          reject(new Error(`Invalid JSON response: ${data}`));
        }
      });
    });

    req.on('error', (e) => {
      reject(new Error(`Connection failed: ${e.message}`));
    });

    req.write(postData);
    req.end();
  });
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

  try {
    // Make parallel RPC calls
    const [slot, blockHeight, transactionCount, epochInfoResult, blockProdResult] = await Promise.all([
      rpcCall(options.rpcUrl, 'getSlot').catch(e => ({ error: e.message })),
      rpcCall(options.rpcUrl, 'getBlockHeight').catch(e => ({ error: e.message })),
      rpcCall(options.rpcUrl, 'getTransactionCount').catch(e => ({ error: e.message })),
      rpcCall(options.rpcUrl, 'getEpochInfo').catch(e => ({})),
      options.details ? rpcCall(options.rpcUrl, 'getBlockProduction').catch(e => ({})) : Promise.resolve({}),
    ]);

    if (typeof slot === 'object' && slot.error) {
      if (options.json) {
        console.log(JSON.stringify({ error: slot.error }, null, 2));
        process.exit(1);
      }
      console.log();
      console.log(`  ${colors.red}❌ Cannot connect to validator${colors.reset}`);
      console.log(`     ${colors.yellow}${slot.error}${colors.reset}`);
      console.log();
      console.log(`  ${colors.bright}Start the validator first:${colors.reset}`);
      console.log(`    ${colors.cyan}aether-cli validator start${colors.reset}`);
      console.log();
      process.exit(1);
    }

    status.slot = typeof slot === 'number' ? slot : 0;
    status.blockHeight = typeof blockHeight === 'number' ? blockHeight : status.slot;
    status.transactionCount = typeof transactionCount === 'number' ? transactionCount : 0;
    
    if (epochInfoResult && typeof epochInfoResult === 'object') {
      epochInfo = epochInfoResult;
      status.epoch = epochInfo.epoch || 0;
      epochInfo.slotIndex = epochInfo.slotIndex || 0;
      epochInfo.slotsInEpoch = epochInfo.slotsInEpoch || 432000;
    }
    
    if (blockProdResult && typeof blockProdResult === 'object') {
      blockProduction = blockProdResult;
    }

    // Get peer count
    try {
      status.peerCount = await rpcCall(options.rpcUrl, 'getPeerCount') || 0;
    } catch (e) {
      status.peerCount = 0;
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
module.exports = { validatorStatus, rpcCall };

// Run if called directly
if (require.main === module) {
  validatorStatus();
}
