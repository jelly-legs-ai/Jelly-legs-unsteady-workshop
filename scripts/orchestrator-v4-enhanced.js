#!/usr/bin/env node
/**
 * 🤖 AI Team Orchestrator v4-ENHANCED - Continuous Development Chain
 * 
 * Key Improvements:
 * - PR lifecycle completion (create → push → merge → cleanup)
 * - Single issue thread discipline (no branch deviations)
 * - Auto-chain: spawn follow-up issues on completion
 * - No orphaned branches ever
 */

const { Octokit } = require('@octokit/rest');
const { execSync } = require('child_process');
const path = require('path');

// Load .env file
const envPath = path.join(__dirname, '..', '.env');
try {
  require('dotenv').config({ path: envPath });
  console.log('✅ Loaded .env file');
} catch (e) {
  // dotenv not available, try manual parsing
  try {
    const fs = require('fs');
    const envContent = fs.readFileSync(envPath, 'utf8');
    envContent.split('\n').forEach(line => {
      const [key, ...valueParts] = line.split('=');
      if (key && valueParts.length > 0) {
        process.env[key.trim()] = valueParts.join('=').trim();
      }
    });
    console.log('✅ Loaded .env file manually');
  } catch (e2) {
    console.log('⚠️  Could not load .env file:', e2.message);
  }
}

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const OWNER = process.env.REPO_OWNER || 'jelly-legs-ai';
const REPO = process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop';
const OPENCLAW_URL = process.env.OPENCLAW_URL || 'http://localhost:3000';

const DEFAULT_MODEL = 'nemotron-3-super:cloud';

