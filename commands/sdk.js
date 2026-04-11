#!/usr/bin/env node
/**
 * aether-cli sdk
 *
 * Direct SDK access for developers - query any SDK method from the command line.
 * Makes REAL HTTP RPC calls to the Aether blockchain.
 * No stubs, no mocks - every function makes actual RPC calls.
 *
 * Usage:
 *   aether sdk getSlot                           Get current slot
 *   aether sdk getBalance <address>              Get account balance
 *   aether sdk getAccountInfo <address>          Get full account info
 *   aether sdk getEpochInfo                    Get epoch information
 *   aether sdk getBlockHeight                  Get block height
 *   aether sdk getTransaction <signature>      Get transaction by signature
 *   aether sdk getStakePositions <address>       Get stake positions
 *   aether sdk getRewards <address>              Get rewards info
 *   aether sdk getValidators                  List validators
 *   aether sdk getSupply                     Get token supply
 *   aether sdk getTPS                        Get transactions per second
 *   aether sdk getHealth                      Check node health
 *   aether sdk getVersion                    Get node version
 *   aether sdk getRecentBlockhash              Get recent blockhash
 *   aether sdk getSlotProduction               Get slot production stats
 *   aether sdk getClusterPeers                 Get peer nodes
 *   aether sdk getFees                       Get fee schedule
 *   aether sdk getNFT <id>                   Get NFT details
 *   aether sdk getNFTHoldings <address>       Get NFT holdings
 *   aether sdk getRecentTransactions <addr>    Get recent transactions
 *   aether sdk ping                           Ping RPC endpoint
 *   aether sdk --rpc <url>                    Use specific RPC endpoint
 *   aether sdk --json                           Output JSON format
 *   aether sdk --list                          List all available SDK methods
 *
 * SDK Methods Used:
 *   - All methods from @jellylegsai/aether-sdk
 *   - Real HTTP RPC calls to http://127.0.0.1:8899
 */

const path = require('path');
const readline = require('readline');

// Import SDK - REAL blockchain RPC calls
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

const CLI_VERSION = '1.2.0';

// ============================================================================
// SDK Client Setup
// ============================================================================

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpcUrl) {
  return new aether.AetherClient({ rpcUrl });
}

// ============================================================================
// Format Helpers
// ============================================================================

function formatAether(lamports) {
  if (!lamports || lamports === '0') return '0 AETH';
  const aeth = Number(lamports) / 1e9;
  return aeth.toFixed(4).replace(/\.?0+$/, '') + ' AETH';
}

