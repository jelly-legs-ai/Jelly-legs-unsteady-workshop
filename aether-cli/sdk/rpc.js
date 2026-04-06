#!/usr/bin/env node
/**
 * @jellylegsai/aether-sdk - RPC Client
 * 
 * Low-level HTTP RPC client for Aether blockchain.
 * All functions make REAL HTTP calls to the blockchain RPC endpoint.
 * No stubs, no mocks.
 */

const http = require('http');
const https = require('https');

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

/**
 * Make a GET request to the RPC endpoint
 * @param {string} rpcUrl - RPC endpoint URL
 * @param {string} path - API path (e.g., /v1/slot)
 * @param {number} timeout - Request timeout in ms
 * @returns {Promise<object>} Parsed JSON response
 */
function rpcGet(rpcUrl, path, timeout = 8000) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'GET',
      timeout,
      headers: { 'Content-Type': 'application/json' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch {
          resolve({ raw: data });
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error(`RPC request timeout after ${timeout}ms`));
    });
    req.end();
  });
}

/**
 * Make a POST request to the RPC endpoint
 * @param {string} rpcUrl - RPC endpoint URL
 * @param {string} path - API path
 * @param {object} body - Request body (will be JSON.stringify'd)
 * @param {number} timeout - Request timeout in ms
 * @returns {Promise<object>} Parsed JSON response
 */
function rpcPost(rpcUrl, path, body, timeout = 8000) {
  return new Promise((resolve, reject) => {
    const url = new URL(path, rpcUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;
    const bodyStr = JSON.stringify(body);

    const req = lib.request({
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: 'POST',
      timeout,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch {
          resolve(data);
        }
      });
    });

    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error(`RPC request timeout after ${timeout}ms`));
    });

    req.write(bodyStr);
    req.end();
  });
}

module.exports = {
  rpcGet,
  rpcPost,
  DEFAULT_RPC,
};
