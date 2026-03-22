/**
 * Pixel Art Avatar Generator for Jelly-legs AI Team
 * Creates 32x32 pixel art avatars with 4-frame idle animations
 * Output: PNG-24 with alpha transparency
 */

import fs from 'fs';
import path from 'path';
import { PixelCanvas } from './png-writer-fixed.js';

// Agent definitions with colors and design specs
const AGENTS = {
  'jelly-legs': {
    name: 'Jelly-Legs',
    role: 'Marketing Commander',
    primary: '#ff3333',
    secondary: '#ff6666',
    dark: '#cc0000',
    light: '#ff9999',
    accent: '#ffffff',
    animation: 'bob'
  },
  'data-diver': {
    name: 'Data-Diver',
    role: 'Research Lead',
    primary: '#3366ff',
    secondary: '#6699ff',
    dark: '#0033cc',
    light: '#99ccff',
    accent: '#00ffff',
    animation: 'bob'
  },
  'pattern-seeker': {
    name: 'Pattern-Seeker',
    role: 'Trend Analyst',
    primary: '#9933ff',
    secondary: '#b366ff',
    dark: '#6600cc',
    light: '#cc99ff',
    accent: '#ffff00',
    animation: 'pulse'
  },
  'sketch-bot': {
    name: 'Sketch-Bot',
    role: 'Design Architect',
    primary: '#ff66cc',
    secondary: '#ff99dd',
    dark: '#cc3399',
    light: '#ffccee',
    accent: '#33ff99',
    animation: 'pulse'
  },
  'voice-weaver': {
    name: 'Voice-Weaver',
    role: 'Brand Voice',
    primary: '#ff9933',
    secondary: '#ffbb66',
    dark: '#cc6600',
    light: '#ffdd99',
    accent: '#ffffff',
    animation: 'pulse'
  },
  'hook-maker': {
    name: 'Hook-Maker',
    role: 'Viral Engineer',
    primary: '#ffcc00',
    secondary: '#ffdd44',
    dark: '#cc9900',
    light: '#ffee88',
    accent: '#ff6699',
    animation: 'float'
  },
  'build-bot': {
    name: 'Build-Bot',
    role: 'System Developer',
    primary: '#33cc33',
    secondary: '#66dd66',
    dark: '#009900',
    light: '#99ee99',
    accent: '#ffcc00',
    animation: 'breathe'
  },
  'pipe-layer': {
    name: 'Pipe-Layer',
    role: 'Pipeline Engineer',
    primary: '#33cccc',
    secondary: '#66dddd',
    dark: '#009999',
    light: '#99eeee',
    accent: '#ff6633',
    animation: 'breathe'
  },
  'code-crafter': {
    name: 'Code-Crafter',
    role: 'Implementation',
    primary: '#66ff66',
    secondary: '#99ff99',
    dark: '#00cc00',
    light: '#ccffcc',
    accent: '#00ffff',
    animation: 'float'
  },
  'shield-bot': {
    name: 'Shield-Bot',
    role: 'Security Guard',
    primary: '#999999',
    secondary: '#bbbbbb',
    dark: '#666666',
    light: '#dddddd',
    accent: '#ff3333',
    animation: 'breathe'
  },
  'map-maker': {
    name: 'Map-Maker',
    role: 'Strategy Lead',
    primary: '#6666ff',
    secondary: '#9999ff',
    dark: '#3333cc',
    light: '#ccccff',
    accent: '#ffcc00',
    animation: 'pulse'
  },
  'launch-pad': {
    name: 'Launch-Pad',
    role: 'Deployment Chief',
    primary: '#ffcc00',
    secondary: '#ffdd44',
    dark: '#cc9900',
    light: '#ffee88',
    accent: '#ff6633',
    animation: 'float'
  }
};

const COLORS = {
  black: '#000000',
  white: '#ffffff',
  skin: '#ffdbac',
  metal_light: '#e8e8e8',
  metal_dark: '#4a4a4a',
  outline: '#1a1a1a'
};

// ============== AGENT DRAWING FUNCTIONS ==============

