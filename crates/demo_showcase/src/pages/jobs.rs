//! Jobs page - background task monitoring with progress tracking.
//!
//! This page displays a table of jobs with keyboard navigation and selection,
//! along with a details pane showing job info, parameters, timeline, and logs.

use bubbles::table::{Column, Row, Styles, Table};
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use super::PageModel;
use crate::data::generator::GeneratedData;
use crate::data::{Job, JobKind, JobStatus, LogEntry, LogLevel, LogStream};
use crate::messages::Page;
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Jobs page showing background task monitoring.
pub struct JobsPage {
    /// The jobs table component.
    table: Table,
    /// The jobs data.
    jobs: Vec<Job>,
    /// Current seed for data generation.
    seed: u64,
    /// Log stream with job-correlated entries.
    logs: LogStream,
    /// Scroll offset for details pane.
    details_scroll: usize,
}

impl JobsPage {
    /// Create a new jobs page.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    /// Create a new jobs page with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let data = GeneratedData::generate(seed);
        let jobs = data.jobs;

        let columns = vec![
            Column::new("ID", 6),
            Column::new("Name", 28),
            Column::new("Kind", 10),
            Column::new("Status", 12),
            Column::new("Progress", 10),
            Column::new("Started", 12),
        ];

        let rows = Self::jobs_to_rows(&jobs);

        let table = Table::new()
            .columns(columns)
            .rows(rows)
            .height(20)
            .focused(true);

        // Generate synthetic logs correlated with jobs
        let logs = Self::generate_job_logs(&jobs, seed);

