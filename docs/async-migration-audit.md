# Async Migration Audit: bubbletea Thread Spawning

This document audits all thread usage in the bubbletea crate to plan the migration from `std::thread` to async/await with tokio.

## Summary

| Location | Purpose | Complexity | Blocking |
|----------|---------|------------|----------|
| program.rs:389 | Command execution | Medium | Yes |
| program.rs:396 | Batch command execution | Medium | Yes |
| command.rs:144 | Timer delay (tick) | Easy | No |
| command.rs:178 | Clock-aligned delay (every) | Easy | No |

## Detailed Analysis

### 1. program.rs - Command Execution (Line 389)

```rust
fn handle_command(&self, cmd: Cmd, tx: Sender<Message>) {
    thread::spawn(move || {
        if let Some(msg) = cmd.execute() {
            // Handle batch and sequence messages...
            let _ = tx.send(msg);
        }
    });
}
```

**Purpose**: Execute commands (side effects) in separate threads to avoid blocking the main event loop.

**Data Access**:
- Takes ownership of `Cmd` (the command closure)
- Takes ownership of `Sender<Message>` for sending results back

**Communication**:
- Uses `std::sync::mpsc::Sender` to send resulting messages back to the main loop
- No response expected (fire-and-forget pattern)

**Lifetime**:
- Fire-and-forget: no `JoinHandle` is kept
- Thread runs until command completes
- No graceful shutdown mechanism

**Cancellation Behavior**:
- None - threads run to completion
- No way to cancel in-flight commands when program quits

**Migration Complexity**: **Medium**
- Need to replace with `tokio::spawn`
- Must convert `Cmd` closures to async functions
- May need `spawn_blocking` for CPU-bound or truly blocking operations
- Channel migration: `mpsc` → `tokio::sync::mpsc`

**Migration Strategy**:
1. Add tokio runtime to Program
2. Change `Cmd` to be async-aware (either `async fn` or `spawn_blocking` wrapper)
3. Replace `std::sync::mpsc` with `tokio::sync::mpsc`
4. Use `tokio::spawn` instead of `thread::spawn`

---

### 2. program.rs - Batch Command Execution (Line 396)

```rust
if msg.is::<BatchMsg>() {
    if let Some(batch) = msg.downcast::<BatchMsg>() {
        for cmd in batch.0 {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                if let Some(msg) = cmd.execute() {
                    let _ = tx_clone.send(msg);
                }
            });
        }
    }
}
```

**Purpose**: Execute multiple commands concurrently when a batch is returned.

**Data Access**:
- Takes ownership of individual `Cmd` from the batch
- Clones `Sender<Message>` for each spawned thread

**Communication**:
- Each thread sends its result via cloned sender
- Results arrive in non-deterministic order (as expected for batch)

**Lifetime**:
- Same fire-and-forget pattern as single commands
- All batch commands run independently

**Cancellation Behavior**:
- None

**Migration Complexity**: **Medium**
- Same considerations as single command execution
- Could use `tokio::task::JoinSet` for better batch management
- Consider adding cancellation support via `CancellationToken`

**Migration Strategy**:
1. Use `tokio::spawn` for each batch command
2. Consider collecting `JoinHandle`s in a `JoinSet` for cancellation
3. Add optional timeout support for batches

---

### 3. command.rs - tick() Function (Line 144)

```rust
pub fn tick<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    Cmd::new(move || {
        std::thread::sleep(duration);
        f(Instant::now())
    })
}
```

**Purpose**: Create a timer that fires after a specified duration.

**Data Access**:
- Captures duration and callback in closure
- No shared state

**Communication**:
- Returns message via closure return value
- Eventually sent to channel by `handle_command`

**Lifetime**:
- Blocks the executing thread for the duration
- Single-shot timer

**Cancellation Behavior**:
- Cannot be cancelled once started

**Migration Complexity**: **Easy**
- Direct replacement: `std::thread::sleep` → `tokio::time::sleep`
- Closure becomes async closure

**Migration Strategy**:
```rust
pub async fn tick<F>(duration: Duration, f: F) -> Message
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    tokio::time::sleep(duration).await;
    f(Instant::now())
}
```

---

### 4. command.rs - every() Function (Line 178)

```rust
pub fn every<F>(duration: Duration, f: F) -> Cmd
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    Cmd::new(move || {
        // Calculate time until next aligned tick
        let now = Instant::now();
        let now_nanos = now.elapsed().as_nanos() as u64;
        let duration_nanos = duration.as_nanos() as u64;
        let next_tick_nanos = ((now_nanos / duration_nanos) + 1) * duration_nanos;
        let sleep_nanos = next_tick_nanos - now_nanos;
        std::thread::sleep(Duration::from_nanos(sleep_nanos));
        f(Instant::now())
    })
}
```

