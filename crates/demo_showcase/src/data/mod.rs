//! Domain model types and data generation for `demo_showcase`.
//!
//! These types represent the data the application displays and manipulates.
//! They are designed to be:
//! - Small and presentation-friendly
//! - Cheaply cloneable
//! - Serializable for persistence/debugging
//!
//! The [`actions`] module provides the domain action API for state changes.
//! The [`animation`] module provides spring-based animation primitives.
//! The [`generator`] module provides seedable, deterministic data generation.
//! The [`simulation`] module provides a background simulation engine.

#![allow(dead_code)] // Types are used by downstream tasks (generator, pages, actions)

pub mod actions;
pub mod animation;
pub mod generator;
pub mod simulation;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use lipgloss::Style;
use serde::{Deserialize, Serialize};

use crate::theme::Theme;

/// Unique identifier for entities.
pub type Id = u64;

// ============================================================================
// Service Domain
// ============================================================================

/// Health status of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServiceHealth {
    /// Service is operating normally.
    #[default]
    Healthy,
    /// Service is degraded but operational.
    Degraded,
    /// Service is unhealthy/failing.
    Unhealthy,
    /// Health status unknown (no recent checks).
    Unknown,
}

impl ServiceHealth {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Degraded => "Degraded",
            Self::Unhealthy => "Unhealthy",
            Self::Unknown => "Unknown",
        }
    }

    /// Get status icon/indicator.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Healthy => "●",
            Self::Degraded => "◐",
            Self::Unhealthy => "○",
            Self::Unknown => "?",
        }
    }
}

/// Programming language/runtime of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Rust,
    Go,
    Python,
    TypeScript,
    Java,
    Ruby,
    Other,
}

impl Language {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Go => "Go",
            Self::Python => "Python",
            Self::TypeScript => "TypeScript",
            Self::Java => "Java",
            Self::Ruby => "Ruby",
            Self::Other => "Other",
        }
    }
}

/// A service in the platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    /// Unique identifier.
    pub id: Id,
    /// Service name (e.g., "api-gateway", "user-service").
    pub name: String,
    /// Programming language/runtime.
    pub language: Language,
    /// Current health status.
    pub health: ServiceHealth,
    /// Current deployed version.
    pub version: String,
    /// Number of environments this service is deployed to.
    pub environment_count: usize,
    /// Optional description.
    pub description: Option<String>,
}

impl Service {
    /// Create a new service.
    #[must_use]
    pub fn new(id: Id, name: impl Into<String>, language: Language) -> Self {
        Self {
            id,
            name: name.into(),
            language,
            health: ServiceHealth::default(),
            version: "0.0.0".to_string(),
            environment_count: 0,
            description: None,
        }
    }
}

// ============================================================================
// Environment Domain
// ============================================================================

/// Geographic region for deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Region {
    #[default]
    UsEast1,
    UsWest2,
    EuWest1,
    EuCentral1,
    ApSoutheast1,
    ApNortheast1,
}

impl Region {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UsEast1 => "us-east-1",
            Self::UsWest2 => "us-west-2",
            Self::EuWest1 => "eu-west-1",
            Self::EuCentral1 => "eu-central-1",
            Self::ApSoutheast1 => "ap-southeast-1",
            Self::ApNortheast1 => "ap-northeast-1",
        }
    }

    /// Get all regions.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::UsEast1,
            Self::UsWest2,
            Self::EuWest1,
            Self::EuCentral1,
            Self::ApSoutheast1,
            Self::ApNortheast1,
        ]
    }
}

/// An environment where services are deployed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// Unique identifier.
    pub id: Id,
    /// Environment name (e.g., "production", "staging", "dev").
    pub name: String,
    /// Geographic region.
    pub region: Region,
    /// Number of replicas running.
    pub replicas: u32,
    /// Target number of replicas.
    pub target_replicas: u32,
    /// Whether auto-scaling is enabled.
    pub autoscale: bool,
}

impl Environment {
    /// Create a new environment.
    #[must_use]
    pub fn new(id: Id, name: impl Into<String>, region: Region) -> Self {
        Self {
            id,
            name: name.into(),
            region,
            replicas: 1,
            target_replicas: 1,
            autoscale: false,
        }
    }

    /// Check if replicas match target.
    #[must_use]
    pub const fn is_scaled(&self) -> bool {
        self.replicas == self.target_replicas
    }
}

// ============================================================================
// Deployment Domain
// ============================================================================

