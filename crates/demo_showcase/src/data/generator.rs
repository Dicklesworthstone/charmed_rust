//! Deterministic data generator for `demo_showcase`.
//!
//! Provides seedable generation of realistic demo data.
//! Two runs with the same seed produce identical datasets.

// Template strings use {placeholder} syntax, not format! syntax
#![allow(clippy::literal_string_with_formatting_args)]

use std::collections::BTreeMap;

use chrono::{DateTime, TimeDelta, Utc};
use rand::Rng;
use rand::prelude::IndexedRandom;
use rand_pcg::Pcg64;

use super::{
    Alert, AlertSeverity, Deployment, DeploymentStatus, DocPage, Environment, Id, Job, JobKind,
    JobStatus, Language, LogEntry, LogLevel, Region, Service, ServiceHealth,
};

// ============================================================================
// Static Data Pools
// ============================================================================

/// Service name prefixes (microservice style).
const SERVICE_PREFIXES: &[&str] = &[
    "api",
    "auth",
    "billing",
    "cache",
    "config",
    "data",
    "email",
    "event",
    "file",
    "gateway",
    "identity",
    "inventory",
    "job",
    "kafka",
    "log",
    "media",
    "notification",
    "order",
    "payment",
    "queue",
    "redis",
    "search",
    "telemetry",
    "user",
    "web",
    "worker",
];

/// Service name suffixes.
const SERVICE_SUFFIXES: &[&str] = &[
    "service",
    "svc",
    "worker",
    "handler",
    "processor",
    "gateway",
    "proxy",
    "server",
    "api",
    "engine",
    "manager",
];

/// Environment names.
const ENV_NAMES: &[&str] = &[
    "production",
    "staging",
    "development",
    "qa",
    "sandbox",
    "canary",
    "preview",
    "demo",
];

/// Author names for deployments.
const AUTHORS: &[&str] = &[
    "alice", "bob", "carol", "david", "eve", "frank", "grace", "henry", "iris", "jack", "kate",
    "leo", "mia", "noah", "olivia", "peter", "quinn", "ruby", "sam", "tara",
];

/// Job name templates.
const JOB_TEMPLATES: &[&str] = &[
    "database-backup-{n}",
    "log-rotation-{n}",
    "cache-warmup-{n}",
    "index-rebuild-{n}",
    "data-sync-{n}",
    "metrics-aggregate-{n}",
    "report-generation-{n}",
    "cleanup-expired-{n}",
    "schema-migration-{n}",
    "health-check-{n}",
    "certificate-renewal-{n}",
    "security-scan-{n}",
    "dependency-update-{n}",
    "image-build-{n}",
    "test-suite-{n}",
];

/// Alert message templates.
const ALERT_MESSAGES: &[&str] = &[
    "High CPU usage on {service}",
    "Memory threshold exceeded on {service}",
    "Disk space low on {service}",
    "Connection pool exhausted in {service}",
    "Error rate spike detected in {service}",
    "Latency increase on {service} endpoints",
    "Certificate expiring soon for {service}",
    "Rate limiting triggered on {service}",
    "Database connection timeout in {service}",
    "Queue depth critical on {service}",
    "Health check failing for {service}",
    "Deployment rollback triggered for {service}",
];

/// Log message templates.
const LOG_MESSAGES: &[&str] = &[
    "Request processed successfully",
    "Connection established to database",
    "Cache hit for key",
    "Retry attempt {n} for operation",
    "Background job completed",
    "Configuration reloaded",
    "Metric published to aggregator",
    "Session created for user",
    "Rate limit check passed",
    "Webhook delivered successfully",
    "File uploaded to storage",
    "Event published to queue",
    "Health check passed",
    "Token validated successfully",
    "Query executed in {n}ms",
];

/// Log targets/modules.
const LOG_TARGETS: &[&str] = &[
    "api::handlers",
    "api::middleware",
    "auth::jwt",
    "auth::session",
    "cache::redis",
    "db::postgres",
    "queue::kafka",
    "storage::s3",
    "metrics::prometheus",
    "tracing::span",
    "http::client",
    "http::server",
];

/// Documentation page titles.
const DOC_TITLES: &[&str] = &[
    "Getting Started",
    "Installation",
    "Configuration",
    "Authentication",
    "Authorization",
    "API Reference",
    "Architecture Overview",
    "Deployment Guide",
    "Troubleshooting",
    "FAQ",
    "Changelog",
    "Migration Guide",
    "Security Best Practices",
    "Performance Tuning",
    "Monitoring Setup",
];

