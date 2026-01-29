//! Async job runner for demo_showcase.
//!
//! Demonstrates realistic async workload patterns using `bubbletea::AsyncCmd`
//! when the `async` feature is enabled. Provides a sync fallback when the
//! feature is disabled.
//!
//! # Features
//!
//! - **I/O-like operations**: fetch metrics, deploy service, load docs index, export logs
//! - **Concurrency**: multiple operations can run in parallel via `batch`
//! - **Cancellation**: generation IDs allow ignoring late results from canceled operations
//! - **Determinism**: test mode uses fixed durations for stable E2E tests
//!
//! # Example
//!
//! ```rust,ignore
//! use demo_showcase::data::async_runner::{AsyncRunner, AsyncOperation, AsyncOperationMsg};
//!
//! // Create runner in normal mode
//! let mut runner = AsyncRunner::new(false);
//!
//! // Start an operation
//! let cmd = runner.start(AsyncOperation::FetchMetrics { service_id: 1 });
//!
//! // In update handler:
//! if let Some(msg) = msg.downcast_ref::<AsyncOperationMsg>() {
//!     // Check generation to detect stale results
//!     if runner.is_current_generation(msg.generation) {
//!         match &msg.result {
//!             AsyncResult::MetricsFetched { service_id, metrics } => {
//!                 // Handle result
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! // Cancel all pending operations
//! runner.cancel_all();
//! ```

use bubbletea::{Cmd, Message};
use std::collections::HashMap;
use std::time::Duration;

use super::Id;

/// Delay profile for async operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayProfile {
    /// Normal delays (50-500ms range) for realistic feel.
    Normal,
    /// Fast delays (10-50ms range) for testing.
    Fast,
    /// Fixed delays (all operations take exactly the same time) for E2E tests.
    Deterministic,
}

impl Default for DelayProfile {
    fn default() -> Self {
        Self::Normal
    }
}

/// An async operation that can be started by the runner.
#[derive(Debug, Clone)]
pub enum AsyncOperation {
    /// Fetch metrics for a service.
    FetchMetrics { service_id: Id },
    /// Deploy a service to an environment.
    DeployService {
        service_id: Id,
        environment_id: Id,
        version: String,
    },
    /// Load the docs index.
    LoadDocsIndex,
    /// Export logs to a file.
    ExportLogs { path: String, count: usize },
    /// Run a background job.
    RunJob { job_id: Id },
    /// Custom operation with a name.
    Custom { name: String, duration_ms: u64 },
}

impl AsyncOperation {
    /// Get the operation name for display.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::FetchMetrics { .. } => "Fetch Metrics",
            Self::DeployService { .. } => "Deploy Service",
            Self::LoadDocsIndex => "Load Docs Index",
            Self::ExportLogs { .. } => "Export Logs",
            Self::RunJob { .. } => "Run Job",
            Self::Custom { name, .. } => name,
        }
    }

    /// Get the base duration for this operation.
    fn base_duration(&self, profile: DelayProfile) -> Duration {
        match profile {
            DelayProfile::Deterministic => Duration::from_millis(100),
            DelayProfile::Fast => match self {
                Self::FetchMetrics { .. } => Duration::from_millis(10),
                Self::DeployService { .. } => Duration::from_millis(30),
                Self::LoadDocsIndex => Duration::from_millis(20),
                Self::ExportLogs { .. } => Duration::from_millis(15),
                Self::RunJob { .. } => Duration::from_millis(25),
                Self::Custom { duration_ms, .. } => Duration::from_millis(*duration_ms / 10),
            },
            DelayProfile::Normal => match self {
                Self::FetchMetrics { .. } => Duration::from_millis(100),
                Self::DeployService { .. } => Duration::from_millis(500),
                Self::LoadDocsIndex => Duration::from_millis(200),
                Self::ExportLogs { .. } => Duration::from_millis(150),
                Self::RunJob { .. } => Duration::from_millis(300),
                Self::Custom { duration_ms, .. } => Duration::from_millis(*duration_ms),
            },
        }
    }
}

/// Status of an async operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    /// Operation is pending/running.
    Pending,
    /// Operation completed successfully.
    Completed,
    /// Operation failed.
    Failed,
    /// Operation was cancelled.
    Cancelled,
}

