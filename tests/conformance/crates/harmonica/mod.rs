//! Conformance tests for the harmonica crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of spring physics and projectile motion matches
//! the behavior of the original Go library.
//!
//! Test categories:
//! - Spring physics: damped harmonic oscillator tests
//! - Projectile motion: 3D physics with gravity
//! - FPS utility: frame rate to delta time conversion

use crate::harness::{FixtureLoader, TestFixture};
use harmonica::{fps, Point, Projectile, Spring, Vector, GRAVITY};
use serde::Deserialize;

/// Epsilon for floating point comparisons
/// Note: 1e-6 accounts for floating-point differences between Go and Rust
/// due to compiler optimizations, order of operations, and math library differences.
/// The spring physics equations involve exp(), cos(), sin() which can have small
/// implementation differences. Velocity calculations tend to have larger deltas
/// (up to ~2e-8) compared to position (up to ~3e-9) for single steps.
const EPSILON: f64 = 1e-6;

/// Looser epsilon for projectile tests due to floating point accumulation
const PROJECTILE_EPSILON: f64 = 1e-6;

/// FPS epsilon for truncated decimal comparisons
const FPS_EPSILON: f64 = 1e-7;

/// Check if two f64 values are approximately equal
/// Uses both absolute and relative comparison for robustness with large values
fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    let diff = (a - b).abs();
    // Absolute comparison for small values
    if diff < epsilon {
        return true;
    }
    // Relative comparison for large values (handles proportional scaling)
    let max_val = a.abs().max(b.abs());
    if max_val > 1.0 {
        diff / max_val < epsilon
    } else {
        false
    }
}

/// Input for spring tests
#[derive(Debug, Deserialize)]
struct SpringInput {
    frequency: f64,
    damping: f64,
    current_pos: f64,
    target_pos: f64,
    velocity: f64,
    delta_time: f64,
}

/// Expected output for spring tests
#[derive(Debug, Deserialize)]
struct SpringOutput {
    new_pos: f64,
    new_velocity: f64,
}

/// Input for spring convergence tests
#[derive(Debug, Deserialize)]
struct SpringConvergenceInput {
    frequency: f64,
    damping: f64,
    start_pos: f64,
    target_pos: f64,
    steps: usize,
}

/// Step in convergence trajectory
#[derive(Debug, Deserialize)]
struct ConvergenceStep {
    pos: f64,
    vel: f64,
}

/// Input for projectile tests
#[derive(Debug, Deserialize)]
struct ProjectileInput {
    x: f64,
    y: f64,
    z: f64,
    vel_x: f64,
    vel_y: f64,
    vel_z: f64,
    gravity: f64,
    delta_time: f64,
}

/// Expected output for projectile tests
#[derive(Debug, Deserialize)]
struct ProjectileOutput {
    x: f64,
    y: f64,
    z: f64,
    vel_x: f64,
    vel_y: f64,
    vel_z: f64,
}

/// Input for projectile trajectory tests
#[derive(Debug, Deserialize)]
struct TrajectoryInput {
    gravity: f64,
    start_pos: Vec<f64>,
    start_vel: Vec<f64>,
    steps: usize,
}

/// Step in trajectory
#[derive(Debug, Deserialize)]
struct TrajectoryStep {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
}

/// Input for FPS tests
#[derive(Debug, Deserialize)]
struct FpsInput {
    fps: u32,
}

/// Expected output for FPS tests
#[derive(Debug, Deserialize)]
struct FpsOutput {
    delta: f64,
}

/// Run a single spring test
fn run_spring_test(fixture: &TestFixture) -> Result<(), String> {
    let input: SpringInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: SpringOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let spring = Spring::new(input.delta_time, input.frequency, input.damping);
    let (new_pos, new_vel) = spring.update(input.current_pos, input.velocity, input.target_pos);

    if !approx_eq(new_pos, expected.new_pos, EPSILON) {
        return Err(format!(
            "Position mismatch: expected {:.15}, got {:.15}, delta {:.15e}",
            expected.new_pos,
            new_pos,
            (expected.new_pos - new_pos).abs()
        ));
    }

    if !approx_eq(new_vel, expected.new_velocity, EPSILON) {
        return Err(format!(
            "Velocity mismatch: expected {:.15}, got {:.15}, delta {:.15e}",
            expected.new_velocity,
            new_vel,
            (expected.new_velocity - new_vel).abs()
        ));
    }

    Ok(())
}

