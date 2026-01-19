# Async Command Execution Architecture

This document defines the architecture for migrating bubbletea from `std::thread` to an async runtime using tokio.

## Design Goals

1. **Maintain Elm Architecture semantics** - Commands remain the only source of side effects
2. **Backward compatibility** - Existing `Cmd` closures continue to work
3. **Modern async patterns** - Use async/await where beneficial
4. **Graceful shutdown** - Support clean cancellation of in-flight commands
5. **Minimal API changes** - Preserve the simple bubbletea API

## 1. Runtime Design

### Decision: Multi-threaded Runtime

Use `tokio::runtime::Runtime` with multi-thread scheduler:

```rust
// In Program::run()
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(4)  // Configurable
    .build()?;
```

**Rationale**:
- Commands may perform blocking I/O (file, network)
- Multiple concurrent commands benefit from parallelism
- `spawn_blocking` requires multi-thread runtime

### Runtime Lifecycle

```rust
impl<M: Model> Program<M> {
    pub fn run(self) -> Result<M, Error> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        rt.block_on(self.async_run())
    }

    async fn async_run(self) -> Result<M, Error> {
        // Async event loop implementation
    }
}
```

### Configuration

Add optional runtime configuration to `ProgramOptions`:

```rust
pub struct ProgramOptions {
    // ... existing fields ...

    /// Number of tokio worker threads (default: available parallelism)
    pub worker_threads: Option<usize>,

    /// Enable tokio's time driver (default: true)
    pub enable_time: bool,
}
```

## 2. Command Trait Extension

### Existing Cmd (Preserved)

The current `Cmd` type is preserved for backward compatibility:

```rust
pub struct Cmd(Box<dyn FnOnce() -> Option<Message> + Send + 'static>);
```

### New AsyncCmd Type

Add a new async command type:

```rust
pub struct AsyncCmd(
    Box<dyn FnOnce() -> BoxFuture<'static, Option<Message>> + Send + 'static>
);

impl AsyncCmd {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Message> + Send + 'static,
    {
        Self(Box::new(move || Box::pin(async move { Some(f().await) })))
    }

    pub async fn execute(self) -> Option<Message> {
        (self.0)().await
    }
}
```

### Command Enum (Internal)

Internally, use an enum to handle both types:

```rust
pub(crate) enum CommandKind {
    Sync(Cmd),
    Async(AsyncCmd),
}

impl CommandKind {
    pub async fn execute(self) -> Option<Message> {
        match self {
            CommandKind::Sync(cmd) => {
                // Run blocking code on tokio's blocking thread pool
                tokio::task::spawn_blocking(move || cmd.execute())
                    .await
                    .ok()
                    .flatten()
            }
            CommandKind::Async(cmd) => cmd.execute().await,
        }
    }
}
```

### Updated Command Functions

```rust
// Async tick using tokio::time
pub fn tick<F>(duration: Duration, f: F) -> AsyncCmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    AsyncCmd::new(move || async move {
        tokio::time::sleep(duration).await;
        f(Instant::now())
    })
}

// Async every using tokio::time
pub fn every<F>(duration: Duration, f: F) -> AsyncCmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    AsyncCmd::new(move || async move {
        let now = Instant::now();
        let now_nanos = now.elapsed().as_nanos() as u64;
        let duration_nanos = duration.as_nanos() as u64;
        let next_tick_nanos = ((now_nanos / duration_nanos) + 1) * duration_nanos;
        let sleep_nanos = next_tick_nanos - now_nanos;
        tokio::time::sleep(Duration::from_nanos(sleep_nanos)).await;
        f(Instant::now())
    })
}
```

### User API

Users can return either type from `update()`:

```rust
// Sync (existing)
fn update(&mut self, msg: Message) -> Option<Cmd> {
    Some(Cmd::new(|| Message::new("done")))
}

// Async (new)
fn update(&mut self, msg: Message) -> Option<AsyncCmd> {
    Some(AsyncCmd::new(|| async {
        let data = fetch_data().await;
        Message::new(data)
    }))
}
```

## 3. Channel Architecture

### Message Channel

Replace `std::sync::mpsc` with `tokio::sync::mpsc`:

```rust
use tokio::sync::mpsc::{self, Sender, Receiver};

const CHANNEL_BUFFER: usize = 256;  // Configurable backpressure

async fn async_run(mut self) -> Result<M, Error> {
    let (tx, mut rx) = mpsc::channel::<Message>(CHANNEL_BUFFER);
    // ...
}
```

### Channel Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Event Loop (async)                        │
│                                                              │
│  ┌─────────────┐     ┌─────────────────┐                   │
│  │   Events    │────▶│   Event Handler  │                   │
│  │  (crossterm)│     │   (select! loop) │                   │
│  └─────────────┘     └────────┬────────┘                   │
│                               │                              │
│                               ▼                              │
│                      ┌───────────────┐                      │
│                      │  tx.send(msg) │                      │
│                      └───────┬───────┘                      │
│                              │                               │
│                              ▼                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              mpsc::Receiver<Message>                  │  │
│  │                (bounded, 256 msgs)                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│                     ┌────────────────┐                      │
│                     │ model.update() │                      │
│                     └────────┬───────┘                      │
│                              │                               │
│                              ▼                               │
│                    ┌─────────────────┐                      │
│                    │ tokio::spawn()  │                      │
│                    │   (commands)    │                      │
│                    └─────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

### Backpressure Handling

```rust
// Non-blocking send with fallback
match tx.try_send(msg) {
    Ok(()) => {},
    Err(TrySendError::Full(msg)) => {
        // Buffer full, log warning and wait
        tracing::warn!("message channel full, applying backpressure");
        tx.send(msg).await.ok();
    }
    Err(TrySendError::Closed(_)) => {
        // Channel closed, program shutting down
        break;
    }
}
```

