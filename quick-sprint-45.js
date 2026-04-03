const https = require('https');

const token = process.env.GITHUB_TOKEN;
const owner = 'jelly-legs-ai';
const repo = 'Jelly-legs-unsteady-workshop';
const issueNumber = 109;

const body = `**Sprint 45 Update - Liquidity Pool Contract**

**Backend (Chain Code):**
- Added contracts/liquidity_pool.rs - FLUX/ATH liquidity pool contract
  - Initialize pools with custom amplification coefficients
  - Add/remove liquidity with proportional token minting
  - Swap execution with constant product formula (x*y=k)
  - Impermanent loss calculation
  - Fee tracking and claiming
  - Historical pool snapshots for charting
  - Full test suite

**Repo:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop
**Commit:** b8d946f

More to come in next sprint!`;

if (!token) {
  console.log('GITHUB_TOKEN not set. Set it with: set GITHUB_TOKEN=your_token_here');
  process.exit(1);
}

const data = JSON.stringify({ body });

const options = {
  hostname: 'api.github.com',
  path: `/repos/${owner}/${repo}/issues/${issueNumber}/comments`,
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
    'Content-Length': data.length,
  },
};

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('Comment posted successfully!');
    } else {
      console.error('Failed. Status: ' + res.statusCode);
      console.error(responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error('Request error: ' + e.message);
});

req.write(data);
req.end();