// Agent definitions with model mapping
const AGENTS = {
  'data-diver': {
    name: 'Data-Diver',
    emoji: '🤿',
    model: 'deepseek-v3.2:cloud',
    thinking: 'high',
    label: 'research',
    zone: 'research',
    directive: 'Deep-dive research and data analysis. Study complex topics, synthesize findings.',
    capabilities: ['web research', 'data mining', 'comparative analysis', 'literature review'],
    handoff: 'sketch-bot'
  },
  'pattern-seeker': {
    name: 'Pattern-Seeker',
    emoji: '🔮',
    model: 'ministral-3:14b-cloud',
    thinking: 'medium',
    label: 'research',
    zone: 'research',
    directive: 'Identify trends, anomalies, and viral mechanics.',
    capabilities: ['pattern recognition', 'anomaly detection', 'viral mechanics'],
    handoff: 'sketch-bot'
  },
  'sketch-bot': {
    name: 'Sketch-Bot',
    emoji: '🎨',
    model: 'qwen3.5:397b-cloud',
    thinking: 'high',
    label: 'design',
    zone: 'design',
    directive: 'Create detailed design specifications and system architecture.',
    capabilities: ['UX/UI design', 'system architecture', 'specification writing'],
    handoff: 'code-crafter'
  },
  'voice-weaver': {
    name: 'Voice-Weaver',
    emoji: '🎭',
    model: 'minimax-m2.7:cloud',
    thinking: 'medium',
    label: 'design',
    zone: 'design',
    directive: 'Craft brand voice, content, and storytelling.',
    capabilities: ['content creation', 'brand voice', 'storytelling'],
    handoff: 'code-crafter'
  },
  'hook-maker': {
    name: 'Hook-Maker',
    emoji: '🪝',
    model: 'minimax-m2.5:cloud',
    thinking: 'medium',
    label: 'design',
    zone: 'design',
    directive: 'Design engagement loops and viral hooks.',
    capabilities: ['viral engineering', 'engagement optimization'],
    handoff: 'code-crafter'
  },
  'build-bot': {
    name: 'Build-Bot',
    emoji: '⚙️',
    model: 'devstral-small-2:24b-cloud',
    thinking: 'medium',
    label: 'build',
    zone: 'build',
    directive: 'Build system infrastructure, CI/CD pipelines, and DevOps automation.',
    capabilities: ['DevOps', 'CI/CD', 'infrastructure', 'workflow automation'],
    handoff: 'watcher'
  },
  'pipe-layer': {
    name: 'Pipe-Layer',
    emoji: '🧩',
    model: 'qwen3-vl:235b-instruct-cloud',
    thinking: 'medium',
    label: 'build',
    zone: 'build',
    directive: 'Design data pipelines, integrations, and ETL workflows.',
    capabilities: ['data pipelines', 'ETL design', 'API integration'],
    handoff: 'watcher'
  },
  'code-crafter': {
    name: 'Code-Crafter',
    emoji: '💻',
    model: 'qwen3-coder-next:cloud',
    thinking: 'high',
    label: 'build',
    zone: 'build',
    directive: 'Build working implementations. Write clean, functional code.',
    capabilities: ['full-stack development', 'code generation', 'API design', 'debugging'],
    handoff: 'watcher',
    isCodeAgent: true
  },
  'shield-bot': {
    name: 'Shield-Bot',
    emoji: '🛡️',
    model: 'mistral-large-3:675b-cloud',
    thinking: 'high',
    label: 'security',
    zone: 'security',
    directive: 'Conduct security audits and threat analysis. Be paranoid.',
    capabilities: ['security auditing', 'threat analysis', 'code review', 'vulnerability scanning'],
    handoff: 'launch-pad'
  },
  'watcher': {
    name: 'Watcher',
    emoji: '👁️',
    model: 'gemma3:27b-cloud',
    thinking: 'medium',
    label: 'review',
    zone: 'security',
    directive: 'Quality assurance and code review. Catch errors, validate logic.',
    capabilities: ['code review', 'QA testing', 'logic validation'],
    handoff: 'shield-bot'
  },
  'map-maker': {
    name: 'Map-Maker',
    emoji: '🗺️',
    model: 'glm-5:cloud',
    thinking: 'medium',
    label: 'strategy',
    zone: 'strategy',
    directive: 'Strategic planning, roadmaps, and milestone setting.',
    capabilities: ['strategic planning', 'roadmap creation', 'milestone setting'],
    handoff: 'build-bot'
  },
  'launch-pad': {
    name: 'Launch-Pad',
    emoji: '🚀',
    model: 'glm-4.7:cloud',
    thinking: 'low',
    label: 'deploy',
    zone: 'deploy',
    directive: 'Release management and deployment verification.',
    capabilities: ['release management', 'deployment automation', 'verification'],
    handoff: null,
    isCodeAgent: true
  },
  'jelly-legs': {
    name: 'Jelly-Legs',
    emoji: '🪼',
    model: 'gpt-oss:120b-cloud',
    thinking: 'medium',
    label: 'marketing',
    zone: 'all',
    directive: 'Marketing Commander. Craft narratives, manage community, design viral campaigns.',
    capabilities: ['marketing strategy', 'community management', 'viral campaigns'],
    handoff: 'sketch-bot'
  }
};

