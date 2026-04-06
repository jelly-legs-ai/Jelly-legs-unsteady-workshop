#!/usr/bin/env node
/**
 * aether-cli installer
 * 
 * One-command installer for aether-cli across platforms.
 * Handles: npm install, config init, environment checks, path setup.
 * 
 * Usage:
 *   curl -sSL https://get.aether.network/install.sh | bash
 *   # or
 *   npm install -g @jellylegsai/aether-cli
 *   aether-cli install
 *   aether-cli install --path ~/.local/bin
 *   aether-cli install --rpc https://my-node:8899
 *   aether-cli install --skip-rpc-check
 *   aether-cli install --uninstall
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync, spawn } = require('child_process');
const readline = require('readline');
const https = require('https');
const http = require('http');

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

// Installer version
const VERSION = '1.0.0';

// Detect platform
const isWindows = os.platform() === 'win32';
const isMac = os.platform() === 'darwin';
const isLinux = os.platform() === 'linux';
const platform = isWindows ? 'windows' : isMac ? 'macos' : isLinux ? 'linux' : 'unknown';

// ============================================================================
// Output Helpers
// ============================================================================

function log(msg, color = C.reset) {
  console.log(`${color}${msg}${C.reset}`);
}

function info(msg) { log(`  ℹ  ${msg}`, C.cyan); }
function ok(msg) { log(`  ✓  ${msg}`, C.green); }
function warn(msg) { log(`  ⚠  ${msg}`, C.yellow); }
function error(msg) { log(`  ✗  ${msg}`, C.red); }
function step(msg) { log(`  ▶  ${msg}`, C.magenta); }
function header(msg) { log(`\n  ${C.bright}${C.cyan}${msg}${C.reset}`); }
function sub(msg) { log(`      ${C.dim}${msg}${C.reset}`); }

function separator() {
  log(`  ${C.dim}${'─'.repeat(60)}${C.reset}`);
}

// ============================================================================
// Environment Detection
// ============================================================================

function getNodeVersion() {
  try {
    return process.version.replace('v', '');
  } catch {
    return 'unknown';
  }
}

function getNpmVersion() {
  try {
    return execSync('npm --version', { encoding: 'utf8' }).trim();
  } catch {
    return 'not found';
  }
}

function getInstallPath() {
  try {
    const globalPath = execSync('npm root -g', { encoding: 'utf8' }).trim();
    return globalPath;
  } catch {
    return path.join(os.homedir(), '.npm-global', 'lib', 'node_modules');
  }
}

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function checkExistingInstall() {
  try {
    const result = execSync('npm list -g @jellylegsai/aether-cli --depth=0 2>nul', { encoding: 'utf8', shell: true });
    if (result.includes('@jellylegsai/aether-cli')) {
      const match = result.match(/@jellylegsai\/aether-cli@([\d.]+)/);
      return { installed: true, version: match ? match[1] : 'unknown' };
    }
  } catch {
    // Not found via npm
  }
  
  // Check if CLI is in PATH
  const cliPath = which('aether') || which('aether-cli');
  if (cliPath) {
    return { installed: true, version: 'unknown', path: cliPath };
  }
  
  return { installed: false };
}

function which(cmd) {
  try {
    const result = execSync(isWindows ? `where ${cmd}` : `which ${cmd}`, { encoding: 'utf8' });
    return result.split('\n')[0].trim();
  } catch {
    return null;
  }
}

// ============================================================================
// RPC Validation
// ============================================================================

function validateRpcUrl(url, timeoutMs = 5000) {
  return new Promise((resolve) => {
    try {
      const urlObj = new URL(url);
      const lib = urlObj.protocol === 'https:' ? https : http;
      
      const req = lib.request({
        hostname: urlObj.hostname,
        port: urlObj.port || (urlObj.protocol === 'https:' ? 443 : 80),
        path: '/v1/health',
        method: 'GET',
        timeout: timeoutMs,
        headers: { 'Content-Type': 'application/json' },
      }, (res) => {
        let data = '';
        res.on('data', (chunk) => data += chunk);
        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);
            resolve({ valid: true, url, latency: 0, status: parsed.status || 'ok' });
          } catch {
            resolve({ valid: true, url, latency: 0, status: 'ok' });
          }
        });
      });
      
      const start = Date.now();
      req.on('response', () => {
        const latency = Date.now() - start;
        resolve({ valid: true, url, latency, status: 'ok' });
      });
      req.on('error', (err) => {
        resolve({ valid: false, url, error: err.message });
      });
      req.on('timeout', () => {
        req.destroy();
        resolve({ valid: false, url, error: 'timeout' });
      });
      req.end();
    } catch (err) {
      resolve({ valid: false, url, error: err.message });
    }
  });
}

// ============================================================================
// Installation Steps
// ============================================================================

async function stepPreInstall(args) {
  header('PRE-INSTALLATION CHECKS');
  
  // Check Node.js version
  const nodeVersion = getNodeVersion();
  const nodeMajor = parseInt(nodeVersion.split('.')[0].replace('v', ''), 10);
  info(`Node.js version: ${nodeVersion}`);
  
  if (nodeMajor < 14) {
    error(`Node.js ${nodeVersion} detected. aether-cli requires Node.js >= 14.0.0`);
    error('Please upgrade Node.js and try again.');
    info('Visit: https://nodejs.org/');
    return { ok: false, error: 'Node.js version too old' };
  }
  ok(`Node.js ${nodeVersion} ✓`);
  
  // Check npm
  const npmVersion = getNpmVersion();
  info(`npm version: ${npmVersion}`);
  if (npmVersion === 'not found') {
    warn('npm not found in PATH. Will attempt npm global install.');
  } else {
    ok(`npm ${npmVersion} ✓`);
  }
  
  // Check disk space
  const homedir = os.homedir();
  try {
    const checkCmd = isWindows 
      ? `wmic logicaldisk where "DeviceID='${homedir.substring(0, 2)}'" get FreeSpace /value` 
      : `df -k ${homedir} | tail -1 | awk '{print $4}'`;
    const output = execSync(checkCmd, { encoding: 'utf8' }).trim();
    const freeKB = parseInt(output.match(/\d+/)?.[0] || '0', 10);
    const freeMB = freeKB / 1024;
    
    if (freeMB < 100) {
      warn(`Only ${Math.round(freeMB)}MB free in ${homedir}`);
      warn('Installing may require more space.');
    } else {
      ok(`${Math.round(freeMB)}MB available ✓`);
    }
  } catch {
    ok('Disk space check skipped ✓');
  }
  
  // Check existing installation
  const existing = checkExistingInstall();
  if (existing.installed) {
    warn(`aether-cli is already installed: ${existing.version}`);
    if (!args.force) {
      info('Use --force to reinstall or upgrade');
      return { ok: true, existing: true };
    }
    info('Reinstalling with --force...');
  }
  
  return { ok: true };
}

async function stepNpmInstall(args) {
  header('INSTALLING aether-cli');
  
  const packageName = args.package || '@jellylegsai/aether-cli';
  const installPath = getInstallPath();
  info(`Package: ${packageName}`);
  info(`Install path: ${installPath}`);
  
  step(`Running: npm install -g ${packageName}`);
  
  return new Promise((resolve) => {
    const installArgs = ['install', '-g', packageName];
    if (args.registry) {
      installArgs.push('--registry', args.registry);
    }
    
    const child = spawn(isWindows ? 'npm.cmd' : 'npm', installArgs, {
      stdio: 'inherit',
      shell: true,
      env: { ...process.env, npm_config_registry: args.registry },
    });
    
    child.on('close', (code) => {
      if (code === 0) {
        ok('Package installed successfully ✓');
        resolve({ ok: true });
      } else {
        error(`npm install failed with code ${code}`);
        resolve({ ok: false, error: `npm exit code ${code}` });
      }
    });
    
    child.on('error', (err) => {
      error(`Installation failed: ${err.message}`);
      resolve({ ok: false, error: err.message });
    });
  });
}

async function stepConfigInit(args) {
  header('CONFIGURING aether-cli');
  
  // Create .aether directory
  const aetherDir = getAetherDir();
  if (!fs.existsSync(aetherDir)) {
    fs.mkdirSync(aetherDir, { recursive: true });
    ok(`Created: ${aetherDir}`);
  } else {
    info(`Config directory exists: ${aetherDir}`);
  }
  
  // Check if config.json exists
  const configPath = getConfigPath();
  if (fs.existsSync(configPath)) {
    if (!args.force) {
      info('Config file already exists, skipping init');
      return { ok: true, existing: true };
    }
    info('Overwriting existing config (--force)');
  }
  
  // Determine RPC URL
  let rpcUrl = args.rpc || process.env.AETHER_RPC || 'http://127.0.0.1:8899';
  
  // Validate RPC URL
  if (!args.skipRpcCheck) {
    step('Validating RPC endpoint...');
    const validation = await validateRpcUrl(rpcUrl);
    
    if (validation.valid) {
      ok(`RPC endpoint reachable: ${rpcUrl}`);
      if (validation.latency) {
        const latencyColor = validation.latency < 50 ? C.green : validation.latency < 200 ? C.cyan : C.yellow;
        sub(`Latency: ${latencyColor}${validation.latency}ms${C.reset}`);
      }
    } else {
      warn(`RPC endpoint not reachable: ${rpcUrl}`);
      warn('You can set it later with: aether config set rpc.url <url>');
      sub('Continuing without RPC validation...');
    }
  } else {
    info('Skipping RPC check (--skip-rpc-check)');
  }
  
  // Create default config
  const defaultConfig = {
    version: 2,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    rpc: {
      url: rpcUrl,
      backup: args.backupRpc || null,
      timeout: 10000,
    },
    wallet: {
      default: null,
      keypair: null,
    },
    validator: {
      tier: 'full',
      identity: null,
    },
    output: {
      format: 'text',
      colors: true,
    },
    network: {
      explorer: 'https://explorer.aether.network',
      faucet: null,
    },
  };
  
  fs.writeFileSync(configPath, JSON.stringify(defaultConfig, null, 2));
  ok(`Config file created: ${configPath}`);
  
  return { ok: true, rpcUrl, configPath };
}

async function stepPathSetup(args) {
  header('PATH CONFIGURATION');
  
  const installPath = getInstallPath();
  const cliBin = path.join(installPath, 'aether-cli', 'index.js');
  
  // Check if CLI is already accessible
  const existingPath = which('aether') || which('aether-cli');
  if (existingPath) {
    ok(`aether-cli is already in PATH: ${existingPath}`);
    return { ok: true, pathSetup: 'existing' };
  }
  
  // Determine what PATH changes are needed
  const pathsToAdd = [];
  
  if (isWindows) {
    // Add npm global to PATH if not already there
    const npmGlobalPath = installPath.replace(/\\lib\\node_modules$/, '');
    const npmBinPath = path.join(npmGlobalPath, 'bin');
    pathsToAdd.push(npmBinPath);
  } else {
    // Unix - add npm global bin
    pathsToAdd.push(path.join(installPath, '..', 'bin'));
  }
  
  // Check if paths are already in PATH
  const currentPath = process.env.PATH || '';
  const newPaths = pathsToAdd.filter(p => !currentPath.includes(p));
  
  if (newPaths.length === 0) {
    ok('CLI is accessible in current PATH');
    return { ok: true, pathSetup: 'ok' };
  }
  
  // Provide instructions
  if (isWindows) {
    warn('aether-cli is installed but may not be in PATH');
    info('Add to PATH: Settings → System → Environment Variables → Path');
    info(`Add this directory:`);
    sub(newPaths[0]);
  } else if (isMac || isLinux) {
    warn('aether-cli is installed but may not be in PATH');
    info('Add to PATH by adding this to your ~/.bashrc or ~/.zshrc:');
    sub(`export PATH="${newPaths[0]}:$PATH"`);
  }
  
  return { ok: true, pathSetup: 'needs_config', paths: newPaths };
}

async function stepPostInstall(args) {
  header('POST-INSTALLATION VERIFICATION');
  
  // Find the CLI
  const cliPath = which('aether') || which('aether-cli');
  
  if (cliPath) {
    ok(`aether-cli found at: ${cliPath}`);
    
    // Try to get version
    try {
      const version = execSync(`"${cliPath}" --version`, { encoding: 'utf8', shell: true }).trim();
      sub(`Version: ${version}`);
    } catch {
      sub('Version check skipped');
    }
    
    // Run doctor to verify
    try {
      step('Running health check...');
      execSync(`"${cliPath}" doctor --tier lite`, { stdio: 'inherit', shell: true });
    } catch {
      warn('Health check had issues (this may be normal if no RPC is running)');
    }
  } else {
    warn('aether-cli not found in PATH after installation');
    info('Try opening a new terminal or restart your shell');
    info('Then run: aether-cli --version');
  }
  
  return { ok: true };
}

function showUninstall() {
  return new Promise((resolve) => {
    header('UNINSTALLING aether-cli');
    
    step('Removing npm global package...');
    
    try {
      execSync(isWindows ? 'npm.cmd uninstall -g @jellylegsai/aether-cli' : 'npm uninstall -g @jellylegsai/aether-cli', {
        stdio: 'inherit',
        shell: true,
      });
      ok('Package removed ✓');
    } catch (err) {
      warn('Failed to remove package: ' + err.message);
    }
    
    // Ask about config
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    rl.question(`\n  ${C.yellow}Remove config directory ~/.aether? [y/N]${C.reset} `, (answer) => {
      rl.close();
      
      if (answer.toLowerCase() === 'y' || answer.toLowerCase() === 'yes') {
        const aetherDir = getAetherDir();
        if (fs.existsSync(aetherDir)) {
          try {
            fs.rmSync(aetherDir, { recursive: true, force: true });
            ok(`Removed: ${aetherDir}`);
          } catch (err) {
            warn(`Failed to remove ${aetherDir}: ${err.message}`);
          }
        }
      } else {
        info('Config directory preserved');
      }
      
      log('\n  aether-cli has been uninstalled', C.green);
      log('  You may need to restart your terminal', C.dim);
      log('  Thanks for trying aether-cli!\n', C.dim);
      
      resolve();
    });
  });
}

function showUsage() {
  log(`
${C.bright}${C.cyan}aether-cli install${C.reset} — Install or upgrade aether-cli

${C.bright}SYNOPSIS${C.reset}
    aether-cli install [--force] [--rpc <url>] [--skip-rpc-check]
    aether-cli install --uninstall

${C.bright}DESCRIPTION${C.reset}
    Installs or upgrades @jellylegsai/aether-cli globally via npm.
    Creates ~/.aether/config.json with your RPC endpoint.
    Checks Node.js version and disk space.
    Configures PATH for CLI access.

${C.bright}OPTIONS${C.reset}
    --force              Overwrite existing config
    --rpc <url>          RPC endpoint URL (default: http://127.0.0.1:8899)
    --backup-rpc <url>   Backup RPC endpoint
    --skip-rpc-check     Skip RPC validation
    --registry <url>     npm registry URL
    --package <name>     Package to install (default: @jellylegsai/aether-cli)
    --uninstall          Remove aether-cli and optional config
    --help, -h           Show this help

${C.bright}EXAMPLES${C.reset}
    aether-cli install
    aether-cli install --rpc https://mainnet.aether.network:8899
    aether-cli install --skip-rpc-check --force
    aether-cli install --uninstall

${C.bright}PLATFORM NOTES${C.reset}
    ${C.cyan}Windows:${C.reset} Adds npm global bin to PATH via Environment Variables
    ${C.cyan}macOS:${C.reset}  Add to ~/.zshrc: export PATH="\$(npm root -g)/aether-cli:$PATH"
    ${C.cyan}Linux:${C.reset}   Same as macOS

${C.bright}SYSTEM REQUIREMENTS${C.reset}
    • Node.js >= 14.0.0
    • npm >= 6.0.0
    • 100MB free disk space
    • Network access for npm and RPC endpoints

${C.bright}QUICK INSTALL (via npm)${C.reset}
    npm install -g @jellylegsai/aether-cli
    aether-cli --version

${C.bright}QUICK INSTALL (via script)${C.reset}
    curl -sSL https://get.aether.network/install.sh | bash
`, C.reset);
}

// ============================================================================
// Main Installer
// ============================================================================

async function main() {
  const args = parseArgs();
  
  // Check for uninstall
  if (args.uninstall) {
    await showUninstall();
    return;
  }
  
  // Check for help
  if (args.help) {
    showUsage();
    return;
  }
  
  console.log(`
${C.bright}${C.cyan}
  ╔═══════════════════════════════════════════════════════════╗
  ║         aether-cli Installer  v${VERSION.toString().padEnd(32)}║
  ║         Platform: ${platform.toUpperCase().padEnd(40)}║
  ╚═══════════════════════════════════════════════════════════╝${C.reset}
`);
  
  log(`  ${C.dim}Node.js ${getNodeVersion()} | npm ${getNpmVersion()}${C.reset}\n`);
  
  // Run installation steps
  const preCheck = await stepPreInstall(args);
  if (!preCheck.ok) {
    error('Pre-installation checks failed');
    process.exit(1);
  }
  
  separator();
  
  const npmResult = await stepNpmInstall(args);
  if (!npmResult.ok) {
    error('npm installation failed');
    process.exit(1);
  }
  
  separator();
  
  const configResult = await stepConfigInit(args);
  
  separator();
  
  const pathResult = await stepPathSetup(args);
  
  separator();
  
  const postResult = await stepPostInstall(args);
  
  separator();
  
  header('INSTALLATION COMPLETE');
  ok('aether-cli is ready to use!');
  
  log('\n  Next steps:', C.bright);
  log('    aether-cli --version     Verify installation', C.dim);
  log('    aether-cli doctor         Run system checks', C.dim);
  log('    aether-cli init           Start onboarding', C.dim);
  log('    aether-cli --help         Show all commands', C.dim);
  
  if (pathResult.pathSetup === 'needs_config') {
    log('\n  ⚠ PATH update required:', C.yellow);
    log(`    Add this to your shell profile:`, C.dim);
    log(`    export PATH="${pathResult.paths[0]}:$PATH"`, C.cyan);
  }
  
  log('\n');
}

function parseArgs() {
  const rawArgs = process.argv.slice(2);
  const args = {
    force: rawArgs.includes('--force') || rawArgs.includes('-f'),
    uninstall: rawArgs.includes('--uninstall'),
    help: rawArgs.includes('--help') || rawArgs.includes('-h'),
    skipRpcCheck: rawArgs.includes('--skip-rpc-check'),
    rpc: null,
    backupRpc: null,
    registry: null,
    package: null,
  };
  
  const rpcIdx = rawArgs.findIndex(a => a === '--rpc' || a === '-r');
  if (rpcIdx !== -1 && rawArgs[rpcIdx + 1] && !rawArgs[rpcIdx + 1].startsWith('--')) {
    args.rpc = rawArgs[rpcIdx + 1];
  }
  
  const backupIdx = rawArgs.findIndex(a => a === '--backup-rpc');
  if (backupIdx !== -1 && rawArgs[backupIdx + 1] && !rawArgs[backupIdx + 1].startsWith('--')) {
    args.backupRpc = rawArgs[backupIdx + 1];
  }
  
  const regIdx = rawArgs.findIndex(a => a === '--registry');
  if (regIdx !== -1 && rawArgs[regIdx + 1] && !rawArgs[regIdx + 1].startsWith('--')) {
    args.registry = rawArgs[regIdx + 1];
  }
  
  const pkgIdx = rawArgs.findIndex(a => a === '--package');
  if (pkgIdx !== -1 && rawArgs[pkgIdx + 1] && !rawArgs[pkgIdx + 1].startsWith('--')) {
    args.package = rawArgs[pkgIdx + 1];
  }
  
  return args;
}

// Export as installCommand for CLI use
function installCommand() {
  main().catch(err => {
    console.error(`\n${C.red}✗ Installation failed:${C.reset}`, err.message, '\n');
    process.exit(1);
  });
}

// Run if called directly
if (require.main === module) {
  installCommand();
}

module.exports = { installCommand };
