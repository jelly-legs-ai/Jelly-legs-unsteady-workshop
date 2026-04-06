/**
 * @jellylegsai/aether-sdk
 * AetherChain SDK - Real blockchain RPC client
 * 
 * All functions make actual HTTP calls to http://127.0.0.1:8899
 * No stubs, no mocks - only real blockchain interactions.
 */

const { AetherClient } = require('./client');

// Export the main client class
module.exports = {
  AetherClient,
  
  // Convenience: create a default client pointing to local node
  createClient: (rpcUrl = 'http://127.0.0.1:8899') => new AetherClient(rpcUrl),
  
  // Default export
  default: AetherClient,
};
