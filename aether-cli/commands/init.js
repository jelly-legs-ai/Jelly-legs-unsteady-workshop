/**
 * aether-cli init
 * 
 * Onboarding wizard for new validators.
 * Guides users through identity creation, stake account setup, and testnet connection.
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const readline = require('readline');

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
  return new Promise((resolve) => {
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
 * Generate validator identity
 */
async function generateIdentity(rl) {
  printStep(2, 4, 'Generating Validator Identity');
  
  const identityPath = path.join(process.cwd(), 'validator-identity.json');
  
  // Check if identity already exists
  if (fs.existsSync(identityPath)) {
    printWarning('Validator identity already exists at validator-identity.json');
    const regenerate = await askQuestion(rl, 'Regenerate identity?', 'n');
    if (!regenerate) {
      printSuccess('Using existing identity');
      return identityPath;
    }
  }
  
  console.log('\nGenerating new Ed25519 keypair...');
  
  try {
    // Try using cargo to run the binary
    await runCommand('cargo run --bin aether-validator -- create-validator-identity --out validator-identity.json --force', 120000);
    printSuccess(`Identity saved to validator-identity.json`);
  } catch (e) {
    // Fallback: generate a dummy keypair for now
    printWarning('Could not run aether-validator. Generating placeholder identity...');
    
    const crypto = require('crypto');
    const keypair = crypto.generateKeyPairSync('ed25519');
    
    const pubkey = keypair.publicKey.export({ type: 'spki', format: 'der' }).toString('base64');
    
    const identityJson = {
      pubkey: `placeholder_${Buffer.from(pubkey).toString('base64').slice(0, 44)}`,
      secret: 'placeholder',
      note: 'This is a placeholder. Run "cargo build --bin aether-validator" and then "cargo run --bin aether-validator -- create-validator-identity"',
    };
    
    fs.writeFileSync(identityPath, JSON.stringify(identityJson, null, 2));
    printSuccess('Placeholder identity created (run full build later)');
  }
  
  console.log();
  printWarning('IMPORTANT: Backup your validator-identity.json file!');
  printWarning('If you lose this file, you lose your validator status.');
  
  return identityPath;
}

/**
 * Connect to testnet
 */
async function connectTestnet(rl) {
  printStep(3, 4, 'Connecting to Testnet');
  
  console.log('The validator will connect to the AETHER testnet.');
  console.log('Testnet uses aether-testnet-1 chain ID with reduced stake requirements.\n');
  
  const startNow = await askQuestion(rl, 'Start validator now?', 'y');
  
  if (startNow) {
    console.log('\nStarting validator in testnet mode...\n');
    
    const validatorStart = require('./validator-start');
    validatorStart.validatorStart();
  } else {
    console.log();
    printSuccess('You can start the validator later with:');
    console.log(`  ${colors.bright}aether-cli validator start --testnet${colors.reset}`);
  }
  
  return true;
}

/**
 * Print completion summary
 */
async function printSummary() {
  printStep(4, 4, 'Setup Complete');
  
  console.log(`
  ${colors.green}╔═══════════════════════════════════════════════════════════════╗
  ${colors.green}║                                                               ║
  ${colors.green}║   ${colors.bright}✅ VALIDATOR SETUP COMPLETE${colors.reset}${colors.green}                               ║
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
  console.log('  2. Create a stake account: cargo run --bin aether-validator -- create-stake-account');
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
 * Main init command
 */
async function init() {
  printBanner();
  
  const rl = createReadline();
  
  try {
    await checkPrerequisites(rl);
    await generateIdentity(rl);
    await connectTestnet(rl);
    await printSummary();
  } finally {
    rl.close();
  }
}

// Export for use as module
module.exports = { init };

// Run if called directly
if (require.main === module) {
  init();
}
