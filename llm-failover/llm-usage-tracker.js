/**
 * LLM Usage Tracker for Jelly-Legs
 * Tracks usage across providers and manages automatic failover
 */

const fs = require('fs');
const path = require('path');

class LLMUsageTracker {
  constructor() {
    this.workspace = process.env.OPENCLAW_WORKSPACE || '/home/rm_ga/.openclaw/workspace';
    this.stateFile = path.join(this.workspace, 'memory', 'llm-usage.json');
    this.state = this.loadState();
    
    this.providers = {
      openrouter: {
        name: 'OpenRouter',
        model: 'openrouter/openrouter/free',
        dailyLimit: 100,
        used: 0,
        lastReset: new Date().toDateString(),
        priority: 1
      },
      gemini: {
        name: 'Google Gemini 2.5 Flash',
        model: 'google/gemini-2.5-flash',
        dailyLimit: Infinity,
        used: 0,
        lastReset: new Date().toDateString(),
        priority: 2
      }
    };
    
    this.loadUsage();
  }
  
  loadState() {
    try {
      if (fs.existsSync(this.stateFile)) {
        return JSON.parse(fs.readFileSync(this.stateFile, 'utf8'));
      }
    } catch (e) {
      console.error('[LLM Tracker] Error loading state:', e);
    }
    return { providers: {}, currentProvider: 'openrouter', switches: [] };
  }
  
  loadUsage() {
    const today = new Date().toDateString();
    
    Object.keys(this.providers).forEach(key => {
      const persisted = this.state.providers[key] || { used: 0, lastReset: today };
      
      if (persisted.lastReset !== today) {
        this.providers[key].used = 0;
        this.state.providers[key] = { used: 0, lastReset: today };
      } else {
        this.providers[key].used = persisted.used;
      }
    });
    
    this.currentProvider = this.state.currentProvider || 'openrouter';
    this.switches = this.state.switches || [];
  }
  
  saveUsage() {
    try {
      const dir = path.dirname(this.stateFile);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      
      this.state.providers = this.providers;
      this.state.currentProvider = this.currentProvider;
      this.state.switches = this.switches;
      
      fs.writeFileSync(this.stateFile, JSON.stringify(this.state, null, 2));
    } catch (e) {
      console.error('[LLM Tracker] Error saving state:', e);
    }
  }
  
  recordRequest(provider = null) {
    if (!provider) {
      provider = this.currentProvider;
    }
    
    if (this.providers[provider]) {
      this.providers[provider].used++;
      console.log(`[LLM] ${this.providers[provider].name} request recorded (${this.providers[provider].used} today)`);
      this.saveUsage();
      this.evaluateSwitch();
    }
  }
  
  evaluateSwitch() {
    const openrouterUsed = this.providers.openrouter.used;
    const geminiUsed = this.providers.gemini.used;
    
    // Switch criteria
    if (this.currentProvider === 'openrouter') {
      if (openrouterUsed >= 95) {
        this.switchTo('gemini', 'OpenRouter critical limit (95+)');
      } else if (openrouterUsed >= 80) {
        this.switchTo('gemini', 'OpenRouter warning (80+)');
      }
    } else if (this.currentProvider === 'gemini') {
      if (openrouterUsed < 50) {
        this.switchTo('openrouter', 'OpenRouter capacity available (<50)');
      }
    }
  }
  
  switchTo(provider, reason) {
    if (this.currentProvider !== provider && this.providers[provider]) {
      const oldProvider = this.currentProvider;
      this.currentProvider = provider;
      
      const switchRecord = {
        from: oldProvider,
        to: provider,
        reason: reason,
        timestamp: new Date().toISOString(),
        usage: {
          openrouter: this.providers.openrouter.used,
          gemini: this.providers.gemini.used
        }
      };
      
      this.switches.push(switchRecord);
      console.log(`[LLM Failover] ${oldProvider} → ${provider}: ${reason}`);
      this.saveUsage();
      
      this.notifySwitch(switchRecord);
    }
  }
  
  notifySwitch(switchRecord) {
    try {
      const notification = `📊 LLM Failover: ${switchRecord.from} → ${switchRecord.to}\nReason: ${switchRecord.reason}\nUsage: OpenRouter=${switchRecord.usage.openrouter}/100, Gemini=${switchRecord.usage.gemini}/∞`;
      
      // Send notification via OpenClaw messaging system
      console.log(`[Notification] ${notification}`);
      
      // Also write to status file
      const statusFile = path.join(this.workspace, 'memory', 'llm-status.txt');
      fs.writeFileSync(statusFile, notification);
    } catch (e) {
      console.error('[LLM Failover] Notification error:', e);
    }
  }
  
  getCurrentModel() {
    return this.providers[this.currentProvider].model;
  }
  
  getStatus() {
    return {
      currentProvider: this.currentProvider,
      currentModel: this.getCurrentModel(),
      providers: Object.keys(this.providers).map(key => ({
        name: this.providers[key].name,
        model: this.providers[key].model,
        used: this.providers[key].used,
        limit: this.providers[key].dailyLimit,
        percentage: this.providers[key].dailyLimit === Infinity ? 0 : Math.round((this.providers[key].used / this.providers[key].dailyLimit) * 100)
      })),
      lastSwitch: this.switches.length > 0 ? this.switches[this.switches.length - 1] : null
    };
  }
  
  // Method for manual override
  manualSwitch(provider) {
    if (this.providers[provider]) {
      this.switchTo(provider, 'Manual admin switch');
    }
  }
}

module.exports = { LLMUsageTracker };
EOF
