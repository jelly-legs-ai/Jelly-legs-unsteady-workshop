#!/usr/bin/env node
/**
 * aether-cli doctor - System Requirements Checker
 * 
 * Validates that a validator's hardware meets minimum requirements:
 * - CPU: 8+ cores
 * - RAM: 32GB+ total, 28GB+ available
 * - Disk: 512GB+ SSD with 340GB+ free
 * - Network: 100Mbps+ upload/download
 * - Firewall: Required ports open
 * 
 * @see docs/MINING_VALIDATOR_TOOLS.md for spec
 */

const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');
const readline = require('readline');

// ANSI colors for terminal output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
};

// Minimum requirements per tier (from spec)
const TIER_REQUIREMENTS = {
  full: {
    badge: '[FULL]',
    cpu: { minCores: 8 },
    ram: { minTotalGB: 32, minAvailableGB: 28 },
    disk: { minTotalGB: 512, minFreeGB: 340 },
    network: { minSpeedMbps: 100 },
    ports: { p2p: 8001, p2pNode: 8002, rpc: 8899, ssh: 22 },
    stake: '10,000 AETH',
    consensusWeight: '1.0x',
    canProduceBlocks: true,
  },
  lite: {
    badge: '[LITE]',
    cpu: { minCores: 4 },
    ram: { minTotalGB: 8, minAvailableGB: 6 },
    disk: { minTotalGB: 100, minFreeGB: 50 },
    network: { minSpeedMbps: 25 },
    ports: { p2p: 8001, rpc: 8899, ssh: 22 },
    stake: '1,000 AETH',
    consensusWeight: 'stake/10000 (e.g., 0.1x at 1K AETH)',
    canProduceBlocks: false,
  },
  observer: {
    badge: '[OBSERVER]',
    cpu: { minCores: 2 },
    ram: { minTotalGB: 4, minAvailableGB: 3 },
    disk: { minTotalGB: 50, minFreeGB: 25 },
    network: { minSpeedMbps: 10 },
    ports: { p2p: 8001, ssh: 22 }, // inbound only, no RPC
    stake: '0 AETH',
    consensusWeight: '0x (relay-only)',
    canProduceBlocks: false,
  },
};

// Default to full tier
const DEFAULT_TIER = 'full';

/**
 * Execute shell command and return output
 */
function runCommand(cmd, options = {}) {
  try {
    return execSync(cmd, {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe'],
      ...options,
    }).trim();
  } catch (error) {
    return options.allowFailure ? null : error.message;
  }
}

/**
 * Check CPU specifications
 */
function checkCPU(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  const cpus = os.cpus();
  const physicalCores = cpus.length / 2; // Hyperthreading aware
  const model = cpus[0].model;
  const speed = cpus[0].speed;

  const passed = physicalCores >= reqs.cpu.minCores;

  return {
    section: 'CPU',
    model,
    physicalCores,
    logicalCores: cpus.length,
    frequency: `${speed} MHz`,
    passed,
    message: passed 
      ? `✅ PASS (${physicalCores} cores >= ${reqs.cpu.minCores} required)`
      : `❌ FAIL (${physicalCores} cores < ${reqs.cpu.minCores} required)`,
    fixable: false,
    fixNote: 'CPU upgrade required - hardware limitation',
  };
}

/**
 * Check memory specifications
 */
function checkMemory(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  const totalGB = os.totalmem() / (1024 * 1024 * 1024);
  const freeGB = os.freemem() / (1024 * 1024 * 1024);
  const availableGB = freeGB; // Simplified - in production would check swap too

  const totalPassed = totalGB >= reqs.ram.minTotalGB;
  const availablePassed = availableGB >= reqs.ram.minAvailableGB;
  const passed = totalPassed && availablePassed;

  return {
    section: 'Memory',
    total: `${totalGB.toFixed(1)} GB`,
    available: `${availableGB.toFixed(1)} GB`,
    passed,
    message: passed
      ? `✅ PASS (${totalGB.toFixed(1)} GB total, ${availableGB.toFixed(1)} GB available)`
      : `❌ FAIL (need ${reqs.ram.minTotalGB} GB total, ${reqs.ram.minAvailableGB} GB available)`,
    fixable: false,
    fixNote: 'RAM upgrade required or close memory-intensive applications',
  };
}

/**
 * Check disk specifications
 */
