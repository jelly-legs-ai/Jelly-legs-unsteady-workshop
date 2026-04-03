const https = require('https');

const commentBody = "## Sprint 40 Update 🦑\n\n**Frontend (aether-site):**\n- Fixed inconsistent key naming in blockchain-stats API: changed Chinese '活跃节点' to English 'activeNodes'\n\n**Backend (Jelly-legs-unsteady-workshop):**\n- Added anyhow::Context import to validator config.rs\n- Fixed async write lock: changed `write().await` to `blocking_write()` in peer pubkeys state management\n\nBoth fixes pushed to main. Chain code building toward stability.";

const body = JSON.stringify({ body: commentBody });

const opts = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': 'Bearer ' + process.env.GITHUB_TOKEN,
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
      console.log('Posted!');
    } else {
      console.log('Failed:', res.statusCode, d);
    }
  });
});

req.on('error', e => console.error(e));
req.write(body);
req.end();