function drawJellyLegs(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  // Bell (head)
  const bellY = 6 + offset;
  for (let y = bellY; y < bellY + 10; y++) {
    const width = 8 - Math.abs(y - (bellY + 5));
    for (let x = 16 - width; x < 16 + width; x++) {
      canvas.setPixel(x, y, p);
    }
  }
  
  // Highlight
  canvas.setPixel(14, bellY + 3, l);
  canvas.setPixel(15, bellY + 4, l);
  
  // Eyes
  const eyeY = bellY + 5;
  canvas.rect(12, eyeY, 2, 2, COLORS.white);
  canvas.rect(18, eyeY, 2, 2, COLORS.white);
  canvas.setPixel(13, eyeY + 1, COLORS.black);
  canvas.setPixel(19, eyeY + 1, COLORS.black);
  
  // Crown
  const crownY = bellY - 2;
  canvas.rect(13, crownY, 6, 2, a);
  canvas.setPixel(14, crownY - 1, a);
  canvas.setPixel(17, crownY - 1, a);
  
  // Tentacles
  const tentacleOffset = ([0, 1, 2, 1][frame] + frame) % 3 - 1;
  [12, 16, 20].forEach((x, i) => {
    const wave = (tentacleOffset + i) % 3 - 1;
    for (let j = 0; j < 8; j++) {
      const tx = x + (j % 2 === 0 ? wave : 0);
      const ty = bellY + 10 + j;
      if (tx >= 0 && tx < 32 && ty < 32) {
        canvas.setPixel(tx, ty, s);
      }
    }
  });
}

function drawDataDiver(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const helmetY = 8 + offset;
  
  // Helmet
  canvas.circle(16, helmetY + 6, 7, p);
  canvas.circleOutline(16, helmetY + 6, 7, d);
  
  // Viewport
  canvas.circle(16, helmetY + 6, 5, a);
  canvas.circle(16, helmetY + 6, 4, COLORS.metal_light);
  
  // Eyes
  canvas.setPixel(14, helmetY + 6, COLORS.black);
  canvas.setPixel(18, helmetY + 6, COLORS.black);
  
  // Reflection
  canvas.setPixel(13, helmetY + 4, COLORS.white);
  canvas.setPixel(14, helmetY + 3, COLORS.white);
  
  // Tank
  canvas.rect(22, helmetY + 8, 4, 8, d);
  canvas.rect(23, helmetY + 9, 2, 6, s);
  
  // Binary pattern
  if (frame % 2 === 0) {
    canvas.setPixel(23, helmetY + 10, a);
    canvas.setPixel(24, helmetY + 12, a);
  } else {
    canvas.setPixel(24, helmetY + 10, a);
    canvas.setPixel(23, helmetY + 12, a);
  }
  
  // Flippers
  const flipperY = helmetY + 18;
  canvas.rect(10, flipperY, 4, 2, d);
  canvas.rect(18, flipperY, 4, 2, d);
  canvas.setPixel(9, flipperY + 1, d);
  canvas.setPixel(22, flipperY + 1, d);
  
  // Bubbles
  const bubbleY = helmetY - [0, 2, 4, 2][frame];
  canvas.setPixel(24, bubbleY, a);
  canvas.setPixel(26, bubbleY - 2, a);
}

function drawPatternSeeker(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const hatY = 4 + offset;
  
  // Wizard hat
  for (let i = 0; i < 8; i++) {
    const width = i + 1;
    const y = hatY + i;
    for (let x = 16 - width; x < 16 + width; x++) {
      canvas.setPixel(x, y, p);
    }
  }
  
  // Star
  canvas.setPixel(16, hatY + 3, a);
  canvas.setPixel(15, hatY + 4, a);
  canvas.setPixel(17, hatY + 4, a);
  canvas.setPixel(16, hatY + 5, a);
  
  // Face
  const faceY = hatY + 8;
  canvas.rect(12, faceY, 8, 6, s);
  
  // One eye
  canvas.rect(14, faceY + 2, 2, 2, COLORS.white);
  canvas.setPixel(15, faceY + 3, COLORS.black);
  
  // Crystal ball
  const ballY = faceY + 8;
  const glowIntensity = [l, a, l, s][frame];
  canvas.circle(20, ballY + 2, 3, glowIntensity);
  canvas.circle(20, ballY + 2, 2, a);
  canvas.setPixel(20, ballY + 2, COLORS.white);
  
  // Rays
  if (frame % 2 === 0) {
    canvas.setPixel(23, ballY, a);
    canvas.setPixel(24, ballY - 1, a);
    canvas.setPixel(18, ballY - 1, a);
  }
}