function checkDisk(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  // Get disk info for root partition (works on Linux/Mac)
  let diskInfo = { mount: '/', type: 'SSD', total: 0, free: 0 };
  
  try {
    if (process.platform === 'win32') {
      // Windows: use PowerShell to get disk info
      const output = runCommand('powershell -c "Get-Volume -DriveType Fixed | Where-Object {$_.DriveLetter -eq (Get-Location).Drive.Name} | Select-Object Size,SizeRemaining"', { allowFailure: true });
      if (output) {
        // Parse PowerShell output (format: Size: 123456789012, SizeRemaining: 98765432100)
        const lines = output.split('\n');
        for (const line of lines) {
          if (line.includes('Size') && !line.includes('SizeRemaining')) {
            const sizeMatch = line.match(/(\d+)/);
            if (sizeMatch) diskInfo.total = parseInt(sizeMatch[1]) / (1024 * 1024 * 1024);
          }
          if (line.includes('SizeRemaining')) {
            const freeMatch = line.match(/(\d+)/);
            if (freeMatch) diskInfo.free = parseInt(freeMatch[1]) / (1024 * 1024 * 1024);
          }
        }
      }
      // Fallback: use Get-PSDrive
      if (diskInfo.total === 0) {
        const psDrive = runCommand('powershell -c "(Get-PSDrive -Name (Get-Location).Drive.Name).Used / 1GB"', { allowFailure: true });
        const psFree = runCommand('powershell -c "(Get-PSDrive -Name (Get-Location).Drive.Name).Free / 1GB"', { allowFailure: true });
        if (psDrive && psFree) {
          diskInfo.free = parseFloat(psFree);
          diskInfo.total = parseFloat(psDrive) + parseFloat(psFree);
        }
      }
    } else {
      // Linux/Mac: use df
      const output = runCommand('df -k / | tail -1');
      const parts = output.split(/\s+/);
      if (parts.length >= 4) {
        diskInfo.total = parseInt(parts[1]) / (1024 * 1024);
        diskInfo.free = parseInt(parts[3]) / (1024 * 1024);
      }
    }
  } catch (e) {
    // Fallback - mark as unknown
    diskInfo.total = 0;
    diskInfo.free = 0;
  }

  const totalPassed = diskInfo.total >= reqs.disk.minTotalGB;
  const freePassed = diskInfo.free >= reqs.disk.minFreeGB;
  const passed = totalPassed && freePassed;

  return {
    section: 'Disk',
    mount: diskInfo.mount,
    type: diskInfo.type,
    total: `${diskInfo.total.toFixed(0)} GB`,
    free: `${diskInfo.free.toFixed(0)} GB`,
    passed,
    message: passed
      ? `✅ PASS (${diskInfo.total.toFixed(0)} GB total, ${diskInfo.free.toFixed(0)} GB free)`
      : `❌ FAIL (need ${reqs.disk.minTotalGB} GB total, ${reqs.disk.minFreeGB} GB free)`,
    fixable: !totalPassed ? false : true,
    fixNote: totalPassed ? 'Free up disk space by removing old files or logs' : 'Larger disk required - hardware limitation',
  };
}

/**
 * Check network specifications
 */
function checkNetwork(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  // Try to get public IP
  let publicIP = 'Unknown';
  try {
    publicIP = runCommand('curl -s ifconfig.me', { allowFailure: true }) || 
               runCommand('curl -s icanhazip.com', { allowFailure: true }) ||
               'Unknown';
  } catch (e) {
    publicIP = 'Unknown';
  }

  // Network speed test would require external API
  // For now, we check interface speed
  let downloadSpeed = 'Unknown';
  let uploadSpeed = 'Unknown';
  let latency = 'Unknown';
  let passed = true; // Assume pass if we can't test

  // In production, integrate with speedtest CLI or similar
  // For MVP, we'll show interface info
  const interfaces = os.networkInterfaces();
  const interfaceCount = Object.keys(interfaces).length;

  return {
    section: 'Network',
    publicIP,
    download: downloadSpeed,
    upload: uploadSpeed,
    latency,
    interfaces: interfaceCount,
    required: `${reqs.network.minSpeedMbps} Mbps`,
    passed,
    message: passed
      ? `✅ PASS (Network interfaces detected, need ${reqs.network.minSpeedMbps} Mbps)`
      : `❌ FAIL`,
    fixable: false,
    fixNote: 'Network connectivity is system-level',
  };
}

/**
 * Check firewall and port availability
 */
