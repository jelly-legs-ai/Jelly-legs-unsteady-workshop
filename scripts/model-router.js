/**
 * Model Router for Ollama Pro
 * Manages 3-model concurrency with optimal agent-to-model assignments
 */

const MODEL_CONFIG = {
  // Fast tier - for quick tasks
  'glm-4.7-flash': {
    maxTokens: 8000,
    timeout: 120,
    bestFor: ['quick-edits', 'simple-fixes', 'deployments', 'content-generation']
  },
  
  // General tier - for balanced tasks
  'kimi-k2.5:cloud': {
    maxTokens: 16000,
    timeout: 300,
    bestFor: ['coordination', 'spawning', 'general-tasks', 'ui-design']
  },
  
  'minimax-m2.5:cloud': {
    maxTokens: 16000,
    timeout: 300,
    bestFor: ['research', 'analysis', 'documentation', 'content']
  },
  
  // Deep tier - for complex tasks
  'qwen3:8b': {
    maxTokens: 32000,
    timeout: 600,
    bestFor: ['coding', 'architecture', 'debugging', 'implementation']
  },
  
  'lfm2.5-thinking:1.2b': {
    maxTokens: 24000,
    timeout: 600,
    bestFor: ['planning', 'strategy', 'security-review', 'pattern-analysis']
  }
};

const AGENT_MODELS = {
  'jelly-legs': { model: 'kimi-k2.5:cloud', timeout: 300, priority: 1 },
  'data-diver': { model: 'minimax-m2.5:cloud', timeout: 300, priority: 2 },
  'pattern-seeker': { model: 'lfm2.5-thinking:1.2b', timeout: 600, priority: 3 },
  'sketch-bot': { model: 'kimi-k2.5:cloud', timeout: 300, priority: 2 },
  'voice-weaver': { model: 'minimax-m2.5:cloud', timeout: 300, priority: 2 },
  'hook-maker': { model: 'glm-4.7-flash', timeout: 120, priority: 1 },
  'build-bot': { model: 'qwen3:8b', timeout: 600, priority: 3 },
  'pipe-layer': { model: 'qwen3:8b', timeout: 600, priority: 3 },
  'code-crafter': { model: 'qwen3:8b', timeout: 600, priority: 3 },
  'shield-bot': { model: 'lfm2.5-thinking:1.2b', timeout: 600, priority: 3 },
  'map-maker': { model: 'lfm2.5-thinking:1.2b', timeout: 600, priority: 3 },
  'launch-pad': { model: 'glm-4.7-flash', timeout: 120, priority: 1 }
};

class ModelRouter {
  constructor() {
    this.activeSlots = new Map(); // slot -> {agent, model, startTime}
    this.maxSlots = 3; // Ollama Pro limit
    this.queue = [];
  }
  
  /**
   * Get optimal model for an agent
   */
  getModelForAgent(agentId) {
    const config = AGENT_MODELS[agentId];
    if (!config) {
      console.warn(`Unknown agent ${agentId}, defaulting to kimi-k2.5:cloud`);
      return AGENT_MODELS['jelly-legs'];
    }
    return config;
  }
  
  /**
   * Check if we can spawn a new agent
   */
  canSpawn() {
    return this.activeSlots.size < this.maxSlots;
  }
  
  /**
   * Get available slot
   */
  getAvailableSlot() {
    for (let i = 0; i < this.maxSlots; i++) {
      if (!this.activeSlots.has(i)) {
        return i;
      }
    }
    return null;
  }
  
  /**
   * Spawn an agent with optimal model
   */
  async spawnAgent(agentId, task, options = {}) {
    const agentConfig = this.getModelForAgent(agentId);
    const model = options.model || agentConfig.model;
    const timeout = options.timeout || agentConfig.timeout;
    
    if (!this.canSpawn()) {
      console.log(`⏳ All slots full, queueing ${agentId}`);
      this.queue.push({ agentId, task, options, queuedAt: Date.now() });
      return { status: 'queued', position: this.queue.length };
    }
    
    const slot = this.getAvailableSlot();
    const spawnConfig = {
      agentId,
      model,
      timeout,
      task,
      slot,
      startTime: Date.now()
    };
    
    this.activeSlots.set(slot, spawnConfig);
    
    console.log(`🤖 Spawned ${agentId} on ${model} (slot ${slot + 1}/3)`);
    
    // Return spawn config for sessions_spawn
    return {
      status: 'spawned',
      slot: slot + 1,
      model,
      timeout,
      config: spawnConfig
    };
  }
  
  /**
   * Release a slot when agent completes
   */
  releaseSlot(slot) {
    if (this.activeSlots.has(slot)) {
      const agent = this.activeSlots.get(slot);
      const duration = Date.now() - agent.startTime;
      console.log(`✅ ${agent.agentId} completed in ${Math.round(duration / 1000)}s (slot ${slot + 1})`);
      this.activeSlots.delete(slot);
      
      // Process queue
      this.processQueue();
    }
  }
  
  /**
   * Process queued agents
   */
  async processQueue() {
    if (this.queue.length === 0 || !this.canSpawn()) {
      return;
    }
    
    // Sort by priority (lower = higher priority)
    this.queue.sort((a, b) => {
      const aPriority = AGENT_MODELS[a.agentId]?.priority || 99;
      const bPriority = AGENT_MODELS[b.agentId]?.priority || 99;
      return aPriority - bPriority;
    });
    
    // Spawn next agent
    const next = this.queue.shift();
    if (next) {
      await this.spawnAgent(next.agentId, next.task, next.options);
    }
  }
  
  /**
   * Get current status
   */
  getStatus() {
    return {
      active: this.activeSlots.size,
      max: this.maxSlots,
      slots: Array.from(this.activeSlots.entries()).map(([slot, config]) => ({
        slot: slot + 1,
        agent: config.agentId,
        model: config.model,
        runtime: Math.round((Date.now() - config.startTime) / 1000)
      })),
      queued: this.queue.length
    };
  }
  
  /**
   * Batch spawn agents (up to 3)
   */
  async batchSpawn(assignments) {
    const results = [];
    
    for (const assignment of assignments.slice(0, 3)) {
      const result = await this.spawnAgent(
        assignment.agentId,
        assignment.task,
        assignment.options
      );
      results.push(result);
    }
    
    // Queue remaining
    for (const assignment of assignments.slice(3)) {
      this.queue.push({
        agentId: assignment.agentId,
        task: assignment.task,
        options: assignment.options,
        queuedAt: Date.now()
      });
    }
    
    return results;
  }
}

// Export singleton
module.exports = new ModelRouter();
