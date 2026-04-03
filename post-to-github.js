const https = require('https');
const fs = require('fs');

const token = process.env.GITHUB_TOKEN;
const issueNumber = 109;
const owner = 'jelly-legs-ai';
const repo = 'Jelly-legs-unsteady-workshop';

// Read sprint update
const updatePath = 'C:\\Users\\RM_Ga\\.openclaw\\workspace\\sprint-38-update.md';
let sprintUpdate;

try {
  sprintUpdate = fs.readFileSync(updatePath, 'utf8');
} catch (e) {
  console.log('Could not read sprint update file');
  process.exit(1);
}

const body = JSON.stringify({
  body: sprintUpdate
});

const options = {
  hostname: 'api.github.com',
  path: `/repos/${owner}/${repo}/issues/${issueNumber}/comments`,
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(body)
  }
};

console.log('📤 Posting sprint update to GitHub...\n');

const req = https.request(options, (res) => {
  let data = '';
  res.on('data', chunk => data += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Successfully posted to GitHub issue #' + issueNumber);
    } else {
      console.log('❌ HTTP ' + res.statusCode + ': ' + data);
    }
  });
});

req.on('error', e => console.error('❌ Error:', e.message));
req.write(body);
req.end();
