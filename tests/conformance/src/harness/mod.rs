//! Conformance Testing Harness
//!
//! This module provides the core infrastructure for conformance testing:
//!
//! - [`TestLogger`]: Hierarchical logging with timestamps
//! - [`OutputComparator`]: Diff generation for comparing outputs
//! - [`BenchContext`]: Statistical benchmarking framework
//! - [`TestContext`]: Integration layer for running tests
//! - [`FixtureLoader`]: Loading test fixtures and expected outputs
//! - [`ConformanceTest`]: Trait for implementing conformance tests
//! - Test runner for executing test suites

mod benchmark;
mod comparison;
mod context;
mod fixtures;
mod logging;
mod runner;
mod traits;

pub use benchmark::{
    BaselineComparison, BenchBaseline, BenchConfig, BenchContext, BenchResult, OutlierRemoval,
    StoredBenchResult,
};
pub use comparison::{
    CompareOptions, CompareResult, Diff, DiffType, OutputComparator, SemanticCompareResult,
    StyledSpan, WhitespaceOptions, compare_styled_semantic, extract_styled_spans, strip_ansi,
};
pub use context::TestContext;
pub use fixtures::{
    FixtureError, FixtureLoader, FixtureMetadata, FixtureResult, FixtureSet, FixtureStatus,
    TestFixture,
};
pub use logging::{LogLevel, TestLogger};
pub use runner::{ReportConfig, ReportGenerator, TestRunResult, TestRunner, TestSummary};
pub use traits::{ConformanceTest, TestCategory, TestResult};