/// Run spring convergence test
fn run_spring_convergence_test(fixture: &TestFixture) -> Result<(), String> {
    let input: SpringConvergenceInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: Vec<ConvergenceStep> = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let delta_time = fps(60); // 60 FPS as per Go tests
    let spring = Spring::new(delta_time, input.frequency, input.damping);

    let mut pos = input.start_pos;
    let mut vel = 0.0;

    for (i, expected_step) in expected.iter().enumerate() {
        (pos, vel) = spring.update(pos, vel, input.target_pos);

        if !approx_eq(pos, expected_step.pos, EPSILON) {
            return Err(format!(
                "Step {} position mismatch: expected {:.15}, got {:.15}",
                i + 1,
                expected_step.pos,
                pos
            ));
        }

        if !approx_eq(vel, expected_step.vel, EPSILON) {
            return Err(format!(
                "Step {} velocity mismatch: expected {:.15}, got {:.15}",
                i + 1,
                expected_step.vel,
                vel
            ));
        }
    }

    Ok(())
}

/// Run a single projectile test
fn run_projectile_test(fixture: &TestFixture) -> Result<(), String> {
    let input: ProjectileInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: ProjectileOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    // In Go harmonica, positive gravity means downward acceleration (terminal coordinates)
    // negative gravity means upward acceleration (inverted)
    // The gravity input is the magnitude with sign indicating direction
    // We need to apply it in the Y direction with the sign preserved but inverted
    // because Go's projectile uses: vel.Y += acceleration.Y * dt
    // For standard gravity (9.81), we want negative Y acceleration
    // For terminal gravity (-9.81 in fixture), we want positive Y acceleration
    let gravity = Vector::new(0.0, -input.gravity, 0.0);

    let mut projectile = Projectile::new(
        input.delta_time,
        Point::new(input.x, input.y, input.z),
        Vector::new(input.vel_x, input.vel_y, input.vel_z),
        gravity,
    );

    let new_pos = projectile.update();
    let new_vel = projectile.velocity();

    // Check position (use looser epsilon due to floating point differences)
    if !approx_eq(new_pos.x, expected.x, PROJECTILE_EPSILON) {
        return Err(format!(
            "X position mismatch: expected {}, got {}",
            expected.x, new_pos.x
        ));
    }
    if !approx_eq(new_pos.y, expected.y, PROJECTILE_EPSILON) {
        return Err(format!(
            "Y position mismatch: expected {}, got {}",
            expected.y, new_pos.y
        ));
    }
    if !approx_eq(new_pos.z, expected.z, PROJECTILE_EPSILON) {
        return Err(format!(
            "Z position mismatch: expected {}, got {}",
            expected.z, new_pos.z
        ));
    }

    // Check velocity
    if !approx_eq(new_vel.x, expected.vel_x, PROJECTILE_EPSILON) {
        return Err(format!(
            "X velocity mismatch: expected {}, got {}",
            expected.vel_x, new_vel.x
        ));
    }
    if !approx_eq(new_vel.y, expected.vel_y, PROJECTILE_EPSILON) {
        return Err(format!(
            "Y velocity mismatch: expected {}, got {}",
            expected.vel_y, new_vel.y
        ));
    }
    if !approx_eq(new_vel.z, expected.vel_z, PROJECTILE_EPSILON) {
        return Err(format!(
            "Z velocity mismatch: expected {}, got {}",
            expected.vel_z, new_vel.z
        ));
    }

    Ok(())
}

/// Run projectile trajectory test
fn run_trajectory_test(fixture: &TestFixture) -> Result<(), String> {
    let input: TrajectoryInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: Vec<TrajectoryStep> = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let delta_time = fps(60);
    let gravity = Vector::new(0.0, -input.gravity, 0.0);

    let mut projectile = Projectile::new(
        delta_time,
        Point::new(input.start_pos[0], input.start_pos[1], input.start_pos[2]),
        Vector::new(input.start_vel[0], input.start_vel[1], input.start_vel[2]),
        gravity,
    );

    for (i, expected_step) in expected.iter().enumerate() {
        let pos = projectile.update();
        let vel = projectile.velocity();

        if !approx_eq(pos.x, expected_step.x, PROJECTILE_EPSILON) {
            return Err(format!(
                "Step {} X position mismatch: expected {}, got {}",
                i + 1,
                expected_step.x,
                pos.x
            ));
        }
        if !approx_eq(pos.y, expected_step.y, PROJECTILE_EPSILON) {
            return Err(format!(
                "Step {} Y position mismatch: expected {}, got {}",
                i + 1,
                expected_step.y,
                pos.y
            ));
        }
        if !approx_eq(vel.x, expected_step.vx, PROJECTILE_EPSILON) {
            return Err(format!(
                "Step {} X velocity mismatch: expected {}, got {}",
                i + 1,
                expected_step.vx,
                vel.x
            ));
        }
        if !approx_eq(vel.y, expected_step.vy, PROJECTILE_EPSILON) {
            return Err(format!(
                "Step {} Y velocity mismatch: expected {}, got {}",
                i + 1,
                expected_step.vy,
                vel.y
            ));
        }
    }

    Ok(())
}

