#!/usr/bin/env node
/**
 * aether-cli sdk-test
 *
 * Comprehensive SDK test suite - exercises all major SDK functions
 * with REAL HTTP RPC calls to verify the SDK works end-to-end.
 *
 * Uses @jellylegsai/aether-sdk for all blockchain interactions.
 * No stubs, no mocks - every function makes actual RPC calls.
 *
 * Usage:
 *   aether sdk-test                  Run full test suite
 *   aether sdk-test --rpc <url>      Test against specific RPC endpoint
 *   aether sdk-test --quick          Run only essential tests (slot, balance, health)
 *   aether sdk-test --json           JSON output for CI/monitoring
 *
 * Default RPC: http://127.0.0.1:8899 (or AETHER_RPC env var)
 */

const path = require('path');
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

// ANSI colors
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

const CLI_VERSION = '1.0.0';

// Test results storage
const testResults = {
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: [],
};

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

function recordTest(name, passed, error = null, data = null) {
  testResults.tests.push({ name, passed, error, data });
  if (passed) {
    testResults.passed++;
  } else if (error === 'SKIPPED') {
    testResults.skipped++;
  } else {
    testResults.failed++;
  }
}

function printTestResult(name, passed, error, data, asJson) {
  if (asJson) return;
  
  const icon = passed ? `${C.green}✓${C.reset}` : error === 'SKIPPED' ? `${C.yellow}○${C.reset}` : `${C.red}✗${C.reset}`;
  const status = passed ? `${C.green}PASS${C.reset}` : error === 'SKIPPED' ? `${C.yellow}SKIP${C.reset}` : `${C.red}FAIL${C.reset}`;
  
  console.log(`  ${icon} ${name.padEnd(35)} ${status}`);
  
  if (data && !passed && error !== 'SKIPPED') {
    console.log(`      ${C.red}Error: ${error}${C.reset}`);
  } else if (data && passed) {
    console.log(`      ${C.dim}${formatData(data)}${C.reset}`);
  } else if (error && error !== 'SKIPPED') {
    console.log(`      ${C.red}${error}${C.reset}`);
  }
}

function formatData(data) {
  if (typeof data === 'number') return data.toLocaleString();
  if (typeof data === 'string') return data.length > 60 ? data.substring(0, 60) + '...' : data;
  if (typeof data === 'object') {
    try {
      return JSON.stringify(data).substring(0, 80);
    } catch {
      return '[object]';
    }
  }
  return String(data);
}

function getDefaultRpc() {
  return process.env.AETHER_RPC || aether.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
}

// ---------------------------------------------------------------------------
// Test Cases - All use REAL RPC calls via SDK
// ---------------------------------------------------------------------------

