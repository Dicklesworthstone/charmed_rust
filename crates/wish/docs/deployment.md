# Deployment Guide

This guide covers production deployment of Wish SSH servers.

## Production Configuration

### Server Setup

```rust
use std::time::Duration;
use wish::{ServerBuilder, with_host_key_path};
use wish::auth::{AuthorizedKeysAuth, RateLimitedAuth};
use wish::middleware::{logging, ratelimiter};

#[tokio::main]
async fn main() -> Result<(), wish::Error> {
    // Production-ready server configuration
    let auth = RateLimitedAuth::new(
        AuthorizedKeysAuth::new("/etc/wish/authorized_keys")?
    )
    .with_rejection_delay(100)
    .with_max_attempts(6);

    let limiter = ratelimiter::new_rate_limiter(10.0, 50, 10000);

    let server = ServerBuilder::new()
        .address("0.0.0.0:22")
        .version("SSH-2.0-MyService")
        .host_key_path("/etc/wish/host_key")       // Persistent key
        .idle_timeout(Duration::from_secs(600))    // 10 min idle timeout
        .max_auth_attempts(6)
        .auth_rejection_delay(100)
        .auth_handler(auth)
        .with_middleware(ratelimiter::middleware(limiter))
        .with_middleware(logging::structured_middleware())
        .handler(my_handler)
        .build()?;

    server.listen().await
}
```

### Host Keys

**Always use persistent host keys in production!**

Generate a host key:
```bash
ssh-keygen -t ed25519 -f /etc/wish/host_key -N ""
```

Configure permissions:
```bash
chmod 600 /etc/wish/host_key
chmod 644 /etc/wish/host_key.pub
```

Use in server:
```rust
ServerBuilder::new()
    .host_key_path("/etc/wish/host_key")
```

### Environment Variables

Configure via environment:
```rust
use std::env;

let address = env::var("WISH_BIND_ADDR").unwrap_or("0.0.0.0:22".into());
let key_path = env::var("WISH_HOST_KEY").unwrap_or("/etc/wish/host_key".into());
let timeout = env::var("WISH_IDLE_TIMEOUT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(600);

ServerBuilder::new()
    .address(&address)
    .host_key_path(&key_path)
    .idle_timeout(Duration::from_secs(timeout))
```

## Systemd Service

### Service File

Create `/etc/systemd/system/wish.service`:

```ini
[Unit]
Description=Wish SSH Server
After=network.target

[Service]
Type=simple
User=wish
Group=wish
ExecStart=/usr/local/bin/wish-server
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictSUIDSGID=true
RestrictNamespaces=true

# Allow binding to privileged port 22
AmbientCapabilities=CAP_NET_BIND_SERVICE
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Environment
Environment=RUST_LOG=info
Environment=WISH_BIND_ADDR=0.0.0.0:22
Environment=WISH_HOST_KEY=/etc/wish/host_key

# Working directory
WorkingDirectory=/var/lib/wish

# File descriptors
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

### Setup Commands

```bash
# Create user
sudo useradd -r -s /bin/false wish

# Create directories
sudo mkdir -p /etc/wish /var/lib/wish
sudo chown wish:wish /etc/wish /var/lib/wish

# Generate host key
sudo -u wish ssh-keygen -t ed25519 -f /etc/wish/host_key -N ""

# Install binary
sudo cp target/release/wish-server /usr/local/bin/
sudo chmod 755 /usr/local/bin/wish-server

# Enable and start service
sudo systemctl enable wish
sudo systemctl start wish
```

### Management

```bash
# Check status
sudo systemctl status wish

# View logs
sudo journalctl -u wish -f

# Restart
sudo systemctl restart wish

# Reload configuration
sudo systemctl reload wish
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.79 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/wish-server /usr/local/bin/

# Create non-root user
RUN useradd -r -s /bin/false wish
USER wish

EXPOSE 2222

ENV RUST_LOG=info
ENV WISH_BIND_ADDR=0.0.0.0:2222

