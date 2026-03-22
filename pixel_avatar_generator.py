#!/usr/bin/env python3
"""
Pixel Art Avatar Generator for Jelly-legs AI Team
Creates 32x32 pixel art avatars with 4-frame idle animations
Output: PNG-24 with alpha transparency
"""

from PIL import Image, ImageDraw
import os
import json

# Agent definitions with colors and design specs
AGENTS = {
    "jelly-legs": {
        "name": "Jelly-Legs",
        "role": "Marketing Commander",
        "primary": "#ff3333",
        "secondary": "#ff6666",
        "dark": "#cc0000",
        "light": "#ff9999",
        "accent": "#ffffff",
        "animation": "bob"
    },
    "data-diver": {
        "name": "Data-Diver",
        "role": "Research Lead",
        "primary": "#3366ff",
        "secondary": "#6699ff",
        "dark": "#0033cc",
        "light": "#99ccff",
        "accent": "#00ffff",
        "animation": "bob"
    },
    "pattern-seeker": {
        "name": "Pattern-Seeker",
        "role": "Trend Analyst",
        "primary": "#9933ff",
        "secondary": "#b366ff",
        "dark": "#6600cc",
        "light": "#cc99ff",
        "accent": "#ffff00",
        "animation": "pulse"
    },
    "sketch-bot": {
        "name": "Sketch-Bot",
        "role": "Design Architect",
        "primary": "#ff66cc",
        "secondary": "#ff99dd",
        "dark": "#cc3399",
        "light": "#ffccee",
        "accent": "#33ff99",
        "animation": "pulse"
    },
    "voice-weaver": {
        "name": "Voice-Weaver",
        "role": "Brand Voice",
        "primary": "#ff9933",
        "secondary": "#ffbb66",
        "dark": "#cc6600",
        "light": "#ffdd99",
        "accent": "#ffffff",
        "animation": "pulse"
    },
    "hook-maker": {
        "name": "Hook-Maker",
        "role": "Viral Engineer",
        "primary": "#ffcc00",
        "secondary": "#ffdd44",
        "dark": "#cc9900",
        "light": "#ffee88",
        "accent": "#ff6699",
        "animation": "float"
    },
    "build-bot": {
        "name": "Build-Bot",
        "role": "System Developer",
        "primary": "#33cc33",
        "secondary": "#66dd66",
        "dark": "#009900",
        "light": "#99ee99",
        "accent": "#ffcc00",
        "animation": "breathe"
    },
    "pipe-layer": {
        "name": "Pipe-Layer",
        "role": "Pipeline Engineer",
        "primary": "#33cccc",
        "secondary": "#66dddd",
        "dark": "#009999",
        "light": "#99eeee",
        "accent": "#ff6633",
        "animation": "breathe"
    },
    "code-crafter": {
        "name": "Code-Crafter",
        "role": "Implementation",
        "primary": "#66ff66",
        "secondary": "#99ff99",
        "dark": "#00cc00",
        "light": "#ccffcc",
        "accent": "#00ffff",
        "animation": "float"
    },
    "shield-bot": {
        "name": "Shield-Bot",
        "role": "Security Guard",
        "primary": "#999999",
        "secondary": "#bbbbbb",
        "dark": "#666666",
        "light": "#dddddd",
        "accent": "#ff3333",
        "animation": "breathe"
    },
    "map-maker": {
        "name": "Map-Maker",
        "role": "Strategy Lead",
        "primary": "#6666ff",
        "secondary": "#9999ff",
        "dark": "#3333cc",
        "light": "#ccccff",
        "accent": "#ffcc00",
        "animation": "pulse"
    },
    "launch-pad": {
        "name": "Launch-Pad",
        "role": "Deployment Chief",
        "primary": "#ffcc00",
        "secondary": "#ffdd44",
        "dark": "#cc9900",
        "light": "#ffee88",
        "accent": "#ff6633",
        "animation": "float"
    }
}

# Shared colors
COLORS = {
    "black": "#000000",
    "white": "#ffffff",
    "skin": "#ffdbac",
    "metal_light": "#e8e8e8",
    "metal_dark": "#4a4a4a",
    "outline": "#1a1a1a"
}

def hex_to_rgb(hex_color):
    """Convert hex color to RGB tuple"""
    hex_color = hex_color.lstrip('#')
    return tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))

def create_canvas(size=32):
    """Create a new transparent canvas"""
    return Image.new('RGBA', (size, size), (0, 0, 0, 0))

def draw_pixel(draw, x, y, color, size=1):
    """Draw a single pixel or block"""
    if isinstance(color, str):
        color = hex_to_rgb(color) + (255,)
    draw.rectangle([x, y, x + size - 1, y + size - 1], fill=color)