// Follow-up issue templates for continuous development chain
const FOLLOW_UP_TEMPLATES = {
  'data-diver': {
    title: (original) => `DESIGN: ${original.title.replace(/RESEARCH:/i, '').trim()} - Architecture & Design`,
    body: (original, findings) => `## Design Phase

**Previous Research:** #${original.number}
**Findings:** ${findings}

### Design Tasks
- [ ] System architecture diagram
- [ ] Technical specifications
- [ ] Component breakdown
- [ ] API design
- [ ] Data flow documentation

*Auto-generated from completed research phase.*`,
    assignee: 'sketch-bot',
    labels: ['design', 'sketch-bot', 'in-progress']
  },
  'sketch-bot': {
    title: (original) => `BUILD: ${original.title.replace(/DESIGN:/i, '').trim()} - Implementation`,
    body: (original, designDoc) => `## Build Phase

**Previous Design:** #${original.number}
**Design Doc:** ${designDoc}

### Implementation Tasks
- [ ] Core implementation
- [ ] Unit tests
- [ ] Integration tests
- [ ] Documentation updates
- [ ] Performance optimization

*Auto-generated from completed design phase.*`,
    assignee: 'code-crafter',
    labels: ['build', 'code-crafter', 'in-progress']
  },
  'code-crafter': {
    title: (original) => `REVIEW: ${original.title.replace(/BUILD:/i, '').trim()} - Code Review & QA`,
    body: (original, prUrl) => `## Review Phase

**Previous Build:** #${original.number}
**Pull Request:** ${prUrl}

### Review Tasks
- [ ] Code review
- [ ] Security scan
- [ ] QA testing
- [ ] Performance benchmarks
- [ ] Documentation review

*Auto-generated from completed build phase.*`,
    assignee: 'watcher',
    labels: ['review', 'watcher', 'in-progress']
  },
  'watcher': {
    title: (original) => `SECURITY: ${original.title.replace(/REVIEW:/i, '').trim()} - Security Audit`,
    body: (original, reviewNotes) => `## Security Phase

**Previous Review:** #${original.number}
**Review Notes:** ${reviewNotes}

### Security Tasks
- [ ] Vulnerability assessment
- [ ] Dependency audit
- [ ] Access control review
- [ ] Security test cases
- [ ] Penetration testing

*Auto-generated from completed review phase.*`,
    assignee: 'shield-bot',
    labels: ['security', 'shield-bot', 'in-progress']
  },
  'shield-bot': {
    title: (original) => `DEPLOY: ${original.title.replace(/SECURITY:/i, '').trim()} - Production Release`,
    body: (original, securityReport) => `## Deploy Phase

**Previous Security:** #${original.number}
**Security Report:** ${securityReport}

### Deployment Tasks
- [ ] Final verification
- [ ] Deployment checklist
- [ ] Rollback plan
- [ ] Monitoring setup
- [ ] Documentation finalization

*Auto-generated from completed security phase.*`,
    assignee: 'launch-pad',
    labels: ['deploy', 'launch-pad', 'in-progress']
  }
};

