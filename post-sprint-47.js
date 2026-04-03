const https = require('https');

const commentBody = `**Sprint 47 Update** - 2026-04-03 10:34 UTC

**Frontend (aether-site):**
- Enhanced StakingHistory component with interactive chart hover tooltips
- Added data point circles on chart for better mouse hover interaction
- Implemented smooth tooltip display showing date, staked amount, rewards, and APY
- Added CSS transitions for smoother chart animations

**Backend (Jelly-legs-unsteady-workshop):**
- Reviewed staking_v2 API route structure
- Confirmed multi-pool staking system architecture

**Git Status:** aether-site pushed successfully (abc8384)

**Next Sprint:** Continue UI improvements, add more interactive features`;

const token = process.env.GITHUB_TOKEN;

if (!token) {
  console.log('GITHUB_TOKEN not set. Skipping.');
  process.exit(0);
}

const postData = JSON.stringify({ body: commentBody });

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
