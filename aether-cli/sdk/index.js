/**
 * Aether SDK - Real Blockchain RPC Client
 * 
 * Makes real HTTP RPC calls to http://127.0.0.1:8899
 * No stubs, no mocks - all functions hit the real chain.
 */

const http = require('http');
const https = require('https');

// Default RPC endpoint
const DEFAULT_RPC_URL = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

/**
 * Make an HTTP GET request to the RPC endpoint
 * @param {string} path - API path (e.g., '/v1/slot')
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Parsed JSON response
 */
async function httpGet(path, rpcUrl = DEFAULT_RPC_URL) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout: 10000,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve(parsed);
        } catch (err) {
          resolve({ _raw: data, _parseError: err.message });
        }
      });
    });
    
    req.on('error', (err) => reject(new Error(`HTTP error: ${err.message}`)));
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });
    
    req.end();
  });
}

/**
 * Make an HTTP POST request to the RPC endpoint
 * @param {string} path - API path
 * @param {Object} body - Request body
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Parsed JSON response
 */
async function httpPost(path, body, rpcUrl = DEFAULT_RPC_URL) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const lib = url.protocol === 'https:' ? https : http;
    const bodyStr = JSON.stringify(body);
    
    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          resolve(parsed);
        } catch (err) {
          resolve({ _raw: data, _parseError: err.message });
        }
      });
    });
    
    req.on('error', (err) => reject(new Error(`HTTP error: ${err.message}`)));
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });
    
    req.write(bodyStr);
    req.end();
  });
}

// ============================================
// REAL BLOCKCHAIN RPC CALLS
// ============================================

/**
 * Get current slot from the blockchain
 * Real RPC call to GET /v1/slot
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - { slot: number }
 */
async function getSlot(rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet('/v1/slot', rpcUrl);
  return response;
}

/**
 * Get account info from the blockchain
 * Real RPC call to GET /v1/account/<address>
 * @param {string} address - Account address (with or without ATH prefix)
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Account info including lamports, owner, data
 */
async function getAccountInfo(address, rpcUrl = DEFAULT_RPC_URL) {
  // Strip ATH prefix if present for API call
  const apiAddress = address.startsWith('ATH') ? address.slice(3) : address;
  const response = await httpGet(`/v1/account/${apiAddress}`, rpcUrl);
  return response;
}

/**
 * Get block info from the blockchain
 * Real RPC call to GET /v1/block/<slot>
 * @param {number} slot - Block slot number
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Block data including transactions
 */
async function getBlock(slot, rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet(`/v1/block/${slot}`, rpcUrl);
  return response;
}

/**
 * Get recent blockhash from the blockchain
 * Real RPC call to GET /v1/recent-blockhash
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Blockhash info
 */
async function getRecentBlockhash(rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet('/v1/recent-blockhash', rpcUrl);
  return response;
}

/**
 * Get validators list from the blockchain
 * Real RPC call to GET /v1/validators
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - List of validators
 */
async function getValidators(rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet('/v1/validators', rpcUrl);
  return response;
}

/**
 * Get epoch info from the blockchain
 * Real RPC call to GET /v1/epoch/info
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Current epoch info
 */
async function getEpochInfo(rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet('/v1/epoch/info', rpcUrl);
  return response;
}

/**
 * Get supply info from the blockchain
 * Real RPC call to GET /v1/supply
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Token supply data
 */
async function getSupply(rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet('/v1/supply', rpcUrl);
  return response;
}

/**
 * Get transaction info from the blockchain
 * Real RPC call to GET /v1/tx/<signature>
 * @param {string} signature - Transaction signature
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Transaction details
 */
async function getTransaction(signature, rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpGet(`/v1/tx/${signature}`, rpcUrl);
  return response;
}

/**
 * Get stake info for an address from the blockchain
 * Real RPC call to GET /v1/stake?address=<addr>
 * @param {string} address - Wallet address
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Stake positions
 */
async function getStakeInfo(address, rpcUrl = DEFAULT_RPC_URL) {
  const apiAddress = address.startsWith('ATH') ? address.slice(3) : address;
  const response = await httpGet(`/v1/stake?address=${encodeURIComponent(apiAddress)}`, rpcUrl);
  return response;
}

