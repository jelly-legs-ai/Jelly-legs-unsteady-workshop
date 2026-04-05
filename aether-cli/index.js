#!/usr/bin/env node
/**
 * aether-cli - AeTHer Validator Command Line Interface
 * 
 * Main entry point for the validator CLI tool.
 * Provides onboarding, system checks, validator management, and KYC integration.
 */

const { doctorCommand } = require('./commands/doctor');
const { validatorStart } = require('./commands/validator-start');
const { validatorStatus } = require('./commands/validator-status');
const { init } = require('./commands/init');
const { monitorLoop } = require('./commands/monitor');
const { logsCommand } = require('./commands/logs');
const { sdkCommand } = require('./commands/sdk');
const { snapshotCommand } = require('./commands/snapshot');
const { walletCommand } = require('./commands/wallet');
const { networkCommand } = require('./commands/network');
const { validatorsListCommand } = require('./commands/validators');
const { delegationsCommand } = require('./commands/delegations');
const { rewardsCommand } = require('./commands/rewards');
const readline = require('readline');

// CLI version
const VERSION = '1.0.5';

// Parse args early to support flags on commands
function getCommandArgs() {
  return process.argv.slice(2);
}

// Tier colours
const TIER_COLORS = {
  FULL: '\x1b[36m',    // cyan
  LITE: '\x1b[33m',    // yellow
  OBSERVER: '\x1b[32m', // green
  reset: '\x1b[0m',
};

/**
 * Display the interactive main menu
 */
async function showMenu() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const prompt = (q) => new Promise((res) => rl.question(q, res));

  console.log(
    TIER_COLORS.FULL + '\n  ╔═══════════════════════════════════════════════╗\n' +
    '  ║     AETHER CHAIN — Validator Setup Wizard     ║\n' +
    '  ╚═══════════════════════════════════════════════╝' + TIER_COLORS.reset + '\n'
  );

  console.log('  Welcome to AeTHer Chain. What would you like to do?\n');
  console.log('  ' + TIER_COLORS.FULL + '1)' + TIER_COLORS.reset + '  🩺  Doctor  — Check if your system meets requirements');
  console.log('  ' + TIER_COLORS.FULL + '2)' + TIER_COLORS.reset + '  🚀  Start  — Begin validator onboarding (recommended)');
  console.log('  ' + TIER_COLORS.FULL + '3)' + TIER_COLORS.reset + '  📊  Monitor — Watch live validator stats');
  console.log('  ' + TIER_COLORS.FULL + '4)' + TIER_COLORS.reset + '  📋  Logs   — Tail and colourise validator logs');
  console.log('  ' + TIER_COLORS.FULL + '5)' + TIER_COLORS.reset + '  📦  SDK    — Get SDK links and install tools');
  console.log('  ' + TIER_COLORS.FULL + '6)' + TIER_COLORS.reset + '  🌐  Network — Aether network status (slot, peers, TPS)');
  console.log('  ' + TIER_COLORS.FULL + '7)' + TIER_COLORS.reset + '  ❓  Help   — Show all commands\n');
  console.log('  ' + TIER_COLORS.reset + '  Type a number or command name. Press Ctrl+C to exit.\n');

  const VALID_CHOICES = ['1', '2', '3', '4', '5', '6', '7', 'doctor', 'init', 'monitor', 'logs', 'sdk', 'network', 'help'];

  while (true) {
    const answer = (await prompt(`  > `)).trim().toLowerCase();

    if (answer === '' || answer === '1' || answer === 'doctor') {
      rl.close();
      const { doctorCommand } = require('./commands/doctor');
      doctorCommand({ autoFix: false, tier: 'full' });
      return;
    }

    if (answer === '2' || answer === 'init' || answer === 'start') {
      rl.close();
      const { init } = require('./commands/init');
      init();
      return;
    }

    if (answer === '3' || answer === 'monitor') {
      rl.close();
      const { main } = require('./commands/monitor');
      main();
      return;
    }

    if (answer === '4' || answer === 'logs') {
      rl.close();
      const { logsCommand } = require('./commands/logs');
      logsCommand();
      return;
    }

    if (answer === '5' || answer === 'sdk') {
      rl.close();
      const { sdkCommand } = require('./commands/sdk');
const { snapshotCommand } = require('./commands/snapshot');
      sdkCommand();
      return;
    }

    if (answer === '6' || answer === 'network') {
      rl.close();
      const { networkCommand } = require('./commands/network');
      networkCommand();
      return;
    }

    if (answer === '7' || answer === 'help') {
      showHelp();
      console.log("  Press Ctrl+C to exit or select an option above.\n");
      continue;
    }

    console.log(`\n  ⚠️  Unknown option: "${answer}". Type 1-6 or a command name.\n`);
  }
}

