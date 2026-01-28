//! Dashboard page - platform health overview.
//!
//! The dashboard provides an at-a-glance view of the platform's health,
//! showing key metrics, service status, recent deployments, and jobs.
//!
//! This page integrates with the simulation engine to provide live-updating
//! metrics with trends, health indicators, and notifications.

use std::time::Duration;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, tick};
use lipgloss::{Position, Style};

use super::PageModel;
use crate::components::{
    DeltaDirection, StatusLevel, badge, chip, divider_with_label, stat_widget,
};
use crate::data::animation::Animator;
use crate::data::simulation::{MetricHealth, MetricTrend, SimConfig, Simulation, TickMsg};
use crate::data::{Deployment, DeploymentStatus, Job, JobStatus, Service, ServiceHealth};
use crate::messages::{Notification, NotificationMsg, Page};
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Tick interval for simulation updates (100ms = 10 fps).
const TICK_INTERVAL_MS: u64 = 100;

/// Dashboard page showing platform health overview.
///
/// Uses the simulation engine to provide live-updating metrics with
/// trends, health states, and automatic notifications.
///
/// # Animations
///
/// Metric values are animated using spring physics for smooth transitions.
/// Pass `animations_enabled: false` to the constructor to disable animations
/// (values will snap instantly instead).
pub struct DashboardPage {
    /// Simulation engine managing all live data.
    simulation: Simulation,
    /// Current seed for data generation.
    seed: u64,
    /// Simulated uptime in seconds.
    uptime_seconds: u64,
    /// Ticks since last uptime increment (10 ticks = 1 second at 100ms/tick).
    ticks_since_uptime: u64,
    /// Counter for generating unique notification IDs.
    next_notification_id: u64,
    /// Animator for smooth metric value transitions.
    animator: Animator,
}

