#!/usr/bin/env node
/**
 * aether-cli config
 *
 * Centralized configuration management for the Aether CLI.
 * Persistently store settings like RPC URL, default wallet, preferences.
 * All RPC URLs are validated with real HTTP calls before saving.
 *
 * Usage:
 *   aether config get <key>              Get a config value
 *   aether config set <key> <value>      Set a config value (validates RPC URLs)
 *   aether config list                     Show all config
 *   aether config init                   Create default config
 *   aether config reset                  Reset to defaults
 *   aether config validate               Validate current config (test RPC, etc.)
 *   aether config import --file <path>   Import config from JSON file
 *   aether config export --file <path>   Export config to JSON file
 *
 * Config keys:
 *   rpc.url          Default RPC endpoint (validates on set)
 *   rpc.backup       Backup RPC endpoint
 *   wallet.default   Default wallet address
 *   wallet.keypair   Path to keypair file
 *   validator.tier   Default validator tier (full|lite|observer)
 *   output.format    Default output format (text|json)
 *   output.colors    Enable/disable ANSI colors (true|false)
 *   network.timeout  RPC timeout in ms (default: 10000)
 *
 * SDK wired to: GET /v1/health, GET /v1/slot, GET /v1/version
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const readline = require('readline');

// Import SDK for RPC validation
const sdkPath = path.join(__dirname, '..', 'sdk', 'index.js');
const aether = require(sdkPath);

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

const CLI_VERSION = '1.3.0';
const CONFIG_VERSION = 2;

// ---------------------------------------------------------------------------
// Paths & Config Management
// ---------------------------------------------------------------------------

function getAetherDir() {
  return path.join(os.homedir(), '.aether');
}

function getConfigPath() {
  return path.join(getAetherDir(), 'config.json');
}

function ensureAetherDir() {
  const dir = getAetherDir();
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

/**
 * Default configuration values
 */
function getDefaultConfig() {
  return {
    version: CONFIG_VERSION,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    rpc: {
      url: 'http://127.0.0.1:8899',
      backup: null,
      timeout: 10000,
    },
    wallet: {
      default: null,
      keypair: null,
    },
    validator: {
      tier: 'full',
      identity: null,
    },
    output: {
      format: 'text',
      colors: true,
    },
    network: {
      explorer: 'https://explorer.aether.network',
      faucet: null,
    },
  };
}

/**
 * Load config from disk
 */
function loadConfig() {
  const configPath = getConfigPath();
  if (!fs.existsSync(configPath)) {
    return null;
  }
  try {
    const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
    return migrateConfig(config);
  } catch (err) {
    return null;
  }
}

/**
 * Save config to disk
 */
function saveConfig(config) {
  ensureAetherDir();
  config.updated_at = new Date().toISOString();
  fs.writeFileSync(getConfigPath(), JSON.stringify(config, null, 2));
  return config;
}

/**
 * Migrate old config versions
 */
function migrateConfig(config) {
  if (!config.version || config.version < CONFIG_VERSION) {
    // Merge with defaults to add any missing keys
    const defaults = getDefaultConfig();
    const migrated = {
      ...defaults,
      ...config,
      version: CONFIG_VERSION,
      rpc: { ...defaults.rpc, ...config.rpc },
      wallet: { ...defaults.wallet, ...config.wallet },
      validator: { ...defaults.validator, ...config.validator },
      output: { ...defaults.output, ...config.output },
      network: { ...defaults.network, ...config.network },
    };
    return migrated;
  }
  return config;
}

// ---------------------------------------------------------------------------
// Validation Functions (with REAL RPC calls)
// ---------------------------------------------------------------------------

/**
 * Validate an RPC URL with real HTTP call
 */
