/**
 * aether-cli init
 * 
 * Onboarding wizard for new validators.
 * Guides users through identity creation, stake account setup, and testnet connection.
 */

const fs = require('fs');
const path = require('path');
const { spawn, execSync } = require('child_process');
const nacl = require('tweetnacl');
const bs58 = require('bs58').default;
const readline = require('readline');
const os = require('os');

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  red: '\x1b[31m',
  dim: '\x1b[2m',
  magenta: '\x1b[35m',
};

/**
 * Create readline interface for user input
 */
function createReadline() {
  return readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
}

/**
 * Ask a yes/no question
 */
function askQuestion(rl, question, defaultValue = 'y') {
  return new Promise((resolve, reject) => {
    if (rl.closed) {
      reject(new Error('Readline interface closed'));
      return;
    }
    const suffix = defaultValue === 'y' ? ' [Y/n]' : ' [y/N]';
    rl.question(`${colors.cyan}${question}${suffix}: ${colors.reset}`, (answer) => {
      const normalized = answer.trim().toLowerCase();
      if (normalized === '') {
        resolve(defaultValue === 'y');
      } else {
        resolve(normalized === 'y' || normalized === 'yes');
      }
    });
  });
}

/**
 * Ask for a string value
 */
function askValue(rl, question, defaultValue = '') {
  return new Promise((resolve) => {
    const suffix = defaultValue ? ` [${defaultValue}]` : '';
    rl.question(`${colors.cyan}${question}${suffix}: ${colors.reset}`, (value) => {
      resolve(value.trim() || defaultValue);
    });
  });
}

/**
 * Print section header
 */
function printSection(title) {
  console.log();
  console.log(`${colors.bright}${colors.cyan}${'═'.repeat(60)}${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}  ${title}${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}${'═'.repeat(60)}${colors.reset}`);
  console.log();
}

/**
 * Print a step indicator
 */
function printStep(step, total, title) {
  console.log();
  console.log(`${colors.yellow}Step ${step}/${total}:${colors.reset} ${colors.bright}${title}${colors.reset}`);
  console.log(`${colors.dim}${'─'.repeat(60)}${colors.reset}`);
  console.log();
}

/**
 * Print success message
 */
function printSuccess(message) {
  console.log(`  ${colors.green}✓${colors.reset} ${message}`);
}

/**
 * Print warning message
 */
function printWarning(message) {
  console.log(`  ${colors.yellow}⚠${colors.reset} ${message}`);
}

/**
 * Print error message
 */
function printError(message) {
  console.log(`  ${colors.red}✗${colors.reset} ${message}`);
}

/**
 * Print the welcome banner
 */
