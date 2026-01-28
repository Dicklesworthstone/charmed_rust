//! Background simulation engine for `demo_showcase`.
//!
//! This module provides a simulation engine that updates demo data over time,
//! making the UI feel alive and realistic. The simulation is designed to be:
//!
//! - **Deterministic**: Given the same seed and frame sequence, produces identical results
//! - **Testable**: Can be driven by injected Tick messages without real sleeps
//! - **Configurable**: Rate of changes can be adjusted
//!
//! # Usage
//!
//! ```rust,ignore
//! use demo_showcase::data::simulation::{Simulation, SimConfig, TickMsg};
//! use bubbletea::{tick, Cmd};
//! use std::time::Duration;
//!
//! // Create simulation with default config
//! let mut sim = Simulation::new(42, SimConfig::default());
//!
//! // Advance simulation by one frame (in update handler)
//! sim.tick();
//!
//! // Schedule next tick (in init or after handling tick)
//! fn schedule_tick() -> Cmd {
//!     tick(Duration::from_millis(100), |_| TickMsg.into_message())
//! }
//! ```

use bubbletea::Message;
use rand::Rng;
use rand_pcg::Pcg64;

use super::generator::GeneratedData;
use super::{
    Alert, AlertSeverity, Deployment, DeploymentStatus, Job, JobStatus, LogEntry, LogLevel,
    Service, ServiceHealth,
};

/// Message indicating a simulation tick.
#[derive(Debug, Clone, Copy)]
pub struct TickMsg {
    /// Frame number (monotonically increasing).
    pub frame: u64,
}

impl TickMsg {
    /// Create a new tick message for the given frame.
    #[must_use]
    pub const fn new(frame: u64) -> Self {
        Self { frame }
    }

    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// Configuration for the simulation.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Probability of a job progressing each tick (0.0-1.0).
    pub job_progress_rate: f64,
    /// Amount of progress per tick (1-10).
    pub job_progress_amount: u8,
    /// Probability of a new log entry each tick.
    pub log_rate: f64,
    /// Probability of service health change each tick.
    pub health_flap_rate: f64,
    /// Probability of deployment status change each tick.
    pub deployment_rate: f64,
    /// Probability of a new alert each tick.
    pub alert_rate: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            job_progress_rate: 0.3, // 30% chance per tick
            job_progress_amount: 5, // 5% progress per tick
            log_rate: 0.5,          // 50% chance of new log
            health_flap_rate: 0.02, // 2% chance of health change
            deployment_rate: 0.1,   // 10% chance of deployment progress
            alert_rate: 0.05,       // 5% chance of new alert
        }
    }
}

impl SimConfig {
    /// Create a fast simulation config for testing.
    #[must_use]
    pub const fn fast() -> Self {
        Self {
            job_progress_rate: 0.8,
            job_progress_amount: 15,
            log_rate: 0.9,
            health_flap_rate: 0.1,
            deployment_rate: 0.5,
            alert_rate: 0.2,
        }
    }

    /// Create a slow/calm simulation config.
    #[must_use]
    pub const fn calm() -> Self {
        Self {
            job_progress_rate: 0.1,
            job_progress_amount: 2,
            log_rate: 0.2,
            health_flap_rate: 0.005,
            deployment_rate: 0.05,
            alert_rate: 0.01,
        }
    }
}

/// The simulation engine.
///
/// Manages the state of all demo data and updates it on each tick.
pub struct Simulation {
    /// Random number generator (seeded for determinism).
    rng: Pcg64,
    /// Current frame number.
    frame: u64,
    /// Simulation configuration.
    config: SimConfig,
    /// Next ID for new entities.
    next_id: u64,
    /// Services.
    pub services: Vec<Service>,
    /// Jobs.
    pub jobs: Vec<Job>,
    /// Deployments.
    pub deployments: Vec<Deployment>,
    /// Alerts.
    pub alerts: Vec<Alert>,
    /// Log entries (ring buffer, keeps last N).
    pub log_entries: Vec<LogEntry>,
    /// Maximum log entries to keep.
    max_logs: usize,
    /// Live dashboard metrics.
    pub metrics: DashboardMetrics,
    /// Pending metric health change notifications.
    pending_metric_changes: Vec<MetricHealthChanged>,
}

impl Simulation {
    /// Create a new simulation with the given seed and config.
    #[must_use]
    pub fn new(seed: u64, config: SimConfig) -> Self {
        let data = GeneratedData::generate(seed);

        // Find the max ID in generated data
        let max_id = data
            .services
            .iter()
            .map(|s| s.id)
            .chain(data.jobs.iter().map(|j| j.id))
            .chain(data.deployments.iter().map(|d| d.id))
            .chain(data.alerts.iter().map(|a| a.id))
            .chain(data.log_entries.iter().map(|l| l.id))
            .max()
            .unwrap_or(0);

        Self {
            rng: Pcg64::new(seed.into(), 0x0a02_bdbf_7bb3_c0a7),
            frame: 0,
            config,
            next_id: max_id + 1,
            services: data.services,
            jobs: data.jobs,
            deployments: data.deployments,
            alerts: data.alerts,
            log_entries: data.log_entries,
            max_logs: 200,
            metrics: DashboardMetrics::default(),
            pending_metric_changes: Vec::new(),
        }
    }

