/**
 * Minimal PNG Writer for 32x32 RGBA pixel art
 * Pure JavaScript implementation - no native dependencies
 */

import fs from 'fs';
import path from 'path';
import zlib from 'zlib';

// PNG signature
const PNG_SIGNATURE = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

// CRC32 table
function makeCRCTable() {
  const crcTable = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
    }
    crcTable[n] = c >>> 0; // Force unsigned 32-bit
  }
  return crcTable;
}

const CRC_TABLE = makeCRCTable();

function crc32(data) {
  let crc = 0xFFFFFFFF;
  for (let i = 0; i < data.length; i++) {
    crc = CRC_TABLE[(crc ^ data[i]) & 0xFF] ^ (crc >>> 8);
  }
  return (crc ^ 0xFFFFFFFF) >>> 0; // Force unsigned 32-bit
}

function writeChunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  
  const typeBuffer = Buffer.from(type, 'ascii');
  const crcData = Buffer.concat([typeBuffer, data]);
  const crcValue = crc32(crcData);
  
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crcValue >>> 0, 0); // Ensure unsigned
  
  return Buffer.concat([length, typeBuffer, data, crc]);
}

function createIHDR(width, height) {
  const data = Buffer.alloc(13);
  data.writeUInt32BE(width, 0);
  data.writeUInt32BE(height, 4);
  data.writeUInt8(8, 8); // bit depth
  data.writeUInt8(6, 9); // color type: RGBA
  data.writeUInt8(0, 10); // compression
  data.writeUInt8(0, 11); // filter method
  data.writeUInt8(0, 12); // interlace
  return writeChunk('IHDR', data);
}

function createIDAT(pixels, width, height) {
  // Each row starts with filter byte (0 = none)
  const rowSize = 1 + width * 4; // 1 filter byte + RGBA for each pixel
  const rawData = Buffer.alloc(rowSize * height);
  
  for (let y = 0; y < height; y++) {
    const rowOffset = y * rowSize;
    rawData[rowOffset] = 0; // filter byte
    
    for (let x = 0; x < width; x++) {
      const pixelOffset = rowOffset + 1 + x * 4;
      const pixel = pixels[y * width + x];
      if (pixel) {
        rawData[pixelOffset] = pixel.r;
        rawData[pixelOffset + 1] = pixel.g;
        rawData[pixelOffset + 2] = pixel.b;
        rawData[pixelOffset + 3] = pixel.a;
      } else {
        // Transparent
        rawData[pixelOffset] = 0;
        rawData[pixelOffset + 1] = 0;
        rawData[pixelOffset + 2] = 0;
        rawData[pixelOffset + 3] = 0;
      }
    }
  }
  
  // Compress with zlib
  const compressed = zlib.deflateSync(rawData, { level: 9 });
  return writeChunk('IDAT', compressed);
}

function createIEND() {
  return writeChunk('IEND', Buffer.alloc(0));
}

/**
 * Create a 32x32 PNG from pixel data
 * @param {Array} pixels - Array of {r, g, b, a} objects or null for transparent
 * @param {string} filename - Output file path
 */
export function savePNG(pixels, width, height, filename) {
  const ihdr = createIHDR(width, height);
  const idat = createIDAT(pixels, width, height);
  const iend = createIEND();
  
  const png = Buffer.concat([PNG_SIGNATURE, ihdr, idat, iend]);
  fs.writeFileSync(filename, png);
  return filename;
}

/**
 * Pixel canvas class
 */
export class PixelCanvas {
  constructor(width, height) {
    this.width = width;
    this.height = height;
    this.pixels = new Array(width * height).fill(null);
  }
  
  // Parse hex color
  parseColor(color) {
    if (color === null || color === undefined) {
      return null;
    }
    if (typeof color === 'object' && color !== null && 'r' in color) {
      return color;
    }
    if (typeof color === 'string') {
      if (color.startsWith('#')) {
        const hex = color.slice(1);
        // Handle 3-char hex
        if (hex.length === 3) {
          const r = parseInt(hex[0] + hex[0], 16);
          const g = parseInt(hex[1] + hex[1], 16);
          const b = parseInt(hex[2] + hex[2], 16);
          return { r, g, b, a: 255 };
        }
        // Handle 6-char hex
        const bigint = parseInt(hex, 16);
        const r = (bigint >> 16) & 255;
        const g = (bigint >> 8) & 255;
        const b = bigint & 255;
        return { r, g, b, a: 255 };
      }
    }
    return { r: 0, g: 0, b: 0, a: 255 };
  }
  
  // Lighten a color
  lighten(color, percent) {
    const c = this.parseColor(color);
    if (!c) return null;
    const amt = Math.round(2.55 * percent);
    return {
      r: Math.min(255, c.r + amt),
      g: Math.min(255, c.g + amt),
      b: Math.min(255, c.b + amt),
      a: c.a
    };
  }
  
  // Darken a color
  darken(color, percent) {
    const c = this.parseColor(color);
    if (!c) return null;
    const amt = Math.round(2.55 * percent);
    return {
      r: Math.max(0, c.r - amt),
      g: Math.max(0, c.g - amt),
      b: Math.max(0, c.b - amt),
      a: c.a
    };
  }
  
  // Set a single pixel
  setPixel(x, y, color) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) return;
    if (color === null || color === undefined) return;
    const index = y * this.width + x;
    this.pixels[index] = this.parseColor(color);
  }
  
  // Draw a filled rectangle
  rect(x, y, w, h, color) {
    for (let py = y; py < y + h; py++) {
      for (let px = x; px < x + w; px++) {
        this.setPixel(px, py, color);
      }
    }
  }
  
  // Draw a circle (filled)
  circle(cx, cy, r, color) {
    for (let y = -r; y <= r; y++) {
      for (let x = -r; x <= r; x++) {
        if (x * x + y * y <= r * r) {
          this.setPixel(cx + x, cy + y, color);
        }
      }
    }
  }
  
  // Draw a circle outline
  circleOutline(cx, cy, r, color) {
    for (let y = -r; y <= r; y++) {
      for (let x = -r; x <= r; x++) {
        const dist = Math.sqrt(x * x + y * y);
        if (dist >= r - 0.8 && dist <= r + 0.8) {
          this.setPixel(cx + x, cy + y, color);
        }
      }
    }
  }
  
  // Draw a line (Bresenham's algorithm)
  line(x1, y1, x2, y2, color) {
    const dx = Math.abs(x2 - x1);
    const dy = Math.abs(y2 - y1);
    const sx = x1 < x2 ? 1 : -1;
    const sy = y1 < y2 ? 1 : -1;
    let err = dx - dy;
    
    while (true) {
      this.setPixel(x1, y1, color);
      if (x1 === x2 && y1 === y2) break;
      const e2 = 2 * err;
      if (e2 > -dy) { err -= dy; x1 += sx; }
      if (e2 < dx) { err += dx; y1 += sy; }
    }
  }
  
  // Save to file
  save(filename) {
    return savePNG(this.pixels, this.width, this.height, filename);
  }
}