// Available commands
const COMMANDS = {
  start: {
    description: 'Launch interactive menu (default if no args) — same as running aether-cli with no arguments',
    handler: () => showMenu(),
  },
  doctor: {
    description: 'Run system requirements checks (CPU/RAM/Disk/Network/Firewall)',
    handler: () => {
      const args = getCommandArgs();
      const autoFix = args.includes('--fix') || args.includes('-f');
      
      // Parse --tier flag
      let tier = 'full';
      const tierIndex = args.findIndex(arg => arg === '--tier');
      if (tierIndex !== -1 && args[tierIndex + 1]) {
        tier = args[tierIndex + 1].toLowerCase();
      }
      
      doctorCommand({ autoFix, tier });
    },
  },
  init: {
    description: 'Start onboarding wizard (generate identity, create stake account, connect to testnet)',
    handler: init,
  },
  'kyc generate': {
    description: 'Generate pre-filled KYC link with pubkey, node ID, signature',
    handler: () => console.log('🚧 kyc generate command under development'),
  },
  monitor: {
    description: 'Real-time validator dashboard (slot, block height, peers, TPS)',
    handler: () => {
      // monitor command runs its own loop
      const { main } = require('./commands/monitor');
      main();
    },
  },
  logs: {
    description: 'Tail validator logs with colour-coded output (ERROR=red, WARN=yellow, INFO=green)',
    handler: logsCommand,
  },
  sdk: {
    description: 'Aether SDK download links and install instructions (JS, Rust, FLUX/ATH tokens)',
    handler: sdkCommand,
  },
  wallet: {
    description: 'Wallet management — create, import, list, default, connect, balance, stake, transfer',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      walletCommand();
    },
  },
  stake: {
    description: 'Stake AETH to a validator — aether stake --validator <addr> --amount <aeth>',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      // Intercept argv so walletCommand receives 'stake' as the subcmd
      const originalArgv = process.argv;
      process.argv = [...originalArgv.slice(0, 2), 'wallet', 'stake', ...originalArgv.slice(3)];
      walletCommand();
      process.argv = originalArgv;
    },
  },
  transfer: {
    description: 'Transfer AETH to another address — aether transfer --to <addr> --amount <aeth>',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      const originalArgv = process.argv;
      process.argv = [...originalArgv.slice(0, 2), 'wallet', 'transfer', ...originalArgv.slice(3)];
      walletCommand();
      process.argv = originalArgv;
    },
  },
  tx: {
    description: 'Transaction history — aether tx history --address <addr> [--limit 20] [--json]',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      const originalArgv = process.argv;
      process.argv = [...originalArgv.slice(0, 2), 'wallet', 'history', ...originalArgv.slice(3)];
      walletCommand();
      process.argv = originalArgv;
    },
  },
  network: {
    description: 'Aether network status — slot, block height, peers, TPS, epoch info',
    handler: () => {
      const { networkCommand } = require('./commands/network');
      networkCommand();
    },
  },
  history: {
    description: 'Transaction history for an address — alias for tx history',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      const originalArgv = process.argv;
      process.argv = [...originalArgv.slice(0, 2), 'wallet', 'history', ...originalArgv.slice(3)];
      walletCommand();
      process.argv = originalArgv;
    },
  },
  validator: {
    description: 'Validator node management',
    handler: () => {
      // Handle validator subcommands
      const subcmd = process.argv[3];
      
      if (!subcmd) {
        console.log('Usage: aether-cli validator <command>');
        console.log('');
        console.log('Commands:');
        console.log('  start    Start the validator node');
        console.log('  status   Check validator status');
        console.log('');
        return;
      }
      
      switch (subcmd) {
        case 'start':
          validatorStart();
          break;
        case 'status':
          validatorStatus();
          break;
        default:
          console.error(`Unknown validator command: ${subcmd}`);
          console.error('Valid commands: start, status');
          process.exit(1);
      }
    },
  },
  delegations: {
    description: 'List/claim stake delegations — aether delegations list --address <addr>',
    handler: () => {
      delegationsCommand();
    },
  },
  rewards: {
    description: 'View staking rewards — aether rewards list --address <addr> | rewards summary | rewards claim',
    handler: () => {
      rewardsCommand();
    },
  },
  snapshot: {
    description: 'Node sync status, snapshot slot info, and network slot comparison',
    handler: () => {
      const { snapshotCommand } = require('./commands/snapshot');
      snapshotCommand();
    },
  },
  validators: {
    description: 'List active validators — aether validators list [--tier full|lite|observer] [--json]',
    handler: () => {
      validatorsListCommand();
    },
  },
  help: {
    description: 'Show this help message',
    handler: showHelp,
  },
  version: {
    description: 'Show version number',
    handler: () => console.log(`aether-cli v${VERSION}`),
  },
};