async function validateRpcUrl(url) {
  const errors = [];

  // Basic URL validation
  if (!url || typeof url !== 'string') {
    return { valid: false, error: 'URL is required' };
  }

  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    return { valid: false, error: 'Invalid URL format' };
  }

  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    return { valid: false, error: 'URL must be http:// or https://' };
  }

  // Real RPC validation via SDK
  try {
    const client = new aether.AetherClient({ rpcUrl: url, timeoutMs: 5000 });
    const start = Date.now();
    const [health, slot] = await Promise.all([
      client.getHealth().catch(() => null),
      client.getSlot().catch(() => null),
    ]);
    const latency = Date.now() - start;

    if (slot === null && health === null) {
      return {
        valid: false,
        error: 'RPC endpoint did not respond to health/slot checks',
        url,
      };
    }

    return {
      valid: true,
      url,
      latency,
      health: health || 'unknown',
      slot,
    };
  } catch (err) {
    return {
      valid: false,
      error: `RPC validation failed: ${err.message}`,
      url,
    };
  }
}

/**
 * Validate wallet address format
 */
function validateAddress(addr) {
  if (!addr || typeof addr !== 'string') {
    return { valid: false, error: 'Address is required' };
  }
  if (!addr.startsWith('ATH')) {
    return { valid: false, error: 'Address must start with ATH' };
  }
  if (addr.length < 36) {
    return { valid: false, error: 'Address too short' };
  }
  return { valid: true, address: addr };
}

/**
 * Validate tier value
 */
function validateTier(tier) {
  const validTiers = ['full', 'lite', 'observer'];
  if (!tier || typeof tier !== 'string') {
    return { valid: false, error: 'Tier is required' };
  }
  if (!validTiers.includes(tier.toLowerCase())) {
    return { valid: false, error: `Tier must be one of: ${validTiers.join(', ')}` };
  }
  return { valid: true, tier: tier.toLowerCase() };
}

/**
 * Validate output format
 */
function validateFormat(format) {
  const validFormats = ['text', 'json'];
  if (!format || typeof format !== 'string') {
    return { valid: false, error: 'Format is required' };
  }
  if (!validFormats.includes(format.toLowerCase())) {
    return { valid: false, error: `Format must be one of: ${validFormats.join(', ')}` };
  }
  return { valid: true, format: format.toLowerCase() };
}

/**
 * Validate boolean string
 */
function validateBoolean(value) {
  const truthy = ['true', 'yes', '1', 'on'];
  const falsy = ['false', 'no', '0', 'off'];
  const lower = String(value).toLowerCase();
  if (truthy.includes(lower)) return { valid: true, value: true };
  if (falsy.includes(lower)) return { valid: true, value: false };
  return { valid: false, error: 'Must be true/false, yes/no, 1/0, or on/off' };
}

/**
 * Validate timeout
 */
function validateTimeout(value) {
  const num = parseInt(value, 10);
  if (isNaN(num) || num < 1000) {
    return { valid: false, error: 'Timeout must be at least 1000ms' };
  }
  if (num > 60000) {
    return { valid: false, error: 'Timeout cannot exceed 60000ms' };
  }
  return { valid: true, timeout: num };
}

// ---------------------------------------------------------------------------
// Config Value Getters/Setters
// ---------------------------------------------------------------------------

