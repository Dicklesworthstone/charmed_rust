//! ConformanceTest - Trait for implementing conformance tests
//!
//! Defines the interface that all conformance tests must implement.
//!
//! TODO: Full implementation pending (charmed_rust-5x5.1.6)

use super::benchmark::{BenchContext, BenchResult};
use super::context::TestContext;

/// Category of conformance test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    /// Unit test - single function or behavior
    Unit,
    /// Integration test - component interactions
    Integration,
    /// Edge case - boundary conditions, error handling
    EdgeCase,
    /// Performance test - benchmarking
    Performance,
}

impl TestCategory {
    /// Get the string name of the category
    pub fn as_str(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit",
            TestCategory::Integration => "integration",
            TestCategory::EdgeCase => "edge_case",
            TestCategory::Performance => "performance",
        }
    }
}

/// Result of a conformance test
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    /// Test passed
    Pass,
    /// Test failed with a reason
    Fail { reason: String },
    /// Test was skipped with a reason
    Skipped { reason: String },
}

impl TestResult {
    /// Returns true if the test passed
    pub fn is_pass(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    /// Returns true if the test failed
    pub fn is_fail(&self) -> bool {
        matches!(self, TestResult::Fail { .. })
    }

    /// Returns true if the test was skipped
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skipped { .. })
    }
}

/// Trait for conformance tests
///
/// Each conformance test implements this trait to integrate with
/// the test runner and logging infrastructure.
///
/// # Example
///
/// ```rust,ignore
/// use charmed_conformance::harness::{ConformanceTest, TestContext, TestResult, TestCategory};
///
/// struct SpringPhysicsTest;
///
/// impl ConformanceTest for SpringPhysicsTest {
///     fn name(&self) -> &str {
///         "spring_physics_basic"
///     }
///
///     fn crate_name(&self) -> &str {
///         "harmonica"
///     }
///
///     fn category(&self) -> TestCategory {
///         TestCategory::Unit
///     }
///
///     fn run(&self, ctx: &mut TestContext) -> TestResult {
///         let spring = harmonica::Spring::default();
///         ctx.log_input("spring", &spring);
///
///         let (pos, vel) = spring.update(0.0, 1.0, 1.0);
///         ctx.log_actual("position", &pos);
///         ctx.log_actual("velocity", &vel);
///
///         // Compare with expected Go behavior
///         ctx.assert_f64_eq(0.5, pos, 0.001);
///
///         ctx.result()
///     }
/// }
/// ```
pub trait ConformanceTest: Send + Sync {
    /// Human-readable name of the test
    fn name(&self) -> &str;

    /// Which crate this test verifies
    fn crate_name(&self) -> &str;

    /// Category of the test (unit, integration, edge_case, performance)
    fn category(&self) -> TestCategory;

    /// Execute the test with the given context
    fn run(&self, ctx: &mut TestContext) -> TestResult;

    /// Optional benchmark variant of the test
    ///
    /// Override this to provide performance measurements.
    fn benchmark(&self, _ctx: &mut BenchContext) -> Option<BenchResult> {
        None
    }

    /// Full test ID in the format "crate::name"
    fn id(&self) -> String {
        format!("{}::{}", self.crate_name(), self.name())
    }
}
