/**
 * aether-cli logs
 * 
 * Tails and colourises validator log files.
 * Supports both live process output and existing log files.
 * 
 * Usage:
 *   aether-cli logs                    # Tail default validator log
 *   aether-cli logs --file <path>      # Tail specific log file
 *   aether-cli logs --follow           # Follow mode (like tail -f)
 *   aether-cli logs --lines <n>        # Show last N lines
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const readline = require('readline');

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  black: '\x1b[30m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  bgRed: '\x1b[41m',
  bgYellow: '\x1b[43m',
  bgGreen: '\x1b[42m',
};

// Log level color mapping
const levelColors = {
  ERROR: colors.bgRed + colors.white + colors.bright,
  WARN: colors.bgYellow + colors.black,
  INFO: colors.green,
  DEBUG: colors.blue,
  TRACE: colors.dim,
};

// Regex patterns for log levels
const logPatterns = {
  ERROR: /\bERROR\b|\bERR\b|\bFATAL\b|\[error\]|\[ERROR\]/i,
  WARN: /\bWARN\b|\bWARNING\b|\bWARN\b|\[warn\]|\[WARNING\]/i,
  INFO: /\bINFO\b|\[info\]|\[INFO\]/i,
  DEBUG: /\bDEBUG\b|\[debug\]|\[DEBUG\]/i,
  TRACE: /\bTRACE\b|\[trace\]|\[TRACE\]/i,
};

/**
 * Detect log level in a line and return colorized version
 */
function colorizeLogLine(line) {
  // Check for log levels in order of severity
  for (const [level, pattern] of Object.entries(logPatterns)) {
    if (pattern.test(line)) {
      const color = levelColors[level] || colors.reset;
      return `${color}${line}${colors.reset}`;
    }
  }
  
  // Check for common patterns
  if (/failed|error|exception|crash|panic/i.test(line)) {
    return `${colors.red}${line}${colors.reset}`;
  }
  if (/success|complete|ready|started|connected/i.test(line)) {
    return `${colors.green}${line}${colors.reset}`;
  }
  if (/warning|timeout|retry|slow/i.test(line)) {
    return `${colors.yellow}${line}${colors.reset}`;
  }
  
  return line;
}

/**
 * Print the logs banner
 */
function printBanner(options) {
  const timestamp = new Date().toISOString().split('T')[0];
  
  console.log(`
${colors.cyan}╔═══════════════════════════════════════════════════════════════╗
${colors.cyan}║                                                               ║
${colors.cyan}║   ${colors.bright}AETHER LOGS${colors.reset}${colors.cyan}                                              ║
${colors.cyan}║   ${colors.bright}Validator Log Viewer${colors.reset}${colors.cyan}                                    ║
${colors.cyan}║                                                               ║
${colors.cyan}╚═══════════════════════════════════════════════════════════════╝${colors.reset}
  `);
  
  console.log(`  ${colors.bright}Log File:${colors.reset} ${options.filePath}`);
  console.log(`  ${colors.bright}Mode:${colors.reset} ${options.follow ? 'Follow (tail -f)' : 'Static'}`);
  console.log(`  ${colors.bright}Lines:${colors.reset} ${options.lines === -1 ? 'All' : `Last ${options.lines}`}`);
  console.log();
  console.log(`  ${colors.dim}Legend: ${colors.bgRed + colors.white}ERROR${colors.reset} ${colors.bgYellow + colors.black}WARN${colors.reset} ${colors.green}INFO${colors.reset} ${colors.blue}DEBUG${colors.reset} ${colors.dim}TRACE${colors.reset}${colors.reset}`);
  console.log(`  ${colors.dim}Press Ctrl+C to exit${colors.reset}`);
  console.log();
  console.log(`${colors.cyan}${'─'.repeat(60)}${colors.reset}`);
}

/**
 * Find default validator log file
 */
function findDefaultLogFile() {
  const workspaceRoot = path.join(__dirname, '..', '..');
  const repoPath = path.join(workspaceRoot, 'Jelly-legs-unsteady-workshop');
  
  // Common log file locations
  const candidates = [
    path.join(repoPath, 'validator.log'),
    path.join(repoPath, 'testnet-debug.log'),
    path.join(repoPath, 'testnet', 'node1.log'),
    path.join(repoPath, 'aether-validator.log'),
    path.join(process.cwd(), 'validator.log'),
  ];
  
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  
  // Return the most likely location even if it doesn't exist yet
  return path.join(repoPath, 'validator.log');
}

/**
 * Parse command line args
 */