function formatNumber(n) {
  if (n === null || n === undefined) return 'N/A';
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function shortAddress(addr) {
  if (!addr || addr.length < 16) return addr || 'unknown';
  return addr.slice(0, 8) + '...' + addr.slice(-8);
}

// ============================================================================
// SDK Method Definitions
// ============================================================================

const SDK_METHODS = {
  // Core chain queries
  getSlot: {
    desc: 'Get current slot number',
    args: [],
    returns: 'slot number',
    example: 'aether sdk getSlot',
  },
  getBlockHeight: {
    desc: 'Get current block height',
    args: [],
    returns: 'block height',
    example: 'aether sdk getBlockHeight',
  },
  getEpochInfo: {
    desc: 'Get epoch information',
    args: [],
    returns: 'epoch data',
    example: 'aether sdk getEpochInfo',
  },
  getHealth: {
    desc: 'Check node health',
    args: [],
    returns: 'health status',
    example: 'aether sdk getHealth',
  },
  getVersion: {
    desc: 'Get node version',
    args: [],
    returns: 'version info',
    example: 'aether sdk getVersion',
  },
  getTPS: {
    desc: 'Get transactions per second',
    args: [],
    returns: 'TPS number',
    example: 'aether sdk getTPS',
  },
  getSupply: {
    desc: 'Get token supply info',
    args: [],
    returns: 'supply data',
    example: 'aether sdk getSupply',
  },
  getFees: {
    desc: 'Get fee schedule',
    args: [],
    returns: 'fee info',
    example: 'aether sdk getFees',
  },
  getRecentBlockhash: {
    desc: 'Get recent blockhash',
    args: [],
    returns: 'blockhash',
    example: 'aether sdk getRecentBlockhash',
  },
  getSlotProduction: {
    desc: 'Get slot production stats',
    args: [],
    returns: 'production stats',
    example: 'aether sdk getSlotProduction',
  },
  getClusterPeers: {
    desc: 'Get cluster peers',
    args: [],
    returns: 'peers list',
    example: 'aether sdk getClusterPeers',
  },
  getValidators: {
    desc: 'Get validators list',
    args: [],
    returns: 'validators',
    example: 'aether sdk getValidators',
  },
  
  // Account queries
  getAccountInfo: {
    desc: 'Get account info',
    args: ['address'],
    returns: 'account data',
    example: 'aether sdk getAccountInfo ATHxxx...',
  },
  getBalance: {
    desc: 'Get account balance',
    args: ['address'],
    returns: 'balance in lamports',
    example: 'aether sdk getBalance ATHxxx...',
  },
  getStakePositions: {
    desc: 'Get stake positions',
    args: ['address'],
    returns: 'stake positions',
    example: 'aether sdk getStakePositions ATHxxx...',
  },
  getRewards: {
    desc: 'Get rewards info',
    args: ['address'],
    returns: 'rewards data',
    example: 'aether sdk getRewards ATHxxx...',
  },
  getStakeAccounts: {
    desc: 'Get all stake accounts',
    args: ['address'],
    returns: 'stake accounts',
    example: 'aether sdk getStakeAccounts ATHxxx...',
  },
  getTokenAccounts: {
    desc: 'Get token accounts',
    args: ['address'],
    returns: 'token accounts',
    example: 'aether sdk getTokenAccounts ATHxxx...',
  },
  getNFTHoldings: {
    desc: 'Get NFT holdings',
    args: ['address'],
    returns: 'NFT holdings',
    example: 'aether sdk getNFTHoldings ATHxxx...',
  },
  getNFTsByCreator: {
    desc: 'Get NFTs by creator',
    args: ['address'],
    returns: 'NFTs list',
    example: 'aether sdk getNFTsByCreator ATHxxx...',
  },
  
  // Transaction queries
  getTransaction: {
    desc: 'Get transaction by signature',
    args: ['signature'],
    returns: 'transaction data',
    example: 'aether sdk getTransaction SIGxxx...',
  },
  getRecentTransactions: {
    desc: 'Get recent transactions',
    args: ['address', 'limit?'],
    returns: 'transactions list',
    example: 'aether sdk getRecentTransactions ATHxxx...',
  },
  getTransactionHistory: {
    desc: 'Get full tx history with details',
    args: ['address', 'limit?'],
    returns: 'tx history',
    example: 'aether sdk getTransactionHistory ATHxxx...',
  },
  
  // NFT queries
  getNFT: {
    desc: 'Get NFT details',
    args: ['id'],
    returns: 'NFT data',
    example: 'aether sdk getNFT <id>',
  },
  
  // Utilities
  ping: {
    desc: 'Ping RPC endpoint',
    args: ['url?'],
    returns: 'ping result',
    example: 'aether sdk ping',
  },
};

// ============================================================================
// Argument Parsing
// ============================================================================

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    method: null,
    args: [],
    rpc: getDefaultRpc(),
    json: false,
    list: false,
    help: false,
  };

  // Check for --list
  if (args.includes('--list') || args.includes('-l')) {
    opts.list = true;
    return opts;
  }

  // Check for --help
  if (args.includes('--help') || args.includes('-h')) {
    opts.help = true;
    return opts;
  }

  // Parse method and arguments
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--rpc' || arg === '-r') {
      opts.rpc = args[++i];
    } else if (arg === '--json' || arg === '-j') {
      opts.json = true;
    } else if (arg === '--list' || arg === '-l') {
      opts.list = true;
    } else if (arg === '--help' || arg === '-h') {
      opts.help = true;
    } else if (!opts.method && !arg.startsWith('-')) {
      // First non-flag argument is the method
      opts.method = arg;
    } else if (opts.method && !arg.startsWith('-')) {
      // Subsequent non-flag arguments are method args
      opts.args.push(arg);
    }
  }

  return opts;
}

