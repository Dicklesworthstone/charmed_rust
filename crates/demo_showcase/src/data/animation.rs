//! Animation subsystem using spring physics.
//!
//! This module provides reusable animation primitives driven by harmonica's
//! damped spring physics. Animations are deterministic and bounded.
//!
//! # Usage
//!
//! For simple single-value animations:
//!
//! ```rust,ignore
//! use demo_showcase::data::animation::AnimatedValue;
//!
//! let mut progress = AnimatedValue::new(0.0);
//! progress.animate_to(100.0);
//!
//! // In your update loop:
//! if progress.tick() {
//!     // Value changed, re-render
//! }
//!
//! println!("Progress: {}", progress.get());
//! ```
//!
//! For managing multiple named animations:
//!
//! ```rust,ignore
//! use demo_showcase::data::animation::Animator;
//!
//! let mut animator = Animator::new(true); // animations enabled
//! animator.animate("progress", 100.0);
//! animator.animate("opacity", 1.0);
//!
//! // Tick all at once:
//! if animator.tick() {
//!     // At least one value changed
//! }
//! ```
//!
//! # Respecting Reduce Motion
//!
//! The `Animator` struct takes an `enabled` flag. When disabled,
//! `animate()` calls snap instantly to the target value.

use std::collections::HashMap;

use harmonica::{Spring, fps};

/// Default frames per second for animation timing.
///
/// This should match the application's tick rate.
const DEFAULT_FPS: u32 = 60;

/// Default angular frequency for spring animations.
///
/// Higher values = faster motion. 6.0 is a good balance
/// of responsiveness without being too snappy.
const DEFAULT_FREQUENCY: f64 = 6.0;

/// Default damping ratio (slightly underdamped for natural bounce).
///
/// - `< 1.0`: underdamped (oscillates, bouncy)
/// - `= 1.0`: critically damped (fastest without overshoot)
/// - `> 1.0`: overdamped (slow, no overshoot)
///
/// 0.8 gives a subtle, pleasant bounce.
const DEFAULT_DAMPING: f64 = 0.8;

/// Threshold below which a value is considered "at rest".
const REST_THRESHOLD: f64 = 0.001;

/// Velocity threshold for considering animation complete.
const VELOCITY_THRESHOLD: f64 = 0.01;

/// A value that animates toward a target using spring physics.
///
/// This is the core primitive for all animations. It wraps a single `f64`
/// and uses harmonica's [`Spring`] to smoothly transition toward targets.
///
/// # Determinism
///
/// Animations are fully deterministic given the same sequence of
/// `tick()` calls and target changes. This makes testing straightforward.
#[derive(Debug, Clone)]
pub struct AnimatedValue {
    /// Current value.
    value: f64,
    /// Current velocity.
    velocity: f64,
    /// Target value.
    target: f64,
    /// Spring physics configuration.
    spring: Spring,
    /// Whether the animation is currently active.
    active: bool,
}

impl AnimatedValue {
    /// Create a new animated value at rest.
    ///
    /// The value starts at `initial` with no velocity and no animation.
    #[must_use]
    pub fn new(initial: f64) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            spring: Spring::new(fps(DEFAULT_FPS), DEFAULT_FREQUENCY, DEFAULT_DAMPING),
            active: false,
        }
    }

    /// Create a new animated value with custom spring parameters.
    ///
    /// # Arguments
    ///
    /// * `initial` - Starting value
    /// * `frequency` - Angular frequency (higher = faster)
    /// * `damping` - Damping ratio (`< 1.0` bouncy, `1.0` smooth, `> 1.0` sluggish)
    #[must_use]
    pub fn with_spring(initial: f64, frequency: f64, damping: f64) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            spring: Spring::new(fps(DEFAULT_FPS), frequency, damping),
            active: false,
        }
    }

    /// Set the target value, starting animation.
    ///
    /// If the target is already close enough, no animation starts.
    pub fn animate_to(&mut self, target: f64) {
        if (self.target - target).abs() > REST_THRESHOLD {
            self.target = target;
            self.active = true;
        }
    }

    /// Immediately set the value (no animation).
    ///
    /// This snaps the value to the given position, clearing any
    /// ongoing animation and velocity.
    pub const fn set(&mut self, value: f64) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
        self.active = false;
    }

    /// Update the animation by one frame.
    ///
    /// Returns `true` if the value changed (animation is active).
    ///
    /// Call this once per frame/tick. The animation uses the FPS
    /// configured at construction time.
    pub fn tick(&mut self) -> bool {
        if !self.active {
            return false;
        }

        let (new_value, new_velocity) = self.spring.update(self.value, self.velocity, self.target);
        self.value = new_value;
        self.velocity = new_velocity;

        // Check if at rest
        let distance = (self.value - self.target).abs();
        let velocity_mag = self.velocity.abs();

        if distance < REST_THRESHOLD && velocity_mag < VELOCITY_THRESHOLD {
            // Snap to exact target and stop
            self.value = self.target;
            self.velocity = 0.0;
            self.active = false;
        }

        true
    }

    /// Get the current value.
    #[must_use]
    pub const fn get(&self) -> f64 {
        self.value
    }

    /// Get the current value as an integer (rounded).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn get_int(&self) -> i64 {
        self.value.round() as i64
    }

    /// Get the current value as a positive integer (clamped and rounded).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn get_usize(&self) -> usize {
        self.value.max(0.0).round() as usize
    }

    /// Get the target value.
    #[must_use]
    pub const fn target(&self) -> f64 {
        self.target
    }

    /// Check if animation is active.
    #[must_use]
    pub const fn is_animating(&self) -> bool {
        self.active
    }

    /// Get the current velocity.
    #[must_use]
    pub const fn velocity(&self) -> f64 {
        self.velocity
    }
}