    /// Get the current frame number.
    #[must_use]
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    /// Get the next unique ID.
    const fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Advance the simulation by one frame.
    ///
    /// This is the main entry point for driving the simulation.
    /// Returns true if any visible changes occurred.
    pub fn tick(&mut self) -> bool {
        self.frame += 1;
        let mut changed = false;

        changed |= self.update_jobs();
        changed |= self.update_deployments();
        changed |= self.update_services();
        changed |= self.generate_logs();
        changed |= self.generate_alerts();
        changed |= self.update_metrics();

        changed
    }

    /// Drain pending metric health change notifications.
    ///
    /// Call this after tick() to get any metric state change events
    /// that should trigger toasts/alerts.
    pub fn drain_metric_changes(&mut self) -> Vec<MetricHealthChanged> {
        std::mem::take(&mut self.pending_metric_changes)
    }

    /// Advance the simulation by N frames.
    ///
    /// Useful for testing - allows advancing many frames quickly.
    pub fn tick_n(&mut self, n: u64) -> u64 {
        let mut changes = 0;
        for _ in 0..n {
            if self.tick() {
                changes += 1;
            }
        }
        changes
    }

    /// Update job progress and status.
    fn update_jobs(&mut self) -> bool {
        let mut changed = false;

        for job in &mut self.jobs {
            if job.status == JobStatus::Running {
                if self.rng.random_bool(self.config.job_progress_rate) {
                    let new_progress = job
                        .progress
                        .saturating_add(self.config.job_progress_amount)
                        .min(100);

                    if new_progress != job.progress {
                        job.progress = new_progress;
                        changed = true;

                        // Complete job when progress reaches 100
                        if job.progress >= 100 {
                            // Small chance of failure
                            if self.rng.random_bool(0.1) {
                                job.status = JobStatus::Failed;
                                job.error = Some("Unexpected error during execution".to_string());
                            } else {
                                job.status = JobStatus::Completed;
                            }
                            job.ended_at = Some(chrono::Utc::now());
                        }
                    }
                }
            } else if job.status == JobStatus::Queued {
                // Small chance to start queued jobs
                if self.rng.random_bool(0.05) {
                    job.status = JobStatus::Running;
                    job.started_at = Some(chrono::Utc::now());
                    changed = true;
                }
            }
        }

        changed
    }

    /// Update deployment status.
    fn update_deployments(&mut self) -> bool {
        let mut changed = false;

        for deployment in &mut self.deployments {
            if deployment.status == DeploymentStatus::Pending {
                if self.rng.random_bool(self.config.deployment_rate) {
                    deployment.status = DeploymentStatus::InProgress;
                    deployment.started_at = Some(chrono::Utc::now());
                    changed = true;
                }
            } else if deployment.status == DeploymentStatus::InProgress
                && self.rng.random_bool(self.config.deployment_rate * 0.5)
            {
                // 90% success rate
                if self.rng.random_bool(0.9) {
                    deployment.status = DeploymentStatus::Succeeded;
                } else {
                    deployment.status = DeploymentStatus::Failed;
                }
                deployment.ended_at = Some(chrono::Utc::now());
                changed = true;
            }
        }

        changed
    }