function printBanner() {
  console.log(`
${colors.cyan}╔═══════════════════════════════════════════════════════════════╗
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}███████╗██╗███████╗██╗  ██╗████████╗${colors.reset}${colors.cyan}                       ║
${colors.cyan}║   ${colors.bright}██╔════╝██║██╔════╝╚██╗██╔╝╚══██╔══╝${colors.reset}${colors.cyan}                       ║
${colors.cyan}║   ${colors.bright}███████╗██║███████╗ ╚███╔╝    ██║${colors.reset}${colors.cyan}                          ║
${colors.cyan}║   ${colors.bright}╚════██║██║╚════██║ ██╔██╗    ██║${colors.reset}${colors.cyan}                          ║
${colors.cyan}║   ${colors.bright}███████║██║███████║██╔╝ ██╗   ██║${colors.reset}${colors.cyan}                          ║
${colors.cyan}║   ${colors.bright}╚══════╝╚═╝╚══════╝╚═╝  ╚═╝   ╚═╝${colors.reset}${colors.cyan}                          ║
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}AETHER VALIDATOR ONBOARDING WIZARD${colors.reset}${colors.cyan}                       ║
${colors.cyan}║                                                               ║
${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
}

/**
 * Check prerequisites
 */
async function checkPrerequisites(rl) {
  printStep(1, 4, 'Checking Prerequisites');
  
  const checks = [];
  
  // Check Node.js
  const nodeVersion = process.version;
  checks.push({
    name: 'Node.js',
    passed: parseInt(nodeVersion.slice(1).split('.')[0]) >= 14,
    message: `Node.js ${nodeVersion}`,
  });
  
  // Check Rust
  try {
    const rustVersion = await runCommand('rustc --version');
    checks.push({
      name: 'Rust',
      passed: true,
      message: rustVersion.trim(),
    });
  } catch (e) {
    checks.push({
      name: 'Rust',
      passed: false,
      message: 'Not installed',
    });
  }
  
  // Check Cargo
  try {
    await runCommand('cargo --version');
    checks.push({
      name: 'Cargo',
      passed: true,
      message: 'Installed',
    });
  } catch (e) {
    checks.push({
      name: 'Cargo',
      passed: false,
      message: 'Not installed',
    });
  }
  
  // Check disk space
  const diskSpace = getDiskSpace();
  checks.push({
    name: 'Disk Space',
    passed: diskSpace.free >= 100,
    message: `${diskSpace.free} GB free`,
  });

  console.log('Checking system requirements...\n');
  
  let allPassed = true;
  for (const check of checks) {
    const status = check.passed ? `${colors.green}✓${colors.reset}` : `${colors.red}✗${colors.reset}`;
    console.log(`  ${status} ${check.name}: ${check.message}`);
    if (!check.passed) {
      allPassed = false;
    }
  }
  
  if (!allPassed) {
    console.log();
    printWarning('Some prerequisites are missing. Please install them before continuing.');
    const continueAnyway = await askQuestion(rl, 'Continue anyway?', 'n');
    if (!continueAnyway) {
      console.log('\nRun the following to install prerequisites:');
      console.log('  curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh');
      console.log('  # Then restart your terminal');
      process.exit(1);
    }
  }
  
  return true;
}

/**
 * Find the aether-validator binary
 * Searches in order:
 *   1. JELLY_LEGS_ROOT env var (for CI/automated setups)
 *   2. Current working directory (when running from repo clone)
 *   3. Platform-specific default install locations
 *   4. Sibling repo relative path (when aether-cli is in node_modules)
 *   5. System PATH
 */
function findValidatorBinary() {
  const platform = os.platform();
  const isWindows = platform === 'win32';
  const binaryName = isWindows ? 'aether-validator.exe' : 'aether-validator';
  const execExt = isWindows ? '.exe' : '';

  // Helper to check a location and return it if it exists
  const tryPath = (loc) => {
    try {
      if (fs.existsSync(loc)) return { type: 'binary', path: loc };
    } catch {}
    return null;
  };

  // 1. JELLY_LEGS_ROOT env var
  const jellyRoot = process.env.JELLY_LEGS_ROOT;
  if (jellyRoot) {
    const r = tryPath(path.join(jellyRoot, 'target', 'debug', binaryName))
           || tryPath(path.join(jellyRoot, 'target', 'release', binaryName));
    if (r) return r;
  }

  // 2. Current working directory
  const cwd = process.cwd();
  const r = tryPath(path.join(cwd, 'target', 'debug', binaryName))
         || tryPath(path.join(cwd, 'target', 'release', binaryName))
         || tryPath(path.join(cwd, binaryName));
  if (r) return r;

  // 3. Platform-specific locations
  if (isWindows) {
    const r2 = tryPath('C:\\Users\\' + (process.env.USERNAME || 'User') + '\\.jelly-legs\\aether-validator.exe')
           || tryPath('C:\\jelly-legs\\aether-validator.exe');
    if (r2) return r2;
  } else {
    const r2 = tryPath(path.join(os.homedir(), '.jelly-legs', 'aether-validator'))
           || tryPath('/usr/local/bin/aether-validator')
           || tryPath('/usr/bin/aether-validator');
    if (r2) return r2;
  }

  // 4. Sibling repo path (when aether-cli is inside node_modules of jelly-legs repo)
  const siblingRepo = path.join(__dirname, '..', '..', 'Jelly-legs-unsteady-workshop');
  const r3 = tryPath(path.join(siblingRepo, 'target', 'debug', binaryName))
         || tryPath(path.join(siblingRepo, 'target', 'release', binaryName));
  if (r3) return r3;

  // 5. Local aether-cli target
  const r4 = tryPath(path.join(__dirname, '..', 'target', 'debug', binaryName))
         || tryPath(path.join(__dirname, '..', 'target', 'release', binaryName));
  if (r4) return r4;

  // 6. System PATH
  try {
    const checkCmd = isWindows ? 'where' : 'which';
    const out = execSync(`${checkCmd} aether-validator${execExt}`, { stdio: 'pipe' }).toString().trim();
    const firstLine = out.split('\n')[0];
    if (firstLine && fs.existsSync(firstLine)) return { type: 'binary', path: firstLine, inPath: true };
  } catch {}

  // Not found
  return { type: 'missing', path: null };
}

/**
 * Build the validator binary if missing
 * Returns the found binary path on success, null on failure.
 */
function buildValidator() {
  // Find the repo root (where Cargo.toml lives)
  const repoPath = path.join(__dirname, '..', '..', 'Jelly-legs-unsteady-workshop');
  const altRepoPath = path.join(__dirname, '..', '..'); // fallback: aether-cli itself

  // Pick whichever has a Cargo.toml
  const cargoToml = path.join(repoPath, 'Cargo.toml');
  const effectiveRepo = fs.existsSync(cargoToml) ? repoPath : altRepoPath;

  // Use explicit cargo path on Windows
  const platform = os.platform();
  const isWindows = platform === 'win32';
  let cargoCmd = 'cargo';
  if (isWindows) {
    const explicitCargo = 'C:\\Users\\RM_Ga\\.cargo\\bin\\cargo.exe';
    if (fs.existsSync(explicitCargo)) cargoCmd = explicitCargo;
  }

  console.log(`  ${colors.cyan}Building aether-validator...${colors.reset}`);
  if (isWindows) console.log(`  (using ${cargoCmd})`);

  try {
    execSync(`${cargoCmd} build --bin aether-validator`, {
      cwd: effectiveRepo,
      stdio: 'inherit',
      shell: false,   // don't rely on shell for path lookup
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
    // Fallback: try once more with shell enabled (handles PATH issues on some systems)
    try {
      console.log(`  ${colors.yellow}Retry with shell fallback...${colors.reset}`);
      execSync(`${cargoCmd} build --bin aether-validator`, {
        cwd: effectiveRepo,
        stdio: 'inherit',
        shell: true,
      });
      const result = findValidatorBinary();
      if (result.type === 'binary') {
        console.log(`  ${colors.green}✓ Build successful!${colors.reset}`);
        return result;
      }
    } catch {}
    console.error(`  ${colors.red}✗ Build failed: ${err.message}${colors.reset}`);
    return null;
  }
}

/**
 * Run the validator binary with args
 * Handles missing binary by offering to build it
 */
function runValidatorBinary(args, options = {}, rl = null) {
  let result = findValidatorBinary();
  
  // Handle missing binary
  if (result.type === 'missing') {
    console.log(`  ${colors.yellow}⚠ Validator binary not found${colors.reset}`);
    console.log(`  ${colors.cyan}Would you like to build it now? (cargo build --bin aether-validator)${colors.reset}`);
    console.log();
    
    if (rl) {
      return new Promise((resolve, reject) => {
        rl.question('  Build now? [Y/n] ', (answer) => {
          if (answer.toLowerCase() === 'n' || answer.toLowerCase() === 'no') {
            reject(new Error('User declined to build validator'));
            return;
          }
          
          const built = buildValidator();
          if (!built) {
            reject(new Error('Build failed'));
            return;
          }
          result = built;
          resolve(runValidatorBinaryInternal(result, args, options));
        });
      });
    } else {
      // No readline available - try to build automatically
      const built = buildValidator();
      if (!built) {
        throw new Error('Validator binary not found and build failed');
      }
      result = built;
    }
  }
  
  return runValidatorBinaryInternal(result, args, options);
}

/**
 * Internal function to run the validator binary (assumes binary exists)
 */
function runValidatorBinaryInternal({ type, path: binaryPath, inPath }, args, options = {}) {
  const workspaceRoot = path.join(__dirname, '..', '..');
  const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
  
  const result = execSync(`"${binaryPath}" ${args.join(' ')}`, {
    cwd: inPath ? undefined : repoPath,
    stdio: 'inherit',
    shell: true,
    ...options,
  });
  
  return { type, path: binaryPath, inPath };
}

/**
 * Generate a new validator Ed25519 keypair using TweetNaCl (pure JS, no Rust needed)
 * Compatible with the Rust keypair format:
 *   Rust SigningKey::from_bytes expects 32-byte seed
 *   TweetNaCl secretKey is 64 bytes: seed(32) || publicKey(32)
 *   We encode only the 32-byte seed as bs58 to match the Rust format
 */
function generateEd25519Identity(outPath, force = false) {
  const identityFile = outPath || path.join(process.cwd(), 'validator-identity.json');

  if (fs.existsSync(identityFile) && !force) {
    throw new Error(`Identity file already exists at ${identityFile}. Use --force to overwrite.`);
  }

  // Generate a new Ed25519 keypair using TweetNaCl
  const keyPair = nacl.sign.keyPair();

  // TweetNaCl secretKey is 64 bytes: seed(32) || publicKey(32)
  // Rust SigningKey::from_bytes expects the 32-byte seed only
  const seed32 = keyPair.secretKey.slice(0, 32);

  const secretBs58 = bs58.encode(seed32);
  const publicBs58 = bs58.encode(keyPair.publicKey);

  const identity = {
    pubkey: publicBs58,
    secret: secretBs58,
  };

  fs.writeFileSync(identityFile, JSON.stringify(identity, null, 2));
  return { identityFile, publicKey: publicBs58 };
}

async function generateIdentity(rl, tier = 'full') {
  printStep(3, 5, 'Generating Validator Identity');

  const identityPath = path.join(process.cwd(), 'validator-identity.json');

  // Check if identity already exists
  if (fs.existsSync(identityPath)) {
    printWarning('Validator identity already exists at validator-identity.json');
    try {
      const regenerate = await askQuestion(rl, 'Regenerate identity?', 'n');
      if (!regenerate) {
        printSuccess('Using existing identity');
        return identityPath;
      }
    } catch (err) {
      // Readline may have closed - skip regeneration prompt
      printSuccess('Using existing identity');
      return identityPath;
    }
  }

  console.log('\nGenerating new Ed25519 keypair (pure JS)...');
  console.log(`${colors.dim}Tier: ${tier.toUpperCase()}${colors.reset}`);

  try {
    const { identityFile, publicKey } = generateEd25519Identity(identityPath, true);
    printSuccess(`Identity saved to ${path.basename(identityFile)}`);
    console.log(`  ${colors.dim}Public key: ${publicKey}${colors.reset}`);
  } catch (e) {
    printError(`Failed to create identity: ${e.message}`);
    printWarning('You can create it manually with: node -e "const nacl=require(\'tweetnacl\');..."');
    process.exit(1);
  }

  console.log();
  printWarning('IMPORTANT: Backup your validator-identity.json file!');
  printWarning('If you lose this file, you lose your validator status.');

  return identityPath;
}

/**
 * Connect to testnet
 */
async function connectTestnet(rl, tier = 'full') {
  printStep(4, 5, 'Connecting to Testnet');
  
  console.log('The validator will connect to the AETHER testnet.');
  console.log('Testnet uses aether-testnet-1 chain ID with reduced stake requirements.\n');
  console.log(`${colors.dim}Selected tier: ${tier.toUpperCase()}${colors.reset}`);
  console.log();
  
  try {
    const startNow = await askQuestion(rl, 'Start validator now?', 'y');
    
    if (startNow) {
      console.log('\nStarting validator in testnet mode...\n');
      
      // Close readline before handing off to validator (it manages its own stdin)
      rl.close();
      
      const validatorStart = require('./validator-start');
      validatorStart.validatorStart(tier);
      return true;
    }
  } catch (err) {
    // Readline closed or error - skip the prompt
    console.log(`  ${colors.yellow}⚠ Skipping prompt (interactive mode unavailable)${colors.reset}`);
  }
  
  console.log();
  printSuccess('You can start the validator later with:');
  console.log(`  ${colors.bright}aether-cli validator start --testnet --tier ${tier}${colors.reset}`);
  
  return true;
}

/**
 * Print completion summary
 */
async function printSummary(tierBadge = '[FULL]') {
  printStep(5, 5, 'Setup Complete');
  
  console.log(`
  ${colors.green}╔═══════════════════════════════════════════════════════════════╗
  ${colors.green}║                                                               ║
  ${colors.green}║   ${colors.bright}✅ VALIDATOR SETUP COMPLETE${colors.reset}${colors.green}                               ║
  ${colors.green}║   ${colors.dim}Tier: ${tierBadge}${colors.reset}${colors.green}                                              ║
  ${colors.green}║                                                               ║
  ${colors.green}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
  
  console.log('Useful commands:');
  console.log(`  ${colors.bright}aether-cli validator status${colors.reset}    Check validator status`);
  console.log(`  ${colors.bright}aether-cli validator start${colors.reset}     Start the validator`);
  console.log(`  ${colors.bright}aether-cli doctor${colors.reset}              Run system checks`);
  console.log();
  
  console.log('Next steps:');
  console.log('  1. Fund your validator wallet with testnet AETH');
  console.log('  2. Create a stake account: aether-validator create-stake-account');
  console.log('  3. Monitor your validator: aether-cli validator status');
  console.log();
}

/**
 * Run an external command
 */
function runCommand(cmd, timeout = 30000) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, [], { 
      shell: true,
      stdio: 'pipe',
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => { stdout += data.toString(); });
    child.stderr.on('data', (data) => { stderr += data.toString(); });
    
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error('Command timed out'));
    }, timeout);
    
    child.on('close', (code) => {
      clearTimeout(timer);
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(stderr || `Command exited with code ${code}`));
      }
    });
    
    child.on('error', reject);
  });
}

