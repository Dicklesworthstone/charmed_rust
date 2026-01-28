# Wizard Page Design Specification

> Detailed design for the multi-step wizard workflow (bd-1ynf)

## Overview

The Wizard page demonstrates `huh` form integration by guiding users through a realistic "Deploy a Service" workflow. Each step maps to specific huh components and produces meaningful state changes in the app.

---

## Narrative: "Deploy a Service"

The wizard walks users through deploying a new service to the fictional Charmed Cloud platform. This narrative was chosen because it:
- Naturally requires multiple distinct steps
- Uses all major huh component types
- Produces visible state changes (new service appears in Services page)
- Includes validation, confirmation, and async progress

---

## Step-by-Step Design

### Step 1: Service Selection

**Purpose**: Choose what type of service to deploy.

**huh Components**:
- `Select<ServiceType>` - Single-select dropdown

**Options**:
| Option | Description |
|--------|-------------|
| Web Service | HTTP server with port binding |
| Background Worker | Queue processor, no external ports |
| Scheduled Job | Cron-style recurring task |

**State Changes**:
- `wizard.service_type: Option<ServiceType>` set to selected value
- Determines which fields appear in Step 2

**Validation**:
- Required selection (cannot proceed with None)

**UX Notes**:
- Pre-select first option as default
- Show brief description for each option

---

### Step 2: Basic Configuration

**Purpose**: Configure service identity and runtime settings.

**huh Components**:
- `Input` - Service name (text)
- `Input` - Description (text)
- `Select<Environment>` - Target environment

**Fields**:

| Field | Component | Validation | Default |
|-------|-----------|------------|---------|
| Name | `Input` | Required, alphanumeric + hyphens, 3-40 chars | "" |
| Description | `Input` | Optional, max 200 chars | "" |
| Environment | `Select` | Required | "staging" |

**Environment Options**:
- `development` - Local testing
- `staging` - Pre-production
- `production` - Live environment (shows warning)

**State Changes**:
- `wizard.name: String`
- `wizard.description: String`
- `wizard.environment: Environment`

**Validation**:
- Name uniqueness check (simulated)
- Production environment triggers confirmation in Step 4

---

### Step 3: Type-Specific Options

**Purpose**: Configure options specific to the service type chosen in Step 1.

**Dynamic content based on `service_type`**:

#### If Web Service:

| Field | Component | Validation | Default |
|-------|-----------|------------|---------|
| Port | `Input` | Required, 1024-65535 | "8080" |
| Health Check Path | `Input` | Required, starts with "/" | "/health" |
| Replicas | `Select` | Required | "2" |

**huh Components**: 2x `Input`, 1x `Select`

#### If Background Worker:

| Field | Component | Validation | Default |
|-------|-----------|------------|---------|
| Queue Name | `Input` | Required, alphanumeric | "default" |
| Concurrency | `Select` | Required | "4" |
| Max Retries | `Select` | Required | "3" |

**huh Components**: 1x `Input`, 2x `Select`

#### If Scheduled Job:

| Field | Component | Validation | Default |
|-------|-----------|------------|---------|
| Schedule | `Input` | Required, cron expression | "0 * * * *" |
| Timeout | `Select` | Required | "5m" |
| Run on Deploy | `Confirm` | Optional | false |

**huh Components**: 1x `Input`, 1x `Select`, 1x `Confirm`

**State Changes**:
- `wizard.type_config: TypeConfig` (enum with type-specific data)

---

### Step 4: Environment Variables

**Purpose**: Select environment variables to inject into the service.

**huh Components**:
- `MultiSelect<EnvVar>` - Environment variable selection

**Available Variables** (pre-defined for the fictional platform):

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis cache connection |
| `API_KEY` | Internal service API key |
| `LOG_LEVEL` | Logging verbosity (debug/info/warn/error) |
| `METRICS_ENDPOINT` | Prometheus push gateway |
| `SENTRY_DSN` | Error tracking endpoint |

**State Changes**:
- `wizard.env_vars: Vec<EnvVar>` - List of selected variables

**Validation**:
- No minimum required (service can run without extras)
- Soft warning if `LOG_LEVEL` not selected

**UX Notes**:
- Show checkboxes with [x] for selected
- j/k navigation, space to toggle
- "a" to select all, "n" to select none

---

### Step 5: Review & Confirm

**Purpose**: Summary of all configuration before deployment.

**huh Components**:
- `Note` - Read-only summary display
- `Confirm` - Final deployment confirmation

**Summary Display**:
```
╭─ Deploy Service ────────────────────────────────╮
│                                                 │
│  Service: my-api-service                        │
│  Type: Web Service                              │
│  Environment: production                        │
│                                                 │
│  Configuration:                                 │
│    Port: 8080                                   │
│    Health Check: /health                        │
│    Replicas: 2                                  │
│                                                 │
│  Environment Variables: 3 selected             │
│    DATABASE_URL, REDIS_URL, LOG_LEVEL          │
│                                                 │
╰─────────────────────────────────────────────────╯
```

**Confirmation**:
- Affirmative: "Deploy Now"
- Negative: "Go Back"

**State Changes**:
- `wizard.confirmed: bool` - Set to true on confirmation
- Triggers Step 6 transition

**UX Notes**:
- Production deployments show warning banner
- Display all settings in organized sections
- Allow editing by navigating back