CMD ["wish-server"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  wish:
    build: .
    ports:
      - "2222:2222"
    volumes:
      - ./host_key:/etc/wish/host_key:ro
      - ./authorized_keys:/etc/wish/authorized_keys:ro
    environment:
      - RUST_LOG=info
      - WISH_HOST_KEY=/etc/wish/host_key
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 256M
```

### Running

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f wish

# Scale (if stateless)
docker-compose up -d --scale wish=3
```

## Security Hardening

### Network Security

1. **Firewall Rules**
   ```bash
   # Allow SSH from specific IPs only
   sudo ufw allow from 10.0.0.0/8 to any port 22
   ```

2. **Port Knocking** (optional)
   ```bash
   # Use knockd or similar
   ```

3. **Fail2ban Integration**
   ```ini
   # /etc/fail2ban/jail.d/wish.conf
   [wish]
   enabled = true
   filter = wish
   logpath = /var/log/wish/auth.log
   maxretry = 3
   bantime = 3600
   ```

### Application Security

1. **Use Strong Authentication**
   ```rust
   // Public key only, no passwords
   let auth = AuthorizedKeysAuth::new("/etc/wish/authorized_keys")?;
   ```

2. **Enable Rate Limiting**
   ```rust
   let limiter = ratelimiter::new_rate_limiter(1.0, 5, 10000);
   ```

3. **Set Timeouts**
   ```rust
   .idle_timeout(Duration::from_secs(300))
   .max_timeout(Duration::from_secs(3600))
   ```

4. **Limit Authentication Attempts**
   ```rust
   .max_auth_attempts(3)
   .auth_rejection_delay(500)
   ```

### Logging and Monitoring

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Structured logging with JSON output
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .init();
```

Monitor these metrics:
- Connection attempts (success/failure)
- Authentication failures per IP
- Active connections
- Session duration
- Error rates

## Scaling

### Horizontal Scaling

For high availability, run multiple instances behind a load balancer:

```text
                    ┌─────────────────┐
                    │  Load Balancer  │
                    │    (HAProxy)    │
                    └────────┬────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
    ┌──────▼──────┐   ┌──────▼──────┐   ┌──────▼──────┐
    │   Wish 1    │   │   Wish 2    │   │   Wish 3    │
    │  :2222      │   │  :2222      │   │  :2222      │
    └─────────────┘   └─────────────┘   └─────────────┘
```

HAProxy configuration:
```
frontend ssh
    bind *:22
    default_backend wish_servers

backend wish_servers
    balance roundrobin
    server wish1 10.0.0.1:2222 check
    server wish2 10.0.0.2:2222 check
    server wish3 10.0.0.3:2222 check
```

### Connection Limits

Configure per-server limits:
```rust
// In your handler or middleware
static CONNECTIONS: AtomicU64 = AtomicU64::new(0);
const MAX_CONNECTIONS: u64 = 1000;

fn connection_limit_middleware() -> Middleware {
    Arc::new(|next| {
        Arc::new(move |session| {
            let next = next.clone();
            Box::pin(async move {
                let count = CONNECTIONS.fetch_add(1, Ordering::SeqCst);
                if count >= MAX_CONNECTIONS {
                    CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
                    wish::fatalln(&session, "Server at capacity");
                    return;
                }

                next(session).await;
                CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
            })
        })
    })
}
```

## Troubleshooting

### Common Issues

**1. "Address already in use"**
```bash
# Find process using port
sudo lsof -i :22
# or
sudo ss -tlnp | grep :22
```

**2. "Permission denied" on port 22**
```bash
# Either use a non-privileged port
.address("0.0.0.0:2222")

# Or grant capability
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/wish-server
```

**3. Host key warnings**
Ensure you're using a persistent host key:
```rust
.host_key_path("/etc/wish/host_key")  // NOT ephemeral
```

**4. High memory usage**
- Check for connection leaks
- Monitor with `htop` or `pmap`
- Set memory limits in systemd/Docker

### Debug Logging

Enable verbose logging:
```bash
RUST_LOG=wish=debug,russh=debug ./wish-server
```

Or in code:
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```
