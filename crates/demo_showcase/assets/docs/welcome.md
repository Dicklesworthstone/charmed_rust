# Welcome to Charmed Control Center

Your unified platform for infrastructure observability and operations.

> **Note**: This showcase demonstrates the [charmed_rust](https://github.com/charmed-rust) TUI
> framework capabilities. All data is simulated for demonstration purposes.

## Getting Started

Navigate using keyboard shortcuts or the sidebar:

| Key | Action | Context |
|-----|--------|---------|
| `1-7` | Switch pages | Global |
| `[` | Toggle sidebar | Global |
| `j/k` | Navigate lists | Lists/Tables |
| `Enter` | Select/confirm | Interactive |
| `Esc` | Back/unfocus | All pages |
| `?` | Help overlay | Global |
| `t` | Cycle theme | Global |
| `q` | Quit | Global |

### Quick Actions

Use these shortcuts for common operations:

- **Filter**: Press `/` to open the filter bar
- **Sort**: Press `s` to cycle sort columns, `S` for reverse
- **Refresh**: Press `r` to refresh data
- **Export**: Press `e` for text, `E` for HTML export

## Dashboard

The dashboard provides at-a-glance health metrics:

- **Live Metrics**: Request rate, latency, error rate
- **Service Status**: Health of all registered services
- **Recent Activity**: Latest deployments and jobs
- **Alerts**: Active warnings and incidents

### Metric Cards

Each metric displays:

```
┌─────────────────────┐
│ Requests/s          │
│ ━━━━━━━━━━━━━━━━━━  │
│ 12,847    (+3.2%)   │
└─────────────────────┘
```

## Services

Monitor your microservices infrastructure:

| Service | Status | Uptime | Last Deploy |
|---------|--------|--------|-------------|
| api-gateway | ● Healthy | 7d 23h | 2h ago |
| auth-handler | ● Healthy | 14d 5h | 1d ago |
| billing-worker | ◐ Degraded | 3d 12h | 4h ago |
| cache-proxy | ● Healthy | 30d 0h | 5d ago |

> **Tip**: Use the status filter (`1-4`) to show only services with specific health states.

### Service Configuration

Services are configured via TOML:

```toml
[service]
name = "api-gateway"
port = 8080
workers = 4

[health_check]
interval_seconds = 30
timeout_seconds = 5
path = "/health/ready"

[metrics]
enabled = true
port = 9090
path = "/metrics"
```

## Jobs

Track and manage background tasks:

```
database-backup-001    [==========] 100%  done
log-rotation-042       [=====>    ]  55%  running
cache-warmup-013       [          ]   0%  queued
security-scan-007      [===       ]  30%  running
```

### Job Lifecycle

Jobs progress through these states:

1. **Queued** → Waiting for resources
2. **Running** → Actively executing
3. **Completed** → Finished successfully
4. **Failed** → Error occurred (retryable)
5. **Cancelled** → User-initiated stop

Use `/` to filter, `Enter` to view details.

### Creating Jobs

Jobs can be triggered via the API:

```rust
use demo_showcase::data::Job;

let job = Job::new("database-backup")
    .with_schedule("0 0 * * *")  // Daily at midnight
    .with_timeout(Duration::from_secs(3600))
    .with_retry_policy(RetryPolicy::exponential(3));

scheduler.submit(job).await?;
```

## Logs

Real-time log streaming with color-coded levels:

| Level | Color | Description |
|-------|-------|-------------|
| `TRACE` | Dim gray | Verbose debugging |
| `DEBUG` | Gray | Debug information |
| `INFO` | Blue | Normal operation |
| `WARN` | Yellow | Attention needed |
| `ERROR` | Red | Requires action |

### Sample Output

```log
2024-01-15T10:23:45.123Z INFO  api-gateway: Request received path=/api/users method=GET
2024-01-15T10:23:45.156Z DEBUG auth-handler: Token validated user_id=42
2024-01-15T10:23:45.201Z INFO  api-gateway: Response sent status=200 latency_ms=78
2024-01-15T10:23:46.001Z WARN  billing-worker: High queue depth depth=1523 threshold=1000
2024-01-15T10:23:47.892Z ERROR cache-proxy: Connection timeout host=redis-01 timeout_ms=5000
```

### Log Filtering

Filter logs with these keyboard shortcuts:

- `1` Toggle ERROR level
- `2` Toggle WARN level
- `3` Toggle INFO level
- `4` Toggle DEBUG level
- `5` Toggle TRACE level
- `/` Text search
- `c` Clear all filters

Press `f` to toggle follow mode.

## Wizard

Step-by-step workflows for common operations:

1. **Configure** → Set deployment parameters
2. **Validate** → Review configuration
3. **Deploy** → Execute with progress tracking
4. **Verify** → Confirm successful deployment

> **Warning**: Production deployments require approval from a team lead.

### Deployment Configuration

```yaml
deployment:
  name: api-gateway
  environment: production
  replicas: 4

strategy:
  type: rolling
  max_surge: 25%
  max_unavailable: 1

health_check:
  initial_delay: 10s
  period: 5s
  failure_threshold: 3
```

## Keyboard Reference

### Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Quit immediately |
| `?` | Toggle help overlay |
| `[` | Toggle sidebar |
| `t` | Cycle theme (Dark → Light → Dracula) |

### Navigation

| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### Page Shortcuts

| Key | Page |
|-----|------|
| `1` | Dashboard |
| `2` | Services |
| `3` | Jobs |
| `4` | Logs |
| `5` | Docs |
| `6` | Wizard |
| `7` | Settings |

---

## Technical Details

### Framework Stack

Charmed Control Center is built with:

- **bubbletea** - Elm-architecture TUI framework
- **lipgloss** - Terminal styling and layout
- **bubbles** - Reusable UI components
- **glamour** - Markdown rendering
- **harmonica** - Spring animations

### Data Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  Input   │ ──▶ │  Update  │ ──▶ │   View   │
│  Events  │     │  Model   │     │  Render  │
└──────────┘     └──────────┘     └──────────┘
      ▲                                 │
      └─────────────────────────────────┘
```

### Performance

The application targets:

- **60 FPS** render loop
- **< 16ms** per frame budget
- **< 100MB** memory footprint
- **Zero allocations** in hot paths

---

*Built with [charmed_rust](https://github.com/charmed-rust) TUI framework*
