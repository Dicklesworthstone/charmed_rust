#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::branches_sharing_code)]

use harmonica::{GRAVITY, Point, Projectile, Spring, Vector, fps};
use proptest::prelude::*;

// =============================================================================
// Spring convergence properties
// =============================================================================

proptest! {
    #[test]
    fn spring_converges_to_target(
        angular_freq in 3.0f64..50.0,
        damping in 0.3f64..3.0,
        initial_pos in -200.0f64..200.0,
        target in -200.0f64..200.0,
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        let mut pos = initial_pos;
        let mut vel = 0.0;

        // Simulate for 20 seconds (1200 frames at 60 FPS)
        for _ in 0..1200 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Should converge to within 5% of initial displacement or 2.0 absolute
        let tolerance = ((initial_pos - target).abs() * 0.05).max(2.0);
        prop_assert!(
            (pos - target).abs() < tolerance,
            "Spring did not converge: pos={}, target={}, freq={}, damp={}, diff={}",
            pos, target, angular_freq, damping, (pos - target).abs()
        );
    }

    #[test]
    fn spring_velocity_decays(
        angular_freq in 2.0f64..20.0,
        damping in 0.3f64..5.0,
        target in -100.0f64..100.0,
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        let mut pos = 0.0;
        let mut vel = 0.0;

        // Simulate for 20 seconds (1200 frames)
        for _ in 0..1200 {
            (pos, vel) = spring.update(pos, vel, target);
        }

        // Velocity should decay to near-zero
        prop_assert!(
            vel.abs() < 1.0,
            "Velocity should decay, got vel={}",
            vel
        );
    }
}

// =============================================================================
// Spring stability properties
// =============================================================================

proptest! {
    #[test]
    fn spring_no_nan_or_inf(
        angular_freq in 0.0f64..100.0,
        damping in 0.0f64..20.0,
        initial_pos in -1e6f64..1e6,
        initial_vel in -1e6f64..1e6,
        target in -1e6f64..1e6,
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        let mut pos = initial_pos;
        let mut vel = initial_vel;

        for _ in 0..120 {
            (pos, vel) = spring.update(pos, vel, target);
            prop_assert!(pos.is_finite(), "pos is not finite: {}", pos);
            prop_assert!(vel.is_finite(), "vel is not finite: {}", vel);
        }
    }

    #[test]
    fn spring_new_never_panics(
        dt in 0.0f64..1.0,
        angular_freq in -10.0f64..100.0,
        damping in -5.0f64..20.0,
    ) {
        // Should never panic, even with negative values (clamped to 0)
        let spring = Spring::new(dt, angular_freq, damping);
        let _ = spring.update(0.0, 0.0, 100.0);
    }

    #[test]
    fn spring_update_never_panics(
        pos in -1e10f64..1e10,
        vel in -1e10f64..1e10,
        target in -1e10f64..1e10,
    ) {
        let spring = Spring::new(fps(60), 5.0, 0.5);
        let (new_pos, new_vel) = spring.update(pos, vel, target);
        prop_assert!(new_pos.is_finite(), "new_pos not finite: {}", new_pos);
        prop_assert!(new_vel.is_finite(), "new_vel not finite: {}", new_vel);
    }
}

// =============================================================================
// Spring physical correctness
// =============================================================================

proptest! {
    #[test]
    fn over_damped_no_overshoot(
        angular_freq in 1.0f64..20.0,
        damping in 1.5f64..10.0, // strictly over-damped
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        let mut max_pos: f64 = 0.0;

        for _ in 0..600 {
            (pos, vel) = spring.update(pos, vel, target);
            max_pos = max_pos.max(pos);
        }

        // Over-damped should not overshoot
        prop_assert!(
            max_pos <= target + 0.1,
            "Over-damped (damping={}) overshot: max_pos={}, target={}",
            damping, max_pos, target
        );
    }

    #[test]
    fn under_damped_oscillates(
        angular_freq in 3.0f64..30.0,
        damping in 0.01f64..0.5, // clearly under-damped
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        let mut pos = 0.0;
        let mut vel = 0.0;
        let target = 100.0;

        let mut overshot = false;

        for _ in 0..300 {
            (pos, vel) = spring.update(pos, vel, target);
            if pos > target + 0.5 {
                overshot = true;
                break;
            }
        }

        prop_assert!(
            overshot,
            "Under-damped (damping={}, freq={}) should overshoot",
            damping, angular_freq
        );
    }

    #[test]
    fn higher_stiffness_reaches_faster(
        base_freq in 2.0f64..10.0,
    ) {
        let slow = Spring::new(fps(60), base_freq, 1.0);
        let fast = Spring::new(fps(60), base_freq * 3.0, 1.0);
        let target = 100.0;

        let mut slow_pos = 0.0;
        let mut slow_vel = 0.0;
        let mut fast_pos = 0.0;
        let mut fast_vel = 0.0;

        // After 30 frames, higher stiffness should be closer to target
        for _ in 0..30 {
            (slow_pos, slow_vel) = slow.update(slow_pos, slow_vel, target);
            (fast_pos, fast_vel) = fast.update(fast_pos, fast_vel, target);
        }

        let slow_dist = (slow_pos - target).abs();
        let fast_dist = (fast_pos - target).abs();

        prop_assert!(
            fast_dist < slow_dist + 1.0,
            "Higher stiffness should be closer: fast_dist={}, slow_dist={}",
            fast_dist, slow_dist
        );
    }

    #[test]
    fn equilibrium_stays_at_target(
        angular_freq in 0.5f64..50.0,
        damping in 0.1f64..10.0,
        target in -1000.0f64..1000.0,
    ) {
        let spring = Spring::new(fps(60), angular_freq, damping);
        // Start exactly at target with zero velocity
        let (new_pos, new_vel) = spring.update(target, 0.0, target);

        prop_assert!(
            (new_pos - target).abs() < 1e-9,
            "At equilibrium, position should stay: new_pos={}, target={}",
            new_pos, target
        );
        prop_assert!(
            new_vel.abs() < 1e-9,
            "At equilibrium, velocity should be zero: new_vel={}",
            new_vel
        );
    }
}

