# System Architecture

> This document describes the architecture of the simulated microservices
> platform that Charmed Control Center monitors.

## Overview

Charmed Control Center monitors a microservices deployment consisting of
multiple interconnected services, each with specific responsibilities.

```
                        ┌─────────────────┐
                        │  Load Balancer  │
                        │    (ingress)    │
                        └────────┬────────┘
                                 │
            ┌────────────────────┼────────────────────┐
            │                    │                    │
            ▼                    ▼                    ▼
     ┌──────────┐         ┌──────────┐         ┌──────────┐
     │   API    │         │   Auth   │         │  Static  │
     │ Gateway  │ ◄─────▶ │ Handler  │         │  Assets  │
     └────┬─────┘         └────┬─────┘         └──────────┘
          │                    │
          │    ┌───────────────┴───────────────┐
          │    │                               │
          ▼    ▼                               ▼
     ┌──────────────┐                   ┌──────────────┐
     │   Billing    │                   │   Metrics    │
     │   Worker     │                   │  Collector   │
     └──────┬───────┘                   └──────┬───────┘
            │                                  │
            └──────────────┬───────────────────┘
                           │
                    ┌──────┴──────┐
                    │  Database   │
                    │  (Primary)  │
                    └─────────────┘
```

## Service Catalog

### Core Services

| Service | Port | Language | Owner |
|---------|------|----------|-------|
| api-gateway | 8080 | Rust | Platform Team |
| auth-handler | 8081 | Go | Security Team |
| billing-worker | 8082 | Python | Finance Team |
| cache-proxy | 6379 | Rust | Platform Team |
| metrics-collector | 9090 | Go | SRE Team |

### Support Services

| Service | Purpose | Dependencies |
|---------|---------|--------------|
| redis-cluster | Session cache | None |
| postgres-primary | Data store | None |
| kafka-broker | Event streaming | ZooKeeper |
| elasticsearch | Log aggregation | None |

## API Gateway

The API Gateway is the primary entry point for all external requests.

### Responsibilities

- **Routing**: Direct requests to appropriate backends
- **Rate limiting**: Protect downstream services
- **Authentication**: Validate tokens via auth-handler
- **Request transformation**: Normalize headers and payloads
- **Response caching**: Reduce backend load

### Configuration

```rust
use api_gateway::{Config, RateLimiter, Router};

let config = Config::builder()
    .bind_addr("0.0.0.0:8080")
    .worker_threads(4)
    .rate_limiter(RateLimiter::new()
        .requests_per_second(1000)
        .burst_size(50))
    .health_check_interval(Duration::from_secs(30))
    .build()?;

let router = Router::new()
    .route("/api/v1/*", backend("api-service"))
    .route("/auth/*", backend("auth-handler"))
    .route("/static/*", backend("static-assets"))
    .fallback(handler::not_found);

Server::new(config, router).run().await
```

### Rate Limiting

Rate limits are applied per-client:

| Tier | Requests/sec | Burst | Scope |
|------|--------------|-------|-------|
| Free | 10 | 20 | IP address |
| Pro | 100 | 200 | API key |
| Enterprise | 1000 | 2000 | API key |
| Internal | Unlimited | N/A | Service mesh |

## Auth Handler

Manages authentication and authorization for all services.

### Supported Methods

1. **OAuth 2.0** - Third-party authentication
2. **JWT** - Stateless tokens
3. **API Keys** - Service-to-service auth
4. **mTLS** - Certificate-based auth

### Token Lifecycle

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│ Request │ ──▶ │ Validate│ ──▶ │ Refresh │
│  Token  │     │  Token  │     │ if near │
└─────────┘     └────┬────┘     │  expiry │
                     │          └────┬────┘
              ┌──────▼──────┐       │
              │   Cache     │◀──────┘
              │  Validated  │
              │   Token     │
              └─────────────┘
