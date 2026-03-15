#!/usr/bin/env python3
"""
Pixel Art Avatar Generator for Agent Factory
Generates 12 unique 32x32 pixel art characters with 3-frame idle animations
"""

from PIL import Image, ImageDraw
import os

# Ensure output directory exists
os.makedirs('dashboard/assets/sprites', exist_ok=True)

# Color palettes for each agent
PALETTES = {
    'jelly_legs': {
        'primary': (255, 105, 180),      # Hot pink
        'secondary': (255, 182, 193),    # Light pink
        'accent': (147, 112, 219),       # Medium purple
        'dark': (75, 0, 130),            # Indigo
        'glow': (255, 255, 224),         # Light yellow glow
    },
    'data_diver': {
        'primary': (0, 191, 255),        # Deep sky blue
        'secondary': (70, 130, 180),     # Steel blue
        'accent': (0, 255, 127),         # Spring green (data)
        'dark': (25, 25, 112),           # Midnight blue
        'glow': (0, 255, 255),           # Cyan glow
    },
    'pattern_seeker': {
        'primary': (138, 43, 226),       # Blue violet
        'secondary': (75, 0, 130),       # Indigo
        'accent': (255, 215, 0),         # Gold
        'dark': (25, 25, 25),            # Near black
        'glow': (255, 255, 0),           # Yellow glow
    },
    'sketch_bot': {
        'primary': (255, 140, 0),      # Dark orange
        'secondary': (255, 165, 0),      # Orange
        'accent': (255, 255, 255),       # White
        'dark': (139, 69, 19),           # Saddle brown
        'glow': (255, 200, 100),         # Light orange
    },
    'voice_weaver': {
        'primary': (220, 20, 60),        # Crimson
        'secondary': (255, 255, 255),    # White
        'accent': (0, 0, 0),             # Black
        'dark': (139, 0, 0),             # Dark red
        'glow': (255, 182, 193),         # Pink glow
    },
    'hook_maker': {
        'primary': (255, 215, 0),        # Gold
        'secondary': (184, 134, 11),     # Dark goldenrod
        'accent': (255, 69, 0),            # Red orange
        'dark': (139, 69, 19),           # Saddle brown
        'glow': (255, 255, 200),         # Light yellow
    },
    'build_bot': {
        'primary': (255, 140, 0),      # Dark orange
        'secondary': (169, 169, 169),    # Dark gray
        'accent': (192, 192, 192),       # Silver
        'dark': (64, 64, 64),            # Dark gray
        'glow': (255, 165, 0),           # Orange glow
    },
    'pipe_layer': {
        'primary': (0, 128, 128),        # Teal
        'secondary': (0, 206, 209),        # Dark turquoise
        'accent': (255, 140, 0),         # Dark orange
        'dark': (0, 100, 100),           # Dark teal
        'glow': (64, 224, 208),          # Turquoise glow
    },
    'code_crafter': {
        'primary': (0, 255, 0),          # Lime
        'secondary': (0, 128, 0),          # Green
        'accent': (0, 0, 0),             # Black
        'dark': (0, 50, 0),              # Dark green
        'glow': (50, 255, 50),           # Bright green glow
    },
    'shield_bot': {
        'primary': (70, 130, 180),       # Steel blue
        'secondary': (176, 196, 222),    # Light steel blue
        'accent': (255, 215, 0),         # Gold
        'dark': (25, 25, 112),           # Midnight blue
        'glow': (135, 206, 250),         # Light sky blue
    },
    'map_maker': {
        'primary': (34, 139, 34),        # Forest green
        'secondary': (107, 142, 35),     # Olive drab
        'accent': (139, 69, 19),         # Saddle brown
        'dark': (0, 100, 0),             # Dark green
        'glow': (144, 238, 144),         # Light green
    },
    'launch_pad': {
        'primary': (255, 69, 0),         # Red orange
        'secondary': (255, 140, 0),      # Dark orange
        'accent': (255, 255, 255),       # White
        'dark': (139, 0, 0),             # Dark red
        'glow': (255, 200, 100),         # Light orange glow
    },
}