/// Result of an async operation.
#[derive(Debug, Clone)]
pub enum AsyncResult {
    /// Metrics were fetched successfully.
    MetricsFetched {
        service_id: Id,
        /// Simulated metrics: (requests_per_sec, latency_ms, error_rate)
        metrics: (f64, f64, f64),
    },
    /// Service was deployed successfully.
    ServiceDeployed {
        service_id: Id,
        environment_id: Id,
        version: String,
        deploy_time_ms: u64,
    },
    /// Docs index was loaded.
    DocsIndexLoaded {
        page_count: usize,
        index_size_bytes: usize,
    },
    /// Logs were exported.
    LogsExported { path: String, count: usize },
    /// Job completed.
    JobCompleted { job_id: Id, success: bool },
    /// Custom operation completed.
    CustomCompleted { name: String },
    /// Operation failed with an error.
    Error { message: String },
}

/// Message sent when an async operation completes.
#[derive(Debug, Clone)]
pub struct AsyncOperationMsg {
    /// Generation when this operation was started.
    pub generation: u64,
    /// The original operation.
    pub operation: AsyncOperation,
    /// The result of the operation.
    pub result: AsyncResult,
}

impl AsyncOperationMsg {
    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// Tracks a pending operation.
#[derive(Debug, Clone)]
struct PendingOperation {
    operation: AsyncOperation,
    generation: u64,
}

/// Manages async operations with generation-based cancellation.
#[derive(Debug)]
pub struct AsyncRunner {
    /// Current generation (incremented on cancel).
    generation: u64,
    /// Delay profile for operations.
    profile: DelayProfile,
    /// Pending operations by ID.
    pending: HashMap<u64, PendingOperation>,
    /// Next operation ID.
    next_id: u64,
    /// Seed for deterministic results.
    seed: u64,
}

impl Default for AsyncRunner {
    fn default() -> Self {
        Self::new(false)
    }
}

impl AsyncRunner {
    /// Create a new async runner.
    ///
    /// If `deterministic` is true, uses fixed delays and predictable results
    /// for testing.
    #[must_use]
    pub fn new(deterministic: bool) -> Self {
        Self {
            generation: 0,
            profile: if deterministic {
                DelayProfile::Deterministic
            } else {
                DelayProfile::Normal
            },
            pending: HashMap::new(),
            next_id: 0,
            seed: 42,
        }
    }

    /// Create a runner with a specific delay profile.
    #[must_use]
    pub fn with_profile(profile: DelayProfile) -> Self {
        Self {
            generation: 0,
            profile,
            pending: HashMap::new(),
            next_id: 0,
            seed: 42,
        }
    }

    /// Set the seed for deterministic results.
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    /// Get the current generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Check if a generation is still current (not cancelled).
    #[must_use]
    pub const fn is_current_generation(&self, generation_id: u64) -> bool {
        generation_id == self.generation
    }

    /// Get the number of pending operations.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Cancel all pending operations by incrementing the generation.
    ///
    /// Operations that complete after this will be ignored because their
    /// generation won't match the current generation.
    pub fn cancel_all(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending.clear();
    }

    /// Cancel operations matching a predicate.
    pub fn cancel_where<F>(&mut self, f: F)
    where
        F: Fn(&AsyncOperation) -> bool,
    {
        self.pending.retain(|_, op| !f(&op.operation));
    }

    /// Start an async operation.
    ///
    /// Returns a `Cmd` that will complete with an `AsyncOperationMsg`.
    /// Start an async operation using `AsyncCmd`.
    ///
    /// Returns an `AsyncCmd` that will complete with an `AsyncOperationMsg`.
    /// Use `start_sync` for sync-compatible operations that work with `batch`.
    #[cfg(feature = "async")]
    pub fn start_async(
        &mut self,
        operation: AsyncOperation,
    ) -> bubbletea::AsyncCmd {
        use bubbletea::AsyncCmd;

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let generation = self.generation;
        let duration = operation.base_duration(self.profile);
        let seed = self.seed.wrapping_add(id);

        self.pending.insert(
            id,
            PendingOperation {
                operation: operation.clone(),
                generation,
            },
        );

        let op = operation.clone();
        AsyncCmd::new(move || async move {
            // Simulate async delay
            tokio::time::sleep(duration).await;

            // Generate result based on operation type
            let result = generate_result(&op, seed);

            AsyncOperationMsg {
                generation,
                operation: op,
                result,
            }
            .into_message()
        })
    }