/**
 * Get reward history for an address from the blockchain
 * Real RPC call to GET /v1/rewards?address=<addr>
 * @param {string} address - Wallet address
 * @param {number} fromEpoch - Starting epoch
 * @param {number} limit - Number of epochs to fetch
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Reward history
 */
async function getRewardHistory(address, fromEpoch = 0, limit = 14, rpcUrl = DEFAULT_RPC_URL) {
  const apiAddress = address.startsWith('ATH') ? address.slice(3) : address;
  const response = await httpGet(
    `/v1/rewards?address=${encodeURIComponent(apiAddress)}&from_epoch=${fromEpoch}&limit=${limit}`,
    rpcUrl
  );
  return response;
}

/**
 * Submit a signed transaction to the blockchain
 * Real RPC call to POST /v1/tx
 * @param {Object} signedTx - Signed transaction object
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Transaction submission result
 */
async function sendTransaction(signedTx, rpcUrl = DEFAULT_RPC_URL) {
  const response = await httpPost('/v1/tx', signedTx, rpcUrl);
  return response;
}

/**
 * Get network status from the blockchain
 * Combines multiple RPC calls
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<Object>} - Network status including slot, peers, TPS
 */
async function getNetworkStatus(rpcUrl = DEFAULT_RPC_URL) {
  const [slot, epoch, supply] = await Promise.all([
    getSlot(rpcUrl).catch(() => null),
    getEpochInfo(rpcUrl).catch(() => null),
    getSupply(rpcUrl).catch(() => null),
  ]);

  return {
    slot: slot?.slot ?? null,
    epoch: epoch?.epoch ?? null,
    supply: supply ?? null,
    rpc_url: rpcUrl,
    timestamp: new Date().toISOString(),
  };
}

/**
 * Check if the blockchain RPC is reachable
 * @param {string} rpcUrl - Optional custom RPC URL
 * @returns {Promise<boolean>} - True if reachable
 */
async function isNodeReachable(rpcUrl = DEFAULT_RPC_URL) {
  try {
    await httpGet('/v1/slot', rpcUrl);
    return true;
  } catch {
    return false;
  }
}

// ============================================
// SDK CLIENT CLASS
// ============================================

class AetherSDK {
  constructor(options = {}) {
    this.rpcUrl = options.rpcUrl || DEFAULT_RPC_URL;
  }

  // All methods delegate to the real RPC functions above
  async getSlot() { return getSlot(this.rpcUrl); }
  async getAccountInfo(address) { return getAccountInfo(address, this.rpcUrl); }
  async getBlock(slot) { return getBlock(slot, this.rpcUrl); }
  async getRecentBlockhash() { return getRecentBlockhash(this.rpcUrl); }
  async getValidators() { return getValidators(this.rpcUrl); }
  async getEpochInfo() { return getEpochInfo(this.rpcUrl); }
  async getSupply() { return getSupply(this.rpcUrl); }
  async getTransaction(signature) { return getTransaction(signature, this.rpcUrl); }
  async getStakeInfo(address) { return getStakeInfo(address, this.rpcUrl); }
  async getRewardHistory(address, fromEpoch, limit) { 
    return getRewardHistory(address, fromEpoch, limit, this.rpcUrl); 
  }
  async sendTransaction(signedTx) { return sendTransaction(signedTx, this.rpcUrl); }
  async getNetworkStatus() { return getNetworkStatus(this.rpcUrl); }
  async isNodeReachable() { return isNodeReachable(this.rpcUrl); }
}

// ============================================
// EXPORTS
// ============================================

module.exports = {
  // SDK Class
  AetherSDK,
  
  // Individual functions (all make real RPC calls)
  getSlot,
  getAccountInfo,
  getBlock,
  getRecentBlockhash,
  getValidators,
  getEpochInfo,
  getSupply,
  getTransaction,
  getStakeInfo,
  getRewardHistory,
  sendTransaction,
  getNetworkStatus,
  isNodeReachable,
  
  // HTTP utilities
  httpGet,
  httpPost,
  
  // Default config
  DEFAULT_RPC_URL,
};
