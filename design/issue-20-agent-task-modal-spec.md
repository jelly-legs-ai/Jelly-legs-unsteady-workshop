# 🎨 Agent Task Modal - Design Specification

**Issue:** #20  
**Status:** 🟡 Design Phase Complete  
**Designer:** Sketch-Bot Agent 🎨  
**Date:** 2026-03-20  
**Parent:** Agent Factory v2.0 Dashboard  
**Phase:** Phase 2 - Design (Following Issue #2 Research)

---

## 📋 Executive Summary

This document provides comprehensive design specifications for the **Agent Task Modal** - a critical UI component of the Agent Factory v2.0 Dashboard that enables users to request assistance from the 12-agent AI team, delegate tasks, and track work progress in real-time.

The modal serves as the primary human-agent interaction interface, bridging the gap between the visual agent factory floor and the underlying autonomous GitHub issue management system.

---

## 🎯 Design Objectives

### Primary Goals

| Objective | Priority | Success Metric |
|-----------|----------|----------------|
| **Intuitive Agent Selection** | P0 | User can select an agent within 3 clicks |
| **Clear Task Delegation** | P0 | Task description → GitHub issue creation < 30s |
| **Real-time Status Feedback** | P1 | Live updates on task queue position |
| **Contextual Agent Information** | P1 | Agent availability/skills visible at-a-glance |
| **Mobile Responsiveness** | P2 | Full functionality on 375px+ screens |

### User Stories

1. **As a** user viewing the Agent Factory dashboard, **I want** to request help from a specific agent, **so that** I can delegate work to the autonomous AI team.

2. **As a** project manager, **I want** to see which agents are available and their current workload, **so that** I can make informed delegation decisions.

3. **As a** developer, **I want** to submit a task with technical requirements, **so that** the appropriate agent can create a GitHub issue and begin work.

4. **As a** team lead, **I want** to view the status of my submitted tasks, **so that** I can track progress without leaving the dashboard.

---

## 🏗️ System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AGENT TASK MODAL SYSTEM                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        PRESENTATION LAYER                            │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │  Agent       │  │  Task Form   │  │  Status      │              │   │
│  │  │  Selection   │  │  Component   │  │  Dashboard   │              │   │
│  │  │  Grid        │  │              │  │              │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  │                                                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │  Agent       │  │  Real-time   │  │  Notification│              │   │
│  │  │  Detail      │  │  Updates     │  │  Toast       │              │   │
│  │  │  Panel       │  │              │  │              │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        STATE MANAGEMENT                              │   │
│  │                                                                     │   │
│  │   ┌─────────────────┐      ┌─────────────────┐                     │   │
│  │   │  Modal State    │      │  Task Queue     │                     │   │
│  │   │  Machine        │      │  State          │                     │   │
│  │   │                 │      │                 │                     │   │
│  │   │ - Open/Close    │      │ - Pending       │                     │   │
│  │   │ - Step tracking │      │ - In Progress   │                     │   │
│  │   │ - Form data     │      │ - Completed     │                     │   │
│  │   │                 │      │ - Failed        │                     │   │
│  │   └────────┬────────┘      └────────┬────────┘                     │   │
│  │            │                        │                             │   │
│  │            ▼                        ▼                             │   │
│  │   ┌─────────────────────────────────────────┐                      │   │
│  │   │        Event Bus / Pub-Sub             │                      │   │
│  │   │                                        │                      │   │
│  │   │  modal:open  modal:close               │                      │   │
│  │   │  agent:selected  task:submitted         │                      │   │
│  │   │  status:updated  toast:show           │                      │   │
│  │   └─────────────────────────────────────────┘                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        API INTEGRATION LAYER                         │   │
│  │                                                                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │   │
│  │  │  GitHub      │  │  WebSocket   │  │  REST API    │              │   │
│  │  │  Issues API  │  │  Real-time   │  │  Fallback    │              │   │
│  │  │              │  │              │  │              │              │   │
│  │  │ - Create     │  │ - Agent      │  │ - Polling    │              │   │
│  │  │ - Update     │  │   activity   │  │   fallback   │              │   │
│  │  │ - Query      │  │ - Task       │  │ - Rate limit │              │   │
│  │  │              │  │   status     │  │   handling   │              │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     ORCHESTRATION LAYER                              │   │
│  │                                                                     │   │
│  │   ┌─────────────────────────────────────────────────────────┐      │   │
│  │   │            GitHub Actions / AI Orchestrator              │      │   │
│  │   │                                                           │      │   │
│  │   │  1. Receive task request from modal                       │      │   │
│  │   │  2. Validate and create GitHub issue                    │      │   │
│  │   │  3. Assign agent based on labels/requirements           │      │   │
│  │   │  4. Spawn agent sub-agent for execution                 │      │   │
│  │   │  5. Stream updates back to dashboard                    │      │   │
│  │   │                                                           │      │   │
│  │   └─────────────────────────────────────────────────────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow Architecture

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│  User   │────▶│  Modal  │────▶│   API   │────▶│ GitHub  │────▶│  Agent  │
│ Action  │     │  State  │     │  Layer  │     │ Issues  │     │ Spawned │
└─────────┘     └─────────┘     └─────────┘     └─────────┘     └─────────┘
     │               │               │               │               │
     │               │               │               │               │
     ▼               ▼               ▼               ▼               ▼
Click agent    Update state   POST /issues    Issue #N      Sub-agent
  avatar      → Selected      Create task    created       spawned
                                                     │
                     ▲                               │
                     │                               │
                     └───────────────────────────────┘
                     WebSocket update: "Agent assigned to #N"
```

---

## 🧩 Component Breakdown

### Component Hierarchy

```
AgentTaskModal (Root Container)
├── ModalOverlay
│   └── ModalContainer
│       ├── ModalHeader
│       │   ├── Title: "Request Agent Assistance"
│       │   ├── Subtitle: Dynamic based on step
│       │   └── CloseButton
│       │
│       ├── StepIndicator (Progress bar)
│       │   ├── Step 1: Select Agent
│       │   ├── Step 2: Describe Task
│       │   └── Step 3: Confirm & Submit
│       │
│       ├── StepContent (Dynamic)
│       │   ├── AgentSelectionStep
│       │   │   ├── FilterBar
│       │   │   │   ├── SearchInput
│       │   │   │   ├── RoleFilter
│       │   │   │   └── AvailabilityToggle
│       │   │   ├── AgentGrid
│       │   │   │   └── AgentCard (x12)
│       │   │   └── SelectedAgentPreview
│       │   │
│       │   ├── TaskDescriptionStep
│       │   │   ├── SelectedAgentBanner
│       │   │   ├── TaskTitleInput
│       │   │   ├── TaskDescriptionTextarea
│       │   │   ├── TaskTypeSelector
│       │   │   ├── PrioritySelector
│       │   │   └── TagsInput
│       │   │
│       │   └── ConfirmationStep
│       │       ├── TaskSummaryCard
│       │       ├── AgentConfirmation
│       │       ├── EstimatedTime
│       │       └── SubmitButton
│       │
│       └── ModalFooter
│           ├── BackButton (conditional)
│           ├── NextButton / SubmitButton
│           └── CancelButton
│
└── ToastContainer (for notifications)
```

---

## 🎨 UI/UX Design Specifications

### Visual Design System

#### Color Palette

```css
:root {
  /* Primary Colors */
  --color-primary: #6366f1;        /* Indigo 500 - Agent Factory brand */
  --color-primary-dark: #4f46e5;   /* Indigo 600 */
  --color-primary-light: #818cf8;  /* Indigo 400 */
  
  /* Agent-specific accent colors */
  --color-jellylegs: #ec4899;      /* Pink 500 */
  --color-datadiver: #06b6d4;      /* Cyan 500 */
  --color-patternseeker: #8b5cf6;  /* Violet 500 */
  --color-sketchbot: #f59e0b;      /* Amber 500 */
  --color-voiceweaver: #10b981;    /* Emerald 500 */
  --color-hookmaker: #ef4444;      /* Red 500 */
  --color-buildbot: #3b82f6;       /* Blue 500 */
  --color-pipelayer: #14b8a6;      /* Teal 500 */
  --color-codecrafter: #6366f1;    /* Indigo 500 */
  --color-shieldbot: #f97316;      /* Orange 500 */
  --color-mapmaker: #84cc16;       /* Lime 500 */
  --color-launchpad: #a855f7;      /* Purple 500 */
  
  /* Status Colors */
  --color-available: #10b981;      /* Green */
  --color-busy: #f59e0b;           /* Amber */
  --color-offline: #6b7280;        /* Gray */
  --color-error: #ef4444;          /* Red */
  
  /* Neutral Colors */
  --color-bg-primary: #0f172a;     /* Slate 900 */
  --color-bg-secondary: #1e293b;   /* Slate 800 */
  --color-bg-tertiary: #334155;    /* Slate 700 */
  --color-text-primary: #f8fafc;   /* Slate 50 */
  --color-text-secondary: #94a3b8; /* Slate 400 */
  --color-text-muted: #64748b;     /* Slate 500 */
  
  /* Semantic */
  --color-border: rgba(148, 163, 184, 0.2);
  --color-shadow: rgba(0, 0, 0, 0.5);
}
```

#### Typography

```css
:root {
  /* Font Families */
  --font-primary: 'Inter', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
  
  /* Font Sizes */
  --text-xs: 0.75rem;    /* 12px - Captions */
  --text-sm: 0.875rem;   /* 14px - Secondary text */
  --text-base: 1rem;     /* 16px - Body */
  --text-lg: 1.125rem;   /* 18px - Lead text */
  --text-xl: 1.25rem;    /* 20px - Small headings */
  --text-2xl: 1.5rem;    /* 24px - Modal title */
  --text-3xl: 1.875rem;  /* 30px - Page titles */
  
  /* Font Weights */
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  
  /* Line Heights */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.625;
}
```

#### Spacing & Layout

```css
:root {
  /* Spacing Scale */
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-10: 2.5rem;   /* 40px */
  --space-12: 3rem;     /* 48px */
  
  /* Border Radius */
  --radius-sm: 0.25rem;  /* 4px */
  --radius-md: 0.5rem;   /* 8px */
  --radius-lg: 0.75rem;  /* 12px */
  --radius-xl: 1rem;     /* 16px */
  --radius-full: 9999px;
  
  /* Modal Dimensions */
  --modal-max-width: 800px;
  --modal-max-height: 90vh;
  --modal-padding: var(--space-6);
  
  /* Grid */
  --agent-grid-columns: 3; /* Desktop */
  --agent-grid-gap: var(--space-4);
}
```

### Component Specifications

#### 1. Modal Container

```
┌─────────────────────────────────────────────────────────────────┐
│  Modal Overlay (rgba(0, 0, 0, 0.8))                             │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Modal Container                                          │  │
│  │  Background: --color-bg-primary                           │  │
│  │  Border: 1px solid --color-border                        │  │
│  │  Border-radius: --radius-xl                               │  │
│  │  Box-shadow: 0 25px 50px -12px --color-shadow           │  │
│  │  Max-width: 800px                                         │  │
│  │  Max-height: 90vh                                         │  │
│  │                                                           │  │
│  │  [Header]                                                 │  │
│  │  ──────────────────────────────────────────────────────   │  │
│  │  [Step Indicator]                                         │  │
│  │  ──────────────────────────────────────────────────────   │  │
│  │  [Content Area - scrollable]                              │  │
│  │  │                                                       │  │
│  │  │  Dynamic content based on step                        │  │
│  │  │                                                       │  │
│  │  │                                                       │  │
│  │  ──────────────────────────────────────────────────────   │  │
│  │  [Footer]                                                 │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

#### 2. Agent Card

```
┌──────────────────────────────────────────┐
│  Agent Card                              │
│  Background: --color-bg-secondary        │
│  Border: 2px solid transparent         │
│  Border-radius: --radius-lg            │
│  Padding: --space-4                      │
│  Hover: border-color: --color-primary  │
│  Selected: border-color: agent-accent    │
│                                          │
│  ┌────────┬────────────────────────────┐ │
│  │        │  [Status Indicator]        │ │
│  │        │  ● Online                 │ │
│  │ Avatar │                            │ │
│  │  48px  │  Agent Name               │ │
│  │  emoji │  ──────────────────────   │ │
│  │        │  Role: Research Lead       │ │
│  │        │  Model: deepseek-v3.2     │ │
│  │        │  Workload: ●●●○○ (3/5)    │ │
│  └────────┴────────────────────────────┘ │
│                                          │
│  [Specialties: Research, Analysis]      │
│                                          │
└──────────────────────────────────────────┘
```

**Agent Card States:**

| State | Visual Indicator | Interaction |
|-------|-----------------|-------------|
| **Available** | Green dot + pulse animation | Clickable, full opacity |
| **Busy** | Amber dot + loading spinner | Clickable, 90% opacity |
| **Offline** | Gray dot + dimmed | Disabled, 50% opacity |
| **Selected** | Colored border (agent-specific) + checkmark | Highlighted |

#### 3. Task Form

```
┌─────────────────────────────────────────────────────────────────┐
│  Task Description Form                                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Selected Agent: 🤿 Data-Diver (Research Lead)                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                                                           │  │
│  │  Task Title *                                             │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ RESEARCH: Analyze competitor pricing strategies     │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │  Help text: Keep it concise but descriptive             │  │
│  │                                                           │  │
│  │  Description *                                            │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │                                                     │  │  │
│  │  │ Research top 5 competitors in the DeFi space...     │  │  │
│  │  │                                                     │  │  │
│  │  │ - Compare pricing models                          │  │  │
│  │  │ - Identify market gaps                              │  │  │
│  │  │ - Document findings                                 │  │  │
│  │  │                                                     │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │  Character count: 247/2000                                │  │
│  │                                                           │  │
│  │  Task Type                Priority                       │  │
│  │  ┌──────────────┐         ┌──────────────┐              │  │
│  │  │ ▼ Research   │         │ ▼ Normal     │              │  │
│  │  └──────────────┘         └──────────────┘              │  │
│  │                                                           │  │
│  │  Tags (optional)                                          │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │ [defi] [pricing] [competitors] [+ Add tag]          │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 4. Step Indicator

```
┌─────────────────────────────────────────────────────────────────┐
│  Step Indicator                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━    │
│  ╱                                                           ╲   │
│ │    [1]         [2]         [3]                              │  │
│ │   ●───────○   ○───────○   ○                                │  │
│ │  Complete  Current   Pending                                │  │
│ │                                                            │  │
│ │  Select    Describe   Confirm                              │  │
│ │  Agent      Task       & Submit                            │  │
│  ╲                                                           ╱   │
│   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━    │
│                                                                 │
│  States:                                                        │
│  ● Complete (filled circle, accent color)                       │
│  ○ Current (filled circle, primary color + pulse)             │
│  ○ Pending (outline circle, muted color)                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Responsive Breakpoints

```
┌─────────────────────────────────────────────────────────────────┐
│  Desktop (≥1024px)                                              │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Modal: 800px wide                                         │  │
│  │ Agent Grid: 3 columns                                     │  │
│  │ Form: Side-by-side fields                                 │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Tablet (768px - 1023px)                                        │
│  ┌────────────────────────────────────────┐                      │
│  │ Modal: 90% viewport width             │                      │
│  │ Agent Grid: 2 columns                   │                      │
│  │ Form: Stacked fields                    │                      │
│  │                                         │                      │
│  └────────────────────────────────────────┘                      │
├─────────────────────────────────────────────────────────────────┤
│  Mobile (< 768px)                                               │
│  ┌─────────────────────┐                                       │
│  │ Modal: Full screen  │                                       │
│  │ Agent Grid: 1 column│                                       │
│  │ Form: Full width    │                                       │
│  │                       │                                       │
│  └─────────────────────┘                                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔌 API Design

### REST API Endpoints

#### Create Task (Primary)

```http
POST /api/v1/tasks
Content-Type: application/json
Authorization: Bearer {token}

{
  "agent_id": "data-diver",
  "title": "RESEARCH: Analyze competitor pricing strategies",
  "description": "Research top 5 competitors in the DeFi space...",
  "task_type": "research",
  "priority": "normal",
  "tags": ["defi", "pricing", "competitors"],
  "metadata": {
    "requested_by": "user_id",
    "source": "dashboard",
    "expected_output": "markdown_report"
  }
}
```

**Response:**

```http
HTTP/1.1 201 Created
Content-Type: application/json

{
  "task_id": "task_abc123",
  "issue_number": 42,
  "issue_url": "https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/42",
  "agent_assigned": "data-diver",
  "status": "queued",
  "estimated_start": "2026-03-20T20:00:00Z",
  "estimated_completion": "2026-03-20T22:00:00Z",
  "position_in_queue": 2,
  "created_at": "2026-03-20T19:45:00Z"
}
```

#### Get Agent Status

```http
GET /api/v1/agents
Authorization: Bearer {token}
```

**Response:**

```json
{
  "agents": [
    {
      "id": "data-diver",
      "name": "Data-Diver",
      "emoji": "🤿",
      "role": "Research Lead",
      "model": "deepseek-v3.2:cloud",
      "status": "available",
      "workload": {
        "active": 2,
        "capacity": 5,
        "queue_position": null
      },
      "specialties": ["research", "data-analysis", "market-research"],
      "avg_task_duration": "2h 15m",
      "success_rate": 0.94,
      "last_active": "2026-03-20T19:30:00Z"
    },
    {
      "id": "sketch-bot",
      "name": "Sketch-Bot",
      "emoji": "🎨",
      "role": "Design Architect",
      "model": "qwen3.5:397b-cloud",
      "status": "busy",
      "workload": {
        "active": 5,
        "capacity": 5,
        "queue_position": 3
      },
      "specialties": ["design", "architecture", "documentation"],
      "avg_task_duration": "4h 30m",
      "success_rate": 0.97,
      "last_active": "2026-03-20T19:40:00Z"
    }
  ],
  "updated_at": "2026-03-20T19:45:00Z"
}
```

#### Get Task Status

```http
GET /api/v1/tasks/{task_id}
Authorization: Bearer {token}
```

**Response:**

```json
{
  "task_id": "task_abc123",
  "issue_number": 42,
  "issue_url": "https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/42",
  "agent": {
    "id": "data-diver",
    "name": "Data-Diver",
    "emoji": "🤿"
  },
  "status": "in_progress",
  "status_history": [
    {
      "status": "queued",
      "timestamp": "2026-03-20T19:45:00Z",
      "message": "Task queued, waiting for agent"
    },
    {
      "status": "in_progress",
      "timestamp": "2026-03-20T20:15:00Z",
      "message": "Agent started working on task"
    }
  ],
  "progress": {
    "percent": 35,
    "current_step": "analyzing_competitor_2_of_5",
    "message": "Currently analyzing Uniswap pricing model"
  },
  "created_at": "2026-03-20T19:45:00Z",
  "started_at": "2026-03-20T20:15:00Z",
  "estimated_completion": "2026-03-20T22:00:00Z",
  "output_preview": null
}
```

### WebSocket Events

#### Connection

```javascript
// Client connects to WebSocket
const ws = new WebSocket('wss://api.jelly-legs.ai/ws');

// Authenticate
ws.send(JSON.stringify({
  type: 'auth',
  token: 'user_jwt_token'
}));
```

#### Subscribe to Task Updates

```javascript
// Subscribe to task updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'task:task_abc123'
}));
```

#### Incoming Events

```javascript
// Task status update
{
  "type": "task_update",
  "task_id": "task_abc123",
  "data": {
    "status": "in_progress",
    "progress": {
      "percent": 50,
      "current_step": "compiling_findings",
      "message": "Compiling research findings into report"
    },
    "timestamp": "2026-03-20T21:00:00Z"
  }
}

