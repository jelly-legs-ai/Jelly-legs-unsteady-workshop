# 🔗 AI Team Enhanced Workflow — Continuous Development Chain

**Deployed:** 2026-03-20  
**Version:** v4-ENHANCED

---

## 🎯 What Changed

### 1. **No Orphaned Branches Ever**
- Automatic cleanup of branches after PR merge
- Agents must complete PR lifecycle: create → review → merge → delete branch
- No manual branch management required

### 2. **Single Issue Thread Discipline**
- Work stays on one issue until complete
- No branch deviations or parallel work streams
- Clear handoff to next phase via follow-up issue

### 3. **Continuous Development Chain**
```
Research → Design → Build → Review → Security → Deploy
   #1      #2      #3      #4       #5        #6
   ↓       ↓       ↓       ↓        ↓         ↓
 Auto   Auto   Auto   Auto   Auto    Auto
 spawn  spawn  spawn  spawn  spawn   spawn
```

Each completed issue automatically spawns the next phase issue.

---

## 📋 Development Chain Flow

### Phase 1: Research (Data-Diver)
```markdown
Issue #1: RESEARCH: Topic Analysis
├── Agent: 🤿 Data-Diver
├── Model: deepseek-v3.2:cloud
├── Output: Research findings, data analysis
└── On Complete: → Spawns Issue #2
```

### Phase 2: Design (Sketch-Bot)
```markdown
Issue #2: DESIGN: Topic Architecture
├── Agent: 🎨 Sketch-Bot  
├── Model: qwen3.5:397b-cloud (397B)
├── Output: Technical specs, architecture docs
└── On Complete: → Spawns Issue #3
```

### Phase 3: Build (Code-Crafter)
```markdown
Issue #3: BUILD: Topic Implementation
├── Agent: 💻 Code-Crafter
├── Model: qwen3-coder-next:cloud
├── Output: Working code, PR merged
└── On Complete: → Spawns Issue #4
```

### Phase 4: Review (Watcher)
```markdown
Issue #4: REVIEW: Topic QA
├── Agent: 👁️ Watcher
├── Model: gemma3:27b-cloud
├── Output: Code review, QA passed
└── On Complete: → Spawns Issue #5
```

### Phase 5: Security (Shield-Bot)
```markdown
Issue #5: SECURITY: Topic Audit
├── Agent: 🛡️ Shield-Bot
├── Model: mistral-large-3:675b-cloud (675B)
├── Output: Security audit, vulnerabilities fixed
└── On Complete: → Spawns Issue #6
```

### Phase 6: Deploy (Launch-Pad)
```markdown
Issue #6: DEPLOY: Topic Release
├── Agent: 🚀 Launch-Pad
├── Model: glm-4.7:cloud
├── Output: Production deployment
└── On Complete: → CHAIN_COMPLETE ✅
```

---

## 🔧 Agent Workflow Requirements

### For Code Agents (Code-Crafter, Build-Bot, Launch-Pad):

```javascript
// PR Lifecycle — MUST complete all steps:
1. Create branch: feature/issue-{number}
2. Commit work with descriptive messages
3. Push to GitHub
4. Create Pull Request
5. Address review feedback (loop until approved)
6. **Merge the PR** (squash or merge commit)
7. **Delete the branch** after merge
8. Comment: "PR #X merged, branch deleted, chain continuing..."
```

### For Documentation Agents (Data-Diver, Sketch-Bot, etc.):

```javascript
// Single Thread Workflow:
1. Read issue requirements
2. Produce comprehensive documentation
3. Post findings as comment on SAME issue
4. Update issue description with deliverables
5. Comment "Phase complete, next phase spawning..."
6. System auto-creates follow-up issue
```

---

## 🧹 Branch Cleanup Rules

The orchestrator automatically:

1. **Finds branches** matching `agent-*`, `feature/*`, `issue-*`
2. **Checks for PRs**:
   - If no PR exists → **Delete branch** (orphaned)
   - If PR merged → **Delete branch** (cleanup)
   - If PR open → **Keep branch** (active work)
3. **Reports cleanup** in logs

---

## 🔄 Follow-Up Issue Templates

Each agent has a template for the next phase:

| Current Agent | Follow-Up Agent | Auto-Generated Issue |
|---------------|-----------------|---------------------|
| 🤿 Data-Diver | 🎨 Sketch-Bot | "DESIGN: {topic}" |
| 🔮 Pattern-Seeker | 🎨 Sketch-Bot | "DESIGN: {topic}" |
| 🎨 Sketch-Bot | 💻 Code-Crafter | "BUILD: {topic}" |
| 🎭 Voice-Weaver | 💻 Code-Crafter | "BUILD: {topic}" |
| 🪝 Hook-Maker | 💻 Code-Crafter | "BUILD: {topic}" |
| 💻 Code-Crafter | 👁️ Watcher | "REVIEW: {topic}" |
| ⚙️ Build-Bot | 👁️ Watcher | "REVIEW: {topic}" |
| 🧩 Pipe-Layer | 👁️ Watcher | "REVIEW: {topic}" |
| 👁️ Watcher | 🛡️ Shield-Bot | "SECURITY: {topic}" |
| 🛡️ Shield-Bot | 🚀 Launch-Pad | "DEPLOY: {topic}" |
| 🚀 Launch-Pad | — | CHAIN_COMPLETE |

---

## 📁 Files Updated

| File | Purpose |
|------|---------|
| `scripts/orchestrator-v4-enhanced.js` | Enhanced orchestrator with chain logic |
| `HEARTBEAT.md` | Updated dispatch instructions |
| `cron job` | Updated to use enhanced orchestrator |
| `ENHANCED_WORKFLOW.md` | This documentation |

---

## 🚀 Next Steps

1. ✅ **Test the enhanced orchestrator**
   ```bash
   node scripts/orchestrator-v4-enhanced.js
   ```

2. ✅ **Create a test issue** to see the chain in action:
   - Create issue with label `research`
   - System will spawn data-diver
   - When completed, should auto-spawn design issue

3. ✅ **Monitor branch cleanup**
   - Check that merged PR branches are deleted
   - Verify no orphaned branches accumulate

4. ✅ **Verify chain continuity**
   - Complete an issue
   - Confirm follow-up issue created
   - Check labels and assignment

---

## 🎓 Example Complete Chain

```markdown
#1: RESEARCH: Solana Fork Analysis
     ↓ Completed by 🤿 Data-Diver
     ↓ Auto-spawned #2

#2: DESIGN: Solana Fork Architecture  
     ↓ Completed by 🎨 Sketch-Bot
     ↓ Auto-spawned #3

#3: BUILD: Implement Solana Fork
     ↓ PR #18 merged, branch deleted
     ↓ Auto-spawned #4

#4: REVIEW: Solana Fork QA
     ↓ Completed by 👁️ Watcher
     ↓ Auto-spawned #5

#5: SECURITY: Solana Fork Audit
     ↓ Completed by 🛡️ Shield-Bot
     ↓ Auto-spawned #6

#6: DEPLOY: Solana Fork Release
     ↓ Completed by 🚀 Launch-Pad
     ↓ CHAIN_COMPLETE ✅
```

---

**The AI Team now maintains continuous development chains with automatic phase transitions and zero orphaned branches.**

🦑 *Jelly-Legs*
