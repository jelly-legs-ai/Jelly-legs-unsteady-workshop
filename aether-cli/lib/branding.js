/**
 * aether-cli lib/branding.js
 * 
 * Centralized ASCII art and branding for consistent CLI theming.
 * All visual elements use the same color scheme and ASCII art style.
 */

// Theme colors - use these consistently across the CLI
const THEME = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
};

// Tier-specific colors
const TIER_COLORS = {
  FULL: '\x1b[36m',     // Cyan
  LITE: '\x1b[33m',     // Yellow  
  OBSERVER: '\x1b[32m', // Green
  reset: '\x1b[0m',
};

/**
 * Main Aether CLI ASCII Art Logo
 */
function getLogo() {
  return `
${THEME.cyan}    █████╗ ████████╗██╗  ██╗███████╗██████╗     ██████╗██╗      ██████╗ ${THEME.reset}
${THEME.cyan}   ██╔══██╗╚══██╔══╝██║  ██║██╔════╝██╔══██╗   ██╔════╝██║     ██╔════╝ ${THEME.reset}
${THEME.cyan}   ███████║   ██║   ███████║█████╗  ██████╔╝   ██║     ██║     ██║      ${THEME.reset}
${THEME.cyan}   ██╔══██║   ██║   ██╔══██║██╔══╝  ██╔══██╗   ██║     ██║     ██║      ${THEME.reset}
${THEME.cyan}   ██║  ██║   ██║   ██║  ██║███████╗██║  ██║██╗╚██████╗███████╗╚██████╗ ${THEME.reset}
${THEME.cyan}   ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝╚══════╝ ╚═════╝ ${THEME.reset}`;
}

/**
 * Compact logo for smaller displays
 */
function getCompactLogo() {
  return `
${THEME.cyan}   ╔══════════════════════════════════════════════╗${THEME.reset}
${THEME.cyan}   ║${THEME.bright}        AETHER VALIDATOR CLI v2.0.0${THEME.reset}${THEME.cyan}           ║${THEME.reset}
${THEME.cyan}   ║${THEME.dim}        Real RPC • Real Blockchain • No Mocks${THEME.reset}${THEME.cyan} ║${THEME.reset}
${THEME.cyan}   ╚══════════════════════════════════════════════╝${THEME.reset}`;
}

/**
 * Get header with version
 */
function getHeader(version = '2.0.0') {
  return `${getLogo()}

${THEME.bright}Aether Validator Command Line Interface${THEME.reset} ${THEME.dim}v${version}${THEME.reset}
${THEME.dim}Official SDK: @jellylegsai/aether-validator-cli${THEME.reset}`;
}

/**
 * Get menu header
 */
function getMenuHeader() {
  return `${getLogo()}

${THEME.bright}              Validator Setup Wizard${THEME.reset} ${THEME.dim}v2.0.0${THEME.reset}
${THEME.dim}         Real RPC • Real Blockchain • No Mocks${THEME.reset}`;
}

/**
 * Get success banner
 */
function getSuccessBanner(message) {
  return `
${THEME.green}${THEME.bright}╔══════════════════════════════════════════════════════════════╗${THEME.reset}
${THEME.green}${THEME.bright}║  ✓ ${message.padEnd(56)}║${THEME.reset}
${THEME.green}${THEME.bright}╚══════════════════════════════════════════════════════════════╝${THEME.reset}`;
}

/**
 * Get error banner
 */
function getErrorBanner(message) {
  return `
${THEME.red}${THEME.bright}╔══════════════════════════════════════════════════════════════╗${THEME.reset}
${THEME.red}${THEME.bright}║  ✗ ${message.padEnd(56)}║${THEME.reset}
${THEME.red}${THEME.bright}╚══════════════════════════════════════════════════════════════╝${THEME.reset}`;
}

/**
 * Get info box
 */
function getInfoBox(title, lines) {
  const maxLen = Math.max(title.length, ...lines.map(l => l.length));
  const width = maxLen + 4;
  
  let result = `\n${THEME.cyan}${THEME.bright}┌${'─'.repeat(width)}┐${THEME.reset}\n`;
  result += `${THEME.cyan}${THEME.bright}│${THEME.reset} ${THEME.bright}${title.padEnd(maxLen)}${THEME.cyan}${THEME.bright} │${THEME.reset}\n`;
  result += `${THEME.cyan}${THEME.bright}├${'─'.repeat(width)}┤${THEME.reset}\n`;
  
  lines.forEach(line => {
    result += `${THEME.cyan}${THEME.bright}│${THEME.reset} ${line.padEnd(maxLen)} ${THEME.cyan}${THEME.bright}│${THEME.reset}\n`;
  });
  
  result += `${THEME.cyan}${THEME.bright}└${'─'.repeat(width)}┘${THEME.reset}\n`;
  return result;
}

/**
 * Get tier badge
 */
function getTierBadge(tier) {
  const tierUpper = tier.toUpperCase();
  const color = TIER_COLORS[tierUpper] || THEME.dim;
  return `${color}[${tierUpper}]${THEME.reset}`;
}

/**
 * Format section header
 */
function formatSection(title) {
  return `\n${THEME.cyan}${THEME.bright}── ${title} ${'─'.repeat(60 - title.length)}${THEME.reset}`;
}

/**
 * Format key-value pair
 */
function formatKeyValue(key, value, valueColor = THEME.bright) {
  return `  ${THEME.dim}${key}:${THEME.reset} ${valueColor}${value}${THEME.reset}`;
}

module.exports = {
  // Theme constants
  THEME,
  TIER_COLORS,
  
  // Branding functions
  getLogo,
  getCompactLogo,
  getHeader,
  getMenuHeader,
  getSuccessBanner,
  getErrorBanner,
  getInfoBox,
  getTierBadge,
  formatSection,
  formatKeyValue,
};