/// Status of a deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DeploymentStatus {
    /// Deployment is queued/pending.
    #[default]
    Pending,
    /// Deployment is in progress.
    InProgress,
    /// Deployment completed successfully.
    Succeeded,
    /// Deployment failed.
    Failed,
    /// Deployment was rolled back.
    RolledBack,
}

impl DeploymentStatus {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::RolledBack => "Rolled Back",
        }
    }

    /// Get status icon.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::InProgress => "◐",
            Self::Succeeded => "●",
            Self::Failed => "✕",
            Self::RolledBack => "↩",
        }
    }

    /// Check if deployment is terminal (no more state changes expected).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::RolledBack)
    }
}

/// A deployment of a service to an environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployment {
    /// Unique identifier.
    pub id: Id,
    /// Service being deployed.
    pub service_id: Id,
    /// Target environment.
    pub environment_id: Id,
    /// Git commit SHA.
    pub sha: String,
    /// Author who triggered the deployment.
    pub author: String,
    /// Current status.
    pub status: DeploymentStatus,
    /// When the deployment was created.
    pub created_at: DateTime<Utc>,
    /// When the deployment started running.
    pub started_at: Option<DateTime<Utc>>,
    /// When the deployment ended (success or failure).
    pub ended_at: Option<DateTime<Utc>>,
}

impl Deployment {
    /// Create a new pending deployment.
    #[must_use]
    pub fn new(
        id: Id,
        service_id: Id,
        environment_id: Id,
        sha: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id,
            service_id,
            environment_id,
            sha: sha.into(),
            author: author.into(),
            status: DeploymentStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
        }
    }
}

// ============================================================================
// Job Domain
// ============================================================================

/// Kind of background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JobKind {
    /// General background task.
    #[default]
    Task,
    /// Scheduled cron job.
    Cron,
    /// Data migration job.
    Migration,
    /// Backup job.
    Backup,
    /// Build/compile job.
    Build,
    /// Test suite execution.
    Test,
}

impl JobKind {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Task => "Task",
            Self::Cron => "Cron",
            Self::Migration => "Migration",
            Self::Backup => "Backup",
            Self::Build => "Build",
            Self::Test => "Test",
        }
    }

    /// Get kind icon.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Task => "⚙",
            Self::Cron => "⏰",
            Self::Migration => "↗",
            Self::Backup => "💾",
            Self::Build => "🔨",
            Self::Test => "✓",
        }
    }
}

/// Status of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum JobStatus {
    /// Job is queued.
    #[default]
    Queued,
    /// Job is running.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed.
    Failed,
    /// Job was cancelled.
    Cancelled,
}

impl JobStatus {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Get status icon.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Queued => "○",
            Self::Running => "◐",
            Self::Completed => "●",
            Self::Failed => "✕",
            Self::Cancelled => "⊘",
        }
    }

    /// Check if job is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// A background job or task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier.
    pub id: Id,
    /// Job name/title.
    pub name: String,
    /// Kind of job.
    pub kind: JobKind,
    /// Current status.
    pub status: JobStatus,
    /// Progress percentage (0-100).
    pub progress: u8,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
    /// When the job started running.
    pub started_at: Option<DateTime<Utc>>,
    /// When the job ended.
    pub ended_at: Option<DateTime<Utc>>,
    /// Optional error message if failed.
    pub error: Option<String>,
}

impl Job {
    /// Create a new queued job.
    #[must_use]
    pub fn new(id: Id, name: impl Into<String>, kind: JobKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            status: JobStatus::Queued,
            progress: 0,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            error: None,
        }
    }
}

// ============================================================================
// Alert Domain
// ============================================================================

/// Severity level of an alert.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum AlertSeverity {
    /// Informational notice.
    Info,
    /// Warning that may require attention.
    #[default]
    Warning,
    /// Error that needs attention.
    Error,
    /// Critical issue requiring immediate action.
    Critical,
}

impl AlertSeverity {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Critical => "Critical",
        }
    }

    /// Get severity icon.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Warning => "⚠",
            Self::Error => "✕",
            Self::Critical => "‼",
        }
    }
}

/// An alert or notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier.
    pub id: Id,
    /// Severity level.
    pub severity: AlertSeverity,
    /// Alert message.
    pub message: String,
    /// Deduplication key (alerts with same key are grouped).
    pub dedupe_key: String,
    /// When the alert was created.
    pub created_at: DateTime<Utc>,
    /// Optional source (service name, component, etc.).
    pub source: Option<String>,
    /// Whether the alert has been acknowledged.
    pub acknowledged: bool,
}

