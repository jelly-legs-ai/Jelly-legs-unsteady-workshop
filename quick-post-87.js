const https = require('https');

const commentBody = `## 🦑 Sprint 87 Update

**Frontend:** Added GasTracker component with real-time gas prices (Slow/Standard/Fast/Instant tiers), block status monitoring, and network capacity visualization.

**Backend:** Reviewed chain contracts - mining_rewards.rs, staking_contract.rs, database_schema.rs - all stable with comprehensive FLUX staking + mining APIs.

**Next:** FLUX/AETH swap calculator, expanded agent registration

---
*Sprint 87 complete*`;

const token = process.env.GITHUB_TOKEN;

if (!token) {
  console.log('GITHUB_TOKEN not set - skipping GitHub comment');
  process.exit(0);
}

const postData = JSON.stringify({ body: commentBody });

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'User-Agent': 'Jellylegs-AI-Sprint',
    'Accept': 'application/vnd.github+json',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(postData)
  }
};

const req = https.request(options, (res) => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('Posted sprint 87 update to GitHub issue #109');
    } else {
      console.log('Status:', res.statusCode);
    }
  });
});

req.on('error', e => console.error(e));
req.write(postData);
req.end();