### Error Propagation

Commands can now return errors:

```rust
pub struct CmdError {
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub struct ErrorMsg(pub CmdError);

// In command execution
async fn execute_command(cmd: CommandKind, tx: Sender<Message>) {
    match cmd.execute().await {
        Some(msg) => { tx.send(msg).await.ok(); }
        None => {} // No message produced
    }
}
```

## 4. Cancellation Strategy

### CancellationToken

Use `tokio_util::sync::CancellationToken` for cooperative cancellation:

```rust
use tokio_util::sync::CancellationToken;

struct Runtime {
    cancel_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl Runtime {
    fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            task_tracker: TaskTracker::new(),
        }
    }
}
```

### Task Tracking

Track spawned tasks using `tokio_util::task::TaskTracker`:

```rust
use tokio_util::task::TaskTracker;

async fn handle_command(
    cmd: CommandKind,
    tx: Sender<Message>,
    tracker: &TaskTracker,
    cancel: CancellationToken,
) {
    tracker.spawn(async move {
        tokio::select! {
            result = cmd.execute() => {
                if let Some(msg) = result {
                    tx.send(msg).await.ok();
                }
            }
            _ = cancel.cancelled() => {
                // Command cancelled, cleanup
            }
        }
    });
}
```

### Shutdown Sequence

```rust
async fn shutdown(runtime: Runtime, timeout: Duration) {
    // 1. Signal cancellation
    runtime.cancel_token.cancel();

    // 2. Wait for tasks with timeout
    tokio::select! {
        _ = runtime.task_tracker.wait() => {
            tracing::info!("all tasks completed gracefully");
        }
        _ = tokio::time::sleep(timeout) => {
            tracing::warn!("shutdown timeout, {} tasks still running",
                runtime.task_tracker.len());
        }
    }

    // 3. Close the task tracker
    runtime.task_tracker.close();
}
```

### Graceful vs Forced Shutdown

```rust
pub enum ShutdownMode {
    /// Wait for all commands to complete
    Graceful,
    /// Wait with timeout, then force
    GracefulWithTimeout(Duration),
    /// Cancel immediately
    Immediate,
}

impl Default for ShutdownMode {
    fn default() -> Self {
        Self::GracefulWithTimeout(Duration::from_secs(5))
    }
}
```

## 5. Event Loop Architecture

### Updated Event Loop

```rust
async fn async_run(mut self) -> Result<M, Error> {
    let (tx, mut rx) = mpsc::channel::<Message>(256);
    let cancel_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    // Handle init command
    if let Some(cmd) = self.model.init() {
        handle_command(cmd.into(), tx.clone(), &task_tracker, cancel_token.clone());
    }

    // Render initial view
    let mut last_view = String::new();
    self.render(&mut last_view)?;

    let frame_duration = Duration::from_secs_f64(1.0 / self.options.fps as f64);
    let mut frame_interval = tokio::time::interval(frame_duration);

    loop {
        tokio::select! {
            // Check for terminal events
            event = poll_event() => {
                if let Some(event) = event? {
                    let msg = convert_event(event);
                    tx.send(msg).await.ok();
                }
            }

            // Process incoming messages
            Some(msg) = rx.recv() => {
                if msg.is::<QuitMsg>() {
                    break;
                }

                if let Some(cmd) = self.model.update(msg) {
                    handle_command(
                        cmd.into(),
                        tx.clone(),
                        &task_tracker,
                        cancel_token.clone()
                    );
                }
            }

            // Frame rendering
            _ = frame_interval.tick() => {
                self.render(&mut last_view)?;
            }
        }
    }

    // Graceful shutdown
    shutdown(Runtime { cancel_token, task_tracker }, Duration::from_secs(5)).await;

    Ok(self.model)
}
```

### Event Polling (Async)

Wrap crossterm event polling in async:

```rust
async fn poll_event() -> Result<Option<Event>, Error> {
    // crossterm doesn't have native async support
    // Use spawn_blocking for the poll
    let result = tokio::task::spawn_blocking(|| {
        if event::poll(Duration::from_millis(10))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }).await.map_err(|_| Error::Runtime)?;

    result
}
```

## 6. API Summary

### New Public Types

```rust
// Async command type
pub struct AsyncCmd { ... }

// Cancellation token (re-export)
pub use tokio_util::sync::CancellationToken;

// Shutdown mode
pub enum ShutdownMode { ... }

// Error message for command failures
pub struct ErrorMsg(pub CmdError);
```

### New Program Options

```rust
pub struct ProgramOptions {
    // ... existing ...
    pub worker_threads: Option<usize>,
    pub shutdown_mode: ShutdownMode,
    pub channel_buffer: usize,
}
```

### Feature Flags

```toml
[features]
default = ["async"]
async = ["tokio", "tokio-util"]
```

## 7. Migration Path

### Phase 1: Add Runtime (Non-breaking)
- Add tokio dependency
- Create runtime in `Program::run()`
- Keep existing thread-based command execution

### Phase 2: Add AsyncCmd (Additive)
- Add `AsyncCmd` type
- Migrate `tick` and `every` to async versions
- Keep `Cmd` working via `spawn_blocking`

### Phase 3: Migrate Event Loop
- Replace `std::sync::mpsc` with `tokio::sync::mpsc`
- Use `tokio::select!` for event loop
- Add cancellation support

### Phase 4: Cleanup
- Remove direct `std::thread` usage
- Document migration guide for users
- Add async examples

## 8. Dependencies

New dependencies required:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
tokio-util = { version = "0.7", features = ["rt"] }

[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
```

---

*Architecture designed: 2026-01-19*
*Architect: Claude Code Agent*
*Status: Ready for implementation (charmed_rust-u2y)*