def create_base_image():
    """Create a 32x32 transparent image"""
    return Image.new('RGBA', (32, 32), (0, 0, 0, 0))

def draw_pixel_rect(draw, x, y, w, h, color):
    """Draw a rectangle of pixels"""
    draw.rectangle([x, y, x + w - 1, y + h - 1], fill=color)

def draw_jelly_legs(frame=0):
    """Jelly-Legs: Marketing Commander - jellyfish with tentacles"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['jelly_legs']
    
    # Bell/head (rounded top)
    bell_colors = [p['primary'], p['secondary'], p['accent']]
    draw_pixel_rect(draw, 10, 4, 12, 8, p['primary'])
    draw_pixel_rect(draw, 8, 6, 16, 6, p['primary'])
    draw_pixel_rect(draw, 6, 8, 20, 4, p['secondary'])
    
    # Inner glow
    draw_pixel_rect(draw, 12, 6, 8, 4, p['glow'])
    
    # Tentacles (animated - sway based on frame)
    sway = frame * 2  # 0, 2, 4
    tentacle_color = p['accent']
    
    # Left tentacles
    for i in range(3):
        tx = 8 + i * 4
        ty = 12
        # Sway animation
        offset = (frame + i) % 3 - 1
        draw_pixel_rect(draw, tx + offset, ty, 2, 4, tentacle_color)
        draw_pixel_rect(draw, tx + offset * 2, ty + 4, 2, 4, tentacle_color)
        draw_pixel_rect(draw, tx + offset, ty + 8, 2, 4, tentacle_color)
    
    # Eyes
    eye_y = 8
    draw_pixel_rect(draw, 12, eye_y, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 18, eye_y, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 13, eye_y + 1, 1, 1, (0, 0, 0))
    draw_pixel_rect(draw, 19, eye_y + 1, 1, 1, (0, 0, 0))
    
    return img

def draw_data_diver(frame=0):
    """Data-Diver: Research Lead - scuba diver with data goggles"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['data_diver']
    
    # Helmet
    draw_pixel_rect(draw, 10, 4, 12, 10, p['secondary'])
    draw_pixel_rect(draw, 8, 6, 16, 8, p['secondary'])
    
    # Goggles (data display)
    goggle_colors = [p['glow'], p['accent'], (0, 255, 0)]
    draw_pixel_rect(draw, 9, 7, 14, 5, p['dark'])
    draw_pixel_rect(draw, 10, 8, 5, 3, goggle_colors[frame % 3])
    draw_pixel_rect(draw, 17, 8, 5, 3, goggle_colors[frame % 3])
    
    # Tank
    draw_pixel_rect(draw, 22, 8, 4, 8, p['primary'])
    
    # Body
    draw_pixel_rect(draw, 10, 14, 12, 10, p['primary'])
    draw_pixel_rect(draw, 8, 16, 16, 8, p['secondary'])
    
    # Fins (animated swimming)
    fin_offset = frame * 2
    draw_pixel_rect(draw, 8 - fin_offset, 24, 4, 4, p['dark'])
    draw_pixel_rect(draw, 20 + fin_offset, 24, 4, 4, p['dark'])
    
    # Data bubbles
    bubble_y = 4 - frame * 2
    draw_pixel_rect(draw, 26, bubble_y, 2, 2, p['glow'])
    draw_pixel_rect(draw, 28, bubble_y - 3, 2, 2, p['accent'])
    
    return img