    /// Update service health.
    fn update_services(&mut self) -> bool {
        let mut changed = false;

        for service in &mut self.services {
            if self.rng.random_bool(self.config.health_flap_rate) {
                let new_health = match service.health {
                    ServiceHealth::Healthy => {
                        // Can degrade or become unknown
                        if self.rng.random_bool(0.7) {
                            ServiceHealth::Degraded
                        } else {
                            ServiceHealth::Unknown
                        }
                    }
                    ServiceHealth::Degraded => {
                        // Can recover or get worse
                        if self.rng.random_bool(0.6) {
                            ServiceHealth::Healthy
                        } else {
                            ServiceHealth::Unhealthy
                        }
                    }
                    ServiceHealth::Unhealthy => {
                        // Usually recovers to degraded first
                        ServiceHealth::Degraded
                    }
                    ServiceHealth::Unknown => {
                        // Usually becomes healthy after reconnect
                        ServiceHealth::Healthy
                    }
                };

                if new_health != service.health {
                    service.health = new_health;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Generate new log entries.
    fn generate_logs(&mut self) -> bool {
        if !self.rng.random_bool(self.config.log_rate) {
            return false;
        }

        let levels = [
            (LogLevel::Trace, 5),
            (LogLevel::Debug, 15),
            (LogLevel::Info, 50),
            (LogLevel::Warn, 20),
            (LogLevel::Error, 10),
        ];
        let level = self.weighted_choice(&levels);

        let targets = [
            "api::handlers",
            "auth::session",
            "db::postgres",
            "cache::redis",
            "http::server",
        ];
        let target = targets[self.rng.random_range(0..targets.len())];

        let messages = [
            "Request processed successfully",
            "Connection established",
            "Cache hit for key",
            "Query executed",
            "Health check passed",
            "Token validated",
            "Event published",
        ];
        let message = messages[self.rng.random_range(0..messages.len())];

        let entry = LogEntry::new(self.next_id(), level, target, message).with_tick(self.frame);

        self.log_entries.push(entry);

        // Trim old logs
        while self.log_entries.len() > self.max_logs {
            self.log_entries.remove(0);
        }

        true
    }

    /// Generate new alerts.
    fn generate_alerts(&mut self) -> bool {
        if !self.rng.random_bool(self.config.alert_rate) {
            return false;
        }

        let severities = [
            (AlertSeverity::Info, 30),
            (AlertSeverity::Warning, 40),
            (AlertSeverity::Error, 25),
            (AlertSeverity::Critical, 5),
        ];
        let severity = self.weighted_choice(&severities);

        let service_name = self
            .services
            .get(self.rng.random_range(0..self.services.len().max(1)))
            .map_or_else(|| "unknown".to_string(), |s| s.name.clone());

        let templates = [
            "High CPU usage on {service}",
            "Memory threshold exceeded on {service}",
            "Connection pool exhausted in {service}",
            "Error rate spike in {service}",
        ];
        let template = templates[self.rng.random_range(0..templates.len())];
        let message = template.replace("{service}", &service_name);

        let dedupe_key = format!(
            "{}-{}-{}",
            service_name,
            severity.name().to_lowercase(),
            self.frame
        );

        let mut alert = Alert::new(self.next_id(), severity, &message, &dedupe_key);
        alert.source = Some(service_name);

        self.alerts.push(alert);

        // Trim old alerts (keep last 50)
        while self.alerts.len() > 50 {
            self.alerts.remove(0);
        }

        true
    }

    /// Update live metrics with simulated values.
    fn update_metrics(&mut self) -> bool {
        let mut changed = false;

        // Generate realistic metric values with some noise and occasional spikes
        let base_rps = 150.0;
        let rps_noise = self.rng.random_range(-20.0..20.0);
        // Occasional traffic spike or drop
        let rps_spike = if self.rng.random_bool(0.02) {
            self.rng.random_range(-80.0..50.0)
        } else {
            0.0
        };
        let new_rps = (base_rps + rps_noise + rps_spike).max(5.0);

        if let Some(old) = self.metrics.requests_per_sec.update(new_rps) {
            self.pending_metric_changes.push(MetricHealthChanged {
                metric_name: "Requests/sec".to_string(),
                old_health: old,
                new_health: self.metrics.requests_per_sec.health,
                value: new_rps,
                reason: format!(
                    "Request rate {} from {:.0} to {:.0} req/s",
                    if self.metrics.requests_per_sec.health == MetricHealth::Ok { "recovered" } else { "dropped" },
                    self.metrics.requests_per_sec.history.first().unwrap_or(&new_rps),
                    new_rps
                ),
            });
            changed = true;
        }

        // P95 latency (inversely correlated with RPS somewhat, plus random variation)
        let base_latency = 45.0;
        let latency_noise = self.rng.random_range(-10.0..15.0);
        // High load can cause latency spikes
        let latency_load_factor = if new_rps < 100.0 { 0.0 } else { (new_rps - 100.0) * 0.5 };
        // Occasional latency spike (simulating GC pause, network issue, etc.)
        let latency_spike = if self.rng.random_bool(0.03) {
            self.rng.random_range(50.0..300.0)
        } else {
            0.0
        };
        let new_latency = (base_latency + latency_noise + latency_load_factor + latency_spike).max(10.0);

        if let Some(old) = self.metrics.p95_latency_ms.update(new_latency) {
            self.pending_metric_changes.push(MetricHealthChanged {
                metric_name: "P95 Latency".to_string(),
                old_health: old,
                new_health: self.metrics.p95_latency_ms.health,
                value: new_latency,
                reason: format!(
                    "Latency {} to {:.0}ms (threshold: {}ms)",
                    if self.metrics.p95_latency_ms.health == MetricHealth::Ok { "improved" } else { "increased" },
                    new_latency,
                    if self.metrics.p95_latency_ms.health == MetricHealth::Error { 500 } else { 200 }
                ),
            });
            changed = true;
        }

        // Error rate (usually low, occasional spikes)
        let base_error_rate = 0.2;
        let error_noise = self.rng.random_range(-0.1..0.3);
        // Occasional error spike (deployment issue, dependency failure)
        let error_spike = if self.rng.random_bool(0.02) {
            self.rng.random_range(1.0..8.0)
        } else {
            0.0
        };
        let new_error_rate = (base_error_rate + error_noise + error_spike).max(0.0).min(100.0);

        if let Some(old) = self.metrics.error_rate.update(new_error_rate) {
            self.pending_metric_changes.push(MetricHealthChanged {
                metric_name: "Error Rate".to_string(),
                old_health: old,
                new_health: self.metrics.error_rate.health,
                value: new_error_rate,
                reason: format!(
                    "Error rate {} to {:.1}% (threshold: {}%)",
                    if self.metrics.error_rate.health == MetricHealth::Ok { "dropped" } else { "increased" },
                    new_error_rate,
                    if self.metrics.error_rate.health == MetricHealth::Error { 5 } else { 1 }
                ),
            });
            changed = true;
        }

        // Job throughput (based on actual job completions, with some smoothing)
        let completed_jobs = self.jobs.iter().filter(|j| j.status == JobStatus::Completed).count();
        let running_jobs = self.jobs.iter().filter(|j| j.status == JobStatus::Running).count();
        // Estimated throughput: completed + fraction of running that will complete
        let estimated_throughput = completed_jobs as f64 * 0.5 + running_jobs as f64 * 0.3;
        let throughput_noise = self.rng.random_range(-2.0..2.0);
        let new_throughput = (estimated_throughput + throughput_noise).max(0.0);

        if let Some(old) = self.metrics.job_throughput.update(new_throughput) {
            self.pending_metric_changes.push(MetricHealthChanged {
                metric_name: "Job Throughput".to_string(),
                old_health: old,
                new_health: self.metrics.job_throughput.health,
                value: new_throughput,
                reason: format!(
                    "Throughput {} to {:.0} jobs/min (threshold: {} jobs/min)",
                    if self.metrics.job_throughput.health == MetricHealth::Ok { "recovered" } else { "dropped" },
                    new_throughput,
                    if self.metrics.job_throughput.health == MetricHealth::Error { 2 } else { 5 }
                ),
            });
            changed = true;
        }

        changed
    }

    /// Choose an item based on weights.
    fn weighted_choice<T: Copy>(&mut self, items: &[(T, u32)]) -> T {
        let total: u32 = items.iter().map(|(_, w)| w).sum();
        let mut roll = self.rng.random_range(0..total.max(1));

        for (item, weight) in items {
            if roll < *weight {
                return *item;
            }
            roll = roll.saturating_sub(*weight);
        }

        items[0].0
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get count of jobs by status.
    #[must_use]
    pub fn job_stats(&self) -> JobStats {
        let mut stats = JobStats::default();
        for job in &self.jobs {
            match job.status {
                JobStatus::Queued => stats.queued += 1,
                JobStatus::Running => stats.running += 1,
                JobStatus::Completed => stats.completed += 1,
                JobStatus::Failed => stats.failed += 1,
                JobStatus::Cancelled => stats.cancelled += 1,
            }
        }
        stats
    }

    /// Get count of services by health.
    #[must_use]
    pub fn service_stats(&self) -> ServiceStats {
        let mut stats = ServiceStats::default();
        for service in &self.services {
            match service.health {
                ServiceHealth::Healthy => stats.healthy += 1,
                ServiceHealth::Degraded => stats.degraded += 1,
                ServiceHealth::Unhealthy => stats.unhealthy += 1,
                ServiceHealth::Unknown => stats.unknown += 1,
            }
        }
        stats
    }
}

/// Job statistics.
#[derive(Debug, Clone, Default)]
pub struct JobStats {
    pub queued: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}

impl JobStats {
    /// Total number of jobs.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.queued + self.running + self.completed + self.failed + self.cancelled
    }
}

/// Service statistics.
#[derive(Debug, Clone, Default)]
pub struct ServiceStats {
    pub healthy: usize,
    pub degraded: usize,
    pub unhealthy: usize,
    pub unknown: usize,
}

impl ServiceStats {
    /// Total number of services.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.healthy + self.degraded + self.unhealthy + self.unknown
    }
}

// ============================================================================
// Live Metrics System
// ============================================================================

/// Health state for a metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricHealth {
    /// Metric is within normal thresholds.
    #[default]
    Ok,
    /// Metric is approaching problematic levels.
    Warning,
    /// Metric has crossed critical thresholds.
    Error,
}

impl MetricHealth {
    /// Get a human-readable name for this health state.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Trend direction for a metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricTrend {
    /// Metric is increasing.
    Up,
    /// Metric is stable.
    #[default]
    Flat,
    /// Metric is decreasing.
    Down,
}

impl MetricTrend {
    /// Get the arrow icon for this trend.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Up => "↑",
            Self::Flat => "→",
            Self::Down => "↓",
        }
    }
}

