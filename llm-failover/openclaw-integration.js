/**
 * OpenClaw LLM Failover Integration
 * Automatically tracks usage and switches models in OpenClaw
 */

const { LLMUsageTracker } = require('./llm-usage-tracker');
const tracker = new LLMUsageTracker();

// Integration with OpenClaw's execution system
async function trackLLMCall(context, next) {
  return async (ctx) => {
    const startTime = Date.now();
    let success = false;
    let provider = tracker.getCurrentProvider();
    
    try {
      // Execute the LLM call
      const result = await next(ctx);
      success = result && !result.error;
      return result;
    } finally {
      // Record the request regardless of outcome
      tracker.recordRequest(provider);
      
      const duration = Date.now() - startTime;
      console.log(`[LLM Integration] Provider: ${provider}, Success: ${success}, Duration: ${duration}ms`);
    }
  };
}

// Get current status
function getLLMStatus() {
  return tracker.getStatus();
}

// Manual switch (for admin control)
function manualSwitch(provider) {
  if (tracker.providers[provider]) {
    tracker.switchTo(provider, 'Manual admin switch');
    return true;
  }
  return false;
}

module.exports = {
  trackLLMCall,
  getLLMStatus,
  manualSwitch
};
EOF
