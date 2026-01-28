//! Damped harmonic oscillator (spring) implementation.
//!
//! This is a port of Ryan Juckett's simple damped harmonic motion algorithm,
//! originally written in C++ and ported to Go by Charmbracelet.
//!
//! For background on the algorithm see:
//! <https://www.ryanjuckett.com/damped-springs/>
//!
//! # License
//!
//! ```text
//! Copyright (c) 2008-2012 Ryan Juckett
//! http://www.ryanjuckett.com/
//!
//! This software is provided 'as-is', without any express or implied
//! warranty. In no event will the authors be held liable for any damages
//! arising from the use of this software.
//!
//! Permission is granted to anyone to use this software for any purpose,
//! including commercial applications, and to alter it and redistribute it
//! freely, subject to the following restrictions:
//!
//! 1. The origin of this software must not be misrepresented; you must not
//!    claim that you wrote the original software. If you use this software
//!    in a product, an acknowledgment in the product documentation would be
//!    appreciated but is not required.
//!
//! 2. Altered source versions must be plainly marked as such, and must not be
//!    misrepresented as being the original software.
//!
//! 3. This notice may not be removed or altered from any source
//!    distribution.
//!
//! Ported to Go by Charmbracelet, Inc. in 2021.
//! Ported to Rust by Charmed Rust in 2026.
//! ```

/// Machine epsilon for floating point comparisons.
const EPSILON: f64 = f64::EPSILON;

/// Returns a time delta for a given number of frames per second.
///
/// This value can be used as the time delta when initializing a [`Spring`].
/// Note that game engines often provide the time delta as well, which you
/// should use instead of this function if possible.
///
/// If `n` is 0, this returns `0.0`.
///
/// # Example
///
/// ```rust
/// use harmonica::{fps, Spring};
///
/// let spring = Spring::new(fps(60), 5.0, 0.2);
/// ```
#[inline]
pub fn fps(n: u32) -> f64 {
    if n == 0 {
        return 0.0;
    }
    debug_assert!(n > 0, "fps() requires a non-zero frame rate");
    1.0 / n as f64
}

/// Precomputed spring motion parameters for efficient animation updates.
///
/// A `Spring` contains cached coefficients that can be used to efficiently
/// update multiple springs using the same time step, angular frequency, and
/// damping ratio.
///
/// # Creating a Spring
///
/// Use [`Spring::new`] with the time delta (animation frame length), angular
/// frequency, and damping ratio:
///
/// ```rust
/// use harmonica::{fps, Spring};
///
/// // Precompute spring coefficients
/// let spring = Spring::new(fps(60), 5.0, 0.2);
/// ```
///
/// # Damping Ratios
///
/// The damping ratio determines how the spring behaves:
///
/// - **Over-damped (ζ > 1)**: No oscillation, slow approach to equilibrium
/// - **Critically-damped (ζ = 1)**: Fastest approach without oscillation
/// - **Under-damped (ζ < 1)**: Oscillates around equilibrium with decay
///
/// # Example
///
/// ```rust
/// use harmonica::{fps, Spring};
///
/// // Create spring for X and Y positions
/// let mut x = 0.0;
/// let mut x_vel = 0.0;
/// let mut y = 0.0;
/// let mut y_vel = 0.0;
///
/// let spring = Spring::new(fps(60), 5.0, 0.2);
///
/// // In your update loop:
/// (x, x_vel) = spring.update(x, x_vel, 10.0);  // Move X toward 10
/// (y, y_vel) = spring.update(y, y_vel, 20.0);  // Move Y toward 20
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Spring {
    pos_pos_coef: f64,
    pos_vel_coef: f64,
    vel_pos_coef: f64,
    vel_vel_coef: f64,
}

