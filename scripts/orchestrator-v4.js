#!/usr/bin/env node
/**
 * 🤖 AI Team Orchestrator v4 - ENHANCED MODEL ROUTING
 * 
 * Spawns OpenClaw sessions with optimized models for each agent role.
 * Enhanced with 20+ Ollama models including gpt-oss, qwen3.5, nemotron-3-super, etc.
 * 
 * Default Model: nemotron-3-super:cloud
 */

const { Octokit } = require('@octokit/rest');
const { execSync } = require('child_process');

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const OWNER = process.env.REPO_OWNER || 'jelly-legs-ai';
const REPO = process.env.REPO_NAME || 'Jelly-legs-unsteady-workshop';
const OPENCLAW_URL = process.env.OPENCLAW_URL || 'http://localhost:3000';

// Default model for general use
const DEFAULT_MODEL = 'nemotron-3-super:cloud';

// Enhanced agent definitions with optimized model routing
const AGENTS = {
  // Research Zone
  'data-diver': {
    name: 'Data-Diver',
    emoji: '🤿',
    model: 'deepseek-v3.2:cloud',
    thinking: 'high',
    label: 'research',
    zone: 'research',
    directive: 'Deep-dive research and data analysis. Study complex topics, synthesize findings, identify patterns and opportunities. Produce comprehensive research reports with actionable insights.',
    capabilities: ['web research', 'data mining', 'comparative analysis', 'literature review', 'trend identification'],
    handoff: 'design'
  },
  'pattern-seeker': {
    name: 'Pattern-Seeker',
    emoji: '🔮',
    model: 'ministral-3:14b-cloud',
    thinking: 'medium',
    label: 'research',
    zone: 'research',
    directive: 'Identify trends, anomalies, and viral mechanics. Analyze data patterns, spot opportunities, and predict outcomes. Efficient pattern recognition for trend analysis.',
    capabilities: ['pattern recognition', 'anomaly detection', 'viral mechanics', 'trend forecasting', 'data analysis'],
    handoff: 'design'
  },
  
  // Design Zone
  'sketch-bot': {
    name: 'Sketch-Bot',
    emoji: '🎨',
    model: 'qwen3.5:397b-cloud',
    thinking: 'high',
    label: 'design',
    zone: 'design',
    directive: 'Create detailed design specifications and system architecture. Use massive context capacity for comprehensive specs, user flows, and technical documentation.',
    capabilities: ['UX/UI design', 'system architecture', 'specification writing', 'diagram creation', 'technical documentation'],
    handoff: 'build'
  },
  'voice-weaver': {
    name: 'Voice-Weaver',
    emoji: '🎭',
    model: 'minimax-m2.7:cloud',
    thinking: 'medium',
    label: 'design',
    zone: 'design',
    directive: 'Craft brand voice, content, and storytelling. Maintain consistent tone across all communications. Create engaging narratives and brand-aligned content.',
    capabilities: ['content creation', 'brand voice', 'storytelling', 'tone adaptation', 'copywriting'],
    handoff: 'build'
  },
  'hook-maker': {
    name: 'Hook-Maker',
    emoji: '🪝',
    model: 'minimax-m2.5:cloud',
    thinking: 'medium',
    label: 'design',
    zone: 'design',
    directive: 'Design engagement loops and viral hooks. Optimize for maximum impact. Fast iteration for testing multiple hook variants.',
    capabilities: ['viral engineering', 'engagement optimization', 'hook design', 'A/B testing', 'conversion optimization'],
    handoff: 'build'
  },
  
  // Build Zone
  'build-bot': {
    name: 'Build-Bot',
    emoji: '⚙️',
    model: 'devstral-small-2:24b-cloud',
    thinking: 'medium',
    label: 'build',
    zone: 'build',
    directive: 'Build system infrastructure, CI/CD pipelines, and DevOps automation. Purpose-built for infrastructure-as-code and workflow optimization.',
    capabilities: ['DevOps', 'CI/CD', 'infrastructure', 'workflow automation', 'system optimization'],
    handoff: 'review'
  },
  'pipe-layer': {
    name: 'Pipe-Layer',
    emoji: '🧩',
    model: 'qwen3-vl:235b-instruct-cloud',
    thinking: 'medium',
    label: 'build',
    zone: 'build',
    directive: 'Design data pipelines, integrations, and ETL workflows. Vision+language capabilities for understanding complex data flow diagrams.',
    capabilities: ['data pipelines', 'ETL design', 'API integration', 'data flows', 'diagramming'],
    handoff: 'review'
  },
  'code-crafter': {
    name: 'Code-Crafter',
    emoji: '💻',
    model: 'qwen3-coder-next:cloud',
    thinking: 'high',
    label: 'build',
    zone: 'build',
    directive: 'Build working implementations. Write clean, functional code. Specialized for code generation — best-in-class for producing production-ready features.',
    capabilities: ['full-stack development', 'code generation', 'API design', 'debugging', 'prototyping'],
    handoff: 'review',
    isCodeAgent: true
  },
  
  // Security Zone
  'shield-bot': {
    name: 'Shield-Bot',
    emoji: '🛡️',
    model: 'mistral-large-3:675b-cloud',
    thinking: 'high',
    label: 'security',
    zone: 'security',
    directive: 'Conduct security audits, threat analysis, and code reviews. 675B parameters for deep security analysis. Be paranoid — assume attackers are smarter.',
    capabilities: ['security auditing', 'threat analysis', 'code review', 'vulnerability scanning', 'compliance validation'],
    handoff: 'deploy'
  },
  'watcher': {
    name: 'Watcher',
    emoji: '👁️',
    model: 'gemma3:27b-cloud',
    thinking: 'medium',
    label: 'review',
    zone: 'security',
    directive: 'Quality assurance and code review. Excellent balance for careful validation. Catch errors, validate logic, ensure quality standards.',
    capabilities: ['code review', 'QA testing', 'logic validation', 'quality assurance', 'compliance checking'],
    handoff: 'security'
  },
  
  // Strategy Zone
  'map-maker': {
    name: 'Map-Maker',
    emoji: '🗺️',
    model: 'glm-5:cloud',
    thinking: 'medium',
    label: 'strategy',
    zone: 'strategy',
    directive: 'Strategic planning, roadmaps, and milestone setting. Strong reasoning for decomposing complex strategies into actionable steps.',
    capabilities: ['strategic planning', 'roadmap creation', 'milestone setting', 'goal decomposition', 'project planning'],
    handoff: 'build'
  },
  
  // Deploy Zone
  'launch-pad': {
    name: 'Launch-Pad',
    emoji: '🚀',
    model: 'glm-4.7:cloud',
    thinking: 'low',
    label: 'deploy',
    zone: 'deploy',
    directive: 'Release management and deployment verification. Systematic approach to production readiness checklists and final validation.',
    capabilities: ['release management', 'deployment automation', 'verification', 'checklist creation', 'production readiness'],
    handoff: null,
    isCodeAgent: true
  },
  
  // Marketing (All Zones)
  'jelly-legs': {
    name: 'Jelly-Legs',
    emoji: '🪼',
    model: 'gpt-oss:120b-cloud',
    thinking: 'medium',
    label: 'marketing',
    zone: 'all',
    directive: 'Marketing Commander. Craft narratives, manage community, design viral campaigns. OpenAI\'s open model for excellent marketing copy and community engagement.',
    capabilities: ['marketing strategy', 'community management', 'viral campaigns', 'narrative crafting', 'brand storytelling'],
    handoff: 'design'
  }
};