// Agent activity
{
  "type": "agent_activity",
  "agent_id": "data-diver",
  "data": {
    "activity": "task_completed",
    "task_id": "task_abc123",
    "message": "Research task completed successfully",
    "timestamp": "2026-03-20T21:45:00Z"
  }
}

// Queue update
{
  "type": "queue_update",
  "data": {
    "agent_id": "sketch-bot",
    "queue_position": 2,
    "estimated_wait": "15m"
  }
}
```

### Error Handling

```json
{
  "error": {
    "code": "AGENT_UNAVAILABLE",
    "message": "Selected agent is currently offline",
    "details": {
      "agent_id": "sketch-bot",
      "suggested_alternatives": ["voice-weaver", "hook-maker"]
    }
  }
}
```

**Error Codes:**

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_AGENT` | 400 | Agent ID not found |
| `AGENT_UNAVAILABLE` | 503 | Agent offline or at capacity |
| `VALIDATION_ERROR` | 400 | Missing or invalid fields |
| `RATE_LIMITED` | 429 | Too many requests |
| `GITHUB_API_ERROR` | 502 | GitHub API failure |

---

## 📊 State Machine

### Modal State Machine

```
                    ┌─────────────────┐
                    │     CLOSED      │
                    │  (Initial)      │
                    └────────┬────────┘
                             │ open()
                             ▼
                    ┌─────────────────┐
                    │    OPENING      │
                    │  (Animation)    │
                    └────────┬────────┘
                             │ animation complete
                             ▼
            ┌────────────────────────────────┐
            │          OPENED                │
            │  ┌──────────────────────────┐  │
            │  │ Step 1: AGENT_SELECTION  │  │◄────┐
            │  │   - Select agent         │  │     │
            │  │   - Filter/search        │  │     │
            │  │   - View details         │  │     │
            │  └───────────┬──────────────┘  │     │
            │              │ select()        │     │
            │              ▼                 │     │
            │  ┌──────────────────────────┐  │     │
            │  │ Step 2: TASK_FORM        │  │     │
            │  │   - Fill task details    │  │─────┘
            │  │   - Validation           │  │ back()
            │  └───────────┬──────────────┘  │
            │              │ validate()      │
            │              ▼                 │
            │  ┌──────────────────────────┐  │
            │  │ Step 3: CONFIRMATION     │  │─────┐
            │  │   - Review task          │  │     │
            │  │   - Submit               │  │     │
            │  └───────────┬──────────────┘  │     │
            │              │ submit()        │     │
            │              ▼                 │     │
            │  ┌──────────────────────────┐  │     │
            │  │   SUBMITTING             │  │     │
            │  │   (API call in progress) │  │     │
            │  └───────────┬──────────────┘  │     │
            │              │ success / error │     │
            │              ▼                 │     │
            │  ┌──────────────────────────┐  │     │
            │  │   SUCCESS / ERROR        │  │─────┘
            │  │   (Show result)          │  │ close()
            │  └───────────┬──────────────┘  │
            └──────────────┼──────────────────┘
                           │ close()
                           ▼
                    ┌─────────────────┐
                    │    CLOSING      │
                    │  (Animation)    │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │     CLOSED      │
                    └─────────────────┘
```

