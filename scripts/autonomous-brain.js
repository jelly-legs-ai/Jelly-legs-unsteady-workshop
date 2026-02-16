#!/usr/bin/env node
/**
 * 🧠 Autonomous Agent Brain - WITH REAL AI
 * 
 * This script uses Ollama to generate actual AI responses for each agent.
 * Each agent role-plays with real language model outputs.
 */

const { Octokit } = require('@octokit/rest');
const https = require('https');
const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
const OLLAMA_URL = process.env.OLLAMA_URL || 'http://localhost:11434';
const OWNER = 'jelly-legs-ai';
const REPO = 'Jelly-legs-unsteady-workshop';

// Use free OpenRouter model
const AI_MODEL = 'qwen/qwen-2.5-7b-instruct:free';
const OPENROUTER_API_KEY = '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';

const AGENT_PROMPTS = {
  'jelly-legs': {
    name: 'Jelly-Legs', emoji: '🪼', role: 'Marketing Commander',
    prompt: 'You are Jelly-Legs, Marketing Commander. Analyze this task and provide strategic guidance. Focus on market opportunity, viral potential, and success metrics. Be decisive and strategic.'
  },
  'data-diver': {
    name: 'Data-Diver', emoji: '🤿', role: 'Research Lead',
    prompt: 'You are Data-Diver, Research Lead. Dive deep into this topic. Provide factual data, competitive analysis, and key insights. Find numbers, facts, and concrete information.'
  },
  'pattern-seeker': {
    name: 'Pattern-Seeker', emoji: '🔮', role: 'Trend Analyst',
    prompt: 'You are Pattern-Seeker, Trend Analyst. Identify patterns, trends, and predictions. Connect the dots and forecast what comes next. Think creatively about what patterns emerge.'
  },
  'sketch-bot': {
    name: 'Sketch-Bot', emoji: '🎨', role: 'Design Architect',
    prompt: 'You are Sketch-Bot, Design Architect. Create visual concepts and design recommendations. Focus on UI/UX, visual hierarchy, color theory, and user experience. Be specific and actionable.'
  },
  'voice-weaver': {
    name: 'Voice-Weaver', emoji: '🎭', role: 'Brand Voice',
    prompt: 'You are Voice-Weaver, Brand Voice specialist. Craft the narrative and messaging. Focus on copywriting, tone, storytelling, and emotional resonance. Make it compelling.'
  },
  'hook-maker': {
    name: 'Hook-Maker', emoji: '🪝', role: 'Viral Engineer',
    prompt: 'You are Hook-Maker, Viral Engineer. Engineer for virality. Focus on attention triggers, shareability, and growth mechanics. What makes people click and share?'
  },
  'build-bot': {
    name: 'Build-Bot', emoji: '⚙️', role: 'System Developer',
    prompt: 'You are Build-Bot, System Developer. Architect technical solutions. Focus on infrastructure, automation, and system design. Provide technical implementation plans.'
  },
  'pipe-layer': {
    name: 'Pipe-Layer', emoji: '🧩', role: 'Pipeline Engineer',
    prompt: 'You are Pipe-Layer, Pipeline Engineer. Design data flows and integrations. Focus on how data moves between systems. Be specific about connections and data pipelines.'
  },
  'code-crafter': {
    name: 'Code-Crafter', emoji: '💻', role: 'Implementation',
    prompt: 'You are Code-Crafter, Implementation specialist. Write actual code. Provide working code snippets, functions, or scripts. Be precise and functional.'
  },
  'shield-bot': {
    name: 'Shield-Bot', emoji: '🛡️', role: 'Security Guard',
    prompt: 'You are Shield-Bot, Security Guard. Analyze for vulnerabilities and risks. Focus on security concerns, compliance, and risk mitigation. Be thorough and cautious.'
  },
  'map-maker': {
    name: 'Map-Maker', emoji: '🗺️', role: 'Strategy Lead',
    prompt: 'You are Map-Maker, Strategy Lead. Create strategic plans and roadmaps. Focus on milestones, phases, and success metrics. Provide a clear path forward.'
  },
  'launch-pad': {
    name: 'Launch-Pad', emoji: '🚀', role: 'Deployment Chief',
    prompt: 'You are Launch-Pad, Deployment Chief. Assess readiness for deployment. Focus on final checks, release procedures, and launch checklist. Be precise about what needs to happen.'
  }
};

const LABEL_TO_AGENTS = {
  'research': ['data-diver', 'pattern-seeker'],
  'design': ['sketch-bot', 'voice-weaver', 'hook-maker'],
  'build': ['build-bot', 'pipe-layer', 'code-crafter'],
  'security': ['shield-bot'],
  'strategy': ['map-maker', 'jelly-legs'],
  'deploy': ['launch-pad']
};

const STAGE_ORDER = ['research', 'design', 'build', 'security', 'deploy'];

