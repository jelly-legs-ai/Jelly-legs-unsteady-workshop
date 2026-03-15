# AI Agent Team - Model Routing Configuration (Official 9 Agents)

## Available Models (Ollama Pro)

| Model | Strengths | Best For |
|-------|-----------|----------|
| **kimi-k2.5:cloud** | General purpose, fast, reliable | Coordination, design, spawning |
| **qwen3:8b** | Complex coding, algorithms, backend logic | Implementation, systems, debugging |
| **minimax-m2.5:cloud** | Balanced reasoning, good context window | Research, analysis, documentation |
| **glm-4.7-flash** | Ultra-fast, lightweight | Quick fixes, deployments, real-time |
| **lfm2.5-thinking:1.2b** | Deep reasoning, planning, strategy | Review, security, architecture |

## Official 9-Agent-to-Model Mapping

### 🔬 Researcher
**Label:** `research` | **Priority:** 1
**Model:** minimax-m2.5:cloud
**Why:** Excellent for research, data analysis, synthesizing information
**Tasks:** Web research, pattern analysis, documentation, knowledge base building
**Directive:** "Study the task, research solutions, document findings."

### 🎨 Designer  
**Label:** `design` | **Priority:** 2
**Model:** kimi-k2.5:cloud
**Why:** Good balance of creativity and technical implementation
**Tasks:** UX/UI design, product planning, visual identity, design specs
**Directive:** "Look at requirements and ask: what's missing? How could this be better?"

### 💻 Developer
**Label:** `build` | **Priority:** 3
**Model:** qwen3:8b
**Why:** Best coding model for implementation tasks
**Tasks:** Full-stack development, API design, feature implementation
**Directive:** "Build growth frameworks, engagement systems, design architecture."

### 👁️ Watcher
**Label:** `review` | **Priority:** 4
**Model:** lfm2.5-thinking:1.2b
**Why:** Deep reasoning for code review and validation
**Tasks:** Code review, logic validation, security pre-scan, quality assurance
**Directive:** "Evaluate proposals, sanity check for validity and functionality."

### ⚙️ Engineer
**Label:** `engineer` | **Priority:** 5
**Model:** qwen3:8b
**Why:** Strong at systems architecture and automation
**Tasks:** Workflow automation, DevOps, system optimization, infrastructure
**Directive:** "Think in systems. Create repeatable workflows. Reduce entropy."

### 🛡️ Cybersecurity
**Label:** `security` | **Priority:** 6
**Model:** lfm2.5-thinking:1.2b
**Why:** Deep analysis for security review and risk assessment
**Tasks:** Risk assessment, code audit, threat analysis, compliance validation
**Directive:** "Evaluate risk exposure, protect brand integrity, identify scam vectors."

### 🚀 Deployment
**Label:** `deploy` | **Priority:** 7
**Model:** glm-4.7-flash
**Why:** Fast execution for deployment tasks
**Tasks:** Release management, deployment automation, final verification
**Directive:** "Finalize everything for live deployment. Package deliverables."

### 🧠 Skiller (On-Demand)
**Label:** `skill`
**Model:** minimax-m2.5:cloud
**Why:** Good for learning and knowledge synthesis
**Tasks:** Skill creation, Clawhub integration, agent development
**Directive:** "Increase Jelly's capabilities by constantly learning or creating skills."

### 🩹 Doc (On-Demand)
**Label:** `fix`
**Model:** glm-4.7-flash
**Why:** Fast response for emergency fixes
**Tasks:** Debug analysis, hotfixes, emergency response, system recovery
**Directive:** "Resolve errors when they occur. Work with Cybersecurity."

## Spawn Configuration

```javascript
const AGENT_MODELS = {
  'researcher': { model: 'minimax-m2.5:cloud', timeout: 300 },
  'designer': { model: 'kimi-k2.5:cloud', timeout: 300 },
  'developer': { model: 'qwen3:8b', timeout: 600 },
  'watcher': { model: 'lfm2.5-thinking:1.2b', timeout: 600 },
  'engineer': { model: 'qwen3:8b', timeout: 600 },
  'cybersecurity': { model: 'lfm2.5-thinking:1.2b', timeout: 600 },
  'deployment': { model: 'glm-4.7-flash', timeout: 120 },
  'skiller': { model: 'minimax-m2.5:cloud', timeout: 300 },
  'doc': { model: 'glm-4.7-flash', timeout: 120 }
};
```

## Parallel Execution Strategy

With Ollama Pro (3 concurrent models):

**Tier 1 (Fast - Flash):**
- Deployment (glm-4.7-flash)
- Doc (glm-4.7-flash)

**Tier 2 (Standard - General):**
- Researcher (minimax-m2.5:cloud)
- Designer (kimi-k2.5:cloud)
- Skiller (minimax-m2.5:cloud)

**Tier 3 (Deep - Thinking/Coding):**
- Developer (qwen3:8b)
- Watcher (lfm2.5-thinking:1.2b)
- Engineer (qwen3:8b)
- Cybersecurity (lfm2.5-thinking:1.2b)

## Pipeline Flow

```
Researcher (minimax) → Designer (kimi) → Developer (qwen3) → Watcher (lfm2.5) → Engineer (qwen3) → Cybersecurity (lfm2.5) → Deployment (flash)
```

Each agent uses their optimal model for their specific role in the pipeline.
