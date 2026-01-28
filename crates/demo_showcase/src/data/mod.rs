//! Domain model types and data generation for `demo_showcase`.
//!
//! These types represent the data the application displays and manipulates.
//! They are designed to be:
//! - Small and presentation-friendly
//! - Cheaply cloneable
//! - Serializable for persistence/debugging
//!
//! The [`generator`] module provides seedable, deterministic data generation.

#![allow(dead_code)] // Types are used by downstream tasks (generator, pages, actions)

pub mod generator;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier.
    pub id: Id,
    /// Timestamp of the log entry.
    pub timestamp: DateTime<Utc>,
    /// Log level.
    pub level: LogLevel,
    /// Target/module that emitted the log.
    pub target: String,
    /// Log message.
    pub message: String,
    /// Structured fields (key-value pairs).
    pub fields: BTreeMap<String, String>,
    /// Optional span/trace ID.
    pub trace_id: Option<String>,
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
            level,
            target: target.into(),
            message: message.into(),
            fields: BTreeMap::new(),
            trace_id: None,
        }
    }

    /// Add a field to the log entry.
    #[must_use]
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
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
}
