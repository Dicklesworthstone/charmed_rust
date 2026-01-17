//! Projectile motion simulation for particles and projectiles.
//!
//! This module provides simple physics-based projectile motion in 3D space.
//!
//! # Example
//!
//! ```rust
//! use harmonica::{fps, Point, Vector, Projectile, TERMINAL_GRAVITY};
//!
//! let mut projectile = Projectile::new(
//!     fps(60),
//!     Point::new(0.0, 0.0, 0.0),
//!     Vector::new(10.0, -5.0, 0.0),
//!     TERMINAL_GRAVITY,
//! );
//!
//! for _ in 0..60 {
//!     let pos = projectile.update();
//!     println!("Position: ({}, {}, {})", pos.x, pos.y, pos.z);
//! }
//! ```

use core::ops::{Add, AddAssign, Mul, Sub};

/// A point in 3D space.
///
/// # Example
///
/// ```rust
/// use harmonica::Point;
///
/// let origin = Point::default();
/// let p = Point::new(1.0, 2.0, 3.0);
///
/// // Points support arithmetic operations
/// let p2 = Point::new(4.0, 5.0, 6.0);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate.
    pub z: f64,
}

impl Point {
    /// Creates a new point with the given coordinates.
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a new 2D point (z = 0).
    #[inline]
    pub const fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Returns the origin point (0, 0, 0).
    #[inline]
    pub const fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    #[inline]
    fn add(self, v: Vector) -> Point {
        Point {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

impl AddAssign<Vector> for Point {
    #[inline]
    fn add_assign(&mut self, v: Vector) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
    }
}

impl Sub for Point {
    type Output = Vector;

    #[inline]
    fn sub(self, other: Point) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

/// A vector in 3D space representing magnitude and direction.
///
/// Vectors are represented as displacement from the origin, where the
/// magnitude is the Euclidean distance and the direction points toward
/// the coordinates.
///
/// # Example
///
/// ```rust
/// use harmonica::Vector;
///
/// let v = Vector::new(1.0, 2.0, 3.0);
/// let scaled = v * 2.0;
/// assert_eq!(scaled.x, 2.0);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl Vector {
    /// Creates a new vector with the given components.
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a new 2D vector (z = 0).
    #[inline]
    pub const fn new_2d(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Returns the zero vector.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Returns the magnitude (length) of the vector.
    #[cfg(feature = "std")]
    #[inline]
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns a normalized (unit) vector with magnitude 1.
    #[cfg(feature = "std")]
    #[inline]
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            return *self;
        }
        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
}

impl Add for Vector {
    type Output = Vector;

    #[inline]
    fn add(self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vector {
    #[inline]
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vector {
    type Output = Vector;

    #[inline]
    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    #[inline]
    fn mul(self, scalar: f64) -> Vector {
        Vector {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    #[inline]
    fn mul(self, v: Vector) -> Vector {
        v * self
    }
}

/// Standard gravity vector for traditional coordinate systems.
///
/// This assumes a coordinate plane where:
/// - Origin is at the bottom-left corner
/// - Y increases upward
/// - Gravity pulls downward (negative Y)
///
/// ```text
///   y             y ±z
///   │             │ /
///   │             │/
///   └───── ±x     └───── ±x
/// ```
pub const GRAVITY: Vector = Vector {
    x: 0.0,
    y: -9.81,
    z: 0.0,
};

/// Gravity vector for terminal coordinate systems.
///
/// This assumes a coordinate plane where:
/// - Origin is at the top-left corner
/// - Y increases downward (typical for terminals)
/// - Gravity pulls downward (positive Y)
pub const TERMINAL_GRAVITY: Vector = Vector {
    x: 0.0,
    y: 9.81,
    z: 0.0,
};

/// A projectile with position, velocity, and acceleration.
///
/// Projectiles simulate simple physics-based motion, updating position
/// based on velocity and velocity based on acceleration each frame.
///
/// # Example
///
/// ```rust
/// use harmonica::{fps, Point, Vector, Projectile, GRAVITY};
///
/// // Create a ball thrown upward
/// let mut ball = Projectile::new(
///     fps(60),
///     Point::new(0.0, 0.0, 0.0),
///     Vector::new(5.0, 20.0, 0.0),  // Initial velocity
///     GRAVITY,
/// );
///
/// // Simulate for 1 second
/// for _ in 0..60 {
///     let pos = ball.update();
///     println!("Ball at y={}", pos.y);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    pos: Point,
    vel: Vector,
    acc: Vector,
    delta_time: f64,
}

impl Projectile {
    /// Creates a new projectile with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - Time step per update (use [`fps`](crate::fps) to compute)
    /// * `position` - Initial position
    /// * `velocity` - Initial velocity
    /// * `acceleration` - Constant acceleration (e.g., gravity)
    ///
    /// # Example
    ///
    /// ```rust
    /// use harmonica::{fps, Point, Vector, Projectile, TERMINAL_GRAVITY};
    ///
    /// let projectile = Projectile::new(
    ///     fps(60),
    ///     Point::new(10.0, 0.0, 0.0),
    ///     Vector::new(5.0, 2.0, 0.0),
    ///     TERMINAL_GRAVITY,
    /// );
    /// ```
    #[inline]
    pub const fn new(
        delta_time: f64,
        position: Point,
        velocity: Vector,
        acceleration: Vector,
    ) -> Self {
        Self {
            pos: position,
            vel: velocity,
            acc: acceleration,
            delta_time,
        }
    }

    /// Updates the projectile's position and velocity for one frame.
    ///
    /// Returns the new position after the update.
    ///
    /// # Example
    ///
    /// ```rust
    /// use harmonica::{fps, Point, Vector, Projectile, GRAVITY};
    ///
    /// let mut p = Projectile::new(
    ///     fps(60),
    ///     Point::origin(),
    ///     Vector::new(10.0, 0.0, 0.0),
    ///     GRAVITY,
    /// );
    ///
    /// // Update returns the new position
    /// let new_pos = p.update();
    /// ```
    #[inline]
    pub fn update(&mut self) -> Point {
        // Update position based on velocity
        self.pos.x += self.vel.x * self.delta_time;
        self.pos.y += self.vel.y * self.delta_time;
        self.pos.z += self.vel.z * self.delta_time;

        // Update velocity based on acceleration
        self.vel.x += self.acc.x * self.delta_time;
        self.vel.y += self.acc.y * self.delta_time;
        self.vel.z += self.acc.z * self.delta_time;

        self.pos
    }

    /// Returns the current position of the projectile.
    #[inline]
    pub const fn position(&self) -> Point {
        self.pos
    }

    /// Returns the current velocity of the projectile.
    #[inline]
    pub const fn velocity(&self) -> Vector {
        self.vel
    }

    /// Returns the acceleration of the projectile.
    #[inline]
    pub const fn acceleration(&self) -> Vector {
        self.acc
    }

    /// Sets the position of the projectile.
    #[inline]
    pub fn set_position(&mut self, pos: Point) {
        self.pos = pos;
    }

    /// Sets the velocity of the projectile.
    #[inline]
    pub fn set_velocity(&mut self, vel: Vector) {
        self.vel = vel;
    }

    /// Sets the acceleration of the projectile.
    #[inline]
    pub fn set_acceleration(&mut self, acc: Vector) {
        self.acc = acc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fps;

    const TOLERANCE: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < TOLERANCE
    }

    #[test]
    fn test_point_new() {
        let p = Point::new(1.0, 2.0, 3.0);
        assert!(approx_eq(p.x, 1.0));
        assert!(approx_eq(p.y, 2.0));
        assert!(approx_eq(p.z, 3.0));
    }

    #[test]
    fn test_point_2d() {
        let p = Point::new_2d(1.0, 2.0);
        assert!(approx_eq(p.z, 0.0));
    }

    #[test]
    fn test_point_add_vector() {
        let p = Point::new(1.0, 2.0, 3.0);
        let v = Vector::new(4.0, 5.0, 6.0);
        let result = p + v;

        assert!(approx_eq(result.x, 5.0));
        assert!(approx_eq(result.y, 7.0));
        assert!(approx_eq(result.z, 9.0));
    }

    #[test]
    fn test_point_sub_point() {
        let p1 = Point::new(5.0, 7.0, 9.0);
        let p2 = Point::new(1.0, 2.0, 3.0);
        let v = p1 - p2;

        assert!(approx_eq(v.x, 4.0));
        assert!(approx_eq(v.y, 5.0));
        assert!(approx_eq(v.z, 6.0));
    }

    #[test]
    fn test_vector_mul_scalar() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let scaled = v * 2.0;

        assert!(approx_eq(scaled.x, 2.0));
        assert!(approx_eq(scaled.y, 4.0));
        assert!(approx_eq(scaled.z, 6.0));
    }

    #[test]
    fn test_scalar_mul_vector() {
        let v = Vector::new(1.0, 2.0, 3.0);
        let scaled = 2.0 * v;

        assert!(approx_eq(scaled.x, 2.0));
        assert!(approx_eq(scaled.y, 4.0));
        assert!(approx_eq(scaled.z, 6.0));
    }

    #[test]
    fn test_vector_magnitude() {
        let v = Vector::new(3.0, 4.0, 0.0);
        assert!(approx_eq(v.magnitude(), 5.0));
    }

    #[test]
    fn test_vector_normalized() {
        let v = Vector::new(3.0, 4.0, 0.0);
        let n = v.normalized();

        assert!(approx_eq(n.magnitude(), 1.0));
        assert!(approx_eq(n.x, 0.6));
        assert!(approx_eq(n.y, 0.8));
    }

    #[test]
    fn test_gravity_constants() {
        assert!(approx_eq(GRAVITY.y, -9.81));
        assert!(approx_eq(TERMINAL_GRAVITY.y, 9.81));
    }

    #[test]
    fn test_projectile_constant_velocity() {
        let dt = fps(60);
        let mut p = Projectile::new(
            dt,
            Point::origin(),
            Vector::new(60.0, 0.0, 0.0), // 60 units/sec
            Vector::zero(),
        );

        // After 1 second (60 frames), should move 60 units
        for _ in 0..60 {
            p.update();
        }

        // Allow for small floating point error
        assert!(
            (p.position().x - 60.0).abs() < 0.1,
            "Expected x ≈ 60, got {}",
            p.position().x
        );
    }

    #[test]
    fn test_projectile_with_gravity() {
        let dt = fps(60);
        let mut p = Projectile::new(
            dt,
            Point::new(0.0, 100.0, 0.0),
            Vector::zero(),
            TERMINAL_GRAVITY,
        );

        // After 1 second, should have fallen due to gravity
        for _ in 0..60 {
            p.update();
        }

        // With terminal gravity (positive y), y should increase
        assert!(p.position().y > 100.0, "Should have fallen (y increased)");
    }

    #[test]
    fn test_projectile_parabolic_motion() {
        let dt = fps(60);
        let mut p = Projectile::new(
            dt,
            Point::origin(),
            Vector::new(10.0, -10.0, 0.0), // Throw up and right
            TERMINAL_GRAVITY,
        );

        let mut max_height = f64::MAX;

        // Find the apex
        for _ in 0..120 {
            p.update();
            if p.position().y < max_height {
                max_height = p.position().y;
            }
        }

        // Should have reached a minimum y (apex) and then fallen back
        assert!(
            max_height < 0.0,
            "Should have gone up (negative y in terminal coords)"
        );
    }

    #[test]
    fn test_projectile_accessors() {
        let p = Projectile::new(
            fps(60),
            Point::new(1.0, 2.0, 3.0),
            Vector::new(4.0, 5.0, 6.0),
            Vector::new(7.0, 8.0, 9.0),
        );

        assert_eq!(p.position(), Point::new(1.0, 2.0, 3.0));
        assert_eq!(p.velocity(), Vector::new(4.0, 5.0, 6.0));
        assert_eq!(p.acceleration(), Vector::new(7.0, 8.0, 9.0));
    }
}
