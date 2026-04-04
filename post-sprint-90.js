const https = require('https');

const commentBody = `**Sprint 90 Update** (2026-04-04 09:14 UTC)

✅ **Frontend Improvements:**
- Created new **GlobalNetworkMap** component with animated world map
- Shows validator distribution across 7 global regions (NA, EU, AP, SA, AF, ME, AU)
- Live hashrate visualization per region with animated pulse points
- Real-time stats: total validators, hashrate, latency, active nodes
- Expandable region cards with detailed metrics (latency, uptime %)
- Decentralization index indicator in footer
- Integrated into homepage between NetworkCongestion and LiveActivityFeed

🔗 **Branch:** main
📦 **Files:** GlobalNetworkMap.tsx (new), page.tsx (updated)

---

*Posted by 🦑 Jelly-legs AI Agent*`;

async function postComment() {
  const token = process.env.GITHUB_TOKEN;
  
  if (!token) {
    console.log('⚠️ GITHUB_TOKEN not set');
    return;
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
      'Content-Length': data.length,
    },
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      let responseBody = '';
      res.on('data', chunk => responseBody += chunk);
      res.on('end', () => {
        if (res.statusCode === 201) {
          console.log('✅ Comment posted successfully!');
          resolve(JSON.parse(responseBody));
        } else {
          console.error(`❌ Status: ${res.statusCode}`);
          reject(new Error(`HTTP ${res.statusCode}`));
        }
      });
    });
    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

postComment()
  .then(() => console.log('✅ Done!'))
  .catch(err => console.log('⚠️ Error:', err.message));
