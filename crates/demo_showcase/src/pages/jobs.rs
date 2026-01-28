//! Jobs page - background task monitoring with progress tracking.
//!
//! This page displays a table of jobs with keyboard navigation and selection.

use bubbles::table::{Column, Row, Styles, Table};
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use super::PageModel;
use crate::data::generator::GeneratedData;
use crate::data::{Job, JobStatus};
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

        Self { table, jobs, seed }
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
    fn render_details(&self, theme: &Theme, width: usize) -> String {
        let Some(job) = self.selected_job() else {
            return theme.muted_style().render("No job selected");
        };

        let status_style = match job.status {
            JobStatus::Completed => theme.success_style(),
            JobStatus::Running => theme.info_style(),
            JobStatus::Failed => theme.error_style(),
            JobStatus::Cancelled => theme.warning_style(),
            JobStatus::Queued => theme.muted_style(),
        };

        let title = theme.heading_style().render(&job.name);
        let status_line = format!(
            "{} {}",
            status_style.render(&format!("{} {}", job.status.icon(), job.status.name())),
            theme
                .muted_style()
                .render(&format!("({})", job.kind.name()))
        );

        let progress_bar = Self::render_progress_bar(job.progress, 20, theme);

        let created = format!("Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S"));
        let started = job.started_at.map_or_else(
            || "Started: —".to_string(),
            |t| format!("Started: {}", t.format("%Y-%m-%d %H:%M:%S")),
        );
        let ended = job.ended_at.map_or_else(
            || "Ended:   —".to_string(),
            |t| format!("Ended:   {}", t.format("%Y-%m-%d %H:%M:%S")),
        );

        let times = theme
            .muted_style()
            .render(&format!("{created}\n{started}\n{ended}"));

        let error_section = job.error.as_ref().map_or(String::new(), |e| {
            format!("\n{}", theme.error_style().render(&format!("Error: {e}")))
        });

        let content =
            format!("{title}\n{status_line}\n\nProgress: {progress_bar}\n\n{times}{error_section}");

        // Wrap in a box
        #[expect(clippy::cast_possible_truncation)]
        let box_width = width.min(50) as u16;

        theme
            .box_style()
            .width(box_width)
            .padding(1)
            .render(&content)
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
        let details = page.render_details(theme, width);

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
