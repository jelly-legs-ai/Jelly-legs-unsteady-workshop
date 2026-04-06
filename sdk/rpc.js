#!/usr/bin/env node
/**
 * @jellylegsai/aether-sdk - RPC Client
 * 
 * Low-level HTTP RPC client for Aether blockchain with retry logic.
 * All functions make REAL HTTP calls to the blockchain RPC endpoint.
 * No stubs, no mocks.
 * 
 * Features:
 * - Retry logic with exponential backoff
 * - Enhanced error handling for network timeouts
 * - Rate limiting support
 */

const http = require('http');
const https = require('https');

const DEFAULT_RPC = process.env.AETHER_RPC || 'http://127.0.0.1:8899';

// Retry configuration
const DEFAULT_RETRIES = 3;
const DEFAULT_RETRY_DELAY_MS = 1000;
const DEFAULT_BACKOFF_MULTIPLIER = 2;
const DEFAULT_MAX_RETRY_DELAY_MS = 30000;

/**
 * Check if error is retryable
 */
function isRetryableError(error) {
  if (!error) return false;
  
  // Network errors
  if (error.code === 'ECONNREFUSED') return true;
  if (error.code === 'ENOTFOUND') return true;
  if (error.code === 'ETIMEDOUT') return true;
  if (error.code === 'ECONNRESET') return true;
  if (error.code === 'EPIPE') return true;
  
  // Timeout errors
  if (error.message && error.message.includes('timeout')) return true;
  
  // HTTP errors
  if (error.statusCode >= 500) return true;
  if (error.statusCode === 429) return true; // Rate limit
  
  return false;
}

/**
 * Calculate delay for exponential backoff with jitter
 */
function calculateDelay(attempt, baseDelay = DEFAULT_RETRY_DELAY_MS, maxDelay = DEFAULT_MAX_RETRY_DELAY_MS) {
  const delay = baseDelay * Math.pow(DEFAULT_BACKOFF_MULTIPLIER, attempt);
  const jitter = Math.random() * 100; // Add up to 100ms jitter
  return Math.min(delay + jitter, maxDelay);
}

/**
 * Make a GET request to the RPC endpoint with retry logic
 * @param {string} rpcUrl - RPC endpoint URL
 * @param {string} path - API path (e.g., /v1/slot)
 * @param {number} timeout - Request timeout in ms
 * @param {number} retries - Number of retry attempts
 * @returns {Promise<object>} Parsed JSON response
 */
async function rpcGet(rpcUrl, path, timeout = 8000, retries = DEFAULT_RETRIES) {
  let lastError = null;
  
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const result = await _rpcGetInternal(rpcUrl, path, timeout);
      return result;
    } catch (error) {
      lastError = error;
      
      // Don't retry if not retryable or on last attempt
      if (attempt === retries || !isRetryableError(error)) {
        throw error;
      }
      
      // Calculate and wait for backoff
      const delay = calculateDelay(attempt);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError;
}

/**
 * Internal: Make HTTP GET request
 */
function _rpcGetInternal(rpcUrl, path, timeout) {
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
      headers: { 
        'Content-Type': 'application/json',
        'Accept': 'application/json',
        'User-Agent': 'aether-sdk/1.0.0',
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          if (parsed.error) {
            const err = new Error(parsed.error.message || JSON.stringify(parsed.error));
            err.statusCode = res.statusCode;
            err.responseData = parsed;
            reject(err);
          } else {
            resolve(parsed);
          }
        } catch (e) {
          resolve({ raw: data });
        }
      });
    });

    req.on('error', (err) => {
      err.rpcUrl = rpcUrl;
      err.path = path;
      reject(err);
    });
    
    req.on('timeout', () => {
      req.destroy();
      const err = new Error(`RPC request timeout after ${timeout}ms`);
      err.code = 'ETIMEDOUT';
      err.rpcUrl = rpcUrl;
      err.path = path;
      reject(err);
    });
    
    req.end();
  });
}

/**
 * Make a POST request to the RPC endpoint with retry logic
 * @param {string} rpcUrl - RPC endpoint URL
 * @param {string} path - API path
 * @param {object} body - Request body (will be JSON.stringify'd)
 * @param {number} timeout - Request timeout in ms
 * @param {number} retries - Number of retry attempts
 * @returns {Promise<object>} Parsed JSON response
 */
async function rpcPost(rpcUrl, path, body, timeout = 8000, retries = DEFAULT_RETRIES) {
  let lastError = null;
  
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const result = await _rpcPostInternal(rpcUrl, path, body, timeout);
      return result;
    } catch (error) {
      lastError = error;
      
      // Don't retry if not retryable or on last attempt
      if (attempt === retries || !isRetryableError(error)) {
        throw error;
      }
      
      // Calculate and wait for backoff
      const delay = calculateDelay(attempt);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError;
}

/**
 * Internal: Make HTTP POST request
 */
function _rpcPostInternal(rpcUrl, path, body, timeout) {
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
        'Accept': 'application/json',
        'Content-Length': Buffer.byteLength(bodyStr),
        'User-Agent': 'aether-sdk/1.0.0',
      },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          const parsed = JSON.parse(data);
          if (parsed.error) {
            const err = new Error(parsed.error.message || JSON.stringify(parsed.error));
            err.statusCode = res.statusCode;
            err.responseData = parsed;
            reject(err);
          } else {
            resolve(parsed);
          }
        } catch (e) {
          resolve(data);
        }
      });
    });

    req.on('error', (err) => {
      err.rpcUrl = rpcUrl;
      err.path = path;
      reject(err);
    });
    
    req.on('timeout', () => {
      req.destroy();
      const err = new Error(`RPC request timeout after ${timeout}ms`);
      err.code = 'ETIMEDOUT';
      err.rpcUrl = rpcUrl;
      err.path = path;
      reject(err);
    });

    req.write(bodyStr);
    req.end();
  });
}

module.exports = {
  rpcGet,
  rpcPost,
  DEFAULT_RPC,
  DEFAULT_RETRIES,
  DEFAULT_RETRY_DELAY_MS,
  DEFAULT_BACKOFF_MULTIPLIER,
  DEFAULT_MAX_RETRY_DELAY_MS,
  isRetryableError,
  calculateDelay,
};
