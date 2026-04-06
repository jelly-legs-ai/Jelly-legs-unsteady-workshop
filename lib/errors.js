/**
 * aether-cli lib/errors.js
 * 
 * Centralized error handling for production-grade CLI experience.
 * Provides consistent error formatting, retry logic, and user-friendly messages.
 */

// ANSI colours
const C = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

/**
 * Error categories for user-friendly messaging
 */
const ERROR_CATEGORIES = {
  NETWORK: 'network',
  RPC: 'rpc',
  VALIDATION: 'validation',
  CONFIG: 'config',
  WALLET: 'wallet',
  PERMISSION: 'permission',
  TIMEOUT: 'timeout',
  UNKNOWN: 'unknown',
};

/**
 * User-friendly error messages by category
 */
const ERROR_MESSAGES = {
  [ERROR_CATEGORIES.NETWORK]: {
    title: 'Network Error',
    description: 'Unable to reach the Aether network.',
    suggestions: [
      'Check your internet connection',
      'Verify the RPC endpoint is accessible',
      'Try using --rpc to specify a different endpoint',
    ],
  },
  [ERROR_CATEGORIES.RPC]: {
    title: 'RPC Error',
    description: 'The Aether node returned an error.',
    suggestions: [
      'The node may be syncing - try again in a few moments',
      'Check if the RPC endpoint is correct',
      'Try a different RPC endpoint with --rpc <url>',
    ],
  },
  [ERROR_CATEGORIES.VALIDATION]: {
    title: 'Validation Error',
    description: 'Invalid input provided.',
    suggestions: [
      'Check the command syntax with --help',
      'Verify addresses are valid base58 format',
      'Ensure amounts are positive numbers',
    ],
  },
  [ERROR_CATEGORIES.CONFIG]: {
    title: 'Configuration Error',
    description: 'CLI configuration issue detected.',
    suggestions: [
      'Run "aether config init" to create default config',
      'Check ~/.aether/config.json exists and is valid JSON',
      'Use "aether config validate" to check settings',
    ],
  },
  [ERROR_CATEGORIES.WALLET]: {
    title: 'Wallet Error',
    description: 'Unable to access wallet.',
    suggestions: [
      'Ensure you have created a wallet: aether wallet create',
      'Check the wallet address is correct',
      'Verify wallet files exist in ~/.aether/wallets/',
    ],
  },
  [ERROR_CATEGORIES.PERMISSION]: {
    title: 'Permission Error',
    description: 'Insufficient permissions.',
    suggestions: [
      'Check file permissions in ~/.aether/',
      'Ensure you own the wallet files',
      'Try running with appropriate privileges',
    ],
  },
  [ERROR_CATEGORIES.TIMEOUT]: {
    title: 'Request Timeout',
    description: 'The request took too long to complete.',
    suggestions: [
      'The network may be congested - try again',
      'Increase timeout with AETHER_RPC_TIMEOUT env var',
      'Check if the RPC endpoint is responsive',
    ],
  },
  [ERROR_CATEGORIES.UNKNOWN]: {
    title: 'Unexpected Error',
    description: 'An unexpected error occurred.',
    suggestions: [
      'Try the command again',
      'Check logs with --verbose flag',
      'Report this issue with the error details',
    ],
  },
};

/**
 * Categorize an error based on its properties
 */
function categorizeError(error) {
  if (!error) return ERROR_CATEGORIES.UNKNOWN;
  
  const message = (error.message || '').toLowerCase();
  const code = error.code || '';
  
  // Network errors
  if (message.includes('enetunreach') || 
      message.includes('econnrefused') ||
      message.includes('econnreset') ||
      message.includes('socket') ||
      message.includes('network') ||
      code === 'ECONNREFUSED' ||
      code === 'ENOTFOUND' ||
      code === 'ENETUNREACH') {
    return ERROR_CATEGORIES.NETWORK;
  }
  
  // RPC errors
  if (message.includes('rpc') ||
      message.includes('json-rpc') ||
      message.includes('-32000') ||
      message.includes('method not found') ||
      message.includes('invalid params') ||
      error.statusCode >= 500) {
    return ERROR_CATEGORIES.RPC;
  }
  
  // Validation errors
  if (message.includes('invalid') ||
      message.includes('required') ||
      message.includes('must be') ||
      message.includes('cannot be') ||
      message.includes('bad request') ||
      error.statusCode === 400) {
    return ERROR_CATEGORIES.VALIDATION;
  }
  
  // Config errors
  if (message.includes('config') ||
      message.includes('enoent') && message.includes('config')) {
    return ERROR_CATEGORIES.CONFIG;
  }
  
  // Wallet errors
  if (message.includes('wallet') ||
      message.includes('keypair') ||
      message.includes('mnemonic') ||
      message.includes('signature')) {
    return ERROR_CATEGORIES.WALLET;
  }
  
  // Permission errors
  if (message.includes('eperm') ||
      message.includes('eacces') ||
      code === 'EACCES' ||
      code === 'EPERM') {
    return ERROR_CATEGORIES.PERMISSION;
  }
  
  // Timeout errors
  if (message.includes('timeout') ||
      message.includes('etimedout') ||
      code === 'ETIMEDOUT') {
    return ERROR_CATEGORIES.TIMEOUT;
  }
  
  return ERROR_CATEGORIES.UNKNOWN;
}

