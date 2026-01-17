# Harmonica - Spring Animation Library

## Essence

Harmonica provides physics-based animation primitives for smooth, realistic motion:
1. **Spring** - Damped harmonic oscillator for easing animations
2. **Projectile** - Simple projectile motion for particles

## Source Analysis

### Go API Surface

```go
// Spring - cached coefficients for efficient updates
type Spring struct {
    posPosCoef, posVelCoef float64
    velPosCoef, velVelCoef float64
}

func FPS(n int) float64
func NewSpring(deltaTime, angularFrequency, dampingRatio float64) Spring
func (s Spring) Update(pos, vel, equilibriumPos float64) (newPos, newVel float64)

// Projectile - 3D projectile motion
type Point struct { X, Y, Z float64 }
type Vector struct { X, Y, Z float64 }

var Gravity = Vector{0, -9.81, 0}
var TerminalGravity = Vector{0, 9.81, 0}

func NewProjectile(deltaTime float64, pos Point, vel, acc Vector) *Projectile
func (p *Projectile) Update() Point
func (p *Projectile) Position() Point
func (p *Projectile) Velocity() Vector
func (p *Projectile) Acceleration() Vector
```

### Algorithm Details

The spring uses Ryan Juckett's damped harmonic motion algorithm:
- **Over-damped** (ζ > 1): No oscillation, slow return
- **Critically-damped** (ζ = 1): Fastest return without oscillation
- **Under-damped** (ζ < 1): Oscillates with decay

The algorithm precomputes four coefficients based on damping regime, then applies:
```
newPos = oldPos * posPosCoef + oldVel * posVelCoef + equilibriumPos
newVel = oldPos * velPosCoef + oldVel * velVelCoef
```

## Rust Design

### Public API

```rust
/// Frame-rate helper
pub fn fps(n: u32) -> f64;

/// Precomputed spring coefficients
#[derive(Debug, Clone, Copy, Default)]
pub struct Spring {
    pos_pos_coef: f64,
    pos_vel_coef: f64,
    vel_pos_coef: f64,
    vel_vel_coef: f64,
}

impl Spring {
    /// Create spring with given time delta, angular frequency, and damping ratio
    pub fn new(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self;

    /// Update position and velocity toward equilibrium
    pub fn update(&self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64);
}

/// 3D point
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 3D vector
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Standard gravity (y-down at bottom)
pub const GRAVITY: Vector = Vector { x: 0.0, y: -9.81, z: 0.0 };

/// Terminal gravity (y-down at top, for terminal UIs)
pub const TERMINAL_GRAVITY: Vector = Vector { x: 0.0, y: 9.81, z: 0.0 };

/// Projectile with position, velocity, acceleration
pub struct Projectile {
    pos: Point,
    vel: Vector,
    acc: Vector,
    delta_time: f64,
}

impl Projectile {
    pub fn new(delta_time: f64, pos: Point, vel: Vector, acc: Vector) -> Self;
    pub fn update(&mut self) -> Point;
    pub fn position(&self) -> Point;
    pub fn velocity(&self) -> Vector;
    pub fn acceleration(&self) -> Vector;
}
```

### Rust Enhancements

1. **`#[derive(Copy)]`** - Spring and primitives are Copy for zero-cost passing
2. **`no_std` support** - Pure math, no heap allocation needed
3. **`const fn`** where possible for compile-time evaluation
4. **Operator overloads** for Point/Vector arithmetic
5. **`From` implementations** for easy construction

### Implementation Notes

- Use `f64::EPSILON` from std or define machine epsilon
- Match Go's `math.Max(0.0, x)` with `x.max(0.0)`
- All math functions available in `std::f64` (or `libm` for no_std)

## Test Cases

1. **Identity spring** - zero angular frequency returns unchanged values
2. **Critically damped** - reaches target without oscillation
3. **Under-damped** - oscillates around target
4. **Over-damped** - slow approach without oscillation
5. **Projectile gravity** - parabolic motion

## File Structure

```
crates/harmonica/
├── Cargo.toml
└── src/
    ├── lib.rs       # Re-exports
    ├── spring.rs    # Spring implementation
    └── projectile.rs # Projectile implementation
```
