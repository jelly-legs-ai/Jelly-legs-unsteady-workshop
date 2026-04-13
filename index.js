#!/usr/bin/env node
/**
 * aether-cli - Aether Blockchain Command Line Interface
 * 
 * Main entry point for the validator CLI tool.
 * Provides onboarding, system checks, validator management, and blockchain operations.
 * 
 * @version 2.0.0
 * @author Jelly-legs AI Team
 */

const readline = require('readline');
const path = require('path');

// Import UI framework for consistent branding
const { BRANDING, C, indicators, success, error, warning, info, code, key, value, formatHelp, drawBox } = require('./lib/ui');

// CLI version
const VERSION = '2.1.3';

// Command imports
const { doctorCommand } = require('./commands/doctor');
const { validatorStartCommand } = require('./commands/validator-start');
const { validatorStatus } = require('./commands/validator-status');
const { validatorInfo } = require('./commands/validator-info');
const { validatorCommand } = require('./commands/validator');
const { init } = require('./commands/init');
const { monitorLoop } = require('./commands/monitor');
const { logsCommand } = require('./commands/logs');
const { sdkCommand } = require('./commands/sdk');
const { snapshotCommand } = require('./commands/snapshot');
const { walletCommand } = require('./commands/wallet');
const { networkCommand } = require('./commands/network');
const { networkDiagnosticsCommand } = require('./commands/network-diagnostics');
const { validatorsCommand } = require('./commands/validators');
const { delegationsCommand } = require('./commands/delegations');
const { rewardsCommand } = require('./commands/rewards');
const { validatorRegisterCommand } = require('./commands/validator-register');
const { accountCommand } = require('./commands/account');
const { emergencyCommand } = require('./commands/emergency');
const { priceCommand } = require('./commands/price');
const { epochCommand } = require('./commands/epoch');
const { supplyCommand } = require('./commands/supply');
const { statusCommand } = require('./commands/status');
const { chainCommand } = require('./commands/chain');
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
const { pingCommand } = require('./commands/ping');
const { claimCommand } = require('./commands/claim');
const { unstakeCommand } = require('./commands/unstake');
const { txCommand } = require('./commands/tx');
const { multisigCommand } = require('./commands/multisig');
const { deployCommand } = require('./commands/deploy');
const { callCommand } = require('./commands/call');
const { blockheightCommand } = require('./commands/blockheight');
const { versionCommand } = require('./commands/version');
const { tokenAccountsCommand } = require('./commands/token-accounts');

<<<<<<< HEAD
// Note: kyc.js exists but is not wired - pending compliance requirements
=======
// CLI version
const VERSION = '2.0.0';
>>>>>>> 239519c2d7d5a771b9d8163096b2aadaf79a239f

// Parse args early to support flags on commands
function getCommandArgs() {
  return process.argv.slice(2);
}

<<<<<<< HEAD
=======
// Import branding and theme
const { THEME, TIER_COLORS, getLogo, getMenuHeader, getHeader, getSuccessBanner, getErrorBanner } = require('./lib/branding');

>>>>>>> 239519c2d7d5a771b9d8163096b2aadaf79a239f
/**
 * Display the interactive main menu
 */
async function showMenu() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const prompt = (q) => new Promise((res) => rl.question(q, res));

<<<<<<< HEAD
  console.log(BRANDING.header(VERSION));
=======
  console.log(getMenuHeader());
  console.log(
    THEME.cyan + '\n  ╔═══════════════════════════════════════════════╗\n' +
    '  ║     AETHER CHAIN — Validator Setup Wizard     ║\n' +
    '  ╚═══════════════════════════════════════════════╝' + THEME.reset + '\n'
  );
