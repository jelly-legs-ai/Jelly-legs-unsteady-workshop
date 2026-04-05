const https = require('https');

const comment = `## 🦑 Sprint 40 Update - April 2, 2026

### Completed:

**Frontend (aether-site):**
- ✅ Added Subscribe nav link to Navbar — links to /subscribe page
- ✅ Added Premium Tiers feature card on homepage (4 plans, benefits, specs)

**Backend (Jelly-legs-unsteady-workshop):**
- ✅ Pushed validator CLI refactor (struct destructuring pattern)

### Next Sprint:
- Enhance subscription/pricing page UI
- Agent registration flow improvements
- Backend API refinements

---
*Time: ~4 min sprint | Branch: main | Status: ✅ Pushed*`;

const token = process.env.GITHUB_TOKEN;
const data = JSON.stringify({ body: comment });

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/109/comments',
  method: 'POST',
  headers: {
    'Authorization': 'Bearer ' + token,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(data),
  },
};

const req = https.request(options, (res) => {
  let body = '';
  res.on('data', c => body += c);
  res.on('end', () => {
    if (res.statusCode === 201) {
      console.log('✅ Comment posted successfully!');
    } else {
      console.log('❌ Status:', res.statusCode);
      console.log(body);
    }
  });
});

req.on('error', e => console.error('Error:', e.message));
req.write(data);
req.end();
