/**
 * Agent Pixel Art Sprites - 32x32px CSS Box-Shadow Style
 * Each agent has a distinct visual identity based on their role
 * Based on research: simple, meme-ready, animal-inspired where appropriate
 */

const AGENT_SPRITES = {
  // RESEARCH STAGE
  data_diver: {
    name: "Data-Diver",
    emoji: "🤿",
    color: "#00CED1", // Cyan
    accent: "#008B8B", // Dark cyan
    // Pixel art as box-shadow values (each pixel = 2px for 32x32 at 1px scale)
    shadow: `
      0 2px 0 #00CED1, 2px 2px 0 #00CED1, 4px 2px 0 #00CED1, 6px 2px 0 #00CED1,
      0 4px 0 #00CED1, 2px 4px 0 #00CED1, 4px 4px 0 #008B8B, 6px 4px 0 #00CED1,
      0 6px 0 #00CED1, 2px 6px 0 #008B8B, 4px 6px 0 #00796B, 6px 6px 0 #00CED1,
      0 8px 0 #00CED1, 2px 8px 0 #00CED1, 4px 8px 0 #00CED1, 6px 8px 0 #00CED1,
      0 10px 0 #008B8B, 2px 10px 0 #000, 4px 10px 0 #000, 6px 10px 0 #008B8B,
      0 12px 0 #008B8B, 2px 12px 0 #fff, 4px 12px 0 #fff, 6px 12px 0 #008B8B,
      0 14px 0 #00CED1, 2px 14px 0 #00CED1, 4px 14px 0 #00CED1, 6px 14px 0 #00CED1
    `,
    width: 8,
    height: 8
  },
  
  pattern_seeker: {
    name: "Pattern-Seeker",
    emoji: "🔮",
    color: "#8B008B", // Dark magenta
    accent: "#4B0082", // Indigo
    shadow: `
      2px 0 0 #8B008B, 4px 0 0 #8B008B,
      0 2px 0 #4B0082, 2px 2px 0 #DA70D6, 4px 2px 0 #DA70D6, 6px 2px 0 #4B0082,
      0 4px 0 #4B0082, 2px 4px 0 #DA70D6, 4px 4px 0 #fff, 6px 4px 0 #4B0082,
      0 6px 0 #8B008B, 2px 6px 0 #8B008B, 4px 6px 0 #8B008B, 6px 6px 0 #8B008B,
      2px 8px 0 #4B0082, 4px 8px 0 #4B0082,
      2px 10px 0 #4B0082, 4px 10px 0 #4B0082
    `,
    width: 8,
    height: 12
  },

  // DESIGN STAGE
  sketch_bot: {
    name: "Sketch-Bot",
    emoji: "🎨",
    color: "#FF69B4", // Hot pink
    accent: "#FFD700", // Gold
    shadow: `
      0 0 0 #FF69B4, 2px 0 0 #FFD700, 4px 0 0 #32CD32, 6px 0 0 #1E90FF,
      0 2px 0 #FF69B4, 2px 2px 0 #000, 4px 2px 0 #000, 6px 2px 0 #32CD32,
      0 4px 0 #FFD700, 2px 4px 0 #FFD700, 4px 4px 0 #FF69B4, 6px 4px 0 #FF69B4,
      0 6px 0 #1E90FF, 2px 6px 0 #32CD32, 4px 6px 0 #FFD700
    `,
    width: 8,
    height: 8
  },
  
  voice_weaver: {
    name: "Voice-Weaver",
    emoji: "🎭",
    color: "#FF69B4", // Pink
    accent: "#FFD700", // Gold
    shadow: `
      2px 0 0 #FF69B4, 4px 0 0 #FF69B4,
      0 2px 0 #FFD700, 2px 2px 0 #FF69B4, 4px 2px 0 #FF69B4, 6px 2px 0 #FFD700,
      2px 4px 0 #000, 4px 4px 0 #000,
      0 6px 0 #FFD700, 2px 6px 0 #FF69B4, 4px 6px 0 #FF69B4, 6px 6px 0 #FFD700,
      0 8px 0 #000, 6px 8px 0 #000
    `,
    width: 8,
    height: 10
  },
  
  hook_maker: {
    name: "Hook-Maker",
    emoji: "🪝",
    color: "#32CD32", // Lime green
    accent: "#00FF00", // Bright green
    shadow: `
      6px 0 0 #32CD32,
      4px 2px 0 #32CD32, 6px 2px 0 #00FF00,
      2px 4px 0 #32CD32, 4px 4px 0 #228B22,
      0 6px 0 #32CD32, 2px 6px 0 #228B22,
      2px 8px 0 #00FF00,
      4px 10px 0 #00FF00
    `,
    width: 8,
    height: 12
  },

  // BUILD STAGE
  build_bot: {
    name: "Build-Bot",
    emoji: "⚙️",
    color: "#FF8C00", // Orange
    accent: "#B22222", // Firebrick
    shadow: `
      2px 0 0 #FF8C00, 4px 0 0 #FF8C00,
      0 2px 0 #B22222, 2px 2px 0 #FF8C00, 4px 2px 0 #FF8C00, 6px 2px 0 #B22222,
      0 4px 0 #B22222, 2px 4px 0 #000, 4px 4px 0 #000, 6px 4px 0 #B22222,
      0 6px 0 #B22222, 2px 6px 0 #FF8C00, 4px 6px 0 #FF8C00, 6px 6px 0 #B22222,
      2px 8px 0 #FF8C00, 4px 8px 0 #FF8C00
    `,
    width: 8,
    height: 10
  },
  
  pipe_layer: {
    name: "Pipe-Layer",
    emoji: "🧩",
    color: "#708090", // Slate gray
    accent: "#4682B4", // Steel blue
    shadow: `
      0 0 0 #4682B4, 2px 0 0 #708090, 4px 0 0 #708090, 6px 0 0 #708090,
      0 2px 0 #708090, 2px 2px 0 #4682B4, 4px 2px 0 #708090, 6px 2px 0 #708090,
      0 4px 0 #708090, 2px 4px 0 #708090, 4px 4px 0 #4682B4, 6px 4px 0 #708090,
      0 6px 0 #708090, 2px 6px 0 #708090, 4px 6px 0 #708090, 6px 6px 0 #4682B4
    `,
    width: 8,
    height: 8
  },
  
  code_crafter: {
    name: "Code-Crafter",
    emoji: "💻",
    color: "#00FF00", // Terminal green
    accent: "#00AA00", // Darker green
    shadow: `
      0 0 0 #333, 2px 0 0 #333, 4px 0 0 #333, 6px 0 0 #333, 8px 0 0 #333,
      0 2px 0 #333, 2px 2px 0 #00FF00, 4px 2px 0 #0f0, 6px 2px 0 #0f0, 8px 2px 0 #333,
      0 4px 0 #333, 2px 4px 0 #00FF00, 4px 4px 0 #0f0, 6px 4px 0 #0f0, 8px 4px 0 #333,
      0 6px 0 #333, 2px 6px 0 #333, 4px 6px 0 #333, 6px 6px 0 #333, 8px 6px 0 #333
    `,
    width: 10,
    height: 8
  },

  // SECURITY STAGE
  shield_bot: {
    name: "Shield-Bot",
    emoji: "🛡️",
    color: "#C0C0C0", // Silver
    accent: "#FFD700", // Gold
    shadow: `
      2px 0 0 #FFD700, 4px 0 0 #FFD700,
      0 2px 0 #FFD700, 2px 2px 0 #C0C0C0, 4px 2px 0 #C0C0C0, 6px 2px 0 #FFD700,
      0 4px 0 #C0C0C0, 2px 4px 0 #C0C0C0, 4px 4px 0 #006400, 6px 4px 0 #C0C0C0,
      0 6px 0 #C0C0C0, 2px 6px 0 #006400, 4px 6px 0 #C0C0C0, 6px 6px 0 #C0C0C0,
      0 8px 0 #C0C0C0, 2px 8px 0 #C0C0C0, 4px 8px 0 #C0C0C0, 6px 8px 0 #C0C0C0,
      2px 10px 0 #C0C0C0, 4px 10px 0 #C0C0C0
    `,
    width: 8,
    height: 12
  },

  // STRATEGY STAGE
  map_maker: {
    name: "Map-Maker",
    emoji: "🗺️",
    color: "#8B4513", // Brown
    accent: "#228B22", // Forest green
    shadow: `
      0 0 0 #DEB887, 2px 0 0 #DEB887, 4px 0 0 #DEB887, 6px 0 0 #DEB887,
      0 2px 0 #DEB887, 2px 2px 0 #8B4513, 4px 2px 0 #228B22, 6px 2px 0 #DEB887,
      0 4px 0 #DEB887, 2px 4px 0 #228B22, 4px 4px 0 #1E90FF, 6px 4px 0 #DEB887,
      0 6px 0 #DEB887, 2px 6px 0 #8B4513, 4px 6px 0 #8B4513, 6px 6px 0 #DEB887,
      0 8px 0 #DEB887, 2px 8px 0 #DEB887, 4px 8px 0 #DEB887, 6px 8px 0 #DEB887
    `,
    width: 8,
    height: 10
  },
  
  jelly_legs: {
    name: "Jelly-Legs",
    emoji: "🪼",
    color: "#FF1493", // Deep pink
    accent: "#9370DB", // Medium purple
    shadow: `
      2px 0 0 #FF1493, 4px 0 0 #FF1493,
      0 2px 0 #9370DB, 2px 2px 0 #FF1493, 4px 2px 0 #FF1493, 6px 2px 0 #9370DB,
      0 4px 0 #FF1493, 2px 4px 0 #fff, 4px 4px 0 #fff, 6px 4px 0 #FF1493,
      0 6px 0 #9370DB, 2px 6px 0 #FF1493, 4px 6px 0 #FF1493, 6px 6px 0 #9370DB,
      2px 8px 0 #9370DB, 4px 8px 0 #9370DB,
      0 10px 0 #FF1493, 6px 10px 0 #FF1493,
      0 12px 0 #9370DB, 6px 12px 0 #9370DB
    `,
    width: 8,
    height: 14
  },

  // DEPLOY STAGE
  launch_pad: {
    name: "Launch-Pad",
    emoji: "🚀",
    color: "#FF4500", // Orange red
    accent: "#FFA500", // Orange
    shadow: `
      2px 0 0 #FF4500, 4px 0 0 #FF4500,
      2px 2px 0 #FFA500, 4px 2px 0 #FFA500,
      0 4px 0 #FF4500, 2px 4px 0 #FFA500, 4px 4px 0 #FFA500, 6px 4px 0 #FF4500,
      0 6px 0 #FF4500, 2px 6px 0 #FFA500, 4px 6px 0 #FFA500, 6px 6px 0 #FF4500,
      0 8px 0 #DC143C, 2px 8px 0 #FF4500, 4px 8px 0 #FF4500, 6px 8px 0 #DC143C,
      0 10px 0 #FFA500, 6px 10px 0 #FFA500,
      2px 12px 0 #FFD700, 4px 12px 0 #FFD700
    `,
    width: 8,
    height: 14
  }
};