impl DashboardPage {
    /// Create a new dashboard page with animations enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(DEFAULT_SEED, true)
    }

    /// Create a new dashboard page with the given seed and animations enabled.
    #[must_use]
    #[allow(dead_code)] // Used by Pages struct and tests
    pub fn with_seed(seed: u64) -> Self {
        Self::with_options(seed, true)
    }

    /// Create a new dashboard page with full control over options.
    #[must_use]
    pub fn with_options(seed: u64, animations_enabled: bool) -> Self {
        let simulation = Simulation::new(seed, SimConfig::default());

        // Initialize animator with current metric values (snap, don't animate from 0)
        let mut animator = Animator::new(animations_enabled);
        animator.set(
            "requests_per_sec",
            simulation.metrics.requests_per_sec.value,
        );
        animator.set("p95_latency_ms", simulation.metrics.p95_latency_ms.value);
        animator.set("error_rate", simulation.metrics.error_rate.value);
        animator.set("job_throughput", simulation.metrics.job_throughput.value);

        Self {
            simulation,
            seed,
            uptime_seconds: 86400 * 7 + 3600 * 5 + 60 * 23, // 7d 5h 23m
            ticks_since_uptime: 0,
            next_notification_id: 1,
            animator,
        }
    }

    /// Set whether animations are enabled.
    ///
    /// When disabled, metric values snap instantly to their targets.
    #[allow(dead_code)] // API for config integration
    pub const fn set_animations(&mut self, enabled: bool) {
        self.animator.set_enabled(enabled);
    }

    /// Check if animations are enabled.
    #[must_use]
    #[allow(dead_code)] // API for config integration
    pub const fn animations_enabled(&self) -> bool {
        self.animator.is_enabled()
    }

    /// Refresh data by resetting the simulation with the current seed.
    pub fn refresh(&mut self) {
        self.simulation = Simulation::new(self.seed, SimConfig::default());
        self.ticks_since_uptime = 0;

        // Re-initialize animator with fresh metric values (snap, don't animate)
        let animations_enabled = self.animator.is_enabled();
        self.animator = Animator::new(animations_enabled);
        self.animator.set(
            "requests_per_sec",
            self.simulation.metrics.requests_per_sec.value,
        );
        self.animator.set(
            "p95_latency_ms",
            self.simulation.metrics.p95_latency_ms.value,
        );
        self.animator
            .set("error_rate", self.simulation.metrics.error_rate.value);
        self.animator.set(
            "job_throughput",
            self.simulation.metrics.job_throughput.value,
        );
    }

    /// Schedule the next simulation tick.
    fn schedule_tick(&self) -> Cmd {
        let frame = self.simulation.frame();
        tick(Duration::from_millis(TICK_INTERVAL_MS), move |_| {
            TickMsg::new(frame + 1).into_message()
        })
    }

    /// Process a simulation tick, returning notifications for any metric changes.
    fn process_tick(&mut self) -> Vec<NotificationMsg> {
        self.simulation.tick();

        // Update animator targets with new metric values (animator handles smoothing)
        self.animator.animate(
            "requests_per_sec",
            self.simulation.metrics.requests_per_sec.value,
        );
        self.animator.animate(
            "p95_latency_ms",
            self.simulation.metrics.p95_latency_ms.value,
        );
        self.animator
            .animate("error_rate", self.simulation.metrics.error_rate.value);
        self.animator.animate(
            "job_throughput",
            self.simulation.metrics.job_throughput.value,
        );

        // Advance animations
        self.animator.tick();

        // Update uptime (10 ticks = 1 second at 100ms/tick)
        self.ticks_since_uptime += 1;
        if self.ticks_since_uptime >= 10 {
            self.ticks_since_uptime = 0;
            self.uptime_seconds += 1;
        }

        // Convert metric health changes to notifications
        let changes = self.simulation.drain_metric_changes();
        changes
            .into_iter()
            .filter_map(|change| {
                // Only notify on significant changes (to warning/error or recovery)
                let level = match change.new_health {
                    MetricHealth::Ok => StatusLevel::Success,
                    MetricHealth::Warning => StatusLevel::Warning,
                    MetricHealth::Error => StatusLevel::Error,
                };

                // Skip ok->ok transitions
                if change.old_health == MetricHealth::Ok && change.new_health == MetricHealth::Ok {
                    return None;
                }

                let id = self.next_notification_id;
                self.next_notification_id += 1;

                let notification = Notification::new(id, &change.reason, level);
                Some(NotificationMsg::Show(notification))
            })
            .collect()
    }

    // ========================================================================
    // Data Accessors
    // ========================================================================

    /// Get the services from the simulation.
    fn services(&self) -> &[Service] {
        &self.simulation.services
    }

    /// Get the deployments from the simulation.
    fn deployments(&self) -> &[Deployment] {
        &self.simulation.deployments
    }

    /// Get the jobs from the simulation.
    fn jobs(&self) -> &[Job] {
        &self.simulation.jobs
    }

    // ========================================================================
    // Stats Helpers
    // ========================================================================

    /// Count services by health status.
    fn service_health_counts(&self) -> (usize, usize, usize, usize) {
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut unknown = 0;

        for service in self.services() {
            match service.health {
                ServiceHealth::Healthy => healthy += 1,
                ServiceHealth::Degraded => degraded += 1,
                ServiceHealth::Unhealthy => unhealthy += 1,
                ServiceHealth::Unknown => unknown += 1,
            }
        }

        (healthy, degraded, unhealthy, unknown)
    }

    /// Count jobs by status.
    fn job_status_counts(&self) -> (usize, usize, usize, usize) {
        let mut queued = 0;
        let mut running = 0;
        let mut completed = 0;
        let mut failed = 0;

        for job in self.jobs() {
            match job.status {
                JobStatus::Queued => queued += 1,
                JobStatus::Running => running += 1,
                JobStatus::Completed => completed += 1,
                JobStatus::Failed | JobStatus::Cancelled => failed += 1,
            }
        }

        (queued, running, completed, failed)
    }

    /// Get recent deployments (last 3).
    fn recent_deployments(&self) -> Vec<&Deployment> {
        let mut sorted: Vec<_> = self.deployments().iter().collect();
        sorted.sort_by_key(|d| std::cmp::Reverse(d.created_at));
        sorted.into_iter().take(3).collect()
    }

    /// Get recent jobs (last 4).
    fn recent_jobs(&self) -> Vec<&Job> {
        let mut sorted: Vec<_> = self.jobs().iter().collect();
        sorted.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        sorted.into_iter().take(4).collect()
    }

    /// Format uptime as human-readable string.
    fn format_uptime(&self) -> String {
        let days = self.uptime_seconds / 86400;
        let hours = (self.uptime_seconds % 86400) / 3600;
        let minutes = (self.uptime_seconds % 3600) / 60;

        if days > 0 {
            format!("{days}d {hours}h {minutes}m")
        } else if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    }

    // ========================================================================
    // Render Helpers
    // ========================================================================

    /// Render the status bar (top row).
    fn render_status_bar(&self, theme: &Theme, width: usize) -> String {
        let (healthy, degraded, unhealthy, _) = self.service_health_counts();
        let total = self.services().len();

        // Platform status
        let platform_status = if unhealthy > 0 {
            badge(theme, StatusLevel::Error, "DEGRADED")
        } else if degraded > 0 {
            badge(theme, StatusLevel::Warning, "PARTIAL")
        } else {
            badge(theme, StatusLevel::Success, "HEALTHY")
        };

        // Service summary
        let service_summary = format!("{healthy}/{total} services healthy");
        let service_styled = if unhealthy > 0 {
            theme.error_style().render(&service_summary)
        } else if degraded > 0 {
            theme.warning_style().render(&service_summary)
        } else {
            theme.success_style().render(&service_summary)
        };

        // Uptime
        let uptime = format!("Uptime: {}", self.format_uptime());
        let uptime_styled = theme.muted_style().render(&uptime);

        // Compose status bar
        let content = format!("{platform_status}  {service_styled}  {uptime_styled}");

        // Truncate if needed
        if content.len() > width {
            content
                .chars()
                .take(width.saturating_sub(3))
                .collect::<String>()
                + "..."
        } else {
            content
        }
    }

    /// Render the stats row (key metrics).
    fn render_stats_row(&self, theme: &Theme, width: usize) -> String {
        let (healthy, degraded, unhealthy, _) = self.service_health_counts();
        let (queued, running, completed, failed) = self.job_status_counts();
        let recent_deploys = self
            .deployments()
            .iter()
            .filter(|d| !d.status.is_terminal())
            .count();

        // Calculate card width (divide into 4 columns)
        let card_width = width.saturating_sub(6) / 4;

        // Prepare delta strings
        let issues_str = format!("{} issues", unhealthy + degraded);
        let running_str = format!("{running} running");
        let failed_str = format!("{failed} failed");

        let stat1 = stat_widget(
            theme,
            "Services",
            &format!("{healthy}/{}", self.services().len()),
            if unhealthy > 0 || degraded > 0 {
                Some((&issues_str, DeltaDirection::Down))
            } else {
                None
            },
        );

        let stat2 = stat_widget(
            theme,
            "Active Jobs",
            &format!("{}", queued + running),
            if running > 0 {
                Some((&running_str, DeltaDirection::Neutral))
            } else {
                None
            },
        );

        let stat3 = stat_widget(
            theme,
            "Completed",
            &completed.to_string(),
            if failed > 0 {
                Some((&failed_str, DeltaDirection::Down))
            } else {
                Some(("all passed", DeltaDirection::Up))
            },
        );

        let stat4 = stat_widget(
            theme,
            "Deploys",
            &recent_deploys.to_string(),
            if recent_deploys > 0 {
                Some(("in progress", DeltaDirection::Neutral))
            } else {
                Some(("idle", DeltaDirection::Neutral))
            },
        );

        // Render each stat in a box
        #[expect(clippy::cast_possible_truncation)]
        let card_w = card_width as u16;

        let box1 = theme.box_style().width(card_w).render(&stat1);
        let box2 = theme.box_style().width(card_w).render(&stat2);
        let box3 = theme.box_style().width(card_w).render(&stat3);
        let box4 = theme.box_style().width(card_w).render(&stat4);

        lipgloss::join_horizontal(Position::Top, &[&box1, &box2, &box3, &box4])
    }

    /// Render the services section.
    fn render_services(&self, theme: &Theme, width: usize) -> String {
        let header = divider_with_label(theme, "Services", width);

        let mut lines = Vec::new();

        for service in self.services().iter().take(6) {
            let status = match service.health {
                ServiceHealth::Healthy => chip(theme, StatusLevel::Success, ""),
                ServiceHealth::Degraded => chip(theme, StatusLevel::Warning, ""),
                ServiceHealth::Unhealthy => chip(theme, StatusLevel::Error, ""),
                ServiceHealth::Unknown => chip(theme, StatusLevel::Info, ""),
            };

            let name = Style::new()
                .foreground(theme.text)
                .width(18)
                .render(&service.name);

            let version = theme.muted_style().render(&service.version);

            lines.push(format!("{status} {name} {version}"));
        }

        let content = lines.join("\n");
        format!("{header}\n{content}")
    }

    /// Render the deployments section.
    fn render_deployments(&self, theme: &Theme, width: usize) -> String {
        let header = divider_with_label(theme, "Recent Deployments", width);

        let recent = self.recent_deployments();

        if recent.is_empty() {
            let empty = theme.muted_style().render("No recent deployments");
            return format!("{header}\n{empty}");
        }

        let mut lines = Vec::new();

        for deploy in recent {
            let status_chip = match deploy.status {
                DeploymentStatus::Pending => chip(theme, StatusLevel::Info, "pending"),
                DeploymentStatus::InProgress => chip(theme, StatusLevel::Running, "deploying"),
                DeploymentStatus::Succeeded => chip(theme, StatusLevel::Success, "success"),
                DeploymentStatus::Failed => chip(theme, StatusLevel::Error, "failed"),
                DeploymentStatus::RolledBack => chip(theme, StatusLevel::Warning, "rolled back"),
            };

            let sha_short = if deploy.sha.len() > 7 {
                &deploy.sha[..7]
            } else {
                &deploy.sha
            };
            let sha_styled = theme.muted_style().render(sha_short);

            let author = Style::new().foreground(theme.text).render(&deploy.author);

            lines.push(format!("{status_chip}  {sha_styled}  {author}"));
        }

        let content = lines.join("\n");
        format!("{header}\n{content}")
    }

    /// Render the jobs section.
    fn render_jobs(&self, theme: &Theme, width: usize) -> String {
        let header = divider_with_label(theme, "Recent Jobs", width);

        let recent = self.recent_jobs();

        if recent.is_empty() {
            let empty = theme.muted_style().render("No recent jobs");
            return format!("{header}\n{empty}");
        }

        let mut lines = Vec::new();

        for job in recent {
            let status_chip = match job.status {
                JobStatus::Queued => chip(theme, StatusLevel::Info, ""),
                JobStatus::Running => chip(theme, StatusLevel::Running, ""),
                JobStatus::Completed => chip(theme, StatusLevel::Success, ""),
                JobStatus::Failed | JobStatus::Cancelled => chip(theme, StatusLevel::Error, ""),
            };

            let name = Style::new()
                .foreground(theme.text)
                .width(20)
                .render(&job.name);

            let progress = if job.status == JobStatus::Running {
                theme.info_style().render(&format!("{}%", job.progress))
            } else if job.status == JobStatus::Completed {
                theme.success_style().render("done")
            } else if job.status == JobStatus::Failed {
                theme.error_style().render("failed")
            } else {
                theme.muted_style().render("queued")
            };

            lines.push(format!("{status_chip} {name} {progress}"));
        }

        let content = lines.join("\n");
        format!("{header}\n{content}")
    }

    /// Render the live metrics panel showing real-time metrics with trends.
    fn render_live_metrics(&self, theme: &Theme, width: usize) -> String {
        let header = divider_with_label(theme, "Live Metrics", width);

        let metrics = &self.simulation.metrics;
        let mut lines = Vec::new();

        // Helper to render a single metric line with value, health, and trend
        let render_metric = |label: &str,
                             value: f64,
                             unit: &str,
                             health: MetricHealth,
                             trend: MetricTrend|
         -> String {
            // Health indicator
            let health_chip = match health {
                MetricHealth::Ok => chip(theme, StatusLevel::Success, ""),
                MetricHealth::Warning => chip(theme, StatusLevel::Warning, ""),
                MetricHealth::Error => chip(theme, StatusLevel::Error, ""),
            };

            // Format value
            let value_str = if value >= 100.0 {
                format!("{value:.0}{unit}")
            } else if value >= 10.0 {
                format!("{value:.1}{unit}")
            } else {
                format!("{value:.2}{unit}")
            };

            // Trend indicator with color
            let trend_icon = trend.icon();
            let trend_styled = match (health, trend) {
                // For metrics where "up" is bad (latency, error rate), show good trends in green
                (MetricHealth::Ok, MetricTrend::Down) => theme.success_style().render(trend_icon),
                // Warning with upward trend is concerning
                (MetricHealth::Warning, MetricTrend::Up) => {
                    theme.warning_style().render(trend_icon)
                }
                // Error state always shows red
                (MetricHealth::Error, _) => theme.error_style().render(trend_icon),
                // All other combinations (ok/flat, ok/up, warning/flat, warning/down) use muted
                _ => theme.muted_style().render(trend_icon),
            };

            // Label and value
            let label_styled = Style::new().foreground(theme.text).width(14).render(label);
            let value_styled = theme.heading_style().render(&value_str);

            format!("{health_chip} {label_styled} {value_styled} {trend_styled}")
        };

        // Render each metric using animated values for smooth transitions
        // Health and trend indicators use raw simulation data (categorical, not animated)
        lines.push(render_metric(
            "Requests/s",
            self.animator
                .get_or("requests_per_sec", metrics.requests_per_sec.value),
            "",
            metrics.requests_per_sec.health,
            metrics.requests_per_sec.trend,
        ));

        lines.push(render_metric(
            "P95 Latency",
            self.animator
                .get_or("p95_latency_ms", metrics.p95_latency_ms.value),
            "ms",
            metrics.p95_latency_ms.health,
            metrics.p95_latency_ms.trend,
        ));

        lines.push(render_metric(
            "Error Rate",
            self.animator.get_or("error_rate", metrics.error_rate.value),
            "%",
            metrics.error_rate.health,
            metrics.error_rate.trend,
        ));

        lines.push(render_metric(
            "Job Throughput",
            self.animator
                .get_or("job_throughput", metrics.job_throughput.value),
            "/min",
            metrics.job_throughput.health,
            metrics.job_throughput.trend,
        ));

        let content = lines.join("\n");
        format!("{header}\n{content}")
    }
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for DashboardPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Handle simulation ticks
        if msg.downcast_ref::<TickMsg>().is_some() {
            // Process tick and collect any notifications (for future toast display)
            let _notifications = self.process_tick();
            // Note: In a full implementation, notifications would be emitted via App-level
            // message routing. For now, the view reflects the live metric state changes.

            return Some(self.schedule_tick());
        }

        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && key.key_type == KeyType::Runes
            && key.runes.as_slice() == ['r']
        {
            self.refresh();
        }

        None
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        // Start the tick loop when entering the dashboard
        Some(self.schedule_tick())
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Left and right column widths
        let left_width = (width * 55) / 100;
        let right_width = width.saturating_sub(left_width + 1);

        // Render sections
        let status_bar = self.render_status_bar(theme, width);
        let stats_row = self.render_stats_row(theme, width);

        let services = self.render_services(theme, left_width);
        let deployments = self.render_deployments(theme, right_width);
        let jobs = self.render_jobs(theme, left_width);

        // Live metrics panel with trends and health indicators
        let live_metrics = self.render_live_metrics(theme, right_width);

        // Compose main content
        let left_col = format!("{services}\n\n{jobs}");
        let right_col = format!("{deployments}\n\n{live_metrics}");

        let main_content = lipgloss::join_horizontal(Position::Top, &[&left_col, " ", &right_col]);

        // Final layout
        let content = format!("{status_bar}\n\n{stats_row}\n\n{main_content}");

        // Place in available space (allow scrolling if needed)
        if height > 20 {
            lipgloss::place(width, height, Position::Left, Position::Top, &content)
        } else {
            content
        }
    }

    fn page(&self) -> Page {
        Page::Dashboard
    }

    fn hints(&self) -> &'static str {
        "r refresh  s services  j jobs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_creates_with_data() {
        let page = DashboardPage::new();
        assert!(!page.services().is_empty());
        assert!(!page.jobs().is_empty());
    }

    #[test]
    fn dashboard_deterministic() {
        let page1 = DashboardPage::with_seed(123);
        let page2 = DashboardPage::with_seed(123);

        assert_eq!(page1.services().len(), page2.services().len());
        for (s1, s2) in page1.services().iter().zip(page2.services().iter()) {
            assert_eq!(s1.name, s2.name);
        }
    }

    #[test]
    fn health_counts_correct() {
        let page = DashboardPage::new();
        let (healthy, degraded, unhealthy, unknown) = page.service_health_counts();
        assert_eq!(
            healthy + degraded + unhealthy + unknown,
            page.services().len()
        );
    }

    #[test]
    fn job_counts_correct() {
        let page = DashboardPage::new();
        let (queued, running, completed, failed) = page.job_status_counts();
        assert_eq!(queued + running + completed + failed, page.jobs().len());
    }

    #[test]
    fn uptime_format_days() {
        let mut page = DashboardPage::new();
        page.uptime_seconds = 86400 + 3600 + 60;
        assert_eq!(page.format_uptime(), "1d 1h 1m");
    }

    #[test]
    fn uptime_format_hours() {
        let mut page = DashboardPage::new();
        page.uptime_seconds = 3600 * 5 + 60 * 30;
        assert_eq!(page.format_uptime(), "5h 30m");
    }

    #[test]
    fn recent_deployments_limited() {
        let page = DashboardPage::new();
        let recent = page.recent_deployments();
        assert!(recent.len() <= 3);
    }

    #[test]
    fn recent_jobs_limited() {
        let page = DashboardPage::new();
        let recent = page.recent_jobs();
        assert!(recent.len() <= 4);
    }

    #[test]
    fn simulation_tick_advances() {
        let mut page = DashboardPage::new();
        let initial_frame = page.simulation.frame();

        // Process a tick
        page.process_tick();

        assert_eq!(page.simulation.frame(), initial_frame + 1);
    }

    #[test]
    fn uptime_increments_after_10_ticks() {
        let mut page = DashboardPage::new();
        let initial_uptime = page.uptime_seconds;

        // 9 ticks should not increment uptime
        for _ in 0..9 {
            page.process_tick();
        }
        assert_eq!(page.uptime_seconds, initial_uptime);

        // 10th tick should increment
        page.process_tick();
        assert_eq!(page.uptime_seconds, initial_uptime + 1);
    }

    #[test]
    fn live_metrics_have_values() {
        let page = DashboardPage::new();
        let metrics = &page.simulation.metrics;

        // All metrics should have positive initial values
        assert!(metrics.requests_per_sec.value > 0.0);
        assert!(metrics.p95_latency_ms.value > 0.0);
        assert!(metrics.error_rate.value >= 0.0);
        assert!(metrics.job_throughput.value >= 0.0);
    }

    #[test]
    fn animator_initialized_with_metric_values() {
        let page = DashboardPage::new();
        let metrics = &page.simulation.metrics;

        // Animator should start with the same values as the simulation
        let animated_rps = page.animator.get("requests_per_sec").unwrap();
        assert!((animated_rps - metrics.requests_per_sec.value).abs() < 0.001);

        let animated_latency = page.animator.get("p95_latency_ms").unwrap();
        assert!((animated_latency - metrics.p95_latency_ms.value).abs() < 0.001);
    }

    #[test]
    fn animations_disabled_snaps_values() {
        let mut page = DashboardPage::with_options(42, false); // animations disabled

        // Process multiple ticks to change metric values
        for _ in 0..20 {
            page.process_tick();
        }

        // With animations disabled, animated value should match simulation exactly
        let sim_rps = page.simulation.metrics.requests_per_sec.value;
        let animated_rps = page.animator.get("requests_per_sec").unwrap();
        assert!((animated_rps - sim_rps).abs() < 0.001);
    }

    #[test]
    fn animations_enabled_tracks_metrics() {
        let mut page = DashboardPage::with_options(42, true); // animations enabled

        // Process a few ticks to change the simulation value
        for _ in 0..5 {
            page.process_tick();
        }

        // All metrics should have animated values tracked
        assert!(page.animator.get("requests_per_sec").is_some());
        assert!(page.animator.get("p95_latency_ms").is_some());
        assert!(page.animator.get("error_rate").is_some());
        assert!(page.animator.get("job_throughput").is_some());
    }

    #[test]
    fn refresh_reinitializes_animator() {
        let mut page = DashboardPage::new();

        // Process some ticks
        for _ in 0..10 {
            page.process_tick();
        }

        // Refresh should re-initialize the animator
        page.refresh();

        // After refresh, animated values should match simulation
        let sim_rps = page.simulation.metrics.requests_per_sec.value;
        let animated_rps = page.animator.get("requests_per_sec").unwrap();
        assert!((animated_rps - sim_rps).abs() < 0.001);
    }
}