>>>>>>> 239519c2d7d5a771b9d8163096b2aadaf79a239f

  console.log(`  ${C.dim}Welcome to Aether CLI. What would you like to do?${C.reset}\n`);
  
  const menuItems = [
    { num: '1', icon: indicators.bullet, label: 'Doctor', desc: 'Check system requirements', cmd: 'doctor' },
    { num: '2', icon: indicators.bullet, label: 'Start', desc: 'Begin validator onboarding', cmd: 'init' },
    { num: '3', icon: indicators.bullet, label: 'Monitor', desc: 'Watch live validator stats', cmd: 'monitor' },
    { num: '4', icon: indicators.bullet, label: 'Logs', desc: 'Tail validator logs', cmd: 'logs' },
    { num: '5', icon: indicators.bullet, label: 'SDK', desc: 'SDK tools and info', cmd: 'sdk' },
    { num: '6', icon: indicators.bullet, label: 'Network', desc: 'Network status', cmd: 'network' },
    { num: '7', icon: indicators.bullet, label: 'Help', desc: 'Show all commands', cmd: 'help' },
  ];

  for (const item of menuItems) {
    console.log(`  ${C.cyan}${item.num})${C.reset}  ${C.bright}${item.label}${C.reset}  ${C.dim}${item.desc}${C.reset}`);
  }

  console.log(`\n  ${C.dim}Type a number or command name. Press Ctrl+C to exit.${C.reset}\n`);

  const VALID_CHOICES = ['1', '2', '3', '4', '5', '6', '7', 'doctor', 'init', 'monitor', 'logs', 'sdk', 'network', 'help'];

  while (true) {
    const answer = (await prompt(`${C.cyan}>${C.reset} `)).trim().toLowerCase();

    if (answer === '' || answer === '1' || answer === 'doctor') {
      rl.close();
      doctorCommand({ autoFix: false, tier: 'full' });
      return;
    }

    if (answer === '2' || answer === 'init' || answer === 'start') {
      rl.close();
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
      logsCommand();
      return;
    }

    if (answer === '5' || answer === 'sdk') {
      rl.close();
      sdkCommand();
      return;
    }

    if (answer === '6' || answer === 'network') {
      rl.close();
      networkCommand();
      return;
    }

    if (answer === '7' || answer === 'help') {
      showHelp();
      console.log(`  ${C.dim}Press Ctrl+C to exit or select an option above.${C.reset}\n`);
      continue;
    }

    console.log(`\n  ${warning(`Unknown option: "${answer}". Type 1-7 or a command name.`)}\n`);
  }
}