```

### Token Claims

```json
{
  "sub": "user-42",
  "iss": "auth-handler",
  "iat": 1705312800,
  "exp": 1705316400,
  "roles": ["user", "admin"],
  "scopes": ["read:api", "write:api"],
  "metadata": {
    "org_id": "acme-corp",
    "tier": "enterprise"
  }
}
```

## Billing Worker

Processes payment transactions and generates invoices.

> **Note**: The billing worker uses an async job queue for reliability.
> Failed jobs are retried with exponential backoff.

### Job Types

| Job | Priority | Max Retries | Timeout |
|-----|----------|-------------|---------|
| invoice-generate | Normal | 3 | 5 min |
| payment-process | High | 5 | 2 min |
| subscription-renew | Normal | 3 | 5 min |
| refund-process | High | 5 | 2 min |
| report-generate | Low | 1 | 30 min |

### Processing Pipeline

```python
from billing import JobQueue, PaymentProcessor

queue = JobQueue("billing-jobs")
processor = PaymentProcessor()

@queue.handler("payment-process")
async def process_payment(job: Job) -> Result:
    # Validate payment details
    payment = await processor.validate(job.payload)

    # Process with payment provider
    result = await processor.charge(payment)

    # Update subscription status
    if result.success:
        await update_subscription(job.user_id, result)

    return result
```

## Metrics Collection

### Collected Metrics

All services expose Prometheus-compatible metrics:

| Metric | Type | Labels |
|--------|------|--------|
| `http_requests_total` | Counter | method, path, status |
| `http_request_duration_seconds` | Histogram | method, path |
| `active_connections` | Gauge | service |
| `error_rate` | Gauge | service, error_type |
| `queue_depth` | Gauge | queue_name |

### SLA Targets

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Request latency (P95) | < 100ms | > 200ms |
| Error rate | < 0.1% | > 1% |
| Throughput | > 10,000 req/s | < 5,000 req/s |
| Uptime | > 99.9% | < 99.5% |

### Alerting Rules

```yaml
groups:
  - name: sla-alerts
    rules:
      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 0.2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High P95 latency on {{ $labels.service }}"

      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Error rate above 1% on {{ $labels.service }}"
```

## Health Checks

Each service exposes standard health endpoints:

### Endpoints

| Path | Purpose | Response |
|------|---------|----------|
| `/health/live` | Liveness probe | 200 if running |
| `/health/ready` | Readiness probe | 200 if ready to serve |
| `/metrics` | Prometheus metrics | Metrics payload |

### Readiness Check Implementation

```go
func (s *Server) ReadinessHandler(w http.ResponseWriter, r *http.Request) {
    checks := []HealthCheck{
        s.checkDatabase(),
        s.checkCache(),
        s.checkDownstreamServices(),
    }

    for _, check := range checks {
        if !check.Healthy {
            w.WriteHeader(http.StatusServiceUnavailable)
            json.NewEncoder(w).Encode(map[string]interface{}{
                "status": "unhealthy",
                "checks": checks,
            })
            return
        }
    }

    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(map[string]string{
        "status": "healthy",
    })
}
```

## Database Schema

### Core Tables

```sql
-- Users table
CREATE TABLE users (
    id          UUID PRIMARY KEY,
    email       VARCHAR(255) UNIQUE NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    updated_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions table
CREATE TABLE subscriptions (
    id          UUID PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    tier        VARCHAR(50) NOT NULL,
    status      VARCHAR(20) DEFAULT 'active',
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log
CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    timestamp   TIMESTAMPTZ DEFAULT NOW(),
    metadata    JSONB
);
```

## Deployment

### Kubernetes Resources

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  labels:
    app: api-gateway
spec:
  replicas: 4
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
        - name: api-gateway
          image: registry.example.com/api-gateway:v1.2.3
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: "500m"
              memory: "512Mi"
            limits:
              cpu: "2000m"
              memory: "2Gi"
          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
```

---

## Further Reading

- [API Gateway Documentation](/docs/api-gateway)
- [Auth Handler Integration Guide](/docs/auth)
- [Billing System Overview](/docs/billing)
- [Metrics and Alerting](/docs/metrics)
- [Deployment Runbook](/docs/deployment)

---

*Architecture v2.1 - Last updated January 2024*