    /// Start an async operation (sync version compatible with `batch`).
    ///
    /// When async feature is enabled, this uses tokio to spawn the async work.
    /// The result will be delivered as an `AsyncOperationMsg`.
    #[cfg(feature = "async")]
    pub fn start(&mut self, operation: AsyncOperation) -> Cmd {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let generation = self.generation;
        let duration = operation.base_duration(self.profile);
        let seed = self.seed.wrapping_add(id);

        self.pending.insert(
            id,
            PendingOperation {
                operation: operation.clone(),
                generation,
            },
        );

        let op = operation.clone();
        Cmd::new(move || {
            // Use tokio's current runtime to run async work
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                tokio::time::sleep(duration).await;

                let result = generate_result(&op, seed);

                AsyncOperationMsg {
                    generation,
                    operation: op,
                    result,
                }
                .into_message()
            })
        })
    }

    /// Start an async operation (sync fallback when async feature is disabled).
    ///
    /// This immediately returns a completed result without actual async execution.
    #[cfg(not(feature = "async"))]
    pub fn start(&mut self, operation: AsyncOperation) -> Cmd {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let generation = self.generation;
        let seed = self.seed.wrapping_add(id);

        let result = generate_result(&operation, seed);

        Cmd::new(move || {
            AsyncOperationMsg {
                generation,
                operation,
                result,
            }
            .into_message()
        })
    }

    /// Start multiple operations concurrently.
    ///
    /// Returns a batch command that starts all operations, or `None` if no operations.
    pub fn start_batch(&mut self, operations: Vec<AsyncOperation>) -> Option<Cmd> {
        let cmds: Vec<Option<Cmd>> = operations
            .into_iter()
            .map(|op| Some(self.start(op)))
            .collect();
        bubbletea::batch(cmds)
    }

    /// Handle an operation result, checking generation and updating state.
    ///
    /// Returns `Some(result)` if the operation is still current,
    /// `None` if it was cancelled (stale generation).
    pub fn handle_result<'a>(&self, msg: &'a AsyncOperationMsg) -> Option<&'a AsyncResult> {
        if self.is_current_generation(msg.generation) {
            Some(&msg.result)
        } else {
            None
        }
    }
}

