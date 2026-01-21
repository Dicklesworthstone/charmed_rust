# Middleware Guide

This guide explains how to use and create middleware in Wish.

## Overview

Middleware in Wish follows the "onion" pattern, wrapping handlers to add
cross-cutting functionality like logging, authentication checks, and rate limiting.

## Middleware Concept

```text
Request  ──────────────────────────────────────────────────▶

         ┌─────────────────────────────────────────────────┐
         │  Middleware 1 (outermost)                       │
         │  ┌─────────────────────────────────────────────┐│
         │  │  Middleware 2                               ││
         │  │  ┌─────────────────────────────────────────┐││
         │  │  │  Middleware 3                           │││
         │  │  │  ┌─────────────────────────────────────┐│││
         │  │  │  │                                     ││││
         │  │  │  │          Handler                    ││││
         │  │  │  │                                     ││││
         │  │  │  └─────────────────────────────────────┘│││
         │  │  └─────────────────────────────────────────┘││
         │  └─────────────────────────────────────────────┘│
         └─────────────────────────────────────────────────┘

◀────────────────────────────────────────────────── Response
```

Each middleware can:
- Execute code before the next handler
- Execute code after the next handler
- Short-circuit the chain (skip the handler)
- Modify the session or response

## Built-in Middleware

### Logging Middleware

Logs connection and disconnection events:

```rust
use wish::middleware::logging;

// Basic logging
ServerBuilder::new()
    .with_middleware(logging::middleware())

// Structured logging with tracing events
ServerBuilder::new()
    .with_middleware(logging::structured_middleware())

// Custom logger implementation
struct MyLogger;
impl logging::Logger for MyLogger {
    fn log(&self, format: &str, args: &[&dyn std::fmt::Display]) {
        // Custom logging logic
    }
}

ServerBuilder::new()
    .with_middleware(logging::middleware_with_logger(MyLogger))
```

### Active Terminal Middleware

Requires clients to have an allocated PTY:

```rust
use wish::middleware::activeterm;

ServerBuilder::new()
    .with_middleware(activeterm::middleware())
    .handler(|session| async move {
        // This code only runs if PTY is allocated
    })
```

### Access Control Middleware

Restricts which commands can be executed:

```rust
use wish::middleware::accesscontrol;

// Only allow git commands
let allowed = vec![
    "git-receive-pack".to_string(),
    "git-upload-pack".to_string(),
];

ServerBuilder::new()
    .with_middleware(accesscontrol::middleware(allowed))
```

### Rate Limiting Middleware

Prevents abuse with token-bucket rate limiting:

```rust
use wish::middleware::ratelimiter;

// 1 request/second, burst of 10, max 1000 tracked IPs
let limiter = ratelimiter::new_rate_limiter(1.0, 10, 1000);

ServerBuilder::new()
    .with_middleware(ratelimiter::middleware(limiter))

// Or with config
let config = ratelimiter::Config {
    rate_per_sec: 2.0,
    burst: 5,
    max_entries: 1000,
};
ServerBuilder::new()
    .with_middleware(ratelimiter::middleware_with_config(config))
```

### Elapsed Time Middleware

Displays session duration:

```rust
use wish::middleware::elapsed;

// Default format
ServerBuilder::new()
    .with_middleware(elapsed::middleware())
// Output: "elapsed time: 1.234s"

// Custom format
ServerBuilder::new()
    .with_middleware(elapsed::middleware_with_format("Session lasted: %v"))
```

### Comment Middleware

Adds a message to the output:

```rust
use wish::middleware::comment;

ServerBuilder::new()
    .with_middleware(comment::middleware("Thank you for using our service!"))
```

### Recovery Middleware

Catches panics (basic implementation):

```rust
use wish::middleware::recover;

ServerBuilder::new()
    .with_middleware(recover::middleware())
```

## Creating Custom Middleware

### Basic Structure

```rust
use std::sync::Arc;
use wish::{BoxFuture, Handler, Middleware, Session};

fn my_middleware() -> Middleware {
    Arc::new(|next: Handler| {
        Arc::new(move |session: Session| {
            let next = next.clone();
            Box::pin(async move {
                // Pre-processing (before handler)
                println!("Before handler");

                // Call the next handler
                next(session).await;

                // Post-processing (after handler)
                println!("After handler");
            }) as BoxFuture<'static, ()>
        })
    })
}
```

### Pre-Processing Middleware

Execute logic before the handler:

```rust
fn auth_check_middleware() -> Middleware {
    Arc::new(|next| {
        Arc::new(move |session| {
            let next = next.clone();
            Box::pin(async move {
                // Check some condition
                if session.user() == "blocked" {
                    wish::fatalln(&session, "Access denied");
                    return; // Short-circuit - don't call handler
                }

                // Continue to handler
                next(session).await;
            })
        })
    })
}
```