// ============================================================================
// Generator
// ============================================================================

/// Deterministic data generator.
///
/// Uses a seeded PRNG to produce reproducible datasets.
pub struct Generator {
    rng: Pcg64,
    base_time: DateTime<Utc>,
    next_id: Id,
}

impl Generator {
    /// Create a new generator with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64::new(seed.into(), 0x0a02_bdbf_7bb3_c0a7),
            base_time: DateTime::from_timestamp(1_700_000_000, 0).unwrap_or_else(Utc::now),
            next_id: 1,
        }
    }

    /// Get the next unique ID.
    #[allow(clippy::missing_const_for_fn)] // Cannot be const: mutates self
    fn next_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Generate a random timestamp within the last N hours from base time.
    fn random_time(&mut self, hours_ago_max: i64) -> DateTime<Utc> {
        let secs = self.rng.random_range(0..hours_ago_max * 3600);
        self.base_time - TimeDelta::seconds(secs)
    }

    /// Generate a random git SHA-like string.
    fn random_sha(&mut self) -> String {
        let chars: Vec<char> = "0123456789abcdef".chars().collect();
        (0..7)
            .map(|_| *chars.choose(&mut self.rng).unwrap_or(&'0'))
            .collect()
    }

    /// Generate a random semver version string.
    fn random_version(&mut self) -> String {
        let major = self.rng.random_range(0_u8..3);
        let minor = self.rng.random_range(0_u8..20);
        let patch = self.rng.random_range(0_u8..50);
        format!("{major}.{minor}.{patch}")
    }

    // ========================================================================
    // Domain Generators
    // ========================================================================

    /// Generate a single service.
    #[must_use]
    pub fn service(&mut self) -> Service {
        let prefix = SERVICE_PREFIXES.choose(&mut self.rng).unwrap_or(&"app");
        let suffix = SERVICE_SUFFIXES.choose(&mut self.rng).unwrap_or(&"service");
        let name = format!("{prefix}-{suffix}");

        let languages = [
            Language::Rust,
            Language::Go,
            Language::Python,
            Language::TypeScript,
            Language::Java,
            Language::Ruby,
        ];
        let language = *languages.choose(&mut self.rng).unwrap_or(&Language::Rust);

        let healths = [
            (ServiceHealth::Healthy, 70),
            (ServiceHealth::Degraded, 15),
            (ServiceHealth::Unhealthy, 10),
            (ServiceHealth::Unknown, 5),
        ];
        let health = self.weighted_choice(&healths);

        Service {
            id: self.next_id(),
            name,
            language,
            health,
            version: self.random_version(),
            environment_count: self.rng.random_range(1..5),
            description: if self.rng.random_bool(0.3) {
                Some(format!("Handles {prefix} operations"))
            } else {
                None
            },
        }
    }

    /// Generate N services.
    #[must_use]
    pub fn services(&mut self, count: usize) -> Vec<Service> {
        (0..count).map(|_| self.service()).collect()
    }

    /// Generate a single environment.
    #[must_use]
    pub fn environment(&mut self) -> Environment {
        let name = ENV_NAMES.choose(&mut self.rng).unwrap_or(&"default");
        let regions = Region::all();
        let region = *regions.choose(&mut self.rng).unwrap_or(&Region::UsEast1);

        let target = self.rng.random_range(1_u32..10);
        let replicas = if self.rng.random_bool(0.8) {
            target
        } else {
            self.rng.random_range(0..target)
        };

        Environment {
            id: self.next_id(),
            name: (*name).to_string(),
            region,
            replicas,
            target_replicas: target,
            autoscale: self.rng.random_bool(0.4),
        }
    }

    /// Generate N environments.
    #[must_use]
    pub fn environments(&mut self, count: usize) -> Vec<Environment> {
        (0..count).map(|_| self.environment()).collect()
    }

    /// Generate a deployment for a service/environment pair.
    #[must_use]
    pub fn deployment(&mut self, service_id: Id, environment_id: Id) -> Deployment {
        let author = AUTHORS.choose(&mut self.rng).unwrap_or(&"unknown");
        let created_at = self.random_time(72);

        let statuses = [
            (DeploymentStatus::Succeeded, 60),
            (DeploymentStatus::InProgress, 15),
            (DeploymentStatus::Pending, 10),
            (DeploymentStatus::Failed, 10),
            (DeploymentStatus::RolledBack, 5),
        ];
        let status = self.weighted_choice(&statuses);

        let started_at = if status == DeploymentStatus::Pending {
            None
        } else {
            Some(created_at + TimeDelta::seconds(self.rng.random_range(5..60)))
        };

        let ended_at = if status.is_terminal() {
            started_at.map(|s| s + TimeDelta::seconds(self.rng.random_range(30..300)))
        } else {
            None
        };

        Deployment {
            id: self.next_id(),
            service_id,
            environment_id,
            sha: self.random_sha(),
            author: (*author).to_string(),
            status,
            created_at,
            started_at,
            ended_at,
        }
    }

    /// Generate N deployments for given service/environment IDs.
    #[must_use]
    pub fn deployments(
        &mut self,
        count: usize,
        service_ids: &[Id],
        environment_ids: &[Id],
    ) -> Vec<Deployment> {
        (0..count)
            .map(|_| {
                let sid = service_ids.choose(&mut self.rng).copied().unwrap_or(1);
                let eid = environment_ids.choose(&mut self.rng).copied().unwrap_or(1);
                self.deployment(sid, eid)
            })
            .collect()
    }

    /// Generate a single job.
    #[must_use]
    pub fn job(&mut self) -> Job {
        let template = JOB_TEMPLATES.choose(&mut self.rng).unwrap_or(&"task-{n}");
        let n = self.rng.random_range(1000_u32..9999);
        let name = template.replace("{n}", &n.to_string());

        let kinds = [
            (JobKind::Task, 25),
            (JobKind::Cron, 20),
            (JobKind::Backup, 15),
            (JobKind::Build, 15),
            (JobKind::Test, 15),
            (JobKind::Migration, 10),
        ];
        let kind = self.weighted_choice(&kinds);

        let statuses = [
            (JobStatus::Completed, 50),
            (JobStatus::Running, 20),
            (JobStatus::Queued, 15),
            (JobStatus::Failed, 10),
            (JobStatus::Cancelled, 5),
        ];
        let status = self.weighted_choice(&statuses);

        let created_at = self.random_time(24);
        let started_at = if status == JobStatus::Queued {
            None
        } else {
            Some(created_at + TimeDelta::seconds(self.rng.random_range(1..30)))
        };
        let ended_at = if status.is_terminal() {
            started_at.map(|s| s + TimeDelta::seconds(self.rng.random_range(10..600)))
        } else {
            None
        };

        let progress = match status {
            JobStatus::Completed => 100,
            JobStatus::Failed | JobStatus::Cancelled => self.rng.random_range(10..90),
            JobStatus::Running => self.rng.random_range(1..99),
            JobStatus::Queued => 0,
        };

        let error = if status == JobStatus::Failed {
            Some("Process exited with non-zero status".to_string())
        } else {
            None
        };

        Job {
            id: self.next_id(),
            name,
            kind,
            status,
            progress,
            created_at,
            started_at,
            ended_at,
            error,
        }
    }

    /// Generate N jobs.
    #[must_use]
    pub fn jobs(&mut self, count: usize) -> Vec<Job> {
        (0..count).map(|_| self.job()).collect()
    }

    /// Generate a single alert.
    #[must_use]
    pub fn alert(&mut self, service_name: Option<&str>) -> Alert {
        let service = service_name.unwrap_or("unknown-service");
        let template = ALERT_MESSAGES.choose(&mut self.rng).unwrap_or(&"Alert");
        let message = template.replace("{service}", service);

        let severities = [
            (AlertSeverity::Info, 20),
            (AlertSeverity::Warning, 40),
            (AlertSeverity::Error, 30),
            (AlertSeverity::Critical, 10),
        ];
        let severity = self.weighted_choice(&severities);

        let dedupe_key = format!(
            "{}-{}-{}",
            service,
            severity.name().to_lowercase(),
            self.rng.random_range(1_u16..100)
        );

        Alert {
            id: self.next_id(),
            severity,
            message,
            dedupe_key,
            created_at: self.random_time(12),
            source: Some(service.to_string()),
            acknowledged: self.rng.random_bool(0.2),
        }
    }

    /// Generate N alerts for the given services.
    #[must_use]
    pub fn alerts(&mut self, count: usize, service_names: &[&str]) -> Vec<Alert> {
        (0..count)
            .map(|_| {
                let svc = service_names.choose(&mut self.rng).copied();
                self.alert(svc)
            })
            .collect()
    }

    /// Generate a single log entry.
    #[must_use]
    pub fn log_entry(&mut self) -> LogEntry {
        let levels = [
            (LogLevel::Trace, 5),
            (LogLevel::Debug, 15),
            (LogLevel::Info, 50),
            (LogLevel::Warn, 20),
            (LogLevel::Error, 10),
        ];
        let level = self.weighted_choice(&levels);

        let target = LOG_TARGETS.choose(&mut self.rng).unwrap_or(&"app");
        let template = LOG_MESSAGES.choose(&mut self.rng).unwrap_or(&"Log message");
        let n = self.rng.random_range(1_u32..500);
        let message = template.replace("{n}", &n.to_string());

        let mut fields = BTreeMap::new();
        if self.rng.random_bool(0.3) {
            fields.insert("request_id".to_string(), self.random_sha());
        }
        if self.rng.random_bool(0.2) {
            fields.insert(
                "duration_ms".to_string(),
                self.rng.random_range(1_u32..500).to_string(),
            );
        }
        if self.rng.random_bool(0.15) {
            fields.insert(
                "user_id".to_string(),
                self.rng.random_range(1000_u32..9999).to_string(),
            );
        }

        let trace_id = if self.rng.random_bool(0.4) {
            Some(format!("{:016x}", self.rng.random::<u64>()))
        } else {
            None
        };

        LogEntry {
            id: self.next_id(),
            timestamp: self.random_time(6),
            level,
            target: (*target).to_string(),
            message,
            fields,
            trace_id,
        }
    }

    /// Generate N log entries.
    #[must_use]
    pub fn log_entries(&mut self, count: usize) -> Vec<LogEntry> {
        (0..count).map(|_| self.log_entry()).collect()
    }

    /// Generate documentation pages.
    #[must_use]
    pub fn doc_pages(&mut self, count: usize) -> Vec<DocPage> {
        let titles: Vec<&str> = DOC_TITLES
            .iter()
            .take(count.min(DOC_TITLES.len()))
            .copied()
            .collect();

        titles
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let slug = title.to_lowercase().replace(' ', "-");
                let content = format!(
                    "# {title}\n\nThis is the documentation for {title}.\n\n## Overview\n\nContent goes here."
                );
                DocPage {
                    id: self.next_id(),
                    title: (*title).to_string(),
                    slug,
                    content,
                    parent_id: None,
                    order: u32::try_from(i).unwrap_or(0),
                }
            })
            .collect()
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Choose an item based on weights.
    ///
    /// # Panics
    /// Panics if `items` is empty.
    fn weighted_choice<T: Copy>(&mut self, items: &[(T, u32)]) -> T {
        debug_assert!(
            !items.is_empty(),
            "weighted_choice requires non-empty items"
        );
        let total: u32 = items.iter().map(|(_, w)| w).sum();
        let mut roll = self.rng.random_range(0..total);

        for (item, weight) in items {
            if roll < *weight {
                return *item;
            }
            roll -= weight;
        }

        // Fallback (should not happen with correct weights)
        items[0].0
    }
}