def draw_rect(draw, x, y, w, h, color):
    """Draw a rectangle"""
    if isinstance(color, str):
        color = hex_to_rgb(color) + (255,)
    draw.rectangle([x, y, x + w - 1, y + h - 1], fill=color)

def draw_circle(draw, cx, cy, r, color):
    """Draw a filled circle"""
    if isinstance(color, str):
        color = hex_to_rgb(color) + (255,)
    for y in range(-r, r + 1):
        for x in range(-r, r + 1):
            if x * x + y * y <= r * r:
                draw.point([cx + x, cy + y], fill=color)

def draw_outline_circle(draw, cx, cy, r, color):
    """Draw a circle outline"""
    if isinstance(color, str):
        color = hex_to_rgb(color) + (255,)
    for y in range(-r, r + 1):
        for x in range(-r, r + 1):
            dist = (x * x + y * y) ** 0.5
            if r - 0.8 <= dist <= r + 0.8:
                draw.point([cx + x, cy + y], fill=color)

def draw_line(draw, x1, y1, x2, y2, color):
    """Draw a line using Bresenham's algorithm"""
    if isinstance(color, str):
        color = hex_to_rgb(color) + (255,)
    
    dx = abs(x2 - x1)
    dy = abs(y2 - y1)
    sx = 1 if x1 < x2 else -1
    sy = 1 if y1 < y2 else -1
    err = dx - dy
    
    while True:
        draw.point([x1, y1], fill=color)
        if x1 == x2 and y1 == y2:
            break
        e2 = 2 * err
        if e2 > -dy:
            err -= dy
            x1 += sx
        if e2 < dx:
            err += dx
            y1 += sy

def lighten(hex_color, percent):
    """Lighten a hex color"""
    rgb = hex_to_rgb(hex_color)
    amt = int(2.55 * percent)
    new_rgb = tuple(min(255, c + amt) for c in rgb)
    return '#{:02x}{:02x}{:02x}'.format(*new_rgb)

def darken(hex_color, percent):
    """Darken a hex color"""
    rgb = hex_to_rgb(hex_color)
    amt = int(2.55 * percent)
    new_rgb = tuple(max(0, c - amt) for c in rgb)
    return '#{:02x}{:02x}{:02x}'.format(*new_rgb)

# ============== AGENT DRAWING FUNCTIONS ==============

def draw_jelly_legs(draw, frame, colors):
    """Draw Jelly-Legs (jellyfish commander)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    # Animation offset
    offset = [0, -1, 0, 1][frame]
    
    # Bell (head) - 16px wide, 10px tall
    bell_y = 6 + offset
    for y in range(bell_y, bell_y + 10):
        width = 8 - abs(y - (bell_y + 5))
        for x in range(16 - width, 16 + width):
            draw_pixel(draw, x, y, p)
    
    # Highlight on bell
    draw_pixel(draw, 14, bell_y + 3, l)
    draw_pixel(draw, 15, bell_y + 4, l)
    
    # Eyes
    eye_y = bell_y + 5
    draw_rect(draw, 12, eye_y, 2, 2, COLORS['white'])
    draw_rect(draw, 18, eye_y, 2, 2, COLORS['white'])
    draw_pixel(draw, 13, eye_y + 1, COLORS['black'])
    draw_pixel(draw, 19, eye_y + 1, COLORS['black'])
    
    # Crown (commander)
    crown_y = bell_y - 2
    draw_rect(draw, 13, crown_y, 6, 2, a)
    draw_pixel(draw, 14, crown_y - 1, a)
    draw_pixel(draw, 17, crown_y - 1, a)
    
    # Tentacles (animated)
    tentacle_offset = [0, 1, 2, 1][frame]
    for i, x in enumerate([12, 16, 20]):
        wave = (tentacle_offset + i) % 3 - 1
        for j in range(8):
            tx = x + wave if j % 2 == 0 else x
            ty = bell_y + 10 + j
            if 0 <= tx < 32 and ty < 32:
                draw_pixel(draw, tx, ty, s)

def draw_data_diver(draw, frame, colors):
    """Draw Data-Diver (deep sea diver)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Diving helmet (round)
    helmet_y = 8 + offset
    draw_circle(draw, 16, helmet_y + 6, 7, p)
    draw_outline_circle(draw, 16, helmet_y + 6, 7, d)
    
    # Glass viewport
    draw_circle(draw, 16, helmet_y + 6, 5, a)
    draw_circle(draw, 16, helmet_y + 6, 4, COLORS['metal_light'])
    
    # Eyes inside viewport
    draw_pixel(draw, 14, helmet_y + 6, COLORS['black'])
    draw_pixel(draw, 18, helmet_y + 6, COLORS['black'])
    
    # Reflection highlight
    draw_pixel(draw, 13, helmet_y + 4, COLORS['white'])
    draw_pixel(draw, 14, helmet_y + 3, COLORS['white'])
    
    # Scuba tank
    draw_rect(draw, 22, helmet_y + 8, 4, 8, d)
    draw_rect(draw, 23, helmet_y + 9, 2, 6, s)
    
    # Binary pattern on tank (changes with frame)
    if frame % 2 == 0:
        draw_pixel(draw, 23, helmet_y + 10, a)
        draw_pixel(draw, 24, helmet_y + 12, a)
    else:
        draw_pixel(draw, 24, helmet_y + 10, a)
        draw_pixel(draw, 23, helmet_y + 12, a)
    
    # Flippers
    flipper_y = helmet_y + 18
    draw_rect(draw, 10, flipper_y, 4, 2, d)
    draw_rect(draw, 18, flipper_y, 4, 2, d)
    draw_pixel(draw, 9, flipper_y + 1, d)
    draw_pixel(draw, 22, flipper_y + 1, d)
    
    # Bubbles (animated)
    bubble_y = helmet_y - [0, 2, 4, 2][frame]
    draw_pixel(draw, 24, bubble_y, a)
    draw_pixel(draw, 26, bubble_y - 2, a)

