/**
 * aether-cli/lib/ui.js
 * 
 * Unified UI/UX Framework for Aether CLI
 * 
 * Provides:
 * - Consistent ASCII art branding
 * - Standardized color palette
 * - Status indicators and spinners
 * - Progress feedback helpers
 * - Box drawing utilities
 * - Table formatting
 */

// ============================================================================
// ANSI Color Palette - Consistent across all commands
// ============================================================================

const colors = {
  // Standard colors
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  dim: '\x1b[2m',
  underscore: '\x1b[4m',
  
  // Foreground colors
  black: '\x1b[30m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  
  // Bright variants
  brightRed: '\x1b[91m',
  brightGreen: '\x1b[92m',
  brightYellow: '\x1b[93m',
  brightBlue: '\x1b[94m',
  brightMagenta: '\x1b[95m',
  brightCyan: '\x1b[96m',
  brightWhite: '\x1b[97m',
  
  // Background colors
  bgBlack: '\x1b[40m',
  bgRed: '\x1b[41m',
  bgGreen: '\x1b[42m',
  bgYellow: '\x1b[43m',
  bgBlue: '\x1b[44m',
  bgMagenta: '\x1b[45m',
  bgCyan: '\x1b[46m',
  bgWhite: '\x1b[47m',
};

// Short aliases for convenience
const C = colors;

// ============================================================================
// ASCII Art Branding
// ============================================================================

const BRANDING = {
  // Main Aether logo - cosmic blockchain aesthetic
  logo: `
${C.cyan}                    ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄                     ${C.reset}
${C.cyan}                  ▄▀░░░░░░░░░░░░░░░░░░░░░░░▀▄                   ${C.reset}
${C.cyan}                ▄▀░░░░░░░░░${C.bright}◆${C.cyan}░░░░░░░░░░░░░░░░░▀▄                 ${C.reset}
${C.cyan}               █░░░░░${C.blue}╔═╗╔═╗╔╦╗╔═╗╔═╗╦═╗${C.cyan}░░░░░░░█                ${C.reset}
${C.cyan}              █░░░░░░${C.blue}╠═╣╠╦╝║║║║ ╦║╣ ╠╦╝${C.cyan}░░░░░░░░█               ${C.reset}
${C.cyan}              █░░░░░░${C.blue}╩ ╩╩╚═╩ ╩╚═╝╚═╝╩╚═${C.cyan}░░░░░░░░█               ${C.reset}
${C.cyan}               █░░░░░░░░░${C.bright}◆${C.cyan}░░░░░░░░░░░░░░░░░█                ${C.reset}
${C.cyan}                ▀▄░░░░░░░░░░░░░░░░░░░░░▄▀                  ${C.reset}
${C.cyan}                  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀                     ${C.reset}`,

  // Compact logo for tight spaces
  logoCompact: `
${C.cyan} ╔═╗╔═╗╔╦╗╔═╗╔═╗╦═╗${C.reset}
${C.cyan} ╠═╣╠╦╝║║║║ ╦║╣ ╠╦╝${C.reset}
${C.cyan} ╩ ╩╩╚═╩ ╩╚═╝╚═╝╩╚═${C.reset}`,

  // Validator node branding
  validatorLogo: `
${C.cyan}    ▄▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▄                    ${C.reset}
${C.cyan}   █  ${C.bright}${C.cyan}AETHER${C.reset}  ${C.bright}VALIDATOR NODE${C.reset}               █                   ${C.reset}
${C.cyan}   █                                      █                   ${C.reset}
${C.cyan}    ▀▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▀                    ${C.reset}`,

  // CLI header with version - cosmic theme
  header: (version) => `
${C.cyan}╔══════════════════════════════════════════════════════════════════════════╗${C.reset}
${C.cyan}║${C.reset}                                                                          ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}  ${C.cyan}╔═╗${C.bright}${C.cyan}AETHER${C.reset}${C.cyan}╔═╗${C.reset}  ${C.bright}BLOCKCHAIN CLI${C.reset}${' '.repeat(34 - version.length)}${C.dim}v${version}${C.reset}  ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}  ${C.cyan}╠═╣${C.bright}${C.cyan}NETWORK${C.reset}${C.cyan}╠═╣${C.reset}  ${C.dim}◆ Decentralized Infrastructure for the Future ◆${C.reset}  ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}  ${C.cyan}╚═╝${C.reset}${' '.repeat(10)}${C.cyan}╚═╝${C.reset}                                          ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}                                                                          ${C.cyan}║${C.reset}
${C.cyan}╚══════════════════════════════════════════════════════════════════════════╝${C.reset}`,

  // Subtle header without borders
  headerMinimal: (version) => `
${C.cyan}  ◆ AETHER CLI${C.reset} ${C.dim}v${version}${C.reset} ${C.dim}─ Decentralized Infrastructure ◆${C.reset}
`,

  // Section headers - cosmic dividers
  section: (title) => `
${C.cyan}  ════════════════════════════════════════════════════════════════════${C.reset}
${C.cyan}  ◆${C.reset} ${C.bright}${title.toUpperCase()}${C.reset}
${C.cyan}  ────────────────────────────────────────────────────────────────────${C.reset}`,

  // Subsection
  subsection: (title) => `
  ${C.dim}┌─ ${C.bright}${C.cyan}${title}${C.reset}${C.dim} ─────────────────────────────────────────┐${C.reset}`,

  // Command banner
  commandBanner: (cmd, desc) => `
${C.cyan}┌──────────────────────────────────────────────────────────────────────────┐${C.reset}
${C.cyan}│${C.reset}  ${C.bright}${C.cyan}COMMAND:${C.reset} ${C.bright}${cmd}${C.reset}${' '.repeat(60 - cmd.length)}${C.cyan}│${C.reset}
${C.cyan}│${C.reset}  ${C.dim}${desc}${C.reset}${' '.repeat(66 - desc.length)}${C.cyan}│${C.reset}
${C.cyan}└──────────────────────────────────────────────────────────────────────────┘${C.reset}`,

  // Welcome banner for init
  welcomeBanner: `
${C.cyan}╔══════════════════════════════════════════════════════════════════════════╗${C.reset}
${C.cyan}║${C.reset}                                                                          ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}  ${C.cyan}╔═╗${C.bright}${C.cyan}WELCOME TO AETHER${C.reset}${C.cyan}╔═╗${C.reset}${' '.repeat(39)}${C.cyan}║${C.reset}
${C.cyan}║${C.reset}  ${C.cyan}╚═╝${C.reset}  ${C.dim}Validator Onboarding & Node Management${C.reset}               ${C.cyan}║${C.reset}
${C.cyan}║${C.reset}                                                                          ${C.cyan}║${C.reset}
${C.cyan}╚══════════════════════════════════════════════════════════════════════════╝${C.reset}`,

  // Success banner
  successBanner: (text) => `
${C.cyan}╔══════════════════════════════════════════════════════════════════════════╗${C.reset}
${C.cyan}║${C.reset}  ${C.green}${C.bright}✓ ${text}${C.reset}${' '.repeat(66 - text.length)}${C.cyan}║${C.reset}
${C.cyan}╚══════════════════════════════════════════════════════════════════════════╝${C.reset}`,

  // Error banner
  errorBanner: (text) => `
${C.cyan}╔══════════════════════════════════════════════════════════════════════════╗${C.reset}
${C.cyan}║${C.reset}  ${C.red}${C.bright}✗ ${text}${C.reset}${' '.repeat(66 - text.length)}${C.cyan}║${C.reset}
${C.cyan}╚══════════════════════════════════════════════════════════════════════════╝${C.reset}`,
};