### Task Lifecycle State Machine

```
┌─────────────┐
│   DRAFT     │ (Client-side only)
└──────┬──────┘
       │ create()
       ▼
┌─────────────┐
│   QUEUED    │
│  (Waiting   │
│   for agent)│
└──────┬──────┘
       │ agent_assigned
       ▼
┌─────────────┐     ┌─────────────┐
│  ASSIGNED   │────▶│   FAILED    │
│  (Agent     │reject│ (Assignment │
│   notified) │     │   failed)   │
└──────┬──────┘     └─────────────┘
       │ agent_start
       ▼
┌─────────────┐     ┌─────────────┐
│ IN_PROGRESS │────▶│   BLOCKED   │
│  (Working)  │block│ (Needs      │
│             │     │  input)     │
└──────┬──────┘     └──────┬──────┘
       │                   │ unblock
       │ complete          ▼
       │            ┌─────────────┐
       │            │ IN_PROGRESS │
       │            │ (Resumed)   │
       │            └─────────────┘
       │
       ▼
┌─────────────┐
│   REVIEW    │ (Quality check)
└──────┬──────┘
       │ approve / reject
       ▼
┌─────────────┐     ┌─────────────┐
│   COMPLETE  │     │   REVISION  │
│             │     │  REQUESTED  │
└─────────────┘     └──────┬──────┘
                           │
                           └────────┐
                                    ▼
                           ┌─────────────┐
                           │ IN_PROGRESS │
                           │ (Revision)  │
                           └─────────────┘
```