function drawSketchBot(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const headY = 6 + offset;
  
  // Robot head
  canvas.rect(10, headY, 12, 10, p);
  canvas.rect(11, headY + 1, 10, 8, s);
  
  // Screen eyes
  canvas.rect(12, headY + 3, 3, 3, COLORS.black);
  canvas.rect(17, headY + 3, 3, 3, COLORS.black);
  
  // Animated expression
  if (frame % 2 === 0) {
    canvas.setPixel(13, headY + 4, a);
    canvas.setPixel(18, headY + 4, a);
  } else {
    canvas.setPixel(13, headY + 4, a);
    canvas.setPixel(13, headY + 5, a);
    canvas.setPixel(18, headY + 4, a);
    canvas.setPixel(18, headY + 5, a);
  }
  
  // Beret
  const beretY = headY - 2;
  canvas.rect(11, beretY, 10, 3, d);
  canvas.rect(9, beretY + 1, 2, 2, d);
  
  // Paintbrush
  const brushX = 8 - [0, 1, 0, -1][frame];
  canvas.line(brushX, headY + 8, brushX - 2, headY + 4, a);
  canvas.setPixel(brushX - 2, headY + 4, COLORS.white);
  
  // Palette
  canvas.circle(22, headY + 10, 3, d);
  canvas.setPixel(21, headY + 9, p);
  canvas.setPixel(23, headY + 10, a);
  
  // Paint splatter
  const splatterX = 14 + (frame % 2);
  canvas.setPixel(splatterX, headY + 12, p);
}

function drawVoiceWeaver(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const maskY = 8 + offset;
  
  // Mask
  canvas.rect(10, maskY, 12, 10, COLORS.white);
  canvas.line(16, maskY, 16, maskY + 9, d);
  
  // Comedy side
  canvas.setPixel(12, maskY + 6, COLORS.black);
  canvas.setPixel(13, maskY + 7, COLORS.black);
  canvas.setPixel(14, maskY + 7, COLORS.black);
  canvas.setPixel(15, maskY + 6, COLORS.black);
  
  // Tragedy side
  canvas.setPixel(17, maskY + 7, COLORS.black);
  canvas.setPixel(18, maskY + 6, COLORS.black);
  canvas.setPixel(19, maskY + 6, COLORS.black);
  canvas.setPixel(20, maskY + 7, COLORS.black);
  
  // Eyes
  canvas.rect(12, maskY + 3, 2, 2, p);
  canvas.rect(18, maskY + 3, 2, 2, s);
  
  // Collar
  const collarY = maskY + 10;
  for (let i = 0; i < 5; i++) {
    canvas.setPixel(11 + i * 2, collarY, a);
    canvas.setPixel(12 + i * 2, collarY + 1, a);
  }
  
  // Microphone
  const micX = 8 - [0, 1, 0, 1][frame];
  canvas.rect(micX, maskY + 6, 2, 6, d);
  canvas.circle(micX + 1, maskY + 4, 2, COLORS.metal_light);
  
  // Sound waves
  const waveX = micX - 2 - (frame % 2);
  canvas.setPixel(waveX, maskY + 4, a);
  canvas.setPixel(waveX - 1, maskY + 3, a);
  canvas.setPixel(waveX - 1, maskY + 5, a);
}

function drawHookMaker(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const hatY = 4 + offset;
  
  // Fisherman hat
  canvas.rect(10, hatY, 12, 4, d);
  canvas.rect(8, hatY + 3, 16, 2, d);
  
  // Lure
  canvas.setPixel(20, hatY + 1, a);
  canvas.setPixel(21, hatY + 2, s);
  
  // Face
  const faceY = hatY + 6;
  canvas.rect(12, faceY, 8, 6, COLORS.skin);
  canvas.setPixel(14, faceY + 2, COLORS.black);
  canvas.setPixel(18, faceY + 2, COLORS.black);
  
  // Fishing rod
  const rodX = 24 + [0, 1, 0, -1][frame];
  canvas.line(rodX, faceY + 8, rodX + 4, faceY - 2, p);
  
  // Hook
  const hookY = faceY - 2 + (frame % 2);
  canvas.circle(rodX + 4, hookY, 2, a);
  canvas.setPixel(rodX + 4, hookY + 2, a);
  
  // Sparkles
  if (frame % 2 === 0) {
    canvas.setPixel(rodX + 6, hookY - 1, COLORS.white);
    canvas.setPixel(rodX + 2, hookY + 1, COLORS.white);
  }
  
  // Ocean pattern
  const shirtY = faceY + 6;
  for (let i = 0; i < 4; i++) {
    canvas.setPixel(13 + i * 2, shirtY, s);
  }
}