// ============================================================================
// Status Indicators - Consistent across all commands
// ============================================================================

const indicators = {
  // Success states
  success: `${C.green}✓${C.reset}`,
  successBright: `${C.green}${C.bright}✓${C.reset}`,
  successBox: `${C.green}[✓]${C.reset}`,
  
  // Error states  
  error: `${C.red}✗${C.reset}`,
  errorBright: `${C.red}${C.bright}✗${C.reset}`,
  errorBox: `${C.red}[✗]${C.reset}`,
  
  // Warning states
  warning: `${C.yellow}⚠${C.reset}`,
  warningBright: `${C.yellow}${C.bright}⚠${C.reset}`,
  warningBox: `${C.yellow}[⚠]${C.reset}`,
  
  // Info states
  info: `${C.cyan}ℹ${C.reset}`,
  infoBright: `${C.cyan}${C.bright}ℹ${C.reset}`,
  infoBox: `${C.cyan}[ℹ]${C.reset}`,
  
  // Progress/loading
  bullet: `${C.cyan}●${C.reset}`,
  bulletDim: `${C.dim}●${C.reset}`,
  arrow: `${C.cyan}→${C.reset}`,
  arrowRight: `${C.cyan}→${C.reset}`,
  arrowLeft: `${C.cyan}←${C.reset}`,
  
  // Network/connection
  connected: `${C.green}●${C.reset}`,
  disconnected: `${C.red}●${C.reset}`,
  syncing: `${C.yellow}◐${C.reset}`,
  
  // Checkboxes
  checked: `${C.green}[✓]${C.reset}`,
  unchecked: `${C.dim}[ ]${C.reset}`,
  
  // Stars/ratings
  star: `${C.yellow}★${C.reset}`,
  starDim: `${C.dim}★${C.reset}`,
};