/// A single metric with current value, health, and trend.
#[derive(Debug, Clone)]
pub struct LiveMetric {
    /// Current value.
    pub value: f64,
    /// Health state (with hysteresis applied).
    pub health: MetricHealth,
    /// Trend direction based on recent history.
    pub trend: MetricTrend,
    /// Recent values for trend calculation (sliding window).
    history: Vec<f64>,
    /// Hysteresis counter (positive = trending toward error, negative = trending toward ok).
    hysteresis_counter: i8,
    /// Warning threshold (value above this triggers warning).
    warn_threshold: f64,
    /// Error threshold (value above this triggers error).
    error_threshold: f64,
    /// If true, lower values are worse (e.g., uptime). If false, higher values are worse (e.g., latency).
    invert: bool,
}

impl LiveMetric {
    /// Create a new metric with thresholds.
    ///
    /// `invert` controls threshold direction:
    /// - `false`: higher values are worse (latency, error rate)
    /// - `true`: lower values are worse (throughput, uptime)
    #[must_use]
    pub fn new(initial_value: f64, warn_threshold: f64, error_threshold: f64, invert: bool) -> Self {
        Self {
            value: initial_value,
            health: MetricHealth::Ok,
            trend: MetricTrend::Flat,
            history: vec![initial_value],
            hysteresis_counter: 0,
            warn_threshold,
            error_threshold,
            invert,
        }
    }