/**
 * Format error for display
 */
function formatError(error, options = {}) {
  const { verbose = false, exit = true } = options;
  const category = categorizeError(error);
  const template = ERROR_MESSAGES[category];
  
  let output = '\n';
  output += `${C.red}${C.bright}✖ ${template.title}${C.reset}\n`;
  output += `${C.dim}${template.description}${C.reset}\n\n`;
  
  if (error.message) {
    output += `${C.yellow}Error: ${error.message}${C.reset}\n`;
  }
  
  if (verbose && error.stack) {
    output += `\n${C.dim}Stack trace:${C.reset}\n`;
    output += `${C.dim}${error.stack}${C.reset}\n`;
  }
  
  output += `\n${C.bright}Suggestions:${C.reset}\n`;
  template.suggestions.forEach(suggestion => {
    output += `  ${C.cyan}•${C.reset} ${suggestion}\n`;
  });
  
  return output;
}

/**
 * Display error and optionally exit
 */
function displayError(error, options = {}) {
  const { exit = true, exitCode = 1, verbose = process.env.AETHER_VERBOSE === '1' } = options;
  
  console.error(formatError(error, { verbose }));
  
  if (exit) {
    process.exit(exitCode);
  }
}

/**
 * Wrap an async function with error handling
 */
function withErrorHandling(fn, options = {}) {
  return async function(...args) {
    try {
      return await fn(...args);
    } catch (error) {
      displayError(error, options);
      return null;
    }
  };
}

/**
 * Retry an async operation with exponential backoff
 */
async function withRetry(operation, options = {}) {
  const {
    maxRetries = 3,
    initialDelay = 1000,
    maxDelay = 30000,
    backoffMultiplier = 2,
    retryableErrors = null, // null = retry all
    onRetry = null,
  } = options;
  
  let lastError;
  let delay = initialDelay;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error;
      
      // Check if we should retry this error
      if (retryableErrors && !retryableErrors.some(e => error instanceof e)) {
        throw error;
      }
      
      // Don't retry on validation errors
      const category = categorizeError(error);
      if (category === ERROR_CATEGORIES.VALIDATION) {
        throw error;
      }
      
      if (attempt < maxRetries) {
        if (onRetry) {
          onRetry(error, attempt + 1, maxRetries);
        }
        
        await sleep(delay);
        delay = Math.min(delay * backoffMultiplier, maxDelay);
      }
    }
  }
  
  throw lastError;
}

/**
 * Sleep utility
 */
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Create a RPC call wrapper with retry logic
 */
function createRpcCaller(clientMethod, options = {}) {
  return async function(...args) {
    return withRetry(
      () => clientMethod(...args),
      {
        maxRetries: 3,
        initialDelay: 1000,
        onRetry: (error, attempt, max) => {
          console.log(
            `${C.yellow}⚠ RPC call failed (attempt ${attempt}/${max}): ${error.message}${C.reset}`
          );
        },
        ...options,
      }
    );
  };
}

/**
 * Validate required arguments
 */
function validateRequired(args, requirements) {
  const missing = [];
  
  for (const requirement of requirements) {
    if (requirement.validator ? !requirement.validator(args) : !args[requirement.name]) {
      missing.push(requirement.displayName || requirement.name);
    }
  }
  
  if (missing.length > 0) {
    const error = new Error(`Missing required arguments: ${missing.join(', ')}`);
    error.isValidationError = true;
    throw error;
  }
}

/**
 * Validate address format (base58)
 */
function validateAddress(address, fieldName = 'address') {
  if (!address) {
    throw new Error(`${fieldName} is required`);
  }
  
  // Base58 regex (simplified)
  const base58Regex = /^[1-9A-HJ-NP-Za-km-z]+$/;
  if (!base58Regex.test(address)) {
    throw new Error(`Invalid ${fieldName} format: must be base58 encoded`);
  }
  
  // Typical Solana/Aether addresses are 32-44 chars
  if (address.length < 32 || address.length > 44) {
    throw new Error(`Invalid ${fieldName} length: expected 32-44 characters, got ${address.length}`);
  }
  
  return true;
}

/**
 * Validate amount (positive number)
 */
function validateAmount(amount, fieldName = 'amount') {
  if (amount === undefined || amount === null || amount === '') {
    throw new Error(`${fieldName} is required`);
  }
  
  const num = Number(amount);
  if (isNaN(num) || !isFinite(num)) {
    throw new Error(`Invalid ${fieldName}: must be a valid number`);
  }
  
  if (num <= 0) {
    throw new Error(`Invalid ${fieldName}: must be greater than 0`);
  }
  
  return num;
}

module.exports = {
  // Constants
  ERROR_CATEGORIES,
  ERROR_MESSAGES,
  
  // Core functions
  categorizeError,
  formatError,
  displayError,
  withErrorHandling,
  withRetry,
  createRpcCaller,
  
  // Validation
  validateRequired,
  validateAddress,
  validateAmount,
  
  // Utilities
  sleep,
  C,
};
