/**
 * Agent Avatar Component - Jelly-legs AI Team
 * Auto-generated - Do not edit manually
 */

const AGENTS = {
    'jelly-legs': {
        id: 'jelly-legs',
        name: 'Jelly-Legs',
        role: 'Marketing Commander',
        primary: '#ff3333',
        secondary: '#ff6666',
        accent: '#ffffff',
        animation: 'bob',
        frames: 4
    },
    'data-diver': {
        id: 'data-diver',
        name: 'Data-Diver',
        role: 'Research Lead',
        primary: '#3366ff',
        secondary: '#6699ff',
        accent: '#00ffff',
        animation: 'bob',
        frames: 4
    },
    'pattern-seeker': {
        id: 'pattern-seeker',
        name: 'Pattern-Seeker',
        role: 'Trend Analyst',
        primary: '#9933ff',
        secondary: '#b366ff',
        accent: '#ffff00',
        animation: 'pulse',
        frames: 4
    },
    'sketch-bot': {
        id: 'sketch-bot',
        name: 'Sketch-Bot',
        role: 'Design Architect',
        primary: '#ff66cc',
        secondary: '#ff99dd',
        accent: '#33ff99',
        animation: 'pulse',
        frames: 4
    },
    'voice-weaver': {
        id: 'voice-weaver',
        name: 'Voice-Weaver',
        role: 'Brand Voice',
        primary: '#ff9933',
        secondary: '#ffbb66',
        accent: '#ffffff',
        animation: 'pulse',
        frames: 4
    },
    'hook-maker': {
        id: 'hook-maker',
        name: 'Hook-Maker',
        role: 'Viral Engineer',
        primary: '#ffcc00',
        secondary: '#ffdd44',
        accent: '#ff6699',
        animation: 'float',
        frames: 4
    },
    'build-bot': {
        id: 'build-bot',
        name: 'Build-Bot',
        role: 'System Developer',
        primary: '#33cc33',
        secondary: '#66dd66',
        accent: '#ffcc00',
        animation: 'breathe',
        frames: 4
    },
    'pipe-layer': {
        id: 'pipe-layer',
        name: 'Pipe-Layer',
        role: 'Pipeline Engineer',
        primary: '#33cccc',
        secondary: '#66dddd',
        accent: '#ff6633',
        animation: 'breathe',
        frames: 4
    },
    'code-crafter': {
        id: 'code-crafter',
        name: 'Code-Crafter',
        role: 'Implementation',
        primary: '#66ff66',
        secondary: '#99ff99',
        accent: '#00ffff',
        animation: 'float',
        frames: 4
    },
    'shield-bot': {
        id: 'shield-bot',
        name: 'Shield-Bot',
        role: 'Security Guard',
        primary: '#999999',
        secondary: '#bbbbbb',
        accent: '#ff3333',
        animation: 'breathe',
        frames: 4
    },
    'map-maker': {
        id: 'map-maker',
        name: 'Map-Maker',
        role: 'Strategy Lead',
        primary: '#6666ff',
        secondary: '#9999ff',
        accent: '#ffcc00',
        animation: 'pulse',
        frames: 4
    },
    'launch-pad': {
        id: 'launch-pad',
        name: 'Launch-Pad',
        role: 'Deployment Chief',
        primary: '#ffcc00',
        secondary: '#ffdd44',
        accent: '#ff6633',
        animation: 'float',
        frames: 4
    },
};

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
