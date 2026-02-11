# OpenClaw-Compatible LLM Free Trials & Limits

## Current Configuration
Based on your openclaw.json, you have access to:

| Provider | Model | Free Tier | Rate Limits | Notes |
|----------|-------|-----------|-------------|-------|
| OpenRouter | openrouter/free | ✅ Yes | 100 requests/day | Multiple models via OpenRouter |
| Google | gemini-2.5-flash | ✅ Yes | 60 requests/minute | Native Google AI |
| Google | gemini-live-2.5-flash-preview-native-audio | ✅ Yes | 60 requests/minute | Audio capabilities |

## Detailed Breakdown

### 1. OpenRouter (openrouter/free)
- **Free Tier:** Yes, included with OpenRouter account
- **Limits:** 100 requests per day
- **Models Available:** Multiple models through OpenRouter API
- **Setup:** Already configured in your system
- **Best For:** General purpose, diverse model selection

### 2. Google Gemini 2.5 Flash
- **Free Tier:** Yes, through Google AI Studio
- **Limits:** 60 requests per minute
- **Features:** Text + image support, 1M token context
- **Setup:** Requires Google API key (likely already configured)
- **Best For:** High-quantity, fast responses

### 3. Google Gemini Live 2.5 Flash (Audio)
- **Free Tier:** Yes
- **Limits:** 60 requests/minute
- **Features:** Native audio support, real-time streaming
- **Setup:** Specialized for voice interactions
- **Best For:** Voice-enabled applications

## Recommended Free Stack

For maximum free capacity, I recommend:

1. **Primary:** `openrouter/auto` (auto-selects best free model)
2. **Backup:** `google/gemini-2.5-flash` (high rate limits)
3. **Specialized:** `google/gemini-live-2.5-flash-preview-native-audio` for voice

## Cost Optimization Strategy

- Use `openrouter/free` for general tasks (100/day limit)
- Switch to Google Gemini for high-volume needs (60/min = ~86,400/day)
- Implement failover: if OpenRouter hits limit, automatically switch to Gemini
- Monitor usage in `memory/heartbeat-state.json`

## Next Steps

1. Test current free tier performance
2. Implement automatic model switching based on remaining quotas
3. Set up usage alerts when approaching limits
4. Consider upgrading if production needs exceed free limits

---
*Last Updated: 2026-02-11*
*Agent: Jelly-Legs Clawdbot Intelligence*
EOF && git add LLM_FREE_OFFERS.md && git commit -m "Research: Add comprehensive LLM free tier comparison" && git push origin main
