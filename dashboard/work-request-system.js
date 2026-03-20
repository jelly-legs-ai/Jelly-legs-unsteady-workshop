/**
 * Agent Work Request System
 * Allows agents to create work requests and delegate to other agents
 */

(function() {
  'use strict';

  class WorkRequestSystem {
    constructor() {
      this.agents = [];
      this.workRequests = [];
      this.currentRequestingAgent = null;
      this.modal = null;
      this.githubToken = localStorage.getItem('github_token') || '';
      this.repoOwner = 'jelly-legs-ai';
      this.repoName = 'Jelly-legs-unsteady-workshop';
      
      this.init();
    }

    async init() {
      await this.loadAgents();
      this.loadWorkRequests();
      this.createModal();
      this.injectStyles();
      this.addRequestButton();
      this.renderHistorySection();
      console.log('🔧 Work Request System initialized');
    }

    async loadAgents() {
      try {
        const response = await fetch('data/dashboard-state.json');
        const data = await response.json();
        this.agents = Object.values(data.agents).map(agent => ({
          id: agent.id,
          name: agent.name,
          emoji: agent.emoji,
          role: agent.role,
          color: agent.color,
          status: agent.status
        }));
        console.log(`✅ Loaded ${this.agents.length} agents`);
      } catch (error) {
        console.error('❌ Failed to load agents:', error);
        // Fallback to default agents
        this.agents = this.getDefaultAgents();
      }
    }

    getDefaultAgents() {
      return [
        { id: 'jelly-legs', name: 'Jelly-Legs', emoji: '🪼', role: 'Marketing Commander', color: '#ff3333', status: 'idle' },
        { id: 'data-diver', name: 'Data-Diver', emoji: '🤿', role: 'Research Lead', color: '#3366ff', status: 'idle' },
        { id: 'pattern-seeker', name: 'Pattern-Seeker', emoji: '🔮', role: 'Trend Analyst', color: '#9933ff', status: 'idle' },
        { id: 'sketch-bot', name: 'Sketch-Bot', emoji: '🎨', role: 'Design Architect', color: '#ff66cc', status: 'idle' },
        { id: 'voice-weaver', name: 'Voice-Weaver', emoji: '🎭', role: 'Brand Voice', color: '#ff9933', status: 'idle' },
        { id: 'hook-maker', name: 'Hook-Maker', emoji: '🪝', role: 'Viral Engineer', color: '#ffcc00', status: 'idle' },
        { id: 'build-bot', name: 'Build-Bot', emoji: '⚙️', role: 'System Developer', color: '#33cc33', status: 'idle' },
        { id: 'pipe-layer', name: 'Pipe-Layer', emoji: '🧩', role: 'Pipeline Engineer', color: '#33cccc', status: 'idle' },
        { id: 'code-crafter', name: 'Code-Crafter', emoji: '💻', role: 'Implementation', color: '#66ff66', status: 'idle' },
        { id: 'shield-bot', name: 'Shield-Bot', emoji: '🛡️', role: 'Security Guard', color: '#999999', status: 'idle' },
        { id: 'map-maker', name: 'Map-Maker', emoji: '🗺️', role: 'Strategy Lead', color: '#6666ff', status: 'idle' },
        { id: 'launch-pad', name: 'Launch-Pad', emoji: '🚀', role: 'Deployment Chief', color: '#ffcc00', status: 'idle' }
      ];
    }

    loadWorkRequests() {
      const saved = localStorage.getItem('work_requests');
      if (saved) {
        try {
          this.workRequests = JSON.parse(saved);
        } catch (e) {
          this.workRequests = [];
        }
      }
    }

    saveWorkRequests() {
      localStorage.setItem('work_requests', JSON.stringify(this.workRequests));
    }

    injectStyles() {
      const style = document.createElement('style');
      style.textContent = `
        /* Work Request Button */
        .work-request-system-btn {
          position: fixed;
          top: 30px;
          right: 200px;
          background: linear-gradient(135deg, #33cc33 0%, #22aa22 100%);
          color: white;
          border: none;
          border-radius: 50px;
          padding: 15px 30px;
          font-size: 16px;
          font-weight: 700;
          cursor: pointer;
          box-shadow: 0 6px 20px rgba(51, 204, 51, 0.4);
          z-index: 1000;
          transition: all 0.3s ease;
          backdrop-filter: blur(10px);
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .work-request-system-btn:hover {
          transform: translateY(-3px);
          box-shadow: 0 8px 30px rgba(51, 204, 51, 0.6), 0 0 30px rgba(51, 204, 51, 0.3);
          background: linear-gradient(135deg, #22aa22 0%, #33cc33 100%);
        }
        
        @media (max-width: 768px) {
          .work-request-system-btn {
            right: 20px;
            top: 90px;
            padding: 12px 20px;
            font-size: 14px;
          }
        }

        /* Modal */
        .work-request-modal-overlay {
          display: none;
          position: fixed;
          top: 0;
          left: 0;
          width: 100%;
          height: 100%;
          background: rgba(0, 0, 0, 0.8);
          backdrop-filter: blur(10px);
          z-index: 2000;
          justify-content: center;
          align-items: center;
        }
        
        .work-request-modal-overlay.active {
          display: flex;
        }
        
        .work-request-modal-content {
          background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
          border: 2px solid rgba(255, 51, 51, 0.3);
          border-radius: 20px;
          padding: 40px;
          width: 90%;
          max-width: 600px;
          max-height: 90vh;
          overflow-y: auto;
          box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), 0 0 40px rgba(255, 51, 51, 0.2);
        }
        
        .work-request-modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 30px;
          padding-bottom: 20px;
          border-bottom: 2px solid rgba(255, 51, 51, 0.2);
        }
        
        .work-request-modal-title {
          color: #ff3333;
          font-size: 1.8rem;
          font-weight: 700;
          text-shadow: 0 0 10px rgba(255, 51, 51, 0.3);
          margin: 0;
        }
        
        .work-request-modal-close {
          background: none;
          border: none;
          color: #666;
          font-size: 28px;
          cursor: pointer;
          transition: color 0.3s;
        }
        
        .work-request-modal-close:hover {
          color: #ff3333;
        }
        
        .work-request-form-group {
          margin-bottom: 25px;
        }
        
        .work-request-form-label {
          display: block;
          color: #ff6666;
          font-weight: 600;
          margin-bottom: 10px;
          font-size: 0.95rem;
        }
        
        .work-request-form-input,
        .work-request-form-textarea,
        .work-request-form-select {
          width: 100%;
          padding: 15px;
          background: rgba(0, 0, 0, 0.4);
          border: 2px solid rgba(255, 51, 51, 0.2);
          border-radius: 12px;
          color: #fff;
          font-size: 1rem;
          transition: all 0.3s;
          font-family: inherit;
        }
        
        .work-request-form-input:focus,
        .work-request-form-textarea:focus,
        .work-request-form-select:focus {
          outline: none;
          border-color: rgba(255, 51, 51, 0.6);
          box-shadow: 0 0 15px rgba(255, 51, 51, 0.2);
          background: rgba(0, 0, 0, 0.5);
        }
        
        .work-request-form-textarea {
          min-height: 120px;
          resize: vertical;
        }
        
        .work-request-form-select {
          cursor: pointer;
          appearance: none;
          background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23ff3333' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 15px center;
          padding-right: 40px;
        }
        
        .work-request-priority-options {
          display: flex;
          gap: 15px;
          flex-wrap: wrap;
        }
        
        .work-request-priority-option {
          flex: 1;
          min-width: 100px;
        }
        
        .work-request-priority-radio {
          display: none;
        }
        
        .work-request-priority-label {
          display: block;
          padding: 15px 20px;
          text-align: center;
          border: 2px solid rgba(255, 255, 255, 0.1);
          border-radius: 12px;
          cursor: pointer;
          transition: all 0.3s;
          font-weight: 600;
        }
        
        .priority-p0 { --priority-color: #ff3333; }
        .priority-p1 { --priority-color: #ff9933; }
        .priority-p2 { --priority-color: #ffcc00; }
        .priority-p3 { --priority-color: #66ccff; }
        
        .priority-p0 .work-request-priority-label { border-color: rgba(255, 51, 51, 0.3); color: #ff6666; }
        .priority-p1 .work-request-priority-label { border-color: rgba(255, 153, 51, 0.3); color: #ffaa66; }
        .priority-p2 .work-request-priority-label { border-color: rgba(255, 204, 0, 0.3); color: #ffdd44; }
        .priority-p3 .work-request-priority-label { border-color: rgba(102, 204, 255, 0.3); color: #88ddff; }
        
        .work-request-priority-radio:checked + .work-request-priority-label {
          background: var(--priority-color);
          border-color: var(--priority-color);
          color: #fff;
          box-shadow: 0 0 20px var(--priority-color);
        }
        
        .work-request-submit-btn {
          width: 100%;
          padding: 18px 30px;
          background: linear-gradient(135deg, #ff3333 0%, #cc0000 100%);
          border: none;
          border-radius: 12px;
          color: #fff;
          font-size: 1.1rem;
          font-weight: 700;
          cursor: pointer;
          transition: all 0.3s;
          box-shadow: 0 6px 20px rgba(255, 51, 51, 0.4);
        }
        
        .work-request-submit-btn:hover:not(:disabled) {
          transform: translateY(-3px);
          box-shadow: 0 8px 30px rgba(255, 51, 51, 0.6), 0 0 30px rgba(255, 51, 51, 0.3);
          background: linear-gradient(135deg, #cc0000 0%, #ff3333 100%);
        }
        
        .work-request-submit-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
        
        .work-request-submit-btn.loading {
          position: relative;
          color: transparent;
        }
        
        .work-request-submit-btn.loading::after {
          content: '';
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          width: 24px;
          height: 24px;
          border: 3px solid rgba(255, 255, 255, 0.3);
          border-top-color: #fff;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        
        @keyframes spin {
          to { transform: translate(-50%, -50%) rotate(360deg); }
        }
        
        /* Notification System */
        .work-request-notification-container {
          position: fixed;
          top: 100px;
          right: 30px;
          z-index: 3000;
          display: flex;
          flex-direction: column;
          gap: 15px;
        }
        
        .work-request-notification {
          background: linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%);
          border-left: 4px solid #33cc33;
          border-radius: 12px;
          padding: 20px 25px;
          box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
          animation: slideInRight 0.4s ease;
          max-width: 400px;
          position: relative;
        }
        
        .work-request-notification.error { border-left-color: #ff3333; }
        .work-request-notification.warning { border-left-color: #ffcc00; }
        .work-request-notification.info { border-left-color: #66ccff; }
        
        @keyframes slideInRight {
          from { transform: translateX(100%); opacity: 0; }
          to { transform: translateX(0); opacity: 1; }
        }
        
        @keyframes slideOutRight {
          from { transform: translateX(0); opacity: 1; }
          to { transform: translateX(100%); opacity: 0; }
        }
        
        .work-request-notification.removing {
          animation: slideOutRight 0.3s ease forwards;
        }
        
        .work-request-notification-title {
          color: #fff;
          font-weight: 700;
          margin-bottom: 8px;
          display: flex;
          align-items: center;
          gap: 10px;
        }
        
        .work-request-notification-message {
          color: #b0b0b0;
          font-size: 0.9rem;
          line-height: 1.5;
        }
        
        .work-request-notification-close {
          position: absolute;
          top: 10px;
          right: 10px;
          background: none;
          border: none;
          color: #666;
          font-size: 18px;
          cursor: pointer;
          transition: color 0.3s;
        }
        
        .work-request-notification-close:hover {
          color: #fff;
        }
        
        /* Work Request History Section */
        .work-request-history-section {
          position: fixed;
          bottom: 80px;
          right: 30px;
          width: 380px;
          max-height: 400px;
          background: rgba(26, 26, 26, 0.95);
          backdrop-filter: blur(20px);
          border: 2px solid rgba(255, 51, 51, 0.2);
          border-radius: 16px;
          box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
          z-index: 500;
          overflow: hidden;
          transition: all 0.3s ease;
        }
        
        .work-request-history-section.collapsed {
          max-height: 50px;
        }
        
        .work-request-history-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 15px 20px;
          background: rgba(255, 51, 51, 0.1);
          border-bottom: 1px solid rgba(255, 51, 51, 0.2);
          cursor: pointer;
        }
        
        .work-request-history-title {
          color: #ff3333;
          font-size: 14px;
          font-weight: 700;
          margin: 0;
          text-shadow: 0 0 10px rgba(255, 51, 51, 0.3);
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .work-request-history-count {
          background: rgba(255, 51, 51, 0.2);
          color: #ff6666;
          padding: 2px 10px;
          border-radius: 12px;
          font-weight: 600;
          font-size: 11px;
        }
        
        .work-request-history-toggle {
          background: none;
          border: none;
          color: #666;
          font-size: 20px;
          cursor: pointer;
          transition: transform 0.3s ease;
        }
        
        .work-request-history-section.collapsed .work-request-history-toggle {
          transform: rotate(-90deg);
        }
        
        .work-request-history-list {
          padding: 15px 20px;
          overflow-y: auto;
          max-height: 320px;
        }
        
        .work-request-history-item {
          background: rgba(51, 51, 51, 0.8);
          border-radius: 10px;
          padding: 15px;
          margin-bottom: 12px;
          border-left: 3px solid;
          transition: all 0.3s ease;
        }
        
        .work-request-history-item:hover {
          transform: translateX(3px);
          background: rgba(51, 51, 51, 1);
        }
        
        .work-request-history-item.p0 { border-left-color: #ff3333; }
        .work-request-history-item.p1 { border-left-color: #ff9933; }
        .work-request-history-item.p2 { border-left-color: #ffcc00; }
        .work-request-history-item.p3 { border-left-color: #66ccff; }
        
        .work-request-history-item-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 8px;
        }
        
        .work-request-history-item-title {
          color: #fff;
          font-weight: 600;
          font-size: 13px;
          margin: 0;
          flex: 1;
          margin-right: 10px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
        
        .work-request-history-item-priority {
          padding: 2px 8px;
          border-radius: 10px;
          font-size: 10px;
          font-weight: 700;
          text-transform: uppercase;
        }
        
        .work-request-history-item.p0 .work-request-history-item-priority { background: rgba(255, 51, 51, 0.2); color: #ff6666; }
        .work-request-history-item.p1 .work-request-history-item-priority { background: rgba(255, 153, 51, 0.2); color: #ffaa66; }
        .work-request-history-item.p2 .work-request-history-item-priority { background: rgba(255, 204, 0, 0.2); color: #ffdd44; }
        .work-request-history-item.p3 .work-request-history-item-priority { background: rgba(102, 204, 255, 0.2); color: #88ddff; }
        
        .work-request-history-item-meta {
          display: flex;
          gap: 12px;
          color: #888;
          font-size: 11px;
          flex-wrap: wrap;
          align-items: center;
        }
        
        .work-request-history-item-status {
          display: inline-flex;
          align-items: center;
          gap: 4px;
          padding: 2px 8px;
          border-radius: 10px;
          font-size: 10px;
          font-weight: 600;
        }
        
        .work-request-history-item-status.pending { background: rgba(255, 204, 0, 0.2); color: #ffdd44; }
        .work-request-history-item-status.created { background: rgba(51, 204, 51, 0.2); color: #66ff66; }
        .work-request-history-item-status.failed { background: rgba(255, 51, 51, 0.2); color: #ff6666; }
        
        .work-request-history-empty {
          text-align: center;
          padding: 30px;
          color: #666;
          font-size: 13px;
        }
        
        @media (max-width: 768px) {
          .work-request-modal-content {
            padding: 25px;
            margin: 15px;
          }
          
          .work-request-modal-title {
            font-size: 1.4rem;
          }
          
          .work-request-priority-options {
            flex-direction: column;
          }
          
          .work-request-notification-container {
            right: 15px;
            left: 15px;
            top: 80px;
          }
          
          .work-request-notification {
            max-width: 100%;
          }
          
          .work-request-history-section {
            right: 10px;
            left: 10px;
            width: auto;
            bottom: 70px;
          }
        }
      `;
      document.head.appendChild(style);
    }

    addRequestButton() {
      const btn = document.createElement('button');
      btn.className = 'work-request-system-btn';
      btn.innerHTML = '<span style="font-size: 20px;">📋</span> Request Work';
      btn.onclick = () => this.openModal();
      document.body.appendChild(btn);
    }

    createModal() {
      const modal = document.createElement('div');
      modal.className = 'work-request-modal-overlay';
      modal.id = 'workRequestModal';
      
      modal.innerHTML = `
        <div class="work-request-modal-content">
          <div class="work-request-modal-header">
            <h2 class="work-request-modal-title">📋 Request Work</h2>
            <button class="work-request-modal-close" onclick="workRequestSystem.closeModal()">&times;</button>
          </div>
          
          <form id="workRequestForm" onsubmit="event.preventDefault(); workRequestSystem.handleSubmit(event)">
            <div class="work-request-form-group">
              <label class="work-request-form-label">Requesting Agent *</label>
              <select class="work-request-form-select" id="requestingAgent" required>
                <option value="">Select your agent...</option>
                ${this.agents.map(agent => `
                  <option value="${agent.id}">
                    ${agent.emoji} ${agent.name} - ${agent.role}
                  </option>
                `).join('')}
              </select>
            </div>
            
            <div class="work-request-form-group">
              <label class="work-request-form-label">Target Agent *</label>
              <select class="work-request-form-select" id="targetAgent" required>
                <option value="">Select target agent...</option>
                ${this.agents.map(agent => `
                  <option value="${agent.id}">
                    ${agent.emoji} ${agent.name} - ${agent.role}
                  </option>
                `).join('')}
              </select>
            </div>
            
            <div class="work-request-form-group">
              <label class="work-request-form-label">Task Title *</label>
              <input type="text" class="work-request-form-input" id="taskTitle" 
                     placeholder="e.g., Implement user authentication system" required
                     maxlength="100">
            </div>
            
            <div class="work-request-form-group">
              <label class="work-request-form-label">Task Description *</label>
              <textarea class="work-request-form-textarea" id="taskDescription" 
                        placeholder="Describe the work needed, acceptance criteria, and any relevant details..."
                        required></textarea>
            </div>
            
            <div class="work-request-form-group">
              <label class="work-request-form-label">Priority *</label>
              <div class="work-request-priority-options">
                <div class="work-request-priority-option priority-p0">
                  <input type="radio" name="priority" id="priorityP0" value="P0" class="work-request-priority-radio" required>
                  <label for="priorityP0" class="work-request-priority-label">
                    🔥 P0<br>
                    <small>Critical</small>
                  </label>
                </div>
                <div class="work-request-priority-option priority-p1">
                  <input type="radio" name="priority" id="priorityP1" value="P1" class="work-request-priority-radio" required>
                  <label for="priorityP1" class="work-request-priority-label">
                    ⚡ P1<br>
                    <small>High</small>
                  </label>
                </div>
                <div class="work-request-priority-option priority-p2">
                  <input type="radio" name="priority" id="priorityP2" value="P2" class="work-request-priority-radio" required>
                  <label for="priorityP2" class="work-request-priority-label">
                    📌 P2<br>
                    <small>Normal</small>
                  </label>
                </div>
                <div class="work-request-priority-option priority-p3">
                  <input type="radio" name="priority" id="priorityP3" value="P3" class="work-request-priority-radio" required>
                  <label for="priorityP3" class="work-request-priority-label">
                    💡 P3<br>
                    <small>Low</small>
                  </label>
                </div>
              </div>
            </div>
            
            <div class="work-request-form-group">
              <label class="work-request-form-label">GitHub Token *</label>
              <input type="password" class="work-request-form-input" id="githubToken" 
                     placeholder="ghp_xxxxxxxxxxxx" value="${this.githubToken}" required>
              <small style="color: #666; display: block; margin-top: 8px; font-size: 11px;">
                Your token is stored locally in browser storage
              </small>
            </div>
            
            <button type="submit" class="work-request-submit-btn" id="submitBtn">
              Create Work Request
            </button>
          </form>
        </div>
      `;
      
      // Close modal on backdrop click
      modal.addEventListener('click', (e) => {
        if (e.target === modal) {
          this.closeModal();
        }
      });
      
      document.body.appendChild(modal);
      this.modal = modal;
    }

    openModal() {
      if (this.modal) {
        this.modal.classList.add('active');
        document.body.style.overflow = 'hidden';
        
        // Pre-fill GitHub token if available
        const tokenInput = document.getElementById('githubToken');
        if (tokenInput && this.githubToken) {
          tokenInput.value = this.githubToken;
        }
      }
    }

    closeModal() {
      if (this.modal) {
        this.modal.classList.remove('active');
        document.body.style.overflow = '';
        this.resetForm();
      }
    }

    resetForm() {
      const form = document.getElementById('workRequestForm');
      if (form) {
        form.reset();
      }
      const submitBtn = document.getElementById('submitBtn');
      if (submitBtn) {
        submitBtn.disabled = false;
        submitBtn.classList.remove('loading');
        submitBtn.textContent = 'Create Work Request';
      }
    }

    async handleSubmit(event) {
      const submitBtn = document.getElementById('submitBtn');
      submitBtn.disabled = true;
      submitBtn.classList.add('loading');
      
      const formData = {
        requestingAgent: document.getElementById('requestingAgent').value,
        targetAgent: document.getElementById('targetAgent').value,
        title: document.getElementById('taskTitle').value.trim(),
        description: document.getElementById('taskDescription').value.trim(),
        priority: document.querySelector('input[name="priority"]:checked')?.value || 'P2',
        githubToken: document.getElementById('githubToken').value.trim()
      };
      
      // Use provided token or fall back to localStorage
      const token = formData.githubToken || this.githubToken;
      
      // Save GitHub token if provided
      if (formData.githubToken) {
        this.githubToken = formData.githubToken;
        localStorage.setItem('github_token', this.githubToken);
      }
      
      // Validate
      if (formData.requestingAgent === formData.targetAgent) {
        this.showNotification('Error', 'You cannot request work from yourself!', 'error');
        submitBtn.disabled = false;
        submitBtn.classList.remove('loading');
        return;
      }
      
      // Validate token exists
      if (!token) {
        this.showNotification('Error', 'GitHub token is required. Please enter your token.', 'error');
        submitBtn.disabled = false;
        submitBtn.classList.remove('loading');
        return;
      }
      
      try {
        // Create GitHub issue
        const issue = await this.createGitHubIssue(formData, token);
        
        // Create work request object
        const zone = this.getZoneFromPriority(formData.priority);
        const workRequest = {
          id: `wr-${Date.now()}`,
          ...formData,
          requestingAgentName: this.agents.find(a => a.id === formData.requestingAgent)?.name || formData.requestingAgent,
          targetAgentName: this.agents.find(a => a.id === formData.targetAgent)?.name || formData.targetAgent,
          requestingAgentEmoji: this.agents.find(a => a.id === formData.requestingAgent)?.emoji || '🤖',
          targetAgentEmoji: this.agents.find(a => a.id === formData.targetAgent)?.emoji || '🤖',
          status: 'created',
          zone: zone,
          createdAt: new Date().toISOString(),
          githubIssueNumber: issue.number,
          githubIssueUrl: issue.html_url
        };
        
        // Save to history
        this.workRequests.unshift(workRequest);
        this.saveWorkRequests();
        
        // Show success message with link
        this.showNotification(
          '✅ Work Request Created',
          `GitHub issue #${issue.number} created successfully! <a href="${issue.html_url}" target="_blank" style="color: #66ff66;">View Issue</a>`,
          'success'
        );
        
        // Update UI
        this.renderHistorySection();
        
        // Clear form and close modal on success
        this.closeModal();
        
        // Simulate notification to target agent
        this.notifyTargetAgent(workRequest);
        
      } catch (error) {
        console.error('GitHub API error:', error);
        this.showNotification(
          '❌ Error',
          error.message || 'Failed to create GitHub issue. Please check your token and try again.',
          'error'
        );
        submitBtn.disabled = false;
        submitBtn.classList.remove('loading');
      }
    }

    async createGitHubIssue(formData, token) {
      const url = `https://api.github.com/repos/${this.repoOwner}/${this.repoName}/issues`;
      
      // Get agent names for display
      const requestingAgentName = this.agents.find(a => a.id === formData.requestingAgent)?.name || formData.requestingAgent;
      const targetAgentName = this.agents.find(a => a.id === formData.targetAgent)?.name || formData.targetAgent;
      
      // Build labels array per Sprint 2 spec
      const labels = [
        'build',
        'work-request',
        formData.priority.toLowerCase(),
        `agent-${formData.targetAgent}`
      ];
      
      // Build issue body per Sprint 2 spec
      const body = `**Requested by:** Agent ${requestingAgentName}
**Target Agent:** ${targetAgentName}
**Priority:** ${formData.priority}
**Zone:** ${this.getZoneFromPriority(formData.priority)}

${formData.description}`;

      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Authorization': `token ${token}`,
          'Accept': 'application/vnd.github.v3+json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          title: `[WORK REQUEST] ${formData.title}`,
          body: body,
          labels: labels
        })
      });
      
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || `GitHub API error: ${response.status}`);
      }
      
      return await response.json();
    }

    getZoneFromPriority(priority) {
      const zoneMap = {
        'P0': 'critical-zone',
        'P1': 'urgent-zone',
        'P2': 'standard-zone',
        'P3': 'backlog-zone'
      };
      return zoneMap[priority] || 'standard-zone';
    }

    notifyTargetAgent(workRequest) {
      console.log(`🔔 Notification sent to ${workRequest.targetAgentName}: New work request from ${workRequest.requestingAgentName}`);
      
      // Dispatch custom event that the dashboard can listen for
      const event = new CustomEvent('workRequestCreated', {
        detail: workRequest
      });
      document.dispatchEvent(event);
    }

    showNotification(title, message, type = 'info') {
      let container = document.getElementById('workRequestNotificationContainer');
      if (!container) {
        container = document.createElement('div');
        container.id = 'workRequestNotificationContainer';
        container.className = 'work-request-notification-container';
        document.body.appendChild(container);
      }
      
      const notification = document.createElement('div');
      notification.className = `work-request-notification ${type}`;
      
      const icon = type === 'success' ? '✅' : type === 'error' ? '❌' : type === 'warning' ? '⚠️' : 'ℹ️';
      
      notification.innerHTML = `
        <button class="work-request-notification-close" onclick="this.parentElement.remove()">&times;</button>
        <div class="work-request-notification-title">${icon} ${title}</div>
        <div class="work-request-notification-message">${message}</div>
      `;
      
      container.appendChild(notification);
      
      // Auto-remove after 5 seconds
      setTimeout(() => {
        notification.classList.add('removing');
        setTimeout(() => notification.remove(), 300);
      }, 5000);
    }

    renderHistorySection() {
      // Find or create the history section
      let historySection = document.getElementById('workRequestHistorySection');
      
      if (!historySection) {
        historySection = document.createElement('div');
        historySection.id = 'workRequestHistorySection';
        historySection.className = 'work-request-history-section collapsed';
        document.body.appendChild(historySection);
      }
      
      const count = this.workRequests.length;
      const displayCount = count > 99 ? '99+' : count;
      
      if (this.workRequests.length === 0) {
        historySection.innerHTML = `
          <div class="work-request-history-header" onclick="workRequestSystem.toggleHistory()">
            <h3 class="work-request-history-title">
              📚 Work Request History
              <span class="work-request-history-count">0</span>
            </h3>
            <button class="work-request-history-toggle">▼</button>
          </div>
          <div class="work-request-history-list" style="display: none;">
            <div class="work-request-history-empty">No work requests yet. Click "Request Work" to create your first request!</div>
          </div>
        `;
        return;
      }
      
      const requestsHtml = this.workRequests.slice(0, 10).map(req => `
        <div class="work-request-history-item ${req.priority.toLowerCase()}" data-status="${req.status}" data-priority="${req.priority}">
          <div class="work-request-history-item-header">
            <h4 class="work-request-history-item-title" title="${req.title}">${req.title}</h4>
            <span class="work-request-history-item-priority">${req.priority}</span>
          </div>
          <div class="work-request-history-item-meta">
            <span>${req.requestingAgentEmoji} → ${req.targetAgentEmoji}</span>
            <span>${new Date(req.createdAt).toLocaleDateString()}</span>
            <span class="work-request-history-item-status status-${req.status}">
              ${req.status === 'created' ? '✅ Created' : req.status === 'pending' ? '⏳ Pending' : '❌ Failed'}
            </span>
          </div>
          ${req.githubIssueNumber ? `
            <div style="margin-top: 8px;">
              <a href="${req.githubIssueUrl}" target="_blank" style="color: #66ccff; font-size: 11px; text-decoration: none;">
                Issue #${req.githubIssueNumber} →
              </a>
            </div>
          ` : ''}
        </div>
      `).join('');
      
      historySection.innerHTML = `
        <div class="work-request-history-header" onclick="workRequestSystem.toggleHistory()">
          <h3 class="work-request-history-title">
            📚 Work Request History
            <span class="work-request-history-count">${displayCount}</span>
          </h3>
          <button class="work-request-history-toggle">▼</button>
        </div>
        <div class="work-request-history-list" style="display: none;">
          ${requestsHtml}
          ${count > 10 ? `<div style="text-align: center; color: #666; font-size: 11px; margin-top: 10px;">+${count - 10} more in storage</div>` : ''}
        </div>
      `;
    }

    toggleHistory() {
      const section = document.getElementById('workRequestHistorySection');
      if (section) {
        section.classList.toggle('collapsed');
        const list = section.querySelector('.work-request-history-list');
        if (list) {
          list.style.display = section.classList.contains('collapsed') ? 'none' : 'block';
        }
      }
    }

    // API Methods for external integration
    getWorkRequests() {
      return this.workRequests;
    }

    getWorkRequestsForAgent(agentId) {
      return this.workRequests.filter(req => 
        req.targetAgent === agentId || req.requestingAgent === agentId
      );
    }

    getPendingWorkRequests(agentId = null) {
      if (agentId) {
        return this.workRequests.filter(req => 
          req.targetAgent === agentId && req.status === 'pending'
        );
      }
      return this.workRequests.filter(req => req.status === 'pending');
    }
  }

  // Initialize the work request system
  let workRequestSystem;

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      workRequestSystem = new WorkRequestSystem();
      window.workRequestSystem = workRequestSystem;
    });
  } else {
    workRequestSystem = new WorkRequestSystem();
    window.workRequestSystem = workRequestSystem;
  }
})();