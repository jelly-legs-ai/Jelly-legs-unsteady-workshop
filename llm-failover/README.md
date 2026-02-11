# OpenClaw LLM Failover System

## Overview
Automatic failover system that switches between LLM providers based on usage limits to ensure continuous operation.

## Current Configuration
- **Primary:** OpenRouter (100 requests/day)
- **Backup:** Google Gemini 2.5 Flash (unlimited)
- **Thresholds:** Warning at 80 requests, Critical at 95 requests
- **Fallback:** Automatic switch to Gemini when OpenRouter limit approached

## Integration
The system integrates directly with OpenClaw's execution pipeline to track LLM usage automatically.

## Status
Current provider: OpenRouter (free tier)
Usage tracking: Active
Failover: Enabled

## Manual Control
To manually switch providers:

```javascript
const { manualSwitch } = require('./llm-failover-tracker');
manualSwitch('gemini');  // Switch to Google Gemini
manualSwitch('openrouter');  // Switch back to OpenRouter
```

## Status Check
```javascript
const { getLLMStatus } = require('./llm-failover-tracker');
console.log(getLLMStatus());
```

## Configuration
Edit `llm-failover.js` to adjust:
- Provider models and limits
- Threshold values
- Switch logic
- Notification behavior

## Benefits
- **Reliability:** Never hit hard limits mid-workflow
- **Cost Control:** Stay within free tier limits
- **Performance:** Use high-capacity provider when needed
- **Transparency:** Full audit trail of all switches