        Self {
            table,
            jobs,
            seed,
            logs,
            details_scroll: 0,
        }
    }

    /// Generate synthetic log entries correlated with jobs.
    fn generate_job_logs(jobs: &[Job], seed: u64) -> LogStream {
        use rand::prelude::*;
        use rand_pcg::Pcg64;

        let mut rng = Pcg64::seed_from_u64(seed.wrapping_add(12345));
        let mut logs = LogStream::new(200);

        let messages = [
            "Job initialized",
            "Starting execution",
            "Processing batch",
            "Checkpoint saved",
            "Progress updated",
            "Resource acquired",
            "Step completed",
            "Validation passed",
            "Data processed",
            "Finalizing",
        ];

        for job in jobs {
            // Generate 2-8 log entries per job
            let entry_count = rng.random_range(2..=8);
            for i in 0..entry_count {
                let level = if i == 0 {
                    LogLevel::Info
                } else if rng.random_ratio(1, 10) {
                    LogLevel::Warn
                } else if rng.random_ratio(1, 20) {
                    LogLevel::Error
                } else {
                    LogLevel::Info
                };

                let msg_idx = rng.random_range(0..messages.len());
                let message = format!("{} (step {})", messages[msg_idx], i + 1);
                let target = format!("job::{}", job.kind.name().to_lowercase());

                #[expect(
                    clippy::cast_sign_loss,
                    reason = "i is always non-negative from 0..entry_count"
                )]
                let tick = i as u64;
                let mut entry = LogEntry::new(logs.len() as u64 + 1, level, target, message)
                    .with_job_id(job.id)
                    .with_tick(tick);

                // Set timestamp relative to job start
                if let Some(started) = job.started_at {
                    entry.timestamp = started + chrono::Duration::seconds(i64::from(i) * 5);
                }

                logs.push(entry);
            }
        }

        logs
    }

    /// Convert jobs to table rows.
    fn jobs_to_rows(jobs: &[Job]) -> Vec<Row> {
        jobs.iter().map(Self::job_to_row).collect()
    }

    /// Convert a single job to a table row.
    fn job_to_row(job: &Job) -> Row {
        let id_str = format!("#{}", job.id);
        let kind_str = job.kind.name().to_string();
        let status_str = format!("{} {}", job.status.icon(), job.status.name());
        let progress_str = format!("{}%", job.progress);
        let started_str = job
            .started_at
            .map_or_else(|| "—".to_string(), |t| t.format("%H:%M:%S").to_string());

        vec![
            id_str,
            job.name.clone(),
            kind_str,
            status_str,
            progress_str,
            started_str,
        ]
    }

    /// Get the currently selected job.
    #[must_use]
    pub fn selected_job(&self) -> Option<&Job> {
        self.jobs.get(self.table.cursor())
    }

    /// Refresh data with the current seed.
    pub fn refresh(&mut self) {
        let data = GeneratedData::generate(self.seed);
        self.jobs = data.jobs;
        let rows = Self::jobs_to_rows(&self.jobs);
        self.table.set_rows(rows);
        self.logs = Self::generate_job_logs(&self.jobs, self.seed);
        self.details_scroll = 0;
    }

    /// Apply theme-aware styles to the table.
    fn apply_theme_styles(&mut self, theme: &Theme) {
        let styles = Styles {
            header: Style::new()
                .bold()
                .foreground(theme.text)
                .background(theme.bg_subtle)
                .padding_left(1)
                .padding_right(1),
            cell: Style::new()
                .foreground(theme.text)
                .padding_left(1)
                .padding_right(1),
            selected: Style::new()
                .bold()
                .foreground(theme.primary)
                .background(theme.bg_highlight),
        };
        self.table = std::mem::take(&mut self.table).with_styles(styles);
    }

    /// Render the status summary bar.
    fn render_status_bar(&self, theme: &Theme, width: usize) -> String {
        let total = self.jobs.len();
        let running = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Running)
            .count();
        let completed = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .count();
        let failed = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Failed)
            .count();
        let queued = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Queued)
            .count();

        let running_s = theme.info_style().render(&format!("{running} running"));
        let completed_s = theme
            .success_style()
            .render(&format!("{completed} completed"));
        let failed_s = if failed > 0 {
            theme.error_style().render(&format!("{failed} failed"))
        } else {
            theme.muted_style().render("0 failed")
        };
        let queued_s = theme.muted_style().render(&format!("{queued} queued"));

        let summary = format!("{total} jobs: {running_s}  {completed_s}  {failed_s}  {queued_s}");

        // Truncate if too wide
        if summary.len() > width {
            summary
                .chars()
                .take(width.saturating_sub(3))
                .collect::<String>()
                + "..."
        } else {
            summary
        }
    }

    /// Render the details pane for the selected job.
    fn render_details(&self, theme: &Theme, width: usize, height: usize) -> String {
        let Some(job) = self.selected_job() else {
            return theme.muted_style().render("  No job selected");
        };

        let mut lines: Vec<String> = Vec::new();
        let content_width = width.saturating_sub(4);

        // === HEADER ===
        let status_style = Self::status_style(job.status, theme);
        let title = theme.heading_style().render(&job.name);
        let status_badge =
            status_style.render(&format!(" {} {} ", job.status.icon(), job.status.name()));
        lines.push(format!("{title}  {status_badge}"));
        lines.push(String::new());

        // === SUMMARY ===
        lines.push(theme.heading_style().render("Summary"));
        let duration = Self::calculate_duration(job);
        let progress_bar = Self::render_progress_bar(job.progress, 20, theme);
        lines.push(format!(
            "  Kind:     {}",
            theme.muted_style().render(job.kind.name())
        ));
        lines.push(format!("  Progress: {progress_bar}"));
        lines.push(format!(
            "  Duration: {}",
            theme.muted_style().render(&duration)
        ));
        if let Some(ref error) = job.error {
            lines.push(format!("  Error:    {}", theme.error_style().render(error)));
        }
        lines.push(String::new());

        // === PARAMETERS ===
        lines.push(theme.heading_style().render("Parameters"));
        for (key, value) in Self::derive_parameters(job) {
            let key_styled = theme.muted_style().render(&format!("{key:>12}"));
            lines.push(format!("  {key_styled}  {value}"));
        }
        lines.push(String::new());

        // === TIMELINE ===
        lines.push(theme.heading_style().render("Timeline"));
        for line in Self::render_timeline(job, theme) {
            lines.push(format!("  {line}"));
        }
        lines.push(String::new());

        // === LOGS ===
        lines.push(theme.heading_style().render("Logs"));
        let job_logs: Vec<_> = self.logs.filter_by_job(job.id).collect();
        if job_logs.is_empty() {
            lines.push(format!(
                "  {}",
                theme.muted_style().render("No logs available")
            ));
        } else {
            let display_logs: Vec<_> = job_logs.iter().rev().take(5).collect();
            for entry in display_logs.into_iter().rev() {
                let level_style = match entry.level {
                    LogLevel::Error => theme.error_style(),
                    LogLevel::Warn => theme.warning_style(),
                    LogLevel::Info => theme.info_style(),
                    _ => theme.muted_style(),
                };
                let level_str = level_style.render(entry.level.abbrev());
                let time_str = entry.timestamp.format("%H:%M:%S");
                let msg = if entry.message.len() > content_width.saturating_sub(20) {
                    format!("{}...", &entry.message[..content_width.saturating_sub(23)])
                } else {
                    entry.message.clone()
                };
                lines.push(format!("  {time_str} {level_str} {msg}"));
            }
            if job_logs.len() > 5 {
                lines.push(format!(
                    "  {}",
                    theme
                        .muted_style()
                        .render(&format!("... and {} more entries", job_logs.len() - 5))
                ));
            }
        }

        // Apply height limiting
        let visible_height = height.saturating_sub(1);
        let visible: Vec<&String> = lines.iter().take(visible_height).collect();
        visible
            .iter()
            .map(|s| (*s).clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render a simple progress bar.
    fn render_progress_bar(percent: u8, width: usize, theme: &Theme) -> String {
        let clamped = percent.min(100);
        let fill_width = (usize::from(clamped) * width) / 100;
        let empty_width = width.saturating_sub(fill_width);

        let fill = "#".repeat(fill_width);
        let empty = "-".repeat(empty_width);

        let bar = format!("[{fill}{empty}] {clamped}%");

        if clamped >= 100 {
            theme.success_style().render(&bar)
        } else if clamped > 0 {
            theme.info_style().render(&bar)
        } else {
            theme.muted_style().render(&bar)
        }
    }

    /// Get style for job status.
    fn status_style(status: JobStatus, theme: &Theme) -> Style {
        match status {
            JobStatus::Completed => theme.success_style(),
            JobStatus::Running => theme.info_style(),
            JobStatus::Failed => theme.error_style(),
            JobStatus::Cancelled => theme.warning_style(),
            JobStatus::Queued => theme.muted_style(),
        }
    }

    /// Calculate job duration as a human-readable string.
    fn calculate_duration(job: &Job) -> String {
        let (start, end) = match (job.started_at, job.ended_at) {
            (Some(s), Some(e)) => (s, e),
            (Some(s), None) => (s, chrono::Utc::now()),
            _ => return "—".to_string(),
        };

        let duration = end.signed_duration_since(start);
        let secs = duration.num_seconds();

        if secs < 60 {
            format!("{secs}s")
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// Derive synthetic parameters from job data.
    fn derive_parameters(job: &Job) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();

        let env = match job.id % 4 {
            0 => "production",
            1 => "staging",
            2 => "development",
            _ => "qa",
        };
        params.push(("Environment", env.to_string()));

        let target = match job.kind {
            JobKind::Backup => "database-primary",
            JobKind::Migration => "schema-manager",
            JobKind::Build => "ci-pipeline",
            JobKind::Test => "test-runner",
            JobKind::Cron => "scheduler",
            JobKind::Task => "worker-pool",
        };
        params.push(("Target", target.to_string()));

        let actors = ["alice", "bob", "carol", "system", "scheduler"];
        #[allow(clippy::cast_possible_truncation)] // Safe: modulo keeps result in bounds
        let actor = actors[(job.id as usize) % actors.len()];
        params.push(("Actor", actor.to_string()));

        let priority = match job.kind {
            JobKind::Backup | JobKind::Migration => "high",
            JobKind::Build | JobKind::Test => "normal",
            _ => "low",
        };
        params.push(("Priority", priority.to_string()));

        params
    }

    /// Render the job timeline.
    fn render_timeline(job: &Job, theme: &Theme) -> Vec<String> {
        let mut lines = Vec::new();
        let check = "●";
        let pending = "○";
        let current = "◐";

        let created_time = job.created_at.format("%H:%M:%S").to_string();
        let created_icon = theme.success_style().render(check);
        lines.push(format!(
            "{created_icon} Created     {}",
            theme.muted_style().render(&created_time)
        ));

        let (started_icon, started_time) = job.started_at.map_or_else(
            || (theme.muted_style().render(pending), "—".to_string()),
            |t| {
                let icon = if job.status == JobStatus::Running {
                    theme.info_style().render(current)
                } else {
                    theme.success_style().render(check)
                };
                (icon, t.format("%H:%M:%S").to_string())
            },
        );
        lines.push(format!(
            "{started_icon} Started     {}",
            theme.muted_style().render(&started_time)
        ));

        let (end_icon, end_label, end_time) = match (job.status, job.ended_at) {
            (JobStatus::Completed, Some(t)) => (
                theme.success_style().render(check),
                "Completed",
                t.format("%H:%M:%S").to_string(),
            ),
            (JobStatus::Failed, Some(t)) => (
                theme.error_style().render("✕"),
                "Failed",
                t.format("%H:%M:%S").to_string(),
            ),
            (JobStatus::Cancelled, Some(t)) => (
                theme.warning_style().render("⊘"),
                "Cancelled",
                t.format("%H:%M:%S").to_string(),
            ),
            (JobStatus::Running, _) => (
                theme.muted_style().render(pending),
                "Running...",
                "—".to_string(),
            ),
            _ => (
                theme.muted_style().render(pending),
                "Pending",
                "—".to_string(),
            ),
        };
        lines.push(format!(
            "{end_icon} {end_label:<11} {}",
            theme.muted_style().render(&end_time)
        ));

        lines
    }
}

impl Default for JobsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for JobsPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => match key.runes.as_slice() {
                    ['r'] => {
                        self.refresh();
                        return None;
                    }
                    ['j'] => {
                        self.table.move_down(1);
                        return None;
                    }
                    ['k'] => {
                        self.table.move_up(1);
                        return None;
                    }
                    ['g'] => {
                        self.table.goto_top();
                        return None;
                    }
                    ['G'] => {
                        self.table.goto_bottom();
                        return None;
                    }
                    _ => {}
                },
                KeyType::Up => {
                    self.table.move_up(1);
                    return None;
                }
                KeyType::Down => {
                    self.table.move_down(1);
                    return None;
                }
                KeyType::Home => {
                    self.table.goto_top();
                    return None;
                }
                KeyType::End => {
                    self.table.goto_bottom();
                    return None;
                }
                KeyType::PgUp => {
                    self.table.move_up(self.table.get_height());
                    return None;
                }
                KeyType::PgDown => {
                    self.table.move_down(self.table.get_height());
                    return None;
                }
                _ => {}
            }
        }

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let mut page = self.clone_for_render();
        page.apply_theme_styles(theme);
        page.table.set_width(width);

        // Calculate layout
        let status_bar_height = 1;
        let details_height = 10;
        let table_height = height.saturating_sub(status_bar_height + details_height + 2);

        page.table.set_height(table_height);

        // Render components
        let title = theme.title_style().render("Jobs");
        let status_bar = page.render_status_bar(theme, width);
        let table_view = page.table.view();
        let details = page.render_details(theme, width, details_height);

        // Compose
        format!("{title}  {status_bar}\n\n{table_view}\n\n{details}")
    }

    fn page(&self) -> Page {
        Page::Jobs
    }

    fn hints(&self) -> &'static str {
        "j/k navigate  Enter details  r refresh"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        self.table.focus();
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        self.table.blur();
        None
    }
}