// Model fallback chain for resilience
const MODEL_FALLBACKS = {
  'deepseek-v3.2:cloud': ['nemotron-3-super:cloud', 'mistral-large-3:675b-cloud'],
  'ministral-3:14b-cloud': ['ministral-3:8b-cloud', 'gemma3:12b-cloud'],
  'qwen3.5:397b-cloud': ['qwen3.5:cloud', 'nemotron-3-super:cloud'],
  'minimax-m2.7:cloud': ['minimax-m2.5:cloud', 'gemma3:12b-cloud'],
  'minimax-m2.5:cloud': ['ministral-3:8b-cloud', 'gemma3:4b-cloud'],
  'devstral-small-2:24b-cloud': ['devstral-small-2:24b-cloud', 'nemotron-3-super:cloud'],
  'qwen3-vl:235b-instruct-cloud': ['qwen3-vl:235b-cloud', 'nemotron-3-super:cloud'],
  'qwen3-coder-next:cloud': ['qwen3.5:cloud', 'nemotron-3-super:cloud'],
  'mistral-large-3:675b-cloud': ['nemotron-3-super:cloud', 'qwen3.5:397b-cloud'],
  'gemma3:27b-cloud': ['gemma3:12b-cloud', 'gemma3:4b-cloud'],
  'glm-5:cloud': ['glm-4.7:cloud', 'nemotron-3-super:cloud'],
  'glm-4.7:cloud': ['glm-4.7:cloud', 'nemotron-3-super:cloud'],
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
      // Return first available agent, or rotate based on issue number
      const issueNum = issue.number;
      return agents[issueNum % agents.length];
    }
  }
  
  // Match by title keywords
  if (title.includes('research') || title.includes('analyze') || title.includes('study')) {
    return title.includes('pattern') || title.includes('trend') ? 'pattern-seeker' : 'data-diver';
  }
  if (title.includes('design') || title.includes('spec') || title.includes('architecture')) {
    return title.includes('voice') || title.includes('content') ? 'voice-weaver' : 'sketch-bot';
  }
  if (title.includes('build') || title.includes('implement') || title.includes('create') || title.includes('code')) {
    return 'code-crafter';
  }
  if (title.includes('review') || title.includes('check') || title.includes('qa')) {
    return 'watcher';
  }
  if (title.includes('engineer') || title.includes('workflow') || title.includes('devops')) {
    return 'build-bot';
  }
  if (title.includes('security') || title.includes('audit') || title.includes('vulnerability')) {
    return 'shield-bot';
  }
  if (title.includes('deploy') || title.includes('launch') || title.includes('release')) {
    return 'launch-pad';
  }
  if (title.includes('strategy') || title.includes('roadmap') || title.includes('plan')) {
    return 'map-maker';
  }
  if (title.includes('marketing') || title.includes('community') || title.includes('viral')) {
    return 'jelly-legs';
  }
  
  // Default based on issue number for round-robin
  const defaults = ['data-diver', 'sketch-bot', 'code-crafter', 'shield-bot'];
  return defaults[issue.number % defaults.length];
}