// ============================================================================
// Generated Dataset
// ============================================================================

/// A complete generated dataset for the application.
#[derive(Debug, Clone)]
pub struct GeneratedData {
    /// Generated services.
    pub services: Vec<Service>,
    /// Generated environments.
    pub environments: Vec<Environment>,
    /// Generated deployments.
    pub deployments: Vec<Deployment>,
    /// Generated jobs.
    pub jobs: Vec<Job>,
    /// Generated alerts.
    pub alerts: Vec<Alert>,
    /// Generated log entries.
    pub log_entries: Vec<LogEntry>,
    /// Generated documentation pages.
    pub doc_pages: Vec<DocPage>,
}

impl GeneratedData {
    /// Generate a complete dataset with the given seed.
    #[must_use]
    pub fn generate(seed: u64) -> Self {
        let mut g = Generator::new(seed);

        let services = g.services(12);
        let environments = g.environments(8);

        let service_ids: Vec<Id> = services.iter().map(|s| s.id).collect();
        let env_ids: Vec<Id> = environments.iter().map(|e| e.id).collect();
        let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();

        let deployments = g.deployments(25, &service_ids, &env_ids);
        let jobs = g.jobs(20);
        let alerts = g.alerts(10, &service_names);
        let log_entries = g.log_entries(100);
        let doc_pages = g.doc_pages(10);

        Self {
            services,
            environments,
            deployments,
            jobs,
            alerts,
            log_entries,
            doc_pages,
        }
    }