def draw_pattern_seeker(frame=0):
    """Pattern-Seeker: Trend Analyst - crystal ball, mystical"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['pattern_seeker']
    
    # Crystal ball base
    draw_pixel_rect(draw, 8, 20, 16, 4, p['dark'])
    
    # Crystal ball (glowing orb)
    orb_colors = [p['accent'], p['glow'], (255, 255, 200)]
    glow = orb_colors[frame % 3]
    
    # Ball
    draw_pixel_rect(draw, 10, 10, 12, 12, p['secondary'])
    draw_pixel_rect(draw, 8, 12, 16, 8, p['secondary'])
    draw_pixel_rect(draw, 12, 12, 8, 8, glow)
    draw_pixel_rect(draw, 14, 14, 4, 4, (255, 255, 255))
    
    # Mystical sparkles (animated)
    sparkle_positions = [(6, 8), (26, 10), (28, 18), (6, 20)]
    for i, (sx, sy) in enumerate(sparkle_positions):
        if (i + frame) % 2 == 0:
            draw_pixel_rect(draw, sx, sy, 2, 2, p['glow'])
    
    # Floating runes around ball
    rune_y = 6 + (frame % 3)
    draw_pixel_rect(draw, 14, rune_y, 4, 2, p['accent'])
    
    return img

def draw_sketch_bot(frame=0):
    """Sketch-Bot: Design Architect - paintbrush, creative"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['sketch_bot']
    
    # Body (easel-like)
    draw_pixel_rect(draw, 10, 12, 12, 14, p['dark'])
    draw_pixel_rect(draw, 8, 14, 16, 10, p['secondary'])
    
    # Canvas
    draw_pixel_rect(draw, 10, 10, 12, 10, p['accent'])
    
    # Paint splatters (animated)
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 0), (255, 0, 255)]
    splatter_frame = frame % 3
    if splatter_frame == 0:
        draw_pixel_rect(draw, 12, 12, 3, 3, colors[0])
        draw_pixel_rect(draw, 16, 14, 2, 2, colors[1])
    elif splatter_frame == 1:
        draw_pixel_rect(draw, 14, 11, 2, 3, colors[2])
        draw_pixel_rect(draw, 11, 15, 3, 2, colors[3])
    else:
        draw_pixel_rect(draw, 15, 13, 3, 3, colors[4])
        draw_pixel_rect(draw, 12, 16, 2, 2, colors[0])
    
    # Paintbrush (animated waving)
    brush_angle = frame * 3
    draw_pixel_rect(draw, 22, 8 + brush_angle, 2, 8, p['dark'])
    draw_pixel_rect(draw, 21, 8 + brush_angle, 4, 3, p['primary'])
    
    # Head/eyes
    draw_pixel_rect(draw, 14, 6, 4, 4, p['primary'])
    draw_pixel_rect(draw, 15, 7, 1, 1, (0, 0, 0))
    draw_pixel_rect(draw, 17, 7, 1, 1, (0, 0, 0))
    
    return img

