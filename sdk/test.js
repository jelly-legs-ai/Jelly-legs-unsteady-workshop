#!/usr/bin/env node
/**
 * @jellylegsai/aether-sdk - Test Script
 * 
 * Tests all SDK functions with REAL RPC calls to http://127.0.0.1:8899
 */

const aether = require('./index');

const C = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  dim: '\x1b[2m',
};

async function test(name, fn) {
  try {
    const result = await fn();
    console.log(`  ${C.green}✓${C.reset} ${name}`);
    return { ok: true, result };
  } catch (err) {
    console.log(`  ${C.red}✗${C.reset} ${name}: ${err.message}`);
    return { ok: false, error: err.message };
  }
}

async function main() {
  console.log(`\n${C.cyan}══ @jellylegsai/aether-sdk Test Suite ══${C.reset}\n`);
  console.log(`  ${C.dim}RPC Endpoint: ${aether.DEFAULT_RPC_URL}${C.reset}\n`);

  const results = [];

  // Test ping first
  results.push(await test('ping()', async () => {
    const ping = await aether.ping();
    if (!ping.ok) throw new Error(ping.error);
    return ping;
  }));

  // Test chain queries
  results.push(await test('getSlot()', () => aether.getSlot()));
  results.push(await test('getBlockHeight()', () => aether.getBlockHeight()));
  results.push(await test('getEpoch()', () => aether.getEpoch()));
  results.push(await test('getTPS()', () => aether.getTPS()));
  results.push(await test('getSupply()', () => aether.getSupply()));
  results.push(await test('getFees()', () => aether.getFees()));
  results.push(await test('getValidators()', () => aether.getValidators()));
  results.push(await test('getPeers()', () => aether.getPeers()));
  results.push(await test('getHealth()', () => aether.getHealth()));
  results.push(await test('getSlotProduction()', () => aether.getSlotProduction()));

  // Test with a sample address (may not exist, but tests the call)
  const testAddress = 'ATH111111111111111111111111111111111';
  results.push(await test(`getAccount('${testAddress.substring(0, 12)}...')`, () => 
    aether.getAccount(testAddress)
  ));
  results.push(await test(`getBalance('${testAddress.substring(0, 12)}...')`, () => 
    aether.getBalance(testAddress)
  ));
  results.push(await test(`getStakePositions('${testAddress.substring(0, 12)}...')`, () => 
    aether.getStakePositions(testAddress)
  ));
  results.push(await test(`getRewards('${testAddress.substring(0, 12)}...')`, () => 
    aether.getRewards(testAddress)
  ));

  // Summary
  const passed = results.filter(r => r.ok).length;
  const failed = results.filter(r => !r.ok).length;

  console.log();
  if (failed === 0) {
    console.log(`  ${C.green}══ All ${passed} tests passed! ══${C.reset}\n`);
  } else {
    console.log(`  ${C.yellow}══ ${passed} passed, ${failed} failed ══${C.reset}`);
    console.log(`  ${C.dim}  (Failures may be due to RPC not running or test data not existing)${C.reset}\n`);
  }

  process.exit(failed > 0 ? 1 : 0);
}

main();
