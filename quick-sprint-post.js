const https = require('https');

const commentBody = `**Sprint 46 Update** - 2026-04-03 11:20 UTC

**Frontend (aether-site):**
- Added action=history endpoint to /api/staking-v2 returning 30-day chart data points
- Created new StakingHistory component with chart, timeframe selector, and position tracking

**Backend (Jelly-legs-unsteady-workshop):**
- Reviewed FLUX token contract and staking_v2 API
- Reviewed tokens API for FLUX/ATH trading pairs

**Git Status:** Both repos synced and pushed successfully`;

const token = process.env.GITHUB_TOKEN;

if (!token) {
  console.log('GITHUB_TOKEN not set. Skipping.');
  process.exit(0);
}

const postData = JSON.stringify({
  body: commentBody
});

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json'
  },
};

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    console.log('Status:', res.statusCode);
    if (res.statusCode === 201) {
      console.log('Comment posted successfully!');
    } else {
      console.error('Response:', responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error('Request error:', e.message);
});

req.write(postData);
req.end();