// =============================================================================
// Spring frame independence
// =============================================================================

proptest! {
    #[test]
    fn frame_rate_similar_results(
        angular_freq in 2.0f64..15.0,
        damping in 0.3f64..3.0,
        target in 10.0f64..200.0,
    ) {
        // Compare 60 FPS with 120 FPS after same elapsed time
        let spring_60 = Spring::new(fps(60), angular_freq, damping);
        let spring_120 = Spring::new(fps(120), angular_freq, damping);

        let mut pos_60 = 0.0;
        let mut vel_60 = 0.0;
        let mut pos_120 = 0.0;
        let mut vel_120 = 0.0;

        // Simulate 1 second: 60 frames vs 120 frames
        for _ in 0..60 {
            (pos_60, vel_60) = spring_60.update(pos_60, vel_60, target);
        }
        for _ in 0..120 {
            (pos_120, vel_120) = spring_120.update(pos_120, vel_120, target);
        }

        // Results should be similar (within 10% of target distance)
        let tolerance = target.abs() * 0.1 + 1.0;
        prop_assert!(
            (pos_60 - pos_120).abs() < tolerance,
            "Frame rates diverged: pos_60={}, pos_120={}, diff={}",
            pos_60, pos_120, (pos_60 - pos_120).abs()
        );
    }
}

// =============================================================================
// fps() invariants
// =============================================================================

proptest! {
    #[test]
    fn fps_positive_for_nonzero(n in 1u32..10000) {
        let dt = fps(n);
        prop_assert!(dt > 0.0, "fps({}) should be positive: {}", n, dt);
        prop_assert!(dt.is_finite(), "fps({}) should be finite: {}", n, dt);
    }

    #[test]
    fn fps_inverse_of_n(n in 1u32..10000) {
        let dt = fps(n);
        let expected = 1.0 / n as f64;
        prop_assert!(
            (dt - expected).abs() < 1e-15,
            "fps({}) = {} != 1/{} = {}",
            n, dt, n, expected
        );
    }
}

// =============================================================================
// Projectile stability
// =============================================================================

proptest! {
    #[test]
    fn projectile_no_nan_or_inf(
        px in -1e6f64..1e6,
        py in -1e6f64..1e6,
        pz in -1e6f64..1e6,
        vx in -1e3f64..1e3,
        vy in -1e3f64..1e3,
        vz in -1e3f64..1e3,
        ax in -100.0f64..100.0,
        ay in -100.0f64..100.0,
        az in -100.0f64..100.0,
    ) {
        let mut proj = Projectile::new(
            fps(60),
            Point::new(px, py, pz),
            Vector::new(vx, vy, vz),
            Vector::new(ax, ay, az),
        );

        for _ in 0..120 {
            let pos = proj.update();
            prop_assert!(pos.x.is_finite(), "x not finite: {}", pos.x);
            prop_assert!(pos.y.is_finite(), "y not finite: {}", pos.y);
            prop_assert!(pos.z.is_finite(), "z not finite: {}", pos.z);
        }
    }

    #[test]
    fn projectile_new_never_panics(
        dt in 0.0f64..1.0,
        px in -1e6f64..1e6,
        py in -1e6f64..1e6,
    ) {
        let proj = Projectile::new(
            dt,
            Point::new(px, py, 0.0),
            Vector::zero(),
            GRAVITY,
        );
        let _ = proj.position();
    }
}

