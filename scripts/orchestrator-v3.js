#!/usr/bin/env node
/**
 * 🤖 AI Team Orchestrator v3
 * 
 * Spawns OpenClaw sessions with correct models for each agent.
 * Each agent produces REAL work using AI, not templates.
 */

const { Octokit } = require('@octokit/rest');
const { execSync } = require('child_process');

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const OWNER = process.env.REPO_OWNER || 'jelly-legs-ai';
const REPO = process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop';
const OPENCLAW_URL = process.env.OPENCLAW_URL || 'http://localhost:3000';

// Agent definitions with model mapping
const AGENTS = {
  researcher: {
    name: 'Researcher',
    emoji: '🔬',
    model: 'minimax-m2.7:cloud',
    thinking: 'medium',
    label: 'research',
    directive: 'Study the task, research solutions, document findings. Produce comprehensive research with data, analysis, and actionable insights.',
    capabilities: ['web research', 'data mining', 'comparative analysis', 'documentation'],
    handoff: 'design'
  },
  designer: {
    name: 'Designer', 
    emoji: '🎨',
    model: 'minimax-m2.7:cloud',
    thinking: 'medium',
    label: 'design',
    directive: 'Look at requirements and ask: what\'s missing? How could this be better? Create detailed design specifications with architecture, user flows, and technical requirements.',
    capabilities: ['UX/UI design', 'product planning', 'system architecture', 'specification writing'],
    handoff: 'build'
  },
  developer: {
    name: 'Developer',
    emoji: '💻', 
    model: 'qwen3.5:397b-cloud',
    thinking: 'medium',
    label: 'build',
    directive: 'Build working implementations. Write clean, functional code that solves the problem. Create actual working features, not placeholders.',
    capabilities: ['full-stack development', 'code implementation', 'API design', 'prototyping'],
    handoff: 'review',
    isCodeAgent: true
  },
  watcher: {
    name: 'Watcher',
    emoji: '👁️',
    model: 'qwen3.5:397b-cloud', 
    thinking: 'high',
    label: 'review',
    directive: 'Evaluate proposals, sanity check for validity and functionality. Be thorough - catch errors, validate logic, ensure quality.',
    capabilities: ['code review', 'logic validation', 'quality assurance', 'security pre-scan'],
    handoff: 'security'
  },
  engineer: {
    name: 'Engineer',
    emoji: '⚙️',
    model: 'qwen3.5:397b-cloud',
    thinking: 'medium',
    label: 'engineer',
    directive: 'Think in systems. Create repeatable workflows, automation, and infrastructure. Optimize for efficiency and reliability.',
    capabilities: ['DevOps', 'workflow automation', 'system optimization', 'infrastructure'],
    handoff: 'security'
  },
  cybersecurity: {
    name: 'Cybersecurity',
    emoji: '🛡️',
    model: 'qwen3.5:397b-cloud',
    thinking: 'high',
    label: 'security', 
    directive: 'Evaluate risk exposure, protect brand integrity, identify vulnerabilities. Be paranoid - assume attackers are smarter than you.',
    capabilities: ['risk assessment', 'threat analysis', 'compliance validation', 'code audit'],
    handoff: 'deploy'
  },
  deployment: {
    name: 'Deployment',
    emoji: '🚀',
    model: 'qwen3.5:397b-cloud',
    thinking: 'low',
    label: 'deploy',
    directive: 'Finalize everything for live deployment. Create checklists, verify requirements, ensure production readiness.',
    capabilities: ['release management', 'deployment automation', 'final verification'],
    handoff: null,
    isCodeAgent: true
  }
};

const octokit = new Octokit({ auth: GITHUB_TOKEN });

// Get open issues that need work
async function getOpenIssues() {
  const { data: issues } = await octokit.rest.issues.listForRepo({
    owner: OWNER,
    repo: REPO,
    state: 'open',
    per_page: 20
  });
  return issues.filter(i => !i.pull_request);
}

// Determine which agent should handle this issue
function assignAgent(issue) {
  const labels = issue.labels.map(l => l.name.toLowerCase());
  
  // Match by label
  for (const [agentId, agent] of Object.entries(AGENTS)) {
    if (labels.includes(agent.label)) {
      return agentId;
    }
  }
  
  // Match by title keywords
  const title = issue.title.toLowerCase();
  if (title.includes('research') || title.includes('analyze')) return 'researcher';
  if (title.includes('design') || title.includes('spec')) return 'designer';
  if (title.includes('build') || title.includes('implement') || title.includes('create')) return 'developer';
  if (title.includes('review') || title.includes('check')) return 'watcher';
  if (title.includes('engineer') || title.includes('workflow')) return 'engineer';
  if (title.includes('security') || title.includes('audit')) return 'cybersecurity';
  if (title.includes('deploy') || title.includes('launch')) return 'deployment';
  
  return 'researcher'; // Default
}

