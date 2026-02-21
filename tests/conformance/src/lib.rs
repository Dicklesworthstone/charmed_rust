//! Conformance Testing Harness for Charmed Rust
//!
//! This crate provides a unified testing framework for verifying that the Rust
//! implementations of Charm's Go libraries match the original behavior.
//!
//! ## Architecture
//!
//! The harness provides:
//! - **`TestLogger`**: Hierarchical output with timestamps and indentation
//! - **`OutputComparator`**: Diff generation for comparing expected vs actual
//! - **`BenchContext`**: Statistical analysis for performance benchmarks
//! - **`TestContext`**: Integration layer combining all components
//! - **`FixtureLoader`**: Test data loading from fixtures/
//! - **`ConformanceTest`**: Trait for implementing conformance tests
//!
//! ## Usage
//!
//! ```rust,ignore
//! use charmed_conformance::harness::{ConformanceTest, TestContext, TestResult};
//!
//! struct MyTest;
//!
//! impl ConformanceTest for MyTest {
//!     fn name(&self) -> &str { "my_test" }
//!     fn crate_name(&self) -> &str { "lipgloss" }
//!     fn category(&self) -> TestCategory { TestCategory::Unit }
//!     fn run(&self, ctx: &mut TestContext) -> TestResult {
//!         // Test implementation
//!         TestResult::Pass
//!     }
//! }
//! ```

#![forbid(unsafe_code)]

#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Harness APIs intentionally prioritize conformance clarity over pedantic style in test-only code"
)]
pub mod harness;

// Crate-specific conformance tests
#[path = "../crates/mod.rs"]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Per-crate conformance adapters mirror fixture semantics and keep verbose test structure"
)]
pub mod crates;

// Cross-crate integration tests
#[path = "../integration/mod.rs"]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Integration conformance flows are intentionally scenario-heavy and fixture-oriented"
)]
pub mod integration;

// Benchmark validation tests - verify benchmarked operations produce correct results
#[cfg(test)]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Benchmark validation tests prefer explicit assertions and fixture-friendly layout"
)]
mod benchmark_validation;

// Benchmark e2e tests - verify full benchmark workflow
#[cfg(test)]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Benchmark E2E tests intentionally preserve realistic benchmark pipeline structure"
)]
mod benchmark_e2e;

// Error propagation e2e tests - verify errors work across crate boundaries
#[cfg(test)]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    reason = "Error propagation E2E tests emphasize behavior parity across crate boundaries"
)]
mod error_e2e;

// Re-export the crates under test for convenience
pub use bubbles;
pub use bubbletea;
pub use charmed_log;
pub use glamour;
pub use harmonica;
pub use huh;
pub use lipgloss;
#[cfg(feature = "wish")]
pub use wish;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::harness::{
        BaselineComparison, BenchBaseline, BenchConfig, BenchContext, BenchResult, CompareOptions,
        CompareResult, ConformanceTest, Diff, DiffType, FixtureError, FixtureLoader,
        FixtureMetadata, FixtureResult, FixtureSet, FixtureStatus, OutlierRemoval,
        OutputComparator, StoredBenchResult, TestCategory, TestContext, TestFixture, TestLogger,
        TestResult, WhitespaceOptions,
    };
}