// =============================================================================
// Projectile physical correctness
// =============================================================================

proptest! {
    #[test]
    fn zero_acceleration_constant_velocity(
        vx in -100.0f64..100.0,
        vy in -100.0f64..100.0,
        vz in -100.0f64..100.0,
    ) {
        let mut proj = Projectile::new(
            fps(60),
            Point::origin(),
            Vector::new(vx, vy, vz),
            Vector::zero(),
        );

        // After 60 frames (1 second), position should be ~velocity
        for _ in 0..60 {
            proj.update();
        }

        let pos = proj.position();
        let vel = proj.velocity();

        // Velocity should be unchanged (no acceleration)
        prop_assert!((vel.x - vx).abs() < 1e-10, "vx changed: {} != {}", vel.x, vx);
        prop_assert!((vel.y - vy).abs() < 1e-10, "vy changed: {} != {}", vel.y, vy);
        prop_assert!((vel.z - vz).abs() < 1e-10, "vz changed: {} != {}", vel.z, vz);

        // Position should be approximately velocity * time (Euler integration)
        prop_assert!((pos.x - vx).abs() < 1.0, "x mismatch: {} vs {}", pos.x, vx);
        prop_assert!((pos.y - vy).abs() < 1.0, "y mismatch: {} vs {}", pos.y, vy);
        prop_assert!((pos.z - vz).abs() < 1.0, "z mismatch: {} vs {}", pos.z, vz);
    }

    #[test]
    fn gravity_accelerates_downward(
        py_start in 0.0f64..1000.0,
    ) {
        let mut proj = Projectile::new(
            fps(60),
            Point::new(0.0, py_start, 0.0),
            Vector::zero(),
            GRAVITY, // negative y
        );

        // After 30 frames, should have moved in gravity direction
        for _ in 0..30 {
            proj.update();
        }

        let pos = proj.position();
        prop_assert!(
            pos.y < py_start,
            "Gravity should move y downward: start={}, end={}",
            py_start, pos.y
        );
    }

    #[test]
    fn zero_dt_no_movement(
        vx in -100.0f64..100.0,
        vy in -100.0f64..100.0,
    ) {
        let mut proj = Projectile::new(
            0.0,
            Point::new(5.0, 10.0, 15.0),
            Vector::new(vx, vy, 0.0),
            GRAVITY,
        );

        proj.update();

        let pos = proj.position();
        prop_assert!((pos.x - 5.0).abs() < 1e-10, "x moved with dt=0");
        prop_assert!((pos.y - 10.0).abs() < 1e-10, "y moved with dt=0");
        prop_assert!((pos.z - 15.0).abs() < 1e-10, "z moved with dt=0");
    }

    #[test]
    fn x_displacement_linear_no_gravity(
        vx in 1.0f64..100.0,
        frames in 10usize..120,
    ) {
        let dt = fps(60);
        let mut proj = Projectile::new(
            dt,
            Point::origin(),
            Vector::new(vx, 0.0, 0.0),
            Vector::zero(),
        );

        for _ in 0..frames {
            proj.update();
        }

        // x = v * t (Euler: sum of v*dt for n frames = v * n * dt)
        let elapsed = frames as f64 * dt;
        let expected_x = vx * elapsed;
        let actual_x = proj.position().x;

        prop_assert!(
            (actual_x - expected_x).abs() < 0.5,
            "Expected x={}, got x={} after {} frames at vx={}",
            expected_x, actual_x, frames, vx
        );
    }
}

// =============================================================================
// Projectile setters/getters roundtrip
// =============================================================================

proptest! {
    #[test]
    fn projectile_setters_roundtrip(
        px in -1e6f64..1e6,
        py in -1e6f64..1e6,
        pz in -1e6f64..1e6,
        vx in -1e3f64..1e3,
        vy in -1e3f64..1e3,
        vz in -1e3f64..1e3,
    ) {
        let mut proj = Projectile::new(
            fps(60),
            Point::origin(),
            Vector::zero(),
            Vector::zero(),
        );

        proj.set_position(Point::new(px, py, pz));
        proj.set_velocity(Vector::new(vx, vy, vz));
        proj.set_acceleration(GRAVITY);

        let pos = proj.position();
        prop_assert!((pos.x - px).abs() < 1e-10);
        prop_assert!((pos.y - py).abs() < 1e-10);
        prop_assert!((pos.z - pz).abs() < 1e-10);

        let vel = proj.velocity();
        prop_assert!((vel.x - vx).abs() < 1e-10);
        prop_assert!((vel.y - vy).abs() < 1e-10);
        prop_assert!((vel.z - vz).abs() < 1e-10);

        let acc = proj.acceleration();
        prop_assert!((acc.y - GRAVITY.y).abs() < 1e-10);
    }
}

