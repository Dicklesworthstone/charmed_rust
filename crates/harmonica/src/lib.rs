#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
// Allow these clippy lints for physics/math code readability
#![allow(clippy::must_use_candidate)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::struct_field_names)]

//! # Harmonica
//!
//! Physics-based animation tools for 2D and 3D applications.
//!
//! Harmonica provides:
//! - **Spring**: A damped harmonic oscillator for smooth, realistic motion
//! - **Projectile**: A simple projectile simulator for particles and projectiles
//!
//! ## Spring Example
//!
//! ```rust
//! use harmonica::{fps, Spring};
//!
//! // Initialize the spring once
//! let spring = Spring::new(fps(60), 6.0, 0.2);
//!
//! // Update in your animation loop
//! let mut pos = 0.0;
//! let mut vel = 0.0;
//! let target = 100.0;
//!
//! // Simulate for 2 seconds (120 frames at 60 FPS)
//! for _ in 0..120 {
//!     (pos, vel) = spring.update(pos, vel, target);
//! }
//!
//! // Position should approach target
//! assert!((pos - target).abs() < 5.0);
//! ```
//!
//! ## Projectile Example
//!
//! ```rust
//! use harmonica::{fps, Point, Vector, Projectile, TERMINAL_GRAVITY};
//!
//! // Create a projectile with gravity
//! let mut projectile = Projectile::new(
//!     fps(60),
//!     Point::new(0.0, 0.0, 0.0),
//!     Vector::new(10.0, -5.0, 0.0),
//!     TERMINAL_GRAVITY,
//! );
//!
//! // Update each frame
//! let pos = projectile.update();
//! ```
//!
//! ## Damping Ratios
//!
//! The damping ratio determines the spring's behavior:
//!
//! - **Over-damped (ζ > 1)**: No oscillation, slow return to equilibrium
//! - **Critically-damped (ζ = 1)**: Fastest return without oscillation
//! - **Under-damped (ζ < 1)**: Oscillates around equilibrium with decay
//!
//! ## Attribution
//!
//! The spring algorithm is based on Ryan Juckett's damped harmonic motion:
//! <https://www.ryanjuckett.com/damped-springs/>

mod projectile;
mod spring;

pub use projectile::{Point, Projectile, Vector, GRAVITY, TERMINAL_GRAVITY};
pub use spring::{fps, Spring};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::projectile::{Point, Projectile, Vector, GRAVITY, TERMINAL_GRAVITY};
    pub use crate::spring::{fps, Spring};
}