// Available commands
const COMMANDS = {
  start: {
    description: 'Launch interactive menu (default)',
    handler: () => showMenu(),
  },
  doctor: {
    description: 'Run system requirements checks (CPU/RAM/Disk/Network)',
    handler: () => {
      const args = getCommandArgs();
      const autoFix = args.includes('--fix') || args.includes('-f');
      let tier = 'full';
      const tierIndex = args.findIndex(arg => arg === '--tier');
      if (tierIndex !== -1 && args[tierIndex + 1]) {
        tier = args[tierIndex + 1].toLowerCase();
      }
      doctorCommand({ autoFix, tier });
    },
  },
  init: {
    description: 'Start onboarding wizard (generate identity, wallet, connect)',
    handler: init,
  },
  monitor: {
    description: 'Real-time validator dashboard (slot, height, peers, TPS)',
    handler: () => {
      const { main } = require('./commands/monitor');
      main();
    },
  },
  logs: {
    description: 'Tail validator logs with colour-coded output',
    handler: logsCommand,
  },
  sdk: {
    description: 'Aether SDK tools - direct blockchain RPC access',
    handler: sdkCommand,
  },
  wallet: {
    description: 'Wallet management - create, import, list, balance, transfer',
    handler: walletCommand,
  },
  stake: {
    description: 'Stake AETH to a validator - stake --validator <addr> --amount <aeth>',
    handler: stakeCommand,
  },
  'stake-positions': {
    description: 'Show current stake positions/delegations',
    handler: () => {
      const { stakePositionsCommand } = require('./commands/stake-positions');
      stakePositionsCommand();
    },
  },
  'stake-info': {
    description: 'Get staking info for an address via chain RPC',
    handler: () => {
      const { stakeInfoCommand } = require('./commands/stake-info');
      stakeInfoCommand();
    },
  },
  unstake: {
    description: 'Unstake AETH - deactivate a stake account',
    handler: unstakeCommand,
  },
  claim: {
    description: 'Claim accumulated staking rewards - claim --address <addr>',
    handler: claimCommand,
  },
  transfer: {
    description: 'Transfer AETH to another address - transfer --to <addr> --amount <aeth>',
    handler: transferCommand,
  },
  'tx-history': {
    description: 'Transaction history for an address',
    handler: txHistoryCommand,
  },
  tx: {
    description: 'Look up a transaction by signature - tx <sig> [--json] [--wait]',
    handler: txCommand,
  },
  blockhash: {
    description: 'Get the latest blockhash for transaction signing',
    handler: blockhashCommand,
  },
  balance: {
    description: 'Query account balance - balance [address] [--json]',
    handler: balanceCommand,
  },
  network: {
    description: 'Aether network status - slot, height, peers, TPS',
    handler: networkCommand,
  },
  chain: {
    description: 'Blockchain chain data - slot, epoch, validators, chain ID',
    handler: chainCommand,
  },
  validator: {
    description: 'Validator node management - status, info, start, stop, register, logs',
    handler: () => {
      validatorCommand();
    },
  },
  delegations: {
    description: 'List/claim stake delegations',
    handler: delegationsCommand,
  },
  rewards: {
    description: 'View staking rewards - list | summary | pending | claim | compound',
    handler: rewardsCommand,
  },
  snapshot: {
    description: 'Node sync status, snapshot slot info',
    handler: snapshotCommand,
  },
  info: {
    description: 'Validator info snapshot - identity, sync, peers, stake',
    handler: () => {
      const { infoCommand } = require('./commands/info');
      infoCommand();
    },
  },
  account: {
    description: 'Query on-chain account data - account --address <addr> [--json]',
    handler: accountCommand,
  },
  chain: {
    description: 'Chain status - slot, epoch, validator count, chain ID',
    handler: () => chainCommand(),
  },
  epoch: {
    description: 'Aether epoch info - current epoch, slot, time remaining, APY',
    handler: epochCommand,
  },
  supply: {
    description: 'Aether token supply - total, circulating, staked, burned',
    handler: supplyCommand,
  },
  status: {
    description: 'Full dashboard - epoch, network, supply, validator info',
    handler: statusCommand,
  },
  validators: {
    description: 'Validator network management - validators list|info|top [--tier] [--json]',
    handler: validatorsCommand,
  },
  'validator-info': {
    description: 'Get detailed info for a specific validator',
    handler: () => {
      const { validatorInfoCommand } = require('./commands/validator-info');
      validatorInfoCommand();
    },
  },
  stats: {
    description: 'Wallet stats dashboard - balance, stake, recent txs',
    handler: statsCommand,
  },
  price: {
    description: 'AETH/USD price - price [--pair AETH/USD] [--json]',
    handler: priceCommand,
  },
  broadcast: {
    description: 'Broadcast a signed transaction - broadcast --tx <sig> [--json]',
    handler: broadcastCommand,
  },
  apy: {
    description: 'Validator APY estimator',
    handler: apyCommand,
  },
  ping: {
    description: 'Ping RPC endpoint - measure latency, check node health',
    handler: pingCommand,
  },
  'network-diagnostics': {
    description: 'Network diagnostics with RPC failover',
    handler: networkDiagnosticsCommand,
  },
  emergency: {
    description: 'Emergency response & network alerts',
    handler: emergencyCommand,
  },
  fees: {
    description: 'Network fee estimates - fees [--json] [--verbose] [--rpc <url>]',
    handler: feesCommand,
  },
  tps: {
    description: 'Transactions per second monitor - tps [--monitor] [--interval 2] [--json]',
    handler: tpsCommand,
  },
  slot: {
    description: 'Get current slot number - slot [--json] [--rpc <url>]',
    handler: slotCommand,
  },
  blockheight: {
    description: 'Get current block height - blockheight [--json] [--rpc <url>] [--compare]',
    handler: blockheightCommand,
  },
  'token-accounts': {
    description: 'Get SPL token accounts for wallet - token-accounts [address] [--json]',
    handler: tokenAccountsCommand,
  },
  version: {
    description: 'Get node version info - version [--json] [--cli]',
    handler: versionCommand,
  },
  multisig: {
    description: 'Multi-signature wallet management',
    handler: multisigCommand,
  },
  register: {
    description: 'Register validator with network',
    handler: validatorRegisterCommand,
  },
  'sdk-test': {
    description: 'Test SDK with real RPC calls - sdk-test [--rpc <url>] [--quick] [--json]',
    handler: sdkTestCommand,
  },
  config: {
    description: 'Configuration management - config set/get/list/validate/init',
    handler: configCommand,
  },
  nft: {
    description: 'NFT management - create|list|transfer|info|update',
    handler: nftCommand,
  },
  deploy: {
    description: 'Deploy smart contracts - deploy <file> [--name <name>] [--upgradeable]',
    handler: deployCommand,
  },
  call: {
    description: 'Call smart contract functions - call <program-id> <function> [args...] [--query|--wallet]',
    handler: callCommand,
  },
  'validator-setup': {
    description: 'Setup validator prerequisites (alias for validator start)',
    handler: () => {
      const { validatorStartCommand } = require('./commands/validator-start');
      validatorStartCommand();
    },
  },
  install: {
    description: 'Install or upgrade aether-cli',
    handler: installCommand,
  },
  help: {
    description: 'Show this help message',
    handler: showHelp,
  },
  'cli-version': {
    description: 'Show CLI version number',
    handler: () => {
      console.log(BRANDING.header(VERSION));
      console.log(`  ${C.dim}SDK-powered blockchain CLI for Aether validators${C.reset}\n`);
    },
  },
};