async function testGetSlot(client) {
  const name = 'getSlot()';
  try {
    const slot = await client.getSlot();
    const passed = typeof slot === 'number' && slot >= 0;
    recordTest(name, passed, passed ? null : 'Invalid slot number', slot);
    printTestResult(name, passed, passed ? null : 'Invalid slot number', slot, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetBlockHeight(client) {
  const name = 'getBlockHeight()';
  try {
    const height = await client.getBlockHeight();
    const passed = typeof height === 'number' && height >= 0;
    recordTest(name, passed, passed ? null : 'Invalid block height', height);
    printTestResult(name, passed, passed ? null : 'Invalid block height', height, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetHealth(client) {
  const name = 'getHealth()';
  try {
    const health = await client.getHealth();
    const passed = health === 'ok' || health === 'healthy' || typeof health === 'string';
    recordTest(name, passed, passed ? null : 'Unhealthy node', health);
    printTestResult(name, passed, passed ? null : 'Unhealthy node', health, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetVersion(client) {
  const name = 'getVersion()';
  try {
    const version = await client.getVersion();
    const passed = version && (version.aetherCore || version.featureSet || Object.keys(version).length > 0);
    recordTest(name, passed, passed ? null : 'Empty version info', version);
    printTestResult(name, passed, passed ? null : 'Empty version info', version, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetEpochInfo(client) {
  const name = 'getEpochInfo()';
  try {
    const epoch = await client.getEpochInfo();
    const passed = epoch && (epoch.epoch !== undefined || epoch.slot !== undefined);
    recordTest(name, passed, passed ? null : 'Invalid epoch info', epoch);
    printTestResult(name, passed, passed ? null : 'Invalid epoch info', epoch, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetSupply(client) {
  const name = 'getSupply()';
  try {
    const supply = await client.getSupply();
    const passed = supply && (supply.total !== undefined || supply.circulating !== undefined);
    recordTest(name, passed, passed ? null : 'Invalid supply info', supply);
    printTestResult(name, passed, passed ? null : 'Invalid supply info', supply, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetTPS(client) {
  const name = 'getTPS()';
  try {
    const tps = await client.getTPS();
    const passed = tps !== null && tps !== undefined;
    recordTest(name, passed, passed ? null : 'Invalid TPS', tps);
    printTestResult(name, passed, passed ? null : 'Invalid TPS', tps, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetFees(client) {
  const name = 'getFees()';
  try {
    const fees = await client.getFees();
    const passed = fees && Object.keys(fees).length > 0;
    recordTest(name, passed, passed ? null : 'Empty fee info', fees);
    printTestResult(name, passed, passed ? null : 'Empty fee info', fees, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetValidators(client) {
  const name = 'getValidators()';
  try {
    const validators = await client.getValidators();
    const passed = Array.isArray(validators);
    recordTest(name, passed, passed ? null : 'Not an array', validators ? validators.length : null);
    printTestResult(name, passed, passed ? null : 'Not an array', validators ? validators.length : null, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetClusterPeers(client) {
  const name = 'getClusterPeers()';
  try {
    const peers = await client.getClusterPeers();
    const passed = Array.isArray(peers);
    recordTest(name, passed, passed ? null : 'Not an array', peers ? peers.length : null);
    printTestResult(name, passed, passed ? null : 'Not an array', peers ? peers.length : null, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetRecentBlockhash(client) {
  const name = 'getRecentBlockhash()';
  try {
    const blockhash = await client.getRecentBlockhash();
    const passed = blockhash && (blockhash.blockhash || blockhash.value);
    recordTest(name, passed, passed ? null : 'No blockhash returned', blockhash);
    printTestResult(name, passed, passed ? null : 'No blockhash returned', blockhash, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetAccountInfo(client, testAddress) {
  const name = 'getAccountInfo()';
  try {
    // Use system program address as test (always exists)
    const address = testAddress || '11111111111111111111111111111111';
    const account = await client.getAccountInfo(address);
    const passed = account && (account.lamports !== undefined || account.owner !== undefined);
    recordTest(name, passed, passed ? null : 'Invalid account info', account);
    printTestResult(name, passed, passed ? null : 'Invalid account info', account, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetBalance(client, testAddress) {
  const name = 'getBalance()';
  try {
    const address = testAddress || '11111111111111111111111111111111';
    const balance = await client.getBalance(address);
    const passed = typeof balance === 'number' && balance >= 0;
    recordTest(name, passed, passed ? null : 'Invalid balance', balance);
    printTestResult(name, passed, passed ? null : 'Invalid balance', balance, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testGetSlotProduction(client) {
  const name = 'getSlotProduction()';
  try {
    const stats = await client.getSlotProduction();
    const passed = stats && Object.keys(stats).length > 0;
    recordTest(name, passed, passed ? null : 'Empty slot production stats', stats);
    printTestResult(name, passed, passed ? null : 'Empty slot production stats', stats, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testConvenienceFunctions(rpcUrl) {
  const name = 'Convenience Functions';
  try {
    // Test multiple convenience functions
    const [slot, health, blockHeight] = await Promise.all([
      aether.getSlot(),
      aether.getHealth(),
      aether.getBlockHeight(),
    ]);
    
    const passed = typeof slot === 'number' && health && typeof blockHeight === 'number';
    recordTest(name, passed, passed ? null : 'One or more functions failed', { slot, health, blockHeight });
    printTestResult(name, passed, passed ? null : 'Function failed', { slot, health, blockHeight }, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

async function testPingUtility(rpcUrl) {
  const name = 'ping()';
  try {
    const result = await aether.ping(rpcUrl);
    const passed = result && result.ok === true && result.latency >= 0;
    recordTest(name, passed, passed ? null : 'Ping failed', result);
    printTestResult(name, passed, passed ? null : 'Ping failed', result, false);
    return passed;
  } catch (err) {
    recordTest(name, false, err.message);
    printTestResult(name, false, err.message, null, false);
    return false;
  }
}

// ---------------------------------------------------------------------------
// Test Runner
// ---------------------------------------------------------------------------

async function runFullTestSuite(client, rpcUrl) {
  console.log(`\n${C.bright}${C.cyan}═══ Aether SDK Test Suite ═══${C.reset}\n`);
  console.log(`  ${C.dim}RPC Endpoint: ${rpcUrl}${C.reset}`);
  console.log(`  ${C.dim}Testing @jellylegsai/aether-sdk v1.0.0${C.reset}\n`);
  console.log(`  ${C.bright}Running tests...${C.reset}\n`);

  // Core RPC methods
  console.log(`  ${C.cyan}── Core RPC Methods ──${C.reset}`);
  await testGetSlot(client);
  await testGetBlockHeight(client);
  await testGetHealth(client);
  await testGetVersion(client);
  await testGetEpochInfo(client);
  
  console.log(`\n  ${C.cyan}── Network & Supply ──${C.reset}`);
  await testGetSupply(client);
  await testGetTPS(client);
  await testGetFees(client);
  await testGetValidators(client);
  await testGetClusterPeers(client);
  
  console.log(`\n  ${C.cyan}── Transaction Support ──${C.reset}`);
  await testGetRecentBlockhash(client);
  await testGetSlotProduction(client);
  
  console.log(`\n  ${C.cyan}── Account Operations ──${C.reset}`);
  await testGetAccountInfo(client);
  await testGetBalance(client);
  
  console.log(`\n  ${C.cyan}── Convenience Functions ──${C.reset}`);
  await testConvenienceFunctions(rpcUrl);
  await testPingUtility(rpcUrl);
}

async function runQuickTest(client, rpcUrl) {
  console.log(`\n${C.bright}${C.cyan}═══ Aether SDK Quick Test ═══${C.reset}\n`);
  console.log(`  ${C.dim}RPC Endpoint: ${rpcUrl}${C.reset}\n`);
  
  console.log(`  ${C.cyan}── Essential Tests ──${C.reset}`);
  await testGetSlot(client);
  await testGetHealth(client);
  await testGetBalance(client);
}

function printSummary(asJson) {
  if (asJson) {
    console.log(JSON.stringify({
      total: testResults.passed + testResults.failed + testResults.skipped,
      passed: testResults.passed,
      failed: testResults.failed,
      skipped: testResults.skipped,
      tests: testResults.tests,
      cli_version: CLI_VERSION,
      timestamp: new Date().toISOString(),
    }, null, 2));
    return;
  }
  
  console.log(`\n${C.bright}${C.cyan}═══ Test Summary ═══${C.reset}\n`);
  
  const total = testResults.passed + testResults.failed + testResults.skipped;
  const passRate = total > 0 ? ((testResults.passed / total) * 100).toFixed(1) : 0;
  
  console.log(`  ${C.bright}Total:${C.reset}  ${total}`);
  console.log(`  ${C.green}Passed:${C.reset} ${testResults.passed}`);
  console.log(`  ${C.red}Failed:${C.reset} ${testResults.failed}`);
  console.log(`  ${C.yellow}Skipped:${C.reset} ${testResults.skipped}`);
  console.log(`  ${C.bright}Pass Rate:${C.reset} ${passRate}%\n`);
  
  if (testResults.failed > 0) {
    console.log(`  ${C.red}── Failed Tests ──${C.reset}`);
    testResults.tests.filter(t => !t.passed && t.error !== 'SKIPPED').forEach(t => {
      console.log(`    ${C.red}✗ ${t.name}: ${t.error}${C.reset}`);
    });
    console.log();
  }
  
  if (testResults.failed === 0) {
    console.log(`  ${C.green}✓ All tests passed!${C.reset}\n`);
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function sdkTestCommand() {
  const args = process.argv.slice(2);
  const asJson = args.includes('--json') || args.includes('-j');
  const isQuick = args.includes('--quick') || args.includes('-q');
  
  const rpcIdx = args.findIndex(a => a === '--rpc' || a === '-r');
  const rpcUrl = rpcIdx !== -1 && args[rpcIdx + 1] ? args[rpcIdx + 1] : getDefaultRpc();
  
  if (!asJson) {
    console.log(`\n${C.bright}${C.cyan}╔═══════════════════════════════════════════════════════════╗${C.reset}`);
    console.log(`${C.bright}${C.cyan}║         AETHER SDK COMPREHENSIVE TEST SUITE              ║${C.reset}`);
    console.log(`${C.bright}${C.cyan}╚═══════════════════════════════════════════════════════════╝${C.reset}\n`);
  }
  
  const client = new aether.AetherClient({ rpcUrl });
  
  if (isQuick) {
    await runQuickTest(client, rpcUrl);
  } else {
    await runFullTestSuite(client, rpcUrl);
  }
  
  printSummary(asJson);
  
  // Exit with error if tests failed
  if (testResults.failed > 0) {
    process.exit(1);
  }
}

// Export for module use
module.exports = { sdkTestCommand };

// Run if called directly
if (require.main === module) {
  sdkTestCommand().catch(err => {
    console.error(`${C.red}✗ Test suite failed: ${err.message}${C.reset}`);
    process.exit(1);
  });
}
