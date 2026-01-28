# Charmed Control Center - Product Concept

> demo_showcase UX Architecture and Information Design

## Product Vision

**Charmed Control Center** is an ops-style console for a fictional cloud platform called "Charmed Cloud". It demonstrates every capability of the charmed_rust TUI stack while providing a cohesive, realistic product experience.

The showcase proves that charmed_rust can build production-grade TUIs by implementing features users expect from real ops consoles: service dashboards, job monitoring, log viewers, documentation browsers, interactive wizards, and settings management.

---

## Information Architecture

### Primary Navigation (Sidebar)

| Page | Icon | Description | Components Demonstrated |
|------|------|-------------|------------------------|
| **Dashboard** | `[]` | At-a-glance platform health: service status grid, resource metrics, recent activity | `list`, `spinner`, `progress`, `table` |
| **Services** | `>_` | Service catalog with status, health checks, quick actions | `list` (filterable), `table`, `textinput` |
| **Jobs** | `>>` | Background jobs/tasks: queued, running, completed, failed | `table`, `spinner`, `progress`, `timer` |
| **Logs** | `#` | Aggregated log viewer with filtering, levels, search | `viewport`, `textinput`, `list` |
| **Docs** | `?` | Markdown documentation browser (embedded help) | `viewport`, `glamour`, file navigation |
| **Wizard** | `*` | Multi-step workflow: create service, configure, deploy | `huh` forms, `textinput`, confirmation |
| **Settings** | `@` | Theme selection, preferences, about/diagnostics | `list`, toggle selections, `textinput` |

### Layout Primitives (App Chrome)

```
+------------------------------------------------------------------+
| HEADER: Charmed Control Center      Status: Connected   HH:MM:SS |
+----------+-------------------------------------------------------+
|          |                                                       |
| SIDEBAR  |                    MAIN CONTENT                       |
|          |                                                       |
|  > Dash  |  [Page-specific content rendered here]                |
|    Svc   |                                                       |
|    Jobs  |                                                       |
|    Logs  |                                                       |
|    Docs  |                                                       |
|    Wiz   |                                                       |
|    Set   |                                                       |
|          |                                                       |
+----------+-------------------------------------------------------+
| FOOTER: j/k navigate  Enter select  q quit  ? help              |
+------------------------------------------------------------------+
```

**Header**: Platform name, connection status indicator, clock (demonstrates `timer`)
**Sidebar**: Navigation list with current page highlighted (demonstrates `list`)
**Main Content**: Page-specific rendering area
**Footer**: Context-sensitive key hints (demonstrates `help` component)

---

## Page Specifications

### 1. Dashboard

**Goal**: Provide at-a-glance platform health and recent activity.

**Components**:
- Service status grid (4x2 boxes showing service health with colored indicators)
- Resource metrics (CPU, Memory, Network) with `progress` bars
- Recent activity list (last 5 events with timestamps)
- Animated status indicator using `spinner`

**What makes it "real"**:
- Simulated data updates every few seconds (demonstrates async commands)
- Status colors change based on thresholds
- Clicking a service navigates to Services page (demonstrates routing)

---

### 2. Services

**Goal**: Browse, filter, and manage the service catalog.

**Components**:
- Filterable `list` of services with fuzzy search
- Detail panel showing selected service info
- Quick action menu (restart, scale, view logs)
- `table` view toggle for dense information

**What makes it "real"**:
- Type to filter services in real-time
- Status badges with semantic colors
- Action confirmation dialogs

---

### 3. Jobs

**Goal**: Monitor background jobs and their progress.

**Components**:
- `table` with columns: ID, Name, Status, Progress, Duration, Started
- `spinner` for running jobs
- `progress` bars for jobs with known completion %
- `timer` showing elapsed time
- Status filtering (All, Running, Completed, Failed)

**What makes it "real"**:
- Jobs tick forward in real-time
- Progress bars animate smoothly
- Failed jobs show error details on selection

---