/**
 * Display help message with ASCII art
 */
function showHelp() {
  const header = `
███╗   ███╗██╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
████╗ ████║██║██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
██╔████╔██║██║███████╗███████╗██║██║   ██║██╔██╗ ██║
██║╚██╔╝██║██║╚════██║╚════██║██║██║   ██║██║╚██╗██║
██║ ╚═╝ ██║██║███████║███████║██║╚██████╔╝██║ ╚████║
╚═╝     ╚═╝╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝

Validator CLI v${VERSION}
`.trim();

  console.log(header);
  console.log('\nUsage: aether-cli <command> [options]\n');
  console.log('Commands:');
  Object.entries(COMMANDS).forEach(([cmd, info]) => {
    console.log(`  ${cmd.padEnd(18)} ${info.description}`);
  });
  console.log('\nExamples:');
  console.log('  aether-cli doctor              # Check system requirements');
  console.log('  aether-cli init                # Start onboarding wizard');
  console.log('  aether-cli monitor             # Real-time validator dashboard');
  console.log('  aether-cli validator start     # Start validator node');
  console.log('  aether-cli validator status    # Check validator status');
  console.log('  aether-cli wallet balance      # Query AETH balance');
  console.log('  aether-cli network             # Network status, peers, slot info');
  console.log('  aether-cli network --peers     # Detailed peer list');
  console.log('  aether-cli tx history          # Show transaction history');
  console.log('  aether-cli --version           # Show version');
  console.log('\nDocumentation: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop');
  console.log('Spec: docs/MINING_VALIDATOR_TOOLS.md\n');
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);

  // Handle version flag
  if (args.includes('--version') || args.includes('-v')) {
    return 'version';
  }

  // No args → interactive menu
  if (args.length === 0) {
    return 'start';
  }

  // Handle multi-word commands (e.g., "validator start", "kyc generate")
  if (args.length >= 2) {
    const multiCmd = `${args[0]} ${args[1]}`;
    if (COMMANDS[multiCmd]) {
      return multiCmd;
    }
  }

  // Single word command
  return args[0] || 'help';
}

/**
 * Main CLI entry point
 */
function main() {
  const command = parseArgs();
  
  if (COMMANDS[command]) {
    COMMANDS[command].handler();
  } else {
    console.error(`❌ Unknown command: ${command}`);
    console.error('Run "aether-cli help" for usage.\n');
    process.exit(1);
  }
}

// Run CLI
main();