/**
 * Generate CSS for pixel art sprite
 * @param {string} agentKey - Key from AGENT_SPRITES
 * @returns {string} CSS class definition
 */
function generateSpriteCSS(agentKey) {
  const sprite = AGENT_SPRITES[agentKey];
  if (!sprite) return '';
  
  return `
    .sprite-${agentKey} {
      position: relative;
      width: 2px;
      height: 2px;
      background: transparent;
      box-shadow: ${sprite.shadow.trim()};
      transform: scale(2);
      transform-origin: top left;
    }
  `;
}

/**
 * Get all sprite CSS
 * @returns {string} Complete CSS for all sprites
 */
function getAllSpriteCSS() {
  return Object.keys(AGENT_SPRITES)
    .map(generateSpriteCSS)
    .join('\n');
}

/**
 * Render agent sprite to HTML
 * @param {string} agentKey - Key from AGENT_SPRITES
 * @returns {string} HTML div element
 */
function renderSpriteHTML(agentKey) {
  const sprite = AGENT_SPRITES[agentKey];
  if (!sprite) return '';
  
  return `<div class="sprite-${agentKey}" title="${sprite.name}" aria-label="${sprite.name}"></div>`;
}

/**
 * SVG-based pixel art (alternative approach)
 * These are 32x32 SVG sprites embedded as data URIs
 */
