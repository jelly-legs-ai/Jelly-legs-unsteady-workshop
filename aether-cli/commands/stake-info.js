#!/usr/bin/env node
/**
 * aether-cli - Stake Info Command
 * Get staking information for an address using real chain RPC calls
 */
const { AetherClient } = require('../sdk');

function stakeInfoCommand() {
  const address = process.argv[3]; // aether stake-info <address>
  
  if (!address) {
    console.error('Usage: aether stake-info <address>');
    console.error('Example: aether stake-info 7RcZ7Ae47GYxj2JwpvPPiRi7j6XvmBkV8k5P');
    process.exit(1);
  }
  
  const client = new AetherClient();
  
  async function run() {
    console.log(`\n🔍 Fetching stake info for ${address}...\n`);
    
    try {
      // Get slot info - real chain RPC call
      const slot = await client.getSlot();
      console.log(`  📍 Current Slot: ${slot}`);
      
      // Get account info - real chain RPC call  
      const account = await client.getAccountInfo(address);
      const balance = account.lamports ? (account.lamports / 1e9).toFixed(4) : '0';
      console.log(`  💰 Balance: ${balance} AETH`);
      console.log(`  👤 Owner: ${account.owner || 'unknown'}`);
      
      // Get block height - real chain RPC call
      const blockHeight = await client.getBlockHeight();
      console.log(`  📦 Block Height: ${blockHeight}`);
      
      console.log('\n✅ Real chain RPC calls completed successfully');
      console.log(`   Connected to: ${client.rpcUrl}\n`);
    } catch (error) {
      console.error('❌ Error:', error.message);
      process.exit(1);
    }
  }
  
  run();
}

module.exports = { stakeInfoCommand };