const MODEL_FALLBACKS = {
  'deepseek-v3.2:cloud': ['nemotron-3-super:cloud', 'mistral-large-3:675b-cloud'],
  'ministral-3:14b-cloud': ['ministral-3:8b-cloud', 'gemma3:12b-cloud'],
  'qwen3.5:397b-cloud': ['qwen3.5:cloud', 'nemotron-3-super:cloud'],
  'minimax-m2.7:cloud': ['minimax-m2.5:cloud', 'gemma3:12b-cloud'],
  'minimax-m2.5:cloud': ['ministral-3:8b-cloud', 'gemma3:4b-cloud'],
  'devstral-small-2:24b-cloud': ['nemotron-3-super:cloud', 'glm-5:cloud'],
  'qwen3-vl:235b-instruct-cloud': ['qwen3-vl:235b-cloud', 'nemotron-3-super:cloud'],
  'qwen3-coder-next:cloud': ['qwen3.5:cloud', 'nemotron-3-super:cloud'],
  'mistral-large-3:675b-cloud': ['nemotron-3-super:cloud', 'qwen3.5:397b-cloud'],
  'gemma3:27b-cloud': ['gemma3:12b-cloud', 'gemma3:4b-cloud'],
  'glm-5:cloud': ['glm-4.7:cloud', 'nemotron-3-super:cloud'],
  'glm-4.7:cloud': ['nemotron-3-super:cloud'],
  'gpt-oss:120b-cloud': ['gpt-oss:20b-cloud', 'nemotron-3-super:cloud'],
  'nemotron-3-super:cloud': ['qwen3.5:397b-cloud', 'mistral-large-3:675b-cloud']
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

// Check for completed issues that need follow-up
// ONLY creates follow-ups if there's documented evidence of real work
async function checkCompletedIssues() {
  // Get recently closed issues (last 24 hours)
  const since = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
  const { data: closedIssues } = await octokit.rest.issues.listForRepo({
    owner: OWNER,
    repo: REPO,
    state: 'closed',
    since: since,
    per_page: 50
  });
  
  const needsFollowUp = [];
  
  for (const issue of closedIssues) {
    // Check if this was completed by an agent (has agent label)
    const agentLabels = issue.labels.filter(l => AGENTS[l.name]);
    if (agentLabels.length === 0) continue;
    
    // Check if already has follow-up created
    const { data: comments } = await octokit.rest.issues.listComments({
      owner: OWNER,
      repo: REPO,
      issue_number: issue.number,
      per_page: 20
    });
    
    const hasFollowUp = comments.some(c => 
      c.body.includes('**Follow-up Issue Created:**') || 
      c.body.includes('CHAIN_COMPLETE')
    );
    
    if (hasFollowUp) continue;
    
    // REQUIRE: At least 3 meaningful comments showing real work
    // Filter out only system messages (start/stop announcements)
    const meaningfulComments = comments.filter(c => {
      const body = c.body.toLowerCase();
      // Skip pure system messages
      if (body.includes('starting work') && comments.length < 5) return false;
      if (body.includes('spawning ai session')) return false;
      if (body.includes('development chain continued') && comments.length < 5) return false;
      if (body.length < 100) return false; // Too short to be meaningful
      return true;
    });
    
    if (meaningfulComments.length >= 3) {
      console.log(`   ✅ #${issue.number} has ${meaningfulComments.length} meaningful comments - qualifies for follow-up`);
      needsFollowUp.push(issue);
    } else {
      console.log(`   ⚠️  #${issue.number} only has ${comments.length} total comments (${meaningfulComments.length} meaningful) - skipping`);
    }
  }
  
  return needsFollowUp;
}

// Create follow-up issue for continuous development chain
async function createFollowUpIssue(completedIssue) {
  const agentId = completedIssue.labels.find(l => AGENTS[l.name])?.name;
  if (!agentId) return null;
  
  const template = FOLLOW_UP_TEMPLATES[agentId];
  if (!template) return null; // Chain complete (e.g., launch-pad)
  
  const nextAgent = AGENTS[template.assignee];
  
  // Get completion summary from comments
  const { data: comments } = await octokit.rest.issues.listComments({
    owner: OWNER,
    repo: REPO,
    issue_number: completedIssue.number,
    per_page: 5
  });
  
  const completionComment = comments.reverse().find(c => 
    c.body.includes('✅ Complete') || c.body.includes('**Status:** ✅ DONE')
  );
  
  const findings = completionComment ? 
    completionComment.body.substring(0, 500) + '...' : 
    'See previous issue for details.';
  
  // Create follow-up issue
  const { data: newIssue } = await octokit.rest.issues.create({
    owner: OWNER,
    repo: REPO,
    title: template.title(completedIssue),
    body: template.body(completedIssue, findings),
    labels: template.labels
  });
  
  // Comment on original issue
  await octokit.rest.issues.createComment({
    owner: OWNER,
    repo: REPO,
    issue_number: completedIssue.number,
    body: `## 🔗 Development Chain Continued

**Follow-up Issue Created:** #${newIssue.number}
**Next Phase:** ${nextAgent.name} (${nextAgent.emoji})
**Model:** ${nextAgent.model}

The development chain continues automatically. 🦑`
  });
  
  console.log(`🔗 Created follow-up issue #${newIssue.number} for completed #${completedIssue.number}`);
  return newIssue;
}

// Clean up orphaned branches
async function cleanupOrphanedBranches() {
  try {
    // Get all branches
    const { data: branches } = await octokit.rest.repos.listBranches({
      owner: OWNER,
      repo: REPO,
      per_page: 100
    });
    
    // Find branches that look like agent work branches
    const agentBranches = branches.filter(b => 
      b.name.startsWith('agent-') || 
      b.name.startsWith('feature/') ||
      b.name.startsWith('issue-')
    );
    
    for (const branch of agentBranches) {
      // Check if branch has open PR
      const { data: pulls } = await octokit.rest.pulls.list({
        owner: OWNER,
        repo: REPO,
        head: `${OWNER}:${branch.name}`,
        state: 'all'
      });
      
      if (pulls.length === 0) {
        // No PR - delete orphaned branch
        await octokit.rest.git.deleteRef({
          owner: OWNER,
          repo: REPO,
          ref: `heads/${branch.name}`
        });
        console.log(`�️ Cleaned up orphaned branch: ${branch.name}`);
      } else {
        // Check if PR is merged
        const pr = pulls[0];
        if (pr.state === 'closed' && pr.merged) {
          // PR merged - safe to delete branch
          await octokit.rest.git.deleteRef({
            owner: OWNER,
            repo: REPO,
            ref: `heads/${branch.name}`
          });
          console.log(`�️ Cleaned up merged branch: ${branch.name}`);
        }
      }
    }
  } catch (error) {
    console.error('Cleanup error:', error.message);
  }
}

// Determine which agent should handle this issue
function assignAgent(issue) {
  const labels = issue.labels.map(l => l.name.toLowerCase());
  const title = issue.title.toLowerCase();
  
  // Match by explicit agent label
  for (const [agentId, agent] of Object.entries(AGENTS)) {
    if (labels.includes(agentId)) {
      return agentId;
    }
  }
  
  // Match by zone/stage label
  const zoneMap = {
    'research': ['data-diver', 'pattern-seeker'],
    'design': ['sketch-bot', 'voice-weaver', 'hook-maker'],
    'build': ['build-bot', 'pipe-layer', 'code-crafter'],
    'security': ['shield-bot', 'watcher'],
    'strategy': ['map-maker'],
    'deploy': ['launch-pad']
  };
  
  for (const [zone, agents] of Object.entries(zoneMap)) {
    if (labels.includes(zone)) {
      const issueNum = issue.number;
      return agents[issueNum % agents.length];
    }
  }
  
  // Match by title keywords
  if (title.includes('research') || title.includes('analyze')) {
    return title.includes('pattern') ? 'pattern-seeker' : 'data-diver';
  }
  if (title.includes('design') || title.includes('spec') || title.includes('architecture')) {
    return title.includes('voice') ? 'voice-weaver' : 'sketch-bot';
  }
  if (title.includes('build') || title.includes('implement') || title.includes('code')) {
    return 'code-crafter';
  }
  if (title.includes('review') || title.includes('qa')) return 'watcher';
  if (title.includes('engineer') || title.includes('devops')) return 'build-bot';
  if (title.includes('security') || title.includes('audit')) return 'shield-bot';
  if (title.includes('deploy') || title.includes('release')) return 'launch-pad';
  if (title.includes('strategy') || title.includes('roadmap')) return 'map-maker';
  if (title.includes('marketing') || title.includes('community')) return 'jelly-legs';
  
  // Default round-robin
  const defaults = ['data-diver', 'sketch-bot', 'code-crafter', 'shield-bot'];
  return defaults[issue.number % defaults.length];
}

// Get fallback model
function getFallbackModel(primaryModel) {
  const fallbacks = MODEL_FALLBACKS[primaryModel] || [DEFAULT_MODEL];
  return fallbacks[0];
}

// Spawn agent session
async function spawnAgentSession(agentId, issue) {
  const agent = AGENTS[agentId];
  
  console.log(`🤖 Spawning ${agent.emoji} ${agent.name} (${agent.model}) for #${issue.number}`);
  
  // ENHANCED: Include PR completion and follow-up instructions
  const taskPrompt = `You are the ${agent.emoji} **${agent.name}** for the Jelly-legs AI Team.

**Your Mission:** ${agent.directive}

**Issue #${issue.number}:** ${issue.title}

**Issue Body:**
${issue.body || '(No description provided)'}

## ⚠️ CRITICAL WORKFLOW RULES - FOLLOW THESE OR YOU'RE DOING IT WRONG:

### 1. DOCUMENT ALL WORK AS ISSUE COMMENTS
- You MUST post comments on THIS issue (#${issue.number}) to document your work
- Every finding, decision, and update goes as a comment here
- DO NOT create new issues for follow-up work
- DO NOT create new branches
- The issue thread IS your workspace

### 2. MINIMUM DOCUMENTATION REQUIREMENTS
Before completing this issue, you MUST have posted:
- Analysis/approach comment (at least 100 chars)
- Work progress comment (at least 100 chars)  
- Completion/summary comment (at least 100 chars)
- Total: minimum 3 meaningful comments showing real work

### 3. COMPLETION CRITERIA
When you're done, comment:
"## ✅ COMPLETE - [brief summary of what was accomplished]"
Then the issue will be closed and the next phase will be handled separately.

### 4. NO NEW ISSUES
- Do NOT create new GitHub issues
- Do NOT spawn new agent sessions
- Do NOT create branches or PRs
- Your job is to do the work and document it HERE

**Repository:** ${OWNER}/${REPO}
**Issue Number:** ${issue.number}
**Model:** ${agent.model}
**Thinking Level:** ${agent.thinking}
**Zone:** ${agent.zone}

Work autonomously. Document everything as comments on this issue.

**Capabilities:** ${agent.capabilities.join(', ')}

Begin by posting your first comment on issue #${issue.number}.`;

  try {
    const spawnCmd = `curl -s -X POST "${OPENCLAW_URL}/api/sessions/spawn" \\
      -H "Content-Type: application/json" \\
      -d '{"agentId":"main","task":${JSON.stringify(taskPrompt)},"model":"${agent.model}","thinking":"${agent.thinking}"}'`;
    
    console.log(`  📡 Calling OpenClaw with ${agent.model}...`);
    const result = execSync(spawnCmd, { encoding: 'utf8', timeout: 30000 });
    const response = JSON.parse(result);
    
    console.log(`  ✅ Session spawned: ${response.sessionKey || 'success'}`);
    return { success: true, agentId, sessionKey: response.sessionKey };
    
  } catch (error) {
    console.error(`  ❌ Failed to spawn with ${agent.model}: ${error.message}`);
    
    // Try fallback
    const fallbackModel = getFallbackModel(agent.model);
    if (fallbackModel !== agent.model) {
      console.log(`  🔄 Trying fallback: ${fallbackModel}...`);
      try {
        const fallbackCmd = `curl -s -X POST "${OPENCLAW_URL}/api/sessions/spawn" \\
          -H "Content-Type: application/json" \\
          -d '{"agentId":"main","task":${JSON.stringify(taskPrompt)},"model":"${fallbackModel}","thinking":"${agent.thinking}"}'`;
        
        const fallbackResult = execSync(fallbackCmd, { encoding: 'utf8', timeout: 30000 });
        const fallbackResponse = JSON.parse(fallbackResult);
        
        console.log(`  ✅ Fallback spawned: ${fallbackResponse.sessionKey || 'success'}`);
        return { success: true, agentId, sessionKey: fallbackResponse.sessionKey, usedFallback: true };
      } catch (fallbackError) {
        console.error(`  ❌ Fallback failed: ${fallbackError.message}`);
      }
    }
    
    return { success: false, agentId, error: error.message };
  }
}

// Post start comment
async function postStartComment(issueNumber, agent) {
  await octokit.rest.issues.createComment({
    owner: OWNER,
    repo: REPO,
    issue_number: issueNumber,
    body: `## ${agent.emoji} ${agent.name} Starting Work

**Model:** ${agent.model}  
**Thinking:** ${agent.thinking}  
**Zone:** ${agent.zone}

*Spawning AI session...*

---
**📝 WORKFLOW: Document all progress as comments on this issue. DO NOT create new issues.**

---
*Autonomous AI Team v4-ENHANCED*` 
  });
}

// Main orchestration
async function main() {
  console.log('🧠 AI Team Orchestrator v4-FIXED');
  console.log('📝 Issue Thread Work Mode - No Branching, No Spam');
  console.log(`⏰ ${new Date().toISOString()}`);
  console.log(`📡 ${OWNER}/${REPO}`);
  console.log(`🤖 Default Model: ${DEFAULT_MODEL}`);
  console.log(`📊 Agents: ${Object.keys(AGENTS).length}\n`);
  
  try {
    // Step 1: Clean up orphaned branches
    console.log('🧹 Cleaning up orphaned branches...');
    await cleanupOrphanedBranches();
    
    // Step 2: Check for completed issues needing follow-up
    console.log('🔗 Checking for completed issues...');
    const completedIssues = await checkCompletedIssues();
    console.log(`   Found ${completedIssues.length} issues needing follow-up`);
    
    for (const issue of completedIssues) {
      const followUp = await createFollowUpIssue(issue);
      if (followUp) {
        console.log(`   ✅ Follow-up created: #${followUp.number}`);
      }
    }
    
    // Step 3: Check for issues with phase completions that need next phase
    // These are issues with "✅ PHASE X COMPLETE" in comments but still marked in-progress
    console.log('🔄 Checking for phase completions needing next agent...');
    let processed = 0;
    const { data: allOpenIssues } = await octokit.rest.issues.listForRepo({
      owner: OWNER,
      repo: REPO,
      state: 'open',
      per_page: 50
    });
    
    for (const issue of allOpenIssues) {
      // Check for phase completion markers
      const { data: comments } = await octokit.rest.issues.listComments({
        owner: OWNER,
        repo: REPO,
        issue_number: issue.number,
        per_page: 20
      });
      
      const hasComplete = comments.some(c => 
        c.body.includes('✅ COMPLETE') || 
        c.body.includes('✅ PHASE') ||
        c.body.includes('✅ RESEARCH COMPLETE') ||
        c.body.includes('✅ DESIGN COMPLETE')
      );
      
      const hasFollowUp = comments.some(c => 
        c.body.includes('**Follow-up Issue Created:**') || 
        c.body.includes('CHAIN_COMPLETE')
      );
      
      if (hasComplete && !hasFollowUp) {
        const agentId = assignAgent(issue);
        const agent = AGENTS[agentId];
        
        console.log(`   🔗 Issue #${issue.number} phase complete - needs next agent`);
        console.log(`\n📤 DISPATCH: Spawn ${agent.name} for issue #${issue.number} with model ${agent.model}\n`);
        
        // Post start comment so there's a record
        await postStartComment(issue.number, agent);
        processed++;
      }
    }
    
    // Step 4: Process open issues
    const issues = await getOpenIssues();
    console.log(`📋 Found ${issues.length} open issues\n`);
    
    if (issues.length === 0) {
      console.log('✅ No new issues to process');
      return;
    }
    
    // Process up to 3 new issues (not phase completions)
    let spawnedCount = 0;
    for (const issue of issues) {
      if (spawnedCount >= 3) break;
      
      if (issue.labels.some(l => l.name === 'in-progress')) {
        console.log(`⏭️  Skipping #${issue.number} - already in progress`);
        continue;
      }
      
      const agentId = assignAgent(issue);
      const agent = AGENTS[agentId];
      
      console.log(`\n🎯 Issue #${issue.number}: ${issue.title.substring(0, 60)}`);
      console.log(`   Assigned to: ${agent.emoji} ${agent.name} (${agent.model})`);
      
      await postStartComment(issue.number, agent);
      
      const result = await spawnAgentSession(agentId, issue);
      
      if (result.success) {
        if (result.usedFallback) {
          console.log(`   ⚠️  Used fallback model`);
        }
        
        await octokit.rest.issues.addLabels({
          owner: OWNER,
          repo: REPO,
          issue_number: issue.number,
          labels: ['in-progress', agentId, agent.zone]
        });
        spawnedCount++;
      }
    }
    
    const totalSpawned = processed + spawnedCount;
    console.log(`\n✅ Cycle complete. Phase transitions: ${processed}, New spawns: ${spawnedCount}, Follow-ups: ${completedIssues.length}`);
    
  } catch (error) {
    console.error('❌ Orchestration error:', error.message);
    process.exit(1);
  }
}

main();
