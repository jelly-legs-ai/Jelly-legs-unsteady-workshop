const https = require('https');
const fs = require('fs');

const body = fs.readFileSync('sprint-comment.md', 'utf8').trim();

const data = JSON.stringify({ body });

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${process.env.GITHUB_TOKEN}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(data),
  },
};

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.error(`❌ Failed. Status: ${res.statusCode}`);
      console.error(responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error(`❌ Error: ${e.message}`);
});

req.write(data);
req.end();
