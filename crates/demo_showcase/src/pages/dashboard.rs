//! Dashboard page - platform health overview.
//!
//! The dashboard provides an at-a-glance view of the platform's health,
//! showing key metrics, service status, recent deployments, and jobs.
//!
//! This page integrates with the simulation engine to provide live-updating
//! metrics with trends, health indicators, and notifications.

use std::sync::RwLock;
use std::time::Duration;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, MouseAction, MouseButton, MouseMsg, tick};
use lipgloss::{Position, Style};

use super::PageModel;
use crate::components::{
    DeltaDirection, StatusLevel, badge, chip, divider_with_label, stat_widget,
};
use crate::data::animation::Animator;
use crate::data::simulation::{MetricHealth, MetricTrend, SimConfig, Simulation, TickMsg};
use crate::data::{Deployment, DeploymentStatus, Job, JobStatus, Service, ServiceHealth};
use crate::messages::{AppMsg, Notification, NotificationMsg, Page};
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Tick interval for simulation updates (100ms = 10 fps).
const TICK_INTERVAL_MS: u64 = 100;

/// Dashboard cards that can be selected/clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardCard {
    /// No card selected.
    #[default]
    None,
    /// Services card (navigates to Services page).
    Services,
    /// Jobs card (navigates to Jobs page).
    Jobs,
    /// Deployments card.
    Deployments,
    /// Live metrics card.
    Metrics,
}

/// State for the drill-down details panel (bd-qkxb).
#[derive(Debug, Clone, Default)]
pub struct DetailsPanel {
    /// Whether the panel is open.
    pub open: bool,
    /// Which card's details are being shown.
    pub card: DashboardCard,
    /// Selected item index within the card (for lists).
    pub selected_index: usize,
}

/// SLA urgency level based on remaining time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SlaUrgency {
    /// Plenty of time remaining (> 1 hour).
    Normal,
    /// Getting close (15 min - 1 hour).
    Warning,
    /// Critical (< 15 minutes).
    Critical,
    /// SLA breached (0 or negative).
    Breached,
}

impl DashboardCard {
    /// Get the navigation target page for this card, if any.
    #[must_use]
    pub const fn target_page(self) -> Option<Page> {
        match self {
            Self::Services => Some(Page::Services),
            Self::Jobs => Some(Page::Jobs),
            Self::Deployments | Self::Metrics | Self::None => None,
        }
    }

    /// Cycle to the next card (for keyboard navigation).
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::Services,
            Self::Services => Self::Jobs,
            Self::Jobs => Self::Deployments,
            Self::Deployments => Self::Metrics,
            Self::Metrics => Self::Services,
        }
    }

    /// Cycle to the previous card.
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::None => Self::Metrics,
            Self::Services => Self::Metrics,
            Self::Jobs => Self::Services,
            Self::Deployments => Self::Jobs,
            Self::Metrics => Self::Deployments,
        }
    }
}

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
    /// Currently selected/focused card.
    selected_card: DashboardCard,
    /// Last rendered card bounds for hit testing (y_start, y_end, x_start, x_end).
    /// Uses RwLock for interior mutability since view() takes &self.
    card_bounds: RwLock<CardBounds>,
    /// Drill-down details panel state (bd-qkxb).
    details_panel: DetailsPanel,
    /// Incident SLA countdown in seconds (None = no active incident).
    /// Simulates an SLA timer that counts down. (bd-39hl)
    incident_sla_seconds: Option<u64>,
}

/// Bounds for dashboard cards used for mouse hit testing.
#[derive(Debug, Clone, Default)]
struct CardBounds {
    /// Services card bounds (y_start, y_end, x_start, x_end).
    services: Option<(usize, usize, usize, usize)>,
    /// Jobs card bounds.
    jobs: Option<(usize, usize, usize, usize)>,
    /// Deployments card bounds.
    deployments: Option<(usize, usize, usize, usize)>,
    /// Metrics card bounds.
    metrics: Option<(usize, usize, usize, usize)>,
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
            selected_card: DashboardCard::None,
            card_bounds: RwLock::new(CardBounds::default()),
            details_panel: DetailsPanel::default(),
            // Simulate an active incident with 45 minutes SLA remaining (bd-39hl)
            incident_sla_seconds: Some(45 * 60),
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

            // Decrement incident SLA countdown (bd-39hl)
            if let Some(sla) = self.incident_sla_seconds.as_mut() {
                *sla = sla.saturating_sub(1);
            }
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