function showHelp() {
  console.log(`
${C.bright}${C.cyan}aether-cli sdk${C.reset} - Direct SDK access for developers

${C.bright}USAGE${C.reset}
    aether sdk <method> [args...] [options]

${C.bright}CORE CHAIN QUERIES${C.reset}
    getSlot                           Get current slot number
    getBlockHeight                  Get current block height
    getEpochInfo                    Get epoch information
    getHealth                       Check node health
    getVersion                    Get node version info
    getTPS                        Get transactions per second
    getSupply                     Get token supply
    getFees                        Get fee schedule
    getRecentBlockhash            Get recent blockhash
    getSlotProduction             Get slot production stats
    getClusterPeers              Get cluster peer nodes
    getValidators                List validators

${C.bright}ACCOUNT QUERIES${C.reset}
    getAccountInfo <address>       Get account info
    getBalance <address>            Get balance in lamports
    getStakePositions <address>      Get stake positions
    getRewards <address>            Get rewards info
    getStakeAccounts <address>       Get all stake accounts
    getTokenAccounts <address>       Get token accounts
    getNFTHoldings <address>        Get NFT holdings
    getNFTsByCreator <address>       Get NFTs by creator

${C.bright}TRANSACTION QUERIES${C.reset}
    getTransaction <signature>    Get transaction by signature
    getRecentTransactions <addr>    Get recent transactions
    getTransactionHistory <addr>    Get tx history with details

${C.bright}NFT QUERIES${C.reset}
    getNFT <id>                    Get NFT details

${C.bright}UTILITIES${C.reset}
    ping [url]                      Ping RPC endpoint
    --list                          List all SDK methods

${C.bright}OPTIONS${C.reset}
    --rpc <url>                      RPC endpoint (default: ${getDefaultRpc()})
    --json                          Output as JSON
    --help                          Show this help

${C.bright}EXAMPLES${C.reset}
    aether sdk getSlot
    aether sdk getBalance ATH3abc...
    aether sdk getAccountInfo ATH3abc... --json
    aether sdk getValidators --json
    aether sdk getStakePositions ATH3abc...
    aether sdk ping --rpc http://custom-rpc:8899
    AETHER_RPC=https://rpc.aether.network aether sdk getEpochInfo
`);
}

function showMethodList() {
  console.log(`\n${C.bright}${C.cyan}══ Available SDK Methods ══${C.reset}\n`);
  
  const categories = {
    'Core Chain': ['getSlot', 'getBlockHeight', 'getEpochInfo', 'getHealth', 'getVersion', 'getTPS', 'getSupply', 'getFees', 'getRecentBlockhash', 'getSlotProduction', 'getClusterPeers', 'getValidators'],
    'Account': ['getAccountInfo', 'getBalance', 'getStakePositions', 'getRewards', 'getStakeAccounts', 'getTokenAccounts', 'getNFTHoldings', 'getNFTsByCreator'],
    'Transaction': ['getTransaction', 'getRecentTransactions', 'getTransactionHistory'],
    'NFT': ['getNFT'],
    'Utility': ['ping'],
  };
  
  for (const [category, methods] of Object.entries(categories)) {
    console.log(`\n${C.bright}${C.cyan}${category}${C.reset}`);
    for (const method of methods) {
      const info = SDK_METHODS[method];
      if (info) {
        const argsStr = info.args.length > 0 ? ` <${info.args.join('> <')}>` : '';
        console.log(`  ${C.green}${method.padEnd(25)}${C.reset} ${C.dim}${info.desc}${C.reset}`);
        console.log(`    ${C.dim}Usage: ${info.example}${C.reset}`);
      }
    }
  }
  
  console.log(`\n${C.dim}All methods make REAL RPC calls to ${getDefaultRpc()}${C.reset}\n`);
}

