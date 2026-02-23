#![allow(clippy::doc_markdown)] // Test harness docs reference crate names freely.
//! Conformance Testing Harness for Charmed Rust
//!
//! This crate provides a unified testing framework for verifying that the Rust
//! implementations of Charm's Go libraries match the original behavior.
//!
//! ## Architecture
//!
//! The harness provides:
//! - **TestLogger**: Hierarchical output with timestamps and indentation
//! - **OutputComparator**: Diff generation for comparing expected vs actual
//! - **BenchContext**: Statistical analysis for performance benchmarks
//! - **TestContext**: Integration layer combining all components
//! - **FixtureLoader**: Test data loading from fixtures/
//! - **ConformanceTest**: Trait for implementing conformance tests
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
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::collapsible_if,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::explicit_iter_loop,
    clippy::format_push_string,
    clippy::if_not_else,
    clippy::inherent_to_string,
    clippy::manual_checked_ops,
    clippy::manual_let_else,
    clippy::manual_midpoint,
    clippy::manual_strip,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::redundant_clone,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::stable_sort_primitive,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::uninlined_format_args,
    clippy::unnecessary_literal_bound,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::use_self,
    clippy::useless_format
)]
pub mod harness;

// Crate-specific conformance tests
#[path = "../crates/mod.rs"]
#[allow(
    clippy::assertions_on_constants,
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::default_constructed_unit_structs,
    clippy::empty_loop,
    clippy::equatable_if_let,
    clippy::explicit_iter_loop,
    clippy::if_same_then_else,
    clippy::io_other_error,
    clippy::items_after_statements,
    clippy::large_stack_frames,
    clippy::len_zero,
    clippy::manual_let_else,
    clippy::manual_strip,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::needless_borrow,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::redundant_closure_for_method_calls,
    clippy::single_char_pattern,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_literal_bound,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::wildcard_imports
)]
pub mod crates;

// Cross-crate integration tests
#[path = "../integration/mod.rs"]
#[allow(
    clippy::pedantic,
    clippy::nursery
)]
pub mod integration;

// Benchmark validation tests - verify benchmarked operations produce correct results
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery)]
mod benchmark_validation;

// Benchmark e2e tests - verify full benchmark workflow
#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery, clippy::useless_vec)]
mod benchmark_e2e;

// Error propagation e2e tests - verify errors work across crate boundaries
#[cfg(test)]
#[allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::io_other_error,
    clippy::needless_question_mark
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