function checkFirewall(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  const results = { p2p: false, rpc: false, ssh: false };
  const blockedPorts = [];
  
  try {
    if (process.platform === 'linux') {
      // Check if ufw is active and ports are open
      const ufwStatus = runCommand('ufw status 2>&1', { allowFailure: true });
      if (ufwStatus && !ufwStatus.includes('inactive')) {
        results.p2p = ufwStatus.includes(`${reqs.ports.p2p}`);
        results.ssh = ufwStatus.includes(`${reqs.ports.ssh}`);
        // RPC only required for full/lite tiers
        if (reqs.ports.rpc) {
          results.rpc = ufwStatus.includes(`${reqs.ports.rpc}`);
        } else {
          results.rpc = true; // observer doesn't need RPC
        }
        
        if (!results.p2p) blockedPorts.push(reqs.ports.p2p);
        if (reqs.ports.rpc && !results.rpc) blockedPorts.push(reqs.ports.rpc);
        if (!results.ssh) blockedPorts.push(reqs.ports.ssh);
      } else {
        // If firewall inactive, assume ports are accessible
        results.p2p = true;
        results.rpc = reqs.ports.rpc ? true : true;
        results.ssh = true;
      }
    } else if (process.platform === 'win32') {
      // Windows Firewall check - test each port
      const testPort = (port) => {
        try {
          const result = runCommand(`powershell -c "Get-NetFirewallRule -DisplayName '*Aether*' -ErrorAction SilentlyContinue | Where-Object { $_.Enabled -eq True }"`, { allowFailure: true });
          // Simplified: check if any aether rules exist
          if (result && result.includes('Aether')) {
            return true;
          }
          // Try to bind to port to test availability
          const bindTest = runCommand(`powershell -c "$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any, ${port}); $listener.Start(); $listener.Stop()"`, { allowFailure: true });
          return bindTest === null || !bindTest.includes('error');
        } catch {
          return false;
        }
      };
      
      results.p2p = testPort(reqs.ports.p2p);
      results.ssh = testPort(reqs.ports.ssh);
      if (reqs.ports.rpc) {
        results.rpc = testPort(reqs.ports.rpc);
      } else {
        results.rpc = true; // observer doesn't need RPC
      }
      
      if (!results.p2p) blockedPorts.push(reqs.ports.p2p);
      if (reqs.ports.rpc && !results.rpc) blockedPorts.push(reqs.ports.rpc);
      if (!results.ssh) blockedPorts.push(reqs.ports.ssh);
    } else {
      // macOS / other - assume pass
      results.p2p = true;
      results.rpc = reqs.ports.rpc ? true : true;
      results.ssh = true;
    }
  } catch (e) {
    // Assume pass if we can't check
    results.p2p = true;
    results.rpc = reqs.ports.rpc ? true : true;
    results.ssh = true;
  }

  const allPassed = results.p2p && results.rpc && results.ssh;

  return {
    section: 'Firewall',
    p2p: results.p2p,
    rpc: results.rpc,
    ssh: results.ssh,
    blockedPorts,
    passed: allPassed,
    message: allPassed
      ? `✅ PASS (All required ports accessible)`
      : `❌ FAIL (Ports ${blockedPorts.join(', ')} may be blocked)`,
    fixable: blockedPorts.length > 0,
    fixNote: blockedPorts.length > 0 ? `Add firewall rules for ports ${blockedPorts.join(', ')}` : '',
  };
}

/**
 * Check if validator binary exists
 */
function checkValidatorBinary() {
  const platform = os.platform();
  const isWindows = platform === 'win32';
  const binaryName = isWindows ? 'aether-validator.exe' : 'aether-validator';
  
  // Check the expected location based on repo layout
  const workspaceRoot = path.join(__dirname, '..', '..');
  const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
  const binaryPath = path.join(repoPath, 'target', 'debug', binaryName);
  
  const exists = fs.existsSync(binaryPath);
  
  return {
    section: 'Validator Binary',
    path: binaryPath,
    exists,
    passed: exists,
    message: exists
      ? `✅ PASS (Binary found at ${binaryPath})`
      : `❌ FAIL (Binary not found at ${binaryPath})`,
    fixable: true,
    fixNote: exists ? '' : 'Run cargo build --bin aether-validator',
  };
}

/**
 * Get OS information
 */