const CONFIG_SCHEMA = {
  'rpc.url': {
    category: 'rpc',
    key: 'url',
    validate: validateRpcUrl,
    description: 'Default RPC endpoint URL',
  },
  'rpc.backup': {
    category: 'rpc',
    key: 'backup',
    validate: validateRpcUrl,
    description: 'Backup RPC endpoint URL',
  },
  'rpc.timeout': {
    category: 'rpc',
    key: 'timeout',
    validate: validateTimeout,
    description: 'RPC timeout in milliseconds',
  },
  'wallet.default': {
    category: 'wallet',
    key: 'default',
    validate: validateAddress,
    description: 'Default wallet address',
  },
  'wallet.keypair': {
    category: 'wallet',
    key: 'keypair',
    validate: (v) => ({ valid: fs.existsSync(v), path: v, error: fs.existsSync(v) ? null : 'File does not exist' }),
    description: 'Path to keypair file',
  },
  'validator.tier': {
    category: 'validator',
    key: 'tier',
    validate: validateTier,
    description: 'Default validator tier (full|lite|observer)',
  },
  'validator.identity': {
    category: 'validator',
    key: 'identity',
    validate: (v) => ({ valid: true, path: v }),
    description: 'Path to validator identity file',
  },
  'output.format': {
    category: 'output',
    key: 'format',
    validate: validateFormat,
    description: 'Default output format (text|json)',
  },
  'output.colors': {
    category: 'output',
    key: 'colors',
    validate: validateBoolean,
    description: 'Enable ANSI colors (true|false)',
  },
  'network.explorer': {
    category: 'network',
    key: 'explorer',
    validate: (v) => ({ valid: true, url: v }),
    description: 'Block explorer URL',
  },
  'network.faucet': {
    category: 'network',
    key: 'faucet',
    validate: (v) => ({ valid: true, url: v }),
    description: 'Testnet faucet URL',
  },
};

function getConfigValue(config, key) {
  const schema = CONFIG_SCHEMA[key];
  if (!schema) return { exists: false };

  const value = config[schema.category]?.[schema.key];
  return { exists: true, value, schema };
}

async function setConfigValue(config, key, value) {
  const schema = CONFIG_SCHEMA[key];
  if (!schema) {
    return { success: false, error: `Unknown config key: ${key}` };
  }

  // Validate the value
  const validation = await schema.validate(value);
  if (!validation.valid) {
    return { success: false, error: validation.error };
  }

  // Set the value
  config[schema.category][schema.key] = validation[schema.key] ?? validation.value ?? validation.url ?? validation.path ?? value;
  return { success: true, value: config[schema.category][schema.key] };
}

// ---------------------------------------------------------------------------
// Command Implementations
// ---------------------------------------------------------------------------

async function configGet(args) {
  const key = args[0];
  const config = loadConfig() || getDefaultConfig();

  if (!key) {
    console.log(`\n  ${C.red}✗ Config key required${C.reset}`);
    console.log(`  ${C.dim}Usage: aether config get <key>${C.reset}`);
    console.log(`  ${C.dim}Run 'aether config list' to see all keys${C.reset}\n`);
    return;
  }

  const result = getConfigValue(config, key);
  if (!result.exists) {
    console.log(`\n  ${C.red}✗ Unknown config key:${C.reset} ${key}`);
    console.log(`  ${C.dim}Run 'aether config list' to see available keys${C.reset}\n`);
    return;
  }

  console.log(`\n  ${C.bright}${C.cyan}── Config Value ──${C.reset}\n`);
  console.log(`  ${C.cyan}Key:${C.reset}         ${key}`);
  console.log(`  ${C.cyan}Description:${C.reset} ${result.schema.description}`);
  console.log(`  ${C.cyan}Value:${C.reset}       ${result.value !== null ? C.bright + result.value + C.reset : C.dim + '(not set)' + C.reset}`);
  console.log();
}

