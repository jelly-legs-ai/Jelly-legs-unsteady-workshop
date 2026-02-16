# GitHub Dashboard v2.0 - Agent Factory

## 🎯 Objective
Transform the static HTML dashboard into a real-time, interactive Agent Factory showing live workflow progress with 12 autonomous model agents.

## 📊 Current State
- Static HTML/CSS kanban board
- Single Jelly-Legs avatar
- No real-time data connection
- 4 themed rooms (Research, Design, Build, Deploy)

## 🚀 Proposed State
- Real-time GitHub API integration
- 12 unique agent characters with pixel art avatars
- Bird's eye factory floor view
- Live speech bubbles with thoughts/tasks
- Social plaza for agent congregation
- Agent-to-agent work request system

---

## 🟦 STAGE 1 — Research & Architecture

### Task 1.1: GitHub API Integration Design
**Description:** Design real-time data pipeline from GitHub issues/PRs/workflows to dashboard
**Assignee:** Data-Diver 🤿
**Status:** 🔵 Not Started
**Deliverables:**
- [ ] API endpoint mapping (issues, PRs, workflows, commits)
- [ ] Data refresh strategy (WebSocket vs polling)
- [ ] Issue label → stage mapping system
- [ ] Agent assignment algorithm

### Task 1.2: Agent State Machine Architecture
**Description:** Design 12-agent simulation system with positions, tasks, and behaviors
**Assignee:** Pattern-Seeker 🔮
**Status:** 🔵 Not Started
**Deliverables:**
- [ ] Agent class definition (position, task, energy, mood)
- [ ] Room assignment logic
- [ ] Movement animation system
- [ ] Idle behavior patterns

### Task 1.3: Speech Bubble Content Strategy
**Description:** Design context-aware message generation from GitHub data
**Assignee:** Voice-Weaver 🎭
**Status:** 🔵 Not Started
**Deliverables:**
- [ ] Message templates for each agent role
- [ ] GitHub data → message mapping
- [ ] Cross-agent conversation triggers
- [ ] Humor/personality injection system

---

## 🟨 STAGE 2 — Design

### Task 2.1: Pixel Art Factory Floor Layout
**Description:** Design bird's eye view layout with 7 zones + central plaza
**Assignee:** Sketch-Bot 🎨
**Status:** 🟡 In Design
**Deliverables:**
- [ ] 7 room designs (Research, Design, Build, Security, Strategy, Deploy, Plaza)
- [ ] Pixel art style guide (16x16 or 32x32 sprites)
- [ ] CSS-based pseudo-pixel art implementation
- [ ] Isometric vs top-down decision

### Task 2.2: 12-Agent Character Design
**Description:** Create unique avatars, colors, and personalities for each agent
**Assignee:** Sketch-Bot 🎨
**Status:** 🟡 In Design
**Deliverables:**
- [ ] 12 emoji/pixel avatar designs
- [ ] Color scheme for each agent
- [ ] Personality trait definitions
- [ ] Animation sprite sheets (idle, walk, work, talk)

### Task 2.3: Speech Bubble UI Design
**Description:** Design floating speech bubble system with typing animations
**Assignee:** Hook-Maker 🪝
**Status:** 🟡 In Design
**Deliverables:**
- [ ] Speech bubble CSS components
- [ ] Typing animation effect
- [ ] Queue system for multiple messages
- [ ] Dismiss/timeout behavior

---

## 🟧 STAGE 3 — Build

### Task 3.1: GitHub Actions Real-Time Bridge
**Description:** Build GitHub Actions workflow to stream data to dashboard
**Assignee:** Build-Bot ⚙️
**Status:** 🟠 Awaiting Design
**Deliverables:**
- [ ] Workflow YAML for 30-second polling
- [ ] Data aggregation script (Node.js/Python)
- [ ] JSON output to GitHub Pages
- [ ] Rate limit management

### Task 3.2: Agent Simulation Engine
**Description:** Build JavaScript engine for 12-agent movement and behavior
**Assignee:** Pipe-Layer 🧩
**Status:** 🟠 Awaiting Design
**Deliverables:**
- [ ] Agent class implementation
- [ ] Room boundary detection
- [ ] Pathfinding for agent movement
- [ ] Collision avoidance

### Task 3.3: Speech Bubble System
**Description:** Implement live speech bubble display with GitHub data integration
**Assignee:** Code-Crafter 💻
**Status:** 🟠 Awaiting Design
**Deliverables:**
- [ ] Bubble rendering system
- [ ] Message parsing from GitHub webhooks
- [ ] Conversation threading
- [ ] Text-to-bubble animation

### Task 3.4: Work Request Interface
**Description:** Build UI for requesting agent assistance and task delegation
**Assignee:** Shield-Bot 🛡️
**Status:** 🟠 Awaiting Design
**Deliverables:**
- [ ] Agent selection modal
- [ ] Task request form
- [ ] GitHub issue creation integration
- [ ] Notification system

---

## 🟩 STAGE 4 — Deploy (REQUIRES HUMAN APPROVAL)

### Task 4.1: Dashboard v2.0 Launch
**Description:** Deploy enhanced dashboard to GitHub Pages
**Assignee:** Launch-Pad 🚀
**Status:** 🟢 Pending Approval
**Deliverables:**
- [ ] Final QA testing
- [ ] Performance optimization
- [ ] Mobile responsiveness
- [ ] Documentation

### Task 4.2: Agent Factory Live Stream
**Description:** Set up continuous deployment for real-time updates
**Assignee:** Launch-Pad 🚀
**Status:** 🟢 Pending Approval
**Deliverables:**
- [ ] GitHub Actions automation
- [ ] Error handling and fallbacks
- [ ] Usage monitoring
- [ ] Reggie admin controls

---

## 🧠 Agent Roles Reference

| Agent | Role | Specialty | Primary Zone |
|-------|------|-----------|--------------|
| 🪼 Jelly-Legs | Marketing Commander | Memetics, psychology | All zones |
| 🤿 Data-Diver | Research Lead | Market analysis | Research Lab |
| 🔮 Pattern-Seeker | Trend Analyst | Viral mechanics | Research Lab |
| 🎨 Sketch-Bot | Design Architect | Visual systems | Design Studio |
| 🎭 Voice-Weaver | Brand Voice | Narrative crafting | Design Studio |
| 🪝 Hook-Maker | Viral Engineer | Engagement loops | Design Studio |
| ⚙️ Build-Bot | System Developer | Automation | Build Factory |
| 🧩 Pipe-Layer | Pipeline Engineer | Workflows | Build Factory |
| 💻 Code-Crafter | Implementation | Code generation | Build Factory |
| 🛡️ Shield-Bot | Security Guard | Risk assessment | Security Vault |
| 🗺️ Map-Maker | Strategy Lead | Planning | Strategy Room |
| 🚀 Launch-Pad | Deployment Chief | Go-live | Deploy Pad |

---

## 🔄 Human Intervention Protocol

**Reggie can:**
- Pause any task
- Reassign agents
- Modify scope
- Request changes to design
- Approve/reject Stage 4 deployments

**Dashboard v2.0 requires approval before:**
- Live GitHub API connection (rate limits)
- Public deployment
- Agent automation activation

---

*Proposed by: Jelly-Legs 🪼*
*Date: 2026-02-15*
*Status: Awaiting Reggie review*