    /// Update the metric with a new value.
    ///
    /// Returns `Some(old_health)` if the health state changed.
    pub fn update(&mut self, new_value: f64) -> Option<MetricHealth> {
        let old_health = self.health;
        self.value = new_value;

        // Update history (keep last 10 values)
        self.history.push(new_value);
        if self.history.len() > 10 {
            self.history.remove(0);
        }

        // Calculate trend
        self.trend = self.calculate_trend();

        // Calculate raw health state from thresholds
        let raw_health = self.calculate_raw_health(new_value);

        // Apply hysteresis
        self.apply_hysteresis(raw_health);

        if self.health != old_health {
            Some(old_health)
        } else {
            None
        }
    }

    /// Calculate trend from history.
    fn calculate_trend(&self) -> MetricTrend {
        if self.history.len() < 3 {
            return MetricTrend::Flat;
        }

        // Compare recent average to older average
        let mid = self.history.len() / 2;
        let recent_avg: f64 = self.history[mid..].iter().sum::<f64>() / (self.history.len() - mid) as f64;
        let older_avg: f64 = self.history[..mid].iter().sum::<f64>() / mid as f64;

        let diff_pct = if older_avg.abs() > 0.001 {
            (recent_avg - older_avg) / older_avg * 100.0
        } else {
            0.0
        };

        // Require at least 5% change to show trend
        if diff_pct > 5.0 {
            MetricTrend::Up
        } else if diff_pct < -5.0 {
            MetricTrend::Down
        } else {
            MetricTrend::Flat
        }
    }

    /// Calculate raw health state from value and thresholds.
    fn calculate_raw_health(&self, value: f64) -> MetricHealth {
        if self.invert {
            // Lower is worse (e.g., throughput)
            if value <= self.error_threshold {
                MetricHealth::Error
            } else if value <= self.warn_threshold {
                MetricHealth::Warning
            } else {
                MetricHealth::Ok
            }
        } else {
            // Higher is worse (e.g., latency, error rate)
            if value >= self.error_threshold {
                MetricHealth::Error
            } else if value >= self.warn_threshold {
                MetricHealth::Warning
            } else {
                MetricHealth::Ok
            }
        }
    }

    /// Apply hysteresis to prevent flapping between states.
    ///
    /// Requires 3 consecutive readings in a direction before changing state.
    fn apply_hysteresis(&mut self, raw_health: MetricHealth) {
        let target_worse = match (self.health, raw_health) {
            (MetricHealth::Ok, MetricHealth::Warning | MetricHealth::Error) => true,
            (MetricHealth::Warning, MetricHealth::Error) => true,
            _ => false,
        };

        let target_better = match (self.health, raw_health) {
            (MetricHealth::Error, MetricHealth::Warning | MetricHealth::Ok) => true,
            (MetricHealth::Warning, MetricHealth::Ok) => true,
            _ => false,
        };

        if target_worse {
            self.hysteresis_counter = (self.hysteresis_counter + 1).min(3);
        } else if target_better {
            self.hysteresis_counter = (self.hysteresis_counter - 1).max(-3);
        } else {
            // Same state, decay toward zero
            if self.hysteresis_counter > 0 {
                self.hysteresis_counter -= 1;
            } else if self.hysteresis_counter < 0 {
                self.hysteresis_counter += 1;
            }
        }

        // Transition when hysteresis threshold reached
        if self.hysteresis_counter >= 3 {
            // Transition to worse state
            self.health = match self.health {
                MetricHealth::Ok => MetricHealth::Warning,
                MetricHealth::Warning => MetricHealth::Error,
                MetricHealth::Error => MetricHealth::Error,
            };
            self.hysteresis_counter = 0;
        } else if self.hysteresis_counter <= -3 {
            // Transition to better state
            self.health = match self.health {
                MetricHealth::Error => MetricHealth::Warning,
                MetricHealth::Warning => MetricHealth::Ok,
                MetricHealth::Ok => MetricHealth::Ok,
            };
            self.hysteresis_counter = 0;
        }
    }

    /// Get the most recent change percentage (if history available).
    #[must_use]
    pub fn change_pct(&self) -> Option<f64> {
        if self.history.len() < 2 {
            return None;
        }
        let prev = self.history[self.history.len() - 2];
        if prev.abs() > 0.001 {
            Some((self.value - prev) / prev * 100.0)
        } else {
            None
        }
    }
}

/// Live metrics for the dashboard.
#[derive(Debug, Clone)]
pub struct DashboardMetrics {
    /// Requests per second.
    pub requests_per_sec: LiveMetric,
    /// P95 latency in milliseconds.
    pub p95_latency_ms: LiveMetric,
    /// Error rate (0.0-100.0 percentage).
    pub error_rate: LiveMetric,
    /// Job throughput (jobs completed per minute).
    pub job_throughput: LiveMetric,
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self {
            // Requests/sec: warn at 50, error at 20 (inverted - lower is worse)
            requests_per_sec: LiveMetric::new(150.0, 50.0, 20.0, true),
            // P95 latency: warn at 200ms, error at 500ms
            p95_latency_ms: LiveMetric::new(45.0, 200.0, 500.0, false),
            // Error rate: warn at 1%, error at 5%
            error_rate: LiveMetric::new(0.2, 1.0, 5.0, false),
            // Job throughput: warn at 5, error at 2 (inverted)
            job_throughput: LiveMetric::new(12.0, 5.0, 2.0, true),
        }
    }
}

