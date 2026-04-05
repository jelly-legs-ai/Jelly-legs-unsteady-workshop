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
 * Searches common locations based on OS and repo layout
 */
function findValidatorBinary() {
  const platform = os.platform();
  const isWindows = platform === 'win32';
  const binaryName = isWindows ? 'aether-validator.exe' : 'aether-validator';
  
  // Check common locations
  const locations = [
    // Sibling repo: Jelly-legs-unsteady-workshop/target/debug/
    path.join(__dirname, '..', '..', 'Jelly-legs-unsteady-workshop', 'target', 'debug', binaryName),
    path.join(__dirname, '..', '..', 'Jelly-legs-unsteady-workshop', 'target', 'release', binaryName),
    // Local build in aether-cli (if someone built here)
    path.join(__dirname, '..', 'target', 'debug', binaryName),
    path.join(__dirname, '..', 'target', 'release', binaryName),
    // System PATH
    'aether-validator' + (isWindows ? '.exe' : ''),
  ];

  for (const loc of locations) {
    if (loc.startsWith('aether-validator')) {
      // Check if it's in PATH
      try {
        const { execSync } = require('child_process');
        const checkCmd = isWindows ? 'where' : 'which';
        execSync(`${checkCmd} ${loc}`, { stdio: 'pipe' });
        return { type: 'binary', path: loc, inPath: true };
      } catch {
        // Not in PATH, continue
      }
    }
    if (fs.existsSync(loc)) {
      return { type: 'binary', path: loc };
    }
  }

  // Binary not found - offer to build it
  return { type: 'missing', path: null };
}

/**
 * Parse command line args for validator-start
 * @param {Object} overrideOptions - Options passed directly (e.g. from init.js)
 */
