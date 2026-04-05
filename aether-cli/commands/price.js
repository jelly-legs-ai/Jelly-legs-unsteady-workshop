#!/usr/bin/env node
/**
 * aether-cli price
 *
 * Show real-time AETH/USD price from free public crypto APIs.
 * Supports CoinGecko (free tier), and falls back to simulated data
 * if no API key is available.
 *
 * Usage:
 *   aether price                    Show current AETH/USD price
 *   aether price --pair AETH/USD   Specify trading pair (default: AETH/USD)
 *   aether price --json            JSON output for scripting
 *   aether price --source coingecko Fallback to CoinGecko (no API key needed)
 */

const https = require('https');
const http = require('http');

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

const AETHER_CONTRACT = 'ATH'; // Aether token contract (hypothetical)
const DEFAULT_PAIR = 'AETH/USD';

function httpGet(url) {
  return new Promise((resolve, reject) => {
    const lib = url.startsWith('https') ? https : http;
    const parsed = new URL(url);
    const req = lib.request({
      hostname: parsed.hostname,
      port: parsed.port || (parsed.protocol === 'https:' ? 443 : 80),
      path: parsed.pathname + parsed.search,
      method: 'GET',
      timeout: 10000,
      headers: { 'Accept': 'application/json', 'User-Agent': 'Aether-CLI/1.0' },
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch { resolve({ _raw: data }); }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => { req.destroy(); reject(new Error('Request timeout')); });
    req.end();
  });
}

function formatPrice(num, decimals = 4) {
  if (num === null || num === undefined || isNaN(num)) return '—';
  return num.toFixed(decimals);
}

function formatTime(date) {
  return date.toISOString().replace('T', ' ').substring(0, 19) + ' UTC';
}

/**
 * Fetch AETH price from CoinGecko (free, no API key).
 * Coins API: /coins/list → find ATH/AETH → /coins/{id}/market_chart
 * Fallback: try known Aether token addresses on major DEXes via /simple/price
 */
async function fetchFromCoinGecko() {
  try {
    // Try CoinGecko simple price for AETH token
    // We'll try a few known Aether token addresses on Ethereum mainnet as a proxy
    const url = 'https://api.coingecko.com/api/v3/simple/price?vs_currencies=usd&ids=aether,ath,ath-token,aether-network';
    const data = await httpGet(url);

    if (data && !data.error) {
      // Find first available AETH quote
      const pairs = [
        { key: 'aether-network', name: 'AETH' },
        { key: 'aether', name: 'AETH' },
        { key: 'ath-token', name: 'ATH' },
        { key: 'ath', name: 'ATH' },
      ];

      for (const { key, name } of pairs) {
        if (data[key] && data[key].usd !== undefined) {
          return {
            source: 'CoinGecko',
            symbol: name,
            price: data[key].usd,
            currency: 'USD',
            timestamp: new Date(),
          };
        }
      }
    }

    return null;
  } catch {
    return null;
  }
}

/**
 * Fetch price data from a public DEX aggregator or mock Aether RPC.
 * Primary: use the Aether chain's own price oracle if available.
 * Fallback: use CoinGecko.
 */
async function fetchAetherPrice() {
  // Try Aether chain's built-in price oracle (if validator is running)
  const rpcUrl = process.env.AETHER_RPC || 'http://127.0.0.1:8899';
  try {
    const res = await httpGet(`${rpcUrl}/v1/price`);
    if (res && !res.error && res.price !== undefined) {
      return {
        source: 'Aether Oracle',
        symbol: 'AETH',
        price: parseFloat(res.price),
        currency: res.currency || 'USD',
        timestamp: new Date(),
        change_24h: res.change_24h || null,
        volume_24h: res.volume_24h || null,
      };
    }
  } catch { /* chain oracle not available */ }

  // Try CoinGecko
  const cg = await fetchFromCoinGecko();
  if (cg) return cg;

  return null;
}

/**
 * Fetch 24h price change using CoinGecko market chart.
 */
async function fetchPriceChange24h(symbol) {
  try {
    const idMap = {
      'AETH': 'aether-network',
      'ATH': 'ath-token',
    };
    const id = idMap[symbol] || 'aether-network';
    const url = `https://api.coingecko.com/api/v3/coins/${id}/market_chart?vs_currency=usd&days=1`;
    const data = await httpGet(url);

    if (data && data.prices && data.prices.length >= 2) {
      const latest = data.prices[data.prices.length - 1][1];
      const yesterday = data.prices[0][1];
      const change = ((latest - yesterday) / yesterday) * 100;
      const volume = data.total_volumes ? data.total_volumes[data.total_volumes.length - 1][1] : null;
      return {
        change_24h: change,
        volume_24h: volume,
      };
    }
  } catch { /* ignore */ }
  return { change_24h: null, volume_24h: null };
}

async function priceCommand() {
  const args = process.argv.slice(2);
  const asJson = args.includes('--json') || args.includes('-j');
  const pair = args.includes('--pair')
    ? args[args.indexOf('--pair') + 1] || DEFAULT_PAIR
    : DEFAULT_PAIR;
  const source = args.includes('--source') ? args[args.indexOf('--source') + 1] : null;

  // Parse pair
  const [fromSymbol, toSymbol = 'USD'] = pair.split('/');
  const symbol = fromSymbol.toUpperCase();

  console.log(`\n${C.bright}${C.cyan}── Aether Price ───────────────────────────────────────${C.reset}\n`);

  let priceData;
  if (source === 'coingecko') {
    priceData = await fetchFromCoinGecko();
  } else {
    priceData = await fetchAetherPrice();
  }

  if (!priceData) {
    if (asJson) {
      console.log(JSON.stringify({ error: 'Price data unavailable', symbol, pair }, null, 2));
    } else {
      console.log(`  ${C.yellow}⚠ Price data temporarily unavailable.${C.reset}`);
      console.log(`  ${C.dim}Make sure your validator is running or check network connectivity.${C.reset}`);
      console.log(`  ${C.dim}Set AETHER_RPC env var to your validator's RPC address.${C.reset}`);
      console.log(`  ${C.dim}Fallback: aether price --source coingecko${C.reset}\n`);
    }
    return;
  }

  // Fetch 24h change if available
  const changeData = await fetchPriceChange24h(symbol);
  const priceInfo = { ...priceData, ...changeData };

  if (asJson) {
    console.log(JSON.stringify({
      symbol: priceInfo.symbol,
      pair: `${symbol}/USD`,
      price_usd: priceInfo.price,
      change_24h_pct: priceInfo.change_24h !== null ? parseFloat(priceInfo.change_24h.toFixed(4)) : null,
      volume_24h_usd: priceInfo.volume_24h,
      source: priceInfo.source,
      timestamp: formatTime(priceInfo.timestamp),
    }, null, 2));
    return;
  }

  // Human-readable output
  const change = priceInfo.change_24h;
  const changeColor = change === null ? C.dim : change >= 0 ? C.green : C.red;
  const changeStr = change !== null
    ? `${change >= 0 ? '+' : ''}${change.toFixed(2)}%`
    : '—';

  const arrow = change === null ? ' ' : change >= 0 ? '▲' : '▼';
  const volumeStr = priceInfo.volume_24h !== null
    ? `$${(priceInfo.volume_24h / 1e6).toFixed(2)}M`
    : null;

  console.log(`  ${C.dim}Pair:${C.reset}       ${C.bright}${symbol}/USD${C.reset}`);
  console.log(`  ${C.dim}Source:${C.reset}      ${C.bright}${priceInfo.source}${C.reset}`);
  console.log(`  ${C.dim}Updated:${C.reset}     ${formatTime(priceInfo.timestamp)}`);
  console.log();
  console.log(`  ${C.bright}${C.green}$${formatPrice(priceInfo.price)}${C.reset} ${C.dim}USD${C.reset}`);
  console.log(`  ${C.dim}24h change:  ${changeColor}${arrow} ${changeStr}${C.reset}`);
  if (volumeStr) {
    console.log(`  ${C.dim}24h volume:  ${volumeStr}${C.reset}`);
  }
  console.log();

  // ASCII box
  const barLen = 40;
  const fillLen = change !== null ? Math.min(barLen, Math.round(Math.abs(change) / 2)) : 0;
  const barColor = change !== null && change < 0 ? C.red : C.green;
  const bar = barColor + '█'.repeat(fillLen) + C.dim + '░'.repeat(barLen - fillLen) + C.reset;
  console.log(`  ${C.dim}[${bar}]${C.reset}`);
  console.log();
  console.log(`  ${C.dim}Run with ${C.cyan}--json${C.reset}${C.dim} for scripted integrations.${C.reset}`);
  console.log(`  ${C.dim}Refreshes on each call — set up a cron job for live monitoring.${C.reset}\n`);
}

priceCommand().catch(err => {
  console.error(`\n${C.red}Price error:${C.reset}`, err.message, '\n');
  process.exit(1);
});

module.exports = { priceCommand };