---

## 🎭 Animations & Interactions

### Modal Transitions

| Transition | Duration | Easing | Effect |
|------------|----------|--------|--------|
| **Open** | 300ms | `cubic-bezier(0.16, 1, 0.3, 1)` | Scale 0.95→1, opacity 0→1 |
| **Close** | 200ms | `cubic-bezier(0.4, 0, 0.2, 1)` | Scale 1→0.95, opacity 1→0 |
| **Step Change** | 250ms | `cubic-bezier(0.4, 0, 0.2, 1)` | Slide left/right + fade |
| **Agent Select** | 150ms | `ease-out` | Border color transition + scale 1.02 |

### Micro-interactions

```css
/* Agent Card Hover */
.agent-card {
  transition: transform 150ms ease-out,
              border-color 150ms ease-out,
              box-shadow 150ms ease-out;
}

.agent-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.2);
}

.agent-card.selected {
  border-color: var(--agent-accent-color);
  box-shadow: 0 0 0 4px var(--agent-accent-color-alpha-20);
}

/* Status Indicator Pulse */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-indicator.available {
  animation: pulse 2s ease-in-out infinite;
}

/* Step Progress */
@keyframes step-complete {
  0% { transform: scale(1); }
  50% { transform: scale(1.2); }
  100% { transform: scale(1); }
}

.step-circle.complete {
  animation: step-complete 300ms ease-out;
}

/* Submit Button Loading */
@keyframes spin {
  to { transform: rotate(360deg); }
}

.btn-submit.loading::after {
  content: '';
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
```

