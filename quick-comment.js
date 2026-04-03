const https = require('https');

const token = process.env.GITHUB_TOKEN;
const owner = 'jelly-legs-ai';
const repo = 'Jelly-legs-unsteady-workshop';
const issueNumber = 109;

const body = `**Sprint 46 Update - April 3, 2026**

**Frontend (Aether-Chain):**
- Added FAQ search functionality to whitepaper page with real-time filtering
- Search queries question text, answers, and category
- Clear button and result count display
- Visual enhancement with glow effects on search focus

**Backend (Jelly-legs-unsteady-workshop):**
- Pulled latest updates from main branch
- Chain code fully up to date with staking, mining, and governance contracts

**Status:** ✅ Complete | **Next:** Continue UI polish and backend API improvements`;

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
    'Content-Length': Buffer.byteLength(data),
  },
};

if (!token) {
  console.log('⚠️ GITHUB_TOKEN not set. Skipping comment.');
  process.exit(0);
}

const req = https.request(options, (res) => {
  let responseBody = '';
  res.on('data', chunk => responseBody += chunk);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.log(`❌ Status: ${res.statusCode}`);
      console.log(responseBody);
    }
  });
});

req.on('error', (e) => {
  console.error(`❌ Error: ${e.message}`);
});

req.write(data);
req.end();
