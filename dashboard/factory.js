// Agent Factory Dashboard - Factory View Logic
// Zone Detail Modal and Work Request functionality

// Zone configuration with agents
const zoneConfig = {
    'research': {
        name: '🔬 Research Lab',
        agents: ['data-diver', 'pattern-seeker'],
        description: 'Market research, trend analysis, and data mining'
    },
    'design': {
        name: '✏️ Design Studio',
        agents: ['sketch-bot', 'voice-weaver', 'hook-maker'],
        description: 'Visual design, brand voice, and content creation'
    },
    'build': {
        name: '⚙️ Build Factory',
        agents: ['build-bot', 'pipe-layer', 'code-crafter'],
        description: 'Development, infrastructure, and implementation'
    },
    'security': {
        name: '🔒 Security Zone',
        agents: ['shield-bot'],
        description: 'Risk assessment, audits, and compliance'
    },
    'strategy': {
        name: '📊 Strategy Room',
        agents: ['map-maker'],
        description: 'Planning, roadmaps, and coordination'
    },
    'deploy': {
        name: '🚀 Deploy Pad',
        agents: ['launch-pad'],
        description: 'Launch coordination and deployment'
    },
    'plaza': {
        name: '🌳 Social Plaza',
        agents: ['jelly-legs'],
        description: 'Social coordination and marketing command'
    }
};

// Current dashboard state reference
let dashboardState = null;
let currentZone = null;

// Initialize factory view
function initFactoryView(state) {
    dashboardState = state;
    setupZoneClickHandlers();
    setupModalCloseHandlers();
}

// Setup click handlers for all zones
function setupZoneClickHandlers() {
    const zones = document.querySelectorAll('.zone');
    zones.forEach(zone => {
        zone.style.cursor = 'pointer';
        zone.addEventListener('click', (e) => {
            // Don't trigger if clicking on a task card
            if (e.target.closest('.task-card')) return;
            
            const zoneId = zone.id.replace('zone-', '');
            openZoneModal(zoneId);
        });
    });
}

// Setup modal close handlers
function setupModalCloseHandlers() {
    const zoneModal = document.getElementById('zoneModal');
    const workRequestModal = document.getElementById('workRequestModal');
    
    if (zoneModal) {
        zoneModal.addEventListener('click', (e) => {
            if (e.target === zoneModal) closeZoneModal();
        });
    }
    
    if (workRequestModal) {
        workRequestModal.addEventListener('click', (e) => {
            if (e.target === workRequestModal) closeWorkRequestModal();
        });
    }
    
    // Close on Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            closeZoneModal();
            closeWorkRequestModal();
        }
    });
}

