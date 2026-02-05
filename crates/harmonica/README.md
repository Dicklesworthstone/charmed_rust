# Harmonica

Physics-based animation primitives for terminal UIs and time-based motion.

Harmonica gives you deterministic, frame-stepped motion (springs and projectiles)
that you can drive from a TUI event loop without pulling in a full physics engine.

## TL;DR

**The Problem:** Most terminal animation code is ad-hoc and hard to tune. You end
up with hand-rolled easing that feels inconsistent or jittery across machines.

**The Solution:** Harmonica provides a tiny, deterministic physics core with
stable spring and projectile motion. You control the timestep; the output is
repeatable and testable.

**Why Harmonica**

- **Deterministic**: same inputs → same outputs across platforms.
- **Tunable**: explicit parameters for frequency, damping, and response.
- **Lightweight**: no external runtime, no allocation-heavy engine.
- **no_std ready**: run in constrained environments when needed.

## Role in the charmed_rust (FrankenTUI) stack

Harmonica is the motion layer at the bottom of the stack. `bubbletea` uses it for
animation helpers, `bubbles` uses it to animate components, and the demo
showcase uses it to demonstrate smooth UI transitions.

## Crates.io package

Package name: `charmed-harmonica`  
Library crate name: `harmonica`

## Installation

```toml
[dependencies]
harmonica = { package = "charmed-harmonica", version = "0.1.2" }
```

## Quick Start

```rust
use harmonica::{fps, Spring};

let spring = Spring::new(fps(60), 6.0, 0.2);
let (mut pos, mut vel) = (0.0, 0.0);

// step once
(pos, vel) = spring.update(pos, vel, 100.0);
```

## Key Concepts

- **Spring**: damped harmonic oscillator for smooth, natural motion.
- **Projectile**: simple kinematic motion with gravity.
- **fps(...)**: helper for fixed timesteps (drives deterministic updates).
- **Point / Vector**: simple 2D/3D math helpers for projectile motion.

## API Overview

- `Spring::new(dt, frequency, damping_ratio)` creates a spring.
- `Spring::update(position, velocity, target)` advances one timestep.
- `Projectile::new(dt, position, velocity, gravity)` creates a projectile.
- `Projectile::update()` advances one timestep.
- `fps(60)` returns a fixed timestep value for 60 FPS.

See:
- `crates/harmonica/src/spring.rs`
- `crates/harmonica/src/projectile.rs`

## Feature Flags

- `std` (default): use the standard library.
- `no_std`: supported by disabling default features.

```toml
harmonica = { package = "charmed-harmonica", version = "0.1.2", default-features = false }
```

## Tuning Tips

- **Too bouncy?** Increase damping ratio (e.g., `0.6 → 1.0`).
- **Too sluggish?** Increase frequency (e.g., `4.0 → 8.0`).
- **Overshooting target?** Raise damping ratio or lower frequency.

## Troubleshooting

- **Animation looks jittery**: ensure you feed a fixed timestep (use `fps(...)`).
- **Motion is too slow**: increase frequency and/or lower damping.
- **Motion overshoots**: increase damping ratio.

## Limitations

- Not a full physics engine (no collisions, no rigid bodies).
- Simple projectile model (no drag, no wind, no terrain).

## FAQ

**Does Harmonica allocate on the hot path?**  
No. The API is designed to be lightweight and allocation-free.

**Can I use Harmonica without bubbletea?**  
Yes. It is completely standalone.

**Is it deterministic across platforms?**  
Yes, given the same timestep and inputs.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