---

### Step 6: Deployment Progress

**Purpose**: Show deployment animation and completion.

**Components** (custom, not huh):
- `Spinner` - Deployment progress indicator
- Status messages with checkmarks

**Progress Sequence** (simulated):
1. "Validating configuration..." (0.5s)
2. "Creating container image..." (1s)
3. "Provisioning resources..." (1s)
4. "Starting service..." (0.5s)
5. "Running health checks..." (0.5s)
6. "Deployment complete!"

**State Changes**:
- `wizard.deployment_status: DeploymentStatus`
  - `Pending` -> `InProgress(step)` -> `Complete(service_id)` | `Failed(error)`
- On completion: Add service to `app.services` list

**UX Notes**:
- Disable back navigation during deployment
- Show elapsed time
- On success: "Press Enter to view service" -> navigate to Services page
- On failure: Show error, offer "Retry" or "Go Back"

---

## State Management

### WizardState Struct

```rust
pub struct WizardState {
    /// Current step (0-indexed)
    pub step: usize,

    /// Step 1: Service type
    pub service_type: Option<ServiceType>,

    /// Step 2: Basic config
    pub name: String,
    pub description: String,
    pub environment: Environment,

    /// Step 3: Type-specific config
    pub type_config: Option<TypeConfig>,

    /// Step 4: Environment variables
    pub env_vars: Vec<EnvVar>,

    /// Step 5: Confirmation
    pub confirmed: bool,

    /// Step 6: Deployment
    pub deployment_status: DeploymentStatus,
}
```

### Step Transitions

| From | To | Trigger | Validation |
|------|-----|---------|------------|
| 0 | 1 | Enter/Next | service_type is Some |
| 1 | 2 | Enter/Next | name valid, environment set |
| 2 | 3 | Enter/Next | type_config valid |
| 3 | 4 | Enter/Next | Always valid |
| 4 | 5 | Enter/Next | confirmed = true |
| 5 | - | Complete | deployment_status = Complete |

**Back Navigation**: Allowed from steps 1-4. Step 5 (deployment) is terminal.

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Next step / Submit current field |
| `Escape` | Cancel wizard (confirm dialog) |
| `Backspace` / `b` | Previous step |
| `j` / `Down` | Next field in step |
| `k` / `Up` | Previous field in step |
| `Space` | Toggle selection (MultiSelect, Confirm) |
| `Tab` | Next field (alternative) |

---

## Visual Design

### Step Indicator

```
  ● Select Type  →  ○ Configure  →  ○ Options  →  ○ Variables  →  ○ Review  →  ○ Deploy
  ────────────────────────────────────────────────────────────────────────────────────────
```

- Filled circle (●) = current step
- Empty circle (○) = incomplete step
- Checkmark (✓) = completed step

### Layout

```
╭─ Deploy New Service ─────────────────── Step 1/6 ─╮
│                                                   │
│  What type of service do you want to deploy?      │
│                                                   │
│  > ● Web Service                                  │
│      Background Worker                            │
│      Scheduled Job                                │
│                                                   │
│  ─────────────────────────────────────────────    │
│  Web Service: HTTP server with port binding       │
│                                                   │
╰───────────────────────────────────────────────────╯
 Enter: Continue  Esc: Cancel
```

---

## Validation Strategy

### Inline Validation
- Fields validate on blur (moving to next field)
- Invalid fields show error message below input
- Cannot proceed to next step until current step is valid

### Validation Messages

| Field | Rule | Message |
|-------|------|---------|
| Name | Required | "Service name is required" |
| Name | Pattern | "Name must be alphanumeric with hyphens" |
| Name | Length | "Name must be 3-40 characters" |
| Port | Range | "Port must be between 1024 and 65535" |
| Schedule | Cron | "Invalid cron expression" |

---

## Integration Points

### App State
- Completed wizard creates new `Service` in `app.services`
- Service appears in Services page immediately
- Dashboard "Recent Activity" shows deployment event

### Navigation
- "View Service" after deployment navigates to Services page with new service selected
- Cancelled wizard returns to previous page

---

## Component Mapping Summary

| Step | huh Components Used |
|------|---------------------|
| 1 | `Select` |
| 2 | `Input` (x2), `Select` |
| 3 | `Input`, `Select`, `Confirm` (varies by type) |
| 4 | `MultiSelect` |
| 5 | `Note`, `Confirm` |
| 6 | (custom progress, not huh) |

**Total unique huh components demonstrated**: 5 (Input, Select, MultiSelect, Confirm, Note)

---

## Test Scenarios

1. **Happy Path**: Complete all steps, deploy successfully
2. **Validation**: Enter invalid name, verify error shown
3. **Back Navigation**: Go back from Step 3 to Step 2, verify state preserved
4. **Cancel**: Press Escape, confirm cancellation
5. **Production Warning**: Select production environment, verify warning shown
6. **Deployment Failure**: Simulate error, verify retry option

---

## Open Questions (for implementation)

1. Should the wizard support saving drafts?
2. Should there be a "template" feature to pre-fill common configurations?
3. How should the wizard interact with keyboard shortcuts that navigate pages (1-7)?

---

*This design satisfies bd-1ynf acceptance criteria: each step maps to a huh component and produces meaningful state changes.*