impl Default for AnimatedValue {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Manages a collection of named animated values.
///
/// This is useful when you have multiple values that need to animate
/// and you want to tick them all together.
///
/// # Respecting Reduce Motion
///
/// Pass `enabled: false` to disable animations. When disabled,
/// `animate()` calls will snap instantly to the target value.
#[derive(Debug, Clone)]
pub struct Animator {
    /// Named animated values.
    values: HashMap<String, AnimatedValue>,
    /// Whether animations are enabled (respects `use_animations()`).
    enabled: bool,
}

impl Animator {
    /// Create a new animator.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether animations are enabled. Pass `app.use_animations()`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            values: HashMap::new(),
            enabled,
        }
    }

    /// Set whether animations are enabled.
    ///
    /// Call this when the animation setting changes.
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if animations are enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Animate a value toward a target.
    ///
    /// If the key doesn't exist, creates a new `AnimatedValue` starting at 0.0.
    /// If animations are disabled, snaps instantly to the target.
    pub fn animate(&mut self, key: &str, target: f64) {
        let value = self
            .values
            .entry(key.to_string())
            .or_insert_with(|| AnimatedValue::new(0.0));

        if self.enabled {
            value.animate_to(target);
        } else {
            value.set(target);
        }
    }

    /// Animate a value from a specific starting point toward a target.
    ///
    /// If the key doesn't exist, creates a new `AnimatedValue` starting at `from`.
    /// If animations are disabled, snaps instantly to the target.
    pub fn animate_from(&mut self, key: &str, from: f64, target: f64) {
        let value = self
            .values
            .entry(key.to_string())
            .or_insert_with(|| AnimatedValue::new(from));

        if self.enabled {
            value.animate_to(target);
        } else {
            value.set(target);
        }
    }

    /// Get a value, or `None` if not tracked.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<f64> {
        self.values.get(key).map(AnimatedValue::get)
    }

    /// Get a value as an integer (rounded), or `None` if not tracked.
    #[must_use]
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.values.get(key).map(AnimatedValue::get_int)
    }

    /// Get a value as a usize (clamped and rounded), or `None` if not tracked.
    #[must_use]
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.values.get(key).map(AnimatedValue::get_usize)
    }

    /// Get a value, or a default if not tracked.
    #[must_use]
    pub fn get_or(&self, key: &str, default: f64) -> f64 {
        self.get(key).unwrap_or(default)
    }

    /// Tick all animations.
    ///
    /// Returns `true` if any value changed.
    pub fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let mut any_changed = false;
        for value in self.values.values_mut() {
            if value.tick() {
                any_changed = true;
            }
        }
        any_changed
    }

    /// Check if any animations are active.
    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.enabled && self.values.values().any(AnimatedValue::is_animating)
    }

    /// Get the number of tracked values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if no values are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Remove a tracked value.
    pub fn remove(&mut self, key: &str) -> Option<AnimatedValue> {
        self.values.remove(key)
    }

    /// Clear all tracked values.
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

impl Default for Animator {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // AnimatedValue tests
    // =========================================================================

    #[test]
    fn animated_value_starts_at_rest() {
        let value = AnimatedValue::new(50.0);
        assert!((value.get() - 50.0).abs() < f64::EPSILON);
        assert!(!value.is_animating());
    }