function parseArgs() {
  const args = process.argv.slice(3); // Skip 'aether-cli logs'
  
  const options = {
    filePath: null,
    follow: false,
    lines: 50,
  };
  
  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--file':
      case '-f':
        options.filePath = args[++i];
        break;
      case '--follow':
      case '-F':
        options.follow = true;
        break;
      case '--lines':
      case '-n':
        options.lines = parseInt(args[++i], 10) || 50;
        break;
      case '--help':
      case '-h':
        showHelp();
        process.exit(0);
    }
  }
  
  // Default file if not specified
  if (!options.filePath) {
    options.filePath = findDefaultLogFile();
  }
  
  return options;
}

/**
 * Show help message
 */
function showHelp() {
  console.log(`
${colors.bright}aether-cli logs${colors.reset} - Validator Log Viewer

${colors.cyan}Usage:${colors.reset}
  aether-cli logs [options]

${colors.cyan}Options:${colors.reset}
  --file, -f <path>    Path to log file (default: auto-detect)
  --follow, -F         Follow mode (like tail -f)
  --lines, -n <num>    Number of lines to show (default: 50, use -1 for all)
  --help, -h           Show this help message

${colors.cyan}Examples:${colors.reset}
  aether-cli logs                      # Tail default validator log
  aether-cli logs --follow             # Follow live logs
  aether-cli logs --file ./my.log -n 100
  aether-cli logs -F -n -1             # Follow all lines

${colors.cyan}Log Level Colors:${colors.reset}
  ERROR  ${colors.bgRed + colors.white}Red background${colors.reset}
  WARN   ${colors.bgYellow + colors.black}Yellow background${colors.reset}
  INFO   ${colors.green}Green${colors.reset}
  DEBUG  ${colors.blue}Blue${colors.reset}
  TRACE  ${colors.dim}Dim${colors.reset}
`);
}

/**
 * Read and display log file (static mode)
 */
function readLogFile(filePath, lines) {
  if (!fs.existsSync(filePath)) {
    console.log(`  ${colors.yellow}⚠ Log file not found: ${filePath}${colors.reset}`);
    console.log(`  ${colors.dim}Start the validator first to generate logs.${colors.reset}`);
    return;
  }
  
  const content = fs.readFileSync(filePath, 'utf-8');
  const allLines = content.split('\n').filter(line => line.trim());
  
  // Get last N lines
  const displayLines = lines === -1 
    ? allLines 
    : allLines.slice(-lines);
  
  displayLines.forEach(line => {
    console.log(colorizeLogLine(line));
  });
}

/**
 * Follow log file (tail -f mode)
 */
function followLogFile(filePath, lines) {
  if (!fs.existsSync(filePath)) {
    console.log(`  ${colors.yellow}⚠ Log file not found: ${filePath}${colors.reset}`);
    console.log(`  ${colors.dim}Waiting for file to be created...${colors.reset}`);
    console.log();
  }
  
  // Use platform-specific tail command
  const platform = process.platform;
  let tailCmd, tailArgs;
  
  if (platform === 'win32') {
    // PowerShell Get-Content -Wait is equivalent to tail -f
    tailCmd = 'powershell';
    tailArgs = [
      '-Command',
      `Get-Content "${filePath}" -Wait -Tail ${lines === -1 ? 1000 : lines} 2>$null`
    ];
  } else {
    tailCmd = 'tail';
    tailArgs = ['-F', '-n', lines === -1 ? '1000' : lines.toString(), filePath];
  }
  
  const tail = spawn(tailCmd, tailArgs, {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  
  tail.stdout.on('data', (data) => {
    const text = data.toString();
    const textLines = text.split('\n');
    textLines.forEach(line => {
      if (line.trim()) {
        console.log(colorizeLogLine(line));
      }
    });
  });
  
  tail.stderr.on('data', (data) => {
    const text = data.toString();
    if (text.includes('No such file') || text.includes('cannot open')) {
      // File doesn't exist yet, wait
      return;
    }
    console.log(`${colors.red}${text}${colors.reset}`);
  });
  
  tail.on('error', (err) => {
    console.log(`${colors.red}Error: ${err.message}${colors.reset}`);
  });
  
  tail.on('close', (code) => {
    if (code !== 0 && code !== null) {
      console.log(`\n${colors.yellow}Log viewer exited with code ${code}${colors.reset}`);
    }
  });
  
  // Handle Ctrl+C
  process.on('SIGINT', () => {
    console.log(`\n${colors.cyan}Logs viewer stopped.${colors.reset}`);
    tail.kill();
    process.exit(0);
  });
}

/**
 * Main logs command
 */
function logsCommand() {
  const options = parseArgs();
  
  printBanner(options);
  
  if (options.follow) {
    followLogFile(options.filePath, options.lines);
  } else {
    readLogFile(options.filePath, options.lines);
  }
}

// Export for use as module
module.exports = { logsCommand, colorizeLogLine, findDefaultLogFile };

// Run if called directly
if (require.main === module) {
  logsCommand();
}
