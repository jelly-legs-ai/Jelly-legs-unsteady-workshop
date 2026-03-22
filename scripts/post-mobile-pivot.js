#!/usr/bin/env node
const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const env = {};
fs.readFileSync('.env','utf8').split('\n').forEach(line => {
  const [k,...v] = line.split('=');
  if (k) env[k.trim()] = v.join('=').trim();
});
const octokit = new Octokit({ auth: env.GITHUB_TOKEN });

async function main() {
  await octokit.rest.issues.createComment({
    owner: 'jelly-legs-ai',
    repo: 'Jelly-legs-unsteady-workshop',
    issue_number: 11,
    body: `## Mobile-Mining Pivot - Project AETHER v2

**PIVOT:** Cloud validators → Mobile-Mining Network (Pi-style)

### Architecture Overview

**Hybrid Model:**
- Mobile/Laptop nodes = Work/Mining layer (millions of users)
- Full validators = Consensus layer (ZK-rollup aggregated)

### Device Tiers & Rewards

| Tier | Device | Base Reward |
|------|--------|-------------|
| 1 | Mobile Phone | 0.1x |
| 2 | Laptop | 1.0x |
| 3 | Desktop | 2.5x |
| 4 | Server | 10x |

**Reward Formula:** DailyReward = BaseReward × TrustScore × TierMult × ActiveHours

### Dual-Token Design

**$AETH (Governance)**
- Max Supply: 1B tokens
- Inflation: 5% Year 1, declining
- Use: Staking, governance voting

**$COMPUTE (Gas/Credits)**
- Elastic supply via bonding curve
- Use: Pay for AI agent compute tasks
- Earned by: Mobile miners providing compute

### AETHER Flywheel

1. AI agents pay $COMPUTE for tasks
2. Mobile miners earn $COMPUTE for processing
3. Network grows = more utility
4. More utility = higher $AETH demand
5. Higher $AETH = more validators secure network

### Key Differentiator vs Pi/Helium

AETHER has built-in AI agent demand driver - not dependent on external app usage.

**Year 5 Targets:**
- 100M mobile miners
- $50 AETH price
- 10M daily AI task executions

---
*Pivot complete - mobile-mining architecture adopted*
`
  });
  console.log('Posted mobile-mining pivot summary');
}

main().catch(console.error);
