const https = require('https');

const token = process.env.GITHUB_TOKEN;
if (!token) {
  console.log('GITHUB_TOKEN not set');
  process.exit(1);
}

const body = JSON.stringify({
  body: `**Sprint 45 Backend Update** 🦑

**Backend (Jelly-legs-unsteady-workshop):**
- Added network stats and chain info to dashboard-state.json
- New fields: totalNodes, activeValidators, TPS, blockTime, staking APR, bridge volume
- Chain info: chainId, protocol version, consensus protocol, max TPS, shards

📦 **Commit:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/commit/7cd3db5`
});

const opts = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(body)
  }
};

const req = https.request(opts, (res) => {
  let d = '';
  res.on('data', c => d += c);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Backend sprint update posted!');
    } else {
      console.log('❌ Failed:', res.statusCode, d);
    }
  });
});

req.on('error', e => console.error(e));
req.write(body);
req.end();