def draw_pattern_seeker(draw, frame, colors):
    """Draw Pattern-Seeker (wizard with crystal ball)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Wizard hat
    hat_y = 4 + offset
    # Cone
    for i in range(8):
        width = i + 1
        y = hat_y + i
        for x in range(16 - width, 16 + width):
            draw_pixel(draw, x, y, p)
    
    # Star on hat
    draw_pixel(draw, 16, hat_y + 3, a)
    draw_pixel(draw, 15, hat_y + 4, a)
    draw_pixel(draw, 17, hat_y + 4, a)
    draw_pixel(draw, 16, hat_y + 5, a)
    
    # Face/robe
    face_y = hat_y + 8
    draw_rect(draw, 12, face_y, 8, 6, s)
    
    # One visible eye (mysterious)
    draw_rect(draw, 14, face_y + 2, 2, 2, COLORS['white'])
    draw_pixel(draw, 15, face_y + 3, COLORS['black'])
    
    # Crystal ball (glows)
    ball_y = face_y + 8
    glow_intensity = [l, a, l, s][frame]
    draw_circle(draw, 20, ball_y + 2, 3, glow_intensity)
    draw_circle(draw, 20, ball_y + 2, 2, a)
    draw_pixel(draw, 20, ball_y + 2, COLORS['white'])
    
    # Prediction rays (animated)
    ray_offset = frame % 2
    if ray_offset == 0:
        draw_pixel(draw, 23, ball_y, a)
        draw_pixel(draw, 24, ball_y - 1, a)
        draw_pixel(draw, 18, ball_y - 1, a)

def draw_sketch_bot(draw, frame, colors):
    """Draw Sketch-Bot (artist robot)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Robot head (square)
    head_y = 6 + offset
    draw_rect(draw, 10, head_y, 12, 10, p)
    draw_rect(draw, 11, head_y + 1, 10, 8, s)
    
    # Screen eyes
    draw_rect(draw, 12, head_y + 3, 3, 3, COLORS['black'])
    draw_rect(draw, 17, head_y + 3, 3, 3, COLORS['black'])
    
    # Animated eye expression
    if frame % 2 == 0:
        draw_pixel(draw, 13, head_y + 4, a)
        draw_pixel(draw, 18, head_y + 4, a)
    else:
        draw_rect(draw, 13, head_y + 4, 1, 2, a)
        draw_rect(draw, 18, head_y + 4, 1, 2, a)
    
    # Beret (artist hat)
    beret_y = head_y - 2
    draw_rect(draw, 11, beret_y, 10, 3, d)
    draw_rect(draw, 9, beret_y + 1, 2, 2, d)
    
    # Paintbrush (animated)
    brush_x = 8 - [0, 1, 0, -1][frame]
    draw_line(draw, brush_x, head_y + 8, brush_x - 2, head_y + 4, a)
    draw_pixel(draw, brush_x - 2, head_y + 4, COLORS['white'])
    
    # Palette
    draw_circle(draw, 22, head_y + 10, 3, d)
    draw_pixel(draw, 21, head_y + 9, COLORS['primary'])
    draw_pixel(draw, 23, head_y + 10, a)
    
    # Paint splatter
    splatter_x = 14 + (frame % 2)
    draw_pixel(draw, splatter_x, head_y + 12, p)