impl JobsPage {
    /// Clone the page for rendering (to apply theme styles without mutating self).
    fn clone_for_render(&self) -> Self {
        Self {
            table: self.table.clone(),
            jobs: self.jobs.clone(),
            seed: self.seed,
            logs: self.logs.clone(),
            details_scroll: self.details_scroll,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_page_creates_with_data() {
        let page = JobsPage::new();
        assert!(!page.jobs.is_empty());
        assert_eq!(page.jobs.len(), 20); // Default from generator
    }

    #[test]
    fn jobs_page_deterministic() {
        let page1 = JobsPage::with_seed(123);
        let page2 = JobsPage::with_seed(123);

        assert_eq!(page1.jobs.len(), page2.jobs.len());
        for (j1, j2) in page1.jobs.iter().zip(page2.jobs.iter()) {
            assert_eq!(j1.name, j2.name);
            assert_eq!(j1.status, j2.status);
        }
    }

    #[test]
    fn jobs_page_different_seeds_differ() {
        let page1 = JobsPage::with_seed(1);
        let page2 = JobsPage::with_seed(2);

        // At least some jobs should differ
        let names1: Vec<_> = page1.jobs.iter().map(|j| &j.name).collect();
        let names2: Vec<_> = page2.jobs.iter().map(|j| &j.name).collect();
        assert_ne!(names1, names2);
    }

    #[test]
    fn selected_job_works() {
        let page = JobsPage::new();
        assert!(page.selected_job().is_some());
    }

    #[test]
    fn refresh_regenerates_data() {
        let mut page = JobsPage::with_seed(42);
        let original_first = page.jobs[0].name.clone();

        // Refresh with same seed should produce same data
        page.refresh();
        assert_eq!(page.jobs[0].name, original_first);
    }

    #[test]
    fn job_to_row_format() {
        let data = GeneratedData::generate_minimal(1);
        let job = &data.jobs[0];
        let row = JobsPage::job_to_row(job);

        assert_eq!(row.len(), 6);
        assert!(row[0].starts_with('#')); // ID
        assert!(!row[1].is_empty()); // Name
        assert!(!row[3].is_empty()); // Status with icon
        assert!(row[4].ends_with('%')); // Progress
    }
}