impl Alert {
    /// Create a new alert.
    #[must_use]
    pub fn new(
        id: Id,
        severity: AlertSeverity,
        message: impl Into<String>,
        dedupe_key: impl Into<String>,
    ) -> Self {
        Self {
            id,
            severity,
            message: message.into(),
            dedupe_key: dedupe_key.into(),
            created_at: Utc::now(),
            source: None,
            acknowledged: false,
        }
    }
}

// ============================================================================
// Log Domain
// ============================================================================

/// Log level.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum LogLevel {
    /// Trace-level debugging.
    Trace,
    /// Debug information.
    Debug,
    /// Informational messages.
    #[default]
    Info,
    /// Warning messages.
    Warn,
    /// Error messages.
    Error,
}

impl LogLevel {
    /// Get display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    /// Get abbreviated name (for compact display).
    #[must_use]
    pub const fn abbrev(self) -> &'static str {
        match self {
            Self::Trace => "TRC",
            Self::Debug => "DBG",
            Self::Info => "INF",
            Self::Warn => "WRN",
            Self::Error => "ERR",
        }
    }
}

/// A structured log entry.
///
/// Log entries support structured fields for filtering and correlation:
/// - `target`: The service/component that emitted the log
/// - `job_id`: Optional correlation with a background job
/// - `deployment_id`: Optional correlation with a deployment
/// - `trace_id`: Optional distributed tracing span ID
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier.
    pub id: Id,
    /// Timestamp of the log entry.
    pub timestamp: DateTime<Utc>,
    /// Monotonic tick (simulation frame) when this entry was created.
    /// Useful for ordering entries and correlating with simulation state.
    pub tick: u64,
    /// Log level.
    pub level: LogLevel,
    /// Target/module that emitted the log (e.g., `api::handlers`, `db::postgres`).
    pub target: String,
    /// Log message.
    pub message: String,
    /// Structured fields (key-value pairs).
    pub fields: BTreeMap<String, String>,
    /// Optional span/trace ID for distributed tracing.
    pub trace_id: Option<String>,
    /// Optional job ID for correlation with background jobs.
    pub job_id: Option<Id>,
    /// Optional deployment ID for correlation with deployments.
    pub deployment_id: Option<Id>,
}

impl LogEntry {
    /// Create a new log entry.
    #[must_use]
    pub fn new(
        id: Id,
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            tick: 0,
            level,
            target: target.into(),
            message: message.into(),
            fields: BTreeMap::new(),
            trace_id: None,
            job_id: None,
            deployment_id: None,
        }
    }

    /// Create a new log entry with a specific tick (simulation frame).
    #[must_use]
    pub const fn with_tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    /// Add a field to the log entry.
    #[must_use]
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Set the job ID for correlation.
    #[must_use]
    pub const fn with_job_id(mut self, job_id: Id) -> Self {
        self.job_id = Some(job_id);
        self
    }

    /// Set the deployment ID for correlation.
    #[must_use]
    pub const fn with_deployment_id(mut self, deployment_id: Id) -> Self {
        self.deployment_id = Some(deployment_id);
        self
    }

    /// Set the trace ID for distributed tracing.
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
}

// ============================================================================
// Log Stream
// ============================================================================

/// A bounded, append-only log stream with filtering capabilities.
///
/// The `LogStream` provides:
/// - Efficient append-only storage with automatic trimming
/// - Filtering by level, target, job, deployment, and text search
/// - Iteration in chronological or reverse order
///
/// # Example
///
/// ```rust,ignore
/// use demo_showcase::data::{LogStream, LogEntry, LogLevel};
///
/// let mut stream = LogStream::new(100);
/// stream.push(LogEntry::new(1, LogLevel::Info, "api", "Request received"));
///
/// // Filter by level
/// let errors: Vec<_> = stream.filter_by_level(LogLevel::Error).collect();
/// ```
#[derive(Debug, Clone)]
pub struct LogStream {
    /// Internal storage (ring buffer behavior via truncation).
    entries: Vec<LogEntry>,
    /// Maximum number of entries to retain.
    max_entries: usize,
    /// Counter for generating entry IDs.
    next_id: Id,
}

impl Default for LogStream {
    fn default() -> Self {
        Self::new(200)
    }
}

