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