async function configSet(args) {
  const key = args[0];
  const value = args[1];

  if (!key || value === undefined) {
    console.log(`\n  ${C.red}✗ Key and value required${C.reset}`);
    console.log(`  ${C.dim}Usage: aether config set <key> <value>${C.reset}\n`);
    return;
  }

  let config = loadConfig();
  if (!config) {
    config = getDefaultConfig();
    console.log(`  ${C.yellow}⚠ No existing config. Creating new config file...${C.reset}`);
  }

  const schema = CONFIG_SCHEMA[key];
  if (!schema) {
    console.log(`\n  ${C.red}✗ Unknown config key:${C.reset} ${key}`);
    console.log(`  ${C.dim}Run 'aether config list' to see available keys${C.reset}\n`);
    return;
  }

  // Show validation message for RPC URLs
  if (key.includes('rpc.')) {
    console.log(`\n  ${C.dim}Validating RPC endpoint...${C.reset}`);
  }

  const result = await setConfigValue(config, key, value);

  if (!result.success) {
    console.log(`\n  ${C.red}✗ Validation failed:${C.reset} ${result.error}\n`);
    return;
  }

  saveConfig(config);

  console.log(`\n  ${C.green}✓ Config updated${C.reset}\n`);
  console.log(`  ${C.cyan}Key:${C.reset}   ${key}`);
  console.log(`  ${C.cyan}Value:${C.reset} ${C.bright}${result.value}${C.reset}`);

  // Show extra info for RPC
  if (key === 'rpc.url' && result.latency) {
    const latencyColor = result.latency < 50 ? C.green : result.latency < 200 ? C.cyan : C.yellow;
    console.log(`  ${C.cyan}Health:${C.reset} ${C.green}✓${C.reset} Online (${latencyColor}${result.latency}ms${C.reset})`);
    if (result.slot) {
      console.log(`  ${C.cyan}Slot:${C.reset}   ${result.slot.toLocaleString()}`);
    }
  }

  console.log();
}

async function configList(args, asJson = false) {
  const config = loadConfig() || getDefaultConfig();
  const isCompact = args.includes('--compact');

  if (asJson) {
    console.log(JSON.stringify(config, null, 2));
    return;
  }

  console.log(`\n  ${C.bright}${C.cyan}── Aether CLI Configuration ──${C.reset}\n`);
  console.log(`  ${C.dim}Config file: ${getConfigPath()}${C.reset}`);
  console.log(`  ${C.dim}Version: ${config.version}${C.reset}\n`);

  // Group by category
  const categories = {};
  Object.entries(CONFIG_SCHEMA).forEach(([key, schema]) => {
    if (!categories[schema.category]) {
      categories[schema.category] = [];
    }
    const value = config[schema.category]?.[schema.key];
    categories[schema.category].push({ key, value, schema });
  });

  for (const [category, items] of Object.entries(categories)) {
    console.log(`  ${C.bright}${C.cyan}[${category.toUpperCase()}]${C.reset}`);
    for (const item of items) {
      const displayValue = item.value !== null
        ? C.bright + String(item.value) + C.reset
        : C.dim + '(not set)' + C.reset;
      if (isCompact) {
        console.log(`    ${C.dim}${item.key}:${C.reset} ${displayValue}`);
      } else {
        console.log(`    ${C.cyan}${item.key}${C.reset}`);
        console.log(`      ${C.dim}${item.schema.description}${C.reset}`);
        console.log(`      ${C.dim}Value:${C.reset} ${displayValue}`);
      }
    }
    console.log();
  }

  console.log(`  ${C.dim}Tip: Use ${C.cyan}aether config set <key> <value>${C.reset}${C.dim} to change settings${C.reset}`);
  console.log(`  ${C.dim}      Run ${C.cyan}aether config validate${C.reset}${C.dim} to test your configuration${C.reset}\n`);
}

async function configInit(args) {
  const force = args.includes('--force');
  const configPath = getConfigPath();

  if (fs.existsSync(configPath) && !force) {
    console.log(`\n  ${C.yellow}⚠ Config file already exists${C.reset}`);
    console.log(`  ${C.dim}Path: ${configPath}${C.reset}`);
    console.log(`  ${C.dim}Use --force to overwrite${C.reset}\n`);
    return;
  }

  const config = getDefaultConfig();
  saveConfig(config);

  console.log(`\n  ${C.green}✓ Configuration initialized${C.reset}\n`);
  console.log(`  ${C.dim}Path: ${configPath}${C.reset}`);
  console.log(`  ${C.dim}Edit with:${C.reset} ${C.cyan}aether config set <key> <value>${C.reset}`);
  console.log(`  ${C.dim}View with:${C.reset} ${C.cyan}aether config list${C.reset}\n`);
}

