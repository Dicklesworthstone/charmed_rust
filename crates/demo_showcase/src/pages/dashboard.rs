//! Dashboard page - platform health overview.

use bubbletea::{Cmd, Message};
use lipgloss::Position;

use super::PageModel;
use crate::messages::Page;
use crate::theme::Theme;

/// Dashboard page showing platform health overview.
pub struct DashboardPage {
    /// Simulated service count.
    service_count: usize,
    /// Simulated healthy service count.
    healthy_count: usize,
    /// Simulated active job count.
    active_jobs: usize,
}

impl DashboardPage {
    /// Create a new dashboard page.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            service_count: 8,
            healthy_count: 7,
            active_jobs: 3,
        }
    }
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for DashboardPage {
    fn update(&mut self, _msg: &Message) -> Option<Cmd> {
        // Dashboard updates will be handled in future tasks
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let title = theme.title_style().render("Dashboard");

        let status_line = if self.healthy_count == self.service_count {
            theme
                .success_style()
                .render(&format!("All {} services healthy", self.service_count))
        } else {
            theme.warning_style().render(&format!(
                "{}/{} services healthy",
                self.healthy_count, self.service_count
            ))
        };

        let jobs_line = theme
            .muted_style()
            .render(&format!("{} active jobs", self.active_jobs));

        let metrics = [
            ("CPU", 45, theme.success_style()),
            ("Memory", 68, theme.warning_style()),
            ("Network", 12, theme.success_style()),
        ];

        let metrics_display: Vec<String> = metrics
            .iter()
            .map(|(name, value, style)| {
                let bar_width = 20;
                let filled = (value * bar_width) / 100;
                let empty = bar_width - filled;
                let bar = format!("[{}{}] {}%", "#".repeat(filled), "-".repeat(empty), value);
                format!("{:>8}: {}", name, style.render(&bar))
            })
            .collect();

        let content = format!(
            "{}\n\n{}\n{}\n\n{}\n\n{}",
            title,
            status_line,
            jobs_line,
            "Resource Usage:",
            metrics_display.join("\n")
        );

        let boxed = theme.box_style().padding(1).render(&content);

        lipgloss::place(width, height, Position::Center, Position::Center, &boxed)
    }

    fn page(&self) -> Page {
        Page::Dashboard
    }

    fn hints(&self) -> &'static str {
        "r refresh  s services  j jobs"
    }
}