    /// Format uptime as DD:HH:MM:SS stopwatch format (bd-39hl).
    fn format_uptime_stopwatch(&self) -> String {
        let days = self.uptime_seconds / 86400;
        let hours = (self.uptime_seconds % 86400) / 3600;
        let minutes = (self.uptime_seconds % 3600) / 60;
        let seconds = self.uptime_seconds % 60;

        format!("{days:02}:{hours:02}:{minutes:02}:{seconds:02}")
    }

    /// Format SLA countdown as HH:MM:SS (bd-39hl).
    fn format_sla_countdown(sla_seconds: u64) -> String {
        let hours = sla_seconds / 3600;
        let minutes = (sla_seconds % 3600) / 60;
        let seconds = sla_seconds % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes:02}:{seconds:02}")
        }
    }

    /// Determine SLA urgency level based on remaining time (bd-39hl).
    #[allow(dead_code)]
    fn sla_urgency(sla_seconds: u64) -> SlaUrgency {
        if sla_seconds == 0 {
            SlaUrgency::Breached
        } else if sla_seconds < 15 * 60 {
            // Less than 15 minutes
            SlaUrgency::Critical
        } else if sla_seconds < 60 * 60 {
            // Less than 1 hour
            SlaUrgency::Warning
        } else {
            SlaUrgency::Normal
        }
    }

    /// Get the incident SLA remaining seconds, if any.
    #[must_use]
    pub const fn incident_sla(&self) -> Option<u64> {
        self.incident_sla_seconds
    }

    /// Set the incident SLA countdown (for testing).
    #[allow(dead_code)]
    pub fn set_incident_sla(&mut self, seconds: Option<u64>) {
        self.incident_sla_seconds = seconds;
    }

    // ========================================================================
    // Mouse Interaction
    // ========================================================================

    /// Select the next card (keyboard navigation).
    fn select_next_card(&mut self) {
        self.selected_card = self.selected_card.next();
    }

    /// Select the previous card.
    fn select_prev_card(&mut self) {
        self.selected_card = self.selected_card.prev();
    }

    /// Handle a mouse click at the given position.
    ///
    /// Returns a command to navigate if a card was clicked.
    fn handle_click(&mut self, x: usize, y: usize) -> Option<Cmd> {
        // Check which card was clicked based on bounds
        let clicked_card = self.hit_test(x, y);

        if clicked_card != DashboardCard::None {
            self.selected_card = clicked_card;

            // If card has a navigation target, navigate to it on click
            if let Some(page) = clicked_card.target_page() {
                return Some(Cmd::new(move || AppMsg::Navigate(page).into_message()));
            }
        }

        None
    }

    /// Determine which card (if any) contains the given point.
    fn hit_test(&self, x: usize, y: usize) -> DashboardCard {
        let bounds = self.card_bounds.read().unwrap();

        // Check each card's bounds
        if let Some((y1, y2, x1, x2)) = bounds.services {
            if y >= y1 && y < y2 && x >= x1 && x < x2 {
                return DashboardCard::Services;
            }
        }
        if let Some((y1, y2, x1, x2)) = bounds.jobs {
            if y >= y1 && y < y2 && x >= x1 && x < x2 {
                return DashboardCard::Jobs;
            }
        }
        if let Some((y1, y2, x1, x2)) = bounds.deployments {
            if y >= y1 && y < y2 && x >= x1 && x < x2 {
                return DashboardCard::Deployments;
            }
        }
        if let Some((y1, y2, x1, x2)) = bounds.metrics {
            if y >= y1 && y < y2 && x >= x1 && x < x2 {
                return DashboardCard::Metrics;
            }
        }

        DashboardCard::None
    }

    /// Get the currently selected card.
    #[must_use]
    pub const fn selected_card(&self) -> DashboardCard {
        self.selected_card
    }

    /// Check if the details panel is open.
    #[must_use]
    pub const fn is_details_open(&self) -> bool {
        self.details_panel.open
    }

    /// Open the details panel for the currently selected card.
    fn open_details(&mut self) {
        if self.selected_card != DashboardCard::None {
            self.details_panel.open = true;
            self.details_panel.card = self.selected_card;
            self.details_panel.selected_index = 0;
        }
    }

    /// Close the details panel.
    fn close_details(&mut self) {
        self.details_panel.open = false;
    }

    /// Navigate to the next item in the details panel.
    fn details_next(&mut self) {
        let max = self.details_item_count();
        if max > 0 {
            self.details_panel.selected_index = (self.details_panel.selected_index + 1) % max;
        }
    }

    /// Navigate to the previous item in the details panel.
    fn details_prev(&mut self) {
        let max = self.details_item_count();
        if max > 0 {
            self.details_panel.selected_index = if self.details_panel.selected_index == 0 {
                max - 1
            } else {
                self.details_panel.selected_index - 1
            };
        }
    }

    /// Get the number of items in the current details panel.
    fn details_item_count(&self) -> usize {
        match self.details_panel.card {
            DashboardCard::Services => self.services().len().min(6),
            DashboardCard::Jobs => self.jobs().len().min(4),
            DashboardCard::Deployments => self.recent_deployments().len(),
            DashboardCard::Metrics => 4, // 4 metric types
            DashboardCard::None => 0,
        }
    }

    // ========================================================================
    // Render Helpers
    // ========================================================================

    /// Render a card section with selection highlighting.
    ///
    /// When the card is selected, applies a subtle highlight to indicate focus.
    fn render_card_section(
        &self,
        card: DashboardCard,
        content: &str,
        theme: &Theme,
        _width: usize,
    ) -> String {
        if self.selected_card == card {
            // Apply selection highlight: bold first line (header) and subtle left border
            let lines: Vec<&str> = content.lines().collect();
            if lines.is_empty() {
                return content.to_string();
            }

            // Highlight the header with primary/accent color
            let header = Style::new()
                .foreground(theme.primary)
                .bold()
                .render(lines[0]);

            // Add a subtle selection indicator
            let indicator = Style::new().foreground(theme.primary).render("▸ ");

            if lines.len() == 1 {
                format!("{indicator}{header}")
            } else {
                let rest = lines[1..].join("\n");
                format!("{indicator}{header}\n{rest}")
            }
        } else {
            // No selection, return as-is with spacing for alignment
            format!("  {}", content.replace('\n', "\n  "))
        }
    }

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

        // Uptime stopwatch (bd-39hl)
        let uptime = format!("⏱ {}", self.format_uptime_stopwatch());
        let uptime_styled = theme.muted_style().render(&uptime);

        // SLA countdown with visual emphasis (bd-39hl)
        let sla_styled = if let Some(sla_secs) = self.incident_sla_seconds {
            let sla_text = format!("SLA: {}", Self::format_sla_countdown(sla_secs));
            let urgency = Self::sla_urgency(sla_secs);
            match urgency {
                SlaUrgency::Breached => theme.error_style().bold().render(&format!("⚠ {sla_text}")),
                SlaUrgency::Critical => theme.error_style().render(&format!("⚠ {sla_text}")),
                SlaUrgency::Warning => theme.warning_style().render(&sla_text),
                SlaUrgency::Normal => theme.muted_style().render(&sla_text),
            }
        } else {
            String::new()
        };

        // Compose status bar
        let content = if sla_styled.is_empty() {
            format!("{platform_status}  {service_styled}  {uptime_styled}")
        } else {
            format!("{platform_status}  {service_styled}  {uptime_styled}  {sla_styled}")
        };

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

    /// Render the drill-down details panel (bd-qkxb).
    ///
    /// Shows a centered modal overlay with entity details, metrics, and actions.
    fn render_details_panel(&self, theme: &Theme, width: usize, height: usize) -> String {
        // Panel dimensions (centered, 70% width, 60% height)
        let panel_width = (width * 70 / 100).max(40).min(80);
        let panel_height = (height * 60 / 100).max(15).min(30);

        // Header based on card type
        let (title, icon) = match self.details_panel.card {
            DashboardCard::Services => ("Service Details", ""),
            DashboardCard::Jobs => ("Job Details", ""),
            DashboardCard::Deployments => ("Deployment Details", ""),
            DashboardCard::Metrics => ("Metric Details", ""),
            DashboardCard::None => return String::new(),
        };

        // Render content based on card type
        let content = match self.details_panel.card {
            DashboardCard::Services => self.render_service_details(theme, panel_width - 4),
            DashboardCard::Jobs => self.render_job_details(theme, panel_width - 4),
            DashboardCard::Deployments => self.render_deployment_details(theme, panel_width - 4),
            DashboardCard::Metrics => self.render_metric_details(theme, panel_width - 4),
            DashboardCard::None => return String::new(),
        };

        // Action hints
        let actions = match self.details_panel.card {
            DashboardCard::Services => "Enter: go to Services  j/k: navigate  Esc: close",
            DashboardCard::Jobs => "Enter: go to Jobs  j/k: navigate  Esc: close",
            DashboardCard::Deployments => "j/k: navigate  Esc: close",
            DashboardCard::Metrics => "j/k: navigate  Esc: close",
            DashboardCard::None => "",
        };

        // Build panel header
        let header_text = format!(" {icon} {title} ");
        let header = Style::new()
            .foreground(theme.primary)
            .bold()
            .render(&header_text);

        // Build action hints footer
        let actions_styled = theme.muted_style().render(actions);

        // Compose panel content
        let mut lines = vec![header.clone(), String::new()];
        lines.extend(content.lines().take(panel_height - 5).map(String::from));
        lines.push(String::new());
        lines.push(actions_styled);

        // Pad to panel height
        while lines.len() < panel_height {
            lines.insert(lines.len() - 1, String::new());
        }

        let panel_content = lines.join("\n");

        // Create modal box style
        let modal_box = theme
            .panel_style()
            .width(panel_width as u16)
            .height(panel_height as u16)
            .padding_left(2)
            .padding_right(2)
            .render(&panel_content);

        // Center the modal on screen
        lipgloss::place(
            width,
            height,
            lipgloss::Position::Center,
            lipgloss::Position::Center,
            &modal_box,
        )
    }

    /// Render detailed view for services.
    fn render_service_details(&self, theme: &Theme, width: usize) -> String {
        let services: Vec<_> = self.services().iter().take(6).collect();
        if services.is_empty() {
            return theme.muted_style().render("No services").to_string();
        }

        let selected = self
            .details_panel
            .selected_index
            .min(services.len().saturating_sub(1));
        let service = services[selected];

        let mut lines = Vec::new();

        // Service name and status
        let status_chip = match service.health {
            ServiceHealth::Healthy => chip(theme, StatusLevel::Success, "healthy"),
            ServiceHealth::Degraded => chip(theme, StatusLevel::Warning, "degraded"),
            ServiceHealth::Unhealthy => chip(theme, StatusLevel::Error, "unhealthy"),
            ServiceHealth::Unknown => chip(theme, StatusLevel::Info, "unknown"),
        };
        let name = Style::new()
            .foreground(theme.text)
            .bold()
            .render(&service.name);
        lines.push(format!("{name}  {status_chip}"));
        lines.push(String::new());

        // Service details
        lines.push(format!(
            "Version: {}",
            theme.muted_style().render(&service.version)
        ));
        lines.push(format!("Environments: {}", service.environment_count));
        if let Some(desc) = &service.description {
            lines.push(format!("Description: {}", theme.muted_style().render(desc)));
        }
        lines.push(String::new());

        // Service list (for navigation)
        lines.push(divider_with_label(theme, "All Services", width));
        for (i, svc) in services.iter().enumerate() {
            let marker = if i == selected { "▸ " } else { "  " };
            let health_icon = match svc.health {
                ServiceHealth::Healthy => theme.success_style().render("●"),
                ServiceHealth::Degraded => theme.warning_style().render("●"),
                ServiceHealth::Unhealthy => theme.error_style().render("●"),
                ServiceHealth::Unknown => theme.muted_style().render("○"),
            };
            let name_styled = if i == selected {
                Style::new().foreground(theme.primary).render(&svc.name)
            } else {
                Style::new().foreground(theme.text).render(&svc.name)
            };
            lines.push(format!("{marker}{health_icon} {name_styled}"));
        }

        lines.join("\n")
    }

    /// Render detailed view for jobs.
    fn render_job_details(&self, theme: &Theme, width: usize) -> String {
        let jobs = self.recent_jobs();
        if jobs.is_empty() {
            return theme.muted_style().render("No recent jobs").to_string();
        }

        let selected = self
            .details_panel
            .selected_index
            .min(jobs.len().saturating_sub(1));
        let job = jobs[selected];

        let mut lines = Vec::new();

        // Job name and status
        let status_chip = match job.status {
            JobStatus::Queued => chip(theme, StatusLevel::Info, "queued"),
            JobStatus::Running => chip(theme, StatusLevel::Running, "running"),
            JobStatus::Completed => chip(theme, StatusLevel::Success, "completed"),
            JobStatus::Failed | JobStatus::Cancelled => chip(theme, StatusLevel::Error, "failed"),
        };
        let name = Style::new().foreground(theme.text).bold().render(&job.name);
        lines.push(format!("{name}  {status_chip}"));
        lines.push(String::new());

        // Job details
        lines.push(format!("Progress: {}%", job.progress));
        lines.push(format!(
            "Kind: {}",
            theme.muted_style().render(&format!("{:?}", job.kind))
        ));
        lines.push(String::new());

        // Job list (for navigation)
        lines.push(divider_with_label(theme, "Recent Jobs", width));
        for (i, j) in jobs.iter().enumerate() {
            let marker = if i == selected { "▸ " } else { "  " };
            let status_icon = match j.status {
                JobStatus::Queued => theme.muted_style().render("○"),
                JobStatus::Running => theme.info_style().render("◐"),
                JobStatus::Completed => theme.success_style().render("●"),
                JobStatus::Failed | JobStatus::Cancelled => theme.error_style().render("●"),
            };
            let name_styled = if i == selected {
                Style::new().foreground(theme.primary).render(&j.name)
            } else {
                Style::new().foreground(theme.text).render(&j.name)
            };
            lines.push(format!("{marker}{status_icon} {name_styled}"));
        }

        lines.join("\n")
    }

    /// Render detailed view for deployments.
    fn render_deployment_details(&self, theme: &Theme, width: usize) -> String {
        let deployments = self.recent_deployments();
        if deployments.is_empty() {
            return theme
                .muted_style()
                .render("No recent deployments")
                .to_string();
        }

        let selected = self
            .details_panel
            .selected_index
            .min(deployments.len().saturating_sub(1));
        let deploy = deployments[selected];

        let mut lines = Vec::new();

        // Deployment SHA and status
        let status_chip = match deploy.status {
            DeploymentStatus::Pending => chip(theme, StatusLevel::Info, "pending"),
            DeploymentStatus::InProgress => chip(theme, StatusLevel::Running, "deploying"),
            DeploymentStatus::Succeeded => chip(theme, StatusLevel::Success, "success"),
            DeploymentStatus::Failed => chip(theme, StatusLevel::Error, "failed"),
            DeploymentStatus::RolledBack => chip(theme, StatusLevel::Warning, "rolled back"),
        };
        let sha = Style::new()
            .foreground(theme.text)
            .bold()
            .render(&deploy.sha[..7.min(deploy.sha.len())]);
        lines.push(format!("{sha}  {status_chip}"));
        lines.push(String::new());

        // Deployment details
        lines.push(format!(
            "Author: {}",
            theme.muted_style().render(&deploy.author)
        ));
        lines.push(format!("SHA: {}", theme.muted_style().render(&deploy.sha)));
        lines.push(String::new());

        // Deployment list (for navigation)
        lines.push(divider_with_label(theme, "Recent Deployments", width));
        for (i, d) in deployments.iter().enumerate() {
            let marker = if i == selected { "▸ " } else { "  " };
            let status_icon = match d.status {
                DeploymentStatus::Pending => theme.muted_style().render("○"),
                DeploymentStatus::InProgress => theme.info_style().render("◐"),
                DeploymentStatus::Succeeded => theme.success_style().render("●"),
                DeploymentStatus::Failed => theme.error_style().render("●"),
                DeploymentStatus::RolledBack => theme.warning_style().render("◐"),
            };
            let sha_short = &d.sha[..7.min(d.sha.len())];
            let sha_styled = if i == selected {
                Style::new().foreground(theme.primary).render(sha_short)
            } else {
                theme.muted_style().render(sha_short)
            };
            lines.push(format!(
                "{marker}{status_icon} {sha_styled}  {}",
                theme.muted_style().render(&d.author)
            ));
        }

        lines.join("\n")
    }

    /// Render detailed view for metrics.
    fn render_metric_details(&self, theme: &Theme, _width: usize) -> String {
        let metrics = &self.simulation.metrics;
        let selected = self.details_panel.selected_index % 4;

        let metric_info = [
            (
                "Requests/sec",
                metrics.requests_per_sec.value,
                "",
                metrics.requests_per_sec.health,
                metrics.requests_per_sec.trend,
                "Request throughput",
            ),
            (
                "P95 Latency",
                metrics.p95_latency_ms.value,
                "ms",
                metrics.p95_latency_ms.health,
                metrics.p95_latency_ms.trend,
                "95th percentile response time",
            ),
            (
                "Error Rate",
                metrics.error_rate.value,
                "%",
                metrics.error_rate.health,
                metrics.error_rate.trend,
                "Percentage of failed requests",
            ),
            (
                "Job Throughput",
                metrics.job_throughput.value,
                "/min",
                metrics.job_throughput.health,
                metrics.job_throughput.trend,
                "Jobs completed per minute",
            ),
        ];

        let (name, value, unit, health, trend, description) = &metric_info[selected];

        let mut lines = Vec::new();

        // Metric name and health
        let health_chip = match health {
            MetricHealth::Ok => chip(theme, StatusLevel::Success, "ok"),
            MetricHealth::Warning => chip(theme, StatusLevel::Warning, "warning"),
            MetricHealth::Error => chip(theme, StatusLevel::Error, "error"),
        };
        let name_styled = Style::new().foreground(theme.text).bold().render(*name);
        lines.push(format!("{name_styled}  {health_chip}"));
        lines.push(String::new());

        // Metric value
        let value_str = if *value >= 100.0 {
            format!("{value:.0}{unit}")
        } else if *value >= 10.0 {
            format!("{value:.1}{unit}")
        } else {
            format!("{value:.2}{unit}")
        };
        lines.push(format!(
            "Value: {}",
            theme.heading_style().render(&value_str)
        ));
        lines.push(format!("Trend: {}", trend.icon()));
        lines.push(format!(
            "Description: {}",
            theme.muted_style().render(*description)
        ));
        lines.push(String::new());

        // Metric list (for navigation)
        lines.push("All Metrics".to_string());
        for (i, (m_name, _, _, m_health, _, _)) in metric_info.iter().enumerate() {
            let marker = if i == selected { "▸ " } else { "  " };
            let health_icon = match m_health {
                MetricHealth::Ok => theme.success_style().render("●"),
                MetricHealth::Warning => theme.warning_style().render("●"),
                MetricHealth::Error => theme.error_style().render("●"),
            };
            let name_styled = if i == selected {
                Style::new().foreground(theme.primary).render(*m_name)
            } else {
                Style::new().foreground(theme.text).render(*m_name)
            };
            lines.push(format!("{marker}{health_icon} {name_styled}"));
        }

        lines.join("\n")
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

        // Handle mouse input (bd-3d1w)
        if let Some(mouse) = msg.downcast_ref::<MouseMsg>() {
            // Only handle left button press (click)
            if mouse.button == MouseButton::Left && mouse.action == MouseAction::Press {
                return self.handle_click(mouse.x as usize, mouse.y as usize);
            }
        }

        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            // Handle details panel interactions first (bd-qkxb)
            if self.details_panel.open {
                match key.key_type {
                    KeyType::Esc => {
                        self.close_details();
                        return None;
                    }
                    KeyType::Runes => match key.runes.as_slice() {
                        ['j' | 'n'] => self.details_next(),
                        ['k' | 'p'] => self.details_prev(),
                        _ => {}
                    },
                    KeyType::Down => self.details_next(),
                    KeyType::Up => self.details_prev(),
                    KeyType::Enter => {
                        // Navigate to the full page for this card type
                        if let Some(page) = self.details_panel.card.target_page() {
                            self.close_details();
                            return Some(Cmd::new(move || AppMsg::Navigate(page).into_message()));
                        }
                    }
                    _ => {}
                }
                return None;
            }

            // Normal dashboard keyboard handling
            match key.key_type {
                KeyType::Runes => {
                    match key.runes.as_slice() {
                        ['r'] => self.refresh(),
                        // j to select next card
                        ['j'] => self.select_next_card(),
                        // k to select previous card
                        ['k'] => self.select_prev_card(),
                        _ => {}
                    }
                }
                // Tab cycles through cards
                KeyType::Tab => self.select_next_card(),
                // Arrow keys for card navigation
                KeyType::Down | KeyType::Right => self.select_next_card(),
                KeyType::Up | KeyType::Left => self.select_prev_card(),
                // Enter opens the details panel for the selected card (bd-qkxb)
                KeyType::Enter => {
                    if self.selected_card != DashboardCard::None {
                        self.open_details();
                    }
                }
                // Esc does nothing when details are closed
                KeyType::Esc => {}
                _ => {}
            }
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

        // Render card sections with selection highlighting
        let services = self.render_card_section(
            DashboardCard::Services,
            &self.render_services(theme, left_width),
            theme,
            left_width,
        );
        let deployments = self.render_card_section(
            DashboardCard::Deployments,
            &self.render_deployments(theme, right_width),
            theme,
            right_width,
        );
        let jobs = self.render_card_section(
            DashboardCard::Jobs,
            &self.render_jobs(theme, left_width),
            theme,
            left_width,
        );

        // Live metrics panel with trends and health indicators
        let live_metrics = self.render_card_section(
            DashboardCard::Metrics,
            &self.render_live_metrics(theme, right_width),
            theme,
            right_width,
        );

        // Compose main content
        let left_col = format!("{services}\n\n{jobs}");
        let right_col = format!("{deployments}\n\n{live_metrics}");

        let main_content = lipgloss::join_horizontal(Position::Top, &[&left_col, " ", &right_col]);

        // Calculate card bounds for mouse hit testing
        // Layout: status_bar (1) + blank (1) + stats_row + blank (1) + main_content
        let status_bar_height = 1;
        let stats_row_height = stats_row.lines().count();
        let header_lines = status_bar_height + 1 + stats_row_height + 1; // +1 for each blank line

        let services_lines = services.lines().count();
        let jobs_lines = jobs.lines().count();
        let deployments_lines = deployments.lines().count();
        let live_metrics_lines = live_metrics.lines().count();

        // Update card bounds (interior mutability via RwLock)
        if let Ok(mut bounds) = self.card_bounds.write() {
            // Services: left column, starts at header_lines
            bounds.services = Some((header_lines, header_lines + services_lines, 0, left_width));

            // Jobs: left column, after services + 2 blank lines
            let jobs_start = header_lines + services_lines + 2;
            bounds.jobs = Some((jobs_start, jobs_start + jobs_lines, 0, left_width));

            // Deployments: right column, starts at header_lines
            bounds.deployments = Some((
                header_lines,
                header_lines + deployments_lines,
                left_width + 1,
                width,
            ));

            // Live Metrics: right column, after deployments + 2 blank lines
            let metrics_start = header_lines + deployments_lines + 2;
            bounds.metrics = Some((
                metrics_start,
                metrics_start + live_metrics_lines,
                left_width + 1,
                width,
            ));
        }

        // Final layout
        let content = format!("{status_bar}\n\n{stats_row}\n\n{main_content}");

        // Place in available space (allow scrolling if needed)
        let base_view = if height > 20 {
            lipgloss::place(width, height, Position::Left, Position::Top, &content)
        } else {
            content
        };

        // Overlay details panel if open (bd-qkxb)
        if self.details_panel.open {
            self.render_details_panel(theme, width, height)
        } else {
            base_view
        }
    }

    fn page(&self) -> Page {
        Page::Dashboard
    }

    fn hints(&self) -> &'static str {
        if self.details_panel.open {
            "j/k nav items  Enter go to page  Esc close"
        } else {
            "r refresh  j/k nav  Tab cycle  Enter details"
        }
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
        use crate::content::format_uptime;
        assert_eq!(format_uptime(86400 + 3600 + 60), "1d 1h 1m");
    }

    #[test]
    fn uptime_format_hours() {
        use crate::content::format_uptime;
        assert_eq!(format_uptime(3600 * 5 + 60 * 30), "5h 30m");
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

    #[test]
    fn card_navigation_cycles() {
        let mut page = DashboardPage::new();
        assert_eq!(page.selected_card(), DashboardCard::None);

        // Next from None goes to Services
        page.select_next_card();
        assert_eq!(page.selected_card(), DashboardCard::Services);

        // Continue cycling forward
        page.select_next_card();
        assert_eq!(page.selected_card(), DashboardCard::Jobs);

        page.select_next_card();
        assert_eq!(page.selected_card(), DashboardCard::Deployments);

        page.select_next_card();
        assert_eq!(page.selected_card(), DashboardCard::Metrics);

        // Wraps back to Services
        page.select_next_card();
        assert_eq!(page.selected_card(), DashboardCard::Services);
    }

    #[test]
    fn card_navigation_prev() {
        let mut page = DashboardPage::new();

        // Prev from None goes to Metrics
        page.select_prev_card();
        assert_eq!(page.selected_card(), DashboardCard::Metrics);

        // Continue cycling backward
        page.select_prev_card();
        assert_eq!(page.selected_card(), DashboardCard::Deployments);

        page.select_prev_card();
        assert_eq!(page.selected_card(), DashboardCard::Jobs);

        page.select_prev_card();
        assert_eq!(page.selected_card(), DashboardCard::Services);
    }

    #[test]
    fn card_target_pages() {
        // Services and Jobs have target pages
        assert_eq!(
            DashboardCard::Services.target_page(),
            Some(crate::messages::Page::Services)
        );
        assert_eq!(
            DashboardCard::Jobs.target_page(),
            Some(crate::messages::Page::Jobs)
        );

        // Deployments, Metrics, and None don't navigate
        assert_eq!(DashboardCard::Deployments.target_page(), None);
        assert_eq!(DashboardCard::Metrics.target_page(), None);
        assert_eq!(DashboardCard::None.target_page(), None);
    }

    #[test]
    fn view_updates_card_bounds() {
        use crate::theme::Theme;

        let page = DashboardPage::new();
        let theme = Theme::default();

        // Render the view (this should update card_bounds)
        let _ = page.view(100, 40, &theme);

        // Card bounds should now be populated
        let bounds = page.card_bounds.read().unwrap();
        assert!(bounds.services.is_some(), "Services bounds should be set");
        assert!(bounds.jobs.is_some(), "Jobs bounds should be set");
        assert!(
            bounds.deployments.is_some(),
            "Deployments bounds should be set"
        );
        assert!(bounds.metrics.is_some(), "Metrics bounds should be set");
    }

    // =========================================================================
    // Details Panel Tests (bd-qkxb)
    // =========================================================================

    #[test]
    fn details_panel_starts_closed() {
        let page = DashboardPage::new();
        assert!(!page.is_details_open());
    }

    #[test]
    fn details_panel_opens_on_card_selection() {
        let mut page = DashboardPage::new();

        // Select a card first
        page.select_next_card(); // Now on Services
        assert_eq!(page.selected_card(), DashboardCard::Services);

        // Open details
        page.open_details();
        assert!(page.is_details_open());
        assert_eq!(page.details_panel.card, DashboardCard::Services);
    }

    #[test]
    fn details_panel_does_not_open_for_none() {
        let mut page = DashboardPage::new();
        assert_eq!(page.selected_card(), DashboardCard::None);

        // Try to open details with no card selected
        page.open_details();
        assert!(!page.is_details_open());
    }

    #[test]
    fn details_panel_closes() {
        let mut page = DashboardPage::new();
        page.select_next_card();
        page.open_details();
        assert!(page.is_details_open());

        page.close_details();
        assert!(!page.is_details_open());
    }

    #[test]
    fn details_panel_navigation() {
        let mut page = DashboardPage::new();
        page.select_next_card(); // Services
        page.open_details();

        let initial = page.details_panel.selected_index;
        page.details_next();
        let next = page.details_panel.selected_index;

        // Should have moved to next item
        assert_ne!(initial, next);

        page.details_prev();
        assert_eq!(page.details_panel.selected_index, initial);
    }

    #[test]
    fn details_item_count_by_card() {
        let page = DashboardPage::new();

        // Create a page with each card type selected
        let mut page_services = DashboardPage::new();
        page_services.details_panel.card = DashboardCard::Services;
        assert!(page_services.details_item_count() > 0);

        let mut page_jobs = DashboardPage::new();
        page_jobs.details_panel.card = DashboardCard::Jobs;
        assert!(page_jobs.details_item_count() > 0);

        let mut page_metrics = DashboardPage::new();
        page_metrics.details_panel.card = DashboardCard::Metrics;
        assert_eq!(page_metrics.details_item_count(), 4);

        // None card has 0 items
        assert_eq!(page.details_item_count(), 0);
    }

    #[test]
    fn details_renders_when_open() {
        use crate::theme::Theme;

        let mut page = DashboardPage::new();
        page.select_next_card(); // Services
        page.open_details();

        let theme = Theme::default();
        let view = page.view(100, 40, &theme);

        // View should contain something (not empty)
        assert!(!view.is_empty());
    }

    #[test]
    fn hints_change_when_details_open() {
        let mut page = DashboardPage::new();

        // Default hints
        let closed_hints = page.hints();
        assert!(closed_hints.contains("Enter"));

        // Open details
        page.select_next_card();
        page.open_details();
        let open_hints = page.hints();
        assert!(open_hints.contains("Esc"));
    }
}
