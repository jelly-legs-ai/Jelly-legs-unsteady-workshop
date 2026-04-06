/**
 * Aether SDK Tests - Real blockchain RPC calls
 * Tests against http://127.0.0.1:8899
 */

const { AetherClient } = require('../lib/sdk/client');

const client = new AetherClient('http://127.0.0.1:8899');
let testsPassed = 0;
let testsFailed = 0;

async function test(name, fn) {
  try {
    await fn();
    console.log(`✓ ${name}`);
    testsPassed++;
  } catch (err) {
    console.log(`✗ ${name}: ${err.message}`);
    testsFailed++;
  }
}

async function runTests() {
  console.log('=== Aether SDK Tests ===\n');
  console.log('Testing against http://127.0.0.1:8899\n');

  // Test 1: Ping
  await test('ping() - Chain reachable', async () => {
    const result = await client.ping();
    if (!result.reachable) throw new Error('Chain not reachable');
    if (typeof result.slot !== 'number') throw new Error('Invalid slot');
    console.log(`    Slot: ${result.slot}, Healthy: ${result.healthy}`);
  });

  // Test 2: Get Slot
  await test('getSlot() - Real RPC call', async () => {
    const slot = await client.getSlot();
    if (typeof slot.slot !== 'number') throw new Error('Invalid slot number');
    if (!slot.block_hash) throw new Error('Missing block_hash');
    console.log(`    Slot: ${slot.slot}, Hash: ${slot.block_hash.slice(0, 20)}...`);
  });

  // Test 3: Get Block Height
  await test('getBlockHeight() - Real RPC call', async () => {
    const height = await client.getBlockHeight();
    if (typeof height.blockHeight !== 'number') throw new Error('Invalid blockHeight');
    console.log(`    BlockHeight: ${height.blockHeight}`);
  });

  // Test 4: Get Genesis
  await test('getGenesis() - Real RPC call', async () => {
    const genesis = await client.getGenesis();
    if (!genesis.chain_id) throw new Error('Missing chain_id');
    if (!genesis.genesis_hash) throw new Error('Missing genesis_hash');
    console.log(`    Chain: ${genesis.chain_id}`);
  });

  // Test 5: Get Epoch
  await test('getEpoch() - Real RPC call', async () => {
    const epoch = await client.getEpoch();
    if (typeof epoch.epoch !== 'number') throw new Error('Invalid epoch');
    console.log(`    Epoch: ${epoch.epoch}, SlotIndex: ${epoch.slot_index}`);
  });

  // Test 6: Get Validators
  await test('getValidators() - Real RPC call', async () => {
    const validators = await client.getValidators();
    if (!Array.isArray(validators)) throw new Error('Invalid validators response');
    console.log(`    Validators count: ${validators.length}`);
  });

  // Test 7: Get Vote Accounts
  await test('getVoteAccounts() - Real RPC call', async () => {
    const accounts = await client.getVoteAccounts();
    if (!accounts.vote_accounts) throw new Error('Missing vote_accounts');
    console.log(`    Vote accounts: ${accounts.vote_accounts.length}`);
  });

  // Test 8: Get Block Production
  await test('getBlockProduction() - Real RPC call', async () => {
    const stats = await client.getBlockProduction();
    if (typeof stats.blocks_produced !== 'number') throw new Error('Invalid blocks_produced');
    console.log(`    Blocks produced: ${stats.blocks_produced}`);
  });

  // Test 9: Get Total Supply
  await test('getTotalSupply() - Real RPC call', async () => {
    const supply = await client.getTotalSupply();
    if (!supply.total_supply) throw new Error('Missing total_supply');
    console.log(`    Total supply: ${supply.total_supply} ${supply.unit || 'lamports'}`);
  });

  // Test 10: Get Health
  await test('health() - Real RPC call', async () => {
    const health = await client.health();
    if (!health.status) throw new Error('Missing status');
    console.log(`    Health status: ${health.status}`);
  });

  // Test 11: Get Block
  await test('getBlock() - Real RPC call', async () => {
    const slotInfo = await client.getSlot();
    const block = await client.getBlock(slotInfo.slot);
    if (!block) {
      console.log('    Block not found (expected for new slots)');
      return;
    }
    if (!block.block_hash) throw new Error('Missing block_hash');
    console.log(`    Block slot: ${block.slot}, TXs: ${block.transaction_count}`);
  });

  // Test 12: Get Account (using a dummy address - may fail if chain not initialized)
  await test('getAccount() - Real RPC call', async () => {
    try {
      const account = await client.getAccount('GdGs8zKhYnAYmcLqwX3oGEnptoBB2khX6yxqDqqo9RFZ');
      console.log(`    Account balance: ${account.lamports} lamports`);
    } catch (err) {
      // Account may not exist, that's ok
      console.log('    Account not found (expected if address not on chain)');
    }
  });

  console.log('\n=== Summary ===');
  console.log(`Passed: ${testsPassed}`);
  console.log(`Failed: ${testsFailed}`);
  console.log(`Total: ${testsPassed + testsFailed}`);

  if (testsFailed > 0) {
    process.exit(1);
  }
}

runTests().catch(err => {
  console.error('Test suite failed:', err.message);
  process.exit(1);
});