function drawBuildBot(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const hatY = 4 + offset;
  
  // Hardhat
  canvas.rect(10, hatY, 12, 4, p);
  canvas.rect(8, hatY + 3, 16, 2, p);
  
  // Gear emblem
  canvas.setPixel(16, hatY + 2, a);
  canvas.setPixel(15, hatY + 1, a);
  canvas.setPixel(17, hatY + 1, a);
  
  // Robot head
  const headY = hatY + 5;
  canvas.rect(11, headY, 10, 8, s);
  canvas.rect(12, headY + 1, 8, 6, l);
  
  // Visor eyes
  canvas.rect(13, headY + 3, 2, 2, COLORS.black);
  canvas.rect(17, headY + 3, 2, 2, COLORS.black);
  
  // Wrench
  const wrenchX = 8 - (frame % 2);
  canvas.rect(wrenchX, headY + 6, 2, 6, a);
  canvas.rect(wrenchX - 1, headY + 5, 4, 2, a);
  
  // Tool belt
  const beltY = headY + 10;
  canvas.rect(11, beltY, 10, 2, d);
  canvas.setPixel(14, beltY + 1, a);
  canvas.setPixel(18, beltY + 1, a);
  
  // Rivets
  const rivetFrame = frame % 2;
  canvas.setPixel(12 + rivetFrame, headY + 2, d);
  canvas.setPixel(19 - rivetFrame, headY + 2, d);
}

function drawPipeLayer(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const capY = 5 + offset;
  
  // Cap
  canvas.rect(11, capY, 10, 4, p);
  canvas.rect(9, capY + 3, 14, 2, p);
  
  // Puzzle emblem
  canvas.setPixel(16, capY + 2, a);
  canvas.setPixel(15, capY + 1, a);
  canvas.setPixel(17, capY + 1, a);
  
  // Face
  const faceY = capY + 6;
  canvas.rect(12, faceY, 8, 6, COLORS.skin);
  
  // Mustache
  canvas.rect(13, faceY + 4, 6, 1, d);
  canvas.setPixel(12, faceY + 3, d);
  canvas.setPixel(19, faceY + 3, d);
  
  // Eyes
  canvas.setPixel(14, faceY + 2, COLORS.black);
  canvas.setPixel(18, faceY + 2, COLORS.black);
  
  // Pipe
  const pipeX = 22 + [0, 1, 0, -1][frame];
  canvas.rect(pipeX, faceY + 2, 4, 8, s);
  canvas.rect(pipeX + 1, faceY + 3, 2, 6, l);
  
  // Connectors
  canvas.rect(pipeX - 1, faceY + 4, 1, 2, d);
  canvas.rect(pipeX + 4, faceY + 6, 1, 2, d);
  
  // Overalls
  const overallsY = faceY + 6;
  canvas.rect(11, overallsY, 10, 6, d);
  for (let i = 0; i < 3; i++) {
    canvas.setPixel(13 + i * 3, overallsY + 2, s);
  }
}

function drawCodeCrafter(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const hoodY = 6 + offset;
  
  // Hoodie
  canvas.rect(10, hoodY, 12, 10, d);
  canvas.rect(12, hoodY + 2, 8, 6, s);
  
  // Hood
  canvas.rect(9, hoodY + 1, 2, 6, d);
  canvas.rect(21, hoodY + 1, 2, 6, d);
  
  // Glasses
  const glassesY = hoodY + 4;
  canvas.rect(12, glassesY, 8, 2, COLORS.metal_dark);
  
  // Code reflection
  const codeX = 13 + (frame % 2);
  canvas.setPixel(codeX, glassesY, a);
  canvas.setPixel(codeX + 2, glassesY + 1, a);
  canvas.setPixel(codeX + 4, glassesY, a);
  
  // Keyboard
  const kbY = hoodY + 12 + [0, -1, 0, 1][frame];
  canvas.rect(8, kbY, 16, 4, a);
  canvas.rect(9, kbY + 1, 14, 2, d);
  
  // Keys
  for (let i = 0; i < 5; i++) {
    canvas.setPixel(10 + i * 3, kbY + 1, l);
  }
  
  // Binary stream
  const streamX = 6;
  const streamY = hoodY + 2 + frame;
  canvas.setPixel(streamX, streamY, a);
  canvas.setPixel(streamX, streamY + 2, l);
  canvas.setPixel(streamX - 1, streamY + 4, a);
}