def draw_voice_weaver(draw, frame, colors):
    """Draw Voice-Weaver (theatrical performer)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Mask (half comedy, half tragedy)
    mask_y = 8 + offset
    draw_rect(draw, 10, mask_y, 12, 10, COLORS['white'])
    draw_line(draw, 16, mask_y, 16, mask_y + 9, d)
    
    # Comedy side (smile)
    draw_pixel(draw, 12, mask_y + 6, COLORS['black'])
    draw_pixel(draw, 13, mask_y + 7, COLORS['black'])
    draw_pixel(draw, 14, mask_y + 7, COLORS['black'])
    draw_pixel(draw, 15, mask_y + 6, COLORS['black'])
    
    # Tragedy side (frown)
    draw_pixel(draw, 17, mask_y + 7, COLORS['black'])
    draw_pixel(draw, 18, mask_y + 6, COLORS['black'])
    draw_pixel(draw, 19, mask_y + 6, COLORS['black'])
    draw_pixel(draw, 20, mask_y + 7, COLORS['black'])
    
    # Eyes
    draw_rect(draw, 12, mask_y + 3, 2, 2, p)
    draw_rect(draw, 18, mask_y + 3, 2, 2, s)
    
    # Ruffled collar
    collar_y = mask_y + 10
    for i in range(5):
        x = 11 + i * 2
        draw_pixel(draw, x, collar_y, a)
        draw_pixel(draw, x + 1, collar_y + 1, a)
    
    # Microphone
    mic_x = 8 - [0, 1, 0, 1][frame]
    draw_rect(draw, mic_x, mask_y + 6, 2, 6, d)
    draw_circle(draw, mic_x + 1, mask_y + 4, 2, COLORS['metal_light'])
    
    # Sound waves (animated)
    wave_x = mic_x - 2 - (frame % 2)
    draw_pixel(draw, wave_x, mask_y + 4, a)
    draw_pixel(draw, wave_x - 1, mask_y + 3, a)
    draw_pixel(draw, wave_x - 1, mask_y + 5, a)

def draw_hook_maker(draw, frame, colors):
    """Draw Hook-Maker (fisherman with hook)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Fisherman hat
    hat_y = 4 + offset
    draw_rect(draw, 10, hat_y, 12, 4, d)
    draw_rect(draw, 8, hat_y + 3, 16, 2, d)
    
    # Lure on hat
    lure_y = hat_y + 1
    draw_pixel(draw, 20, lure_y, a)
    draw_pixel(draw, 21, lure_y + 1, s)
    
    # Face
    face_y = hat_y + 6
    draw_rect(draw, 12, face_y, 8, 6, COLORS['skin'])
    
    # Eyes
    draw_pixel(draw, 14, face_y + 2, COLORS['black'])
    draw_pixel(draw, 18, face_y + 2, COLORS['black'])
    
    # Fishing rod
    rod_x = 24 + [0, 1, 0, -1][frame]
    draw_line(draw, rod_x, face_y + 8, rod_x + 4, face_y - 2, p)
    
    # Hook (glowing)
    hook_y = face_y - 2 + (frame % 2)
    draw_circle(draw, rod_x + 4, hook_y, 2, a)
    draw_pixel(draw, rod_x + 4, hook_y + 2, a)
    
    # Sparkles around hook
    if frame % 2 == 0:
        draw_pixel(draw, rod_x + 6, hook_y - 1, COLORS['white'])
        draw_pixel(draw, rod_x + 2, hook_y + 1, COLORS['white'])
    
    # Ocean pattern on shirt
    shirt_y = face_y + 6
    for i in range(4):
        draw_pixel(draw, 13 + i * 2, shirt_y, s)

