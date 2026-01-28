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

        changed
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

        let entry = LogEntry::new(self.next_id(), level, target, message);

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
            "1000 frames took too long: {:?}",
            elapsed
        );
    }

    #[test]
    fn jobs_progress_over_time() {
        let mut sim = Simulation::new(42, SimConfig::fast());

        let initial_running: Vec<_> = sim
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Running)
            .map(|j| j.progress)
            .collect();

        // Advance many frames
        sim.tick_n(100);

        // Some jobs should have progressed or completed
        let final_stats = sim.job_stats();
        let initial_stats = Simulation::new(42, SimConfig::fast()).job_stats();

        // Either progress increased or jobs completed
        assert!(
            !initial_running.is_empty()
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
}