function drawShieldBot(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const helmetY = 5 + offset;
  
  // Helmet
  canvas.rect(11, helmetY, 10, 8, s);
  canvas.rect(10, helmetY + 2, 12, 6, l);
  
  // Visor
  const visorY = helmetY + 4;
  canvas.rect(12, visorY, 8, 3, d);
  
  // Eyes
  const eyeGlow = [a, l, a, s][frame];
  canvas.setPixel(14, visorY + 1, eyeGlow);
  canvas.setPixel(17, visorY + 1, eyeGlow);
  
  // Shield
  const shieldX = 8 - (frame % 2);
  canvas.circle(shieldX + 2, helmetY + 10, 4, a);
  canvas.circle(shieldX + 2, helmetY + 10, 3, COLORS.white);
  
  // Lock
  canvas.rect(shieldX + 1, helmetY + 9, 2, 2, d);
  canvas.setPixel(shieldX + 2, helmetY + 8, d);
  
  // Armor
  const plateY = helmetY + 10;
  canvas.rect(14, plateY, 4, 6, s);
  canvas.rect(15, plateY + 1, 2, 4, l);
  
  // Rivets
  canvas.setPixel(15, plateY + 2, d);
  canvas.setPixel(17, plateY + 2, d);
  canvas.setPixel(15, plateY + 4, d);
  canvas.setPixel(17, plateY + 4, d);
}

function drawMapMaker(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const hatY = 4 + offset;
  
  // Explorer hat
  canvas.rect(11, hatY, 10, 4, d);
  canvas.rect(9, hatY + 3, 14, 2, d);
  
  // Map in hat
  canvas.rect(19, hatY + 1, 2, 4, a);
  canvas.setPixel(20, hatY, a);
  
  // Face
  const faceY = hatY + 6;
  canvas.rect(12, faceY, 8, 6, COLORS.skin);
  canvas.setPixel(14, faceY + 2, COLORS.black);
  canvas.setPixel(18, faceY + 2, COLORS.black);
  
  // Rolled map
  const mapX = 22 + [0, 1, 0, -1][frame];
  canvas.rect(mapX, faceY + 4, 3, 8, a);
  canvas.rect(mapX + 1, faceY + 5, 1, 6, l);
  
  // Compass
  const compassX = 9 - (frame % 2);
  canvas.circle(compassX, faceY + 8, 3, COLORS.metal_light);
  canvas.circle(compassX, faceY + 8, 2, COLORS.white);
  
  // Needle
  const needleAngle = frame % 4;
  if (needleAngle < 2) {
    canvas.setPixel(compassX, faceY + 7, a);
    canvas.setPixel(compassX, faceY + 9, d);
  } else {
    canvas.setPixel(compassX - 1, faceY + 8, a);
    canvas.setPixel(compassX + 1, faceY + 8, d);
  }
  
  // Path
  const pathY = faceY + 12;
  for (let i = 0; i < 4; i++) {
    canvas.setPixel(12 + i * 2, pathY, s);
  }
}

function drawLaunchPad(canvas, frame, colors) {
  const { primary: p, secondary: s, dark: d, light: l, accent: a } = colors;
  const offset = [0, -1, 0, 1][frame];
  
  const helmetY = 6 + offset;
  
  // Space helmet
  canvas.circle(16, helmetY + 5, 6, COLORS.white);
  canvas.circleOutline(16, helmetY + 5, 6, d);
  
  // Visor
  canvas.rect(12, helmetY + 3, 8, 4, a);
  canvas.rect(13, helmetY + 4, 6, 2, d);
  canvas.setPixel(14, helmetY + 4, l);
  
  // Jetpack
  const packX = 22 + [0, 1, 0, -1][frame];
  canvas.rect(packX, helmetY + 6, 5, 8, s);
  canvas.rect(packX + 1, helmetY + 7, 3, 6, d);
  
  // Flames
  const flameY = helmetY + 14;
  const flameColors = [a, l, a, s];
  for (let i = 0; i < 3; i++) {
    const fx = packX + 1 + i;
    const fy = flameY + (frame % 3);
    canvas.setPixel(fx, fy, flameColors[i]);
    canvas.setPixel(fx, fy + 1, a);
  }
  
  // Timer
  const timerX = 10 - (frame % 2);
  canvas.rect(timerX, helmetY + 8, 3, 3, COLORS.black);
  canvas.setPixel(timerX + 1, helmetY + 9, a);
  
  // Stars
  const starPositions = [[8, helmetY], [24, helmetY + 2], [26, helmetY + 8]];
  starPositions.forEach(([sx, sy], i) => {
    if ((frame + i) % 2 === 0) {
      canvas.setPixel(sx, sy, COLORS.white);
    }
  });
  
  // Trajectory
  canvas.setPixel(20, helmetY + 16, s);
  canvas.setPixel(22, helmetY + 17, s);
}

// ============== MAIN GENERATION FUNCTIONS ==============