// Open zone detail modal
function openZoneModal(zoneId) {
    currentZone = zoneId;
    const config = zoneConfig[zoneId];
    if (!config) return;
    
    const modal = document.getElementById('zoneModal');
    const titleEl = document.getElementById('zoneModalTitle');
    const bodyEl = document.getElementById('zoneModalBody');
    
    if (!modal || !titleEl || !bodyEl) return;
    
    titleEl.textContent = config.name;
    
    // Get active tasks for this zone
    const zoneTasks = dashboardState?.tasks?.filter(t => 
        t.stage === zoneId && t.state === 'open'
    ) || [];
    
    // Get agents in this zone
    const zoneAgents = config.agents.map(agentId => {
        const agent = dashboardState?.agents?.[agentId];
        return agent ? { ...agent, id: agentId } : null;
    }).filter(Boolean);
    
    // Build modal content
    let content = `
        <div class="zone-info">
            <p class="zone-description">${config.description}</p>
            <div class="zone-agents">
                <h4>Assigned Agents</h4>
                <div class="agent-chips">
                    ${zoneAgents.map(agent => `
                        <div class="agent-chip" style="border-color: ${agent.color || '#ff3333'}">
                            <span class="agent-emoji">${agent.emoji || '🤖'}</span>
                            <span class="agent-name">${agent.name}</span>
                            <span class="agent-status ${agent.status || 'idle'}">${agent.status || 'idle'}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
    `;
    
    // Add tasks section
    if (zoneTasks.length > 0) {
        content += `
            <div class="zone-tasks-section">
                <h4>Active Tasks (${zoneTasks.length})</h4>
                <div class="zone-tasks-list">
                    ${zoneTasks.map(task => {
                        const agent = dashboardState?.agents?.[task.agent];
                        const timestamp = task.createdAt || task.timestamp;
                        const timeAgo = timestamp ? formatTimeAgo(new Date(timestamp)) : 'recently';
                        
                        return `
                            <div class="zone-task-card">
                                <div class="zone-task-header">
                                    <span class="zone-task-status status-${task.state}">●</span>
                                    <span class="zone-task-title">${escapeHtml(task.title)}</span>
                                </div>
                                <div class="zone-task-desc">${escapeHtml(task.description || task.body || 'No description')}</div>
                                <div class="zone-task-meta">
                                    <span class="zone-task-agent">
                                        ${agent?.emoji || '🤖'} ${agent?.name || task.agent}
                                    </span>
                                    <span class="zone-task-time">⏱️ ${timeAgo}</span>
                                    ${task.url ? `<a href="${task.url}" target="_blank" class="zone-task-link">View →</a>` : ''}
                                </div>
                            </div>
                        `;
                    }).join('')}
                </div>
            </div>
        `;
    } else {
        content += `
            <div class="zone-empty-state">
                <div class="empty-icon">📭</div>
                <p>No active tasks in this zone</p>
                <p class="empty-hint">Create a work request to get started</p>
            </div>
        `;
    }
    
    // Add Create Work Request button
    content += `
        <div class="modal-actions">
            <button class="btn-create-work" onclick="openWorkRequestForZone('${zoneId}')">
                <span>+</span> Create Work Request
            </button>
        </div>
    `;
    
    bodyEl.innerHTML = content;
    modal.classList.add('visible');
}

// Close zone modal
function closeZoneModal() {
    const modal = document.getElementById('zoneModal');
    if (modal) {
        modal.classList.remove('visible');
    }
    currentZone = null;
}

// Open work request modal for specific zone
function openWorkRequestForZone(zoneId) {
    closeZoneModal();
    openWorkRequestModal(zoneId);
}

// Open work request modal
function openWorkRequestModal(preselectedZone = null) {
    const modal = document.getElementById('workRequestModal');
    const form = document.getElementById('workRequestForm');
    const success = document.getElementById('formSuccess');
    const zoneSelect = document.getElementById('workZone');
    
    if (!modal) return;
    
    // Reset form
    if (form) form.style.display = 'block';
    if (success) success.style.display = 'none';
    if (form) form.reset();
    
    // Preselect zone if provided
    if (preselectedZone && zoneSelect) {
        zoneSelect.value = preselectedZone;
    }
    
    modal.classList.add('visible');
}

// Close work request modal
function closeWorkRequestModal() {
    const modal = document.getElementById('workRequestModal');
    if (modal) {
        modal.classList.remove('visible');
    }
}

// Submit work request
function submitWorkRequest(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    
    const request = {
        title: formData.get('title'),
        zone: formData.get('zone'),
        priority: formData.get('priority'),
        description: formData.get('description'),
        createdAt: new Date().toISOString(),
        id: 'wr-' + Date.now()
    };
    
    // In a real implementation, this would send to an API
    console.log('Work Request Created:', request);
    
    // Show success message
    const formEl = document.getElementById('workRequestForm');
    const successEl = document.getElementById('formSuccess');
    
    if (formEl) formEl.style.display = 'none';
    if (successEl) successEl.style.display = 'block';
    
    // Close modal after delay
    setTimeout(() => {
        closeWorkRequestModal();
    }, 2000);
    
    return false;
}

// Utility: Format time ago
function formatTimeAgo(date) {
    const seconds = Math.floor((new Date() - date) / 1000);
    
    const intervals = {
        year: 31536000,
        month: 2592000,
        week: 604800,
        day: 86400,
        hour: 3600,
        minute: 60
    };
    
    for (const [unit, secondsInUnit] of Object.entries(intervals)) {
        const interval = Math.floor(seconds / secondsInUnit);
        if (interval >= 1) {
            return `${interval} ${unit}${interval > 1 ? 's' : ''} ago`;
        }
    }
    
    return 'just now';
}

// Utility: Escape HTML
function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Export functions for use in main dashboard
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        initFactoryView,
        openZoneModal,
        closeZoneModal,
        openWorkRequestModal,
        closeWorkRequestModal,
        submitWorkRequest
    };
}