def draw_voice_weaver(frame=0):
    """Voice-Weaver: Brand Voice - theater masks, speech bubbles"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['voice_weaver']
    
    # Theater masks (comedy and tragedy combined)
    # Main mask face
    draw_pixel_rect(draw, 10, 8, 12, 12, p['secondary'])
    draw_pixel_rect(draw, 8, 10, 16, 8, p['secondary'])
    
    # Mask details (animated expression)
    if frame == 0:  # Happy
        draw_pixel_rect(draw, 12, 12, 2, 2, p['accent'])
        draw_pixel_rect(draw, 18, 12, 2, 2, p['accent'])
        draw_pixel_rect(draw, 13, 16, 6, 2, p['accent'])
    elif frame == 1:  # Neutral
        draw_pixel_rect(draw, 12, 12, 2, 2, p['accent'])
        draw_pixel_rect(draw, 18, 12, 2, 2, p['accent'])
        draw_pixel_rect(draw, 13, 15, 6, 1, p['accent'])
    else:  # Dramatic
        draw_pixel_rect(draw, 12, 11, 2, 3, p['accent'])
        draw_pixel_rect(draw, 18, 11, 2, 3, p['accent'])
        draw_pixel_rect(draw, 13, 17, 6, 1, p['accent'])
    
    # Speech bubbles (animated appearing)
    bubble_alpha = frame * 2
    draw_pixel_rect(draw, 22, 6, 8, 6, p['secondary'])
    draw_pixel_rect(draw, 24, 8, 4, 2, p['accent'])
    
    # Sound waves
    wave_x = 6 - frame
    draw_pixel_rect(draw, wave_x, 10, 2, 4, p['primary'])
    draw_pixel_rect(draw, wave_x - 3, 11, 2, 2, p['primary'])
    
    return img

def draw_hook_maker(frame=0):
    """Hook-Maker: Viral Engineer - fishing hook, engagement"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['hook_maker']
    
    # Body (fishing rod holder)
    draw_pixel_rect(draw, 12, 14, 8, 12, p['secondary'])
    draw_pixel_rect(draw, 10, 16, 12, 8, p['primary'])
    
    # Fishing rod
    draw_pixel_rect(draw, 20, 4, 2, 20, p['dark'])
    draw_pixel_rect(draw, 20, 4, 8, 2, p['dark'])
    
    # Hook (animated swinging)
    hook_swing = frame * 2
    draw_pixel_rect(draw, 26 + hook_swing, 6, 2, 6, p['accent'])
    draw_pixel_rect(draw, 26 + hook_swing, 12, 4, 2, p['accent'])
    draw_pixel_rect(draw, 28 + hook_swing, 12, 2, 4, p['accent'])
    
    # Engagement hearts/baits
    heart_y = 18 + frame
    draw_pixel_rect(draw, 28 + hook_swing, heart_y, 2, 2, (255, 0, 0))
    
    # Eyes
    draw_pixel_rect(draw, 14, 12, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 18, 12, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 15, 13, 1, 1, (0, 0, 0))
    draw_pixel_rect(draw, 19, 13, 1, 1, (0, 0, 0))
    
    return img

def draw_build_bot(frame=0):
    """Build-Bot: System Developer - gears, construction"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['build_bot']
    
    # Body
    draw_pixel_rect(draw, 10, 12, 12, 14, p['secondary'])
    draw_pixel_rect(draw, 8, 14, 16, 10, p['primary'])
    
    # Hard hat
    draw_pixel_rect(draw, 10, 6, 12, 4, p['primary'])
    draw_pixel_rect(draw, 8, 8, 16, 2, p['primary'])
    
    # Gear (animated rotation)
    gear_colors = [p['accent'], p['glow'], p['secondary']]
    center_x, center_y = 16, 16
    
    # Center of gear
    draw_pixel_rect(draw, center_x - 2, center_y - 2, 4, 4, gear_colors[frame])
    
    # Gear teeth (simplified)
    teeth = [
        (center_x - 4, center_y - 1, 2, 2),
        (center_x + 2, center_y - 1, 2, 2),
        (center_x - 1, center_y - 4, 2, 2),
        (center_x - 1, center_y + 2, 2, 2),
    ]
    for i, (tx, ty, tw, th) in enumerate(teeth):
        if (i + frame) % 2 == 0:
            draw_pixel_rect(draw, tx, ty, tw, th, p['accent'])
    
    # Wrench
    wrench_y = 22 + frame
    draw_pixel_rect(draw, 22, wrench_y, 6, 2, p['dark'])
    draw_pixel_rect(draw, 26, wrench_y - 2, 2, 6, p['dark'])
    
    return img

def draw_pipe_layer(frame=0):
    """Pipe-Layer: Pipeline Engineer - pipes, connections"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['pipe_layer']
    
    # Body (pipe junction)
    draw_pixel_rect(draw, 12, 12, 8, 12, p['secondary'])
    draw_pixel_rect(draw, 10, 14, 12, 8, p['primary'])
    
    # Horizontal pipe
    draw_pixel_rect(draw, 4, 14, 24, 4, p['primary'])
    draw_pixel_rect(draw, 4, 15, 24, 2, p['secondary'])
    
    # Vertical pipe
    draw_pixel_rect(draw, 14, 4, 4, 24, p['primary'])
    draw_pixel_rect(draw, 15, 4, 2, 24, p['secondary'])
    
    # Junction (animated flow)
    flow_colors = [p['glow'], p['accent'], p['glow']]
    draw_pixel_rect(draw, 14, 14, 4, 4, flow_colors[frame])
    
    # Flow indicators
    flow_pos = 6 + (frame * 3)
    draw_pixel_rect(draw, flow_pos, 15, 2, 2, p['glow'])
    draw_pixel_rect(draw, 15, flow_pos, 2, 2, p['glow'])
    
    # Eyes
    draw_pixel_rect(draw, 20, 10, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 21, 11, 1, 1, (0, 0, 0))
    
    return img

