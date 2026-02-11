/**
 * LLM Usage Tracker Middleware
 * Integrates with OpenClaw to track LLM usage and trigger failover
 */

const { LLMFailover } = require('./failover');
const failover = new LLMFailover();

class LLMTracker {
  constructor() {
    this.failover = failover;
  }

  /**
   * Intercept LLM calls to track usage
   * @param {Object} context - OpenClaw execution context
   * @param {Function} next - Next middleware function
   */
  async trackUsage(context, next) {
    const startTime = Date.now();
    let success = false;
    let provider = 'openrouter';

    try {
      // Determine which provider will be used
      provider = this.failover.getCurrentProvider();

      // Execute the LLM call (this would be wrapped around actual LLM execution)
      // For now, we'll just proceed and track the outcome
      const result = await next(context);

      success = result && !result.error;
      return result;
    } finally {
      const duration = Date.now() - startTime;

      // Record the request regardless of outcome
      this.failover.recordRequest(provider, success);

      // Log the usage
      console.log(`[LLM Tracker] Provider: ${provider}, Success: ${success}, Duration: ${duration}ms`);

      // Update kanban board if this was a research task
      if (context.session && context.session.label === 'main') {
        this.updateKanbanWithUsage();
      }
    }
  }

  /**
   * Update kanban board with current LLM usage statistics
   */
  updateKanbanWithUsage() {
    try {
      const status = this.failover.getStatus();
      const usageNote = `*LLM Usage*: ${status.currentProvider} active. ` +
        `OpenRouter: ${status.providers.find(p => p.name === 'OpenRouter').usedToday}/100, ` +
        `Gemini: ${status.providers.find(p => p.name.includes('Gemini')).usedToday}/∞`;

      // You could append this to kanban_board.md or another status file
      // For now, just log it
      console.log(`[Kanban Update] ${usageNote}`);
    } catch (error) {
      console.error('Failed to update kanban with usage:', error);
    }
  }

  /**
   * Get current failover status
   */
  getStatus() {
    return this.failover.getStatus();
  }

  /**
   * Manual provider switch (for admin control)
   */
  switchTo(providerId) {
    if (this.failover.providers[providerId]) {
      this.failover.switchProvider(providerId, 'Manual admin switch');
      return true;
    }
    return false;
  }
}

module.exports = { LLMTracker };
```

Now let me create an integration example:

```bash
cat > usage-example.js << 'EOF'
/**
 * Example: How to integrate LLM failover with OpenClaw
 */

const { LLMTracker } = require('./tracker');
const tracker = new LLMTracker();

// Example usage in an agent turn
async function processWithFailover(context, llmCall) {
  return tracker.trackUsage(context, async (ctx) => {
    // This is where the actual LLM call happens
    // You would call your configured LLM with the appropriate model
    const model = tracker.failover.getCurrentModel();
    console.log(`[LLM Call] Using model: ${model}`);

    // This is placeholder - replace with actual LLM call
    const result = await llmCall(ctx, { model });
    return result;
  });
}

// Example standalone usage
const status = tracker.getStatus();
console.log('Current LLM Status:', JSON.stringify(status, null, 2));

module.exports = { processWithFailover, tracker };
EOF
```

Now let me add these files to the repository and create documentation:

```bash
cd $HOME/.openclaw/workspace/repos/Jelly-legs-unsteady-workshop
```

Let me create comprehensive documentation for the failover system:

```bash
cat > llm-failover/README.md << 'EOF'
# LLM Failover System

## Overview
Automatic failover system that switches between LLM providers when usage limits are approached.

## Features
- **Automatic Switching:** Monitors usage and switches before hitting limits
- **Configurable Thresholds:** Warning (80%) and critical (95%) thresholds
- **Usage Tracking:** Daily counters with auto-reset
- **Smart Recovery:** Automatically switches back when capacity restored
- **Notification Integration:** Alerts on provider switches
- **Status Reporting:** Current provider and usage metrics

## Current Configuration

| Provider | Model | Daily Limit | Current Usage |
|-----------|-------|-------------|---------------|
| OpenRouter | openrouter/free | 100 requests | Tracked |
| Google Gemini 2.5 Flash | google/gemini-2.5-flash | ∞ (60/min) | Tracked |

## Usage

### Integration Example

```javascript
const { LLMTracker } = require('./llm-failover/tracker');
const tracker = new LLMTracker();

// In your agent turn processing
async function processWithFailover(context, llmCall) {
  return tracker.trackUsage(context, async (ctx) => {
    const model = tracker.failover.getCurrentModel();
    return await llmCall(ctx, { model });
  });
}
```

### Manual Control

```javascript
// Switch to specific provider
tracker.switchTo('gemini');  // Force Gemini
tracker.switchTo('openrouter');  // Force OpenRouter

// Get current status
const status = tracker.getStatus();
console.log(status.currentProvider);
console.log(status.providers);
```

## Thresholds

- **OpenRouter Warning:** 80 requests (80% of 100)
- **OpenRouter Critical:** 95 requests (95% of 100) - triggers automatic switch to Gemini
- **Gemini to OpenRouter:** Switches back when OpenRouter usage drops below 50 requests

## Monitoring

The system stores state in:
`memory/llm-failover-state.json`

Check status anytime:
```bash
cat memory/llm-failover-state.json
```

## Customization

Edit `llm-failover.js` to:
- Adjust threshold values
- Add more providers (Anthropic, OpenAI, etc.)
- Change switch logic
- Modify notification behavior

## Benefits

1. **Reliability:** Never hit hard limits mid-workflow
2. **Cost Control:** Stay within free tier
3. **Performance:** Use high-capacity provider when needed
4. **Transparency:** Full audit trail of all switches
```

Now let me add all these files to the repository:
