/**
 * Dashboard Data Generator
 * Fetches GitHub data and generates real-time dashboard state
 */

const { Octokit } = require('@octokit/rest');
const fs = require('fs');
const path = require('path');

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });
const [owner, repo] = [process.env.REPO_OWNER, process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop'];

// 12 Agent Definitions
const AGENTS = {
  'jelly-legs': { name: 'Jelly-Legs', emoji: '🪼', role: 'Marketing Commander', color: '#ff3333', zones: ['all'] },
  'data-diver': { name: 'Data-Diver', emoji: '🤿', role: 'Research Lead', color: '#3366ff', zones: ['research'] },
  'pattern-seeker': { name: 'Pattern-Seeker', emoji: '🔮', role: 'Trend Analyst', color: '#9933ff', zones: ['research'] },
  'sketch-bot': { name: 'Sketch-Bot', emoji: '🎨', role: 'Design Architect', color: '#ff66cc', zones: ['design'] },
  'voice-weaver': { name: 'Voice-Weaver', emoji: '🎭', role: 'Brand Voice', color: '#ff9933', zones: ['design'] },
  'hook-maker': { name: 'Hook-Maker', emoji: '🪝', role: 'Viral Engineer', color: '#ffcc00', zones: ['design'] },
  'build-bot': { name: 'Build-Bot', emoji: '⚙️', role: 'System Developer', color: '#33cc33', zones: ['build'] },
  'pipe-layer': { name: 'Pipe-Layer', emoji: '🧩', role: 'Pipeline Engineer', color: '#33cccc', zones: ['build'] },
  'code-crafter': { name: 'Code-Crafter', emoji: '💻', role: 'Implementation', color: '#66ff66', zones: ['build'] },
  'shield-bot': { name: 'Shield-Bot', emoji: '🛡️', role: 'Security Guard', color: '#999999', zones: ['security'] },
  'map-maker': { name: 'Map-Maker', emoji: '🗺️', role: 'Strategy Lead', color: '#6666ff', zones: ['strategy'] },
  'launch-pad': { name: 'Launch-Pad', emoji: '🚀', role: 'Deployment Chief', color: '#ffcc00', zones: ['deploy'] }
};

// Stage mapping from labels
const STAGE_LABELS = {
  'research': ['research', 'analysis', 'study', 'investigation'],
  'design': ['design', 'ui', 'ux', 'prototype', 'mockup'],
  'build': ['build', 'develop', 'implement', 'code', 'feature'],
  'security': ['security', 'audit', 'review', 'guardrail'],
  'strategy': ['strategy', 'planning', 'roadmap', 'milestone'],
  'deploy': ['deploy', 'launch', 'release', 'publish']
};

// Map task to agent based on content
function assignAgent(title, body, labels) {
  const text = (title + ' ' + (body || '')).toLowerCase();
  
  // Check for explicit agent mentions
  for (const [agentId, agent] of Object.entries(AGENTS)) {
    if (text.includes(agent.name.toLowerCase()) || text.includes(agentId.replace('-', ' '))) {
      return agentId;
    }
  }
  
  // Map by stage
  for (const [stage, keywords] of Object.entries(STAGE_LABELS)) {
    if (labels.some(l => keywords.includes(l.toLowerCase()))) {
      // Find agent for this stage
      const stageAgents = Object.entries(AGENTS).filter(([id, a]) => a.zones.includes(stage));
      if (stageAgents.length > 0) {
        return stageAgents[Math.floor(Math.random() * stageAgents.length)][0];
      }
    }
  }
  
  // Default: Jelly-Legs for marketing/communication tasks
  if (text.includes('marketing') || text.includes('social') || text.includes('x account') || text.includes('twitter')) {
    return 'jelly-legs';
  }
  
  // Random assignment for other tasks
  const allAgents = Object.keys(AGENTS);
  return allAgents[Math.floor(Math.random() * allAgents.length)];
}

// Determine stage from labels
function getStage(labels) {
  const labelNames = labels.map(l => l.name.toLowerCase());
  
  for (const [stage, keywords] of Object.entries(STAGE_LABELS)) {
    if (labelNames.some(l => keywords.includes(l) || l === stage)) {
      return stage;
    }
  }
  
  return 'research'; // Default
}

// Generate agent thought based on task
function generateThought(agentId, task) {
  const thoughts = {
    'jelly-legs': [
      'Building attention...', 'Engineering narrative...', 'Growing the legend...',
      'Memetics in motion...', 'Community gravity forming...'
    ],
    'data-diver': [
      'Analyzing trends...', 'Diving deep into data...', 'Pattern detection active...',
      'Market research ongoing...', 'Viral mechanics study...'
    ],
    'pattern-seeker': [
      'Found viral pattern!', 'Trend identified...', 'Correlation mapping...',
      'Signal in the noise...', 'Pattern recognition...'
    ],
    'sketch-bot': [
      'Designing assets...', 'Pixel art rendering...', 'Visual system building...',
      'UI components crafting...', 'Mockup generation...'
    ],
    'voice-weaver': [
      'Crafting brand voice...', 'Narrative threading...', 'Tone calibration...',
      'Story architecture...', 'Message resonance...'
    ],
    'hook-maker': [
      'Engagement loop design...', 'Viral hook testing...', 'Attention capture...',
      'Scroll-stopper crafting...', 'Share trigger...'
    ],
    'build-bot': [
      'System construction...', 'Automation building...', 'Pipeline flowing...',
      'Infrastructure growing...', 'Code architecture...'
    ],
    'pipe-layer': [
      'Workflow connecting...', 'Data pipeline active...', 'Integration flowing...',
      'System bridge building...', 'Automation layer...'
    ],
    'code-crafter': [
      'Implementation coding...', 'Feature building...', 'Function crafting...',
      'Logic weaving...', 'Code generation...'
    ],
    'shield-bot': [
      'Security scanning...', 'Risk assessment...', 'Guardrail checking...',
      'Threat detection...', 'Safety protocols...'
    ],
    'map-maker': [
      'Strategy planning...', 'Roadmap drawing...', 'Milestone mapping...',
      'Campaign architecture...', 'Launch sequence...'
    ],
    'launch-pad': [
      'Ready for deploy!', 'Launch countdown...', 'Deployment prepared...',
      'Release sequence...', 'Go-live pending...'
    ]
  };
  
  const agentThoughts = thoughts[agentId] || thoughts['jelly-legs'];
  return agentThoughts[Math.floor(Math.random() * agentThoughts.length)];
}

async function fetchGitHubData() {
  try {
    // Fetch issues
    const { data: issues } = await octokit.rest.issues.listForRepo({
      owner,
      repo,
      state: 'all',
      per_page: 100
    });
    
    // Fetch pull requests
    const { data: pullRequests } = await octokit.rest.pulls.list({
      owner,
      repo,
      state: 'all',
      per_page: 100
    });
    
    // Fetch recent commits
    const { data: commits } = await octokit.rest.repos.listCommits({
      owner,
      repo,
      per_page: 20
    });
    
    // Fetch workflow runs
    const { data: workflows } = await octokit.rest.actions.listWorkflowRunsForRepo({
      owner,
      repo,
      per_page: 10
    });
    
    return { issues, pullRequests, commits, workflows: workflows.workflow_runs };
  } catch (error) {
    console.error('Error fetching GitHub data:', error);
    return { issues: [], pullRequests: [], commits: [], workflows: [] };
  }
}

function processData({ issues, pullRequests, commits, workflows }) {
  const tasks = [];
  const agentStates = {};
  
  // Initialize agent states
  Object.keys(AGENTS).forEach(id => {
    agentStates[id] = {
      ...AGENTS[id],
      id,
      position: { x: 0, y: 0 },
      currentTask: null,
      status: 'idle',
      lastActivity: null
    };
  });
  
  // Process issues as tasks
  issues.forEach(issue => {
    const agentId = assignAgent(issue.title, issue.body, issue.labels.map(l => l.name));
    const stage = getStage(issue.labels);
    
    const task = {
      id: `issue-${issue.number}`,
      type: 'issue',
      number: issue.number,
      title: issue.title,
      body: issue.body?.substring(0, 200) || '',
      state: issue.state,
      stage,
      agent: agentId,
      labels: issue.labels.map(l => l.name),
      url: issue.html_url,
      createdAt: issue.created_at,
      updatedAt: issue.updated_at,
      assignee: issue.assignee?.login || null,
      thought: generateThought(agentId, issue.title)
    };
    
    tasks.push(task);
    
    // Update agent state
    if (issue.state === 'open') {
      agentStates[agentId].currentTask = task;
      agentStates[agentId].status = 'working';
      agentStates[agentId].lastActivity = issue.updated_at;
    }
  });
  
  // Process PRs as tasks
  pullRequests.forEach(pr => {
    const agentId = assignAgent(pr.title, pr.body, []);
    const stage = pr.state === 'open' ? 'build' : 'deploy';
    
    const task = {
      id: `pr-${pr.number}`,
      type: 'pr',
      number: pr.number,
      title: pr.title,
      body: pr.body?.substring(0, 200) || '',
      state: pr.state,
      stage,
      agent: agentId,
      labels: pr.labels?.map(l => l.name) || [],
      url: pr.html_url,
      createdAt: pr.created_at,
      updatedAt: pr.updated_at,
      merged: pr.merged,
      draft: pr.draft,
      thought: generateThought(agentId, pr.title)
    };
    
    tasks.push(task);
  });
  
  // Calculate statistics
  const stats = {
    totalIssues: issues.length,
    openIssues: issues.filter(i => i.state === 'open').length,
    totalPRs: pullRequests.length,
    openPRs: pullRequests.filter(p => p.state === 'open').length,
    recentCommits: commits.length,
    recentWorkflows: workflows.length,
    activeAgents: Object.values(agentStates).filter(a => a.status === 'working').length,
    lastUpdate: new Date().toISOString()
  };
  
  // Generate positions for agents (bird's eye view layout)
  const zonePositions = {
    'research': { x: 15, y: 15, w: 20, h: 15 },
    'design': { x: 40, y: 15, w: 20, h: 15 },
    'build': { x: 65, y: 15, w: 20, h: 15 },
    'security': { x: 15, y: 45, w: 15, h: 12 },
    'strategy': { x: 70, y: 45, w: 15, h: 12 },
    'deploy': { x: 40, y: 75, w: 20, h: 10 },
    'plaza': { x: 35, y: 35, w: 30, h: 25 }
  };
  
  Object.values(agentStates).forEach(agent => {
    const currentTask = tasks.find(t => t.agent === agent.id && t.state === 'open');
    const zone = currentTask ? currentTask.stage : 'plaza';
    const pos = zonePositions[zone] || zonePositions['plaza'];
    
    // Random position within zone
    agent.position = {
      x: pos.x + Math.random() * pos.w,
      y: pos.y + Math.random() * pos.h
    };
  });
  
  return {
    agents: agentStates,
    tasks,
    stats,
    timestamp: new Date().toISOString()
  };
}

async function main() {
  console.log('🤖 Generating dashboard data...');
  
  const githubData = await fetchGitHubData();
  const dashboardState = processData(githubData);
  
  // Ensure data directory exists
  const dataDir = path.join(__dirname, '..', 'data');
  if (!fs.existsSync(dataDir)) {
    fs.mkdirSync(dataDir, { recursive: true });
  }
  
  // Write dashboard state
  const outputPath = path.join(dataDir, 'dashboard-state.json');
  fs.writeFileSync(outputPath, JSON.stringify(dashboardState, null, 2));
  
  console.log(`✅ Dashboard state saved to ${outputPath}`);
  console.log(`📊 Stats: ${dashboardState.stats.openIssues} open issues, ${dashboardState.stats.activeAgents} active agents`);
  console.log(`🕐 Timestamp: ${dashboardState.timestamp}`);
}

main().catch(console.error);
