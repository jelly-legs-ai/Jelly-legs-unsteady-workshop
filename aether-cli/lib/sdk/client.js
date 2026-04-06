/**
 * AetherChain SDK Client
 * 
 * Real blockchain RPC client for the Aether testnet.
 * All functions make actual HTTP calls to http://127.0.0.1:8899.
 * 
 * Usage:
 *   const { AetherClient } = require('./lib/sdk/client');
 *   const client = new AetherClient('http://127.0.0.1:8899');
 *   const slot = await client.getSlot();
 * 
 * Chain must be running: `aether-validator.exe start --genesis genesis.json --no-stake`
 */

const http = require('http');

class AetherClient {
  /**
   * @param {string} rpcUrl - RPC endpoint, e.g. 'http://127.0.0.1:8899'
   */
  constructor(rpcUrl = 'http://127.0.0.1:8899') {
    this.rpcUrl = rpcUrl;
  }

  /**
   * Make an HTTP GET request to the RPC server.
   * @private
   */
  _get(path) {
    return new Promise((resolve, reject) => {
      const url = new URL(path, this.rpcUrl);
      const req = http.get(url.href, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try {
            resolve(JSON.parse(data));
          } catch {
            resolve(data);
          }
        });
      });
      req.on('error', reject);
      req.setTimeout(5000, () => {
        req.destroy();
        reject(new Error(`Request timeout: ${url.href}`));
      });
    });
  }

  /**
   * Make an HTTP POST request to the RPC server.
   * @private
   */
  _post(path, body) {
    return new Promise((resolve, reject) => {
      const url = new URL(path, this.rpcUrl);
      const data = JSON.stringify(body);
      const req = http.request({
        hostname: url.hostname,
        port: url.port,
        path: url.pathname,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(data),
        },
      }, (res) => {
        let body = '';
        res.on('data', chunk => body += chunk);
        res.on('end', () => {
          try {
            resolve(JSON.parse(body));
          } catch {
            resolve(body);
          }
        });
      });
      req.on('error', reject);
      req.write(data);
      req.end();
      req.setTimeout(5000, () => {
        req.destroy();
        reject(new Error(`Request timeout: ${url.href}`));
      });
    });
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Chain State (Real RPC calls)
  // ─────────────────────────────────────────────────────────────────────────

  /**
   * GET /v1/slot — Current slot info
   * @returns {{ slot: number, block_hash: string, parent_block_hash: string, healthy: boolean, error: string|null }}
   */
  async getSlot() {
    return this._get('/v1/slot');
  }

  /**
   * GET /v1/blockheight | /v1/block_height | /v1/height — Current block height (alias for slot)
   * @returns {{ blockHeight: number, slot: number }}
   */
  async getBlockHeight() {
    return this._get('/v1/blockheight');
  }

  /**
   * GET /v1/block?slot=N — Get block by slot number
   * @param {number} slot
   * @returns {{ slot: number, timestamp: number, block_hash: string, previous_block_hash: string, poh_seed: string, transaction_count: number } | null}
   */
  async getBlock(slot) {
    try {
      return await this._get(`/v1/block?slot=${slot}`);
    } catch {
      return null;
    }
  }

  /**
   * GET /v1/genesis — Genesis configuration
   * @returns {{ chain_id: string, genesis_hash: string }}
   */
  async getGenesis() {
    return this._get('/v1/genesis');
  }

  /**
   * GET /v1/epoch — Epoch information
   * @returns {{ epoch: number, slot_index: number, slots_in_epoch: number, absolute_slot: number, transaction_count: number }}
   */
  async getEpoch() {
    return this._get('/v1/epoch');
  }

  /**
   * GET /v1/block_production — Block production stats
   * @returns {{ blocks_produced: number, entries_produced: number, epoch: number, ... }}
   */
  async getBlockProduction() {
    return this._get('/v1/block_production');
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Validators (Real RPC calls)
  // ─────────────────────────────────────────────────────────────────────────

  /**
   * GET /v1/validators — Connected validators (from genesis bootstrap + connected peers)
   * @returns {Array<{ identity_pubkey: string, activated_stake: number, commission: number, active: boolean }>}
   */
  async getValidators() {
    return this._get('/v1/validators');
  }

  /**
   * GET /v1/validator/info — Current validator tier and consensus weight
   * @returns {{ tier: string, consensus_weight: number, can_produce_blocks: boolean, can_vote: boolean }}
   */
  async getValidatorInfo() {
    return this._get('/v1/validator/info');
  }

  /**
   * GET /v1/voteAccounts | /v1/vote_accounts | /v1/getVoteAccounts — Vote accounts
   * @returns {{ vote_accounts: Array, total_stake: number }}
   */
  async getVoteAccounts() {
    return this._get('/v1/voteAccounts');
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Accounts & Transactions (Real RPC calls)
  // ─────────────────────────────────────────────────────────────────────────

  /**
   * GET /v1/account/<address> — Get account info
   * @param {string} address - Base58 encoded account address
   * @returns {{ address: string, lamports: number, owner: string, data_size: number, rent_epoch: number }}
   */
  async getAccount(address) {
    return this._get(`/v1/account/${address}`);
  }

  /**
   * GET /v1/total_supply — Total token supply
   * @returns {{ total_supply: string, unit: string }}
   */
  async getTotalSupply() {
    return this._get('/v1/total_supply');
  }

  /**
   * POST /v1/tx | /v1/submit — Submit a transaction
   * @param {{ tx_type: string, signer: string, signature: string, payload?: object, fee?: number }} tx
   * @returns {{ signature: string, slot: number }} | {{ error: string }}
   */
  async sendTransaction(tx) {
    return this._post('/v1/tx', tx);
  }

  /**
   * GET /v1/tx/<signature> | /v1/getTransaction?signature=... — Get transaction status
   * @param {string} signature - Base58 transaction signature
   * @returns {{ signature: string, slot: number, block_hash: string, success: boolean, error: string|null, timestamp: number } | null}
   */
  async getTransaction(signature) {
    try {
      return await this._get(`/v1/tx/${signature}`);
    } catch {
      return null;
    }
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Health
  // ─────────────────────────────────────────────────────────────────────────

  /**
   * GET /health — Health check
   * @returns {{ status: string }}
   */
  async health() {
    return this._get('/health');
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Utility
  // ─────────────────────────────────────────────────────────────────────────

  /**
   * Check if chain is reachable and healthy.
   * @returns {{ reachable: boolean, slot: number|null, healthy: boolean }}
   */
  async ping() {
    try {
      const slotInfo = await this.getSlot();
      return {
        reachable: true,
        slot: slotInfo.slot,
        healthy: slotInfo.healthy,
        blockHash: slotInfo.block_hash,
      };
    } catch (err) {
      return { reachable: false, slot: null, healthy: false, error: err.message };
    }
  }
}

module.exports = { AetherClient };