function parseArgs(overrideOptions = {}) {
  const args = process.argv.slice(3); // Skip 'aether-cli validator start'
  
  const options = {
    testnet: false,
    rpcAddr: '127.0.0.1:8899',
    p2pAddr: '0.0.0.0:8001',
    identity: null,
    verbose: false,
    tier: 'full',
    ...overrideOptions, // Allow init.js to pass testnet/tier directly
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
      case '--tier':
        options.tier = args[++i];
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
  const tierBadge = options.tier.toUpperCase();
  const tierLabel = `[${tierBadge}]`;
  
  console.log(`
${colors.cyan}╔═══════════════════════════════════════════════════════════════╗
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}AETHER VALIDATOR${colors.reset}${colors.cyan}                                          ║
${colors.cyan}║   ${colors.bright}Starting Validator Node${colors.reset}${colors.cyan}                                 ║
${colors.cyan}║   ${colors.bright}${tierLabel}${colors.reset}${colors.cyan}                                              ║
${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
  
  console.log(`  ${colors.bright}Network:${colors.reset}`);
  const modeStr = options.testnet
    ? colors.yellow + 'TESTNET'
    : colors.red + 'MAINNET';
  console.log(`    Mode:      ${modeStr}`);
  console.log(`    Tier:      ${tierLabel}`);
  console.log(`    RPC:       http://${options.rpcAddr}`);
  console.log(`    P2P:       ${options.p2pAddr}`);
  if (options.identity) {
    console.log(`    Identity:  ${options.identity}`);
  }
  console.log();
}

/**
 * Build the validator binary if missing
 */
function buildValidator() {
  const { execSync } = require('child_process');
  const platform = os.platform();
  const isWindows = platform === 'win32';
  const workspaceRoot = path.join(__dirname, '..', '..');
  const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
  
  console.log(`  ${colors.cyan}Building aether-validator...${colors.reset}`);
  
  try {
    // Use full cargo path on Windows to avoid spawnSync ENOENT
    const cargoPaths = isWindows
      ? [
          path.join(process.env.USERPROFILE || '', '.cargo', 'bin', 'cargo.exe'),
          path.join(process.env.LOCALAPPDATA || '', 'Rust', 'bin', 'cargo.exe'),
          'C:\\Users\\RM_Ga\\.cargo\\bin\\cargo.exe',
          'cargo',
        ]
      : ['cargo'];
    
    let cargoCmd = 'cargo';
    for (const cp of cargoPaths) {
      if (cp === 'cargo' || fs.existsSync(cp)) {
        cargoCmd = cp;
        break;
      }
    }
    
    console.log(`  ${colors.cyan}Running: ${cargoCmd} build --release --package aether-validator${colors.reset}`);
    
    // Use execSync WITHOUT shell:true — avoids Windows spawnSync cmd.exe ENOENT
    execSync(`${cargoCmd} build --release --package aether-validator`, {
      cwd: repoPath,
      stdio: 'inherit',
    });
    
    // Re-check for binary
    const result = findValidatorBinary();
    if (result.type === 'binary') {
      console.log(`  ${colors.green}✓ Build successful!${colors.reset}`);
      return result;
    }
    
    console.error(`  ${colors.red}✗ Build completed but binary not found${colors.reset}`);
    return null;
  } catch (err) {
    console.error(`  ${colors.red}✗ Build failed: ${err.message}${colors.reset}`);
    return null;
  }
}

/**
 * Main validator start command
 * @param {Object|null} options - { testnet?: boolean, tier?: string }
 */
function validatorStart(options = {}) {
  // Support both old string-style (tier only) and new object-style { testnet, tier }
  const parsedArgs = parseArgs(typeof options === 'object' ? options : { tier: options });
  const optionsObj = typeof options === 'object' ? options : {};
  
  // Merge: explicit options override parseArgs defaults
  const finalOptions = {
    ...parsedArgs,
    ...optionsObj,
    tier: optionsObj.tier || parsedArgs.tier,
    testnet: optionsObj.testnet !== undefined ? optionsObj.testnet : parsedArgs.testnet,
  };
  
  let result = findValidatorBinary();

  printBanner(finalOptions);

  // Handle missing binary
  if (result.type === 'missing') {
    console.log(`  ${colors.yellow}⚠ Validator binary not found${colors.reset}`);
    console.log(`  ${colors.cyan}Would you like to build it now? (cargo build --bin aether-validator)${colors.reset}`);
    console.log();
    
    const readline = require('readline');
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
    
    rl.question('  Build now? [Y/n] ', (answer) => {
      rl.close();
      
      if (answer.toLowerCase() === 'n' || answer.toLowerCase() === 'no') {
        console.log(`  ${colors.red}Aborted. Build the validator first, then try again.${colors.reset}`);
        process.exit(1);
      }
      
      const built = buildValidator();
      if (!built) {
        process.exit(1);
      }
      result = built;
      startValidatorProcess(result, finalOptions);
    });
    return;
  }

  startValidatorProcess(result, finalOptions);
}

/**
 * Spawn the validator process
 */
function startValidatorProcess({ type, path: binaryPath, inPath }, options) {
  // Build command args
  const validatorArgs = ['start'];
  
  if (options.testnet) {
    validatorArgs.push('--testnet');
  }
  validatorArgs.push('--tier', options.tier);
  validatorArgs.push('--rpc-addr', options.rpcAddr);
  validatorArgs.push('--p2p-addr', options.p2pAddr);
  if (options.identity) {
    validatorArgs.push('--identity', options.identity);
  }
  if (options.verbose) {
    validatorArgs.push('-vvv');
  }

  const commandDisplay = inPath ? binaryPath : binaryPath || 'cargo run --bin aether-validator';
  console.log(`  ${colors.bright}Command:${colors.reset} ${commandDisplay} ${validatorArgs.join(' ')}`);
  console.log();
  console.log(`  ${colors.yellow}Starting validator (press Ctrl+C to stop)...${colors.reset}`);
  console.log();

  // Determine working directory
  const workspaceRoot = path.join(__dirname, '..', '..');
  const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
  
  // Spawn the validator process
  const child = inPath || binaryPath === 'aether-validator' || binaryPath === 'aether-validator.exe'
    ? spawn(binaryPath, validatorArgs, {
        stdio: ['inherit', 'pipe', 'pipe'],
      })
    : spawn(binaryPath, validatorArgs, {
        stdio: ['inherit', 'pipe', 'pipe'],
        cwd: repoPath,
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