// Call OpenRouter API (free models)
async function askAI(prompt) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      model: AI_MODEL,
      messages: [{ role: 'user', content: prompt }],
      max_tokens: 500
    });

    const options = {
      hostname: 'openrouter.ai',
      port: 443,
      path: '/api/chat/v1/completions',
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${OPENROUTER_API_KEY}`,
        'Content-Type': 'application/json',
        'HTTP-Referer': 'https://jelly-legs-ai.github.io',
        'X-Title': 'Autonomous Agent Team',
        'Content-Length': data.length
      }
    };

    const req = https.request(options, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(body);
          resolve(json.choices?.[0]?.message?.content || json.error || 'No response');
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

class AIBrain {
  constructor(token) {
    this.octokit = new Octokit({ auth: token });
    this.workingOn = new Map();
  }
  
  async start() {
    console.log('🧠 AI-POWERED AUTONOMOUS BRAIN');
    console.log(`📡 GitHub: ${OWNER}/${REPO}`);
    console.log(`🤖 AI Model: ${AI_MODEL}\n`);
    
    await this.poll();
  }
  
  async poll() {
    try {
      console.log(`[${new Date().toISOString()}] 🔄 Polling...`);
      
      const { data: issues } = await this.octokit.issues.listForRepo({
        owner: OWNER, repo: REPO, state: 'open', per_page: 20
      });
      
      for (const issue of issues) {
        if (this.workingOn.has(issue.number)) continue;
        
        const labels = issue.labels.map(l => l.name);
        const stage = labels.find(l => LABEL_TO_AGENTS[l]);
        
        if (stage) await this.doWork(issue, stage, labels);
      }
      
      console.log(`[${new Date().toISOString()}] ✅ Done\n`);
    } catch (error) {
      console.error('❌ Error:', error.message);
    }
  }
  
  async doWork(issue, stage, labels) {
    const agents = LABEL_TO_AGENTS[stage];
    const agentId = agents[Math.floor(Math.random() * agents.length)];
    const agent = AGENT_PROMPTS[agentId];
    
    console.log(`  🎯 ${agent.emoji} ${agent.name} → #${issue.number}`);
    this.workingOn.set(issue.number, { agent: agentId });
    
    try {
      // Build prompt with full context
      const fullPrompt = `${agent.prompt}

TASK: ${issue.title}
${issue.body || ''}

Provide your analysis and work product.`;

      // Call REAL AI
      console.log(`  🧠 ${agent.name} thinking (AI)...`);
      const aiResponse = await askAI(fullPrompt);
      console.log(`  ✨ Got AI response (${aiResponse.length} chars)`);
      
      // Post start comment
      await this.postComment(issue.number, this.formatStart(agent, issue.title));
      
      // Post AI result
      await this.postComment(issue.number, this.formatResult(agent, aiResponse));
      
      // Handoff
      const currentIdx = STAGE_ORDER.indexOf(stage);
      if (currentIdx >= 0 && currentIdx < STAGE_ORDER.length - 1) {
        const nextStage = STAGE_ORDER[currentIdx + 1];
        await this.handoff(issue, stage, nextStage, agent);
      } else {
        await this.postComment(issue.number, this.formatDeployReady(agent));
      }
      
    } catch (error) {
      console.error(`  ❌ Error: ${error.message}`);
      await this.postComment(issue.number, `❌ Error: ${error.message}`);
    }
    
    this.workingOn.delete(issue.number);
  }
  
  async handoff(issue, fromStage, toStage, fromAgent) {
    try {
      await this.octokit.issues.addLabels({
        owner: OWNER, repo: REPO, issue_number: issue.number, labels: [toStage]
      });
    } catch (e) {}
    
    const nextAgents = LABEL_TO_AGENTS[toStage];
    const nextAgent = AGENT_PROMPTS[nextAgents[0]];
    
    await this.postComment(issue.number, this.formatHandoff(fromAgent, nextAgent, toStage));
    console.log(`  🔄 ${fromAgent.name} → ${nextAgent.name}`);
  }
  
  async postComment(issueNumber, body) {
    await this.octokit.issues.createComment({
      owner: OWNER, repo: REPO, issue_number: issueNumber, body
    });
  }
  
  formatStart(agent, task) {
    return `---
### 🎯 ${agent.emoji} ${agent.name} (${agent.role}) Starting

**Task:** ${task}

*Analyzing and beginning work...*

---
🤖 *AI-Powered Autonomous Agent*`;
  }
  
  formatResult(agent, result) {
    return `---
### ✅ ${agent.emoji} ${agent.name} Complete

${result}

---
*Work performed by AI*`;
  }
  
  formatHandoff(from, to, stage) {
    return `---
### 🔄 HANDOFF: ${from.name} → ${to.name}

${from.name} → **${to.name}** (${stage} phase)

---
*AI-Powered Autonomous Agent*`;
  }
  
  formatDeployReady(agent) {
    return `---
### 🚀 READY FOR HUMAN DEPLOY

${agent.name} completed all AI stages. Human review required for deployment.

---
*AI-Powered Autonomous Agent*`;
  }
}

// Run
if (!GITHUB_TOKEN) {
  console.error('❌ GITHUB_TOKEN required');
  process.exit(1);
}

new AIBrain(GITHUB_TOKEN).start();
