# Harmonica

Physics-based animation primitives for terminal UIs and time-based motion.

## Role in the charmed_rust (FrankenTUI) stack

Harmonica sits at the bottom of the stack as the motion engine. It provides
deterministic physics helpers that higher-level crates can use to animate UI
state over time without depending on any UI framework directly. In practice,
`bubbletea` uses harmonica helpers for animation timing, `bubbles` components
use it to animate progress and transitions, and `charmed-demo-showcase` uses it
to demonstrate smooth, spring-like motion in the overall system.

## Crates.io package

Package name: `charmed-harmonica`  
Library crate name: `harmonica`

## What it provides

- `Spring` for damped harmonic motion.
- `Projectile` for simple ballistic motion.
- `fps(...)` helper for fixed-timestep updates.

## Typical usage

```rust
use harmonica::{fps, Spring};

let spring = Spring::new(fps(60), 6.0, 0.2);
let (mut pos, mut vel) = (0.0, 0.0);

(pos, vel) = spring.update(pos, vel, 100.0);
```

## Where to look next

- `crates/harmonica/src/spring.rs`
- `crates/harmonica/src/projectile.rs`