    #[test]
    fn animated_value_animate_to_starts_animation() {
        let mut value = AnimatedValue::new(0.0);
        value.animate_to(100.0);
        assert!(value.is_animating());
        assert!((value.target() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animated_value_tick_advances() {
        let mut value = AnimatedValue::new(0.0);
        value.animate_to(100.0);

        let old_value = value.get();
        value.tick();
        let new_value = value.get();

        // Value should have moved toward target
        assert!(new_value > old_value);
    }

    #[test]
    fn animated_value_reaches_target() {
        let mut value = AnimatedValue::new(0.0);
        value.animate_to(100.0);

        // Run for many frames
        for _ in 0..300 {
            if !value.is_animating() {
                break;
            }
            value.tick();
        }

        // Should be at target (or very close)
        assert!((value.get() - 100.0).abs() < 0.1);
        assert!(!value.is_animating());
    }

    #[test]
    fn animated_value_set_snaps_instantly() {
        let mut value = AnimatedValue::new(0.0);
        value.animate_to(100.0);
        value.tick(); // Start moving

        value.set(50.0);

        assert!((value.get() - 50.0).abs() < f64::EPSILON);
        assert!(!value.is_animating());
        assert!(value.velocity().abs() < f64::EPSILON);
    }

    #[test]
    fn animated_value_deterministic() {
        let mut v1 = AnimatedValue::new(0.0);
        let mut v2 = AnimatedValue::new(0.0);

        v1.animate_to(100.0);
        v2.animate_to(100.0);

        for _ in 0..60 {
            v1.tick();
            v2.tick();
        }

        assert!((v1.get() - v2.get()).abs() < f64::EPSILON);
    }

    #[test]
    fn animated_value_custom_spring() {
        // Critically damped (no overshoot)
        let mut smooth = AnimatedValue::with_spring(0.0, 6.0, 1.0);
        // Bouncy (underdamped)
        let mut bouncy = AnimatedValue::with_spring(0.0, 6.0, 0.3);

        smooth.animate_to(100.0);
        bouncy.animate_to(100.0);

        // Track max values
        let mut smooth_max: f64 = 0.0;
        let mut bouncy_max: f64 = 0.0;

        for _ in 0..120 {
            smooth.tick();
            bouncy.tick();
            smooth_max = smooth_max.max(smooth.get());
            bouncy_max = bouncy_max.max(bouncy.get());
        }

        // Bouncy should overshoot, smooth should not (or barely)
        assert!(bouncy_max > 100.0, "bouncy should overshoot");
        assert!(smooth_max < 101.0, "smooth should not overshoot much");
    }

    #[test]
    fn animated_value_get_int() {
        let mut value = AnimatedValue::new(0.0);
        value.set(42.7);
        assert_eq!(value.get_int(), 43);

        value.set(-5.3);
        assert_eq!(value.get_int(), -5);
    }

    #[test]
    fn animated_value_get_usize() {
        let mut value = AnimatedValue::new(0.0);
        value.set(42.7);
        assert_eq!(value.get_usize(), 43);

        value.set(-5.3);
        assert_eq!(value.get_usize(), 0); // clamped
    }

    // =========================================================================
    // Animator tests
    // =========================================================================

    #[test]
    fn animator_enabled() {
        let mut animator = Animator::new(true);
        animator.animate("x", 100.0);

        // Should start animating
        let value = animator.values.get("x").unwrap();
        assert!(value.is_animating());
    }

    #[test]
    fn animator_disabled_snaps() {
        let mut animator = Animator::new(false);
        animator.animate("x", 100.0);

        // Should snap instantly
        let value = animator.get("x").unwrap();
        assert!((value - 100.0).abs() < f64::EPSILON);

        // No animation
        let av = animator.values.get("x").unwrap();
        assert!(!av.is_animating());
    }

    #[test]
    fn animator_tick_all() {
        let mut animator = Animator::new(true);
        animator.animate("a", 100.0);
        animator.animate("b", 200.0);

        let changed = animator.tick();
        assert!(changed);

        // Both should have moved
        assert!(animator.get("a").unwrap() > 0.0);
        assert!(animator.get("b").unwrap() > 0.0);
    }

    #[test]
    fn animator_is_animating() {
        let mut animator = Animator::new(true);
        assert!(!animator.is_animating());

        animator.animate("x", 100.0);
        assert!(animator.is_animating());

        // Run until done
        for _ in 0..300 {
            if !animator.is_animating() {
                break;
            }
            animator.tick();
        }

        assert!(!animator.is_animating());
    }

    #[test]
    fn animator_get_or() {
        let animator = Animator::new(true);
        assert!((animator.get_or("missing", 42.0) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn animator_set_enabled_toggle() {
        let mut animator = Animator::new(true);
        animator.animate("x", 100.0);
        assert!(animator.is_animating());

        animator.set_enabled(false);
        animator.animate("y", 200.0);

        // y should snap
        assert!((animator.get("y").unwrap() - 200.0).abs() < f64::EPSILON);
        // tick should do nothing
        assert!(!animator.tick());
    }

    #[test]
    fn animator_remove_and_clear() {
        let mut animator = Animator::new(true);
        animator.animate("a", 10.0);
        animator.animate("b", 20.0);
        assert_eq!(animator.len(), 2);

        animator.remove("a");
        assert_eq!(animator.len(), 1);
        assert!(animator.get("a").is_none());

        animator.clear();
        assert!(animator.is_empty());
    }
}
