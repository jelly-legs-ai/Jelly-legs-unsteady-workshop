const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const env = {};
fs.readFileSync('.env','utf8').split('\n').forEach(line => {
  const [k,...v] = line.split('=');
  if (k) env[k.trim()] = v.join('=').trim();
});
const octokit = new Octokit({ auth: env.GITHUB_TOKEN });
const body = `## 🔄 SPRINT STATUS - 2026-03-22 19:00 UTC

### ✅ CRON IS WORKING - Commits Being Pushed

The team has been making continuous improvements. Recent commits:

| Commit | Description |
|--------|-------------|
| dc8c1c5 | Enhance whitepaper with reading progress bar |
| d0ec46c | Enhance agents page with scroll-triggered stats animation |
| 5c98798 | Major design improvements and new sections |
| 46e78e8 | Major UI/UX overhaul - Spring Sprint #2 |
| 740f4d2 | Fix placeholder links, roadmap status icons |
| 6eaf046 | Fix invalid Tailwind classes |
| 9a30d29 | Final verification, all pages functional |

### What's Been Built:
- ✅ Home, Dashboard, Devices, Agents, Agent Claim, Whitepaper
- ✅ Wallet connection (Phantom/Solflare)
- ✅ Toast notifications, error boundaries
- ✅ Loading skeletons, 404/error pages
- ✅ SEO (sitemap, OG image, metadata)
- ✅ Animated gradients, scroll effects
- ✅ Mock data throughout

### ⚠️ Issue: Comments Posted to Wrong Repo

The cron was posting to issue #2 in Aether-Chain repo (wrong repo). **This has been fixed.** Future updates will post here.

### 🚀 Status: CONTINUOUS DEVELOPMENT

The site is being continuously improved. Team is now focusing on:
- More animations and polish
- Better UI/UX
- Content expansion

**This is an ONGOING project - never "done"** 🦑`;

octokit.rest.issues.createComment({
  owner: 'jelly-legs-ai',
  repo: 'Jelly-legs-unsteady-workshop',
  issue_number: 109,
  body: body
}).then(() => console.log('Posted')).catch(e => console.error(e.message));
