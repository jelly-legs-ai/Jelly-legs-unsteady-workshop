/**
 * LLM Failover System for OpenClaw
 * Automatically switches between providers based on usage limits
 */

const fs = require('fs');
const path = require('path');

class LLMFailover {
  constructor() {
    this.stateFile = path.join(process.env.OPENCLAW_WORKSPACE || '/home/rm_ga/.openclaw/workspace', 'memory', 'llm-failover-state.json');
    this.state = this.loadState();
    this.providers = {
      openrouter: {
        name: 'OpenRouter',
        model: 'openrouter/free',
        dailyLimit: 100,
        costPerRequest: 0,
        priority: 1
      },
      gemini: {
        name: 'Google Gemini 2.5 Flash',
        model: 'google/gemini-2.5-flash',
        dailyLimit: Infinity, // Actually 60/min = ~86,400/day
        costPerRequest: 0,
        priority: 2,
        rateLimitPerMinute: 60
      }
    };
  }

  loadState() {
    try {
      if (fs.existsSync(this.stateFile)) {
        const data = fs.readFileSync(this.stateFile, 'utf8');
        return JSON.parse(data);
      }
    } catch (error) {
      console.error('Failed to load LLM failover state:', error);
    }
    return {
      openrouter: { usedToday: 0, lastReset: new Date().toDateString() },
      gemini: { usedToday: 0, lastReset: new Date().toDateString() },
      currentProvider: 'openrouter',
      lastSwitch: null,
      switchCount: 0
    };
  }

  saveState() {
    try {
      const dir = path.dirname(this.stateFile);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      fs.writeFileSync(this.stateFile, JSON.stringify(this.state, null, 2));
    } catch (error) {
      console.error('Failed to save LLM failover state:', error);
    }
  }

  resetDailyCounters() {
    const today = new Date().toDateString();
    Object.keys(this.providers).forEach(providerId => {
      if (this.state[providerId].lastReset !== today) {
        this.state[providerId].usedToday = 0;
        this.state[providerId].lastReset = today;
      }
    });
  }

  getCurrentProvider() {
    this.resetDailyCounters();

    // Check if we need to switch based on limits
    const openrouterUsage = this.state.openrouter.usedToday;
    const geminiUsage = this.state.gemini.usedToday;

    // Define thresholds (switch before hitting limit to avoid failures)
    const OPENROUTER_WARNING_THRESHOLD = 80; // 80 requests
    const OPENROUTER_CRITICAL_THRESHOLD = 95; // 95 requests

    // If currently on OpenRouter and approaching limit, switch to Gemini
    if (this.state.currentProvider === 'openrouter') {
      if (openrouterUsage >= OPENROUTER_CRITICAL_THRESHOLD) {
        this.switchProvider('gemini', 'OpenRouter limit critically reached');
      } else if (openrouterUsage >= OPENROUTER_WARNING_THRESHOLD) {
        this.switchProvider('gemini', 'OpenRouter approaching limit');
      }
    }

    // If on Gemini and OpenRouter has capacity, consider switching back
    if (this.state.currentProvider === 'gemini') {
      if (openrouterUsage < 50) { // OpenRouter has plenty of room
        this.switchProvider('openrouter', 'OpenRouter capacity restored');
      }
    }

    return this.state.currentProvider;
  }

  switchProvider(newProvider, reason) {
    if (this.state.currentProvider !== newProvider) {
      const oldProvider = this.state.currentProvider;
      this.state.currentProvider = newProvider;
      this.state.lastSwitch = new Date().toISOString();
      this.state.switchCount++;
      this.state.lastSwitchReason = reason;

      console.log(`[LLM Failover] Switching from ${oldProvider} to ${newProvider}: ${reason}`);
      this.saveState();

      // Optional: Send notification
      this.sendNotification(`Switched LLM provider: ${oldProvider} → ${newProvider} (${reason})`);
    }
  }

  recordRequest(provider, success = true) {
    this.resetDailyCounters();
    if (this.state[provider]) {
      this.state[provider].usedToday++;
      this.state[provider].lastRequest = new Date().toISOString();
      this.state[provider].lastSuccess = success;
      this.saveState();
    }
  }

  getProviderInfo(providerId) {
    const provider = this.providers[providerId];
    const state = this.state[providerId];
    return {
      ...provider,
      usedToday: state.usedToday,
      remaining: provider.dailyLimit === Infinity ? Infinity : Math.max(0, provider.dailyLimit - state.usedToday),
      usagePercentage: provider.dailyLimit === Infinity ? 0 : Math.round((state.usedToday / provider.dailyLimit) * 100),
      lastRequest: state.lastRequest,
      isActive: this.state.currentProvider === providerId
    };
  }

  getStatus() {
    this.resetDailyCounters();
    return {
      currentProvider: this.state.currentProvider,
      providers: Object.keys(this.providers).map(id => this.getProviderInfo(id)),
      lastSwitch: this.state.lastSwitch,
      lastSwitchReason: this.state.lastSwitchReason,
      switchCount: this.state.switchCount
    };
  }

  sendNotification(message) {
    // Send notification via OpenClaw messaging system
    try {
      // You can integrate with your notification system here
      console.log(`[LLM Failover Notification] ${message}`);
    } catch (error) {
      console.error('Failed to send notification:', error);
    }
  }

  // Get the appropriate model for the current provider
  getCurrentModel() {
    const providerId = this.getCurrentProvider();
    return this.providers[providerId].model;
  }
}

module.exports = { LLMFailover };
```

Now let me create a usage tracking middleware that integrates with OpenClaw:

```bash
cd /home/rm_ga/.openclaw/workspace/repos/Jelly-legs-unsteady-workshop/llm-failover
```
