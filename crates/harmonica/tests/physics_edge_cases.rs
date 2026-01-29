#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::cast_lossless)]

//! Additional unit tests for harmonica covering physics edge cases,
//! long-duration stability, energy conservation, and extreme parameters.

use harmonica::{fps, Projectile, Spring, Point, Vector, GRAVITY, TERMINAL_GRAVITY};

// =============================================================================
// Spring: extreme parameters
// =============================================================================

#[test]
fn spring_very_high_damping_ratio() {
    // Damping ratio > 100 — should converge without NaN
    let s = Spring::new(fps(60), 10.0, 150.0);
    let (pos, vel) = s.update(0.0, 0.0, 1.0);
    assert!(pos.is_finite());
    assert!(vel.is_finite());
}

#[test]
fn spring_very_small_angular_frequency() {
    // Angular frequency near zero — sluggish but finite
    let s = Spring::new(fps(60), 0.001, 1.0);
    let (pos, vel) = s.update(0.0, 0.0, 1.0);
    assert!(pos.is_finite());
    assert!(vel.is_finite());
    // With such low stiffness, barely moves in one frame
    assert!(pos.abs() < 0.1);
}

#[test]
fn spring_large_displacement() {
    // Start very far from target
    let s = Spring::new(fps(60), 20.0, 1.0);
    let (pos, vel) = s.update(-1000.0, 0.0, 1000.0);
    assert!(pos.is_finite());
    assert!(vel.is_finite());
}

#[test]
fn spring_opposing_velocity() {
    // Velocity pointing away from target — should still converge
    let s = Spring::new(fps(60), 10.0, 1.0);
    let mut pos = 0.0;
    let mut vel = -100.0; // Moving away from target at 1.0
    for _ in 0..600 {
        let (p, v) = s.update(pos, vel, 1.0);
        pos = p;
        vel = v;
    }
    assert!(pos.is_finite());
    // After 10 seconds at 60fps, should have converged
    assert!((pos - 1.0).abs() < 0.1, "pos={pos} should be near 1.0");
}

// =============================================================================
// Spring: long-duration stability
// =============================================================================

#[test]
fn spring_stability_1000_seconds() {
    let s = Spring::new(fps(60), 15.0, 0.8);
    let mut pos = 0.0;
    let mut vel = 50.0;
    // 60000 frames = 1000 seconds
    for _ in 0..60_000 {
        let (p, v) = s.update(pos, vel, 5.0);
        pos = p;
        vel = v;
        assert!(pos.is_finite(), "pos became non-finite");
        assert!(vel.is_finite(), "vel became non-finite");
    }
    assert!((pos - 5.0).abs() < 0.01, "should converge to 5.0, got {pos}");
    assert!(vel.abs() < 0.01, "velocity should be near zero, got {vel}");
}

#[test]
fn spring_critical_damping_fastest_convergence() {
    // Critical damping (ratio = 1) should reach target faster than over-damped
    let critical = Spring::new(fps(60), 20.0, 1.0);
    let over = Spring::new(fps(60), 20.0, 3.0);

    let mut c_pos = 0.0;
    let mut c_vel = 0.0;
    let mut o_pos = 0.0;
    let mut o_vel = 0.0;

    let target = 1.0;
    let threshold = 0.01;
    let mut c_frames = None;
    let mut o_frames = None;

    for i in 0..600 {
        let (cp, cv) = critical.update(c_pos, c_vel, target);
        c_pos = cp;
        c_vel = cv;
        if c_frames.is_none() && (c_pos - target).abs() < threshold {
            c_frames = Some(i);
        }

        let (op, ov) = over.update(o_pos, o_vel, target);
        o_pos = op;
        o_vel = ov;
        if o_frames.is_none() && (o_pos - target).abs() < threshold {
            o_frames = Some(i);
        }
    }

    let cf = c_frames.expect("critical should converge");
    let of = o_frames.expect("over-damped should converge");
    assert!(
        cf <= of,
        "critical ({cf}) should be <= over-damped ({of})"
    );
}

// =============================================================================
// Projectile: energy conservation (approximate)
// =============================================================================

#[test]
fn projectile_energy_approximately_conserved() {
    // With only gravity (conservative force), total energy should be roughly constant
    let dt = fps(60);
    let initial_pos = Point::new(0.0, 100.0, 0.0);
    let initial_vel = Vector::new(10.0, 20.0, 0.0);
    let mut proj = Projectile::new(dt, initial_pos, initial_vel, GRAVITY);

    let mass = 1.0; // Arbitrary
    let initial_ke = 0.5 * mass * (initial_vel.x.powi(2) + initial_vel.y.powi(2) + initial_vel.z.powi(2));
    let initial_pe = mass * 9.81 * initial_pos.y;
    let initial_energy = initial_ke + initial_pe;

    // Run for 2 seconds (120 frames)
    for _ in 0..120 {
        proj.update();
    }

    let pos = proj.position();
    let vel = proj.velocity();
    let ke = 0.5 * mass * (vel.x.powi(2) + vel.y.powi(2) + vel.z.powi(2));
    let pe = mass * 9.81 * pos.y;
    let final_energy = ke + pe;

    // Euler integration has error, but should be within ~10%
    let relative_error = ((final_energy - initial_energy) / initial_energy).abs();
    assert!(
        relative_error < 0.15,
        "energy error {relative_error:.4} exceeds 15%: initial={initial_energy:.2} final={final_energy:.2}"
    );
}