    /// Generate a minimal dataset for testing.
    #[must_use]
    pub fn generate_minimal(seed: u64) -> Self {
        let mut g = Generator::new(seed);

        let services = g.services(3);
        let environments = g.environments(2);

        let service_ids: Vec<Id> = services.iter().map(|s| s.id).collect();
        let env_ids: Vec<Id> = environments.iter().map(|e| e.id).collect();
        let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();

        let deployments = g.deployments(5, &service_ids, &env_ids);
        let jobs = g.jobs(5);
        let alerts = g.alerts(3, &service_names);
        let log_entries = g.log_entries(10);
        let doc_pages = g.doc_pages(3);

        Self {
            services,
            environments,
            deployments,
            jobs,
            alerts,
            log_entries,
            doc_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_generation() {
        let seed = 12345;
        let data1 = GeneratedData::generate(seed);
        let data2 = GeneratedData::generate(seed);

        assert_eq!(data1.services.len(), data2.services.len());
        for (s1, s2) in data1.services.iter().zip(data2.services.iter()) {
            assert_eq!(s1.id, s2.id);
            assert_eq!(s1.name, s2.name);
            assert_eq!(s1.language, s2.language);
            assert_eq!(s1.health, s2.health);
        }

        assert_eq!(data1.jobs.len(), data2.jobs.len());
        for (j1, j2) in data1.jobs.iter().zip(data2.jobs.iter()) {
            assert_eq!(j1.id, j2.id);
            assert_eq!(j1.name, j2.name);
            assert_eq!(j1.kind, j2.kind);
        }
    }

    #[test]
    fn different_seeds_produce_different_data() {
        let data1 = GeneratedData::generate(1);
        let data2 = GeneratedData::generate(2);

        // At least some services should differ
        let names1: Vec<_> = data1.services.iter().map(|s| &s.name).collect();
        let names2: Vec<_> = data2.services.iter().map(|s| &s.name).collect();
        assert_ne!(names1, names2);
    }

    #[test]
    fn generator_produces_expected_counts() {
        let data = GeneratedData::generate(42);

        assert_eq!(data.services.len(), 12);
        assert_eq!(data.environments.len(), 8);
        assert_eq!(data.deployments.len(), 25);
        assert_eq!(data.jobs.len(), 20);
        assert_eq!(data.alerts.len(), 10);
        assert_eq!(data.log_entries.len(), 100);
        assert_eq!(data.doc_pages.len(), 10);
    }

    #[test]
    fn minimal_dataset() {
        let data = GeneratedData::generate_minimal(99);

        assert_eq!(data.services.len(), 3);
        assert_eq!(data.environments.len(), 2);
        assert_eq!(data.deployments.len(), 5);
    }

    #[test]
    fn ids_are_unique() {
        let data = GeneratedData::generate(777);

        let mut all_ids: Vec<Id> = Vec::new();
        all_ids.extend(data.services.iter().map(|s| s.id));
        all_ids.extend(data.environments.iter().map(|e| e.id));
        all_ids.extend(data.deployments.iter().map(|d| d.id));
        all_ids.extend(data.jobs.iter().map(|j| j.id));
        all_ids.extend(data.alerts.iter().map(|a| a.id));
        all_ids.extend(data.log_entries.iter().map(|l| l.id));
        all_ids.extend(data.doc_pages.iter().map(|d| d.id));

        let count = all_ids.len();
        all_ids.sort_unstable();
        all_ids.dedup();
        assert_eq!(all_ids.len(), count, "IDs should be unique");
    }

    #[test]
    fn deployments_reference_valid_ids() {
        let data = GeneratedData::generate(888);

        let service_ids: std::collections::HashSet<_> =
            data.services.iter().map(|s| s.id).collect();
        let env_ids: std::collections::HashSet<_> =
            data.environments.iter().map(|e| e.id).collect();

        for deployment in &data.deployments {
            assert!(
                service_ids.contains(&deployment.service_id),
                "Deployment references invalid service"
            );
            assert!(
                env_ids.contains(&deployment.environment_id),
                "Deployment references invalid environment"
            );
        }
    }

    #[test]
    fn job_progress_matches_status() {
        let mut g = Generator::new(999);
        for _ in 0..50 {
            let job = g.job();
            match job.status {
                JobStatus::Completed => assert_eq!(job.progress, 100),
                JobStatus::Queued => assert_eq!(job.progress, 0),
                _ => {}
            }
        }
    }
}