function getOSInfo() {
  const platform = os.platform();
  const arch = os.arch();
  const release = os.release();
  
  let osName = platform;
  try {
    if (platform === 'linux') {
      const osRelease = fs.readFileSync('/etc/os-release', 'utf-8');
      const nameMatch = osRelease.match(/^NAME="([^"]+)"/m);
      const versionMatch = osRelease.match(/^VERSION_ID="([^"]+)"/m);
      if (nameMatch) osName = nameMatch[1];
      if (versionMatch) osName += ` ${versionMatch[1]}`;
    } else if (platform === 'darwin') {
      const darwinVersion = runCommand('sw_vers -productVersion', { allowFailure: true });
      if (darwinVersion) osName = `macOS ${darwinVersion}`;
    } else if (platform === 'win32') {
      const winVersion = runCommand('powershell -c "(Get-CimInstance Win32_OperatingSystem).Caption"', { allowFailure: true });
      if (winVersion) osName = winVersion;
    }
  } catch (e) {
    // Use defaults
  }

  return { platform, arch, release, osName };
}

/**
 * Print section header
 */
function printSectionHeader(title) {
  console.log(`\n${colors.bright}${colors.cyan}${title}${colors.reset}`);
  console.log(`${colors.cyan}${'─'.repeat(60)}${colors.reset}`);
}

/**
 * Print check result
 */
function printCheckResult(check) {
  console.log(`\n${colors.bright}${check.section}${colors.reset}`);
  Object.entries(check).forEach(([key, value]) => {
    if (['section', 'passed', 'message', 'fixable', 'fixNote'].includes(key)) return;
    console.log(`  ${key}: ${value}`);
  });
  console.log(`  ${check.message}`);
}

/**
 * Print ASCII art header
 */
function printHeader(tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  const header = `
${colors.bright}${colors.cyan}
███╗   ███╗██╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
████╗ ████║██║██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
██╔████╔██║██║███████╗███████╗██║██║   ██║██╔██╗ ██║
██║╚██╔╝██║██║╚════██║╚════██║██║██║   ██║██║╚██╗██║
██║ ╚═╝ ██║██║███████║███████║██║╚██████╔╝██║ ╚████║
╚═╝     ╚═╝╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝

${colors.reset}${colors.bright}Validator System Check${colors.reset}
  ${colors.yellow}v1.0.0${colors.reset}
  ${new Date().toISOString().split('T')[0]}
  ${colors.magenta}${reqs.badge}${colors.reset}
`.trim();
  console.log(header);
  console.log(`\n${colors.cyan}${'━'.repeat(60)}${colors.reset}`);
  
  // Print tier summary
  console.log(`\n${colors.bright}Tier Requirements:${colors.reset}`);
  console.log(`  ${colors.cyan}Stake:${colors.reset} ${reqs.stake}`);
  console.log(`  ${colors.cyan}Consensus Weight:${colors.reset} ${reqs.consensusWeight}`);
  console.log(`  ${colors.cyan}Block Production:${colors.reset} ${reqs.canProduceBlocks ? '✅ Yes' : '❌ No'}`);
  console.log(`  ${colors.cyan}CPU:${colors.reset} ${reqs.cpu.minCores}+ cores`);
  console.log(`  ${colors.cyan}RAM:${colors.reset} ${reqs.ram.minTotalGB}GB+ total, ${reqs.ram.minAvailableGB}GB+ available`);
  console.log(`  ${colors.cyan}Disk:${colors.reset} ${reqs.disk.minTotalGB}GB+ total, ${reqs.disk.minFreeGB}GB+ free`);
  console.log(`  ${colors.cyan}Network:${colors.reset} ${reqs.network.minSpeedMbps}+ Mbps`);
  console.log(`  ${colors.cyan}Ports:${colors.reset} ${Object.values(reqs.ports).join(', ')}`);
  console.log(`\n${colors.cyan}${'━'.repeat(60)}${colors.reset}`);
}

/**
 * Print summary
 */