def draw_code_crafter(frame=0):
    """Code-Crafter: Implementation - laptop, code"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['code_crafter']
    
    # Laptop base
    draw_pixel_rect(draw, 6, 20, 20, 4, p['secondary'])
    draw_pixel_rect(draw, 8, 18, 16, 2, p['accent'])
    
    # Laptop screen
    draw_pixel_rect(draw, 8, 8, 16, 10, p['dark'])
    draw_pixel_rect(draw, 10, 10, 12, 6, (0, 20, 0))
    
    # Code lines (animated typing)
    code_colors = [p['glow'], p['primary'], (0, 255, 100)]
    line_y = 11 + frame
    draw_pixel_rect(draw, 11, line_y, 4, 1, code_colors[0])
    draw_pixel_rect(draw, 16, line_y + 2, 3, 1, code_colors[1])
    draw_pixel_rect(draw, 12, line_y + 4, 5, 1, code_colors[2])
    
    # Cursor blink
    if frame % 2 == 0:
        draw_pixel_rect(draw, 18, line_y + 4, 1, 1, (255, 255, 255))
    
    # Head/character behind laptop
    draw_pixel_rect(draw, 12, 4, 8, 6, p['primary'])
    draw_pixel_rect(draw, 14, 6, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 18, 6, 2, 2, (255, 255, 255))
    
    return img

def draw_shield_bot(frame=0):
    """Shield-Bot: Security Guard - shield, armor"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['shield_bot']
    
    # Body (armored)
    draw_pixel_rect(draw, 10, 14, 12, 12, p['secondary'])
    draw_pixel_rect(draw, 8, 16, 16, 8, p['primary'])
    
    # Helmet
    draw_pixel_rect(draw, 10, 6, 12, 8, p['secondary'])
    draw_pixel_rect(draw, 12, 8, 8, 4, p['glow'])
    
    # Shield (animated pulse)
    shield_colors = [p['primary'], p['glow'], p['secondary']]
    shield_color = shield_colors[frame]
    
    # Shield shape
    draw_pixel_rect(draw, 20, 10, 8, 12, shield_color)
    draw_pixel_rect(draw, 22, 8, 4, 14, shield_color)
    
    # Shield emblem (lock)
    draw_pixel_rect(draw, 22, 14, 4, 4, p['accent'])
    draw_pixel_rect(draw, 23, 12, 2, 2, p['accent'])
    
    # Eyes
    draw_pixel_rect(draw, 13, 10, 2, 2, (0, 0, 0))
    draw_pixel_rect(draw, 17, 10, 2, 2, (0, 0, 0))
    
    return img