impl LogStream {
    /// Create a new log stream with the specified capacity.
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(1000)),
            max_entries,
            next_id: 1,
        }
    }

    /// Push a new log entry to the stream.
    ///
    /// If the stream exceeds `max_entries`, the oldest entries are removed.
    pub fn push(&mut self, entry: LogEntry) {
        self.entries.push(entry);
        self.trim();
    }

    /// Push a new log entry, auto-assigning an ID.
    ///
    /// Returns the assigned ID for correlation purposes.
    pub fn push_new(
        &mut self,
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
    ) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        let entry = LogEntry::new(id, level, target, message);
        self.push(entry);
        id
    }

    /// Push a log entry with tick and optional correlation IDs.
    pub fn push_with_context(
        &mut self,
        tick: u64,
        level: LogLevel,
        target: impl Into<String>,
        message: impl Into<String>,
        job_id: Option<Id>,
        deployment_id: Option<Id>,
    ) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        let mut entry = LogEntry::new(id, level, target, message).with_tick(tick);
        entry.job_id = job_id;
        entry.deployment_id = deployment_id;
        self.push(entry);
        id
    }

    /// Remove oldest entries if over capacity.
    fn trim(&mut self) {
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Get the number of entries in the stream.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the stream is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the maximum capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.max_entries
    }

    /// Set the maximum capacity.
    ///
    /// If the new capacity is smaller than the current size, oldest entries
    /// are removed.
    pub fn set_capacity(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
        self.trim();
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get all entries (oldest first).
    #[must_use]
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Get all entries (newest first).
    pub fn entries_reversed(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().rev()
    }

    /// Filter entries by minimum log level.
    pub fn filter_by_level(&self, min_level: LogLevel) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().filter(move |e| e.level >= min_level)
    }

    /// Filter entries by target (exact match).
    pub fn filter_by_target<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = &'a LogEntry> + 'a {
        self.entries.iter().filter(move |e| e.target == target)
    }

    /// Filter entries by target prefix (e.g., `api::` matches `api::handlers`).
    pub fn filter_by_target_prefix<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = &'a LogEntry> + 'a {
        self.entries
            .iter()
            .filter(move |e| e.target.starts_with(prefix))
    }

    /// Filter entries by job ID.
    pub fn filter_by_job(&self, job_id: Id) -> impl Iterator<Item = &LogEntry> {
        self.entries
            .iter()
            .filter(move |e| e.job_id == Some(job_id))
    }

    /// Filter entries by deployment ID.
    pub fn filter_by_deployment(&self, deployment_id: Id) -> impl Iterator<Item = &LogEntry> {
        self.entries
            .iter()
            .filter(move |e| e.deployment_id == Some(deployment_id))
    }

    /// Search entries by message content (case-insensitive substring match).
    pub fn search<'a>(&'a self, query: &'a str) -> impl Iterator<Item = &'a LogEntry> + 'a {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(move |e| e.message.to_lowercase().contains(&query_lower))
    }

    /// Get entries within a tick range (inclusive).
    pub fn filter_by_tick_range(
        &self,
        start_tick: u64,
        end_tick: u64,
    ) -> impl Iterator<Item = &LogEntry> {
        self.entries
            .iter()
            .filter(move |e| e.tick >= start_tick && e.tick <= end_tick)
    }

    /// Get the latest N entries (newest first).
    pub fn latest(&self, n: usize) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().rev().take(n)
    }

    /// Count entries by level.
    #[must_use]
    pub fn count_by_level(&self) -> LogLevelCounts {
        let mut counts = LogLevelCounts::default();
        for entry in &self.entries {
            match entry.level {
                LogLevel::Trace => counts.trace += 1,
                LogLevel::Debug => counts.debug += 1,
                LogLevel::Info => counts.info += 1,
                LogLevel::Warn => counts.warn += 1,
                LogLevel::Error => counts.error += 1,
            }
        }
        counts
    }
}

/// Counts of log entries by level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LogLevelCounts {
    pub trace: usize,
    pub debug: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
}

impl LogLevelCounts {
    /// Total count across all levels.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.trace + self.debug + self.info + self.warn + self.error
    }
}

// ============================================================================
// Documentation Domain
// ============================================================================

/// A documentation page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocPage {
    /// Unique identifier.
    pub id: Id,
    /// Page title.
    pub title: String,
    /// Page slug/path (e.g., "getting-started", "api/users").
    pub slug: String,
    /// Markdown content.
    pub content: String,
    /// Parent page ID (for hierarchical docs).
    pub parent_id: Option<Id>,
    /// Order within parent (for sorting).
    pub order: u32,
}