def draw_build_bot(draw, frame, colors):
    """Draw Build-Bot (construction robot)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Hardhat
    hat_y = 4 + offset
    draw_rect(draw, 10, hat_y, 12, 4, p)
    draw_rect(draw, 8, hat_y + 3, 16, 2, p)
    
    # Gear emblem on hat
    draw_pixel(draw, 16, hat_y + 2, a)
    draw_pixel(draw, 15, hat_y + 1, a)
    draw_pixel(draw, 17, hat_y + 1, a)
    
    # Robot head (boxy)
    head_y = hat_y + 5
    draw_rect(draw, 11, head_y, 10, 8, s)
    draw_rect(draw, 12, head_y + 1, 8, 6, l)
    
    # Eyes (visor style)
    draw_rect(draw, 13, head_y + 3, 2, 2, COLORS['black'])
    draw_rect(draw, 17, head_y + 3, 2, 2, COLORS['black'])
    
    # Wrench
    wrench_x = 8 - (frame % 2)
    draw_rect(draw, wrench_x, head_y + 6, 2, 6, a)
    draw_rect(draw, wrench_x - 1, head_y + 5, 4, 2, a)
    
    # Tool belt
    belt_y = head_y + 10
    draw_rect(draw, 11, belt_y, 10, 2, d)
    draw_pixel(draw, 14, belt_y + 1, a)
    draw_pixel(draw, 18, belt_y + 1, a)
    
    # Rivets (animated)
    rivet_frame = frame % 2
    draw_pixel(draw, 12 + rivet_frame, head_y + 2, d)
    draw_pixel(draw, 19 - rivet_frame, head_y + 2, d)

def draw_pipe_layer(draw, frame, colors):
    """Draw Pipe-Layer (plumber with pipes)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Cap
    cap_y = 5 + offset
    draw_rect(draw, 11, cap_y, 10, 4, p)
    draw_rect(draw, 9, cap_y + 3, 14, 2, p)
    
    # Puzzle piece emblem
    draw_pixel(draw, 16, cap_y + 2, a)
    draw_pixel(draw, 15, cap_y + 1, a)
    draw_pixel(draw, 17, cap_y + 1, a)
    
    # Face
    face_y = cap_y + 6
    draw_rect(draw, 12, face_y, 8, 6, COLORS['skin'])
    
    # Mustache
    draw_rect(draw, 13, face_y + 4, 6, 1, d)
    draw_pixel(draw, 12, face_y + 3, d)
    draw_pixel(draw, 19, face_y + 3, d)
    
    # Eyes
    draw_pixel(draw, 14, face_y + 2, COLORS['black'])
    draw_pixel(draw, 18, face_y + 2, COLORS['black'])
    
    # Pipe segment over shoulder
    pipe_x = 22 + [0, 1, 0, -1][frame]
    draw_rect(draw, pipe_x, face_y + 2, 4, 8, s)
    draw_rect(draw, pipe_x + 1, face_y + 3, 2, 6, l)
    
    # Pipe connectors
    draw_rect(draw, pipe_x - 1, face_y + 4, 1, 2, d)
    draw_rect(draw, pipe_x + 4, face_y + 6, 1, 2, d)
    
    # Overalls with pipe pattern
    overalls_y = face_y + 6
    draw_rect(draw, 11, overalls_y, 10, 6, d)
    for i in range(3):
        draw_pixel(draw, 13 + i * 3, overalls_y + 2, s)

def draw_code_crafter(draw, frame, colors):
    """Draw Code-Crafter (hacker with terminal)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Hoodie
    hood_y = 6 + offset
    draw_rect(draw, 10, hood_y, 12, 10, d)
    draw_rect(draw, 12, hood_y + 2, 8, 6, s)
    
    # Hood up
    draw_rect(draw, 9, hood_y + 1, 2, 6, d)
    draw_rect(draw, 21, hood_y + 1, 2, 6, d)
    
    # Glasses with code reflection
    glasses_y = hood_y + 4
    draw_rect(draw, 12, glasses_y, 8, 2, COLORS['metal_dark'])
    
    # Code reflection (animated)
    code_x = 13 + (frame % 2)
    draw_pixel(draw, code_x, glasses_y, a)
    draw_pixel(draw, code_x + 2, glasses_y + 1, a)
    draw_pixel(draw, code_x + 4, glasses_y, a)
    
    # Floating holographic keyboard
    kb_y = hood_y + 12 + [0, -1, 0, 1][frame]
    draw_rect(draw, 8, kb_y, 16, 4, a)
    draw_rect(draw, 9, kb_y + 1, 14, 2, d)
    
    # Keys
    for i in range(5):
        draw_pixel(draw, 10 + i * 3, kb_y + 1, l)
    
    # Binary code streaming
    stream_x = 6
    stream_y = hood_y + 2 + frame
    draw_pixel(draw, stream_x, stream_y, a)
    draw_pixel(draw, stream_x, stream_y + 2, l)
    draw_pixel(draw, stream_x - 1, stream_y + 4, a)

def draw_shield_bot(draw, frame, colors):
    """Draw Shield-Bot (security knight)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Helmet
    helmet_y = 5 + offset
    draw_rect(draw, 11, helmet_y, 10, 8, s)
    draw_rect(draw, 10, helmet_y + 2, 12, 6, l)
    
    # Visor
    visor_y = helmet_y + 4
    draw_rect(draw, 12, visor_y, 8, 3, d)
    
    # Eyes (glowing behind visor)
    eye_glow = [a, l, a, s][frame]
    draw_pixel(draw, 14, visor_y + 1, eye_glow)
    draw_pixel(draw, 17, visor_y + 1, eye_glow)
    
    # Shield
    shield_x = 8 - (frame % 2)
    draw_circle(draw, shield_x + 2, helmet_y + 10, 4, a)
    draw_circle(draw, shield_x + 2, helmet_y + 10, 3, COLORS['white'])
    
    # Lock emblem on shield
    draw_rect(draw, shield_x + 1, helmet_y + 9, 2, 2, d)
    draw_pixel(draw, shield_x + 2, helmet_y + 8, d)
    
    # Armor plates
    plate_y = helmet_y + 10
    draw_rect(draw, 14, plate_y, 4, 6, s)
    draw_rect(draw, 15, plate_y + 1, 2, 4, l)
    
    # Rivets
    draw_pixel(draw, 15, plate_y + 2, d)
    draw_pixel(draw, 17, plate_y + 2, d)
    draw_pixel(draw, 15, plate_y + 4, d)
    draw_pixel(draw, 17, plate_y + 4, d)

