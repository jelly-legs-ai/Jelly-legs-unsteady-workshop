const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const env = {};
fs.readFileSync('.env','utf8').split('\n').forEach(line => {
  const [k,...v] = line.split('=');
  if (k) env[k.trim()] = v.join('=').trim();
});
const octokit = new Octokit({ auth: env.GITHUB_TOKEN });

const title = '🌐 Aether Chain Website - Production Development (ONGOING)';
const body = `## 📋 Issue: Aether Chain Website - Production Grade Development

### Context

The Aether Chain website is deployed but NON-FUNCTIONAL. This is an **ONGOING** development issue that will require continuous iteration until the site is production-ready.

**Live Site:** https://aether-chain.jellylegs.repl.co (or similar)
**Repo:** https://github.com/jelly-legs-ai/Aether-Chain

---

## 🎯 Primary Objectives

### 1. Make Everything Work (CRITICAL)

Every button, link, and interactive element must be functional:

- [ ] **Navigation** - All nav links work and route correctly
- [ ] **Wallet Connection** - Connect to Phantom/Solflare, show address, disconnect
- [ ] **Staking Interface** - Stake/unstake $AETH with real on-chain interaction
- [ ] **Reward Claims** - Claim FLUX rewards to connected wallet
- [ ] **Device Registration** - Register devices for mining
- [ ] **Agent KYC Flow** - Complete claiming flow works end-to-end
- [ ] **Whitepaper** - All sections accessible, sidebar navigation works
- [ ] **Forms** - All forms validate, submit, and show success/error states

### 2. Production Quality Standards

- [ ] **No console errors** - Clean browser console
- [ ] **Responsive** - Works on mobile, tablet, desktop
- [ ] **Fast loading** - <3s initial load
- [ ] **Error handling** - All API/wallet errors handled gracefully
- [ ] **Loading states** - All async actions show loading indicators
- [ ] **Empty states** - Meaningful messages when no data
- [ ] **Accessibility** - Basic a11y compliance

### 3. NO Mobile Apps

- [ ] Remove any iOS/Android app references or download links
- [ ] Focus 100% on web experience
- [ ] Mobile web only (responsive PWA is acceptable)

---

## 🔧 Technical Requirements

### Wallet Integration (PRIORITY)

The site MUST connect to **Solana wallets** (Phantom, Solflare, etc.) and interact with the Aether testnet:

Required wallet functions:
- connectWallet() → connects Phantom/Solflare
- disconnectWallet() → disconnects
- getBalance() → returns $AETH and $FLUX balances  
- stake(amount) → stakes $AETH on-chain
- unstake(amount) → unstakes $AETH
- claimRewards() → claims FLUX rewards
- registerDevice(deviceInfo) → registers mining device

**Testnet RPC:** https://testnet.aetherchain.io/rpc

### API Endpoints Needed

- GET  /api/user/:address → User data + balances
- POST /api/stake → Stake $AETH
- POST /api/unstake → Unstake $AETH  
- POST /api/claim → Claim rewards
- GET  /api/devices/:address → User's devices
- POST /api/devices → Register device
- GET  /api/agents → List agents
- POST /api/agents/claim → Claim agent KYC

---

## 🛠️ Development Approach

### Agent Assignments by Specialty

| Agent | Model | Task |
|-------|-------|------|
| 🎨 Sketch-Bot | qwen3.5:397b-cloud | UI/UX design, components, responsive |
| 💻 Code-Crafter | qwen3-coder-next:cloud | Backend APIs, wallet integration |
| 🛡️ Shield-Bot | mistral-large-3:675b-cloud | Security audit, error handling |
| 🎙️ Voice-Weaver | minimax-m2.7:cloud | Content, copy, UX microcopy |
| 🔍 Pattern-Seeker | ministral-3:14b-cloud | Testing, edge cases, flow validation |

### Sprint Cycles

1. **Sprint 1:** Wallet connection + user data API
2. **Sprint 2:** Staking/unstaking flow
3. **Sprint 3:** Device management
4. **Sprint 4:** Agent KYC flow
5. **Sprint 5:** Polish + security audit
6. **Sprint 6+:** Continuous improvement

### Deployment Pipeline

1. Push to GitHub → Replit polls every 10 min
2. Replit builds + deploys automatically
3. Manual trigger available via Replit deploy button

---

## 📝 Deliverables Per Sprint

Each sprint produces:
1. Working feature (button/link actually works)
2. Code committed to Aether-Chain repo
3. Test credentials/endpoints documented
4. GitHub comment marking sprint complete

---

## ⚠️ Important Constraints

1. **No hallucination** - Only build what's specified
2. **No native mobile apps** - Web only
3. **Production grade** - Not MVP, not beta, production ready
4. **Solana compatible** - Wallet connection must work with Phantom/Solflare
5. **Testnet first** - All on testnet until security audit passes

---

## 🚀 Success Criteria

Website is production-ready when:
- ✅ All buttons/links are functional
- ✅ Wallet connects and displays correct balance
- ✅ Staking/unstaking works on-chain
- ✅ No console errors
- ✅ Passes basic security audit
- ✅ Loads <3s on 3G connection
- ✅ Team can demo end-to-end flow

---

**Status:** Ready for Sprint 1
**Priority:** CRITICAL
**Type:** Ongoing Development
`;

octokit.rest.issues.create({
  owner: 'jelly-legs-ai',
  repo: 'Jelly-legs-unsteady-workshop',
  title: title,
  body: body,
  labels: ['website', 'priority-critical', 'ongoing']
}).then(r => {
  console.log('Created:', r.data.number);
  console.log('URL:', r.data.html_url);
}).catch(e => console.error(e.message));
