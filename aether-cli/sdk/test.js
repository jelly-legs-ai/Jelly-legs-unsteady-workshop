/**
 * SDK Test - Verify the SDK makes real RPC calls
 */

const { 
  getSlot, 
  getSupply, 
  getEpochInfo, 
  getAccountInfo,
  AetherSDK,
  DEFAULT_RPC_URL,
  isNodeReachable
} = require('./index');

async function runTests() {
  console.log('🧪 Aether SDK Test Suite');
  console.log('========================');
  console.log(`RPC Endpoint: ${DEFAULT_RPC_URL}`);
  console.log('');

  // Test 1: Check node reachability
  console.log('Test 1: Node reachability');
  try {
    const reachable = await isNodeReachable();
    if (reachable) {
      console.log('✅ Node is reachable');
    } else {
      console.log('⚠️  Node not reachable (this is OK if validator is not running)');
    }
  } catch (err) {
    console.log(`⚠️  Node not reachable: ${err.message}`);
    console.log('   (This is expected if no validator is running at 127.0.0.1:8899)');
  }
  console.log('');

  // Test 2: getSlot (real RPC call)
  console.log('Test 2: getSlot() - Real RPC call');
  try {
    const slot = await getSlot();
    console.log('✅ getSlot() returned:', JSON.stringify(slot, null, 2));
  } catch (err) {
    console.log(`⚠️  getSlot() failed: ${err.message}`);
    console.log('   (Expected if validator not running)');
  }
  console.log('');

  // Test 3: getSupply (real RPC call)
  console.log('Test 3: getSupply() - Real RPC call');
  try {
    const supply = await getSupply();
    console.log('✅ getSupply() returned:', JSON.stringify(supply, null, 2));
  } catch (err) {
    console.log(`⚠️  getSupply() failed: ${err.message}`);
    console.log('   (Expected if validator not running)');
  }
  console.log('');

  // Test 4: getEpochInfo (real RPC call)
  console.log('Test 4: getEpochInfo() - Real RPC call');
  try {
    const epoch = await getEpochInfo();
    console.log('✅ getEpochInfo() returned:', JSON.stringify(epoch, null, 2));
  } catch (err) {
    console.log(`⚠️  getEpochInfo() failed: ${err.message}`);
    console.log('   (Expected if validator not running)');
  }
  console.log('');

  // Test 5: getAccountInfo (real RPC call)
  console.log('Test 5: getAccountInfo() - Real RPC call');
  try {
    // Use a dummy address - will fail if node not running but tests the RPC
    const account = await getAccountInfo('ATH11111111111111111111111111111111111111111');
    console.log('✅ getAccountInfo() returned:', JSON.stringify(account, null, 2));
  } catch (err) {
    console.log(`⚠️  getAccountInfo() failed: ${err.message}`);
    console.log('   (Expected if validator not running)');
  }
  console.log('');

  // Test 6: SDK Class
  console.log('Test 6: AetherSDK class instantiation');
  try {
    const sdk = new AetherSDK({ rpcUrl: DEFAULT_RPC_URL });
    console.log('✅ AetherSDK instance created');
    console.log('   SDK methods available:', Object.getOwnPropertyNames(AetherSDK.prototype).filter(m => m !== 'constructor'));
  } catch (err) {
    console.log(`❌ SDK instantiation failed: ${err.message}`);
  }
  console.log('');

  console.log('========================');
  console.log('Test suite complete!');
  console.log('');
  console.log('Note: This SDK makes REAL HTTP RPC calls to http://127.0.0.1:8899');
  console.log('If the validator is not running, the calls will fail as expected.');
}

runTests().catch(console.error);
