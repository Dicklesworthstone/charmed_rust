# Progress Bar Example

Demonstrates the bubbles Progress component with async tick commands for simulating long-running operations.

## Running

```bash
cargo run -p example-progress
```

## Key Concepts

### Tick Commands

Use `tick()` to schedule periodic updates for animations:

```rust
use bubbletea::tick;
use std::time::Duration;

// Schedule a tick every 50ms
return Some(tick(Duration::from_millis(50), TickMsg::msg));
```

### Custom Tick Message

Create a custom message type for tick events:

```rust
struct TickMsg(Instant);

impl TickMsg {
    fn msg(instant: Instant) -> Message {
        Message::new(Self(instant))
    }
}
```

### Progress Component

The Progress bar renders with a percentage:

```rust
let progress = Progress::new().width(40);

// In view(), use view_as to render at specific percentage
progress.view_as(self.percent / 100.0)
```

### State Machine

Track operation state for different UI modes:

```rust
enum State {
    Ready,      // Waiting to start
    Running,    // Operation in progress
    Done,       // Completed successfully
    Cancelled,  // User cancelled
}
```

### Handling Ticks

Process tick messages to update progress:

```rust
if msg.downcast_ref::<TickMsg>().is_some() {
    if self.state == State::Running {
        self.percent += 2.0;

        if self.percent >= 100.0 {
            self.state = State::Done;
            return None;
        }

        // Continue ticking
        return Some(tick(Duration::from_millis(50), TickMsg::msg));
    }
}
```

## Controls

| Key | Action |
|-----|--------|
| `Enter` / `Space` | Start progress |
| `Esc` | Cancel (while running) / Quit |
| `r` | Reset after completion |
| `q` | Quit |

## Progress Methods

| Method | Description |
|--------|-------------|
| `new()` | Create with defaults |
| `width(n)` | Set bar width in chars |
| `view_as(pct)` | Render at specific 0.0-1.0 percentage |

## Related Examples

- [spinner](../../basic/spinner) - Simple animation
- [viewport](../viewport) - Another bubbles component