def draw_map_maker(draw, frame, colors):
    """Draw Map-Maker (cartographer explorer)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Explorer hat
    hat_y = 4 + offset
    draw_rect(draw, 11, hat_y, 10, 4, d)
    draw_rect(draw, 9, hat_y + 3, 14, 2, d)
    
    # Map tucked in hat
    draw_rect(draw, 19, hat_y + 1, 2, 4, a)
    draw_pixel(draw, 20, hat_y, a)
    
    # Face
    face_y = hat_y + 6
    draw_rect(draw, 12, face_y, 8, 6, COLORS['skin'])
    
    # Eyes
    draw_pixel(draw, 14, face_y + 2, COLORS['black'])
    draw_pixel(draw, 18, face_y + 2, COLORS['black'])
    
    # Rolled map under arm
    map_x = 22 + [0, 1, 0, -1][frame]
    draw_rect(draw, map_x, face_y + 4, 3, 8, a)
    draw_rect(draw, map_x + 1, face_y + 5, 1, 6, l)
    
    # Compass in hand
    compass_x = 9 - (frame % 2)
    draw_circle(draw, compass_x, face_y + 8, 3, COLORS['metal_light'])
    draw_circle(draw, compass_x, face_y + 8, 2, COLORS['white'])
    
    # Needle (animated)
    needle_angle = frame % 4
    if needle_angle < 2:
        draw_pixel(draw, compass_x, face_y + 7, a)
        draw_pixel(draw, compass_x, face_y + 9, d)
    else:
        draw_pixel(draw, compass_x - 1, face_y + 8, a)
        draw_pixel(draw, compass_x + 1, face_y + 8, d)
    
    # Path/trail decoration
    path_y = face_y + 12
    for i in range(4):
        draw_pixel(draw, 12 + i * 2, path_y, s)

def draw_launch_pad(draw, frame, colors):
    """Draw Launch-Pad (astronaut with rocket)"""
    p, s, d, l, a = colors['primary'], colors['secondary'], colors['dark'], colors['light'], colors['accent']
    
    offset = [0, -1, 0, 1][frame]
    
    # Space helmet
    helmet_y = 6 + offset
    draw_circle(draw, 16, helmet_y + 5, 6, COLORS['white'])
    draw_outline_circle(draw, 16, helmet_y + 5, 6, d)
    
    # Gold visor
    draw_rect(draw, 12, helmet_y + 3, 8, 4, a)
    draw_rect(draw, 13, helmet_y + 4, 6, 2, d)
    
    # Reflection in visor
    draw_pixel(draw, 14, helmet_y + 4, l)
    
    # Jetpack
    pack_x = 22 + [0, 1, 0, -1][frame]
    draw_rect(draw, pack_x, helmet_y + 6, 5, 8, s)
    draw_rect(draw, pack_x + 1, helmet_y + 7, 3, 6, d)
    
    # Rocket flames (animated)
    flame_y = helmet_y + 14
    flame_colors = [a, l, a, s]
    for i in range(3):
        fx = pack_x + 1 + i
        fy = flame_y + (frame % 3)
        draw_pixel(draw, fx, fy, flame_colors[i])
        draw_pixel(draw, fx, fy + 1, a)
    
    # Countdown timer on wrist
    timer_x = 10 - (frame % 2)
    draw_rect(draw, timer_x, helmet_y + 8, 3, 3, COLORS['black'])
    draw_pixel(draw, timer_x + 1, helmet_y + 9, a)
    
    # Stars around
    star_positions = [(8, helmet_y), (24, helmet_y + 2), (26, helmet_y + 8)]
    for i, (sx, sy) in enumerate(star_positions):
        if (frame + i) % 2 == 0:
            draw_pixel(draw, sx, sy, COLORS['white'])
    
    # Trajectory lines
    draw_pixel(draw, 20, helmet_y + 16, s)
    draw_pixel(draw, 22, helmet_y + 17, s)

# ============== MAIN GENERATION FUNCTIONS ==============

def generate_agent_frames(agent_id, agent_data):
    """Generate all 4 animation frames for an agent"""
    frames = []
    colors = {
        'primary': agent_data['primary'],
        'secondary': agent_data['secondary'],
        'dark': agent_data['dark'],
        'light': agent_data['light'],
        'accent': agent_data['accent']
    }
    
    drawing_functions = {
        'jelly-legs': draw_jelly_legs,
        'data-diver': draw_data_diver,
        'pattern-seeker': draw_pattern_seeker,
        'sketch-bot': draw_sketch_bot,
        'voice-weaver': draw_voice_weaver,
        'hook-maker': draw_hook_maker,
        'build-bot': draw_build_bot,
        'pipe-layer': draw_pipe_layer,
        'code-crafter': draw_code_crafter,
        'shield-bot': draw_shield_bot,
        'map-maker': draw_map_maker,
        'launch-pad': draw_launch_pad
    }
    
    draw_func = drawing_functions.get(agent_id)
    if not draw_func:
        print(f"No drawing function for {agent_id}")
        return []
    
    for frame in range(4):
        img = create_canvas(32)
        draw = ImageDraw.Draw(img)
        draw_func(draw, frame, colors)
        frames.append(img)
    
    return frames

def create_spritesheet(frames):
    """Create a 384x32 spritesheet from 12 agents x 4 frames"""
    spritesheet = Image.new('RGBA', (384, 32), (0, 0, 0, 0))
    
    x_offset = 0
    for agent_frames in frames:
        for frame in agent_frames:
            spritesheet.paste(frame, (x_offset, 0))
            x_offset += 32
    
    return spritesheet

def save_frames(agent_id, frames, output_dir):
    """Save individual frames as PNG"""
    agent_dir = os.path.join(output_dir, 'individual', agent_id)
    os.makedirs(agent_dir, exist_ok=True)
    
    for i, frame in enumerate(frames):
        frame_path = os.path.join(agent_dir, f'idle-{i}.png')
        frame.save(frame_path, 'PNG', optimize=True)
    
    # Also save spritesheet for this agent
    agent_sheet = Image.new('RGBA', (128, 32), (0, 0, 0, 0))
    for i, frame in enumerate(frames):
        agent_sheet.paste(frame, (i * 32, 0))
    
    sheet_path = os.path.join(output_dir, 'spritesheets', f'{agent_id}_spritesheet.png')
    os.makedirs(os.path.dirname(sheet_path), exist_ok=True)
    agent_sheet.save(sheet_path, 'PNG', optimize=True)

def generate_css(agents, output_dir):
    """Generate CSS file for avatar styling"""
    css = """/* Pixel Art Avatar Styles - Jelly-legs AI Team */