/**
 * Get approximate disk space (cross-platform)
 */
function getDiskSpace() {
  try {
    if (process.platform === 'win32') {
      const { execSync } = require('child_process');
      const output = execSync('powershell -c "(Get-PSDrive -Name C).Free / 1GB"', { encoding: 'utf8' });
      return { free: parseFloat(output.trim()) };
    } else {
      const output = fs.readFileSync('/dev/null', 'utf8');
      const stat = fs.statfsSync('/');
      return { free: stat.bsize * stat.bfree / (1024 * 1024 * 1024) };
    }
  } catch (e) {
    return { free: 100 }; // Assume sufficient if can't check
  }
}

/**
 * Select validator tier with hardware summary
 */
async function selectTier(rl) {
  printStep(2, 5, 'Select Validator Tier');
  
  console.log('Choose your validator tier based on your hardware and stake:');
  console.log();
  console.log(`  ${colors.bright}[1] FULL${colors.reset}    - 10K AETH stake, 8 cores, 32GB RAM, 512GB SSD`);
  console.log(`               Full consensus weight (1.0x), block production, voting rights`);
  console.log();
  console.log(`  ${colors.bright}[2] LITE${colors.reset}    - 1K AETH stake, 4 cores, 8GB RAM, 100GB SSD`);
  console.log(`               Stake-based weight (stake/10000), voting only, no solo blocks`);
  console.log();
  console.log(`  ${colors.bright}[3] OBSERVER${colors.reset} - 0 AETH stake, 2 cores, 4GB RAM, 50GB disk`);
  console.log(`               Relay-only, earns FLUX via data relay, no voting rights`);
  console.log();
  
  const tierChoice = await askValue(rl, 'Select tier [1/2/3]', '1');
  
  let selectedTier = 'full';
  let tierBadge = '[FULL]';
  
  switch (tierChoice.trim()) {
    case '1':
      selectedTier = 'full';
      tierBadge = '[FULL]';
      console.log(`\n  ${colors.green}✓ Selected: FULL Validator${colors.reset}`);
      console.log(`  ${colors.dim}Hardware: 8+ cores, 32GB+ RAM, 512GB+ SSD, 100Mbps+${colors.reset}`);
      console.log(`  ${colors.dim}Stake: 10,000 AETH minimum${colors.reset}`);
      console.log(`  ${colors.dim}Consensus: 1.0x weight, full voting + block production${colors.reset}`);
      break;
    case '2':
      selectedTier = 'lite';
      tierBadge = '[LITE]';
      console.log(`\n  ${colors.green}✓ Selected: LITE Validator${colors.reset}`);
      console.log(`  ${colors.dim}Hardware: 4+ cores, 8GB+ RAM, 100GB+ SSD, 25Mbps+${colors.reset}`);
      console.log(`  ${colors.dim}Stake: 1,000 AETH minimum${colors.reset}`);
      console.log(`  ${colors.dim}Consensus: stake/10000 weight, voting only${colors.reset}`);
      break;
    case '3':
      selectedTier = 'observer';
      tierBadge = '[OBSERVER]';
      console.log(`\n  ${colors.green}✓ Selected: OBSERVER Node${colors.reset}`);
      console.log(`  ${colors.dim}Hardware: 2+ cores, 4GB+ RAM, 50GB+ disk, 10Mbps+${colors.reset}`);
      console.log(`  ${colors.dim}Stake: 0 AETH (relay-only)${colors.reset}`);
      console.log(`  ${colors.dim}Earnings: FLUX via data relay (0.000001 FLUX/byte)${colors.reset}`);
      break;
    default:
      selectedTier = 'full';
      tierBadge = '[FULL]';
      console.log(`\n  ${colors.yellow}⚠ Invalid choice, defaulting to FULL${colors.reset}`);
  }
  
  console.log();
  printWarning('You can change tiers later by re-running init with a different selection.');
  
  return { tier: selectedTier, badge: tierBadge };
}

/**
 * Main init command
 */
async function init() {
  printBanner();
  
  const rl = createReadline();
  
  try {
    await checkPrerequisites(rl);
    const { tier, badge } = await selectTier(rl);
    await generateIdentity(rl, tier);
    await connectTestnet(rl, tier);
    await printSummary(badge);
  } finally {
    rl.close();
  }
}

// Export for use as module
module.exports = { init, findValidatorBinary, buildValidator, runValidatorBinary };

// Run if called directly
if (require.main === module) {
  init();
}