// ============================================================================
// Box Drawing - Consistent borders
// ============================================================================

const box = {
  // Single line borders
  single: {
    topLeft: '┌', topRight: '┐', bottomLeft: '└', bottomRight: '┘',
    horizontal: '─', vertical: '│',
    leftT: '├', rightT: '┤', topT: '┬', bottomT: '┴', cross: '┼'
  },
  
  // Double line borders
  double: {
    topLeft: '╔', topRight: '╗', bottomLeft: '╚', bottomRight: '╝',
    horizontal: '═', vertical: '║',
    leftT: '╠', rightT: '╣', topT: '╦', bottomT: '╩', cross: '╬'
  },
  
  // Rounded corners
  rounded: {
    topLeft: '╭', topRight: '╮', bottomLeft: '╰', bottomRight: '╯',
    horizontal: '─', vertical: '│'
  },
  
  // Thick borders
  thick: {
    topLeft: '▛', topRight: '▜', bottomLeft: '▙', bottomRight: '▟',
    horizontal: '━', vertical: '┃'
  }
};

// ============================================================================
// Message Helpers
// ============================================================================

function success(text) {
  return `${indicators.success} ${text}`;
}

function error(text) {
  return `${indicators.error} ${text}`;
}

function warning(text) {
  return `${indicators.warning} ${text}`;
}

function info(text) {
  return `${indicators.info} ${text}`;
}

function dim(text) {
  return `${C.dim}${text}${C.reset}`;
}

function bright(text) {
  return `${C.bright}${text}${C.reset}`;
}

function highlight(text) {
  return `${C.cyan}${C.bright}${text}${C.reset}`;
}

function code(text) {
  return `${C.cyan}${text}${C.reset}`;
}

function key(text) {
  return `${C.yellow}${C.bright}${text}${C.reset}`;
}

function value(text) {
  return `${C.green}${text}${C.reset}`;
}

// ============================================================================
// Progress/Spinner Helpers
// ============================================================================

const spinnerFrames = ['◐', '◓', '◑', '◒'];
let spinnerInterval = null;
let spinnerIndex = 0;

function startSpinner(text = 'Loading') {
  if (spinnerInterval) clearSpinner();
  
  spinnerIndex = 0;
  process.stdout.write(`  ${C.dim}${spinnerFrames[0]}${C.reset} ${text}...`);
  
  spinnerInterval = setInterval(() => {
    spinnerIndex = (spinnerIndex + 1) % spinnerFrames.length;
    process.stdout.write(`\r  ${C.cyan}${spinnerFrames[spinnerIndex]}${C.reset} ${text}...`);
  }, 120);
}

function updateSpinner(text) {
  if (spinnerInterval) {
    process.stdout.write(`\r  ${C.cyan}${spinnerFrames[spinnerIndex]}${C.reset} ${text}...`);
  }
}

function stopSpinner(success = true, finalText = null) {
  if (spinnerInterval) {
    clearInterval(spinnerInterval);
    spinnerInterval = null;
  }
  
  const icon = success ? indicators.success : indicators.error;
  const text = finalText || (success ? 'Done' : 'Failed');
  process.stdout.write(`\r  ${icon} ${text}\n`);
}

function clearSpinner() {
  if (spinnerInterval) {
    clearInterval(spinnerInterval);
    spinnerInterval = null;
    process.stdout.write('\r' + ' '.repeat(80) + '\r');
  }
}

// ============================================================================
// Progress Bar
// ============================================================================

function progressBar(current, total, width = 40) {
  const pct = Math.min(100, Math.max(0, (current / total) * 100));
  const filled = Math.round((pct / 100) * width);
  const empty = width - filled;
  
  const filledBar = C.green + '█'.repeat(filled) + C.reset;
  const emptyBar = C.dim + '░'.repeat(empty) + C.reset;
  
  return `[${filledBar}${emptyBar}] ${C.bright}${pct.toFixed(1)}%${C.reset}`;
}

function progressBarColored(current, total, width = 40) {
  const pct = Math.min(100, Math.max(0, (current / total) * 100));
  const filled = Math.round((pct / 100) * width);
  const empty = width - filled;
  
  // Color based on progress
  let color = C.red;
  if (pct > 30) color = C.yellow;
  if (pct > 70) color = C.green;
  
  const filledBar = color + '█'.repeat(filled) + C.reset;
  const emptyBar = C.dim + '░'.repeat(empty) + C.reset;
  
  return `[${filledBar}${emptyBar}] ${C.bright}${pct.toFixed(1)}%${C.reset}`;
}