### Keyboard Navigation

| Key | Action |
|-----|--------|
| `Esc` | Close modal |
| `Tab` | Navigate between fields |
| `Shift + Tab` | Navigate backwards |
| `Enter` | Submit form / Next step |
| `Arrow Keys` | Navigate agent grid |
| `Space` | Select agent |

---

## 🔐 Security Considerations

### Authentication & Authorization

1. **JWT Token**: All API requests require valid JWT token
2. **CSRF Protection**: Modal includes CSRF token in forms
3. **Rate Limiting**: 
   - Max 10 tasks per hour per user
   - Max 3 concurrent tasks per user
4. **Input Sanitization**: All user inputs sanitized before GitHub API calls

### Data Validation

| Field | Validation Rules |
|-------|------------------|
| **Title** | Required, 10-120 characters, no special chars except `- _ :` |
| **Description** | Required, 50-5000 characters, Markdown allowed |
| **Agent ID** | Must be valid agent from whitelist |
| **Tags** | Max 5 tags, alphanumeric + hyphens only |
| **Priority** | One of: `low`, `normal`, `high`, `urgent` |

---

## 📱 Accessibility (a11y)

### WCAG 2.1 AA Compliance

1. **Keyboard Navigation**: Full keyboard accessibility
2. **Focus Management**: 
   - Initial focus on first interactive element
   - Trap focus within modal when open
   - Return focus to trigger element on close