const SPRITE_SVGS = {
  data_diver: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect x="4" y="1" width="8" height="2" fill="#00CED1"/><rect x="2" y="3" width="12" height="2" fill="#00CED1"/><rect x="2" y="5" width="4" height="1" fill="#008B8B"/><rect x="6" y="5" width="4" height="1" fill="#00CED1"/><rect x="10" y="5" width="4" height="1" fill="#008B8B"/><rect x="2" y="6" width="12" height="4" fill="#00796B"/><rect x="5" y="7" width="2" height="1" fill="#000"/><rect x="9" y="7" width="2" height="1" fill="#000"/><rect x="2" y="10" width="12" height="2" fill="#00CED1"/><rect x="6" y="12" width="4" height="2" fill="#00CED1"/></svg>')}`,
  
  pattern_seeker: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><ellipse cx="8" cy="5" rx="5" ry="4" fill="#8B008B"/><ellipse cx="8" cy="5" rx="3" ry="2" fill="#DA70D6"/><circle cx="8" cy="5" r="1" fill="#fff"/><rect x="6" y="9" width="4" height="1" fill="#4B0082"/><rect x="5" y="10" width="6" height="2" fill="#4B0082"/><rect x="6" y="12" width="4" height="3" fill="#4B0082"/></svg>')}`,
  
  sketch_bot: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6" fill="#FF69B4"/><rect x="3" y="3" width="3" height="3" fill="#FFD700"/><rect x="10" y="3" width="3" height="3" fill="#32CD32"/><rect x="6" y="6" width="4" height="4" fill="#FF69B4"/><circle cx="6" cy="7" r="1" fill="#000"/><circle cx="10" cy="7" r="1" fill="#000"/><path d="M6 9 Q8 11 10 9" stroke="#FF1493" fill="none"/></svg>')}`,
  
  voice_weaver: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><ellipse cx="8" cy="6" rx="5" ry="4" fill="#FF69B4"/><ellipse cx="5" cy="5" rx="2" ry="2" fill="#FFD700"/><ellipse cx="11" cy="5" rx="2" ry="2" fill="#FFD700"/><rect x="6" y="7" width="1" height="2" fill="#000"/><rect x="9" y="7" width="1" height="2" fill="#000"/><path d="M5 10 Q8 13 11 10" stroke="#FFD700" fill="none"/></svg>')}`,
  
  hook_maker: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M12 1 L12 5 Q12 8 8 8 L8 6" stroke="#32CD32" stroke-width="2" fill="none"/><circle cx="12" cy="10" r="2" fill="#00FF00"/><path d="M10 12 Q8 14 6 12" stroke="#228B22" fill="none"/></svg>')}`,
  
  build_bot: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="6" fill="#FF8C00"/><circle cx="8" cy="8" r="4" fill="#333"/><circle cx="8" cy="8" r="2" fill="#FF8C00"/><rect x="2" y="7" width="2" height="2" fill="#B22222"/><rect x="12" y="7" width="2" height="2" fill="#B22222"/><rect x="7" y="2" width="2" height="2" fill="#B22222"/><rect x="7" y="12" width="2" height="2" fill="#B22222"/></svg>')}`,
  
  pipe_layer: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect x="1" y="1" width="6" height="6" fill="#4682B4" rx="1"/><rect x="9" y="1" width="6" height="6" fill="#708090" rx="1"/><rect x="1" y="9" width="6" height="6" fill="#708090" rx="1"/><rect x="9" y="9" width="6" height="6" fill="#4682B4" rx="1"/><rect x="6" y="6" width="4" height="4" fill="#333"/></svg>')}`,
  
  code_crafter: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect x="1" y="2" width="14" height="10" fill="#333" rx="2"/><rect x="2" y="3" width="12" height="8" fill="#1a1a1a"/><text x="3" y="8" font-family="monospace" font-size="3" fill="#00FF00">&gt;_</text><rect x="1" y="12" width="14" height="2" fill="#333"/></svg>')}`,
  
  shield_bot: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M8 1 L14 3 L14 8 Q14 13 8 15 Q2 13 2 8 L2 3 Z" fill="#C0C0C0"/><path d="M8 2 L13 4 L13 8 Q13 12 8 14" fill="#E8E8E8"/><path d="M8 5 L10 7 L8 14 Q3 12 3 8 L3 4 Z" fill="#FFD700"/><rect x="6" y="6" width="4" height="4" fill="#006400" rx="2"/></svg>')}`,
  
  map_maker: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect x="1" y="1" width="14" height="14" fill="#DEB887"/><path d="M0 0 L16 16" stroke="#8B4513" stroke-width="1"/><path d="M16 0 L0 16" stroke="#8B4513" stroke-width="1"/><circle cx="8" cy="4" r="2" fill="#228B22"/><circle cx="12" cy="10" r="2" fill="#1E90FF"/><rect x="2" y="8" width="2" height="4" fill="#8B4513"/><rect x="12" y="2" width="2" height="2" fill="#FF4500"/></svg>')}`,
  
  jelly_legs: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><ellipse cx="8" cy="6" rx="5" ry="4" fill="#FF1493"/><ellipse cx="6" cy="5" rx="1" ry="1" fill="#fff"/><ellipse cx="10" cy="5" rx="1" ry="1" fill="#fff"/><path d="M5 8 Q8 10 11 8" stroke="#9370DB" fill="none"/><path d="M4 10 Q3 12 2 15" stroke="#FF1493" stroke-width="2" fill="none"/><path d="M7 10 Q7 12 6 15" stroke="#FF1493" stroke-width="2" fill="none"/><path d="M9 10 Q9 12 10 15" stroke="#FF1493" stroke-width="2" fill="none"/><path d="M12 10 Q13 12 14 15" stroke="#FF1493" stroke-width="2" fill="none"/></svg>')}`,
  
  launch_pad: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M8 0 L10 4 L10 8 L6 8 L6 4 Z" fill="#FF4500"/><rect x="6" y="8" width="4" height="4" fill="#DC143C"/><polygon points="4,12 12,12 14,16 2,16" fill="#FFA500"/><polygon points="0,16 4,12 4,16" fill="#FFD700"/><polygon points="16,16 12,12 12,16" fill="#FFD700"/><rect x="7" y="6" width="2" height="2" fill="#fff"/></svg>')}`
};

/**
 * Get image URL for agent sprite
 * @param {string} agentKey - Key from AGENT_SPRITES
 * @returns {string} Data URI for SVG sprite
 */
function getSpriteImageURL(agentKey) {
  return SPRITE_SVGS[agentKey] || '';
}

// Export for use in dashboard
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { AGENT_SPRITES, SPRITE_SVGS, generateSpriteCSS, getAllSpriteCSS, renderSpriteHTML, getSpriteImageURL };
}