// =============================================================================
// Vector invariants
// =============================================================================

proptest! {
    #[test]
    fn vector_normalized_has_unit_magnitude(
        x in -1000.0f64..1000.0,
        y in -1000.0f64..1000.0,
        z in -1000.0f64..1000.0,
    ) {
        let v = Vector::new(x, y, z);
        let mag = v.magnitude();

        if mag > 1e-10 {
            let n = v.normalized();
            let n_mag = n.magnitude();
            prop_assert!(
                (n_mag - 1.0).abs() < 1e-9,
                "Normalized magnitude should be 1.0, got {} for ({}, {}, {})",
                n_mag, x, y, z
            );
        } else {
            // Zero vector stays zero when normalized
            let n = v.normalized();
            prop_assert!(n.magnitude() < 1e-9);
        }
    }

    #[test]
    fn vector_addition_commutative(
        ax in -100.0f64..100.0,
        ay in -100.0f64..100.0,
        az in -100.0f64..100.0,
        bx in -100.0f64..100.0,
        by in -100.0f64..100.0,
        bz in -100.0f64..100.0,
    ) {
        let a = Vector::new(ax, ay, az);
        let b = Vector::new(bx, by, bz);

        let ab = a + b;
        let ba = b + a;

        prop_assert!((ab.x - ba.x).abs() < 1e-10, "x: {} != {}", ab.x, ba.x);
        prop_assert!((ab.y - ba.y).abs() < 1e-10, "y: {} != {}", ab.y, ba.y);
        prop_assert!((ab.z - ba.z).abs() < 1e-10, "z: {} != {}", ab.z, ba.z);
    }

    #[test]
    fn scalar_mul_scales_magnitude(
        x in -100.0f64..100.0,
        y in -100.0f64..100.0,
        z in -100.0f64..100.0,
        scalar in -10.0f64..10.0,
    ) {
        let v = Vector::new(x, y, z);
        let scaled = v * scalar;
        let original_mag = v.magnitude();
        let scaled_mag = scaled.magnitude();

        if original_mag > 1e-10 {
            let expected_mag = original_mag * scalar.abs();
            prop_assert!(
                (scaled_mag - expected_mag).abs() < 1e-6,
                "Expected magnitude {}, got {} for scalar={}",
                expected_mag, scaled_mag, scalar
            );
        }
    }

    #[test]
    fn vector_magnitude_non_negative(
        x in -1000.0f64..1000.0,
        y in -1000.0f64..1000.0,
        z in -1000.0f64..1000.0,
    ) {
        let v = Vector::new(x, y, z);
        prop_assert!(v.magnitude() >= 0.0, "Magnitude should be non-negative");
    }

    #[test]
    fn scalar_mul_commutative(
        x in -100.0f64..100.0,
        y in -100.0f64..100.0,
        z in -100.0f64..100.0,
        s in -10.0f64..10.0,
    ) {
        let v = Vector::new(x, y, z);
        let vs = v * s;
        let sv = s * v;

        prop_assert!((vs.x - sv.x).abs() < 1e-10);
        prop_assert!((vs.y - sv.y).abs() < 1e-10);
        prop_assert!((vs.z - sv.z).abs() < 1e-10);
    }
}

// =============================================================================
// Point invariants
// =============================================================================

proptest! {
    #[test]
    fn point_sub_gives_displacement(
        ax in -100.0f64..100.0,
        ay in -100.0f64..100.0,
        az in -100.0f64..100.0,
        bx in -100.0f64..100.0,
        by in -100.0f64..100.0,
        bz in -100.0f64..100.0,
    ) {
        let a = Point::new(ax, ay, az);
        let b = Point::new(bx, by, bz);
        let v = a - b; // displacement from b to a

        // Adding displacement to b should give a
        let reconstructed = b + v;
        prop_assert!((reconstructed.x - ax).abs() < 1e-10);
        prop_assert!((reconstructed.y - ay).abs() < 1e-10);
        prop_assert!((reconstructed.z - az).abs() < 1e-10);
    }

    #[test]
    fn point_add_vector_roundtrip(
        px in -100.0f64..100.0,
        py in -100.0f64..100.0,
        pz in -100.0f64..100.0,
        vx in -100.0f64..100.0,
        vy in -100.0f64..100.0,
        vz in -100.0f64..100.0,
    ) {
        let p = Point::new(px, py, pz);
        let v = Vector::new(vx, vy, vz);
        let moved = p + v;
        let back = moved - p; // should be v

        prop_assert!((back.x - vx).abs() < 1e-10);
        prop_assert!((back.y - vy).abs() < 1e-10);
        prop_assert!((back.z - vz).abs() < 1e-10);
    }
}
