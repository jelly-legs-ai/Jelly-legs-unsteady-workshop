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

// Parse args early to support flags on commands
function getCommandArgs() {
  return process.argv.slice(2);
}

// CLI version
const VERSION = '1.0.0';

// Available commands
const COMMANDS = {
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
  console.log('  aether-cli --version           # Show version');
  console.log('\nDocumentation: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop');
  console.log('Spec: docs/MINING_VALIDATOR_TOOLS.md\n');
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);
  
  // Handle flags
  if (args.includes('--version') || args.includes('-v')) {
    return 'version';
  }
  if (args.includes('--help') || args.includes('-h') || args.length === 0) {
    return 'help';
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
