/**
 * @jellylegsai/aether-sdk
 * 
 * Official Aether Blockchain SDK - Real HTTP RPC calls to Aether nodes
 * No stubs, no mocks - every function makes actual blockchain calls
 * 
 * Default RPC: http://127.0.0.1:8899 (configurable via constructor or AETHER_RPC env)
 */

const http = require('http');
const https = require('https');

// Default configuration
const DEFAULT_RPC_URL = 'http://127.0.0.1:8899';
const DEFAULT_TIMEOUT_MS = 10000;

/**
 * Aether SDK Client
 * Real blockchain interface layer - every method makes actual HTTP RPC calls
 */
class AetherClient {
  constructor(options = {}) {
    this.rpcUrl = options.rpcUrl || process.env.AETHER_RPC || DEFAULT_RPC_URL;
    this.timeoutMs = options.timeoutMs || DEFAULT_TIMEOUT_MS;
    
    // Parse RPC URL
    const url = new URL(this.rpcUrl);
    this.protocol = url.protocol;
    this.hostname = url.hostname;
    this.port = url.port || (this.protocol === 'https:' ? 443 : 80);
  }

  /**
   * Internal: Make HTTP GET request to RPC endpoint
   */
  _httpGet(path, timeoutMs = this.timeoutMs) {
    return new Promise((resolve, reject) => {
      const lib = this.protocol === 'https:' ? https : http;
      const req = lib.request({
        hostname: this.hostname,
        port: this.port,
        path: path,
        method: 'GET',
        timeout: timeoutMs,
        headers: { 'Content-Type': 'application/json' },
      }, (res) => {
        let data = '';
        res.on('data', (chunk) => data += chunk);
        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);
            if (parsed.error) {
              reject(new Error(parsed.error.message || JSON.stringify(parsed.error)));
            } else {
              resolve(parsed);
            }
          } catch (e) {
            resolve({ raw: data });
          }
        });
      });
      req.on('error', reject);
      req.on('timeout', () => {
        req.destroy();
        reject(new Error(`Request timeout after ${timeoutMs}ms`));
      });
      req.end();
    });
  }

  /**
   * Internal: Make HTTP POST request to RPC endpoint
   */
  _httpPost(path, body = {}, timeoutMs = this.timeoutMs) {
    return new Promise((resolve, reject) => {
      const lib = this.protocol === 'https:' ? https : http;
      const bodyStr = JSON.stringify(body);
      const req = lib.request({
        hostname: this.hostname,
        port: this.port,
        path: path,
        method: 'POST',
        timeout: timeoutMs,
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
            if (parsed.error) {
              reject(new Error(parsed.error.message || JSON.stringify(parsed.error)));
            } else {
              resolve(parsed);
            }
          } catch (e) {
            resolve({ raw: data });
          }
        });
      });
      req.on('error', reject);
      req.on('timeout', () => {
        req.destroy();
        reject(new Error(`Request timeout after ${timeoutMs}ms`));
      });
      req.write(bodyStr);
      req.end();
    });
  }

  // ============================================================
  // Core RPC Methods - Real blockchain calls
  // ============================================================

  /**
   * Get current slot number
   * RPC: GET /v1/slot
   * 
   * @returns {Promise<number>} Current slot number
   */
  async getSlot() {
    const result = await this._httpGet('/v1/slot');
    return result.slot !== undefined ? result.slot : result;
  }

  /**
   * Get current block height
   * RPC: GET /v1/blockheight
   * 
   * @returns {Promise<number>} Current block height
   */
  async getBlockHeight() {
    const result = await this._httpGet('/v1/blockheight');
    return result.blockHeight !== undefined ? result.blockHeight : result;
  }

  /**
   * Get account info including balance
   * RPC: GET /v1/account/<address>
   * 
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Object>} Account info: { lamports, owner, data, rent_epoch }
   */
  async getAccountInfo(address) {
    if (!address) {
      throw new Error('Address is required');
    }
    const result = await this._httpGet(`/v1/account/${address}`);
    return result;
  }

  /**
   * Alias for getAccountInfo
   * @param {string} address - Account address
   * @returns {Promise<Object>} Account info
   */
  async getAccount(address) {
    return this.getAccountInfo(address);
  }

  /**
   * Get balance in lamports
   * RPC: GET /v1/account/<address>
   * 
   * @param {string} address - Account public key (base58)
   * @returns {Promise<number>} Balance in lamports
   */
  async getBalance(address) {
    const account = await this.getAccountInfo(address);
    return account.lamports !== undefined ? account.lamports : 0;
  }

  /**
   * Get epoch info
   * RPC: GET /v1/epoch
   * 
   * @returns {Promise<Object>} Epoch info: { epoch, slotIndex, slotsInEpoch, absoluteSlot }
   */
  async getEpochInfo() {
    const result = await this._httpGet('/v1/epoch');
    return result;
  }

  /**
   * Get transaction by signature
   * RPC: GET /v1/transaction/<signature>
   * 
   * @param {string} signature - Transaction signature (base58)
   * @returns {Promise<Object>} Transaction details
   */
  async getTransaction(signature) {
    if (!signature) {
      throw new Error('Transaction signature is required');
    }
    const result = await this._httpGet(`/v1/transaction/${signature}`);
    return result;
  }

  /**
   * Submit a signed transaction
   * RPC: POST /v1/transaction
   * 
   * @param {Object} tx - Signed transaction object
   * @param {string} tx.signature - Transaction signature (base58)
   * @param {string} tx.signer - Signer public key (base58)
   * @param {string} tx.tx_type - Transaction type
   * @param {Object} tx.payload - Transaction payload
   * @returns {Promise<Object>} Transaction receipt: { signature, slot, confirmed }
   */
  async sendTransaction(tx) {
    if (!tx || !tx.signature) {
      throw new Error('Transaction with signature is required');
    }
    const result = await this._httpPost('/v1/transaction', tx);
    return result;
  }

  /**
   * Get recent blockhash for transaction signing
   * RPC: GET /v1/recent-blockhash
   * 
   * @returns {Promise<Object>} { blockhash, lastValidBlockHeight }
   */
  async getRecentBlockhash() {
    const result = await this._httpGet('/v1/recent-blockhash');
    return result;
  }

  /**
   * Get network peers
   * RPC: GET /v1/peers
   * 
   * @returns {Promise<Array>} List of peer node addresses
   */
  async getClusterPeers() {
    const result = await this._httpGet('/v1/peers');
    return Array.isArray(result) ? result : (result.peers || []);
  }

  /**
   * Get validator info
   * RPC: GET /v1/validators
   * 
   * @returns {Promise<Array>} List of validators with stake, commission, etc.
   */
  async getValidators() {
    const result = await this._httpGet('/v1/validators');
    return Array.isArray(result) ? result : (result.validators || []);
  }

  /**
   * Get supply info
   * RPC: GET /v1/supply
   * 
   * @returns {Promise<Object>} Supply info: { total, circulating, nonCirculating }
   */
  async getSupply() {
    const result = await this._httpGet('/v1/supply');
    return result;
  }

  /**
   * Get health status
   * RPC: GET /v1/health
   * 
   * @returns {Promise<string>} 'ok' if node is healthy
   */
  async getHealth() {
    const result = await this._httpGet('/v1/health');
    return result.status || result;
  }

  /**
   * Get version info
   * RPC: GET /v1/version
   * 
   * @returns {Promise<Object>} Version info: { aetherCore, featureSet }
   */
  async getVersion() {
    const result = await this._httpGet('/v1/version');
    return result;
  }

  /**
   * Get TPS (transactions per second)
   * RPC: GET /v1/tps
   * 
   * @returns {Promise<number>} Current TPS
   */
  async getTPS() {
    const result = await this._httpGet('/v1/tps');
    return result.tps ?? result.tps_avg ?? result.transactions_per_second ?? null;
  }

  /**
   * Get fee estimates
   * RPC: GET /v1/fees
   * 
   * @returns {Promise<Object>} Fee info
   */
  async getFees() {
    const result = await this._httpGet('/v1/fees');
    return result;
  }

  /**
   * Get slot production stats
   * RPC: POST /v1/slot_production
   * 
   * @returns {Promise<Object>} Slot production stats
   */
  async getSlotProduction() {
    const result = await this._httpPost('/v1/slot_production', {});
    return result;
  }

  /**
   * Get stake positions for an address
   * RPC: GET /v1/stake/<address>
   * 
   * @param {string} address - Account address
   * @returns {Promise<Array>} List of stake positions
   */
  async getStakePositions(address) {
    if (!address) throw new Error('Address is required');
    const result = await this._httpGet(`/v1/stake/${address}`);
    return result.delegations ?? result.stakes ?? result ?? [];
  }

  /**
   * Get rewards for an address
   * RPC: GET /v1/rewards/<address>
   * 
   * @param {string} address - Account address
   * @returns {Promise<Object>} Rewards info
   */
  async getRewards(address) {
    if (!address) throw new Error('Address is required');
    const result = await this._httpGet(`/v1/rewards/${address}`);
    return result;
  }

  /**
   * Get validator APY
   * RPC: GET /v1/validator/<address>/apy
   * 
   * @param {string} validatorAddr - Validator address
   * @returns {Promise<Object>} APY info
   */
  async getValidatorAPY(validatorAddr) {
    if (!validatorAddr) throw new Error('Validator address is required');
    const result = await this._httpGet(`/v1/validator/${validatorAddr}/apy`);
    return result;
  }

  /**
   * Get recent transactions for an address
   * RPC: GET /v1/transactions/<address>?limit=<n>
   *
   * @param {string} address - Account address
   * @param {number} limit - Max transactions to return
   * @returns {Promise<Array>} List of recent transactions
   */
  async getRecentTransactions(address, limit = 20) {
    if (!address) throw new Error('Address is required');
    const result = await this._httpGet(`/v1/transactions/${address}?limit=${limit}`);
    return result.transactions ?? result ?? [];
  }

  /**
   * Get all SPL token accounts for a wallet address
   * RPC: GET /v1/tokens/<address>
   *
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Array>} List of token accounts with mint, amount, decimals
   */
  async getTokenAccounts(address) {
    if (!address) throw new Error('Address is required');
    const result = await this._httpGet(`/v1/tokens/${address}`);
    return result.tokens ?? result.accounts ?? result ?? [];
  }

  /**
   * Get all stake accounts for a wallet address
   * RPC: GET /v1/stake-accounts/<address>
   *
   * @param {string} address - Account public key (base58)
   * @returns {Promise<Array>} List of stake accounts
   */
  async getStakeAccounts(address) {
    if (!address) throw new Error('Address is required');
    const result = await this._httpGet(`/v1/stake-accounts/${address}`);
    return result.stake_accounts ?? result.delegations ?? result ?? [];
  }

  // ============================================================
  // Transaction Helpers - Build and send real transactions
  // ============================================================

  /**
   * Build and send a transfer transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.from - Sender address (base58)
   * @param {string} params.to - Recipient address (base58)
   * @param {number} params.amount - Amount in lamports
   * @param {number} params.nonce - Nonce for replay protection
   * @param {Function} params.signFn - Function to sign the transaction (receives tx object, returns signature)
   * @returns {Promise<Object>} Transaction receipt
   */
  async transfer({ from, to, amount, nonce, signFn }) {
    if (!from || !to || !amount === undefined || nonce === undefined) {
      throw new Error('from, to, amount, and nonce are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required (function to sign the transaction)');
    }

    // Get recent blockhash (real RPC call)
    const { blockhash } = await this.getRecentBlockhash();

    // Build transaction payload
    const tx = {
      signature: '', // Will be filled after signing
      signer: from,
      tx_type: 'Transfer',
      payload: {
        recipient: to,
        amount: BigInt(amount),
        nonce: BigInt(nonce),
      },
      fee: 5000, // 5000 lamports fee
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    // Sign transaction (user provides signing function)
    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    // Send to blockchain (real RPC call)
    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send a stake delegation transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.staker - Staker address (base58)
   * @param {string} params.validator - Validator address (base58)
   * @param {number} params.amount - Amount to stake in lamports
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async stake({ staker, validator, amount, signFn }) {
    if (!staker || !validator || !amount === undefined) {
      throw new Error('staker, validator, and amount are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: staker,
      tx_type: 'Stake',
      payload: {
        validator: validator,
        amount: BigInt(amount),
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send an unstake (withdraw) transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.stakeAccount - Stake account address (base58)
   * @param {number} params.amount - Amount to unstake in lamports
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async unstake({ stakeAccount, amount, signFn }) {
    if (!stakeAccount || !amount === undefined) {
      throw new Error('stakeAccount and amount are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: stakeAccount,
      tx_type: 'Unstake',
      payload: {
        stake_account: stakeAccount,
        amount: BigInt(amount),
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Build and send a claim rewards transaction
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.stakeAccount - Stake account address (base58)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async claimRewards({ stakeAccount, signFn }) {
    if (!stakeAccount) {
      throw new Error('stakeAccount is required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: stakeAccount,
      tx_type: 'ClaimRewards',
      payload: {
        stake_account: stakeAccount,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  // ============================================================
  // NFT Methods - Real blockchain calls for NFT operations
  // ============================================================

  /**
   * Create a new NFT
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.creator - Creator address (base58)
   * @param {string} params.metadataUrl - URL to NFT metadata (JSON)
   * @param {number} params.royalties - Royalty basis points (e.g., 500 = 5%)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt with NFT ID
   */
  async createNFT({ creator, metadataUrl, royalties, signFn }) {
    if (!creator || !metadataUrl || royalties === undefined) {
      throw new Error('creator, metadataUrl, and royalties are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: creator,
      tx_type: 'CreateNFT',
      payload: {
        metadata_url: metadataUrl,
        royalties: royalties,
      },
      fee: 10000, // Higher fee for NFT creation
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Transfer an NFT to another address
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.from - Current owner address (base58)
   * @param {string} params.nftId - NFT ID
   * @param {string} params.to - Recipient address (base58)
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async transferNFT({ from, nftId, to, signFn }) {
    if (!from || !nftId || !to) {
      throw new Error('from, nftId, and to are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: from,
      tx_type: 'TransferNFT',
      payload: {
        nft_id: nftId,
        recipient: to,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }

  /**
   * Update NFT metadata URL
   * Makes real RPC calls: getRecentBlockhash() + sendTransaction()
   * 
   * @param {Object} params
   * @param {string} params.creator - NFT creator/owner address (base58)
   * @param {string} params.nftId - NFT ID
   * @param {string} params.metadataUrl - New metadata URL
   * @param {Function} params.signFn - Function to sign the transaction
   * @returns {Promise<Object>} Transaction receipt
   */
  async updateMetadata({ creator, nftId, metadataUrl, signFn }) {
    if (!creator || !nftId || !metadataUrl) {
      throw new Error('creator, nftId, and metadataUrl are required');
    }
    if (!signFn || typeof signFn !== 'function') {
      throw new Error('signFn is required');
    }

    const { blockhash } = await this.getRecentBlockhash();

    const tx = {
      signature: '',
      signer: creator,
      tx_type: 'UpdateMetadata',
      payload: {
        nft_id: nftId,
        metadata_url: metadataUrl,
      },
      fee: 5000,
      slot: await this.getSlot(),
      timestamp: Date.now(),
    };

    const signature = await signFn(tx, blockhash);
    tx.signature = signature;

    const receipt = await this.sendTransaction(tx);
    return receipt;
  }
}

// ============================================================
// Convenience Functions (for quick one-off calls)
// ============================================================

/**
 * Create a new AetherClient instance
 * @param {Object} options - Client options
 * @returns {AetherClient}
 */
function createClient(options = {}) {
  return new AetherClient(options);
}

/**
 * Quick slot check (uses default RPC)
 * @returns {Promise<number>} Current slot
 */
async function getSlot() {
  const client = new AetherClient();
  return client.getSlot();
}

/**
 * Quick balance check (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<number>} Balance in lamports
 */
async function getBalance(address) {
  const client = new AetherClient();
  return client.getBalance(address);
}

/**
 * Quick health check (uses default RPC)
 * @returns {Promise<string>} 'ok' if healthy
 */
async function getHealth() {
  const client = new AetherClient();
  return client.getHealth();
}

/**
 * Get current block height (uses default RPC)
 * @returns {Promise<number>} Block height
 */
async function getBlockHeight() {
  const client = new AetherClient();
  return client.getBlockHeight();
}

/**
 * Get epoch info (uses default RPC)
 * @returns {Promise<Object>} Epoch info
 */
async function getEpoch() {
  const client = new AetherClient();
  return client.getEpochInfo();
}

/**
 * Get TPS (uses default RPC)
 * @returns {Promise<number>} Transactions per second
 */
async function getTPS() {
  const client = new AetherClient();
  return client.getTPS();
}

/**
 * Get supply info (uses default RPC)
 * @returns {Promise<Object>} Supply info
 */
async function getSupply() {
  const client = new AetherClient();
  return client.getSupply();
}

/**
 * Get fees info (uses default RPC)
 * @returns {Promise<Object>} Fee info
 */
async function getFees() {
  const client = new AetherClient();
  return client.getFees();
}

/**
 * Get validators list (uses default RPC)
 * @returns {Promise<Array>} List of validators
 */
async function getValidators() {
  const client = new AetherClient();
  return client.getValidators();
}

/**
 * Get peers list (uses default RPC)
 * @returns {Promise<Array>} List of peers
 */
async function getPeers() {
  const client = new AetherClient();
  return client.getClusterPeers();
}

/**
 * Get slot production stats (uses default RPC)
 * @returns {Promise<Object>} Slot production stats
 */
async function getSlotProduction() {
  const client = new AetherClient();
  return client.getSlotProduction();
}

/**
 * Get account info (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Object>} Account info
 */
async function getAccount(address) {
  const client = new AetherClient();
  return client.getAccount(address);
}

/**
 * Get stake positions (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Stake positions
 */
async function getStakePositions(address) {
  const client = new AetherClient();
  return client.getStakePositions(address);
}

/**
 * Get rewards info (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Object>} Rewards info
 */
async function getRewards(address) {
  const client = new AetherClient();
  return client.getRewards(address);
}

/**
 * Get transaction by signature (uses default RPC)
 * @param {string} signature - Transaction signature
 * @returns {Promise<Object>} Transaction info
 */
async function getTransaction(signature) {
  const client = new AetherClient();
  return client.getTransaction(signature);
}

/**
 * Get recent transactions (uses default RPC)
 * @param {string} address - Account address
 * @param {number} limit - Max transactions
 * @returns {Promise<Array>} Recent transactions
 */
async function getRecentTransactions(address, limit = 20) {
  const client = new AetherClient();
  return client.getRecentTransactions(address, limit);
}

/**
 * Get all SPL token accounts for a wallet (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Token accounts with mint, amount, decimals
 */
async function getTokenAccounts(address) {
  const client = new AetherClient();
  return client.getTokenAccounts(address);
}

/**
 * Get all stake accounts for a wallet (uses default RPC)
 * @param {string} address - Account address
 * @returns {Promise<Array>} Stake accounts list
 */
async function getStakeAccounts(address) {
  const client = new AetherClient();
  return client.getStakeAccounts(address);
}

/**
 * Get validator APY (uses default RPC)
 * @param {string} validatorAddr - Validator address
 * @returns {Promise<Object>} APY info
 */
async function getValidatorAPY(validatorAddr) {
  const client = new AetherClient();
  return client.getValidatorAPY(validatorAddr);
}

/**
 * Send transaction (uses default RPC)
 * @param {Object} tx - Signed transaction
 * @returns {Promise<Object>} Transaction receipt
 */
async function sendTransaction(tx) {
  const client = new AetherClient();
  return client.sendTransaction(tx);
}

/**
 * Ping RPC endpoint
 * @param {string} rpcUrl - RPC URL to ping
 * @returns {Promise<Object>} Ping result with latency
 */
async function ping(rpcUrl) {
  const client = new AetherClient({ rpcUrl });
  const start = Date.now();
  try {
    await client.getSlot();
    return { ok: true, latency: Date.now() - start, rpc: rpcUrl || DEFAULT_RPC_URL };
  } catch (err) {
    return { ok: false, error: err.message, rpc: rpcUrl || DEFAULT_RPC_URL };
  }
}

// Low-level RPC helpers (from rpc.js)
const { rpcGet, rpcPost } = require('./rpc');

// ============================================================
// Exports
// ============================================================

module.exports = {
  // Main class
  AetherClient,
  
  // Factory function
  createClient,
  
  // Convenience functions (all chain queries)
  getSlot,
  getBlockHeight,
  getEpoch,
  getAccount,
  getBalance,
  getTransaction,
  getRecentTransactions,
  getTokenAccounts,
  getStakeAccounts,
  getValidators,
  getTPS,
  getSupply,
  getSlotProduction,
  getFees,
  getStakePositions,
  getRewards,
  getValidatorAPY,
  getPeers,
  getHealth,
  
  // Transactions
  sendTransaction,
  
  // Utilities
  ping,
  
  // Low-level RPC
  rpcGet,
  rpcPost,
  
  // Constants
  DEFAULT_RPC_URL,
  DEFAULT_TIMEOUT_MS,
};
