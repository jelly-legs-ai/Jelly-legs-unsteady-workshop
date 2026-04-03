const https = require('https');

const token = process.env.GITHUB_TOKEN;
const owner = 'jelly-legs-ai';
const repo = 'Jelly-legs-unsteady-workshop';
const issueNumber = 109;

const body = `Sprint 35 Update

Frontend (Aether-Chain):
- Added scroll-animated entrance effects to whitepaper page - sections animate in with fade + slide on scroll
- Enhanced mock agent data with 30 unique names (QuantumTrader, NeuralOracle, HyperionNode, VoidExecutor, etc.)
- Expanded agent capabilities from 8 to 34 unique options
- Added 8 colors and 15 icons for variety

Backend (Jelly-legs-unsteady-workshop):
- Reviewed advanced_staking_contract.rs - auto-compound mechanism looks solid
- Code ready for deployment

Status: Both repos pushed to main`;

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

if (!token) {
  console.log('⚠️ GITHUB_TOKEN not set. Skipping comment.');
  return;
}

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.error(`❌ Status: ${res.statusCode}`);
      console.error(responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error(`❌ Request error: ${e.message}`);
});

req.write(data);
req.end();
