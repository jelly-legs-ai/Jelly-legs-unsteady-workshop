#!/usr/bin/env node
const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const { execSync } = require('child_process');
const path = require('path');

// Load .env
const env = {};
fs.readFileSync('.env','utf8').split('\n').forEach(line => {
  const [k,...v] = line.split('=');
  if (k) env[k.trim()] = v.join('=').trim();
});

const octokit = new Octokit({ auth: env.GITHUB_TOKEN });
const OPENCLAW_URL = 'http://127.0.0.1:18789';
const OWNER = 'jelly-legs-ai';
const REPO = 'Jelly-legs-unsteady-workshop';

async function main() {
  const { data: issue } = await octokit.rest.issues.get({
    owner: OWNER,
    repo: REPO,
    issue_number: 11
  });
  
  const taskPrompt = `You are the 🤿 Data-Diver for the Jelly-legs AI Team.

**Your Mission:** Deep-dive research and data analysis on Project AETHER blockchain.

**Issue #11:** ${issue.title}

**Issue Body:**
${issue.body || '(No description)'}

## ⚠️ CRITICAL WORKFLOW RULES:

### 1. DOCUMENT ALL WORK AS ISSUE COMMENTS
- Post comments on THIS issue (#11) to document your research
- Every finding goes as a comment here
- DO NOT create new issues
- DO NOT create branches

### 2. MINIMUM DOCUMENTATION
Before completing, you MUST have posted at least 5 meaningful comments showing:
- Initial research approach
- Key findings from Solana fork analysis
- AI integration opportunities
- Technical architecture considerations
- Final recommendations

### 3. COMPLETION
When done, comment: "## ✅ RESEARCH COMPLETE" then stop.

**Repository:** ${OWNER}/${REPO}
**Issue Number:** 11
**Model:** deepseek-v3.2:cloud
**Thinking:** high

Begin by posting your first research comment on issue #11.`;

  // Post start comment
  await octokit.rest.issues.createComment({
    owner: OWNER,
    repo: REPO,
    issue_number: 11,
    body: `## 🤿 Data-Diver Starting Research

**Model:** deepseek-v3.2:cloud  
**Thinking:** high  
**Zone:** research

*Continuing Project AETHER blockchain research...*

---
**📝 WORKFLOW: All research documented as comments on this issue. No new issues.**`
  });
  
  console.log('Posted start comment on issue #11');
  
  // Spawn agent via OpenClaw API
  const spawnCmd = `curl -s -X POST "${OPENCLAW_URL}/api/sessions/spawn" -H "Content-Type: application/json" -d '{"agentId":"main","task":${JSON.stringify(taskPrompt)},"model":"deepseek-v3.2:cloud","thinking":"high"}'`;
  
  console.log('Spawning agent...');
  const result = execSync(spawnCmd, { encoding: 'utf8', timeout: 30000 });
  const response = JSON.parse(result);
  console.log('Agent spawned:', response.sessionKey || 'success');
}

main().catch(console.error);
