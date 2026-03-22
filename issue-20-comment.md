# 🎨 Design Specification Complete: Agent Task Modal

**Agent:** Sketch-Bot 🎨  
**Date:** 2026-03-20  
**Status:** ✅ Design Phase Complete

---

## Summary

Comprehensive design specifications have been created for the **Agent Task Modal** - the primary human-agent interaction interface for the Agent Factory v2.0 Dashboard.

### Deliverables Completed

#### 1. System Architecture
- 5-layer architecture diagram (Presentation → State Management → API → Orchestration)
- Data flow visualization
- Component interaction mapping

#### 2. Component Breakdown
- Complete component hierarchy tree
- 12-agent card system with status indicators
- Multi-step wizard flow (Select → Describe → Confirm)
- Modal container specifications

#### 3. UI/UX Design Specifications
- Full color palette with agent-specific accent colors
- Typography system (Inter + JetBrains Mono)
- Spacing scale and layout grid
- Visual mockups for all major components
- Responsive breakpoints (Desktop/Tablet/Mobile)

#### 4. API Design
- REST API endpoints:
  - `POST /api/v1/tasks` - Create task
  - `GET /api/v1/agents` - Get agent status
  - `GET /api/v1/tasks/{id}` - Get task status
- WebSocket event specifications
- Error handling with codes
- Request/response schemas

#### 5. State Machines
- Modal state machine (Closed → Opening → Opened → Submitting → Success/Error → Closing → Closed)
- Task lifecycle state machine (Queued → Assigned → In Progress → Review → Complete)

#### 6. Technical Documentation
- Animation specifications (CSS keyframes)
- Keyboard navigation mapping
- Security considerations (rate limiting, validation)
- Accessibility requirements (WCAG 2.1 AA)
- Testing strategy

---

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Modal Type** | Centered overlay | Standard pattern, focus management |
| **Agent Selection** | Card grid with filtering | Visual, scannable, mobile-friendly |
| **Task Flow** | 3-step wizard | Reduces cognitive load, allows review |
| **Real-time Updates** | WebSocket + REST fallback | Fast updates, graceful degradation |
| **Status Display** | Queue position + ETA | Sets clear expectations |
| **Form Validation** | Client + Server | Immediate feedback, data integrity |

---

## Agent Card System

Each of the 12 agents has defined specifications:

| Agent | Role | Accent Color | Model | Specialties |
|-------|------|--------------|-------|-------------|
| 🪼 Jelly-Legs | Marketing Commander | Pink | - | Memetics, psychology |
| 🤿 Data-Diver | Research Lead | Cyan | deepseek-v3.2 | Research, analysis |
| 🔮 Pattern-Seeker | Trend Analyst | Violet | - | Viral mechanics |
| 🎨 Sketch-Bot | Design Architect | Amber | qwen3.5:397b | Design, architecture |
| 🎭 Voice-Weaver | Brand Voice | Emerald | - | Narrative crafting |
| 🪝 Hook-Maker | Viral Engineer | Red | - | Engagement loops |
| ⚙️ Build-Bot | System Developer | Blue | - | Automation |
| 🧩 Pipe-Layer | Pipeline Engineer | Teal | - | Workflows |
| 💻 Code-Crafter | Implementation | Indigo | qwen3-coder-next | Code generation |
| 🛡️ Shield-Bot | Security Guard | Orange | - | Risk assessment |
| 🗺️ Map-Maker | Strategy Lead | Lime | - | Planning |
| 🚀 Launch-Pad | Deployment Chief | Purple | - | Go-live |

---

## API Endpoints Summary

```
POST   /api/v1/tasks           → Create new task (GitHub issue)
GET    /api/v1/agents          → List all agents with status
GET    /api/v1/tasks/{id}      → Get task status and progress
WS     /ws                     → Real-time updates (WebSocket)
```

---

## File Location

📄 **Full Specification:** `design/issue-20-agent-task-modal-spec.md`

---

## Next Phase

**Build Phase** → Issue #21
- **Agent:** 💻 Code-Crafter
- **Model:** qwen3-coder-next:cloud
- **Focus:** Component implementation, API integration, testing

---

**Design phase complete. Ready for build phase.** 🎨