impl DocPage {
    /// Create a new documentation page.
    #[must_use]
    pub fn new(
        id: Id,
        title: impl Into<String>,
        slug: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            slug: slug.into(),
            content: content.into(),
            parent_id: None,
            order: 0,
        }
    }
}

// ============================================================================
// Log Formatting (bd-32pp)
// ============================================================================

/// Configuration for log entry column widths.
#[derive(Debug, Clone, Copy)]
pub struct LogColumnWidths {
    /// Width of the timestamp column (e.g., "15:04:05").
    pub timestamp: usize,
    /// Width of the level column (e.g., "ERROR").
    pub level: usize,
    /// Width of the target/component column.
    pub target: usize,
}

impl Default for LogColumnWidths {
    fn default() -> Self {
        Self {
            timestamp: 8, // "15:04:05"
            level: 5,     // "ERROR" (longest)
            target: 20,
        }
    }
}

/// Formats log entries with styled, aligned output.
///
/// The `LogFormatter` provides consistent visual presentation of log entries
/// with color-coded levels and column alignment for easy scanning.
///
/// # Example
///
/// ```rust,ignore
/// use demo_showcase::data::{LogFormatter, LogEntry, LogLevel};
/// use demo_showcase::theme::Theme;
///
/// let theme = Theme::dark();
/// let formatter = LogFormatter::new(&theme);
///
/// let entry = LogEntry::new(1, LogLevel::Error, "api::auth", "Login failed");
/// let styled = formatter.format(&entry);
/// ```
#[derive(Debug, Clone)]
pub struct LogFormatter {
    /// Whether to apply color styling.
    use_color: bool,
    /// Column width configuration.
    widths: LogColumnWidths,
    /// Style for TRACE level.
    trace_style: Style,
    /// Style for DEBUG level.
    debug_style: Style,
    /// Style for INFO level.
    info_style: Style,
    /// Style for WARN level.
    warn_style: Style,
    /// Style for ERROR level.
    error_style: Style,
    /// Style for timestamp.
    timestamp_style: Style,
    /// Style for target/component.
    target_style: Style,
    /// Style for message text.
    message_style: Style,
}

impl LogFormatter {
    /// Create a new log formatter with the given theme.
    #[must_use]
    pub fn new(theme: &Theme) -> Self {
        Self {
            use_color: true,
            widths: LogColumnWidths::default(),
            // TRACE/DEBUG are muted - less important
            trace_style: theme.muted_style(),
            debug_style: theme.muted_style(),
            // INFO is informational but not alarming
            info_style: theme.info_style(),
            // WARN/ERROR pop - need attention
            warn_style: theme.warning_style().bold(),
            error_style: theme.error_style().bold(),
            // Metadata styling
            timestamp_style: theme.muted_style(),
            target_style: theme.muted_style(),
            message_style: Style::new().foreground(theme.text),
        }
    }

    /// Disable color output (for no-color mode).
    #[must_use]
    pub const fn without_color(mut self) -> Self {
        self.use_color = false;
        self
    }

    /// Set custom column widths.
    #[must_use]
    pub const fn with_widths(mut self, widths: LogColumnWidths) -> Self {
        self.widths = widths;
        self
    }

    /// Set the target column width.
    #[must_use]
    pub const fn with_target_width(mut self, width: usize) -> Self {
        self.widths.target = width;
        self
    }

    /// Format a log entry as a styled string.
    ///
    /// Output format: `HH:MM:SS LEVEL  target           message`
    #[must_use]
    pub fn format(&self, entry: &LogEntry) -> String {
        let timestamp = self.format_timestamp(entry);
        let level = self.format_level(entry.level);
        let target = self.format_target(&entry.target);
        let message = self.format_message(&entry.message);

        format!("{timestamp} {level} {target} {message}")
    }

    /// Format a log entry with optional field output.
    ///
    /// If the entry has structured fields, they are appended after the message.
    #[must_use]
    pub fn format_with_fields(&self, entry: &LogEntry) -> String {
        let base = self.format(entry);
        if entry.fields.is_empty() {
            return base;
        }

        let fields: Vec<String> = entry
            .fields
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{base} {{{}}}", fields.join(", "))
    }