const DRAWING_FUNCTIONS = {
  'jelly-legs': drawJellyLegs,
  'data-diver': drawDataDiver,
  'pattern-seeker': drawPatternSeeker,
  'sketch-bot': drawSketchBot,
  'voice-weaver': drawVoiceWeaver,
  'hook-maker': drawHookMaker,
  'build-bot': drawBuildBot,
  'pipe-layer': drawPipeLayer,
  'code-crafter': drawCodeCrafter,
  'shield-bot': drawShieldBot,
  'map-maker': drawMapMaker,
  'launch-pad': drawLaunchPad
};

function generateAgentFrames(agentId, agentData) {
  const frames = [];
  const colors = {
    primary: agentData.primary,
    secondary: agentData.secondary,
    dark: agentData.dark,
    light: agentData.light,
    accent: agentData.accent
  };
  
  const drawFunc = DRAWING_FUNCTIONS[agentId];
  if (!drawFunc) {
    console.error(`No drawing function for ${agentId}`);
    return [];
  }
  
  for (let frame = 0; frame < 4; frame++) {
    const canvas = new PixelCanvas(32, 32);
    drawFunc(canvas, frame, colors);
    frames.push(canvas);
  }
  
  return frames;
}

function saveFrames(agentId, frames, outputDir) {
  const agentDir = path.join(outputDir, 'individual', agentId);
  fs.mkdirSync(agentDir, { recursive: true });
  
  frames.forEach((canvas, i) => {
    const framePath = path.join(agentDir, `idle-${i}.png`);
    canvas.save(framePath);
  });
  
  // Create agent spritesheet
  const agentSheet = new PixelCanvas(128, 32);
  frames.forEach((canvas, i) => {
    for (let y = 0; y < 32; y++) {
      for (let x = 0; x < 32; x++) {
        const pixel = canvas.pixels[y * 32 + x];
        if (pixel) {
          agentSheet.setPixel(i * 32 + x, y, pixel);
        }
      }
    }
  });
  
  const sheetDir = path.join(outputDir, 'spritesheets');
  fs.mkdirSync(sheetDir, { recursive: true });
  agentSheet.save(path.join(sheetDir, `${agentId}_spritesheet.png`));
}

function generateCSS(agents, outputDir) {
  let css = `/* Pixel Art Avatar Styles - Jelly-legs AI Team */
/* Auto-generated - Do not edit manually */

:root {
    /* Agent Colors */
`;
  
  for (const [agentId, data] of Object.entries(agents)) {
    const cssName = agentId.replace(/-/g, '_');
    css += `    --${cssName}_primary: ${data.primary};\n`;
    css += `    --${cssName}_secondary: ${data.secondary};\n`;
    css += `    --${cssName}_accent: ${data.accent};\n`;
  }
  
  css += `    
    /* Animation */
    --idle-speed: 0.8s;
    --pixel-scale: 2;
}

/* Base Avatar Styles */
.agent-avatar {
    width: 32px;
    height: 32px;
    image-rendering: pixelated;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
    transform: scale(var(--pixel-scale));
    transform-origin: top left;
}

.agent-avatar-container {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    transition: all 0.3s ease;
}

.agent-avatar-container:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: translateY(-2px);
}

.agent-avatar-container.active {
    box-shadow: 0 0 0 2px var(--agent-color);
}

.agent-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    white-space: nowrap;
}

.agent-role {
    font-size: 10px;
    color: var(--text-secondary);
    text-align: center;
}

/* Size Variants */
.agent-avatar.size-32 { --pixel-scale: 1; }
.agent-avatar.size-64 { --pixel-scale: 2; }
.agent-avatar.size-128 { --pixel-scale: 4; }

/* Animation Keyframes */
@keyframes idle-bob {
    0%, 100% { transform: translateY(0) scale(var(--pixel-scale)); }
    50% { transform: translateY(-2px) scale(var(--pixel-scale)); }
}

@keyframes idle-breathe {
    0%, 100% { transform: scale(var(--pixel-scale)); }
    50% { transform: scale(calc(var(--pixel-scale) * 1.02)); }
}

@keyframes idle-float {
    0%, 100% { transform: translateY(0) rotate(0deg) scale(var(--pixel-scale)); }
    25% { transform: translateY(-1px) rotate(0.5deg) scale(var(--pixel-scale)); }
    75% { transform: translateY(-1px) rotate(-0.5deg) scale(var(--pixel-scale)); }
}

@keyframes idle-pulse {
    0%, 100% { opacity: 1; filter: brightness(1); }
    50% { opacity: 0.9; filter: brightness(1.1); }
}

@keyframes sprite-idle {
    0% { background-position: 0 0; }
    25% { background-position: -32px 0; }
    50% { background-position: -64px 0; }
    75% { background-position: -96px 0; }
    100% { background-position: 0 0; }
}

/* Agent Animation Classes */
`;
  
  const animationClasses = {
    bob: 'idle-bob',
    breathe: 'idle-breathe',
    float: 'idle-float',
    pulse: 'idle-pulse'
  };
  
  for (const [agentId, data] of Object.entries(agents)) {
    const animType = data.animation || 'bob';
    css += `
/* ${data.name} - ${data.role} */
.agent-${agentId} {
    --agent-color: ${data.primary};
    animation: ${animationClasses[animType] || 'idle-bob'} 1.5s ease-in-out infinite;
}

.agent-${agentId} .agent-avatar {
    background-image: url('../avatars/spritesheets/${agentId}_spritesheet.png');
    background-size: 128px 32px;
    animation: sprite-idle var(--idle-speed) steps(4) infinite;
}
`;
  }
  
  css += `
/* Reduced Motion Support */
@media (prefers-reduced-motion: reduce) {
    .agent-avatar,
    [class^="agent-"] {
        animation: none !important;
    }
}

/* High Contrast Mode */
@media (prefers-contrast: high) {
    .agent-avatar {
        filter: contrast(1.5);
    }
}
`;
  
  const cssPath = path.join(outputDir, 'agent-avatars.css');
  fs.writeFileSync(cssPath, css);
  return cssPath;
}

