const https = require('https');

const token = process.env.GITHUB_TOKEN;
if (!token) {
  console.log('GITHUB_TOKEN not set');
  process.exit(1);
}

const body = JSON.stringify({
  body: `**Sprint 45 Update** 🦑

**Added:** Multi-pool Staking V2 API

- 4 tiered pools: Bronze (5% APY), Silver (8%), Gold (12%), Diamond (18%)
- Early unstake penalties per pool
- Tier-based reward multipliers
- Full position management: stake, claim, unstake, auto-compound
- Connects to existing Rust \`staking_contract_v2.rs\` backend

📦 **Commit:** https://github.com/jelly-legs-ai/Aether-Chain/commit/525b9df`
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
      console.log('✅ Posted to GitHub!');
    } else {
      console.log('❌ Failed:', res.statusCode, d);
    }
  });
});

req.on('error', e => console.error(e));
req.write(body);
req.end();
