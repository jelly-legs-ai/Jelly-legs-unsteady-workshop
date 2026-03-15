# AI Agent Team - Model Routing Configuration

## Available Models (Ollama Pro)

| Model | Strengths | Best For |
|-------|-----------|----------|
| **kimi-k2.5:cloud** | General purpose, fast, reliable | Simple tasks, spawning subagents, coordination |
| **qwen3:8b** | Complex coding, algorithms, backend logic | Implementation, debugging, system architecture |
| **minimax-m2.5:cloud** | Balanced reasoning, good context window | Research, analysis, documentation |
| **glm-4.7-flash** | Ultra-fast, lightweight | Quick edits, simple fixes, real-time updates |
| **lfm2.5-thinking:1.2b** | Deep reasoning, planning, strategy | Complex problem solving, architecture design |

## Agent-to-Model Mapping

### 🤿 Data-Diver (Research Lead)
**Model:** minimax-m2.5:cloud
**Why:** Excellent for research, data analysis, and synthesizing information
**Tasks:** Market research, pattern analysis, documentation

### 🔮 Pattern-Seeker (Trend Analyst)
**Model:** lfm2.5-thinking:1.2b
**Why:** Deep reasoning for identifying patterns and trends
**Tasks:** Viral mechanics analysis, trend forecasting, strategic insights

### 🎨 Sketch-Bot (Design Architect)
**Model:** kimi-k2.5:cloud
**Why:** Good balance of creativity and technical implementation
**Tasks:** UI/UX design, CSS, visual systems, pixel art coordination

### 🎭 Voice-Weaver (Brand Voice)
**Model:** minimax-m2.2:cloud
**Why:** Strong language understanding and generation
**Tasks:** Content creation, brand voice, storytelling, copywriting

### 🪝 Hook-Maker (Viral Engineer)
**Model:** glm-4.7-flash
**Why:** Fast generation of multiple hook variations
**Tasks:** Engagement loops, viral content, quick iterations

### ⚙️ Build-Bot (System Developer)
**Model:** qwen3:8b
**Why:** Excellent at complex coding and system architecture
**Tasks:** Infrastructure, CI/CD, backend systems, complex implementations

### 🧩 Pipe-Layer (Pipeline Engineer)
**Model:** qwen3:8b
**Why:** Strong at integration and data flow logic
**Tasks:** API integrations, data pipelines, workflow automation

### 💻 Code-Crafter (Implementation)
**Model:** qwen3:8b
**Why:** Best coding model for implementation tasks
**Tasks:** Feature development, bug fixes, code generation

### 🛡️ Shield-Bot (Security Guard)
**Model:** lfm2.5-thinking:1.2b
**Why:** Deep analysis for security review and risk assessment
**Tasks:** Security audits, code review, vulnerability analysis

### 🗺️ Map-Maker (Strategy Lead)
**Model:** lfm2.5-thinking:1.2b
**Why:** Strategic thinking and long-term planning
**Tasks:** Roadmaps, milestones, campaign architecture

### 🚀 Launch-Pad (Deployment Chief)
**Model:** glm-4.7-flash
**Why:** Fast execution for deployment tasks
**Tasks:** Releases, launches, quick deployments

### 🪼 Jelly-Legs (Marketing Commander)
**Model:** kimi-k2.5:cloud
**Why:** General coordination, spawning subagents, oversight
**Tasks:** Team coordination, high-level decisions, final review

## Spawn Configuration

```javascript
const AGENT_MODELS = {
  'data-diver': { model: 'minimax-m2.5:cloud', timeout: 300 },
  'pattern-seeker': { model: 'lfm2.5-thinking:1.2b', timeout: 600 },
  'sketch-bot': { model: 'kimi-k2.5:cloud', timeout: 300 },
  'voice-weaver': { model: 'minimax-m2.5:cloud', timeout: 300 },
  'hook-maker': { model: 'glm-4.7-flash', timeout: 120 },
  'build-bot': { model: 'qwen3:8b', timeout: 600 },
  'pipe-layer': { model: 'qwen3:8b', timeout: 600 },
  'code-crafter': { model: 'qwen3:8b', timeout: 600 },
  'shield-bot': { model: 'lfm2.5-thinking:1.2b', timeout: 600 },
  'map-maker': { model: 'lfm2.5-thinking:1.2b', timeout: 600 },
  'launch-pad': { model: 'glm-4.7-flash', timeout: 120 },
  'jelly-legs': { model: 'kimi-k2.5:cloud', timeout: 300 }
};
```

## Parallel Execution Strategy

With Ollama Pro (3 concurrent models), the system can run:

**Tier 1 (Fast - Flash models):**
- hook-maker (glm-4.7-flash)
- launch-pad (glm-4.7-flash)
- Simple tasks via jelly-legs (kimi-k2.5:cloud)

**Tier 2 (Standard - General models):**
- data-diver (minimax-m2.5:cloud)
- sketch-bot (kimi-k2.5:cloud)
- voice-weaver (minimax-m2.5:cloud)

**Tier 3 (Deep - Thinking/Coding models):**
- pattern-seeker (lfm2.5-thinking:1.2b)
- build-bot (qwen3:8b)
- pipe-layer (qwen3:8b)
- code-crafter (qwen3:8b)
- shield-bot (lfm2.5-thinking:1.2b)
- map-maker (lfm2.5-thinking:1.2b)

## Usage Rules

1. **Always use the assigned model** for each agent
2. **Spawn up to 3 agents simultaneously** (Ollama Pro limit)
3. **Prioritize by issue urgency:**
   - P0: Use fastest available model
   - P1: Use assigned model
   - P2+: Queue for next available slot
4. **Fallback chain:**
   - If model fails → fallback to kimi-k2.5:cloud
   - If timeout → extend by 50% and retry
   - If still failing → escalate to jelly-legs