### 4. Logs

**Goal**: Aggregate log viewer with filtering and search.

**Components**:
- `viewport` for scrollable log content
- `textinput` for search/filter
- Level filter buttons (DEBUG, INFO, WARN, ERROR)
- Styled log lines with timestamp, level, message

**What makes it "real"**:
- Search highlights matches
- Level filtering reduces visible logs
- Auto-scroll with manual scroll lock
- Styled output using `lipgloss` (colored levels)

---

### 5. Docs

**Goal**: Browse embedded markdown documentation.

**Components**:
- File tree/list for doc navigation
- `glamour` rendered markdown in `viewport`
- Breadcrumb navigation
- Search within docs

**What makes it "real"**:
- Actual markdown rendering with syntax highlighting
- Links navigate between docs
- Table of contents extraction

---

### 6. Wizard

**Goal**: Multi-step workflow demonstrating `huh` form integration.

**Components**:
- `huh::Form` with multiple steps
- Text inputs, selects, confirms
- Progress indicator showing current step
- Review/summary before final action

**Workflow Steps**:
1. **Name Service**: Text input with validation
2. **Select Type**: Single-select from options (Web, Worker, Cron)
3. **Configure**: Type-specific options (port, schedule, etc.)
4. **Environment**: Multi-select env variables to inject
5. **Review**: Summary of all choices
6. **Deploy**: Confirmation with animated deployment progress

**What makes it "real"**:
- Validation feedback on inputs
- Back/Next navigation
- Final action shows spinner then success

---

### 7. Settings

**Goal**: User preferences and application diagnostics.

**Components**:
- Theme selector (`list` with live preview)
- Toggle options (animations, sounds, etc.)
- About section with version, credits
- Diagnostics: terminal size, color depth, capabilities

**What makes it "real"**:
- Theme changes apply immediately
- Keyboard shortcuts customization
- Export/import settings (future)

---

## Core User Journeys

### Journey 1: Check Platform Health
1. Launch app -> Dashboard loads
2. See service status grid (all green = healthy)
3. Notice elevated CPU on one service
4. Click service -> navigate to Services page
5. View details, decide to take action or monitor

### Journey 2: Investigate Failed Job
1. Dashboard shows "1 Failed Job" in recent activity
2. Navigate to Jobs page
3. Filter by "Failed" status
4. Select failed job -> see error details
5. Click "View Logs" -> navigate to Logs page filtered by job ID

### Journey 3: Deploy New Service
1. Navigate to Wizard
2. Complete multi-step form
3. Review configuration
4. Confirm deployment
5. Watch progress animation
6. Success -> option to view in Services

### Journey 4: Find Documentation
1. Navigate to Docs
2. Browse file tree or search
3. Read rendered markdown
4. Follow internal links
5. Use viewport to scroll long docs

---

## Component Usage Summary

| Component | Pages Using It |
|-----------|---------------|
| `list` | Dashboard, Services, Docs, Settings |
| `table` | Services, Jobs |
| `viewport` | Logs, Docs |
| `textinput` | Services, Logs, Wizard |
| `spinner` | Dashboard, Jobs, Wizard |
| `progress` | Dashboard, Jobs, Wizard |
| `timer` | Header, Jobs, Stopwatch demo |
| `help` | Footer (all pages) |
| `huh::Form` | Wizard |
| `glamour` | Docs |

---

## Technical Notes

### Routing
- Single `AppModel` with `CurrentPage` enum
- Page models are nested components
- Message routing via pattern matching

### Theming
- Semantic color tokens (primary, success, warning, error, muted)
- Three presets: Dark (default), Light, Dracula
- All styles use theme tokens, not hardcoded colors

### Data Simulation
- `DataSource` trait for mock data generation
- Tick-based updates for "live" feel
- Configurable update intervals

### Keyboard Model
- Global shortcuts (q quit, ? help, 1-7 page jump)
- Page-specific shortcuts delegated to page models
- Footer hint bar updates per page