async function configReset(args) {
  const confirmed = args.includes('--yes');

  if (!confirmed) {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    const answer = await new Promise(resolve => {
      rl.question(`\n  ${C.yellow}⚠ This will reset all configuration to defaults.${C.reset}\n  Continue? [y/N] `, resolve);
    });
    rl.close();
    if (answer.toLowerCase() !== 'y' && answer.toLowerCase() !== 'yes') {
      console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
      return;
    }
  }

  const config = getDefaultConfig();
  saveConfig(config);

  console.log(`\n  ${C.green}✓ Configuration reset to defaults${C.reset}\n`);
  console.log(`  ${C.dim}Config file: ${getConfigPath()}${C.reset}\n`);
}

async function configValidate(args, asJson = false) {
  const config = loadConfig();

  if (!config) {
    const error = { error: 'No configuration file found. Run: aether config init' };
    if (asJson) {
      console.log(JSON.stringify(error, null, 2));
    } else {
      console.log(`\n  ${C.red}✗ No configuration found${C.reset}`);
      console.log(`  ${C.dim}Run: aether config init${C.reset}\n`);
    }
    return;
  }

  const results = {
    config_valid: true,
    checks: [],
    timestamp: new Date().toISOString(),
  };

  if (!asJson) {
    console.log(`\n  ${C.bright}${C.cyan}── Configuration Validation ──${C.reset}\n`);
    console.log(`  ${C.dim}Testing RPC endpoints and settings...${C.reset}\n`);
  }

  // Test primary RPC
  if (config.rpc?.url) {
    if (!asJson) console.log(`  ${C.dim}Testing primary RPC: ${config.rpc.url}${C.reset}`);
    const rpcTest = await validateRpcUrl(config.rpc.url);
    results.checks.push({
      name: 'Primary RPC',
      key: 'rpc.url',
      passed: rpcTest.valid,
      details: rpcTest.valid
        ? { latency: rpcTest.latency, slot: rpcTest.slot, health: rpcTest.health }
        : { error: rpcTest.error },
    });
    if (!asJson) {
      const icon = rpcTest.valid ? `${C.green}✓` : `${C.red}✗`;
      const status = rpcTest.valid ? `${C.green}Online` : `${C.red}Failed`;
      console.log(`    ${icon} Primary RPC: ${status}${C.reset}`);
      if (rpcTest.valid) {
        const latencyColor = rpcTest.latency < 50 ? C.green : rpcTest.latency < 200 ? C.cyan : C.yellow;
        console.log(`      ${C.dim}Latency: ${latencyColor}${rpcTest.latency}ms${C.reset}`);
        console.log(`      ${C.dim}Slot: ${rpcTest.slot?.toLocaleString()}${C.reset}`);
      } else {
        console.log(`      ${C.red}Error: ${rpcTest.error}${C.reset}`);
        results.config_valid = false;
      }
    } else if (!rpcTest.valid) {
      results.config_valid = false;
    }
  }

  // Test backup RPC if set
  if (config.rpc?.backup) {
    if (!asJson) console.log(`\n  ${C.dim}Testing backup RPC: ${config.rpc.backup}${C.reset}`);
    const backupTest = await validateRpcUrl(config.rpc.backup);
    results.checks.push({
      name: 'Backup RPC',
      key: 'rpc.backup',
      passed: backupTest.valid,
      details: backupTest.valid
        ? { latency: backupTest.latency, slot: backupTest.slot }
        : { error: backupTest.error },
    });
    if (!asJson) {
      const icon = backupTest.valid ? `${C.green}✓` : `${C.yellow}⚠`;
      const status = backupTest.valid ? `${C.green}Online` : `${C.yellow}Unreachable`;
      console.log(`    ${icon} Backup RPC: ${status}${C.reset}`);
      if (backupTest.valid) {
        const latencyColor = backupTest.latency < 50 ? C.green : backupTest.latency < 200 ? C.cyan : C.yellow;
        console.log(`      ${C.dim}Latency: ${latencyColor}${backupTest.latency}ms${C.reset}`);
      }
    }
  }

  // Validate default wallet
  if (config.wallet?.default) {
    const addrTest = validateAddress(config.wallet.default);
    results.checks.push({
      name: 'Default Wallet',
      key: 'wallet.default',
      passed: addrTest.valid,
      details: addrTest.valid ? { address: addrTest.address } : { error: addrTest.error },
    });
    if (!asJson) {
      const icon = addrTest.valid ? `${C.green}✓` : `${C.yellow}⚠`;
      const status = addrTest.valid ? `${C.green}Valid` : `${C.yellow}Invalid format`;
      console.log(`\n    ${icon} Default wallet: ${status}${C.reset}`);
      if (!addrTest.valid) {
        console.log(`      ${C.red}Error: ${addrTest.error}${C.reset}`);
      } else {
        console.log(`      ${C.dim}Address: ${config.wallet.default}${C.reset}`);
      }
    }
  }

  // Validate tier
  if (config.validator?.tier) {
    const tierTest = validateTier(config.validator.tier);
    results.checks.push({
      name: 'Validator Tier',
      key: 'validator.tier',
      passed: tierTest.valid,
      details: tierTest.valid ? { tier: tierTest.tier } : { error: tierTest.error },
    });
    if (!asJson) {
      const icon = tierTest.valid ? `${C.green}✓` : `${C.red}✗`;
      console.log(`\n    ${icon} Validator tier: ${tierTest.valid ? C.green + tierTest.tier : C.red + 'Invalid'}${C.reset}`);
    }
  }

  // Summary
  const passed = results.checks.filter(c => c.passed).length;
  const total = results.checks.length;

  if (asJson) {
    results.summary = { passed, total, success_rate: Math.round((passed / total) * 100) };
    console.log(JSON.stringify(results, null, 2));
  } else {
    console.log(`\n  ${C.bright}Validation Summary:${C.reset} ${passed}/${total} checks passed`);
    if (passed === total) {
      console.log(`\n  ${C.green}✓ Configuration is valid and RPC is reachable${C.reset}\n`);
    } else if (results.config_valid) {
      console.log(`\n  ${C.yellow}⚠ Some optional checks failed, but config is usable${C.reset}\n`);
    } else {
      console.log(`\n  ${C.red}✗ Configuration has errors that need to be fixed${C.reset}`);
      console.log(`  ${C.dim}Run: aether config set rpc.url <working-url>${C.reset}\n`);
    }
  }
}