def draw_map_maker(frame=0):
    """Map-Maker: Strategy Lead - map, compass"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['map_maker']
    
    # Body
    draw_pixel_rect(draw, 10, 14, 12, 12, p['secondary'])
    draw_pixel_rect(draw, 8, 16, 16, 8, p['primary'])
    
    # Map scroll/unfurled
    draw_pixel_rect(draw, 6, 8, 20, 10, p['accent'])
    draw_pixel_rect(draw, 8, 10, 16, 6, (255, 250, 205))
    
    # Map details (animated discovery)
    if frame == 0:
        draw_pixel_rect(draw, 10, 12, 3, 2, p['secondary'])
        draw_pixel_rect(draw, 16, 11, 2, 3, p['secondary'])
    elif frame == 1:
        draw_pixel_rect(draw, 12, 13, 4, 1, p['primary'])
        draw_pixel_rect(draw, 18, 12, 2, 2, p['primary'])
    else:
        draw_pixel_rect(draw, 11, 11, 2, 2, (255, 0, 0))
        draw_pixel_rect(draw, 20, 13, 2, 2, (0, 0, 255))
    
    # Compass (animated needle)
    needle_angle = frame * 2
    draw_pixel_rect(draw, 22, 6, 6, 6, p['accent'])
    draw_pixel_rect(draw, 24, 8, 2, 2, (255, 0, 0))
    draw_pixel_rect(draw, 24 + needle_angle, 7, 1, 3, (255, 0, 0))
    
    # Eyes
    draw_pixel_rect(draw, 14, 6, 2, 2, (255, 255, 255))
    draw_pixel_rect(draw, 18, 6, 2, 2, (255, 255, 255))
    
    return img

def draw_launch_pad(frame=0):
    """Launch-Pad: Deployment Chief - rocket, launch pad"""
    img = create_base_image()
    draw = ImageDraw.Draw(img)
    p = PALETTES['launch_pad']
    
    # Launch pad base
    draw_pixel_rect(draw, 6, 24, 20, 4, p['secondary'])
    draw_pixel_rect(draw, 8, 22, 16, 2, p['dark'])
    
    # Rocket body
    rocket_y = 6 - frame  # Animated hover
    draw_pixel_rect(draw, 12, rocket_y + 4, 8, 16, p['primary'])
    draw_pixel_rect(draw, 14, rocket_y, 4, 4, p['secondary'])
    
    # Rocket fins
    draw_pixel_rect(draw, 10, rocket_y + 14, 2, 4, p['secondary'])
    draw_pixel_rect(draw, 20, rocket_y + 14, 2, 4, p['secondary'])
    
    # Rocket window
    draw_pixel_rect(draw, 14, rocket_y + 6, 4, 4, p['glow'])
    draw_pixel_rect(draw, 15, rocket_y + 7, 2, 2, (0, 0, 0))
    
    # Flame/exhaust (animated)
    flame_colors = [p['accent'], p['glow'], (255, 255, 0)]
    flame_height = 4 + frame * 2
    draw_pixel_rect(draw, 13, rocket_y + 20, 2, flame_height, flame_colors[frame])
    draw_pixel_rect(draw, 17, rocket_y + 20, 2, flame_height, flame_colors[frame])
    
    # Smoke particles
    smoke_y = 26 + frame * 2
    draw_pixel_rect(draw, 8, smoke_y, 3, 2, (200, 200, 200))
    draw_pixel_rect(draw, 20, smoke_y + 1, 3, 2, (200, 200, 200))
    
    return img

# Agent drawing functions mapping
AGENT_DRAWERS = {
    'jelly_legs': draw_jelly_legs,
    'data_diver': draw_data_diver,
    'pattern_seeker': draw_pattern_seeker,
    'sketch_bot': draw_sketch_bot,
    'voice_weaver': draw_voice_weaver,
    'hook_maker': draw_hook_maker,
    'build_bot': draw_build_bot,
    'pipe_layer': draw_pipe_layer,
    'code_crafter': draw_code_crafter,
    'shield_bot': draw_shield_bot,
    'map_maker': draw_map_maker,
    'launch_pad': draw_launch_pad,
}

# Agent display names
AGENT_NAMES = {
    'jelly_legs': 'Jelly-Legs',
    'data_diver': 'Data-Diver',
    'pattern_seeker': 'Pattern-Seeker',
    'sketch_bot': 'Sketch-Bot',
    'voice_weaver': 'Voice-Weaver',
    'hook_maker': 'Hook-Maker',
    'build_bot': 'Build-Bot',
    'pipe_layer': 'Pipe-Layer',
    'code_crafter': 'Code-Crafter',
    'shield_bot': 'Shield-Bot',
    'map_maker': 'Map-Maker',
    'launch_pad': 'Launch-Pad',
}

def generate_agent_sprites(agent_key):
    """Generate 3-frame animation for an agent"""
    frames = []
    drawer = AGENT_DRAWERS[agent_key]
    
    for frame in range(3):
        img = drawer(frame)
        frames.append(img)
    
    return frames

def create_spritesheet():
    """Create a spritesheet with all agents and all frames"""
    # Layout: 12 rows (agents) x 3 columns (frames)
    # Each sprite is 32x32
    # Add 2px padding between sprites
    padding = 2
    sprite_w, sprite_h = 32, 32
    cols = 3
    rows = 12
    
    sheet_width = cols * (sprite_w + padding) + padding
    sheet_height = rows * (sprite_h + padding) + padding
    
    spritesheet = Image.new('RGBA', (sheet_width, sheet_height), (0, 0, 0, 0))
    
    agent_list = list(AGENT_DRAWERS.keys())
    
    for row, agent_key in enumerate(agent_list):
        frames = generate_agent_sprites(agent_key)
        
        for col, frame_img in enumerate(frames):
            x = padding + col * (sprite_w + padding)
            y = padding + row * (sprite_h + padding)
            spritesheet.paste(frame_img, (x, y))
            
            # Also save individual frames
            frame_img.save(f'dashboard/assets/sprites/{agent_key}_frame{col}.png')
        
        print(f"Generated: {AGENT_NAMES[agent_key]}")
    
    spritesheet.save('dashboard/assets/sprites/agents_spritesheet.png')
    print(f"\nSpritesheet saved: {sheet_width}x{sheet_height}px")
    
    return spritesheet, agent_list

def generate_css(agent_list):
    """Generate CSS for sprite animation"""
    css = """/* Agent Factory Sprite Animation CSS */
