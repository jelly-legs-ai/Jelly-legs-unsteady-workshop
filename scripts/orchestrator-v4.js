#!/usr/bin/env node
/**
 * 🤖 AI Team Orchestrator v4 — Issue-Focused Agent Spawner
 * 
 * Dedicated models per issue:
 *   #114 (Website)      → kimi-k2.5:cloud      (large vision + text, great for web dev)
 *   #115 (Blockchain)   → qwen3.5:397b-cloud  (best code能力, massive context)
 *   #116 (CLI/Tools)    → gemma3:27b-cloud     (closest to gemma4:31b available)
 * 
 * Spawns real subagents via OpenClaw sessions API, which do actual work and post results.
 */

const { Octokit } = require('@octokit/rest');
const { execSync } = require('child_process');
const crypto = require('crypto');

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const OWNER = process.env.REPO_OWNER || 'jelly-legs-ai';
const REPO = process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop';
const OPENCLAW_URL = process.env.OPENCLAW_URL || 'http://localhost:18789';

// Issue → Model mapping
const ISSUE_MODELS = {
  114: { model: 'kimi-k2.5:cloud',      emoji: '🌐', name: 'Website Integration Agent' },
  115: { model: 'qwen3.5:397b-cloud',   emoji: '⛓️', name: 'Blockchain Core Agent' },
  116: { model: 'gemma3:27b-cloud',      emoji: '🛠️', name: 'CLI Tools Agent' },
};

const WORKSPACE = process.cwd();

const octokit = new Octokit({ auth: GITHUB_TOKEN });

// Get specific issues we care about
async function getTargetIssues() {
  const issueNumbers = Object.keys(ISSUE_MODELS).map(Number);
  const issues = [];
  
  for (const num of issueNumbers) {
    try {
      const { data } = await octokit.rest.issues.get({ owner: OWNER, repo: REPO, issue_number: num });
      if (!data.pull_request && data.state === 'open') {
        issues.push(data);
      }
    } catch(e) {
      console.log(`  ⚠️  Issue #${num} not found or closed`);
    }
  }
  return issues;
}

// Build task prompt for the specific issue
function buildTaskPrompt(issue, config) {
  const issueContext = {
    114: `You are the 🌐 Website Integration Agent for Jelly-legs AI Team.
Your job: Fix issues, build features, and integrate APIs for the aether-site Next.js app at ${WORKSPACE}/../aether-site.

CURRENT ISSUE: #${issue.number} — ${issue.title}

${issue.body || ''}

Your scope:
- Fix build errors, resolve TypeScript/JSX issues  
- Integrate backend API endpoints (staking, blockchain stats, governance, etc.)
- Improve UI/UX, fix broken components, add live data
- Ensure the site builds clean: cd ${WORKSPACE}/../aether-site && npm run build
- After building, commit your changes and post a detailed result comment to this GitHub issue

When you fix something, commit it with a clear message like: fix(site): resolve [specific issue]
Then post to the GitHub issue what you did, what file changed, and the commit hash.`,

    115: `You are the ⛓️ Blockchain Core Agent for Jelly-legs AI Team.
Your job: Improve the Aether blockchain at ${WORKSPACE}

CURRENT ISSUE: #${issue.number} — ${issue.title}

${issue.body || ''}

Your scope:
- Fix Rust compiler errors: cd ${WORKSPACE} && cargo build --release
- Implement missing RPC endpoints or blockchain features
- Fix consensus, P2P networking, or transaction processing issues
- Ensure clean build with no warnings
- After fixing, commit your changes and post detailed results to this GitHub issue

When you fix something, commit with: feat(chain): implement [feature] or fix(chain): resolve [issue]
Post results to GitHub issue with file changed and commit hash.`,

    116: `You are the 🛠️ CLI Tools Agent for Jelly-legs AI Team.
Your job: Build and improve the aether-cli at ${WORKSPACE}/../aether-cli

CURRENT ISSUE: #${issue.number} — ${issue.title}

${issue.body || ''}

Your scope:
- Fix CLI bugs, add new commands
- Ensure npm packages build and publish cleanly
- Fix aether-cli commands: cd ${WORKSPACE}/../aether-cli && npm run build
- Test commands manually
- Publish updated packages: cd ${WORKSPACE}/../aether-cli && npm version patch && npm publish --access public
- After fixing, commit your changes and post detailed results to this GitHub issue

When you fix something, commit with: fix(cli): resolve [issue] or feat(cli): add [command]
Post results to GitHub issue with what was done and commit hash.`
  };
  
  return issueContext[issue.number] || `Fix issue #${issue.number}: ${issue.title}`;
}

// Spawn agent via OpenClaw sessions API
async function spawnAgent(issue, config) {
  const task = buildTaskPrompt(issue, config);
  const sessionKey = `orch:${issue.number}:${Date.now()}`;
  
  console.log(`  📡 Spawning ${config.emoji} ${config.name} (${config.model})`);
  
  try {
    const cmd = [
      'curl', '-s', '-X', 'POST',
      `${OPENCLAW_URL}/api/sessions/spawn`,
      '-H', 'Content-Type: application/json',
      '-d', JSON.stringify({
        agentId: 'main',
        task: task,
        model: config.model,
        thinking: 'medium',
        label: sessionKey
      })
    ];
    
    const result = execSync(cmd.join(' '), { encoding: 'utf8', timeout: 30000 });
    const response = JSON.parse(result);
    
    console.log(`  ✅ Session: ${response.sessionKey || response.runId || 'spawned'}`);
    return { success: true, sessionKey: response.sessionKey || sessionKey };
  } catch(error) {
    console.log(`  ❌ Spawn failed: ${error.message.substring(0, 100)}`);
    return { success: false, error: error.message };
  }
}

// Post work started comment
async function postStartedComment(issue, config) {
  try {
    await octokit.rest.issues.createComment({
      owner: OWNER,
      repo: REPO,
      issue_number: issue.number,
      body: `## ${config.emoji} ${config.name} — Work Starting
  
**Model:** \`${config.model}\`
**Time:** ${new Date().toISOString().replace('T', ' ').split('.')[0]} UTC

This agent will analyze the issue, implement fixes, and post results here.

*Powered by Jelly-legs AI Orchestrator v4*`
    });
  } catch(e) {
    console.log(`  ⚠️  Could not post comment: ${e.message}`);
  }
}

// Main
async function main() {
  console.log('🧠 AI Team Orchestrator v4 — Issue-Focused');
  console.log(`⏰ ${new Date().toISOString()}`);
  console.log(`📡 ${OWNER}/${REPO}`);
  console.log(`🔗 OpenClaw: ${OPENCLAW_URL}\n`);
  
  try {
    const issues = await getTargetIssues();
    console.log(`🎯 ${issues.length} target issues to process\n`);
    
    for (const issue of issues) {
      const config = ISSUE_MODELS[issue.number];
      
      console.log(`\n📋 Issue #${issue.number}: ${issue.title.substring(0, 70)}`);
      console.log(`   Model: ${config.model}`);
      
      await postStartedComment(issue, config);
      const result = await spawnAgent(issue, config);
      
      if (result.success) {
        console.log(`   ✅ Agent spawned successfully`);
      } else {
        console.log(`   ❌ Failed: ${result.error.substring(0, 80)}`);
      }
      
      // Small delay between issues
      await new Promise(r => setTimeout(r, 1000));
    }
    
    console.log(`\n✅ Cycle complete — ${issues.length} agents spawned`);
    
  } catch(error) {
    console.error('❌ Orchestrator error:', error.message);
    process.exit(1);
  }
}

main();