    /// Format just the timestamp portion.
    fn format_timestamp(&self, entry: &LogEntry) -> String {
        let ts = entry.timestamp.format("%H:%M:%S").to_string();
        let padded = format!("{:>width$}", ts, width = self.widths.timestamp);

        if self.use_color {
            self.timestamp_style.render(&padded)
        } else {
            padded
        }
    }

    /// Format just the level portion.
    fn format_level(&self, level: LogLevel) -> String {
        let name = level.name();
        let padded = format!("{:width$}", name, width = self.widths.level);

        if self.use_color {
            let style = match level {
                LogLevel::Trace => &self.trace_style,
                LogLevel::Debug => &self.debug_style,
                LogLevel::Info => &self.info_style,
                LogLevel::Warn => &self.warn_style,
                LogLevel::Error => &self.error_style,
            };
            style.render(&padded)
        } else {
            padded
        }
    }

    /// Format just the target portion.
    fn format_target(&self, target: &str) -> String {
        // Truncate if too long
        let truncated = if target.len() > self.widths.target {
            format!("{}…", &target[..self.widths.target.saturating_sub(1)])
        } else {
            target.to_string()
        };
        let padded = format!("{:width$}", truncated, width = self.widths.target);

        if self.use_color {
            self.target_style.render(&padded)
        } else {
            padded
        }
    }

    /// Format just the message portion.
    fn format_message(&self, message: &str) -> String {
        if self.use_color {
            self.message_style.render(message)
        } else {
            message.to_string()
        }
    }

    /// Get the style for a given log level.
    ///
    /// Useful for external code that wants to apply level-based styling.
    #[must_use]
    pub const fn level_style(&self, level: LogLevel) -> &Style {
        match level {
            LogLevel::Trace => &self.trace_style,
            LogLevel::Debug => &self.debug_style,
            LogLevel::Info => &self.info_style,
            LogLevel::Warn => &self.warn_style,
            LogLevel::Error => &self.error_style,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_health_icons() {
        assert_eq!(ServiceHealth::Healthy.icon(), "●");
        assert_eq!(ServiceHealth::Unhealthy.icon(), "○");
    }

    #[test]
    fn deployment_status_terminal() {
        assert!(!DeploymentStatus::Pending.is_terminal());
        assert!(!DeploymentStatus::InProgress.is_terminal());
        assert!(DeploymentStatus::Succeeded.is_terminal());
        assert!(DeploymentStatus::Failed.is_terminal());
    }

    #[test]
    fn job_status_terminal() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
    }

    #[test]
    fn log_entry_with_fields() {
        let entry = LogEntry::new(1, LogLevel::Info, "test", "hello")
            .with_field("user_id", "123")
            .with_field("action", "login");

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("user_id"), Some(&"123".to_string()));
    }