/* Auto-generated - do not edit manually */

.agent-sprite {
    width: 32px;
    height: 32px;
    image-rendering: pixelated;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
}

/* Spritesheet container */
.agent-spritesheet {
    background-image: url('agents_spritesheet.png');
    background-repeat: no-repeat;
    width: 32px;
    height: 32px;
    image-rendering: pixelated;
    image-rendering: -moz-crisp-edges;
    image-rendering: crisp-edges;
}

/* Animation keyframes */
@keyframes agent-idle {
    0% { background-position-x: 0; }
    33.33% { background-position-x: -34px; }
    66.66% { background-position-x: -68px; }
    100% { background-position-x: 0; }
}

.agent-animated {
    animation: agent-idle 0.6s steps(1) infinite;
}

/* Individual agent positioning */
"""
    
    padding = 2
    sprite_h = 32 + padding
    
    for row, agent_key in enumerate(agent_list):
        y_pos = -(padding + row * sprite_h)
        css += f"""
/* {AGENT_NAMES[agent_key]} */
.agent-{agent_key} {{
    background-position: 0 {y_pos}px;
}}

.agent-{agent_key}.agent-animated {{
    animation: agent-idle 0.6s steps(1) infinite;
}}
"""
    
    css += """
/* Alternative: Individual frame classes */
"""
    
    for row, agent_key in enumerate(agent_list):
        y_pos = -(padding + row * sprite_h)
        for frame in range(3):
            x_pos = -(padding + frame * 34)
            css += f".agent-{agent_key}-f{frame} {{ background-position: {x_pos}px {y_pos}px; }}\n"
    
    css += """
/* Size variants */
.agent-sprite-sm { width: 16px; height: 16px; background-size: 50%; }
.agent-sprite-lg { width: 64px; height: 64px; background-size: 200%; }
.agent-sprite-xl { width: 96px; height: 96px; background-size: 300%; }