// Spawn OpenClaw session for agent
async function spawnAgentSession(agentId, issue) {
  const agent = AGENTS[agentId];
  
  console.log(`🤖 Spawning ${agent.emoji} ${agent.name} (${agent.model}) for #${issue.number}`);
  
  // Build task prompt
  const taskPrompt = `You are the ${agent.emoji} **${agent.name} Agent** for the Jelly-legs AI Team.

**Your Mission:** ${agent.directive}

**Issue #${issue.number}:** ${issue.title}

**Issue Body:**
${issue.body || '(No description provided)'}

**Your Task:**
1. Analyze the issue thoroughly
2. Produce ${agent.isCodeAgent ? 'working code' : 'comprehensive documentation'} that solves the problem
3. ${agent.isCodeAgent ? 'Create a branch, commit your work, and open a PR' : 'Post your findings as a detailed comment on the issue'}
4. Add the label "${agent.handoff || 'complete'}" to hand off to the next agent

**Repository:** ${OWNER}/${REPO}
**Model:** ${agent.model}
**Thinking Level:** ${agent.thinking}

Work autonomously. Produce real, high-quality output. Do not use templates or placeholders.

**Capabilities:** ${agent.capabilities.join(', ')}

Begin your work now.`;

  try {
    // Call OpenClaw to spawn session
    // This uses the sessions_spawn API
    const spawnCmd = `curl -s -X POST "${OPENCLAW_URL}/api/sessions/spawn" \\
      -H "Content-Type: application/json" \\
      -d '{"agentId":"main","task":${JSON.stringify(taskPrompt)},"model":"${agent.model}","thinking":"${agent.thinking}"}'`;
    
    console.log(`  📡 Calling OpenClaw...`);
    const result = execSync(spawnCmd, { encoding: 'utf8', timeout: 30000 });
    const response = JSON.parse(result);
    
    console.log(`  ✅ Session spawned: ${response.sessionKey || 'success'}`);
    return { success: true, agentId, sessionKey: response.sessionKey };
    
  } catch (error) {
    console.error(`  ❌ Failed to spawn session: ${error.message}`);
    return { success: false, agentId, error: error.message };
  }
}

// Post comment that work has started
async function postStartComment(issueNumber, agent) {
  await octokit.rest.issues.createComment({
    owner: OWNER,
    repo: REPO,
    issue_number: issueNumber,
    body: `## ${agent.emoji} ${agent.name} Starting Work

**Model:** ${agent.model}  
**Thinking:** ${agent.thinking}

*Spawning AI session...*

---
*Autonomous AI Team v3*` 
  });
}

// Main orchestration loop
async function main() {
  console.log('🧠 AI Team Orchestrator v3');
  console.log(`⏰ ${new Date().toISOString()}`);
  console.log(`📡 ${OWNER}/${REPO}\n`);
  
  try {
    const issues = await getOpenIssues();
    console.log(`📋 Found ${issues.length} open issues\n`);
    
    if (issues.length === 0) {
      console.log('✅ No issues to process');
      return;
    }
    
    // Process up to 2 issues per run
    let processed = 0;
    for (const issue of issues) {
      if (processed >= 2) break;
      
      // Skip if already has in-progress label
      if (issue.labels.some(l => l.name === 'in-progress')) {
        console.log(`⏭️  Skipping #${issue.number} - already in progress`);
        continue;
      }
      
      const agentId = assignAgent(issue);
      const agent = AGENTS[agentId];
      
      console.log(`\n🎯 Issue #${issue.number}: ${issue.title.substring(0, 60)}`);
      console.log(`   Assigned to: ${agent.emoji} ${agent.name}`);
      
      // Post start comment
      await postStartComment(issue.number, agent);
      
      // Spawn agent session
      const result = await spawnAgentSession(agentId, issue);
      
      if (result.success) {
        // Add in-progress label
        await octokit.rest.issues.addLabels({
          owner: OWNER,
          repo: REPO,
          issue_number: issue.number,
          labels: ['in-progress', agentId]
        });
        processed++;
      }
    }
    
    console.log(`\n✅ Orchestration complete. Spawned ${processed} agent sessions.`);
    
  } catch (error) {
    console.error('❌ Orchestration error:', error.message);
    process.exit(1);
  }
}

main();