3. **ARIA Labels**:
   - `role="dialog"` on modal container
   - `aria-labelledby` pointing to modal title
   - `aria-describedby` for descriptions
   - `aria-live="polite"` for status updates
4. **Color Contrast**: All text meets 4.5:1 ratio
5. **Screen Reader Support**: Status announcements for all updates

---

## 🧪 Testing Strategy

### Unit Tests

- Component rendering with different props
- State machine transitions
- Form validation logic
- API integration mocking

### Integration Tests

- End-to-end task creation flow
- WebSocket connection and events
- Error handling scenarios
- Rate limiting behavior

### E2E Tests

- Complete user journey: open modal → select agent → submit → verify issue created
- Cross-browser compatibility
- Mobile responsiveness

---

## 📦 Deliverables Checklist

### Design Deliverables

- [x] System architecture diagram
- [x] Component breakdown
- [x] UI/UX design specifications
- [x] Color palette and design tokens
- [x] Responsive layout specifications
- [x] Animation and interaction specifications

### Technical Deliverables

- [x] API design (REST + WebSocket)
- [x] State machine definitions
- [x] Data flow documentation
- [x] Error handling specifications
- [x] Security considerations
- [x] Accessibility requirements

### Implementation Deliverables

- [ ] React/Vue component structure
- [ ] CSS/Tailwind configuration
- [ ] API client implementation
- [ ] State management (Redux/Zustand)
- [ ] Unit test suite
- [ ] Storybook stories

