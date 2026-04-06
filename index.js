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
const { validatorInfo } = require('./commands/validator-info');
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
const { validatorRegisterCommand } = require('./commands/validator-register');
const { accountCommand } = require('./commands/account');
const { emergencyCommand } = require('./commands/emergency');
const { priceCommand } = require('./commands/price');
const { epochCommand } = require('./commands/epoch');
const { supplyCommand } = require('./commands/supply');
const { statusCommand } = require('./commands/status');
const { broadcastCommand } = require('./commands/broadcast');
const { apyCommand } = require('./commands/apy');
const { statsCommand } = require('./commands/stats');
const { txHistoryCommand } = require('./commands/tx-history');
const { feesCommand } = require('./commands/fees');
const { tpsCommand } = require('./commands/tps');
const { blockhashCommand } = require('./commands/blockhash');
const { sdkTestCommand } = require('./commands/sdk-test');
const { balanceCommand } = require('./commands/balance');
const { transferCommand } = require('./commands/transfer');
const { slotCommand } = require('./commands/slot');
const { configCommand } = require('./commands/config');
const { stakeCommand } = require('./commands/stake');
const { nftCommand } = require('./commands/nft');
const { installCommand } = require('./commands/install');
const readline = require('readline');

// CLI version
const VERSION = '1.8.0';

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
    handler: () => {
      const { kycGenerate } = require('./commands/kyc');
      kycGenerate();
    },
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
    description: 'Stake AETH to a validator — aether stake --validator <addr> --amount <aeth> [--list-validators]',
    handler: () => {
      const { stakeCommand } = require('./commands/stake');
      stakeCommand();
    },
  },
  'stake-positions': {
    description: 'Show current stake positions/delegations — aether stake-positions --address <addr> [--json]',
    handler: () => {
      const { stakePositionsCommand } = require('./commands/stake-positions');
      stakePositionsCommand();
    },
  },
  'stake-info': {
    description: 'Get staking info for an address via real chain RPC — aether stake-info <address>',
    handler: () => {
      const { stakeInfoCommand } = require('./commands/stake-info');
      stakeInfoCommand();
    },
  },
  unstake: {
    description: 'Unstake AETH — deactivate a stake account — aether unstake --account <stakeAcct> [--amount <aeth>]',
    handler: () => {
      const { unstakeCommand } = require('./commands/unstake');
      unstakeCommand();
    },
  },
  export: {
    description: 'Export wallet data — aether export --address <addr> [--mnemonic] [--json]',
    handler: () => {
      const { walletCommand } = require('./commands/wallet');
      const originalArgv = process.argv;
      process.argv = [...originalArgv.slice(0, 2), 'wallet', 'export', ...originalArgv.slice(3)];
      walletCommand();
      process.argv = originalArgv;
    },
  },
  transfer: {
    description: 'Transfer AETH to another address — aether transfer --to <addr> --amount <aeth>',
    handler: () => {
      transferCommand();
    },
  },
  tx: {
    description: 'Transaction history — aether tx history --address <addr> [--limit 20] [--json]',
    handler: () => {
      txHistoryCommand();
    },
  },
  'tx-history': {
    description: 'Transaction history for an address — aether tx-history --address <addr> [--limit 20] [--json]',
    handler: () => {
      txHistoryCommand();
    },
  },
  blockhash: {
    description: 'Get the latest blockhash from the chain (required for signing TXs) — aether blockhash [--json] [--watch]',
    handler: () => {
      const { blockhashCommand } = require('./commands/blockhash');
      blockhashCommand();
    },
  },
  balance: {
    description: 'Query account balance — aether balance [address] [--json] [--lamports] [--rpc <url>]',
    handler: () => {
      const { balanceCommand } = require('./commands/balance');
      balanceCommand();
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
      txHistoryCommand();
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
        console.log('  start      Start the validator node');
        console.log('  status     Check validator status');
        console.log('  info       Get validator info');
        console.log('  register   Register validator with the network');
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
        case 'info':
          validatorInfo();
          break;
        case 'register':
          validatorRegisterCommand();
          break;
        default:
          console.error(`Unknown validator command: ${subcmd}`);
          console.error('Valid commands: start, status, info, register');
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
    description: 'View staking rewards — aether rewards list | summary | pending | claim | compound',
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
  info: {
    description: 'Validator info snapshot — identity, sync state, peers, stake positions',
    handler: () => {
      const { infoCommand } = require('./commands/info');
      infoCommand();
    },
  },
  account: {
    description: 'Query on-chain account data — aether account --address <addr> [--json] [--data] [--rpc <url>]',
    handler: () => {
      const { accountCommand } = require('./commands/account');
      accountCommand();
    },
  },
  epoch: {
    description: 'Aether epoch info — current epoch, slot, time remaining, APY estimate — aether epoch [--json] [--schedule]',
    handler: () => {
      const { epochCommand } = require('./commands/epoch');
      epochCommand();
    },
  },
  supply: {
    description: 'Aether token supply — total, circulating, staked, burned — aether supply [--json] [--verbose]',
    handler: () => {
      const { supplyCommand } = require('./commands/supply');
      // Pass full argv so supply.js can parse its own --help etc.
      supplyCommand();
    },
  },
  status: {
    description: 'Full dashboard — epoch, network, supply, validator info — aether status [--json] [--compact] [--validator]',
    handler: () => {
      const { statusCommand } = require('./commands/status');
      statusCommand();
    },
  },
  validators: {
    description: 'List active validators — aether validators list [--tier full|lite|observer] [--json]',
    handler: () => {
      validatorsListCommand();
    },
  },
  'validator-info': {
    description: 'Get detailed info for a specific validator — aether validator-info <address> [--json]',
    handler: () => {
      const { validatorInfoCommand } = require('./commands/validator-info');
      validatorInfoCommand();
    },
  },
  stats: {
    description: 'Wallet stats dashboard — balance, stake positions, recent txs — aether stats --address <addr> [--compact] [--json]',
    handler: () => {
      statsCommand();
    },
  },
  price: {
    description: 'AETH/USD price — aether price [--pair AETH/USD] [--json] [--source coingecko]',
    handler: () => {
      const { priceCommand } = require('./commands/price');
      priceCommand();
    },
  },
  broadcast: {
    description: 'Broadcast a signed transaction — aether broadcast --tx <sig> [--json] [--file <path>]',
    handler: () => {
      const { broadcastCommand } = require('./commands/broadcast');
      broadcastCommand();
    },
  },
  apy: {
    description: 'Validator APY estimator — aether apy [--validator <addr>] [--address <addr>] [--json] [--epochs <n>]',
    handler: () => {
      const { apyCommand } = require('./commands/apy');
      apyCommand();
    },
  },
  ping: {
    description: 'Ping RPC endpoint — measure latency, check node health — aether ping [--rpc <url>] [--count <n>] [--json]',
    handler: () => {
      const { pingCommand } = require('./commands/ping');
      pingCommand();
    },
  },
  emergency: {
    description: 'Emergency response & network alerts — status, monitor, check, alert, failover, history',
    handler: () => {
      const { emergencyCommand } = require('./commands/emergency');
      emergencyCommand();
    },
  },
  fees: {
    description: 'Network fee estimates — aether fees [--json] [--verbose] [--rpc <url>]',
    handler: () => {
      feesCommand();
    },
  },
  tps: {
    description: 'Transactions per second monitor — aether tps [--monitor] [--interval 2] [--json]',
    handler: () => {
      tpsCommand();
    },
  },
  slot: {
    description: 'Get current slot number — aether slot [--json] [--rpc <url>]',
    handler: () => {
      slotCommand();
    },
  },
  claim: {
    description: 'Claim accumulated staking rewards — aether claim --address <addr> [--json] [--dry-run]',
    handler: () => {
      const { claimCommand } = require('./commands/claim');
      claimCommand();
    },
  },
  register: {
    description: 'Register validator with network — aether register --wallet <addr> --amount <aeth> [--tier full]',
    handler: () => {
      validatorRegisterCommand();
    },
  },
  'sdk-test': {
    description: 'Test SDK with real RPC calls — aether sdk-test [--rpc <url>] [--quick] [--json]',
    handler: () => {
      sdkTestCommand();
    },
  },
  config: {
    description: 'Configuration management — aether config set/get/list/validate/init',
    handler: () => {
      configCommand();
    },
  },
  nft: {
    description: 'NFT management — aether nft create|list|transfer|info|update — full SDK-wired suite',
    handler: () => {
      nftCommand();
    },
  },
  install: {
    description: 'Install or upgrade aether-cli — npm install, config init, PATH setup — aether install [--force] [--rpc <url>] [--skip-rpc-check]',
    handler: () => {
      installCommand();
    },
  },
  'install-help': {
    description: 'Show install command help',
    hidden: true,
    handler: () => {
      process.argv = [process.argv[0], process.argv[1], 'install', '--help'];
      installCommand();
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
    if (info.hidden) return;
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
  console.log('  aether-cli price               # AETH/USD price check');
  console.log('  aether-cli nft create          # Create NFT with metadata');
  console.log('  aether-cli nft list            # List NFTs owned by wallet');
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
  if (args.includes('--version') || args.includes('-v') || args.includes('-V')) {
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

  // Handle single word commands
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

// Run CLI only if executed directly
if (require.main === module) {
  main();
}