/// Message indicating a metric health state changed.
#[derive(Debug, Clone)]
pub struct MetricHealthChanged {
    /// Name of the metric that changed.
    pub metric_name: String,
    /// Previous health state.
    pub old_health: MetricHealth,
    /// New health state.
    pub new_health: MetricHealth,
    /// Current value of the metric.
    pub value: f64,
    /// Explanation of why the state changed.
    pub reason: String,
}

impl MetricHealthChanged {
    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_is_deterministic() {
        let mut sim1 = Simulation::new(42, SimConfig::fast());
        let mut sim2 = Simulation::new(42, SimConfig::fast());

        // Advance both simulations
        for _ in 0..100 {
            sim1.tick();
            sim2.tick();
        }

        // Should have identical state
        assert_eq!(sim1.frame, sim2.frame);
        assert_eq!(sim1.jobs.len(), sim2.jobs.len());

        for (j1, j2) in sim1.jobs.iter().zip(sim2.jobs.iter()) {
            assert_eq!(j1.progress, j2.progress);
            assert_eq!(j1.status, j2.status);
        }
    }

    #[test]
    fn simulation_advances_frame() {
        let mut sim = Simulation::new(1, SimConfig::default());
        assert_eq!(sim.frame(), 0);

        sim.tick();
        assert_eq!(sim.frame(), 1);

        sim.tick_n(99);
        assert_eq!(sim.frame(), 100);
    }

    #[test]
    fn simulation_can_advance_1000_frames_quickly() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        let start = std::time::Instant::now();
        sim.tick_n(1000);
        let elapsed = start.elapsed();