/* Usage examples:
   <div class="agent-spritesheet agent-jelly_legs agent-animated"></div>
   <div class="agent-spritesheet agent-data_diver-f1"></div>
*/
"""
    
    with open('dashboard/assets/sprites/agents.css', 'w') as f:
        f.write(css)
    
    print("CSS saved: agents.css")

def generate_html_preview(agent_list):
    """Generate an HTML file to preview all sprites"""
    html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Agent Factory - Sprite Preview</title>
    <link rel="stylesheet" href="agents.css">
    <style>
        body {
            font-family: 'Courier New', monospace;
            background: #1a1a2e;
            color: #eee;
            padding: 20px;
        }
        .agent-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }
        .agent-card {
            background: #16213e;
            border-radius: 8px;
            padding: 15px;
            text-align: center;
            border: 2px solid #0f3460;
        }
        .agent-card:hover {
            border-color: #e94560;
        }
        .agent-preview {
            width: 64px;
            height: 64px;
            margin: 0 auto 10px;
            transform: scale(2);
            transform-origin: center;
        }
        .agent-name {
            font-weight: bold;
            color: #e94560;
            margin-bottom: 5px;
        }
        .agent-key {
            font-size: 0.8em;
            color: #888;
        }
        h1 {
            text-align: center;
            color: #e94560;
        }
        .spritesheet-preview {
            margin: 30px 0;
            text-align: center;
        }
        .spritesheet-preview img {
            border: 2px solid #0f3460;
            border-radius: 4px;
            max-width: 100%;
        }
    </style>
</head>
<body>
    <h1>🎮 Agent Factory - Sprite Preview</h1>
    
    <div class="spritesheet-preview">
        <h2>Spritesheet</h2>
        <img src="agents_spritesheet.png" alt="Agent Spritesheet">
    </div>
    
    <h2>Individual Agents (Animated)</h2>
    <div class="agent-grid">
"""
    
    for agent_key in agent_list:
        name = AGENT_NAMES[agent_key]
        html += f"""        <div class="agent-card">
            <div class="agent-preview">
                <div class="agent-spritesheet agent-{agent_key} agent-animated" style="transform: scale(2); transform-origin: top left;"></div>
            </div>
            <div class="agent-name">{name}</div>
            <div class="agent-key">{agent_key}</div>
        </div>
"""
    
    html += """    </div>
    
    <h2>Static Frames</h2>
    <div class="agent-grid">
"""
    
    for agent_key in agent_list:
        name = AGENT_NAMES[agent_key]
        html += f"""        <div class="agent-card">
            <div style="display: flex; justify-content: center; gap: 5px; margin-bottom: 10px;">
                <div class="agent-spritesheet agent-{agent_key}-f0" style="transform: scale(1.5);"></div>
                <div class="agent-spritesheet agent-{agent_key}-f1" style="transform: scale(1.5);"></div>
                <div class="agent-spritesheet agent-{agent_key}-f2" style="transform: scale(1.5);"></div>
            </div>
            <div class="agent-name">{name}</div>
            <div class="agent-key">Frames 0-2</div>
        </div>
"""
    
    html += """    </div>
</body>
</html>
"""
    
    with open('dashboard/assets/sprites/preview.html', 'w') as f:
        f.write(html)
    
    print("HTML preview saved: preview.html")

def main():
    print("=" * 50)
    print("Agent Factory Pixel Art Generator")
    print("=" * 50)
    print()
    
    # Create spritesheet
    spritesheet, agent_list = create_spritesheet()
    
    # Generate CSS
    generate_css(agent_list)
    
    # Generate HTML preview
    generate_html_preview(agent_list)
    
    print()
    print("=" * 50)
    print("Generation complete!")
    print("=" * 50)
    print()
    print("Files generated:")
    print("  - agents_spritesheet.png (all agents, all frames)")
    print("  - agents.css (animation styles)")
    print("  - preview.html (visual preview)")
    print("  - Individual frame PNGs for each agent")

if __name__ == '__main__':
    main()