/* Auto-generated - Do not edit manually */

:root {
    /* Agent Colors */
"""
    
    for agent_id, data in agents.items():
        css_name = agent_id.replace('-', '_')
        css += f"    --{css_name}_primary: {data['primary']};\n"
        css += f"    --{css_name}_secondary: {data['secondary']};\n"
        css += f"    --{css_name}_accent: {data['accent']};\n"
    
    css += """    
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
"""
    
    animation_classes = {
        'bob': 'idle-bob',
        'breathe': 'idle-breathe',
        'float': 'idle-float',
        'pulse': 'idle-pulse'
    }
    
    for agent_id, data in agents.items():
        anim_type = data.get('animation', 'bob')
        css_name = agent_id.replace('-', '_')
        css += f"""
/* {data['name']} - {data['role']} */
.agent-{agent_id} {{
    --agent-color: {data['primary']};
    animation: {animation_classes.get(anim_type, 'idle-bob')} 1.5s ease-in-out infinite;
}}

.agent-{agent_id} .agent-avatar {{
    background-image: url('../avatars/spritesheets/{agent_id}_spritesheet.png');
    background-size: 128px 32px;
    animation: sprite-idle var(--idle-speed) steps(4) infinite;
}}
"""
    
    css += """
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
"""
    
    css_path = os.path.join(output_dir, 'agent-avatars.css')
    with open(css_path, 'w') as f:
        f.write(css)
    
    return css_path

def generate_js(agents, output_dir):
    """Generate JavaScript component for avatars"""
    js = """/**
 * Agent Avatar Component - Jelly-legs AI Team
 * Auto-generated - Do not edit manually
 */

const AGENTS = {
"""
    
    for agent_id, data in agents.items():
        js += f"""    '{agent_id}': {{
        id: '{agent_id}',
        name: '{data['name']}',
        role: '{data['role']}',
        primary: '{data['primary']}',
        secondary: '{data['secondary']}',
        accent: '{data['accent']}',
        animation: '{data.get('animation', 'bob')}',
        frames: 4
    }},
"""
    
    js += """};

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
        this.container.className = `agent-avatar-container agent-${this.agentId}`;
        this.container.innerHTML = '';
        
        // Create avatar wrapper
        const wrapper = document.createElement('div');
        wrapper.className = 'agent-avatar-wrapper';
        
        // Create avatar element
        this.avatar = document.createElement('div');
        this.avatar.className = `agent-avatar size-${this.options.size}`;
        this.avatar.style.width = '32px';
        this.avatar.style.height = '32px';
        this.avatar.style.backgroundImage = `url('avatars/spritesheets/${this.agentId}_spritesheet.png')`;
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
        const frameDuration = 200; // 200ms per frame = 5fps (idle animation)
        
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
        this.avatar.style.backgroundPosition = `-${offset}px 0`;
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
                img.src = `avatars/spritesheets/${agentId}_spritesheet.png`;
            });
        });
        return Promise.all(promises);
    }
};

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { AgentAvatar, AvatarUtils, AGENTS };
}
"""
    
    js_path = os.path.join(output_dir, 'agent-avatars.js')
    with open(js_path, 'w') as f:
        f.write(js)
    
    return js_path

def generate_spritesheet_json(agents, output_dir):
    """Generate JSON metadata for spritesheet"""
    frames = {}
    x_offset = 0
    
    for agent_id in agents.keys():
        for frame in range(4):
            frame_name = f"{agent_id}-idle-{frame}"
            frames[frame_name] = {
                "frame": {"x": x_offset, "y": 0, "w": 32, "h": 32},
                "rotated": False,
                "trimmed": False,
                "spriteSourceSize": {"x": 0, "y": 0, "w": 32, "h": 32},
                "sourceSize": {"w": 32, "h": 32}
            }
            x_offset += 32
    
    spritesheet_data = {
        "meta": {
            "image": "all-agents-spritesheet.png",
            "size": {"w": 384, "h": 32},
            "scale": "1"
        },
        "frames": frames
    }
    
    json_path = os.path.join(output_dir, 'spritesheet.json')
    with open(json_path, 'w') as f:
        json.dump(spritesheet_data, f, indent=2)
    
    return json_path

def main():
    """Main generation function"""
    output_dir = 'assets/avatars'
    os.makedirs(output_dir, exist_ok=True)
    
    print("=" * 60)
    print("Jelly-legs AI Team - Pixel Art Avatar Generator")
    print("=" * 60)
    
    all_frames = []
    
    for agent_id, agent_data in AGENTS.items():
        print(f"\nGenerating {agent_data['name']}...")
        frames = generate_agent_frames(agent_id, agent_data)
        
        if frames:
            save_frames(agent_id, frames, output_dir)
            all_frames.append(frames)
            print(f"  ✓ Generated 4 frames + spritesheet")
    
    # Create combined spritesheet
    print("\nCreating combined spritesheet...")
    combined = Image.new('RGBA', (384, 32), (0, 0, 0, 0))
    x = 0
    for agent_frames in all_frames:
        for frame in agent_frames:
            combined.paste(frame, (x, 0))
            x += 32
    
    combined_path = os.path.join(output_dir, 'all-agents-spritesheet.png')
    combined.save(combined_path, 'PNG', optimize=True)
    print(f"  ✓ Saved: {combined_path}")
    
    # Generate CSS
    print("\nGenerating CSS...")
    css_path = generate_css(AGENTS, output_dir)
    print(f"  ✓ Saved: {css_path}")
    
    # Generate JS
    print("\nGenerating JavaScript...")
    js_path = generate_js(AGENTS, output_dir)
    print(f"  ✓ Saved: {js_path}")
    
    # Generate JSON
    print("\nGenerating JSON metadata...")
    json_path = generate_spritesheet_json(AGENTS, output_dir)
    print(f"  ✓ Saved: {json_path}")
    
    # Print summary
    print("\n" + "=" * 60)
    print("Generation Complete!")
    print("=" * 60)
    print(f"\nOutput directory: {output_dir}")
    print(f"Total agents: {len(AGENTS)}")
    print(f"Total frames: {len(AGENTS) * 4}")
    print(f"Spritesheet size: 384x32px")
    print("\nFiles generated:")
    print(f"  - Individual frames: {output_dir}/individual/<agent>/")
    print(f"  - Agent spritesheets: {output_dir}/spritesheets/<agent>_spritesheet.png")
    print(f"  - Combined spritesheet: {combined_path}")
    print(f"  - CSS: {css_path}")
    print(f"  - JavaScript: {js_path}")
    print(f"  - JSON metadata: {json_path}")

if __name__ == '__main__':
    main()
