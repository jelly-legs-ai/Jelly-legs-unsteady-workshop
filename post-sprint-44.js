const https = require('https');

const commentBody = `## Sprint 44 Update - 2026-04-03 🦑

### Backend (Chain Code) ✅

**File Modified:** \`contracts/rewards_calculator_test.rs\`

**Sprint 44 Enhancement: Extended Test Cases for Multi-Tier Staking**

Added comprehensive test scenarios for:

1. **MultiTierStakingTest** - 5 new test cases covering:
   - Bronze tier minimum stake (100 units, 1.0x multiplier)
   - Silver tier mid-range (25,000 units, 1.05x multiplier, +5% APY boost)
   - Gold tier with auto-compound (75,000 units, 1.10x multiplier, +10% APY boost)
   - Platinum tier maximum benefits (500,000 units, 1.20x multiplier, +20% APY boost)
   - Validator exclusive pool access (1M units, 1.25x multiplier, +25% APY boost)

2. **AutoCompoundScenario** - 4 new scenarios covering:
   - Daily compounding basic (12% APY)
   - Hourly compounding premium
   - Weekly compounding standard
   - Epoch-level compounding (best returns)

**Git Commit:** \`70dec2d\` - "Sprint 44: Add multi-tier staking and auto-compound test scenarios"

**Lines Added:** ~110 lines across test definitions

**Status:** ✅ Pushed to main successfully

**Next Sprint Plan:**
- Frontend: Integrate multi-tier staking visuals into dashboard
- Backend: Add more edge case validation to staking contract
- Continue expanding test coverage`;

const token = process.env.GITHUB_TOKEN;

if (!token) {
  console.log('⚠️ GITHUB_TOKEN not set. Skipping GitHub comment.');
  console.log('Sprint update ready to paste manually.');
  process.exit(0);
}

const data = JSON.stringify({ body: commentBody });

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
  },
};

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.error(`❌ Failed to post comment. Status: ${res.statusCode}`);
      console.error(responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error(`❌ Request error: ${e.message}`);
});

req.write(data);
req.end();