// ============================================================================
// Box Drawing Functions
// ============================================================================

function drawBox(content, options = {}) {
  const {
    style = 'single',
    padding = 1,
    width = null,
    title = null,
    titleColor = C.cyan,
    borderColor = C.dim,
    align = 'left'
  } = options;
  
  const lines = content.split('\n');
  const maxWidth = width || Math.max(...lines.map(l => stripAnsi(l).length));
  const b = box[style];
  
  let result = '';
  
  // Top border with optional title
  if (title) {
    const titleText = ` ${title} `;
    const sideWidth = Math.max(0, (maxWidth + padding * 2 - stripAnsi(titleText).length) / 2);
    const leftPad = b.horizontal.repeat(Math.floor(sideWidth));
    const rightPad = b.horizontal.repeat(Math.ceil(sideWidth));
    result += `${borderColor}${b.topLeft}${leftPad}${titleColor}${titleText}${borderColor}${rightPad}${b.topRight}${C.reset}\n`;
  } else {
    result += `${borderColor}${b.topLeft}${b.horizontal.repeat(maxWidth + padding * 2)}${b.topRight}${C.reset}\n`;
  }
  
  // Content lines
  for (const line of lines) {
    const stripped = stripAnsi(line);
    const padLeft = ' '.repeat(padding);
    const padRight = ' '.repeat(maxWidth - stripped.length + padding);
    result += `${borderColor}${b.vertical}${C.reset}${padLeft}${line}${padRight}${borderColor}${b.vertical}${C.reset}\n`;
  }
  
  // Bottom border
  result += `${borderColor}${b.bottomLeft}${b.horizontal.repeat(maxWidth + padding * 2)}${b.bottomRight}${C.reset}`;
  
  return result;
}

function drawTable(headers, rows, options = {}) {
  const {
    borderStyle = 'single',
    headerColor = C.cyan + C.bright,
    borderColor = C.dim,
    cellPadding = 1
  } = options;
  
  // Calculate column widths
  const colWidths = headers.map((h, i) => {
    const headerWidth = stripAnsi(h).length;
    const maxDataWidth = Math.max(...rows.map(r => stripAnsi(String(r[i] || '')).length));
    return Math.max(headerWidth, maxDataWidth) + cellPadding * 2;
  });
  
  const b = box[borderStyle];
  let result = '';
  
  // Top border
  const topLine = colWidths.map(w => b.horizontal.repeat(w)).join(b.topT);
  result += `${borderColor}${b.topLeft}${topLine}${b.topRight}${C.reset}\n`;
  
  // Header row
  const headerLine = headers.map((h, i) => {
    const pad = colWidths[i] - stripAnsi(h).length;
    const left = Math.floor(pad / 2);
    const right = pad - left;
    return ' '.repeat(left) + headerColor + h + C.reset + ' '.repeat(right);
  }).join(borderColor + b.vertical + C.reset);
  result += `${borderColor}${b.vertical}${C.reset}${headerLine}${borderColor}${b.vertical}${C.reset}\n`;
  
  // Separator
  const sepLine = colWidths.map(w => b.horizontal.repeat(w)).join(b.cross);
  result += `${borderColor}${b.leftT}${sepLine}${b.rightT}${C.reset}\n`;
  
  // Data rows
  for (const row of rows) {
    const line = row.map((cell, i) => {
      const text = String(cell || '');
      const pad = colWidths[i] - stripAnsi(text).length;
      const left = cellPadding;
      const right = pad - left;
      return ' '.repeat(left) + text + ' '.repeat(right);
    }).join(borderColor + b.vertical + C.reset);
    result += `${borderColor}${b.vertical}${C.reset}${line}${borderColor}${b.vertical}${C.reset}\n`;
  }
  
  // Bottom border
  const bottomLine = colWidths.map(w => b.horizontal.repeat(w)).join(b.bottomT);
  result += `${borderColor}${b.bottomLeft}${bottomLine}${b.bottomRight}${C.reset}`;
  
  return result;
}

// ============================================================================
// Utility Functions
// ============================================================================

