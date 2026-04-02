/**
 * aether-cli validator-start
 * 
 * Spawns the aether-validator binary as a child process.
 * Handles startup, logging, and graceful shutdown.
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
};

/**
 * Find the aether-validator binary
 */
function findValidatorBinary() {
  // Check common locations
  const locations = [
    // From cargo build
    path.join(__dirname, '..', '..', 'target', 'release', 'aether-validator'),
    path.join(__dirname, '..', '..', 'target', 'debug', 'aether-validator'),
    // Direct cargo run
    'cargo',
    // System PATH
    'aether-validator',
  ];

  for (const loc of locations) {
    if (loc === 'cargo') {
      return { type: 'cargo', path: null };
    }
    if (loc === 'aether-validator') {
      return { type: 'binary', path: loc };
    }
    if (fs.existsSync(loc)) {
      return { type: 'binary', path: loc };
    }
  }

  // Default to cargo run
  return { type: 'cargo', path: null };
}

/**
 * Parse command line args for validator-start
 */
function parseArgs() {
  const args = process.argv.slice(3); // Skip 'aether-cli validator start'
  
  const options = {
    testnet: false,
    rpcAddr: '127.0.0.1:8899',
    p2pAddr: '0.0.0.0:8001',
    identity: null,
    verbose: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--testnet':
        options.testnet = true;
        break;
      case '--rpc-addr':
        options.rpcAddr = args[++i];
        break;
      case '--p2p-addr':
        options.p2pAddr = args[++i];
        break;
      case '--identity':
        options.identity = args[++i];
        break;
      case '-v':
      case '--verbose':
        options.verbose = true;
        break;
    }
  }

  return options;
}

/**
 * Print startup banner
 */
function printBanner(options) {
  console.log(`
${colors.cyan}╔═══════════════════════════════════════════════════════════════╗
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}AETHER VALIDATOR${colors.reset}${colors.cyan}                                          ║
${colors.cyan}║   ${colors.bright}Starting Validator Node${colors.reset}${colors.cyan}                                 ║
${colors.cyan}║                                                               ║
${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
  
  console.log(`  ${colors.bright}Network:${colors.reset}`);
  console.log(`    Mode:      ${options.testnet ? colors.yellow + 'TESTNET' : colors.green + 'MAINNET (not implemented)'}`);
  console.log(`    RPC:       http://${options.rpcAddr}`);
  console.log(`    P2P:       ${options.p2pAddr}`);
  if (options.identity) {
    console.log(`    Identity:  ${options.identity}`);
  }
  console.log();
}

/**
 * Main validator start command
 */
function validatorStart() {
  const options = parseArgs();
  const { type, path: binaryPath } = findValidatorBinary();

  printBanner(options);

  // Build command args
  const args = ['run', '--bin', 'aether-validator', '--', 'start'];
  
  if (options.testnet) {
    args.push('--testnet');
  }
  args.push('--rpc-addr', options.rpcAddr);
  args.push('--p2p-addr', options.p2pAddr);
  if (options.identity) {
    args.push('--identity', options.identity);
  }
  if (options.verbose) {
    args.push('-vvv');
  }

  console.log(`  ${colors.bright}Command:${colors.reset} ${type === 'cargo' ? 'cargo ' + args.join(' ') : binaryPath + ' ' + args.slice(3).join(' ')}`);
  console.log();
  console.log(`  ${colors.yellow}Starting validator (press Ctrl+C to stop)...${colors.reset}`);
  console.log();

  // Spawn the validator process
  const child = type === 'cargo' 
    ? spawn('cargo', args, {
        stdio: ['inherit', 'pipe', 'pipe'],
        shell: true,
        cwd: path.join(__dirname, '..', '..'),
      })
    : spawn(binaryPath, args.slice(3), {
        stdio: ['inherit', 'pipe', 'pipe'],
      });

  // Colorize output
  const outputColorizer = (data, isError = false) => {
    const str = data.toString();
    const color = isError ? colors.red : colors.reset;
    process.stdout.write(`${color}${str}${colors.reset}`);
  };

  child.stdout.on('data', (data) => outputColorizer(data));
  child.stderr.on('data', (data) => outputColorizer(data, true));

  child.on('error', (err) => {
    console.error(`${colors.red}Failed to start validator: ${err.message}${colors.reset}`);
    process.exit(1);
  });

  child.on('close', (code) => {
    console.log(`\n${colors.yellow}Validator exited with code ${code}${colors.reset}`);
    process.exit(code);
  });

  // Handle Ctrl+C
  process.on('SIGINT', () => {
    console.log(`\n${colors.yellow}Shutting down validator...${colors.reset}`);
    child.kill('SIGINT');
    setTimeout(() => {
      child.kill('SIGTERM');
    }, 1000);
  });
}

// Export for use as module
module.exports = { validatorStart };

// Run if called directly
if (require.main === module) {
  validatorStart();
}