async function configExport(args) {
  const fileIdx = args.findIndex(a => a === '--file' || a === '-f');
  const filePath = fileIdx !== -1 && args[fileIdx + 1] ? args[fileIdx + 1] : null;

  if (!filePath) {
    console.log(`\n  ${C.red}✗ --file required${C.reset}`);
    console.log(`  ${C.dim}Usage: aether config export --file <path>${C.reset}\n`);
    return;
  }

  const config = loadConfig() || getDefaultConfig();
  const exportData = {
    ...config,
    exported_at: new Date().toISOString(),
    cli_version: CLI_VERSION,
  };

  try {
    fs.writeFileSync(filePath, JSON.stringify(exportData, null, 2));
    console.log(`\n  ${C.green}✓ Configuration exported${C.reset}\n`);
    console.log(`  ${C.dim}File: ${filePath}${C.reset}\n`);
  } catch (err) {
    console.log(`\n  ${C.red}✗ Export failed:${C.reset} ${err.message}\n`);
  }
}

async function configImport(args) {
  const fileIdx = args.findIndex(a => a === '--file' || a === '-f');
  const filePath = fileIdx !== -1 && args[fileIdx + 1] ? args[fileIdx + 1] : null;
  const force = args.includes('--force');

  if (!filePath) {
    console.log(`\n  ${C.red}✗ --file required${C.reset}`);
    console.log(`  ${C.dim}Usage: aether config import --file <path>${C.reset}\n`);
    return;
  }

  if (!fs.existsSync(filePath)) {
    console.log(`\n  ${C.red}✗ File not found:${C.reset} ${filePath}\n`);
    return;
  }

  try {
    const imported = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    const configPath = getConfigPath();

    if (fs.existsSync(configPath) && !force) {
      console.log(`\n  ${C.yellow}⚠ Existing config will be overwritten${C.reset}`);
      const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
      const answer = await new Promise(resolve => {
        rl.question(`  Continue? [y/N] `, resolve);
      });
      rl.close();
      if (answer.toLowerCase() !== 'y' && answer.toLowerCase() !== 'yes') {
        console.log(`\n  ${C.dim}Cancelled.${C.reset}\n`);
        return;
      }
    }

    const migrated = migrateConfig(imported);
    saveConfig(migrated);

    console.log(`\n  ${C.green}✓ Configuration imported${C.reset}\n`);
    console.log(`  ${C.dim}From: ${filePath}${C.reset}`);
    console.log(`  ${C.dim}To:   ${configPath}${C.reset}\n`);
  } catch (err) {
    console.log(`\n  ${C.red}✗ Import failed:${C.reset} ${err.message}\n`);
  }
}