/// Run FPS test
fn run_fps_test(fixture: &TestFixture) -> Result<(), String> {
    let input: FpsInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: FpsOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let actual = fps(input.fps);

    if !approx_eq(actual, expected.delta, FPS_EPSILON) {
        return Err(format!(
            "FPS({}) delta mismatch: expected {}, got {}",
            input.fps, expected.delta, actual
        ));
    }

    Ok(())
}

/// Run all harmonica conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    // Load fixtures
    let fixtures = match loader.load_crate("harmonica") {
        Ok(f) => f,
        Err(e) => {
            results.push(("load_fixtures", Err(format!("Failed to load fixtures: {}", e))));
            return results;
        }
    };

    println!(
        "Loaded {} tests from harmonica.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    // Run each test
    for test in &fixtures.tests {
        let result = run_test(test);
        // Store the test name by leaking since we need 'static lifetime
        let name: &'static str = Box::leak(test.name.clone().into_boxed_str());
        results.push((name, result));
    }

    results
}

/// Run a single test fixture
fn run_test(fixture: &TestFixture) -> Result<(), String> {
    // Skip if marked
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("SKIPPED: {}", reason));
    }

    // Route to appropriate test runner based on test name
    if fixture.name.starts_with("spring_") {
        if fixture.name.contains("convergence") {
            run_spring_convergence_test(fixture)
        } else {
            run_spring_test(fixture)
        }
    } else if fixture.name.starts_with("projectile_") {
        if fixture.name.contains("trajectory") {
            run_trajectory_test(fixture)
        } else if fixture.name.contains("zero_gravity") {
            // Zero gravity test has different input structure
            run_zero_gravity_test(fixture)
        } else {
            run_projectile_test(fixture)
        }
    } else if fixture.name.starts_with("fps_") {
        run_fps_test(fixture)
    } else {
        Err(format!("Unknown test type: {}", fixture.name))
    }
}