// =============================================================================
// Projectile: long-duration stability
// =============================================================================

#[test]
fn projectile_long_duration_no_nan() {
    let dt = fps(60);
    let mut proj = Projectile::new(
        dt,
        Point::origin(),
        Vector::new(100.0, 200.0, -50.0),
        GRAVITY,
    );

    // 10 minutes of simulation at 60fps
    for _ in 0..36_000 {
        let pos = proj.update();
        assert!(pos.x.is_finite());
        assert!(pos.y.is_finite());
        assert!(pos.z.is_finite());
    }
}

#[test]
fn projectile_extreme_acceleration() {
    let dt = fps(60);
    let extreme_acc = Vector::new(1e6, -1e6, 1e6);
    let mut proj = Projectile::new(dt, Point::origin(), Vector::zero(), extreme_acc);

    for _ in 0..60 {
        let pos = proj.update();
        assert!(pos.x.is_finite());
        assert!(pos.y.is_finite());
        assert!(pos.z.is_finite());
    }
}

// =============================================================================
// Projectile: trajectory validation
// =============================================================================

#[test]
fn projectile_horizontal_range() {
    // Purely horizontal motion (no gravity) should be linear
    let dt = fps(60);
    let mut proj = Projectile::new(
        dt,
        Point::origin(),
        Vector::new(10.0, 0.0, 0.0),
        Vector::zero(),
    );

    for i in 1..=60 {
        let pos = proj.update();
        let expected_x = 10.0 * dt * i as f64;
        assert!(
            (pos.x - expected_x).abs() < 1e-6,
            "frame {i}: expected x={expected_x}, got {}", pos.x
        );
        assert!((pos.y).abs() < 1e-10);
        assert!((pos.z).abs() < 1e-10);
    }
}

#[test]
fn projectile_3d_components_independent() {
    // Each axis should be independent
    let dt = fps(60);
    let mut proj_x = Projectile::new(dt, Point::origin(), Vector::new(5.0, 0.0, 0.0), Vector::zero());
    let mut proj_y = Projectile::new(dt, Point::origin(), Vector::new(0.0, 5.0, 0.0), Vector::zero());
    let mut proj_all = Projectile::new(dt, Point::origin(), Vector::new(5.0, 5.0, 0.0), Vector::zero());

    for _ in 0..30 {
        let px = proj_x.update();
        let py = proj_y.update();
        let pa = proj_all.update();

        assert!((pa.x - px.x).abs() < 1e-10, "x components should match");
        assert!((pa.y - py.y).abs() < 1e-10, "y components should match");
    }
}

// =============================================================================
// Vector/Point: additional edge cases
// =============================================================================

#[test]
fn vector_normalize_near_zero() {
    // Very small but non-zero vector
    let v = Vector::new(1e-15, 0.0, 0.0);
    let n = v.normalized();
    // Should either be unit or zero depending on implementation
    assert!(n.x.is_finite());
    assert!(n.y.is_finite());
    assert!(n.z.is_finite());
}

#[test]
fn vector_magnitude_3_4_5() {
    let v = Vector::new(3.0, 4.0, 0.0);
    assert!((v.magnitude() - 5.0).abs() < 1e-10);
}

#[test]
fn point_displacement_symmetry() {
    let a = Point::new(1.0, 2.0, 3.0);
    let b = Point::new(4.0, 5.0, 6.0);
    let ab = b - a;
    let ba = a - b;
    // ab + ba should be zero
    let sum = ab + ba;
    assert!(sum.x.abs() < 1e-10);
    assert!(sum.y.abs() < 1e-10);
    assert!(sum.z.abs() < 1e-10);
}

#[test]
fn point_add_vector_roundtrip() {
    let p = Point::new(1.0, 2.0, 3.0);
    let v = Vector::new(10.0, 20.0, 30.0);
    let moved = p + v;
    let displacement = moved - p;
    assert!((displacement.x - v.x).abs() < 1e-10);
    assert!((displacement.y - v.y).abs() < 1e-10);
    assert!((displacement.z - v.z).abs() < 1e-10);
}

// =============================================================================
// Gravity constants
// =============================================================================

#[test]
fn gravity_constants_correct() {
    assert!((GRAVITY.y - (-9.81)).abs() < 1e-10);
    assert!((GRAVITY.x).abs() < 1e-10);
    assert!((GRAVITY.z).abs() < 1e-10);

    assert!((TERMINAL_GRAVITY.y - 9.81).abs() < 1e-10);
    assert!((TERMINAL_GRAVITY.x).abs() < 1e-10);
    assert!((TERMINAL_GRAVITY.z).abs() < 1e-10);
}

#[test]
fn gravity_opposite_directions() {
    let sum = GRAVITY + TERMINAL_GRAVITY;
    assert!(sum.x.abs() < 1e-10);
    assert!(sum.y.abs() < 1e-10);
    assert!(sum.z.abs() < 1e-10);
}

// =============================================================================
// fps function
// =============================================================================

#[test]
fn fps_common_values() {
    assert!((fps(30) - 1.0 / 30.0).abs() < 1e-10);
    assert!((fps(60) - 1.0 / 60.0).abs() < 1e-10);
    assert!((fps(120) - 1.0 / 120.0).abs() < 1e-10);
    assert!((fps(144) - 1.0 / 144.0).abs() < 1e-10);
    assert!((fps(240) - 1.0 / 240.0).abs() < 1e-10);
}

#[test]
fn fps_one() {
    assert!((fps(1) - 1.0).abs() < 1e-10);
}