// ---------------------------------------------------------------------------
// CLI Parser & Dispatcher
// ---------------------------------------------------------------------------

function parseArgs() {
  // argv = [node, index.js, config, <subcmd>, ...]
  return process.argv.slice(3);
}

function showHelp() {
  console.log(`
  ${C.bright}${C.cyan}aether-cli config${C.reset} — Configuration Management

  ${C.bright}Usage:${C.reset}
    aether config get <key>              Get a config value
    aether config set <key> <value>      Set a config value (validates RPC URLs)
    aether config list [--compact]       Show all configuration
    aether config init [--force]         Create default config
    aether config reset --yes            Reset to defaults
    aether config validate [--json]      Validate config with real RPC calls
    aether config export --file <path>   Export config to JSON
    aether config import --file <path>   Import config from JSON

  ${C.bright}Config Keys:${C.reset}
    rpc.url          Default RPC endpoint (validates on set)
    rpc.backup       Backup RPC endpoint
    rpc.timeout      RPC timeout in milliseconds
    wallet.default   Default wallet address
    wallet.keypair   Path to keypair file
    validator.tier   Default tier (full|lite|observer)
    validator.identity  Path to validator identity
    output.format    Output format (text|json)
    output.colors    Enable colors (true|false)
    network.explorer Block explorer URL
    network.faucet   Testnet faucet URL

  ${C.bright}Examples:${C.reset}
    aether config set rpc.url http://localhost:8899
    aether config set wallet.default ATHabc...
    aether config set validator.tier full
    aether config validate
    aether config export --file ~/aether-config-backup.json

  ${C.dim}SDK Validation:${C.reset}
    Setting rpc.url performs a real HTTP health check via GET /v1/health
`);
}

async function configCommand() {
  const args = parseArgs();
  const subcmd = args[0];
  const subargs = args.slice(1);
  const asJson = args.includes('--json') || args.includes('-j');

  switch (subcmd) {
    case 'get':
      await configGet(subargs);
      break;
    case 'set':
      await configSet(subargs);
      break;
    case 'list':
      await configList(subargs, asJson);
      break;
    case 'init':
      await configInit(subargs);
      break;
    case 'reset':
      await configReset(subargs);
      break;
    case 'validate':
      await configValidate(subargs, asJson);
      break;
    case 'export':
      await configExport(subargs);
      break;
    case 'import':
      await configImport(subargs);
      break;
    case '--help':
    case '-h':
    case 'help':
    default:
      showHelp();
      break;
  }
}

// Export for module use
module.exports = { configCommand };

// Run if called directly
if (require.main === module) {
  configCommand().catch(err => {
    console.error(`\n${C.red}✗ Config command failed:${C.reset}`, err.message, '\n');
    process.exit(1);
  });
}