/// Run zero gravity projectile test
fn run_zero_gravity_test(fixture: &TestFixture) -> Result<(), String> {
    #[derive(Debug, Deserialize)]
    struct ZeroGravityInput {
        acceleration: Vec<f64>,
        start_pos: Vec<f64>,
        start_vel: Vec<f64>,
    }

    let input: ZeroGravityInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: ProjectileOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let delta_time = fps(60);
    let acceleration = Vector::new(input.acceleration[0], input.acceleration[1], input.acceleration[2]);

    let mut projectile = Projectile::new(
        delta_time,
        Point::new(input.start_pos[0], input.start_pos[1], input.start_pos[2]),
        Vector::new(input.start_vel[0], input.start_vel[1], input.start_vel[2]),
        acceleration,
    );

    let new_pos = projectile.update();
    let new_vel = projectile.velocity();

    // Check position
    if !approx_eq(new_pos.x, expected.x, PROJECTILE_EPSILON) {
        return Err(format!(
            "X position mismatch: expected {}, got {}",
            expected.x, new_pos.x
        ));
    }
    if !approx_eq(new_pos.y, expected.y, PROJECTILE_EPSILON) {
        return Err(format!(
            "Y position mismatch: expected {}, got {}",
            expected.y, new_pos.y
        ));
    }
    if !approx_eq(new_pos.z, expected.z, PROJECTILE_EPSILON) {
        return Err(format!(
            "Z position mismatch: expected {}, got {}",
            expected.z, new_pos.z
        ));
    }

    // Check velocity
    if !approx_eq(new_vel.x, expected.vel_x, PROJECTILE_EPSILON) {
        return Err(format!(
            "X velocity mismatch: expected {}, got {}",
            expected.vel_x, new_vel.x
        ));
    }
    if !approx_eq(new_vel.y, expected.vel_y, PROJECTILE_EPSILON) {
        return Err(format!(
            "Y velocity mismatch: expected {}, got {}",
            expected.vel_y, new_vel.y
        ));
    }
    if !approx_eq(new_vel.z, expected.vel_z, PROJECTILE_EPSILON) {
        return Err(format!(
            "Z velocity mismatch: expected {}, got {}",
            expected.vel_z, new_vel.z
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that loads fixtures and runs all conformance tests
    #[test]
    fn test_harmonica_conformance() {
        let results = run_all_tests();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut failures = Vec::new();

        for (name, result) in &results {
            match result {
                Ok(()) => {
                    passed += 1;
                    println!("  PASS: {}", name);
                }
                Err(msg) if msg.starts_with("SKIPPED:") => {
                    skipped += 1;
                    println!("  SKIP: {} - {}", name, msg);
                }
                Err(msg) => {
                    failed += 1;
                    failures.push((name, msg));
                    println!("  FAIL: {} - {}", name, msg);
                }
            }
        }

        println!("\nHarmonica Conformance Results:");
        println!("  Passed:  {}", passed);
        println!("  Failed:  {}", failed);
        println!("  Skipped: {}", skipped);
        println!("  Total:   {}", results.len());

        if !failures.is_empty() {
            println!("\nFailures:");
            for (name, msg) in &failures {
                println!("  {}: {}", name, msg);
            }
            panic!(
                "Harmonica conformance tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
    }

    /// Verify spring physics matches Go within epsilon
    #[test]
    fn test_spring_default_step() {
        let spring = Spring::new(fps(60), 6.0, 1.0);
        let (pos, vel) = spring.update(0.0, 0.0, 1.0);

        // Expected from Go reference
        assert!(approx_eq(pos, 0.004678839798509582, EPSILON), "pos = {}", pos);
        assert!(approx_eq(vel, 0.5429024312770874, EPSILON), "vel = {}", vel);
    }

    /// Verify spring at target position
    #[test]
    fn test_spring_at_target() {
        let spring = Spring::new(fps(60), 6.0, 1.0);
        let (pos, vel) = spring.update(1.0, 0.0, 1.0);

        assert!(approx_eq(pos, 1.0, EPSILON), "pos = {}", pos);
        assert!(approx_eq(vel, 0.0, EPSILON), "vel = {}", vel);
    }

    /// Verify underdamped spring behavior
    #[test]
    fn test_spring_underdamped() {
        let spring = Spring::new(fps(60), 6.0, 0.3);
        let (pos, vel) = spring.update(0.0, 0.0, 1.0);

        // Expected from Go reference
        assert!(approx_eq(pos, 0.0048974149946585666, EPSILON), "pos = {}", pos);
        assert!(approx_eq(vel, 0.5813845939323615, EPSILON), "vel = {}", vel);
    }

    /// Verify overdamped spring behavior
    #[test]
    fn test_spring_overdamped() {
        let spring = Spring::new(fps(60), 6.0, 2.0);
        let (pos, vel) = spring.update(0.0, 0.0, 1.0);

        // Expected from Go reference
        assert!(approx_eq(pos, 0.004391441832238274, EPSILON), "pos = {}", pos);
        assert!(approx_eq(vel, 0.49369831503171113, EPSILON), "vel = {}", vel);
    }

    /// Verify FPS utility function
    #[test]
    fn test_fps_values() {
        assert!(approx_eq(fps(30), 0.033333333, FPS_EPSILON));
        assert!(approx_eq(fps(60), 0.016666666, FPS_EPSILON));
        assert!(approx_eq(fps(120), 0.008333333, FPS_EPSILON));
        assert!(approx_eq(fps(144), 0.006944444, FPS_EPSILON));
        assert!(approx_eq(fps(240), 0.004166666, FPS_EPSILON));
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    /// Spring physics conformance test
    pub struct SpringPhysicsTest {
        name: String,
    }

    impl SpringPhysicsTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for SpringPhysicsTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "harmonica"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, ctx: &mut TestContext) -> TestResult {
            let mut loader = FixtureLoader::new();

            let fixture = match loader.get_test("harmonica", &self.name) {
                Ok(f) => f.clone(),
                Err(e) => {
                    return TestResult::Fail {
                        reason: format!("Failed to load fixture: {}", e),
                    };
                }
            };

            match run_test(&fixture) {
                Ok(()) => TestResult::Pass,
                Err(msg) if msg.starts_with("SKIPPED:") => TestResult::Skipped {
                    reason: msg.replace("SKIPPED: ", ""),
                },
                Err(msg) => TestResult::Fail { reason: msg },
            }
        }
    }

    /// Get all harmonica conformance tests as trait objects
    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("harmonica") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(SpringPhysicsTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
