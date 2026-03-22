# 🤖 AI Team Model Routing v2 - Enhanced Architecture

**Last Updated:** 2026-03-20
**Default Model:** `nemotron-3-super:cloud`

---

## 🎯 Model-to-Agent Assignment Matrix

| Agent | Role | Primary Tasks | **Model** | Reasoning |
|-------|------|---------------|-----------|-----------|
| 🤿 **Data-Diver** | Research Lead | Deep research, data analysis, trend identification | `deepseek-v3.2:cloud` | Excels at deep research, reasoning chains, and comprehensive analysis |
| 🔮 **Pattern-Seeker** | Trend Analyst | Pattern recognition, viral mechanics, anomaly detection | `ministral-3:14b-cloud` | Efficient pattern matching, fast inference for trend analysis |
| 🎨 **Sketch-Bot** | Design Architect | UI/UX, system architecture, technical specs | `qwen3.5:397b-cloud` | Massive 397B parameters — unmatched for complex specification generation |
| 🎭 **Voice-Weaver** | Brand Voice | Content, storytelling, tone consistency | `minimax-m2.7:cloud` | Strong creative writing and style adaptation |
| 🪝 **Hook-Maker** | Viral Engineer | Engagement loops, hooks, viral optimization | `minimax-m2.5:cloud` | Fast, efficient for iterative hook generation |
| ⚙️ **Build-Bot** | System Developer | Infrastructure, CI/CD, DevOps | `devstral-small-2:24b-cloud` | Purpose-built for DevOps and automation workflows |
| 🧩 **Pipe-Layer** | Pipeline Engineer | Integrations, data flows, ETL | `qwen3-vl:235b-instruct-cloud` | Vision + language for pipeline diagramming |
| 💻 **Code-Crafter** | Implementation | Feature development, code writing | `qwen3-coder-next:cloud` | Specialized for code generation — best-in-class |
| 🛡️ **Shield-Bot** | Security Guard | Audits, threat analysis, code review | `mistral-large-3:675b-cloud` | 675B parameters — exceptional for deep security analysis |
| 👁️ **Watcher** | Reviewer | QA, validation, logic checking | `gemma3:27b-cloud` | Excellent balance for careful code review |
| 🗺️ **Map-Maker** | Strategy Lead | Planning, roadmaps, milestones | `glm-5:cloud` | Strong reasoning for strategic decomposition |
| 🚀 **Launch-Pad** | Deployment Chief | Releases, checklists, verification | `glm-4.7:cloud` | Reliable for systematic deployment tasks |
| 🪼 **Jelly-Legs** (you) | Marketing Commander | Narrative, community, viral strategy | `gpt-oss:120b-cloud` | OpenAI's open model — excellent for marketing copy and community engagement |

---

## 🧠 Specialty Model Use Cases

### For Vision Tasks (Diagrams, Charts, Screenshots)
- **Model:** `qwen3-vl:235b-cloud` or `qwen3-vl:235b-instruct-cloud`
- **Use:** Analyzing dashboard screenshots, creating visual documentation

### For Massive Context Windows
- **Model:** `qwen3.5:397b-cloud` (largest available)
- **Use:** Processing entire codebases, comprehensive specs

### For Quick Iterations / Cost-Effective
- **Model:** `ministral-3:3b-cloud` or `ministral-3:8b-cloud`
- **Use:** Draft generation, brainstorming, quick iterations

### For Google's Gemma Ecosystem
- **Model:** `gemma3:4b-cloud` (lightweight), `gemma3:12b-cloud` (balanced), `gemma3:27b-cloud` (powerful)
- **Use:** General tasks, code review, validation

---

## ⚙️ Implementation Notes

### Default Model
```javascript
const DEFAULT_MODEL = 'nemotron-3-super:cloud';
```

### Fallback Chain
```javascript
const FALLBACK_CHAIN = [
  'nemotron-3-super:cloud',      // Primary default
  'gpt-oss:120b-cloud',          // Fallback 1
  'mistral-large-3:675b-cloud',  // Fallback 2
  'qwen3.5:397b-cloud'           // Fallback 3 (largest)
];
```

### Agent Priority Matrix
- **High Priority (Critical Issues):** `mistral-large-3:675b-cloud`, `qwen3.5:397b-cloud`
- **Medium Priority (Standard Work):** `deepseek-v3.2:cloud`, `qwen3-coder-next:cloud`
- **Low Priority (Drafts/Exploration):** `ministral-3:8b-cloud`, `gemma3:4b-cloud`

---

## 🔧 Integration Checklist

- [x] Update `scripts/orchestrator-v3.js` with new model mappings
- [x] Update `HEARTBEAT.md` dispatch logic
- [x] Update GitHub Actions workflow (if applicable)
- [x] Test each model with appropriate agent
- [x] Document fallback behavior
- [x] Set up model availability monitoring

---

*This configuration optimizes for capability-per-agent while maintaining cost-efficiency through model specialization.*