// ============================================================================
// SDK Method Execution
// ============================================================================

async function executeMethod(opts) {
  const { method, args, rpc, json } = opts;
  
  if (!method) {
    throw new Error('No method specified. Use --list to see available methods.');
  }
  
  // Validate method exists
  if (!SDK_METHODS[method]) {
    throw new Error(`Unknown method: ${method}. Use --list to see available methods.`);
  }
  
  const client = createClient(rpc);
  const methodInfo = SDK_METHODS[method];
  
  // Validate required arguments
  if (methodInfo.args.length > args.length) {
    throw new Error(`Method ${method} requires ${methodInfo.args.length} argument(s): ${methodInfo.args.join(', ')}`);
  }
  
  // Execute the method
  let result;
  
  switch (method) {
    // Core chain queries
    case 'getSlot':
      result = await client.getSlot();
      break;
    case 'getBlockHeight':
      result = await client.getBlockHeight();
      break;
    case 'getEpochInfo':
      result = await client.getEpochInfo();
      break;
    case 'getHealth':
      result = await client.getHealth();
      break;
    case 'getVersion':
      result = await client.getVersion();
      break;
    case 'getTPS':
      result = await client.getTPS();
      break;
    case 'getSupply':
      result = await client.getSupply();
      break;
    case 'getFees':
      result = await client.getFees();
      break;
    case 'getRecentBlockhash':
      result = await client.getRecentBlockhash();
      break;
    case 'getSlotProduction':
      result = await client.getSlotProduction();
      break;
    case 'getClusterPeers':
      result = await client.getClusterPeers();
      break;
    case 'getValidators':
      result = await client.getValidators();
      break;
      
    // Account queries
    case 'getAccountInfo':
      result = await client.getAccountInfo(args[0]);
      break;
    case 'getBalance':
      result = await client.getBalance(args[0]);
      break;
    case 'getStakePositions':
      result = await client.getStakePositions(args[0]);
      break;
    case 'getRewards':
      result = await client.getRewards(args[0]);
      break;
    case 'getStakeAccounts':
      result = await client.getStakeAccounts(args[0]);
      break;
    case 'getTokenAccounts':
      result = await client.getTokenAccounts(args[0]);
      break;
    case 'getNFTHoldings':
      result = await client.getNFTHoldings(args[0]);
      break;
    case 'getNFTsByCreator':
      result = await client.getNFTsByCreator(args[0]);
      break;
      
    // Transaction queries
    case 'getTransaction':
      result = await client.getTransaction(args[0]);
      break;
    case 'getRecentTransactions':
      result = await client.getRecentTransactions(args[0], parseInt(args[1]) || 20);
      break;
    case 'getTransactionHistory':
      result = await client.getTransactionHistory(args[0], parseInt(args[1]) || 20);
      break;
      
    // NFT queries
    case 'getNFT':
      result = await client.getNFT(args[0]);
      break;
      
    // Utilities
    case 'ping':
      result = await aether.ping(args[0] || rpc);
      break;
      
    default:
      throw new Error(`Method ${method} not implemented in CLI`);
  }
  
  // Output result
  if (json) {
    console.log(JSON.stringify({
      method,
      args,
      rpc,
      result,
      timestamp: new Date().toISOString(),
      cli_version: CLI_VERSION,
    }, null, 2));
  } else {
    // Pretty output
    console.log(`\n${C.bright}${C.cyan}══ SDK Result: ${method} ══${C.reset}\n`);
    console.log(`  ${C.dim}RPC:${C.reset} ${rpc}`);
    console.log(`  ${C.dim}Method:${C.reset} ${method}`);
    if (args.length > 0) {
      console.log(`  ${C.dim}Args:${C.reset} ${args.join(' ')}`);
    }
    console.log();
    
    // Format result based on type
    if (typeof result === 'object') {
      // Special formatting for specific methods
      if (method === 'getBalance') {
        console.log(`  ${C.green}Balance:${C.reset} ${formatNumber(result)} lamports`);
        console.log(`  ${C.green}Formatted:${C.reset} ${formatAether(result)}`);
      } else if (method === 'getSlot' || method === 'getBlockHeight') {
        console.log(`  ${C.green}Result:${C.reset} ${formatNumber(result)}`);
      } else if (method === 'getTPS') {
        console.log(`  ${C.green}TPS:${C.reset} ${result}`);
      } else if (method === 'getHealth') {
        const status = result === 'ok' ? C.green : C.yellow;
        console.log(`  ${C.green}Status:${C.reset} ${status}${result}${C.reset}`);
      } else {
        // Generic object output
        console.log(`  ${C.green}Result:${C.reset}`);
        console.log(JSON.stringify(result, null, 2).split('\n').map(l => `  ${C.dim}${l}${C.reset}`).join('\n'));
      }
    } else if (Array.isArray(result)) {
      console.log(`  ${C.green}Count:${C.reset} ${result.length}`);
      if (result.length === 0) {
        console.log(`  ${C.dim}(empty array)${C.reset}`);
      } else {
        // Show first few items
        const preview = result.slice(0, 3);
        preview.forEach((item, i) => {
          if (typeof item === 'object') {
            const id = item.pubkey || item.address || item.signature || item.id || `Item ${i + 1}`;
            console.log(`  ${C.dim}[${i}]${C.reset} ${shortAddress(id)}`);
          } else {
            console.log(`  ${C.dim}[${i}]${C.reset} ${item}`);
          }
        });
        if (result.length > 3) {
          console.log(`  ${C.dim}... and ${result.length - 3} more${C.reset}`);
        }
      }
    } else {
      console.log(`  ${C.green}Result:${C.reset} ${result}`);
    }
    console.log();
    console.log(`  ${C.dim}SDK: @jellylegsai/aether-sdk v${CLI_VERSION}${C.reset}`);
  }
  
  return result;
}

