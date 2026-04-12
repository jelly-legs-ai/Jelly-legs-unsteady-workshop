#!/usr/bin/env node
/**
 * aether-cli version
 *
 * Get Aether node version information from the RPC endpoint.
 * Uses @jellylegsai/aether-sdk for REAL HTTP RPC calls.
 *
 * Usage:
 *   aether version                     Show node version info
 *   aether version --json              JSON output for scripting
 *   aether version --rpc <url>         Custom RPC endpoint
 *   aether version --cli               Show CLI version instead
 *
 * RPC Endpoint: GET /v1/version
 * SDK Function: sdk.getVersion()
 */

const path = require('path');

// Import UI framework for consistent branding
const { BRANDING, C, indicators, startSpinner, stopSpinner,
        success, error, code, highlight, drawBox } = require('../lib/ui');

const CLI_VERSION = '2.0.0';

// Import SDK — makes REAL HTTP RPC calls to the blockchain
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

function createClient(rpc) {
  return new aether.AetherClient({ rpcUrl: rpc });
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    rpc: getDefaultRpc(),
    asJson: false,
    cliVersion: false,
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--json' || args[i] === '-j') {
      options.asJson = true;
    } else if (args[i] === '--rpc' || args[i] === '-r') {
      options.rpc = args[++i];
    } else if (args[i] === '--cli' || args[i] === '-c') {
      options.cliVersion = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      showHelp();
      process.exit(0);
    }
  }

  return options;
}

function showHelp() {
  console.log(BRANDING.logoCompact);
  console.log(`
${C.bright}${C.cyan}aether-cli version${C.reset} — Get Aether node version info

${C.bright}USAGE${C.reset}
    aether version [options]

${C.bright}OPTIONS${C.reset}
    -r, --rpc <url>     RPC endpoint (default: ${getDefaultRpc()})
    -j, --json          Output as JSON
    --cli               Show CLI version only
    -h, --help          Show this help

${C.bright}DESCRIPTION${C.reset}
    Queries the Aether node for its version information including
    the core version and supported feature set.

${C.bright}SDK METHOD${C.reset}
    client.getVersion() → GET /v1/version

${C.bright}EXAMPLES${C.reset}
    aether version              # Node version info
    aether version --json       # JSON output
    aether version --cli        # CLI version only
`);
}

// ---------------------------------------------------------------------------
// Version query - REAL RPC call via SDK
// ---------------------------------------------------------------------------

async function fetchVersion(rpc) {
  const client = createClient(rpc);
  const version = await client.getVersion();

  return {
    version,
    rpc,
    timestamp: new Date().toISOString(),
  };
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

function printVersion(data) {
  const { version, rpc } = data;

  // Build version info box
  const versionContent = [
    `${C.bright}Node Version Info${C.reset}`,
    ``,
    `${C.cyan}Core Version:${C.reset}  ${highlight(version.aetherCore || 'N/A')}`,
    `${C.cyan}Feature Set:${C.reset}   ${version.featureSet || 'N/A'}`,
    `${C.dim}RPC Endpoint:${C.reset} ${rpc}`,
  ].join('\n');

  console.log();
  console.log(drawBox(versionContent, {
    style: 'single',
    title: 'AETHER NODE',
    titleColor: C.cyan,
    borderColor: C.dim,
  }));
  console.log();

  console.log(`  ${C.dim}SDK Function:  client.getVersion()${C.reset}`);
  console.log(`  ${C.dim}RPC Endpoint: GET /v1/version${C.reset}\n`);
}

function printCliVersion() {
  console.log(`${CLI_VERSION}`);
}

function printCliVersionFull() {
  console.log();
  console.log(BRANDING.header(CLI_VERSION));
  console.log(`  ${C.dim}SDK: @jellylegsai/aether-sdk${C.reset}`);
  console.log(`  ${C.dim}Node: ${process.version}${C.reset}`);
  console.log(`  ${C.dim}Platform: ${process.platform}${C.reset}\n`);
}

function printJson(data) {
  console.log(JSON.stringify({
    node_version: data.version,
    cli_version: CLI_VERSION,
    rpc: data.rpc,
    timestamp: data.timestamp,
    sdk: '@jellylegsai/aether-sdk',
    rpc_endpoint: 'GET /v1/version',
  }, null, 2));
}

// ---------------------------------------------------------------------------
// Main command
// ---------------------------------------------------------------------------

async function versionCommand() {
  const options = parseArgs();
  const { rpc, asJson, cliVersion } = options;

  // Handle CLI version only
  if (cliVersion) {
    printCliVersion();
    return;
  }

  // Handle CLI version full output
  if (asJson && process.argv.includes('--cli')) {
    console.log(JSON.stringify({
      cli_version: CLI_VERSION,
      node_version: process.version,
      platform: process.platform,
    }, null, 2));
    return;
  }

  if (!asJson) {
    startSpinner('Fetching node version');
  }

  try {
    // Real blockchain RPC call via SDK
    const data = await fetchVersion(rpc);

    if (!asJson) {
      stopSpinner(true, 'Version retrieved');
    }

    if (asJson) {
      printJson(data);
    } else {
      printVersion(data);
    }
  } catch (err) {
    if (!asJson) {
      stopSpinner(false, 'Failed');
    }

    if (asJson) {
      console.log(JSON.stringify({
        error: err.message,
        cli_version: CLI_VERSION,
        rpc,
        timestamp: new Date().toISOString(),
      }, null, 2));
    } else {
      console.log(`\n${indicators.error} ${error('Failed to fetch version:')} ${err.message}`);
      console.log(`  ${C.dim}RPC: ${rpc}${C.reset}`);
      console.log(`  ${C.dim}Make sure the Aether node is running at ${rpc}${C.reset}`);
      console.log();
      console.log(`  ${C.dim}CLI Version: ${CLI_VERSION}${C.reset}\n`);
    }
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

module.exports = { versionCommand, CLI_VERSION };

if (require.main === module) {
  versionCommand().catch(err => {
    console.error(`${indicators.error} ${error('Version command failed:')} ${err.message}`);
    process.exit(1);
  });
}