function stripAnsi(str) {
  return str.replace(/\x1b\[[0-9;]*m/g, '');
}

function center(text, width) {
  const stripped = stripAnsi(text);
  const pad = Math.max(0, width - stripped.length);
  const left = Math.floor(pad / 2);
  const right = pad - left;
  return ' '.repeat(left) + text + ' '.repeat(right);
}

function pad(text, width, align = 'left') {
  const stripped = stripAnsi(text);
  const padLen = Math.max(0, width - stripped.length);
  
  if (align === 'right') return ' '.repeat(padLen) + text;
  if (align === 'center') {
    const left = Math.floor(padLen / 2);
    const right = padLen - left;
    return ' '.repeat(left) + text + ' '.repeat(right);
  }
  return text + ' '.repeat(padLen);
}

function truncate(text, maxLen, suffix = '...') {
  const stripped = stripAnsi(text);
  if (stripped.length <= maxLen) return text;
  return stripped.slice(0, maxLen - suffix.length) + suffix;
}

function wrap(text, width) {
  const words = text.split(' ');
  const lines = [];
  let currentLine = '';
  
  for (const word of words) {
    if (stripAnsi(currentLine + ' ' + word).length > width) {
      lines.push(currentLine);
      currentLine = word;
    } else {
      currentLine += (currentLine ? ' ' : '') + word;
    }
  }
  if (currentLine) lines.push(currentLine);
  
  return lines;
}

// ============================================================================
// Help Formatting
// ============================================================================

function formatHelp(title, description, usage, options, examples) {
  let result = '';
  
  // Title
  result += `\n${C.cyan}${C.bright}${title}${C.reset}\n`;
  
  // Description
  if (description) {
    result += `\n${C.dim}${description}${C.reset}\n`;
  }
  
  // Usage
  if (usage) {
    result += `\n${C.bright}USAGE${C.reset}\n`;
    result += `    ${C.cyan}${usage}${C.reset}\n`;
  }
  
  // Options
  if (options && options.length > 0) {
    result += `\n${C.bright}OPTIONS${C.reset}\n`;
    const maxFlagLen = Math.max(...options.map(o => stripAnsi(o.flag).length));
    for (const opt of options) {
      result += `    ${C.cyan}${pad(opt.flag, maxFlagLen)}${C.reset}  ${opt.desc}\n`;
    }
  }
  
  // Examples
  if (examples && examples.length > 0) {
    result += `\n${C.bright}EXAMPLES${C.reset}\n`;
    for (const ex of examples) {
      result += `    ${C.dim}$${C.reset} ${C.cyan}${ex.cmd}${C.reset}\n`;
      if (ex.desc) {
        result += `    ${C.dim}${' '.repeat(ex.cmd.length + 1)}${ex.desc}${C.reset}\n`;
      }
    }
  }
  
  result += '\n';
  return result;
}

// ============================================================================
// Network Status Helpers
// ============================================================================

function formatLatency(ms) {
  if (ms < 50) return `${C.green}${ms}ms${C.reset}`;
  if (ms < 150) return `${C.yellow}${ms}ms${C.reset}`;
  return `${C.red}${ms}ms${C.reset}`;
}

function formatHealth(status) {
  if (!status) return `${C.dim}unknown${C.reset}`;
  const s = status.toLowerCase();
  if (s === 'ok' || s === 'healthy') return `${C.green}●${C.reset} ${C.green}${status}${C.reset}`;
  if (s === 'degraded') return `${C.yellow}●${C.reset} ${C.yellow}${status}${C.reset}`;
  return `${C.red}●${C.reset} ${C.red}${status}${C.reset}`;
}

function formatSyncStatus(currentSlot, targetSlot) {
  if (!currentSlot || !targetSlot) return `${C.dim}unknown${C.reset}`;
  const pct = (currentSlot / targetSlot) * 100;
  const behind = targetSlot - currentSlot;
  
  if (pct >= 99.9) return `${C.green}synced${C.reset}`;
  if (pct >= 95) return `${C.yellow}syncing${C.reset} ${C.dim}(${behind} behind)${C.reset}`;
  return `${C.red}syncing${C.reset} ${C.dim}(${behind} behind)${C.reset}`;
}

// ============================================================================
// Export
// ============================================================================

module.exports = {
  // Colors
  colors,
  C,
  
  // Branding
  BRANDING,
  
  // Indicators
  indicators,
  
  // Box characters
  box,
  
  // Message helpers
  success,
  error,
  warning,
  info,
  dim,
  bright,
  highlight,
  code,
  key,
  value,
  
  // Spinner/progress
  startSpinner,
  updateSpinner,
  stopSpinner,
  clearSpinner,
  progressBar,
  progressBarColored,
  
  // Box drawing
  drawBox,
  drawTable,
  
  // Utilities
  stripAnsi,
  center,
  pad,
  truncate,
  wrap,
  
  // Help formatting
  formatHelp,
  
  // Network helpers
  formatLatency,
  formatHealth,
  formatSyncStatus,
};