function generateJS(agents, outputDir) {
  let js = `/**
 * Agent Avatar Component - Jelly-legs AI Team
 * Auto-generated - Do not edit manually
 */

const AGENTS = {
`;
  
  for (const [agentId, data] of Object.entries(agents)) {
    js += `    '${agentId}': {
        id: '${agentId}',
        name: '${data.name}',
        role: '${data.role}',
        primary: '${data.primary}',
        secondary: '${data.secondary}',
        accent: '${data.accent}',
        animation: '${data.animation || 'bob'}',
        frames: 4
    },
`;
  }
  
  js += `};

class AgentAvatar {
    constructor(container, agentId, options = {}) {
        this.container = container;
        this.agentId = agentId;
        this.agent = AGENTS[agentId];
        this.options = {
            size: options.size || 64,
            showName: options.showName !== false,
            showRole: options.showRole || false,
            animated: options.animated !== false,
            ...options
        };
        
        this.currentFrame = 0;
        this.animationInterval = null;
        
        this.init();
    }
    
    init() {
        this.container.className = \`agent-avatar-container agent-\${this.agentId}\`;
        this.container.innerHTML = '';
        
        // Create avatar wrapper
        const wrapper = document.createElement('div');
        wrapper.className = 'agent-avatar-wrapper';
        
        // Create avatar element
        this.avatar = document.createElement('div');
        this.avatar.className = \`agent-avatar size-\${this.options.size}\`;
        this.avatar.style.width = '32px';
        this.avatar.style.height = '32px';
        this.avatar.style.backgroundImage = \`url('avatars/spritesheets/\${this.agentId}_spritesheet.png')\`;
        this.avatar.style.backgroundSize = '128px 32px';
        this.avatar.style.backgroundPosition = '0 0';
        this.avatar.style.imageRendering = 'pixelated';
        
        wrapper.appendChild(this.avatar);
        this.container.appendChild(wrapper);
        
        // Add name
        if (this.options.showName) {
            const nameEl = document.createElement('span');
            nameEl.className = 'agent-name';
            nameEl.textContent = this.agent.name;
            this.container.appendChild(nameEl);
        }
        
        // Add role
        if (this.options.showRole) {
            const roleEl = document.createElement('span');
            roleEl.className = 'agent-role';
            roleEl.textContent = this.agent.role;
            this.container.appendChild(roleEl);
        }
        
        // Start animation
        if (this.options.animated) {
            this.startAnimation();
        }
    }
    
    startAnimation() {
        const frameDuration = 200;
        
        this.animationInterval = setInterval(() => {
            this.currentFrame = (this.currentFrame + 1) % this.agent.frames;
            this.updateFrame();
        }, frameDuration);
    }
    
    stopAnimation() {
        if (this.animationInterval) {
            clearInterval(this.animationInterval);
            this.animationInterval = null;
        }
    }
    
    updateFrame() {
        const offset = this.currentFrame * 32;
        this.avatar.style.backgroundPosition = \`-\${offset}px 0\`;
    }
    
    setActive(active) {
        if (active) {
            this.container.classList.add('active');
        } else {
            this.container.classList.remove('active');
        }
    }
    
    destroy() {
        this.stopAnimation();
        this.container.innerHTML = '';
    }
}

// Utility functions
const AvatarUtils = {
    getAgent(agentId) {
        return AGENTS[agentId];
    },
    
    getAllAgents() {
        return Object.values(AGENTS);
    },
    
    preloadSprites() {
        const promises = Object.keys(AGENTS).map(agentId => {
            return new Promise((resolve, reject) => {
                const img = new Image();
                img.onload = resolve;
                img.onerror = reject;
                img.src = \`avatars/spritesheets/\${agentId}_spritesheet.png\`;
            });
        });
        return Promise.all(promises);
    }
};

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { AgentAvatar, AvatarUtils, AGENTS };
}
`;
  
  const jsPath = path.join(outputDir, 'agent-avatars.js');
  fs.writeFileSync(jsPath, js);
  return jsPath;
}