### Post-Processing Middleware

Execute logic after the handler:

```rust
fn cleanup_middleware() -> Middleware {
    Arc::new(|next| {
        Arc::new(move |session| {
            let next = next.clone();
            Box::pin(async move {
                // Run the handler first
                next(session.clone()).await;

                // Cleanup after handler completes
                wish::println(&session, "Goodbye!");
            })
        })
    })
}
```

### Stateful Middleware

Maintain state across connections:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

fn counter_middleware() -> Middleware {
    let count = Arc::new(AtomicU64::new(0));

    Arc::new(move |next| {
        let count = count.clone();
        Arc::new(move |session| {
            let next = next.clone();
            let count = count.clone();
            Box::pin(async move {
                let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                wish::println(&session, format!("Connection #{}", n));
                next(session).await;
            })
        })
    })
}
```

### Parameterized Middleware

Accept configuration parameters:

```rust
fn timeout_middleware(duration: Duration) -> Middleware {
    Arc::new(move |next| {
        let duration = duration;
        Arc::new(move |session| {
            let next = next.clone();
            Box::pin(async move {
                // Set a timeout for the handler
                match tokio::time::timeout(duration, next(session.clone())).await {
                    Ok(()) => {}
                    Err(_) => {
                        wish::errorln(&session, "Session timed out");
                        let _ = session.exit(1);
                    }
                }
            })
        })
    })
}
```

### Composing Middleware

Combine multiple middleware:

```rust
use wish::compose_middleware;

let composed = compose_middleware(vec![
    logging::middleware(),
    activeterm::middleware(),
    my_custom_middleware(),
]);

ServerBuilder::new()
    .with_middleware(composed)
```

## Middleware Execution Order

Middleware is executed in the order added:

```rust
ServerBuilder::new()
    .with_middleware(mw1())  // Runs first (outermost)
    .with_middleware(mw2())  // Runs second
    .with_middleware(mw3())  // Runs third (innermost)
    .handler(handler)        // Runs last
```

Execution flow:
1. mw1 pre-processing
2. mw2 pre-processing
3. mw3 pre-processing
4. handler
5. mw3 post-processing
6. mw2 post-processing
7. mw1 post-processing

## Best Practices

### 1. Keep Middleware Focused

Each middleware should do one thing well:

```rust
// Good: single responsibility
fn logging_middleware() -> Middleware { /* only logging */ }
fn auth_middleware() -> Middleware { /* only auth */ }
fn rate_limit_middleware() -> Middleware { /* only rate limiting */ }

// Bad: doing too much
fn kitchen_sink_middleware() -> Middleware {
    // logs AND authenticates AND rate limits AND...
}
```

### 2. Order Matters

Put security middleware early:

```rust
ServerBuilder::new()
    .with_middleware(ratelimiter::middleware(limiter))  // First: rate limit
    .with_middleware(logging::middleware())              // Second: log
    .with_middleware(auth_check())                       // Third: auth
    .with_middleware(activeterm::middleware())           // Fourth: PTY check
    .handler(handler)
```

### 3. Handle Errors Gracefully

```rust
fn db_middleware(pool: Pool) -> Middleware {
    Arc::new(move |next| {
        let pool = pool.clone();
        Arc::new(move |session| {
            let next = next.clone();
            let pool = pool.clone();
            Box::pin(async move {
                match pool.get_connection().await {
                    Ok(conn) => {
                        // Store connection for handler
                        session.context().set_value("db", conn.id());
                        next(session).await;
                    }
                    Err(e) => {
                        tracing::error!("Database error: {}", e);
                        wish::fatalln(&session, "Service unavailable");
                    }
                }
            })
        })
    })
}
```

### 4. Avoid Blocking

Use async operations within middleware:

```rust
// Good: async database call
async fn check_user_async(user: &str) -> bool {
    db.query(user).await.is_ok()
}

// Bad: blocking call in async context
fn check_user_blocking(user: &str) -> bool {
    std::thread::sleep(Duration::from_secs(1)); // Don't do this!
    true
}
```

### 5. Use Type-Safe State

Instead of string keys, use typed wrappers:

```rust
struct RequestId(String);

fn request_id_middleware() -> Middleware {
    Arc::new(|next| {
        Arc::new(move |session| {
            let next = next.clone();
            let id = uuid::Uuid::new_v4().to_string();
            Box::pin(async move {
                session.context().set_value("request_id", &id);
                next(session).await;
            })
        })
    })
}
```