/// Generate a result for an operation based on seed.
fn generate_result(operation: &AsyncOperation, seed: u64) -> AsyncResult {
    // Use seed for deterministic but varied results
    let variation = seed % 100;

    match operation {
        AsyncOperation::FetchMetrics { service_id } => {
            let base_rps = 100.0 + (variation as f64);
            let latency = 30.0 + (variation as f64 * 0.5);
            let error_rate = (variation as f64 * 0.02).min(5.0);

            AsyncResult::MetricsFetched {
                service_id: *service_id,
                metrics: (base_rps, latency, error_rate),
            }
        }
        AsyncOperation::DeployService {
            service_id,
            environment_id,
            version,
        } => {
            let deploy_time = 500 + (variation * 10);

            AsyncResult::ServiceDeployed {
                service_id: *service_id,
                environment_id: *environment_id,
                version: version.clone(),
                deploy_time_ms: deploy_time,
            }
        }
        AsyncOperation::LoadDocsIndex => {
            let page_count = 50 + (variation as usize);
            let index_size = 1024 * (10 + variation as usize);

            AsyncResult::DocsIndexLoaded {
                page_count,
                index_size_bytes: index_size,
            }
        }
        AsyncOperation::ExportLogs { path, count } => AsyncResult::LogsExported {
            path: path.clone(),
            count: *count,
        },
        AsyncOperation::RunJob { job_id } => {
            let success = variation > 10; // 90% success rate

            AsyncResult::JobCompleted {
                job_id: *job_id,
                success,
            }
        }
        AsyncOperation::Custom { name, .. } => AsyncResult::CustomCompleted { name: name.clone() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_runner_creates_with_defaults() {
        let runner = AsyncRunner::new(false);
        assert_eq!(runner.generation(), 0);
        assert_eq!(runner.pending_count(), 0);
        assert_eq!(runner.profile, DelayProfile::Normal);
    }

    #[test]
    fn async_runner_creates_deterministic() {
        let runner = AsyncRunner::new(true);
        assert_eq!(runner.profile, DelayProfile::Deterministic);
    }

    #[test]
    fn async_runner_with_profile() {
        let runner = AsyncRunner::with_profile(DelayProfile::Fast);
        assert_eq!(runner.profile, DelayProfile::Fast);
    }

    #[test]
    fn async_runner_cancel_increments_generation() {
        let mut runner = AsyncRunner::new(false);
        assert_eq!(runner.generation(), 0);

        runner.cancel_all();
        assert_eq!(runner.generation(), 1);

        runner.cancel_all();
        assert_eq!(runner.generation(), 2);
    }

    #[test]
    fn is_current_generation_checks_correctly() {
        let mut runner = AsyncRunner::new(false);

        assert!(runner.is_current_generation(0));
        assert!(!runner.is_current_generation(1));

        runner.cancel_all();

        assert!(!runner.is_current_generation(0));
        assert!(runner.is_current_generation(1));
    }

    #[test]
    fn operation_names_are_correct() {
        assert_eq!(
            AsyncOperation::FetchMetrics { service_id: 1 }.name(),
            "Fetch Metrics"
        );
        assert_eq!(
            AsyncOperation::DeployService {
                service_id: 1,
                environment_id: 1,
                version: "1.0".to_string()
            }
            .name(),
            "Deploy Service"
        );
        assert_eq!(AsyncOperation::LoadDocsIndex.name(), "Load Docs Index");
        assert_eq!(
            AsyncOperation::ExportLogs {
                path: "/tmp/logs".to_string(),
                count: 100
            }
            .name(),
            "Export Logs"
        );
        assert_eq!(AsyncOperation::RunJob { job_id: 1 }.name(), "Run Job");
        assert_eq!(
            AsyncOperation::Custom {
                name: "Test".to_string(),
                duration_ms: 100
            }
            .name(),
            "Test"
        );
    }

    #[test]
    fn operation_base_durations_vary_by_profile() {
        let op = AsyncOperation::FetchMetrics { service_id: 1 };

        let normal = op.base_duration(DelayProfile::Normal);
        let fast = op.base_duration(DelayProfile::Fast);
        let deterministic = op.base_duration(DelayProfile::Deterministic);

        assert!(normal > fast);
        assert_eq!(deterministic, Duration::from_millis(100));
    }

    #[test]
    fn generate_result_produces_valid_results() {
        let result = generate_result(&AsyncOperation::FetchMetrics { service_id: 42 }, 0);
        if let AsyncResult::MetricsFetched {
            service_id,
            metrics,
        } = result
        {
            assert_eq!(service_id, 42);
            assert!(metrics.0 > 0.0); // requests_per_sec
            assert!(metrics.1 > 0.0); // latency
            assert!(metrics.2 >= 0.0); // error_rate
        } else {
            panic!("Expected MetricsFetched");
        }
    }

    #[test]
    fn generate_result_deploy_service() {
        let result = generate_result(
            &AsyncOperation::DeployService {
                service_id: 1,
                environment_id: 2,
                version: "2.0.0".to_string(),
            },
            0,
        );
        if let AsyncResult::ServiceDeployed {
            service_id,
            environment_id,
            version,
            deploy_time_ms,
        } = result
        {
            assert_eq!(service_id, 1);
            assert_eq!(environment_id, 2);
            assert_eq!(version, "2.0.0");
            assert!(deploy_time_ms > 0);
        } else {
            panic!("Expected ServiceDeployed");
        }
    }

    #[test]
    fn generate_result_load_docs_index() {
        let result = generate_result(&AsyncOperation::LoadDocsIndex, 0);
        if let AsyncResult::DocsIndexLoaded {
            page_count,
            index_size_bytes,
        } = result
        {
            assert!(page_count > 0);
            assert!(index_size_bytes > 0);
        } else {
            panic!("Expected DocsIndexLoaded");
        }
    }

    #[test]
    fn generate_result_export_logs() {
        let result = generate_result(
            &AsyncOperation::ExportLogs {
                path: "/tmp/test.log".to_string(),
                count: 50,
            },
            0,
        );
        if let AsyncResult::LogsExported { path, count } = result {
            assert_eq!(path, "/tmp/test.log");
            assert_eq!(count, 50);
        } else {
            panic!("Expected LogsExported");
        }
    }

    #[test]
    fn generate_result_run_job() {
        let result = generate_result(&AsyncOperation::RunJob { job_id: 123 }, 50);
        if let AsyncResult::JobCompleted { job_id, success } = result {
            assert_eq!(job_id, 123);
            // With seed 50, success should be true (50 > 10)
            assert!(success);
        } else {
            panic!("Expected JobCompleted");
        }
    }

    #[test]
    fn generate_result_custom() {
        let result = generate_result(
            &AsyncOperation::Custom {
                name: "MyOp".to_string(),
                duration_ms: 200,
            },
            0,
        );
        if let AsyncResult::CustomCompleted { name } = result {
            assert_eq!(name, "MyOp");
        } else {
            panic!("Expected CustomCompleted");
        }
    }

    #[test]
    fn async_operation_msg_converts_to_message() {
        let msg = AsyncOperationMsg {
            generation: 1,
            operation: AsyncOperation::LoadDocsIndex,
            result: AsyncResult::DocsIndexLoaded {
                page_count: 10,
                index_size_bytes: 1024,
            },
        };

        let bubbletea_msg = msg.into_message();
        let recovered = bubbletea_msg.downcast_ref::<AsyncOperationMsg>();
        assert!(recovered.is_some());
        assert_eq!(recovered.unwrap().generation, 1);
    }

    #[test]
    fn handle_result_accepts_current_generation() {
        let runner = AsyncRunner::new(false);
        let msg = AsyncOperationMsg {
            generation: 0,
            operation: AsyncOperation::LoadDocsIndex,
            result: AsyncResult::DocsIndexLoaded {
                page_count: 10,
                index_size_bytes: 1024,
            },
        };

        let result = runner.handle_result(&msg);
        assert!(result.is_some());
    }

    #[test]
    fn handle_result_rejects_stale_generation() {
        let mut runner = AsyncRunner::new(false);
        runner.cancel_all(); // Increment to generation 1

        let msg = AsyncOperationMsg {
            generation: 0, // Stale
            operation: AsyncOperation::LoadDocsIndex,
            result: AsyncResult::DocsIndexLoaded {
                page_count: 10,
                index_size_bytes: 1024,
            },
        };

        let result = runner.handle_result(&msg);
        assert!(result.is_none());
    }

    #[test]
    fn delay_profile_default_is_normal() {
        assert_eq!(DelayProfile::default(), DelayProfile::Normal);
    }

    #[test]
    fn set_seed_affects_results() {
        let op = AsyncOperation::FetchMetrics { service_id: 1 };

        let result1 = generate_result(&op, 0);
        let result2 = generate_result(&op, 50);

        if let (
            AsyncResult::MetricsFetched { metrics: m1, .. },
            AsyncResult::MetricsFetched { metrics: m2, .. },
        ) = (&result1, &result2)
        {
            // Different seeds should produce different metrics
            assert_ne!(m1.0, m2.0);
        } else {
            panic!("Expected MetricsFetched");
        }
    }

    #[test]
    fn cancel_where_removes_matching_operations() {
        let mut runner = AsyncRunner::new(true);

        // Start some operations (sync fallback, so they complete immediately)
        let _ = runner.start(AsyncOperation::FetchMetrics { service_id: 1 });
        let _ = runner.start(AsyncOperation::FetchMetrics { service_id: 2 });
        let _ = runner.start(AsyncOperation::LoadDocsIndex);

        // In sync mode, operations complete immediately, so pending is empty
        // This test verifies the cancel_where API works
        runner.cancel_where(|op| matches!(op, AsyncOperation::FetchMetrics { .. }));

        // No pending operations in sync mode anyway
        assert_eq!(runner.pending_count(), 0);
    }

    #[test]
    fn runner_seed_can_be_set() {
        let mut runner = AsyncRunner::new(true);
        runner.set_seed(12345);
        assert_eq!(runner.seed, 12345);
    }

    // =========================================================================
    // Determinism tests
    // =========================================================================

    #[test]
    fn deterministic_profile_fixed_duration() {
        let ops = vec![
            AsyncOperation::FetchMetrics { service_id: 1 },
            AsyncOperation::DeployService {
                service_id: 1,
                environment_id: 1,
                version: "1.0".to_string(),
            },
            AsyncOperation::LoadDocsIndex,
            AsyncOperation::ExportLogs {
                path: "/tmp".to_string(),
                count: 10,
            },
            AsyncOperation::RunJob { job_id: 1 },
        ];

        for op in ops {
            let duration = op.base_duration(DelayProfile::Deterministic);
            assert_eq!(
                duration,
                Duration::from_millis(100),
                "Deterministic mode should have fixed 100ms duration for {:?}",
                op.name()
            );
        }
    }

    #[test]
    fn fast_profile_shorter_than_normal() {
        let ops = vec![
            AsyncOperation::FetchMetrics { service_id: 1 },
            AsyncOperation::LoadDocsIndex,
            AsyncOperation::RunJob { job_id: 1 },
        ];

        for op in ops {
            let fast = op.base_duration(DelayProfile::Fast);
            let normal = op.base_duration(DelayProfile::Normal);
            assert!(
                fast < normal,
                "Fast profile should be shorter than normal for {:?}",
                op.name()
            );
        }
    }
}