impl Spring {
    /// Creates a new spring, computing the parameters needed to simulate
    /// a damped spring over a given period of time.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time step to advance (essentially the framerate).
    ///   Use [`fps`] to compute this from a frame rate.
    /// * `angular_frequency` - The angular frequency of motion, which affects
    ///   the speed. Higher values make the spring move faster.
    /// * `damping_ratio` - The damping ratio, which determines oscillation:
    ///   - `> 1.0`: Over-damped (no oscillation, slow return)
    ///   - `= 1.0`: Critically-damped (fastest without oscillation)
    ///   - `< 1.0`: Under-damped (oscillates with decay)
    ///
    /// # Example
    ///
    /// ```rust
    /// use harmonica::{fps, Spring};
    ///
    /// // Create an under-damped spring (will oscillate)
    /// let bouncy = Spring::new(fps(60), 6.0, 0.2);
    ///
    /// // Create a critically-damped spring (no oscillation)
    /// let smooth = Spring::new(fps(60), 6.0, 1.0);
    ///
    /// // Create an over-damped spring (very slow, no oscillation)
    /// let sluggish = Spring::new(fps(60), 6.0, 2.0);
    /// ```
    pub fn new(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self {
        // Keep values in a legal range
        let angular_frequency = angular_frequency.max(0.0);
        let damping_ratio = damping_ratio.max(0.0);

        // If there is no angular frequency, the spring will not move
        // and we return identity coefficients
        if angular_frequency < EPSILON {
            return Self {
                pos_pos_coef: 1.0,
                pos_vel_coef: 0.0,
                vel_pos_coef: 0.0,
                vel_vel_coef: 1.0,
            };
        }

        if damping_ratio > 1.0 + EPSILON {
            // Over-damped
            Self::over_damped(delta_time, angular_frequency, damping_ratio)
        } else if damping_ratio < 1.0 - EPSILON {
            // Under-damped
            Self::under_damped(delta_time, angular_frequency, damping_ratio)
        } else {
            // Critically damped
            Self::critically_damped(delta_time, angular_frequency)
        }
    }

    /// Computes coefficients for over-damped spring (damping_ratio > 1).
    fn over_damped(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self {
        let za = -angular_frequency * damping_ratio;
        let zb = angular_frequency * (damping_ratio * damping_ratio - 1.0).sqrt();
        let z1 = za - zb;
        let z2 = za + zb;

        let e1 = exp(z1 * delta_time);
        let e2 = exp(z2 * delta_time);

        let inv_two_zb = 1.0 / (2.0 * zb); // = 1 / (z2 - z1)

        let e1_over_two_zb = e1 * inv_two_zb;
        let e2_over_two_zb = e2 * inv_two_zb;

        let z1e1_over_two_zb = z1 * e1_over_two_zb;
        let z2e2_over_two_zb = z2 * e2_over_two_zb;

        Self {
            pos_pos_coef: e1_over_two_zb * z2 - z2e2_over_two_zb + e2,
            pos_vel_coef: -e1_over_two_zb + e2_over_two_zb,
            vel_pos_coef: (z1e1_over_two_zb - z2e2_over_two_zb + e2) * z2,
            vel_vel_coef: -z1e1_over_two_zb + z2e2_over_two_zb,
        }
    }

    /// Computes coefficients for under-damped spring (damping_ratio < 1).
    fn under_damped(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self {
        let omega_zeta = angular_frequency * damping_ratio;
        let alpha = angular_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();

        let exp_term = exp(-omega_zeta * delta_time);
        let cos_term = cos(alpha * delta_time);
        let sin_term = sin(alpha * delta_time);

        let inv_alpha = 1.0 / alpha;

        let exp_sin = exp_term * sin_term;
        let exp_cos = exp_term * cos_term;
        let exp_omega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

        Self {
            pos_pos_coef: exp_cos + exp_omega_zeta_sin_over_alpha,
            pos_vel_coef: exp_sin * inv_alpha,
            vel_pos_coef: -exp_sin * alpha - omega_zeta * exp_omega_zeta_sin_over_alpha,
            vel_vel_coef: exp_cos - exp_omega_zeta_sin_over_alpha,
        }
    }

    /// Computes coefficients for critically-damped spring (damping_ratio ≈ 1).
    fn critically_damped(delta_time: f64, angular_frequency: f64) -> Self {
        let exp_term = exp(-angular_frequency * delta_time);
        let time_exp = delta_time * exp_term;
        let time_exp_freq = time_exp * angular_frequency;

        Self {
            pos_pos_coef: time_exp_freq + exp_term,
            pos_vel_coef: time_exp,
            vel_pos_coef: -angular_frequency * time_exp_freq,
            vel_vel_coef: -time_exp_freq + exp_term,
        }
    }

    /// Updates position and velocity values against a given target value.
    ///
    /// Call this after creating a spring to update values each frame.
    ///
    /// # Arguments
    ///
    /// * `pos` - Current position
    /// * `vel` - Current velocity
    /// * `equilibrium_pos` - Target position to move toward
    ///
    /// # Returns
    ///
    /// A tuple of `(new_position, new_velocity)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use harmonica::{fps, Spring};
    ///
    /// let spring = Spring::new(fps(60), 5.0, 0.2);
    /// let mut pos = 0.0;
    /// let mut vel = 0.0;
    /// let target = 100.0;
    ///
    /// // Simulate 60 frames (1 second at 60 FPS)
    /// for _ in 0..60 {
    ///     (pos, vel) = spring.update(pos, vel, target);
    /// }
    ///
    /// println!("Position: {pos}, Velocity: {vel}");
    /// ```
    #[inline]
    pub fn update(&self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64) {
        // Update in equilibrium-relative space
        let old_pos = pos - equilibrium_pos;
        let old_vel = vel;

        let new_pos = old_pos * self.pos_pos_coef + old_vel * self.pos_vel_coef + equilibrium_pos;
        let new_vel = old_pos * self.vel_pos_coef + old_vel * self.vel_vel_coef;

        (new_pos, new_vel)
    }
}

// Math helper functions that work in both std and no_std environments

#[cfg(feature = "std")]
#[inline]
fn exp(x: f64) -> f64 {
    x.exp()
}

#[cfg(not(feature = "std"))]
#[inline]
fn exp(x: f64) -> f64 {
    // e^x using the constant E
    libm::exp(x)
}

#[cfg(feature = "std")]
#[inline]
fn sin(x: f64) -> f64 {
    x.sin()
}

#[cfg(not(feature = "std"))]
#[inline]
fn sin(x: f64) -> f64 {
    libm::sin(x)
}

#[cfg(feature = "std")]
#[inline]
fn cos(x: f64) -> f64 {
    x.cos()
}

#[cfg(not(feature = "std"))]
#[inline]
fn cos(x: f64) -> f64 {
    libm::cos(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOLERANCE
    }

    #[test]
    fn test_fps() {
        assert!(approx_eq(fps(60), 1.0 / 60.0));
        assert!(approx_eq(fps(30), 1.0 / 30.0));
        assert!(approx_eq(fps(120), 1.0 / 120.0));
        assert!(approx_eq(fps(0), 0.0));
    }

    #[test]
    fn test_identity_spring() {
        // Zero angular frequency should return unchanged values
        let spring = Spring::new(fps(60), 0.0, 0.5);

        let (new_pos, new_vel) = spring.update(10.0, 5.0, 100.0);

        assert!(approx_eq(new_pos, 10.0));
        assert!(approx_eq(new_vel, 5.0));
    }

    #[test]
    fn test_critically_damped_approaches_target() {
        let spring = Spring::new(fps(60), 5.0, 1.0);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        // Run for 5 seconds at 60 FPS
        for _ in 0..300 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should be very close to target
        assert!(
            (pos - target).abs() < 0.01,
            "Expected pos ≈ {target}, got {pos}"
        );
        assert!(vel.abs() < 0.01, "Expected vel ≈ 0, got {vel}");
    }

    #[test]
    fn test_under_damped_oscillates() {
        let spring = Spring::new(fps(60), 10.0, 0.1);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        let mut crossed_target = false;
        let mut overshot = false;

        // Run for 2 seconds
        for _ in 0..120 {
            let old_pos = pos;
            (pos, vel) = spring.update(pos, vel, target);

            // Check if we crossed the target
            if old_pos < target && pos >= target {
                crossed_target = true;
            }

            // Check if we overshot
            if pos > target {
                overshot = true;
            }
        }

        assert!(crossed_target, "Under-damped spring should cross target");
        assert!(overshot, "Under-damped spring should overshoot target");
    }

    #[test]
    fn test_over_damped_no_oscillation() {
        let spring = Spring::new(fps(60), 5.0, 2.0);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        let mut max_pos: f64 = 0.0;

        // Run for 10 seconds
        for _ in 0..600 {
            (pos, vel) = spring.update(pos, vel, target);
            max_pos = max_pos.max(pos);
        }

        // Should never overshoot
        assert!(
            max_pos <= target + TOLERANCE,
            "Over-damped spring should not overshoot: max_pos={max_pos}, target={target}"
        );

        // Should eventually reach target
        assert!(
            (pos - target).abs() < 1.0,
            "Over-damped spring should approach target"
        );
    }

    #[test]
    fn test_spring_is_copy() {
        let spring = Spring::new(fps(60), 5.0, 0.5);
        let spring2 = spring; // Copy
        let _ = spring.update(0.0, 0.0, 100.0);
        let _ = spring2.update(0.0, 0.0, 100.0);
    }

    #[test]
    fn test_negative_values_clamped() {
        // Negative angular frequency should be clamped to 0
        let spring = Spring::new(fps(60), -5.0, 0.5);
        let (new_pos, new_vel) = spring.update(10.0, 5.0, 100.0);

        // Should act as identity
        assert!(approx_eq(new_pos, 10.0));
        assert!(approx_eq(new_vel, 5.0));
    }

    // =========================================================================
    // bd-228s: Additional spring tests
    // =========================================================================

    #[test]
    fn test_zero_damping_oscillates_indefinitely() {
        // Zero damping should cause infinite oscillation (no energy loss)
        let spring = Spring::new(fps(60), 5.0, 0.0);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        // Run for 10 seconds at 60 FPS
        let mut oscillations = 0;
        let mut last_sign = f64::signum(pos - target);

        for _ in 0..600 {
            (pos, vel) = spring.update(pos, vel, target);
            let current_sign = f64::signum(pos - target);
            if current_sign != last_sign && current_sign != 0.0 {
                oscillations += 1;
                last_sign = current_sign;
            }
        }

        // With zero damping, should oscillate many times
        assert!(
            oscillations >= 5,
            "Zero damping should oscillate indefinitely, got {oscillations} oscillations"
        );
    }

    #[test]
    fn test_very_high_stiffness_snaps() {
        // Very high angular frequency should snap quickly to target
        let spring = Spring::new(fps(60), 100.0, 1.0);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        // Run for just a few frames
        for _ in 0..30 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should be very close to target quickly
        assert!(
            (pos - target).abs() < 1.0,
            "High stiffness should snap quickly, got pos={pos}"
        );
    }

    #[test]
    fn test_negative_target() {
        let spring = Spring::new(fps(60), 5.0, 1.0);
        let mut pos = 100.0;
        let mut vel = 0.0;
        let target = -50.0;

        // Run for 5 seconds
        for _ in 0..300 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should approach negative target
        assert!(
            (pos - target).abs() < 0.1,
            "Should approach negative target, got pos={pos}"
        );
    }

    #[test]
    fn test_very_small_movements() {
        let spring = Spring::new(fps(60), 5.0, 1.0);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 0.001; // Very small target

        for _ in 0..300 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should still converge to tiny target
        assert!(
            (pos - target).abs() < 0.0001,
            "Should handle small movements, got pos={pos}, target={target}"
        );
    }

    #[test]
    fn test_large_time_delta() {
        // Large time delta (low FPS)
        let spring = Spring::new(1.0, 5.0, 1.0); // 1 FPS
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        // Run for 10 "frames" (10 seconds)
        for _ in 0..10 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should still converge (though less accurately)
        assert!(
            (pos - target).abs() < 5.0,
            "Large delta should still converge, got pos={pos}"
        );
    }

    #[test]
    fn test_accumulated_error_bounded() {
        // Run for a long time and check error doesn't grow
        let spring = Spring::new(fps(60), 5.0, 0.5);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        // Run for 60 seconds (3600 frames)
        for _ in 0..3600 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // After settling, error should be tiny
        assert!(
            (pos - target).abs() < 0.001,
            "Accumulated error should be bounded, got pos={pos}"
        );
        assert!(
            vel.abs() < 0.001,
            "Velocity should decay completely, got vel={vel}"
        );
    }

    #[test]
    fn test_spring_default() {
        let spring = Spring::default();
        // Default spring has all coefficients = 0.0
        // update() computes: new_pos = old_pos * 0 + old_vel * 0 + equilibrium
        //                    new_vel = old_pos * 0 + old_vel * 0 = 0
        let (new_pos, new_vel) = spring.update(10.0, 5.0, 100.0);
        // With all-zero coefficients, position snaps to equilibrium
        assert!(approx_eq(new_pos, 100.0));
        assert!(approx_eq(new_vel, 0.0));
    }

    #[test]
    fn test_spring_clone() {
        let spring1 = Spring::new(fps(60), 5.0, 0.5);
        let spring2 = spring1;

        // Both should produce same results
        let result1 = spring1.update(0.0, 0.0, 100.0);
        let result2 = spring2.update(0.0, 0.0, 100.0);

        assert!(approx_eq(result1.0, result2.0));
        assert!(approx_eq(result1.1, result2.1));
    }

    #[test]
    fn test_spring_equilibrium_at_target() {
        // When pos == target and vel == 0, should stay at target
        let spring = Spring::new(fps(60), 5.0, 0.5);
        let target = 50.0;
        let (new_pos, new_vel) = spring.update(target, 0.0, target);

        assert!(approx_eq(new_pos, target));
        assert!(approx_eq(new_vel, 0.0));
    }

    #[test]
    fn test_fps_various_rates() {
        // Common frame rates
        assert!(approx_eq(fps(30), 1.0 / 30.0));
        assert!(approx_eq(fps(60), 1.0 / 60.0));
        assert!(approx_eq(fps(120), 1.0 / 120.0));
        assert!(approx_eq(fps(144), 1.0 / 144.0));
        assert!(approx_eq(fps(240), 1.0 / 240.0));
        assert!(approx_eq(fps(1), 1.0));
    }

    #[test]
    fn test_damping_ratio_boundary() {
        // Test exactly at critical damping boundaries
        let under = Spring::new(fps(60), 5.0, 0.999);
        let critical = Spring::new(fps(60), 5.0, 1.0);
        let over = Spring::new(fps(60), 5.0, 1.001);

        // All should work without panicking
        let _ = under.update(0.0, 0.0, 100.0);
        let _ = critical.update(0.0, 0.0, 100.0);
        let _ = over.update(0.0, 0.0, 100.0);
    }

    #[test]
    fn test_initial_velocity() {
        // Spring with initial velocity should still converge
        let spring = Spring::new(fps(60), 5.0, 1.0);
        let mut pos = 0.0;
        let mut vel = 1000.0; // Large initial velocity
        let target = 50.0;

        for _ in 0..600 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        assert!(
            (pos - target).abs() < 0.1,
            "Should converge despite initial velocity"
        );
    }
}