/**
 * Display help message with consistent branding
 */
function showHelp() {
<<<<<<< HEAD
  console.log(BRANDING.header(VERSION));
  
  console.log(`\n  ${C.bright}AETHER CLI${C.reset} — ${C.dim}Decentralized Infrastructure for the Future${C.reset}\n`);
  
  // Group commands by category
  const categories = {
    'Wallet & Accounts': ['wallet', 'balance', 'transfer', 'tx', 'tx-history', 'account', 'stats', 'token-accounts'],
    'Staking': ['stake', 'unstake', 'stake-positions', 'stake-info', 'delegations', 'rewards', 'claim'],
    'Validator': ['init', 'validator', 'validator-info', 'register', 'validators', 'monitor', 'logs'],
    'Network': ['network', 'chain', 'network-diagnostics', 'ping', 'epoch', 'slot', 'blockheight', 'tps', 'fees', 'supply', 'version'],
    'SDK & Tools': ['sdk', 'sdk-test', 'snapshot', 'info', 'status', 'blockhash', 'broadcast', 'price', 'apy', 'deploy'],
    'Advanced': ['nft', 'multisig', 'emergency', 'config', 'doctor', 'install', 'call'],
  };

  for (const [category, cmds] of Object.entries(categories)) {
    console.log(`  ${C.cyan}◆ ${category}${C.reset}`);
    for (const cmd of cmds) {
      const cmdInfo = COMMANDS[cmd];
      if (cmdInfo) {
        console.log(`    ${code(cmd.padEnd(18))} ${C.dim}${cmdInfo.description}${C.reset}`);
      }
    }
    console.log();
  }
=======
  const header = getHeader(VERSION);
>>>>>>> 239519c2d7d5a771b9d8163096b2aadaf79a239f

  console.log(`  ${C.cyan}◆ Quick Start${C.reset}\n`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether doctor')}              ${C.dim}# Check system requirements${C.reset}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether init')}                ${C.dim}# Start validator onboarding${C.reset}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether wallet create')}       ${C.dim}# Create a new wallet${C.reset}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether network')}             ${C.dim}# Check network status${C.reset}`);
  console.log(`    ${C.dim}$${C.reset} ${code('aether sdk getSlot')}         ${C.dim}# Query current slot via SDK${C.reset}`);
  console.log();
  
  console.log(`  ${C.dim}Documentation: ${C.cyan}https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop${C.reset}\n`);
}

/**
 * Parse command line arguments
 */
function parseArgs() {
  const args = process.argv.slice(2);

  // Handle version flags
  if (args.includes('--version') || args.includes('-v') || args.includes('-V')) {
    return 'cli-version';
  }

  // No args → interactive menu
  if (args.length === 0) {
    return 'start';
  }

  // Handle multi-word commands (e.g., "validator start")
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
    try {
      COMMANDS[command].handler();
    } catch (err) {
      console.error(`\n  ${error(`Command failed: ${err.message}`)}\n`);
      process.exit(1);
    }
  } else {
    console.error(`\n  ${error(`Unknown command: ${command}`)}`);
    console.log(`  ${C.dim}Run "aether help" to see available commands.${C.reset}\n`);
    process.exit(1);
  }
}

// Run CLI only if executed directly
if (require.main === module) {
  main();
}

// Export for module use
module.exports = { main, showHelp, COMMANDS, VERSION };