**Purpose**: Create a timer aligned with system clock boundaries.

**Data Access**:
- Same as `tick` - captures duration and callback
- Calculates alignment offset locally

**Communication**:
- Same as `tick`

**Lifetime**:
- Blocks until next aligned interval
- Single-shot (must be re-invoked for periodic behavior)

**Cancellation Behavior**:
- Cannot be cancelled

**Migration Complexity**: **Easy**
- Same as `tick` - replace `std::thread::sleep` with `tokio::time::sleep`
- Could also use `tokio::time::interval_at` for more precise alignment

**Migration Strategy**:
```rust
pub async fn every<F>(duration: Duration, f: F) -> Message
where
    F: FnOnce(Instant) -> Message + Send + 'static,
{
    // Calculate aligned time
    let now = Instant::now();
    let now_nanos = now.elapsed().as_nanos() as u64;
    let duration_nanos = duration.as_nanos() as u64;
    let next_tick_nanos = ((now_nanos / duration_nanos) + 1) * duration_nanos;
    let sleep_nanos = next_tick_nanos - now_nanos;
    tokio::time::sleep(Duration::from_nanos(sleep_nanos)).await;
    f(Instant::now())
}
```

---

## Dependencies Map

```
┌─────────────────────────────────────────────────────────────┐
│                      Event Loop                              │
│                    (program.rs:276)                          │
│                          │                                   │
│                          ▼                                   │
│              ┌─────────────────────┐                        │
│              │   handle_command    │                        │
│              │   (program.rs:387)  │                        │
│              └──────────┬──────────┘                        │
│                         │                                    │
│           ┌─────────────┴─────────────┐                     │
│           ▼                           ▼                      │
│   ┌───────────────┐           ┌───────────────┐             │
│   │ thread::spawn │           │  BatchMsg     │             │
│   │   (single)    │           │  handling     │             │
│   └───────┬───────┘           └───────┬───────┘             │
│           │                           │                      │
│           │                    ┌──────┴──────┐              │
│           │                    ▼             ▼              │
│           │           ┌───────────┐   ┌───────────┐         │
│           │           │ spawn #1  │   │ spawn #N  │         │
│           │           └─────┬─────┘   └─────┬─────┘         │
│           │                 │               │                │
│           ▼                 ▼               ▼                │
│   ┌───────────────────────────────────────────────┐         │
│   │              Cmd::execute()                    │         │
│   │  (may call tick/every with thread::sleep)     │         │
│   └───────────────────────────────────────────────┘         │
│                          │                                   │
│                          ▼                                   │
│              ┌─────────────────────┐                        │
│              │ tx.send(Message)    │                        │
│              │    (mpsc channel)   │                        │
│              └─────────────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

## Migration Order Recommendation

1. **Phase 1: Add tokio runtime** (No breaking changes)
   - Add `tokio` as dependency with `rt-multi-thread` feature
   - Create runtime in `Program::run()`
   - Keep existing thread-based code working

2. **Phase 2: Migrate channels**
   - Replace `std::sync::mpsc` with `tokio::sync::mpsc`
   - Update event loop to use async channel recv

3. **Phase 3: Migrate command execution**
   - Change `handle_command` to use `tokio::spawn`
   - Add `spawn_blocking` wrapper for backward compatibility

4. **Phase 4: Migrate Cmd to async**
   - Create `AsyncCmd` type for native async commands
   - Migrate `tick` and `every` to async versions
   - Keep sync `Cmd` as wrapper using `spawn_blocking`

5. **Phase 5: Cleanup**
   - Remove `std::thread` usage
   - Add cancellation support via `CancellationToken`
   - Add graceful shutdown

## Risks and Considerations

1. **Backward Compatibility**
   - Users may have `Cmd` closures with blocking I/O
   - Need `spawn_blocking` escape hatch or transition period

2. **Runtime Selection**
   - Need to decide: require tokio or be runtime-agnostic?
   - Recommendation: Use tokio directly (most common, well-supported)

3. **Performance**
   - Async overhead for simple commands may be higher than threads
   - But scales better with many concurrent commands

4. **Testing**
   - Need `#[tokio::test]` for async tests
   - Consider `tokio::time::pause()` for timer tests

---

*Audit completed: 2026-01-19*
*Auditor: Claude Code Agent*