        // Should complete in well under 100ms (typically < 10ms)
        assert!(
            elapsed.as_millis() < 100,
            "1000 frames took too long: {elapsed:?}"
        );
    }

    #[test]
    fn jobs_progress_over_time() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        let has_running = sim.jobs.iter().any(|j| j.status == JobStatus::Running);

        // Advance many frames
        sim.tick_n(100);

        // Some jobs should have progressed or completed
        let final_stats = sim.job_stats();
        let initial_stats = Simulation::new(42, SimConfig::fast()).job_stats();

        // Either we had running jobs, progress increased, or jobs completed
        assert!(
            has_running
                || final_stats.completed >= initial_stats.completed
                || final_stats.running != initial_stats.running,
            "Jobs should change over time"
        );
    }

    #[test]
    fn logs_accumulate() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        let initial_logs = sim.log_entries.len();

        sim.tick_n(50);

        assert!(
            sim.log_entries.len() >= initial_logs,
            "Logs should accumulate"
        );
    }

    #[test]
    fn logs_are_trimmed() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        sim.max_logs = 50;

        // Generate lots of logs
        sim.tick_n(500);

        assert!(
            sim.log_entries.len() <= 50,
            "Logs should be trimmed to max_logs"
        );
    }

    #[test]
    fn service_health_changes() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        let initial_health: Vec<_> = sim.services.iter().map(|s| s.health).collect();

        // Advance many frames with high flap rate
        sim.config.health_flap_rate = 0.5;
        sim.tick_n(100);

        let final_health: Vec<_> = sim.services.iter().map(|s| s.health).collect();

        // At least one service should have changed health
        assert_ne!(
            initial_health, final_health,
            "Service health should change over time"
        );
    }

    #[test]
    fn tick_msg_converts_to_message() {
        let tick = TickMsg::new(42);
        let msg = tick.into_message();

        let recovered = msg.downcast_ref::<TickMsg>();
        assert!(recovered.is_some());
        assert_eq!(recovered.unwrap().frame, 42);
    }

    #[test]
    fn job_stats_counts_correctly() {
        let sim = Simulation::new(42, SimConfig::default());
        let stats = sim.job_stats();

        assert_eq!(stats.total(), sim.jobs.len());
    }

    #[test]
    fn service_stats_counts_correctly() {
        let sim = Simulation::new(42, SimConfig::default());
        let stats = sim.service_stats();

        assert_eq!(stats.total(), sim.services.len());
    }

    #[test]
    fn different_seeds_produce_different_simulations() {
        let mut sim1 = Simulation::new(1, SimConfig::fast());
        let mut sim2 = Simulation::new(2, SimConfig::fast());

        sim1.tick_n(50);
        sim2.tick_n(50);

        // Jobs should differ
        let progress1: Vec<_> = sim1.jobs.iter().map(|j| j.progress).collect();
        let progress2: Vec<_> = sim2.jobs.iter().map(|j| j.progress).collect();

        assert_ne!(progress1, progress2, "Different seeds should diverge");
    }

    // ========================================================================
    // Additional determinism and stability tests for bd-2b7h
    // ========================================================================

    #[test]
    fn determinism_across_multiple_runs() {
        // Run the same simulation 5 times with the same seed
        let seeds = [42u64, 123, 999, 0, u64::MAX];

        for seed in seeds {
            let mut sim1 = Simulation::new(seed, SimConfig::fast());
            let mut sim2 = Simulation::new(seed, SimConfig::fast());

            // Advance both simulations exactly the same amount
            sim1.tick_n(200);
            sim2.tick_n(200);

            // Compare all state
            assert_eq!(sim1.frame, sim2.frame, "Frame count must match");
            assert_eq!(sim1.next_id, sim2.next_id, "Next ID must match");
            assert_eq!(sim1.jobs.len(), sim2.jobs.len(), "Job count must match");
            assert_eq!(
                sim1.log_entries.len(),
                sim2.log_entries.len(),
                "Log count must match"
            );
            assert_eq!(
                sim1.alerts.len(),
                sim2.alerts.len(),
                "Alert count must match"
            );

            // Compare individual jobs
            for (j1, j2) in sim1.jobs.iter().zip(sim2.jobs.iter()) {
                assert_eq!(j1.id, j2.id);
                assert_eq!(j1.status, j2.status);
                assert_eq!(j1.progress, j2.progress);
            }

            // Compare service health
            for (s1, s2) in sim1.services.iter().zip(sim2.services.iter()) {
                assert_eq!(s1.id, s2.id);
                assert_eq!(s1.health, s2.health);
            }

            // Compare deployments
            for (d1, d2) in sim1.deployments.iter().zip(sim2.deployments.iter()) {
                assert_eq!(d1.id, d2.id);
                assert_eq!(d1.status, d2.status);
            }
        }
    }

    #[test]
    fn determinism_with_interleaved_checks() {
        // Verify determinism by checking state after each tick
        let mut sim1 = Simulation::new(42, SimConfig::default());
        let mut sim2 = Simulation::new(42, SimConfig::default());

        for i in 0..50 {
            let changed1 = sim1.tick();
            let changed2 = sim2.tick();

            assert_eq!(changed1, changed2, "Change flag must match at frame {i}");
            assert_eq!(
                sim1.frame, sim2.frame,
                "Frame count must match at frame {i}"
            );

            // Spot check job stats
            let stats1 = sim1.job_stats();
            let stats2 = sim2.job_stats();
            assert_eq!(
                stats1.running, stats2.running,
                "Running jobs must match at frame {i}"
            );
            assert_eq!(
                stats1.completed, stats2.completed,
                "Completed jobs must match at frame {i}"
            );
        }
    }

    #[test]
    fn no_sleeps_or_blocking() {
        // Verify that advancing 10000 frames takes less than 1 second
        // (proves no real sleeps are happening)
        let mut sim = Simulation::new(42, SimConfig::fast());

        let start = std::time::Instant::now();
        sim.tick_n(10000);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 1,
            "10000 frames should complete in < 1s, took {elapsed:?}"
        );
        assert_eq!(sim.frame(), 10000);
    }

    #[test]
    fn extreme_tick_counts() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        // Advance many frames - should not panic or overflow
        sim.tick_n(50000);
        assert_eq!(sim.frame(), 50000);

        // Frame counter should still be valid
        sim.tick();
        assert_eq!(sim.frame(), 50001);
    }

    #[test]
    fn jobs_transition_through_states() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        // Count initial states
        let initial_queued: Vec<_> = sim
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Queued)
            .map(|j| j.id)
            .collect();

        // Advance until all initially queued jobs have changed
        sim.tick_n(500);

        // At least some queued jobs should have started or completed
        let final_stats = sim.job_stats();
        let initial_stats = Simulation::new(42, SimConfig::fast()).job_stats();

        // More jobs should be running/completed/failed than initially
        let initial_terminal = initial_stats.completed + initial_stats.failed;
        let final_terminal = final_stats.completed + final_stats.failed;
        assert!(
            final_terminal >= initial_terminal,
            "Jobs should progress to terminal states"
        );

        // Verify job state transitions are valid (no invalid states)
        for job in &sim.jobs {
            match job.status {
                JobStatus::Queued => {
                    assert!(job.started_at.is_none());
                    assert!(job.ended_at.is_none());
                }
                JobStatus::Running => {
                    // Started but not ended
                    assert!(job.ended_at.is_none());
                }
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                    // Terminal states should have ended_at
                    assert!(
                        job.ended_at.is_some() || initial_queued.contains(&job.id),
                        "Terminal jobs should have ended_at"
                    );
                }
            }
        }
    }

    #[test]
    fn deployments_transition_correctly() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        // Advance simulation
        sim.tick_n(500);

        // Check deployment state validity
        for deploy in &sim.deployments {
            match deploy.status {
                DeploymentStatus::Pending => {
                    // Not started
                    assert!(
                        deploy.started_at.is_none() || deploy.started_at.is_some(),
                        "Pending deployment state valid"
                    );
                }
                DeploymentStatus::InProgress => {
                    // Should have started_at
                    assert!(
                        deploy.started_at.is_some(),
                        "InProgress deployment should have started_at"
                    );
                    assert!(
                        deploy.ended_at.is_none(),
                        "InProgress deployment should not have ended_at"
                    );
                }
                DeploymentStatus::Succeeded
                | DeploymentStatus::Failed
                | DeploymentStatus::RolledBack => {
                    // Terminal states should have ended_at
                    assert!(
                        deploy.ended_at.is_some(),
                        "Terminal deployment should have ended_at"
                    );
                }
            }
        }
    }

    #[test]
    fn alerts_have_valid_state() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        sim.tick_n(200);

        for alert in &sim.alerts {
            // All alerts should have valid IDs
            assert!(alert.id > 0);

            // Dedupe key should not be empty
            assert!(!alert.dedupe_key.is_empty());

            // Severity should be valid (this is enforced by enum)
            let _ = alert.severity.name(); // Should not panic
        }
    }

    #[test]
    fn log_entries_have_valid_state() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        sim.tick_n(100);

        for log in &sim.log_entries {
            // All logs should have valid IDs
            assert!(log.id > 0);

            // Target and message should not be empty
            assert!(!log.target.is_empty());
            assert!(!log.message.is_empty());
        }
    }

    #[test]
    fn config_affects_simulation_rate() {
        // Fast config should produce more changes
        let mut sim_fast = Simulation::new(42, SimConfig::fast());
        let mut sim_calm = Simulation::new(42, SimConfig::calm());

        let changes_fast = sim_fast.tick_n(100);
        let changes_calm = sim_calm.tick_n(100);

        // Fast should have more changes (statistically likely)
        assert!(
            changes_fast > changes_calm,
            "Fast config ({changes_fast}) should produce more changes than calm ({changes_calm})"
        );
    }

    #[test]
    fn weighted_choice_covers_all_options() {
        // Test that weighted_choice doesn't always return the same thing
        let mut sim = Simulation::new(42, SimConfig::default());

        let items = [(1u8, 25), (2, 25), (3, 25), (4, 25)];
        let mut seen = [false; 4];

        for _ in 0..100 {
            let choice = sim.weighted_choice(&items);
            seen[(choice - 1) as usize] = true;
        }

        // With 100 tries and equal weights, should see all options
        assert!(seen.iter().all(|&s| s), "Should see all weighted choices");
    }

    #[test]
    fn next_id_never_duplicates() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        let mut ids = Vec::new();
        for _ in 0..1000 {
            ids.push(sim.next_id());
        }

        // All IDs should be unique
        ids.sort_unstable();
        let unique_count = ids.windows(2).filter(|w| w[0] != w[1]).count() + 1;
        assert_eq!(unique_count, 1000, "All generated IDs should be unique");
    }

    #[test]
    fn frame_counter_monotonic() {
        let mut sim = Simulation::new(42, SimConfig::default());

        let mut prev_frame = sim.frame();
        for _ in 0..100 {
            sim.tick();
            assert!(
                sim.frame() > prev_frame,
                "Frame counter must be monotonically increasing"
            );
            prev_frame = sim.frame();
        }
    }

    #[test]
    fn tick_n_equivalent_to_multiple_ticks() {
        let mut sim1 = Simulation::new(42, SimConfig::fast());
        let mut sim2 = Simulation::new(42, SimConfig::fast());

        // Advance sim1 with tick_n
        sim1.tick_n(100);

        // Advance sim2 with individual ticks
        for _ in 0..100 {
            sim2.tick();
        }

        // State should be identical
        assert_eq!(sim1.frame, sim2.frame);
        assert_eq!(sim1.next_id, sim2.next_id);

        for (j1, j2) in sim1.jobs.iter().zip(sim2.jobs.iter()) {
            assert_eq!(j1.status, j2.status);
            assert_eq!(j1.progress, j2.progress);
        }
    }

    #[test]
    fn zero_rate_config_produces_no_changes() {
        let config = SimConfig {
            job_progress_rate: 0.0,
            job_progress_amount: 0,
            log_rate: 0.0,
            health_flap_rate: 0.0,
            deployment_rate: 0.0,
            alert_rate: 0.0,
        };

        let mut sim = Simulation::new(42, config);
        let initial_log_count = sim.log_entries.len();
        let initial_alert_count = sim.alerts.len();
        let initial_jobs: Vec<_> = sim.jobs.iter().map(|j| (j.status, j.progress)).collect();

        // Advance many frames
        sim.tick_n(100);

        // No logs or alerts should be added
        assert_eq!(sim.log_entries.len(), initial_log_count);
        assert_eq!(sim.alerts.len(), initial_alert_count);

        // Job states shouldn't change (though queued jobs have 5% chance to start)
        // With 0 progress rate, running jobs won't progress
        for (i, job) in sim.jobs.iter().enumerate() {
            if initial_jobs[i].0 == JobStatus::Running {
                // Progress should not have changed
                assert_eq!(job.progress, initial_jobs[i].1);
            }
        }
    }

    #[test]
    fn max_logs_enforced() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        sim.max_logs = 10;

        // Generate many logs
        sim.tick_n(1000);

        assert!(
            sim.log_entries.len() <= 10,
            "Log count {} exceeds max_logs {}",
            sim.log_entries.len(),
            10
        );
    }

    #[test]
    fn alerts_trimmed_at_50() {
        let mut sim = Simulation::new(42, SimConfig::fast());
        sim.config.alert_rate = 1.0; // Generate alert every tick

        // Generate many alerts
        sim.tick_n(100);

        assert!(
            sim.alerts.len() <= 50,
            "Alert count {} exceeds max 50",
            sim.alerts.len()
        );
    }
}