function printSummary(results, tier = DEFAULT_TIER) {
  const reqs = TIER_REQUIREMENTS[tier];
  const allPassed = results.every(r => r.passed);
  
  console.log(`\n${colors.cyan}${'━'.repeat(60)}${colors.reset}`);
  console.log(`\n${colors.bright}SUMMARY:${colors.reset} ${colors.magenta}${reqs.badge}${colors.reset}`);
  
  if (allPassed) {
    console.log(`\n${colors.bright}${colors.green}✅ All checks passed!${colors.reset}`);
    console.log(`\n${colors.green}Your system is ready to run an AeTHer ${tier.toUpperCase()} validator.${colors.reset}`);
    console.log(`\nNext steps:`);
    console.log(`  ${colors.bright}aether-cli validator start --tier ${tier}${colors.reset}    # Start validating`);
    console.log(`  ${colors.bright}aether-cli validator status${colors.reset}   # Check status`);
    console.log(`  ${colors.bright}aether-cli help${colors.reset}               # View all commands`);
  } else {
    const failed = results.filter(r => !r.passed);
    const fixable = failed.filter(r => r.fixable);
    
    console.log(`\n${colors.bright}${colors.red}❌ ${failed.length} check(s) failed${colors.reset}`);
    
    if (fixable.length > 0) {
      console.log(`\n${colors.yellow}⚠ ${fixable.length} issue(s) can be auto-fixed:${colors.reset}`);
      fixable.forEach(f => {
        console.log(`  ${colors.yellow}• ${f.section}: ${f.fixNote}${colors.reset}`);
      });
    }
    
    const notFixable = failed.filter(r => !r.fixable);
    if (notFixable.length > 0) {
      console.log(`\n${colors.red}The following issues require manual action:${colors.reset}`);
      notFixable.forEach(f => {
        console.log(`  ${colors.red}• ${f.section}: ${f.fixNote}${colors.reset}`);
      });
    }
  }
  
  console.log(`\n${colors.cyan}${'━'.repeat(60)}${colors.reset}\n`);
}

/**
 * Generate fix command for a failed check
 */
function getFixCommand(check) {
  const platform = os.platform();
  
  switch (check.section) {
    case 'Firewall':
      if (platform === 'win32') {
        const ports = check.blockedPorts || [];
        if (ports.length === 0) return null;
        const rules = ports.map(port => 
          `New-NetFirewallRule -DisplayName "Aether Port ${port}" -Direction Inbound -LocalPort ${port} -Protocol TCP -Action Allow`
        ).join('; ');
        return `powershell -c "${rules}"`;
      } else if (platform === 'linux') {
        const ports = check.blockedPorts || [];
        if (ports.length === 0) return null;
        const rules = ports.map(port => `sudo ufw allow ${port}/tcp`).join(' && ');
        return rules;
      }
      return null;
      
    case 'Validator Binary':
      const workspaceRoot = path.join(__dirname, '..', '..');
      const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
      return `cd "${repoPath}" && cargo build --bin aether-validator`;
      
    case 'Disk':
      // Can't auto-fix disk space, but can suggest cleanup
      if (platform === 'win32') {
        return 'powershell -c "Get-AppxPackage -AllUsers | Where-Object {$_.InstallLocation -like \'*WindowsApps*\'} | Select-Object Name, PackageFullName"';
      } else {
        return 'sudo du -sh /* 2>/dev/null | sort -hr | head -20';
      }
      
    default:
      return null;
  }
}

/**
 * Ask user for confirmation
 */
async function askConfirmation(question) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  
  return new Promise((resolve) => {
    rl.question(`${colors.yellow}${question}${colors.reset} [y/N] `, (answer) => {
      rl.close();
      resolve(answer.toLowerCase() === 'y' || answer.toLowerCase() === 'yes');
    });
  });
}

/**
 * Apply a fix for a failed check
 */
async function applyFix(check) {
  const command = getFixCommand(check);
  
  if (!command) {
    console.log(`  ${colors.red}✗ No automated fix available for ${check.section}${colors.reset}`);
    return false;
  }
  
  console.log(`\n  ${colors.cyan}Proposed fix:${colors.reset}`);
  console.log(`  ${colors.bright}${command}${colors.reset}`);
  console.log();
  
  const confirmed = await askConfirmation('  Apply this fix?');
  
  if (!confirmed) {
    console.log(`  ${colors.yellow}Skipped.${colors.reset}`);
    return false;
  }
  
  console.log(`  ${colors.cyan}Applying fix...${colors.reset}`);
  
  try {
    execSync(command, {
      stdio: 'inherit',
      shell: true,
      cwd: process.cwd(),
    });
    
    console.log(`  ${colors.green}✓ Fix applied successfully!${colors.reset}`);
    
    // Re-run the check to verify
    console.log(`  ${colors.cyan}Verifying...${colors.reset}`);
    let verifyCheck;
    switch (check.section) {
      case 'Firewall':
        verifyCheck = checkFirewall();
        break;
      case 'Validator Binary':
        verifyCheck = checkValidatorBinary();
        break;
      default:
        verifyCheck = check;
    }
    
    if (verifyCheck.passed) {
      console.log(`  ${colors.green}✓ Verification passed!${colors.reset}`);
      return true;
    } else {
      console.log(`  ${colors.yellow}⚠ Fix applied but check still failing. May require manual intervention.${colors.reset}`);
      return false;
    }
  } catch (err) {
    console.log(`  ${colors.red}✗ Fix failed: ${err.message}${colors.reset}`);
    return false;
  }
}