---

## 🚀 Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Set up component library and design tokens
- [ ] Implement Modal container and overlay
- [ ] Build StepIndicator component

### Phase 2: Agent Selection (Week 2)
- [ ] Create AgentCard component
- [ ] Implement AgentGrid with filtering
- [ ] Connect to agent status API

### Phase 3: Task Form (Week 3)
- [ ] Build TaskDescriptionStep form
- [ ] Implement validation logic
- [ ] Add file attachment support (optional)

### Phase 4: Integration (Week 4)
- [ ] Connect to GitHub Issues API
- [ ] Implement WebSocket for real-time updates
- [ ] Add error handling and retry logic

### Phase 5: Polish (Week 5)
- [ ] Animations and transitions
- [ ] Accessibility audit and fixes
- [ ] Mobile responsiveness optimization
- [ ] Performance optimization

---

## 🔄 Next Phase

**Phase 3: Build** - Issue #21
- **Agent:** 💻 Code-Crafter
- **Model:** qwen3-coder-next:cloud
- **Focus:** Component implementation, API integration, testing

**Labels for next issue:** `build`, `frontend`, `dashboard`, `agent-factory`

---

**Design phase complete. Ready for build phase.** 🎨

*Sketch-Bot Agent, Jelly-legs AI Team*  
*Date: 2026-03-20*
