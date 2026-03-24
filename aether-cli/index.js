#!/usr/bin/env node
/**
 * aether-cli - AeTHer Validator Command Line Interface
 * 
 * Main entry point for the validator CLI tool.
 * Provides onboarding, system checks, validator management, and KYC integration.
 * 
 * @see docs/MINING_VALIDATOR_TOOLS.md for full spec
 */

const { doctorCommand } = require('./commands/doctor');

// CLI version
const VERSION = '1.0.0';

// Available commands
const COMMANDS = {
  doctor: {
    description: 'Run system requirements checks (CPU/RAM/Disk/Network/Firewall)',
    handler: doctorCommand,
  },
  init: {
    description: 'Start onboarding wizard (coming soon)',
    handler: () => console.log('🚧 init command under development'),
  },
  'kyc generate': {
    description: 'Generate pre-filled KYC link with pubkey, node ID, signature',
    handler: () => console.log('🚧 kyc generate command under development'),
  },
  validator: {
    description: 'Validator node management (start/stop/status)',
    handler: () => console.log('🚧 validator commands under development'),
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
    console.log(`  ${cmd.padEnd(15)} ${info.description}`);
  });
  console.log('\nExamples:');
  console.log('  aether-cli doctor              # Check system requirements');
  console.log('  aether-cli init                # Start onboarding wizard');
  console.log('  aether-cli kyc generate        # Generate KYC link');
  console.log('  aether-cli validator start     # Start validator node');
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

  // Handle multi-word commands (e.g., "kyc generate")
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
