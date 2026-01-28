//! Dashboard page - platform health overview.
//!
//! The dashboard provides an at-a-glance view of the platform's health,
//! showing key metrics, service status, recent deployments, and jobs.

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::{Position, Style};

use super::PageModel;
use crate::components::{
    DeltaDirection, StatusLevel, badge, chip, divider_with_label, stat_widget,
};
use crate::data::generator::GeneratedData;
use crate::data::{Deployment, DeploymentStatus, Job, JobStatus, Service, ServiceHealth};
use crate::messages::Page;
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Dashboard page showing platform health overview.
pub struct DashboardPage {
    /// Services data.
    services: Vec<Service>,
    /// Deployments data.
    deployments: Vec<Deployment>,
    /// Jobs data.
    jobs: Vec<Job>,
    /// Current seed for data generation.
    seed: u64,
    /// Simulated uptime in seconds.
    uptime_seconds: u64,
}

impl DashboardPage {
    /// Create a new dashboard page.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    /// Create a new dashboard page with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let data = GeneratedData::generate(seed);

        Self {
            services: data.services,
            deployments: data.deployments,
            jobs: data.jobs,
            seed,
            uptime_seconds: 86400 * 7 + 3600 * 5 + 60 * 23, // 7d 5h 23m
        }
    }

    /// Refresh data with the current seed.
    pub fn refresh(&mut self) {
        let data = GeneratedData::generate(self.seed);
        self.services = data.services;
        self.deployments = data.deployments;
        self.jobs = data.jobs;
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

        for service in &self.services {
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

        for job in &self.jobs {
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
        let mut sorted: Vec<_> = self.deployments.iter().collect();
        sorted.sort_by_key(|d| std::cmp::Reverse(d.created_at));
        sorted.into_iter().take(3).collect()
    }

    /// Get recent jobs (last 4).
    fn recent_jobs(&self) -> Vec<&Job> {
        let mut sorted: Vec<_> = self.jobs.iter().collect();
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
        let total = self.services.len();

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
            .deployments
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
            &format!("{healthy}/{}", self.services.len()),
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

        for service in self.services.iter().take(6) {
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

    /// Render a simple sparkline chart.
    fn render_sparkline(theme: &Theme, label: &str, values: &[u8], width: usize) -> String {
        let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max = *values.iter().max().unwrap_or(&100);

        let chart: String = values
            .iter()
            .take(width.saturating_sub(label.len() + 3))
            .map(|&v| {
                let idx = if max > 0 {
                    (usize::from(v) * 7) / usize::from(max)
                } else {
                    0
                };
                blocks[idx.min(7)]
            })
            .collect();

        let chart_styled = theme.info_style().render(&chart);
        let label_styled = theme.muted_style().render(label);

        format!("{label_styled}: {chart_styled}")
    }
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for DashboardPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && key.key_type == KeyType::Runes
            && key.runes.as_slice() == ['r']
        {
            self.refresh();
        }
        None
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

        // Sample sparkline data (simulated request rate)
        let sparkline_data: Vec<u8> = (0..30)
            .map(|i| {
                let base = 50 + ((i * 7) % 30);
                let noise = (i * 13) % 15;
                (base + noise).min(100)
            })
            .collect();
        let sparkline = Self::render_sparkline(theme, "Requests/s", &sparkline_data, right_width);

        // Compose main content
        let left_col = format!("{services}\n\n{jobs}");
        let right_col = format!("{deployments}\n\n{sparkline}");

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
        assert!(!page.services.is_empty());
        assert!(!page.jobs.is_empty());
    }

    #[test]
    fn dashboard_deterministic() {
        let page1 = DashboardPage::with_seed(123);
        let page2 = DashboardPage::with_seed(123);

        assert_eq!(page1.services.len(), page2.services.len());
        for (s1, s2) in page1.services.iter().zip(page2.services.iter()) {
            assert_eq!(s1.name, s2.name);
        }
    }

    #[test]
    fn health_counts_correct() {
        let page = DashboardPage::new();
        let (healthy, degraded, unhealthy, unknown) = page.service_health_counts();
        assert_eq!(
            healthy + degraded + unhealthy + unknown,
            page.services.len()
        );
    }

    #[test]
    fn job_counts_correct() {
        let page = DashboardPage::new();
        let (queued, running, completed, failed) = page.job_status_counts();
        assert_eq!(queued + running + completed + failed, page.jobs.len());
    }

    #[test]
    fn uptime_format_days() {
        let page = DashboardPage {
            uptime_seconds: 86400 + 3600 + 60,
            ..DashboardPage::new()
        };
        assert_eq!(page.format_uptime(), "1d 1h 1m");
    }

    #[test]
    fn uptime_format_hours() {
        let page = DashboardPage {
            uptime_seconds: 3600 * 5 + 60 * 30,
            ..DashboardPage::new()
        };
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
}
