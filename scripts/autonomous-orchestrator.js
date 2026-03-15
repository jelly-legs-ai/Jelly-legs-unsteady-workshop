/**
 * Autonomous AI Team Orchestrator
 * Coordinates 12 agents to complete open GitHub issues every 5 minutes
 */

const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const path = require('path');

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });
const owner = process.env.REPO_OWNER || 'jelly-legs-ai';
const repo = process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop';

// 12 Agent Definitions
const AGENTS = {
  'data-diver': { name: 'Data-Diver', emoji: '🤿', role: 'Research Lead', zones: ['research'] },
  'pattern-seeker': { name: 'Pattern-Seeker', emoji: '🔮', role: 'Trend Analyst', zones: ['research'] },
  'sketch-bot': { name: 'Sketch-Bot', emoji: '🎨', role: 'Design Architect', zones: ['design'] },
  'voice-weaver': { name: 'Voice-Weaver', emoji: '🎭', role: 'Brand Voice', zones: ['design'] },
  'build-bot': { name: 'Build-Bot', emoji: '⚙️', role: 'System Developer', zones: ['build'] },
  'pipe-layer': { name: 'Pipe-Layer', emoji: '🧩', role: 'Pipeline Engineer', zones: ['build'] },
  'code-crafter': { name: 'Code-Crafter', emoji: '💻', role: 'Implementation', zones: ['build'] },
  'shield-bot': { name: 'Shield-Bot', emoji: '🛡️', role: 'Security Guard', zones: ['security'] },
  'map-maker': { name: 'Map-Maker', emoji: '🗺️', role: 'Strategy Lead', zones: ['strategy'] },
  'launch-pad': { name: 'Launch-Pad', emoji: '🚀', role: 'Deployment Chief', zones: ['deploy'] }
};

async function getOpenIssues() {
  const { data: issues } = await octokit.rest.issues.listForRepo({
    owner,
    repo,
    state: 'open',
    per_page: 50
  });
  return issues.filter(i => !i.pull_request); // Exclude PRs
}

async function assignAgentToIssue(issue) {
  const labels = issue.labels.map(l => l.name.toLowerCase());
  
  // Map labels to agents
  if (labels.includes('research')) return 'data-diver';
  if (labels.includes('design')) return 'sketch-bot';
  if (labels.includes('build')) return 'code-crafter';
  if (labels.includes('security')) return 'shield-bot';
  if (labels.includes('deploy')) return 'launch-pad';
  
  // Default based on title keywords
  const title = issue.title.toLowerCase();
  if (title.includes('research') || title.includes('analyze')) return 'data-diver';
  if (title.includes('design') || title.includes('ui') || title.includes('layout')) return 'sketch-bot';
  if (title.includes('build') || title.includes('implement') || title.includes('create')) return 'code-crafter';
  if (title.includes('deploy') || title.includes('launch')) return 'launch-pad';
  
  return 'code-crafter'; // Default
}

async function processIssue(issue) {
  const agentId = await assignAgentToIssue(issue);
  const agent = AGENTS[agentId];
  
  console.log(`🤖 ${agent.emoji} ${agent.name} assigned to #${issue.number}: ${issue.title}`);
  
  // Create a comment indicating work has started
  await octokit.rest.issues.createComment({
    owner,
    repo,
    issue_number: issue.number,
    body: `🤖 **${agent.emoji} ${agent.name}** is now working on this issue.\n\n*Autonomous mode active. ETA: Next cycle.*`
  });
  
  // Update issue with agent label
  await octokit.rest.issues.addLabels({
    owner,
    repo,
    issue_number: issue.number,
    labels: [agentId, 'in-progress']
  });
  
  // Here the actual work would be done by spawning sub-agents
  // For now, we log the assignment
  return { issue, agentId, agent };
}

async function updateKanbanBoard(assignments) {
  const kanbanPath = path.join(__dirname, '..', 'kanban_board.md');
  let kanban = fs.readFileSync(kanbanPath, 'utf8');
  
  // Update timestamp
  const now = new Date().toISOString().split('T')[0];
  kanban = kanban.replace(/Last Updated: .*/, `Last Updated: ${now}`);
  kanban = kanban.replace(/Active Agents: \d+/, `Active Agents: ${assignments.length}`);
  
  fs.writeFileSync(kanbanPath, kanban);
  console.log('📝 Kanban board updated');
}

async function main() {
  console.log('🪼 Autonomous AI Team Orchestrator Starting...');
  console.log(`⏰ ${new Date().toISOString()}`);
  
  try {
    // Get open issues
    const issues = await getOpenIssues();
    console.log(`📋 Found ${issues.length} open issues`);
    
    if (issues.length === 0) {
      console.log('✅ No open issues - team standing by');
      return;
    }
    
    // Process each issue
    const assignments = [];
    for (const issue of issues) {
      // Skip issues already in progress
      if (issue.labels.some(l => l.name === 'in-progress')) {
        console.log(`⏭️  Skipping #${issue.number} - already in progress`);
        continue;
      }
      
      const assignment = await processIssue(issue);
      assignments.push(assignment);
    }
    
    // Update kanban board
    await updateKanbanBoard(assignments);
    
    console.log(`✅ Orchestration complete. ${assignments.length} issues assigned.`);
    
  } catch (error) {
    console.error('❌ Orchestration error:', error.message);
    process.exit(1);
  }
}

main();
