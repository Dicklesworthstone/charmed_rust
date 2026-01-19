# Async Migration Guide for bubbletea

This guide explains how to migrate your bubbletea application to use the async runtime for better performance and resource efficiency.

## Overview

bubbletea now supports an optional `async` feature that uses tokio for:
- Non-blocking command execution
- Efficient I/O handling with `spawn_blocking`
- Graceful shutdown with task cancellation
- Better scalability with many concurrent commands

## Quick Start

### 1. Enable the async feature

In your `Cargo.toml`:

```toml
[dependencies]
bubbletea = { version = "0.1", features = ["async"] }
```

### 2. Use run_async() instead of run()

```rust
use bubbletea::Program;

// Before (sync)
fn main() -> Result<(), bubbletea::Error> {
    let model = MyModel::new();
    let final_model = Program::new(model).run()?;
    Ok(())
}

// After (async)
#[tokio::main]
async fn main() -> Result<(), bubbletea::Error> {
    let model = MyModel::new();
    let final_model = Program::new(model)
        .with_alt_screen()
        .run_async()
        .await?;
    Ok(())
}
```

### 3. Use AsyncCmd for async commands

```rust
use bubbletea::{AsyncCmd, Message};
use std::time::Duration;

// Async command that fetches data
fn fetch_data() -> AsyncCmd {
    AsyncCmd::new(|| async {
        // Async I/O operations
        let data = reqwest::get("https://api.example.com/data")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        Message::new(DataLoaded(data))
    })
}

// Async timer
fn delayed_action() -> AsyncCmd {
    bubbletea::tick_async(Duration::from_secs(1), |_| {
        Message::new(TimerFired)
    })
}
```

## API Changes

### New Types (async feature only)

| Type | Description |
|------|-------------|
| `AsyncCmd` | Async command type for non-blocking operations |
| `tick_async` | Async version of `tick` using `tokio::time` |
| `every_async` | Async version of `every` using `tokio::time` |

### Program Methods

| Method | Description |
|--------|-------------|
| `run()` | Sync version (uses threads) - unchanged |
| `run_async()` | Async version (uses tokio) - new |

### Backward Compatibility

The sync API remains fully functional:
- `Cmd` continues to work unchanged
- `tick` and `every` still use `std::thread::sleep`
- `run()` still works without tokio

When using the async runtime, sync commands are automatically wrapped with `spawn_blocking` to avoid blocking the async executor.

## Migration Steps

### Step 1: Add tokio dependency

If not already present:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Step 2: Update your main function

Change from:
```rust
fn main() -> Result<(), bubbletea::Error> {
    let model = MyModel::new();
    Program::new(model).run()?;
    Ok(())
}
```

To:
```rust
#[tokio::main]
async fn main() -> Result<(), bubbletea::Error> {
    let model = MyModel::new();
    Program::new(model).run_async().await?;
    Ok(())
}
```

### Step 3: Convert commands (optional)

For better performance, convert blocking commands to async:

**Before:**
```rust
fn load_file() -> Cmd {
    Cmd::new(|| {
        let content = std::fs::read_to_string("file.txt").unwrap();
        Message::new(FileLoaded(content))
    })
}
```

**After:**
```rust
fn load_file() -> AsyncCmd {
    AsyncCmd::new(|| async {
        let content = tokio::fs::read_to_string("file.txt").await.unwrap();
        Message::new(FileLoaded(content))
    })
}
```

### Step 4: Use async timers

**Before:**
```rust
fn start_timer() -> Cmd {
    tick(Duration::from_secs(1), |t| Message::new(Tick(t)))
}
```

**After:**
```rust
fn start_timer() -> AsyncCmd {
    tick_async(Duration::from_secs(1), |t| Message::new(Tick(t)))
}
```

## Graceful Shutdown

The async runtime includes graceful shutdown:

1. When `quit()` is called or Ctrl+C is pressed
2. A `CancellationToken` signals all tasks to stop
3. Tasks have 5 seconds to complete gracefully
4. After timeout, remaining tasks are dropped

This ensures:
- In-flight HTTP requests can complete or be cancelled cleanly
- File handles are properly closed
- No orphaned tasks after program exit

## Performance Considerations

### When to use async

- Many concurrent I/O operations (HTTP requests, file reads)
- Long-running background tasks
- Commands that can be cancelled

### When to keep sync

- CPU-bound operations (use `spawn_blocking`)
- Simple, fast commands
- Operations that must complete atomically

### Mixing sync and async

You can mix `Cmd` and `AsyncCmd` in the same application:

```rust
fn update(&mut self, msg: Message) -> Option<Cmd> {
    // Sync commands still work
    if msg.is::<StartDownload>() {
        // But consider returning AsyncCmd for I/O
        return Some(Cmd::new(|| {
            // This will run in spawn_blocking
            let data = blocking_download();
            Message::new(Downloaded(data))
        }));
    }
    None
}
```

## Testing

Use `#[tokio::test]` for async tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_command() {
        let cmd = AsyncCmd::new(|| async { Message::new(42i32) });
        let msg = cmd.execute().await.unwrap();
        assert_eq!(msg.downcast::<i32>().unwrap(), 42);
    }
}
```

## Troubleshooting

### "tokio runtime not found"

Make sure you're using `#[tokio::main]` on your main function or creating a runtime manually:

```rust
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    Program::new(model).run_async().await
})
```

### Commands not executing

Check that:
1. You're using `run_async()`, not `run()`
2. The async feature is enabled in Cargo.toml
3. Async commands return `AsyncCmd`, not `Cmd`

### Blocking the runtime

If your UI becomes unresponsive, you may have blocking code in an async command. Use `spawn_blocking`:

```rust
AsyncCmd::new(|| async {
    let result = tokio::task::spawn_blocking(|| {
        // Blocking code here
        heavy_computation()
    }).await.unwrap();
    Message::new(result)
})
```

---

*Last updated: 2026-01-19*
