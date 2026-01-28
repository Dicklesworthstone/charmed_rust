# Welcome to Charmed Control Center

Your unified platform for infrastructure observability and operations.

## Getting Started

Navigate using keyboard shortcuts or the sidebar:

| Key | Action |
|-----|--------|
| `1-7` | Switch pages |
| `[` | Toggle sidebar |
| `j/k` | Navigate lists |
| `Enter` | Select/confirm |
| `Esc` | Back/unfocus |
| `?` | Help overlay |
| `q` | Quit |

## Dashboard

The dashboard provides at-a-glance health metrics:

- **Live Metrics**: Request rate, latency, error rate
- **Service Status**: Health of all registered services
- **Recent Activity**: Latest deployments and jobs

## Jobs

Track and manage background tasks:

```
database-backup-001    [==========] 100%  done
log-rotation-042       [=====>    ]  55%  running
cache-warmup-013       [          ]   0%  queued
```

Use `/` to filter, `Enter` to view details.

## Logs

Real-time log streaming with color-coded levels:

- **INFO**: Normal operation (blue)
- **WARN**: Attention needed (yellow)
- **ERROR**: Requires action (red)
- **DEBUG**: Verbose output (gray)

Press `f` to toggle follow mode.

## Wizard

Step-by-step workflows for common operations:

1. Configure deployment parameters
2. Review and validate
3. Execute with progress tracking

---

*Built with charmed_rust TUI framework*