function generateSpritesheetJSON(agents, outputDir) {
  const frames = {};
  let xOffset = 0;
  
  for (const agentId of Object.keys(agents)) {
    for (let frame = 0; frame < 4; frame++) {
      const frameName = `${agentId}-idle-${frame}`;
      frames[frameName] = {
        frame: { x: xOffset, y: 0, w: 32, h: 32 },
        rotated: false,
        trimmed: false,
        spriteSourceSize: { x: 0, y: 0, w: 32, h: 32 },
        sourceSize: { w: 32, h: 32 }
      };
      xOffset += 32;
    }
  }
  
  const spritesheetData = {
    meta: {
      image: 'all-agents-spritesheet.png',
      size: { w: 384, h: 32 },
      scale: '1'
    },
    frames
  };
  
  const jsonPath = path.join(outputDir, 'spritesheet.json');
  fs.writeFileSync(jsonPath, JSON.stringify(spritesheetData, null, 2));
  return jsonPath;
}

function main() {
  const outputDir = 'assets/avatars';
  fs.mkdirSync(outputDir, { recursive: true });
  
  console.log('='.repeat(60));
  console.log('Jelly-legs AI Team - Pixel Art Avatar Generator');
  console.log('='.repeat(60));
  
  const allFrames = [];
  
  for (const [agentId, agentData] of Object.entries(AGENTS)) {
    console.log(`\nGenerating ${agentData.name}...`);
    const frames = generateAgentFrames(agentId, agentData);
    
    if (frames.length > 0) {
      saveFrames(agentId, frames, outputDir);
      allFrames.push(frames);
      console.log(`  ✓ Generated 4 frames + spritesheet`);
    }
  }
  
  // Create combined spritesheet
  console.log('\nCreating combined spritesheet...');
  const combined = new PixelCanvas(384, 32);
  let x = 0;
  for (const agentFrames of allFrames) {
    for (const canvas of agentFrames) {
      for (let y = 0; y < 32; y++) {
        for (let px = 0; px < 32; px++) {
          const pixel = canvas.pixels[y * 32 + px];
          if (pixel) {
            combined.setPixel(x + px, y, pixel);
          }
        }
      }
      x += 32;
    }
  }
  
  const combinedPath = path.join(outputDir, 'all-agents-spritesheet.png');
  combined.save(combinedPath);
  console.log(`  ✓ Saved: ${combinedPath}`);
  
  // Generate CSS
  console.log('\nGenerating CSS...');
  const cssPath = generateCSS(AGENTS, outputDir);
  console.log(`  ✓ Saved: ${cssPath}`);
  
  // Generate JS
  console.log('\nGenerating JavaScript...');
  const jsPath = generateJS(AGENTS, outputDir);
  console.log(`  ✓ Saved: ${jsPath}`);
  
  // Generate JSON
  console.log('\nGenerating JSON metadata...');
  const jsonPath = generateSpritesheetJSON(AGENTS, outputDir);
  console.log(`  ✓ Saved: ${jsonPath}`);
  
  // Print summary
  console.log('\n' + '='.repeat(60));
  console.log('Generation Complete!');
  console.log('='.repeat(60));
  console.log(`\nOutput directory: ${outputDir}`);
  console.log(`Total agents: ${Object.keys(AGENTS).length}`);
  console.log(`Total frames: ${Object.keys(AGENTS).length * 4}`);
  console.log(`Spritesheet size: 384x32px`);
  console.log('\nFiles generated:');
  console.log(`  - Individual frames: ${outputDir}/individual/<agent>/`);
  console.log(`  - Agent spritesheets: ${outputDir}/spritesheets/<agent>_spritesheet.png`);
  console.log(`  - Combined spritesheet: ${combinedPath}`);
  console.log(`  - CSS: ${cssPath}`);
  console.log(`  - JavaScript: ${jsPath}`);
  console.log(`  - JSON metadata: ${jsonPath}`);
}

main();