// Get fallback model if primary fails
function getFallbackModel(primaryModel) {
  const fallbacks = MODEL_FALLBACKS[primaryModel] || [DEFAULT_MODEL];
  return fallbacks[0];
}

// Spawn OpenClaw session for agent
async function spawnAgentSession(agentId, issue) {
  const agent = AGENTS[agentId];
  
  console.log(`🤖 Spawning ${agent.emoji} ${agent.name} (${agent.model}) for #${issue.number}`);
  
  // Build task prompt
  const taskPrompt = `You are the ${agent.emoji} **${agent.name}** for the Jelly-legs AI Team.

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
**Zone:** ${agent.zone}

Work autonomously. Produce real, high-quality output. Do not use templates or placeholders.

**Capabilities:** ${agent.capabilities.join(', ')}

Begin your work now.`;

  try {
    // Call OpenClaw to spawn session
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
    
    // Try fallback model
    const fallbackModel = getFallbackModel(agent.model);
    if (fallbackModel !== agent.model) {
      console.log(`  🔄 Trying fallback model: ${fallbackModel}...`);
      try {
        const fallbackCmd = `curl -s -X POST "${OPENCLAW_URL}/api/sessions/spawn" \\
          -H "Content-Type: application/json" \\
          -d '{"agentId":"main","task":${JSON.stringify(taskPrompt)},"model":"${fallbackModel}","thinking":"${agent.thinking}"}'`;
        
        const fallbackResult = execSync(fallbackCmd, { encoding: 'utf8', timeout: 30000 });
        const fallbackResponse = JSON.parse(fallbackResult);
        
        console.log(`  ✅ Fallback session spawned: ${fallbackResponse.sessionKey || 'success'}`);
        return { success: true, agentId, sessionKey: fallbackResponse.sessionKey, usedFallback: true };
      } catch (fallbackError) {
        console.error(`  ❌ Fallback also failed: ${fallbackError.message}`);
      }
    }
    
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
**Zone:** ${agent.zone}

*Spawning AI session...*

---
*Autonomous AI Team v4 - Enhanced Model Routing*` 
  });
}

// Main orchestration loop
async function main() {
  console.log('🧠 AI Team Orchestrator v4 - Enhanced Model Routing');
  console.log(`⏰ ${new Date().toISOString()}`);
  console.log(`📡 ${OWNER}/${REPO}`);
  console.log(`🤖 Default Model: ${DEFAULT_MODEL}`);
  console.log(`📊 Active Agents: ${Object.keys(AGENTS).length}\n`);
  
  try {
    const issues = await getOpenIssues();
    console.log(`📋 Found ${issues.length} open issues\n`);
    
    if (issues.length === 0) {
      console.log('✅ No issues to process');
      return;
    }
    
    // Process up to 3 issues per run
    let processed = 0;
    for (const issue of issues) {
      if (processed >= 3) break;
      
      // Skip if already has in-progress label
      if (issue.labels.some(l => l.name === 'in-progress')) {
        console.log(`⏭️  Skipping #${issue.number} - already in progress`);
        continue;
      }
      
      const agentId = assignAgent(issue);
      const agent = AGENTS[agentId];
      
      console.log(`\n🎯 Issue #${issue.number}: ${issue.title.substring(0, 60)}`);
      console.log(`   Assigned to: ${agent.emoji} ${agent.name} (${agent.model})`);
      
      // Post start comment
      await postStartComment(issue.number, agent);
      
      // Spawn agent session
      const result = await spawnAgentSession(agentId, issue);
      
      if (result.success) {
        if (result.usedFallback) {
          console.log(`   ⚠️  Used fallback model`);
        }
        
        // Add in-progress label
        await octokit.rest.issues.addLabels({
          owner: OWNER,
          repo: REPO,
          issue_number: issue.number,
          labels: ['in-progress', agentId, agent.zone]
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