// ============================================================================
// Interactive Mode
// ============================================================================

async function interactiveMode() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  
  console.log(`\n${C.bright}${C.cyan}══ Aether SDK Interactive Mode ══${C.reset}\n`);
  console.log(`${C.dim}Type a method name or 'help' for usage, 'exit' to quit.${C.reset}\n`);
  
  const ask = () => {
    return new Promise((resolve) => {
      rl.question(`${C.cyan}sdk>${C.reset} `, (answer) => {
        resolve(answer.trim());
      });
    });
  };
  
  while (true) {
    const input = await ask();
    
    if (input === 'exit' || input === 'quit') {
      console.log(`${C.dim}Goodbye!${C.reset}\n`);
      rl.close();
      break;
    }
    
    if (input === 'help' || input === '?') {
      showMethodList();
      continue;
    }
    
    if (input === 'list' || input === 'ls') {
      showMethodList();
      continue;
    }
    
    if (!input) continue;
    
    // Parse input as command
    const parts = input.split(/\s+/);
    const method = parts[0];
    const args = parts.slice(1);
    
    if (!SDK_METHODS[method]) {
      console.log(`${C.red}✗ Unknown method: ${method}${C.reset}`);
      console.log(`${C.dim}Use 'list' to see available methods${C.reset}\n`);
      continue;
    }
    
    // Execute the method
    try {
      const client = createClient(getDefaultRpc());
      let result;
      
      switch (method) {
        case 'getSlot':
          result = await client.getSlot();
          break;
        case 'getBlockHeight':
          result = await client.getBlockHeight();
          break;
        case 'getEpochInfo':
          result = await client.getEpochInfo();
          break;
        case 'getHealth':
          result = await client.getHealth();
          break;
        case 'getVersion':
          result = await client.getVersion();
          break;
        case 'getTPS':
          result = await client.getTPS();
          break;
        case 'getSupply':
          result = await client.getSupply();
          break;
        case 'getFees':
          result = await client.getFees();
          break;
        case 'getRecentBlockhash':
          result = await client.getRecentBlockhash();
          break;
        case 'getSlotProduction':
          result = await client.getSlotProduction();
          break;
        case 'getClusterPeers':
          result = await client.getClusterPeers();
          break;
        case 'getValidators':
          result = await client.getValidators();
          break;
        case 'getAccountInfo':
          result = await client.getAccountInfo(args[0]);
          break;
        case 'getBalance':
          result = await client.getBalance(args[0]);
          break;
        case 'getStakePositions':
          result = await client.getStakePositions(args[0]);
          break;
        case 'getRewards':
          result = await client.getRewards(args[0]);
          break;
        case 'getStakeAccounts':
          result = await client.getStakeAccounts(args[0]);
          break;
        case 'getTokenAccounts':
          result = await client.getTokenAccounts(args[0]);
          break;
        case 'getNFTHoldings':
          result = await client.getNFTHoldings(args[0]);
          break;
        case 'getNFTsByCreator':
          result = await client.getNFTsByCreator(args[0]);
          break;
        case 'getTransaction':
          result = await client.getTransaction(args[0]);
          break;
        case 'getRecentTransactions':
          result = await client.getRecentTransactions(args[0], parseInt(args[1]) || 20);
          break;
        case 'getTransactionHistory':
          result = await client.getTransactionHistory(args[0], parseInt(args[1]) || 20);
          break;
        case 'getNFT':
          result = await client.getNFT(args[0]);
          break;
        case 'ping':
          result = await aether.ping(args[0] || getDefaultRpc());
          break;
        default:
          throw new Error(`Method ${method} not implemented`);
      }
      
      // Show result
      if (typeof result === 'object') {
        if (method === 'getBalance') {
          console.log(`  ${C.green}Balance:${C.reset} ${formatNumber(result)} lamports (${formatAether(result)})`);
        } else if (method === 'getSlot' || method === 'getBlockHeight') {
          console.log(`  ${C.green}Result:${C.reset} ${formatNumber(result)}`);
        } else if (method === 'getTPS') {
          console.log(`  ${C.green}TPS:${C.reset} ${result}`);
        } else if (method === 'getHealth') {
          const status = result === 'ok' ? `${C.green}OK${C.reset}` : `${C.yellow}${result}${C.reset}`;
          console.log(`  ${C.green}Status:${C.reset} ${status}`);
        } else if (Array.isArray(result)) {
          console.log(`  ${C.green}Count:${C.reset} ${result.length}`);
          result.slice(0, 5).forEach((item, i) => {
            const id = item.pubkey || item.address || item.signature || item.id || `Item ${i + 1}`;
            console.log(`    ${C.dim}[${i}]${C.reset} ${shortAddress(id)}`);
          });
          if (result.length > 5) {
            console.log(`    ${C.dim}... and ${result.length - 5} more${C.reset}`);
          }
        } else {
          console.log(`  ${C.green}Result:${C.reset}`);
          console.log(JSON.stringify(result, null, 2).split('\n').map(l => `  ${C.dim}${l}${C.reset}`).join('\n'));
        }
      } else {
        console.log(`  ${C.green}Result:${C.reset} ${result}`);
      }
      console.log();
      
    } catch (err) {
      console.log(`${C.red}✗ Error: ${err.message}${C.reset}\n`);
    }
  }
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function sdkCommand() {
  const opts = parseArgs();
  
  if (opts.help) {
    showHelp();
    return;
  }
  
  if (opts.list) {
    showMethodList();
    return;
  }
  
  if (!opts.method) {
    // Interactive mode if no method specified
    await interactiveMode();
    return;
  }
  
  try {
    await executeMethod(opts);
  } catch (err) {
    console.error(`\n${C.red}✗ SDK command failed:${C.reset} ${err.message}\n`);
    console.error(`${C.dim}Run ${C.cyan}aether sdk --list${C.reset}${C.dim} to see available methods${C.reset}\n`);
    process.exit(1);
  }
}

module.exports = { sdkCommand };

if (require.main === module) {
  sdkCommand().catch(err => {
    console.error(`${C.red}✗ SDK command failed:${C.reset} ${err.message}`);
    process.exit(1);
  });
}
