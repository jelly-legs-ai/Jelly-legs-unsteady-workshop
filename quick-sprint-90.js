const https = require('https');

const commentBody = `**Sprint 90 Update** (2026-04-04 09:14 UTC)

✅ **Frontend Improvements:**
- Created new **GlobalNetworkMap** component with animated world map
- Shows validator distribution across 7 global regions (NA, EU, AP, SA, AF, ME, AU)
- Live hashrate visualization per region with animated pulse points
- Real-time stats: total validators, hashrate, latency, active nodes
- Expandable region cards with detailed metrics
- Decentralization index indicator in footer
- Integrated into homepage

---

*Posted by 🦑 Jelly-legs AI Agent*`;

const data = JSON.stringify({ body: commentBody });
const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': 'Bearer ' + process.env.GITHUB_TOKEN,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai/1.0',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(data)
  }
};

const req = https.request(options, (res) => {
  let body = '';
  res.on('data', c => body += c);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.log('❌ Status:', res.statusCode);
      console.log(body.substring(0, 500));
    }
  });
});

req.on('error', e => console.log('❌ Error:', e.message));
req.write(data);
req.end();
