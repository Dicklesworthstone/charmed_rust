# System Architecture

## Overview

Charmed Control Center monitors a microservices deployment.

```
                        ┌─────────────┐
                        │   Gateway   │
                        └──────┬──────┘
               ┌───────────────┼───────────────┐
               ▼               ▼               ▼
        ┌──────────┐    ┌──────────┐    ┌──────────┐
        │   API    │    │   Auth   │    │  Worker  │
        │ Service  │    │ Handler  │    │  Queue   │
        └────┬─────┘    └────┬─────┘    └────┬─────┘
             │               │               │
             └───────────────┼───────────────┘
                             ▼
                      ┌──────────────┐
                      │   Database   │
                      └──────────────┘
```

## Components

### API Service
- REST and gRPC endpoints
- Request validation
- Rate limiting

### Auth Handler
- OAuth 2.0 / OIDC
- Session management
- Token refresh

### Worker Queue
- Job scheduling
- Retry policies
- Dead letter handling

## Metrics Collected

| Metric | Unit | SLA Target |
|--------|------|------------|
| Request latency (P95) | ms | < 100ms |
| Error rate | % | < 0.1% |
| Throughput | req/s | > 10,000 |
| Uptime | % | > 99.9% |

## Health Checks

Each service exposes:

- `/health/ready` - Readiness probe
- `/health/live` - Liveness probe
- `/metrics` - Prometheus metrics