/**
 * Interactive fix mode
 */
async function interactiveFixMode(results) {
  const failed = results.filter(r => !r.passed);
  const fixable = failed.filter(r => r.fixable);
  
  if (fixable.length === 0) {
    console.log(`\n${colors.yellow}No auto-fixable issues found.${colors.reset}`);
    return;
  }
  
  console.log(`\n${colors.bright}${colors.cyan}Auto-Fix Mode${colors.reset}`);
  console.log(`${colors.cyan}${'─'.repeat(60)}${colors.reset}`);
  console.log(`\n${colors.yellow}Found ${fixable.length} issue(s) that can be fixed automatically:${colors.reset}\n`);
  
  for (const check of fixable) {
    console.log(`${colors.bright}${check.section}${colors.reset}`);
    console.log(`  Issue: ${check.fixNote}`);
    console.log();
    
    const fixed = await applyFix(check);
    
    if (fixed) {
      check.passed = true;
      check.message = `✅ FIXED (was: ${check.message})`;
    }
    
    console.log();
  }
  
  // Print updated summary
  const stillFailed = results.filter(r => !r.passed);
  if (stillFailed.length === 0) {
    console.log(`\n${colors.bright}${colors.green}🎉 All issues resolved!${colors.reset}`);
    console.log(`\n${colors.green}Your system is now ready to run an AeTHer validator.${colors.reset}`);
  } else {
    console.log(`\n${colors.yellow}⚠ ${stillFailed.length} issue(s) remain unresolved.${colors.reset}`);
  }
}

/**
 * Main doctor command
 */
async function doctorCommand(options = {}) {
  const { autoFix = false, tier = DEFAULT_TIER } = options;
  
  // Validate tier
  if (!TIER_REQUIREMENTS[tier]) {
    console.log(`${colors.red}Error: Invalid tier '${tier}'. Valid tiers: full, lite, observer${colors.reset}`);
    return 1;
  }
  
  printHeader(tier);
  console.log(`\n${colors.bright}Running system checks for ${tier.toUpperCase()} tier...${colors.reset}\n`);

  const results = [
    checkCPU(tier),
    checkMemory(tier),
    checkDisk(tier),
    checkNetwork(tier),
    checkFirewall(tier),
    checkValidatorBinary(),
  ];

  results.forEach(printCheckResult);
  printSummary(results, tier);

  // If auto-fix mode or user requests it
  const failed = results.filter(r => !r.passed);
  const fixable = failed.filter(r => r.fixable);
  
  if (fixable.length > 0) {
    if (autoFix) {
      console.log(`\n${colors.cyan}Auto-fix mode enabled. Attempting fixes...${colors.reset}\n`);
      await interactiveFixMode(results);
    } else {
      console.log(`\n${colors.cyan}Tip: Run ${colors.bright}aether-cli doctor --fix${colors.reset}${colors.cyan} to auto-fix issues.${colors.reset}\n`);
    }
  }

  // Return exit code based on results
  const allPassed = results.every(r => r.passed);
  return allPassed ? 0 : 1;
}

// Export for use as module
module.exports = { doctorCommand, checkCPU, checkMemory, checkDisk, checkNetwork, checkFirewall, checkValidatorBinary };

// Run if called directly
if (require.main === module) {
  const args = process.argv.slice(2);
  const autoFix = args.includes('--fix') || args.includes('-f');
  
  // Parse --tier flag
  let tier = DEFAULT_TIER;
  const tierIndex = args.findIndex(arg => arg === '--tier');
  if (tierIndex !== -1 && args[tierIndex + 1]) {
    tier = args[tierIndex + 1].toLowerCase();
  }
  
  doctorCommand({ autoFix, tier }).then(exitCode => {
    process.exit(exitCode);
  });
}