    #[test]
    fn alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Error);
        assert!(AlertSeverity::Error < AlertSeverity::Critical);
    }

    // ========================================================================
    // LogStream tests (bd-33fe)
    // ========================================================================

    #[test]
    fn log_stream_push_and_retrieve() {
        let mut stream = LogStream::new(10);
        assert!(stream.is_empty());

        stream.push(LogEntry::new(1, LogLevel::Info, "api", "hello"));
        assert_eq!(stream.len(), 1);
        assert!(!stream.is_empty());

        stream.push(LogEntry::new(2, LogLevel::Error, "db", "connection failed"));
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn log_stream_auto_trim() {
        let mut stream = LogStream::new(3);

        for i in 1..=5 {
            stream.push(LogEntry::new(i, LogLevel::Info, "test", format!("msg {i}")));
        }

        // Should only keep last 3
        assert_eq!(stream.len(), 3);

        // Oldest entries should be removed
        let entries = stream.entries();
        assert_eq!(entries[0].id, 3);
        assert_eq!(entries[1].id, 4);
        assert_eq!(entries[2].id, 5);
    }

    #[test]
    fn log_stream_filter_by_level() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Debug, "a", "debug"));
        stream.push(LogEntry::new(2, LogLevel::Info, "b", "info"));
        stream.push(LogEntry::new(3, LogLevel::Warn, "c", "warn"));
        stream.push(LogEntry::new(4, LogLevel::Error, "d", "error"));

        let errors: Vec<_> = stream.filter_by_level(LogLevel::Error).collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].id, 4);

        let warn_and_above: Vec<_> = stream.filter_by_level(LogLevel::Warn).collect();
        assert_eq!(warn_and_above.len(), 2);
    }

    #[test]
    fn log_stream_filter_by_target() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "api::handlers", "request"));
        stream.push(LogEntry::new(2, LogLevel::Info, "api::auth", "login"));
        stream.push(LogEntry::new(3, LogLevel::Info, "db::postgres", "query"));

        // Exact match
        let api_handlers: Vec<_> = stream.filter_by_target("api::handlers").collect();
        assert_eq!(api_handlers.len(), 1);

        // Prefix match
        let api_all: Vec<_> = stream.filter_by_target_prefix("api::").collect();
        assert_eq!(api_all.len(), 2);
    }

    #[test]
    fn log_stream_filter_by_job() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "job", "step 1").with_job_id(42));
        stream.push(LogEntry::new(2, LogLevel::Info, "job", "step 2").with_job_id(42));
        stream.push(LogEntry::new(3, LogLevel::Info, "job", "other").with_job_id(99));
        stream.push(LogEntry::new(4, LogLevel::Info, "system", "no job")); // No job_id

        let job_42: Vec<_> = stream.filter_by_job(42).collect();
        assert_eq!(job_42.len(), 2);
    }

    #[test]
    fn log_stream_filter_by_deployment() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "deploy", "starting").with_deployment_id(100));
        stream.push(LogEntry::new(2, LogLevel::Info, "deploy", "finished").with_deployment_id(100));
        stream.push(LogEntry::new(3, LogLevel::Info, "system", "other"));

        let deploy_100: Vec<_> = stream.filter_by_deployment(100).collect();
        assert_eq!(deploy_100.len(), 2);
    }

    #[test]
    fn log_stream_search() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "a", "User logged in"));
        stream.push(LogEntry::new(2, LogLevel::Info, "b", "Request processed"));
        stream.push(LogEntry::new(3, LogLevel::Info, "c", "User logged out"));

        let user_msgs: Vec<_> = stream.search("user").collect();
        assert_eq!(user_msgs.len(), 2);

        // Case insensitive
        let user_msgs_upper: Vec<_> = stream.search("USER").collect();
        assert_eq!(user_msgs_upper.len(), 2);
    }

    #[test]
    fn log_stream_filter_by_tick_range() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "a", "msg").with_tick(10));
        stream.push(LogEntry::new(2, LogLevel::Info, "b", "msg").with_tick(20));
        stream.push(LogEntry::new(3, LogLevel::Info, "c", "msg").with_tick(30));
        stream.push(LogEntry::new(4, LogLevel::Info, "d", "msg").with_tick(40));

        let range: Vec<_> = stream.filter_by_tick_range(15, 35).collect();
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].tick, 20);
        assert_eq!(range[1].tick, 30);
    }

    #[test]
    fn log_stream_latest() {
        let mut stream = LogStream::new(100);
        for i in 1..=10 {
            stream.push(LogEntry::new(i, LogLevel::Info, "test", format!("msg {i}")));
        }

        let latest: Vec<_> = stream.latest(3).collect();
        assert_eq!(latest.len(), 3);
        // Newest first
        assert_eq!(latest[0].id, 10);
        assert_eq!(latest[1].id, 9);
        assert_eq!(latest[2].id, 8);
    }

    #[test]
    fn log_stream_count_by_level() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Debug, "a", ""));
        stream.push(LogEntry::new(2, LogLevel::Info, "b", ""));
        stream.push(LogEntry::new(3, LogLevel::Info, "c", ""));
        stream.push(LogEntry::new(4, LogLevel::Warn, "d", ""));
        stream.push(LogEntry::new(5, LogLevel::Error, "e", ""));
        stream.push(LogEntry::new(6, LogLevel::Error, "f", ""));

        let counts = stream.count_by_level();
        assert_eq!(counts.debug, 1);
        assert_eq!(counts.info, 2);
        assert_eq!(counts.warn, 1);
        assert_eq!(counts.error, 2);
        assert_eq!(counts.total(), 6);
    }

    #[test]
    fn log_stream_push_new() {
        let mut stream = LogStream::new(100);
        let id1 = stream.push_new(LogLevel::Info, "test", "first");
        let id2 = stream.push_new(LogLevel::Warn, "test", "second");

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(stream.len(), 2);
    }

    #[test]
    fn log_stream_push_with_context() {
        let mut stream = LogStream::new(100);
        let id = stream.push_with_context(42, LogLevel::Info, "job", "running", Some(100), None);

        let entry = &stream.entries()[0];
        assert_eq!(entry.id, id);
        assert_eq!(entry.tick, 42);
        assert_eq!(entry.job_id, Some(100));
        assert_eq!(entry.deployment_id, None);
    }

    #[test]
    fn log_stream_clear() {
        let mut stream = LogStream::new(100);
        stream.push(LogEntry::new(1, LogLevel::Info, "test", "msg"));
        assert!(!stream.is_empty());

        stream.clear();
        assert!(stream.is_empty());
    }

    #[test]
    fn log_stream_set_capacity() {
        let mut stream = LogStream::new(10);
        for i in 1..=10 {
            stream.push(LogEntry::new(i, LogLevel::Info, "test", "msg"));
        }
        assert_eq!(stream.len(), 10);

        stream.set_capacity(5);
        assert_eq!(stream.len(), 5);
        assert_eq!(stream.capacity(), 5);
    }

    #[test]
    fn log_entry_correlation_chaining() {
        let entry = LogEntry::new(1, LogLevel::Info, "test", "msg")
            .with_tick(100)
            .with_job_id(42)
            .with_deployment_id(99)
            .with_trace_id("abc123")
            .with_field("key", "value");

        assert_eq!(entry.tick, 100);
        assert_eq!(entry.job_id, Some(42));
        assert_eq!(entry.deployment_id, Some(99));
        assert_eq!(entry.trace_id, Some("abc123".to_string()));
        assert_eq!(entry.fields.get("key"), Some(&"value".to_string()));
    }

    // ========================================================================
    // LogFormatter tests (bd-32pp)
    // ========================================================================

    #[test]
    fn log_formatter_format_basic() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme).without_color();

        let entry = LogEntry::new(1, LogLevel::Info, "api::handlers", "Request received");
        let output = formatter.format(&entry);

        // Should contain all parts (without color codes)
        assert!(output.contains("INFO"));
        assert!(output.contains("api::handlers"));
        assert!(output.contains("Request received"));
    }

    #[test]
    fn log_formatter_format_all_levels() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme).without_color();

        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let entry = LogEntry::new(1, level, "test", "msg");
            let output = formatter.format(&entry);
            assert!(
                output.contains(level.name()),
                "Missing level name for {level:?}"
            );
        }
    }

    #[test]
    fn log_formatter_truncates_long_target() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme)
            .without_color()
            .with_target_width(10);

        let entry = LogEntry::new(1, LogLevel::Info, "very::long::target::name", "msg");
        let output = formatter.format(&entry);

        // Should be truncated with ellipsis
        assert!(output.contains("very::lon…"));
    }

    #[test]
    fn log_formatter_with_fields() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme).without_color();

        let entry = LogEntry::new(1, LogLevel::Info, "api", "Request")
            .with_field("user_id", "123")
            .with_field("method", "GET");
        let output = formatter.format_with_fields(&entry);

        // Should contain the fields
        assert!(output.contains("user_id=123"));
        assert!(output.contains("method=GET"));
        assert!(output.contains('{') && output.contains('}'));
    }

    #[test]
    fn log_formatter_without_fields_no_braces() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme).without_color();

        let entry = LogEntry::new(1, LogLevel::Info, "api", "Request");
        let output = formatter.format_with_fields(&entry);

        // Should not contain braces when no fields
        assert!(!output.contains('{'));
        assert!(!output.contains('}'));
    }

    #[test]
    fn log_formatter_level_style() {
        use crate::theme::Theme;

        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme);

        // Just verify we get valid styles for each level
        let _ = formatter.level_style(LogLevel::Trace);
        let _ = formatter.level_style(LogLevel::Debug);
        let _ = formatter.level_style(LogLevel::Info);
        let _ = formatter.level_style(LogLevel::Warn);
        let _ = formatter.level_style(LogLevel::Error);
    }

    #[test]
    fn log_formatter_custom_widths() {
        use crate::theme::Theme;

        let widths = LogColumnWidths {
            timestamp: 10,
            level: 6,
            target: 15,
        };
        let theme = Theme::dark();
        let formatter = LogFormatter::new(&theme)
            .without_color()
            .with_widths(widths);

        let entry = LogEntry::new(1, LogLevel::Info, "test", "msg");
        let _ = formatter.format(&entry); // Just verify it doesn't panic
    }

    #[test]
    fn log_column_widths_default() {
        let widths = LogColumnWidths::default();
        assert_eq!(widths.timestamp, 8);
        assert_eq!(widths.level, 5);
        assert_eq!(widths.target, 20);
    }
}
