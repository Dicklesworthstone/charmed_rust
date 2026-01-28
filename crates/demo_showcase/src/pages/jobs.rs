//! Jobs page - background task monitoring with progress tracking.
//!
//! This page displays a table of jobs with keyboard navigation and selection,
//! along with a details pane showing job info, parameters, timeline, and logs.
//!
//! # Filtering & Sorting
//!
//! The page provides:
//! - **Query bar**: TextInput for instant name/ID filtering
//! - **Status filters**: Toggle chips for Running/Completed/Failed/Queued
//! - **Sorting**: Click column headers or use `s` to cycle sort order
//!
//! Filtering maintains a `filtered_indices` vector for O(1) row access
//! without rebuilding heavy row structs on each keystroke.

use bubbles::table::{Column, Row, Styles, Table};
use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::Style;

use super::PageModel;
use crate::data::generator::GeneratedData;
use crate::data::{Job, JobKind, JobStatus, LogEntry, LogLevel, LogStream};
use crate::messages::Page;
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

// =============================================================================
// Filtering & Sorting
// =============================================================================

/// Status filter state - which statuses to show.
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusFilter {
    /// Show running jobs.
    pub running: bool,
    /// Show completed jobs.
    pub completed: bool,
    /// Show failed jobs.
    pub failed: bool,
    /// Show queued jobs.
    pub queued: bool,
}

impl StatusFilter {
    /// Create a filter that shows all statuses.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            running: true,
            completed: true,
            failed: true,
            queued: true,
        }
    }

    /// Check if all filters are enabled.
    #[must_use]
    pub const fn all_enabled(&self) -> bool {
        self.running && self.completed && self.failed && self.queued
    }

    /// Check if no filters are enabled.
    #[must_use]
    pub const fn none_enabled(&self) -> bool {
        !self.running && !self.completed && !self.failed && !self.queued
    }

    /// Toggle a specific status filter.
    pub fn toggle(&mut self, status: JobStatus) {
        match status {
            JobStatus::Running => self.running = !self.running,
            JobStatus::Completed => self.completed = !self.completed,
            JobStatus::Failed | JobStatus::Cancelled => self.failed = !self.failed,
            JobStatus::Queued => self.queued = !self.queued,
        }
    }

    /// Check if a job status passes the filter.
    #[must_use]
    pub const fn matches(&self, status: JobStatus) -> bool {
        match status {
            JobStatus::Running => self.running,
            JobStatus::Completed => self.completed,
            JobStatus::Failed | JobStatus::Cancelled => self.failed,
            JobStatus::Queued => self.queued,
        }
    }
}

/// Sort column for jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    /// Sort by start time (default).
    #[default]
    StartTime,
    /// Sort by job name.
    Name,
    /// Sort by status.
    Status,
    /// Sort by progress.
    Progress,
}

impl SortColumn {
    /// Get the display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::StartTime => "time",
            Self::Name => "name",
            Self::Status => "status",
            Self::Progress => "progress",
        }
    }

    /// Cycle to the next sort column.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::StartTime => Self::Name,
            Self::Name => Self::Status,
            Self::Status => Self::Progress,
            Self::Progress => Self::StartTime,
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Ascending order (default).
    #[default]
    Ascending,
    /// Descending order.
    Descending,
}

impl SortDirection {
    /// Toggle direction.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Get the arrow icon.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Ascending => "↑",
            Self::Descending => "↓",
        }
    }
}

/// Focus state for the jobs page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JobsFocus {
    /// Table is focused (default).
    #[default]
    Table,
    /// Query input is focused.
    QueryInput,
}

/// Jobs page showing background task monitoring.
pub struct JobsPage {
    /// The jobs table component.
    table: Table,
    /// The jobs data.
    jobs: Vec<Job>,
    /// Filtered job indices (indices into `jobs`).
    filtered_indices: Vec<usize>,
    /// Current seed for data generation.
    seed: u64,
    /// Log stream with job-correlated entries.
    logs: LogStream,
    /// Scroll offset for details pane.
    details_scroll: usize,
    /// Query input for filtering.
    query_input: TextInput,
    /// Current query text (cached for filtering).
    query: String,
    /// Status filter state.
    status_filter: StatusFilter,
    /// Current sort column.
    sort_column: SortColumn,
    /// Current sort direction.
    sort_direction: SortDirection,
    /// Current focus state.
    focus: JobsFocus,
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

        // Initialize filtered indices to all jobs
        let filtered_indices: Vec<usize> = (0..jobs.len()).collect();

        let rows = Self::indices_to_rows(&jobs, &filtered_indices);

        let table = Table::new()
            .columns(columns)
            .rows(rows)
            .height(20)
            .focused(true);

        // Generate synthetic logs correlated with jobs
        let logs = Self::generate_job_logs(&jobs, seed);

        // Create query input
        let mut query_input = TextInput::new();
        query_input.set_placeholder("Filter jobs... (/ to focus)");
        query_input.width = 40;

        Self {
            table,
            jobs,
            filtered_indices,
            seed,
            logs,
            details_scroll: 0,
            query_input,
            query: String::new(),
            status_filter: StatusFilter::all(),
            sort_column: SortColumn::StartTime,
            sort_direction: SortDirection::Descending, // Most recent first
            focus: JobsFocus::Table,
        }
    }

    // =========================================================================
    // Filtering & Sorting
    // =========================================================================

    /// Apply current filters and sorting, updating filtered_indices.
    fn apply_filter_and_sort(&mut self) {
        let query_lower = self.query.to_lowercase();

        // Build filtered indices
        self.filtered_indices = self
            .jobs
            .iter()
            .enumerate()
            .filter(|(_, job)| {
                // Status filter
                if !self.status_filter.matches(job.status) {
                    return false;
                }

                // Query filter (match name or ID)
                if !query_lower.is_empty() {
                    let name_match = job.name.to_lowercase().contains(&query_lower);
                    let id_match = format!("#{}", job.id).contains(&query_lower);
                    if !name_match && !id_match {
                        return false;
                    }
                }

                true
            })
            .map(|(i, _)| i)
            .collect();

        // Sort filtered indices
        self.sort_filtered_indices();

        // Update table rows
        self.update_table_rows();
    }

    /// Sort the filtered indices based on current sort settings.
    fn sort_filtered_indices(&mut self) {
        let jobs = &self.jobs;
        let sort_column = self.sort_column;
        let ascending = self.sort_direction == SortDirection::Ascending;

        self.filtered_indices.sort_by(|&a, &b| {
            let job_a = &jobs[a];
            let job_b = &jobs[b];

            let cmp = match sort_column {
                SortColumn::StartTime => job_a.started_at.cmp(&job_b.started_at),
                SortColumn::Name => job_a.name.cmp(&job_b.name),
                SortColumn::Status => {
                    // Sort by status priority: Running > Queued > Completed > Failed
                    let priority = |s: JobStatus| -> u8 {
                        match s {
                            JobStatus::Running => 0,
                            JobStatus::Queued => 1,
                            JobStatus::Completed => 2,
                            JobStatus::Failed | JobStatus::Cancelled => 3,
                        }
                    };
                    priority(job_a.status).cmp(&priority(job_b.status))
                }
                SortColumn::Progress => job_a.progress.cmp(&job_b.progress),
            };

            if ascending { cmp } else { cmp.reverse() }
        });
    }

    /// Update table rows from filtered indices.
    fn update_table_rows(&mut self) {
        let rows = Self::indices_to_rows(&self.jobs, &self.filtered_indices);
        self.table.set_rows(rows);
    }

    /// Convert filtered indices to table rows.
    fn indices_to_rows(jobs: &[Job], indices: &[usize]) -> Vec<Row> {
        indices
            .iter()
            .filter_map(|&i| jobs.get(i))
            .map(Self::job_to_row)
            .collect()
    }

    /// Toggle a status filter and reapply.
    fn toggle_status_filter(&mut self, status: JobStatus) {
        self.status_filter.toggle(status);
        self.apply_filter_and_sort();
    }

    /// Cycle to next sort column.
    fn cycle_sort_column(&mut self) {
        self.sort_column = self.sort_column.next();
        self.apply_filter_and_sort();
    }

    /// Toggle sort direction.
    fn toggle_sort_direction(&mut self) {
        self.sort_direction = self.sort_direction.toggle();
        self.apply_filter_and_sort();
    }

    /// Clear all filters.
    fn clear_filters(&mut self) {
        self.query.clear();
        self.query_input.set_value("");
        self.status_filter = StatusFilter::all();
        self.apply_filter_and_sort();
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

    /// Get the currently selected job (using filtered indices).
    #[must_use]
    pub fn selected_job(&self) -> Option<&Job> {
        // Map table cursor to filtered index, then to actual job
        self.filtered_indices
            .get(self.table.cursor())
            .and_then(|&i| self.jobs.get(i))
    }

    /// Refresh data with the current seed, preserving filters.
    pub fn refresh(&mut self) {
        let data = GeneratedData::generate(self.seed);
        self.jobs = data.jobs;
        self.logs = Self::generate_job_logs(&self.jobs, self.seed);
        self.details_scroll = 0;
        // Reapply current filters
        self.apply_filter_and_sort();
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

    /// Render the status summary bar with filter chips.
    fn render_status_bar(&self, theme: &Theme, _width: usize) -> String {
        let total = self.jobs.len();
        let filtered = self.filtered_indices.len();

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

        // Filter chips - show as [X] or [ ] based on active state
        let chip = |label: &str, count: usize, active: bool, style: Style| -> String {
            let prefix = if active { "[●]" } else { "[ ]" };
            let text = format!("{prefix} {label}:{count}");
            if active {
                style.render(&text)
            } else {
                theme.muted_style().render(&text)
            }
        };

        let running_chip = chip("R", running, self.status_filter.running, theme.info_style());
        let completed_chip = chip(
            "C",
            completed,
            self.status_filter.completed,
            theme.success_style(),
        );
        let failed_chip = chip("F", failed, self.status_filter.failed, theme.error_style());
        let queued_chip = chip("Q", queued, self.status_filter.queued, theme.muted_style());

        // Count display
        let count_display = if filtered == total {
            theme.muted_style().render(&format!("{total} jobs"))
        } else {
            theme
                .info_style()
                .render(&format!("{filtered}/{total} shown"))
        };

        // Sort indicator
        let sort_indicator = theme.muted_style().render(&format!(
            "Sort: {}{} ",
            self.sort_column.name(),
            self.sort_direction.icon()
        ));

        format!(
            "{count_display}  {running_chip} {completed_chip} {failed_chip} {queued_chip}  {sort_indicator}"
        )
    }

    /// Render the query bar.
    fn render_query_bar(&self, theme: &Theme, width: usize) -> String {
        // Style based on focus state
        let input_style = if self.focus == JobsFocus::QueryInput {
            theme.input_focused_style()
        } else {
            theme.input_style()
        };

        let label = if self.focus == JobsFocus::QueryInput {
            theme.info_style().render("Filter: ")
        } else {
            theme.muted_style().render("/ filter ")
        };

        let input_view = self.query_input.view();
        let input_styled = input_style.width(width.saturating_sub(12) as u16).render(&input_view);

        format!("{label}{input_styled}")
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
            // Handle query input focus
            if self.focus == JobsFocus::QueryInput {
                match key.key_type {
                    KeyType::Esc => {
                        // Exit query input, return to table
                        self.focus = JobsFocus::Table;
                        self.table.focus();
                        return None;
                    }
                    KeyType::Enter => {
                        // Apply filter and return to table
                        self.query = self.query_input.value().to_string();
                        self.apply_filter_and_sort();
                        self.focus = JobsFocus::Table;
                        self.table.focus();
                        return None;
                    }
                    _ => {
                        // Delegate to text input
                        let cmd = self.query_input.update(msg);
                        // Update filter on each keystroke for instant feedback
                        self.query = self.query_input.value().to_string();
                        self.apply_filter_and_sort();
                        return cmd;
                    }
                }
            }

            // Table focus mode
            match key.key_type {
                KeyType::Runes => match key.runes.as_slice() {
                    ['/'] => {
                        // Enter query input mode
                        self.focus = JobsFocus::QueryInput;
                        self.table.blur();
                        self.query_input.focus();
                        return None;
                    }
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
                    ['s'] => {
                        // Cycle sort column
                        self.cycle_sort_column();
                        return None;
                    }
                    ['S'] => {
                        // Toggle sort direction
                        self.toggle_sort_direction();
                        return None;
                    }
                    ['1'] => {
                        // Toggle running filter
                        self.toggle_status_filter(JobStatus::Running);
                        return None;
                    }
                    ['2'] => {
                        // Toggle completed filter
                        self.toggle_status_filter(JobStatus::Completed);
                        return None;
                    }
                    ['3'] => {
                        // Toggle failed filter
                        self.toggle_status_filter(JobStatus::Failed);
                        return None;
                    }
                    ['4'] => {
                        // Toggle queued filter
                        self.toggle_status_filter(JobStatus::Queued);
                        return None;
                    }
                    ['c'] => {
                        // Clear all filters
                        self.clear_filters();
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

        // Delegate to table for mouse events and other unhandled messages
        self.table.update(msg);
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let mut page = self.clone_for_render();
        page.apply_theme_styles(theme);
        page.table.set_width(width);

        // Calculate layout
        let query_bar_height = 1;
        let status_bar_height = 1;
        let details_height = 10;
        let table_height =
            height.saturating_sub(query_bar_height + status_bar_height + details_height + 4);

        page.table.set_height(table_height);

        // Render components
        let title = theme.title_style().render("Jobs");
        let query_bar = page.render_query_bar(theme, width);
        let status_bar = page.render_status_bar(theme, width);
        let table_view = page.table.view();
        let details = page.render_details(theme, width, details_height);

        // Compose
        format!("{title}\n{query_bar}\n{status_bar}\n\n{table_view}\n\n{details}")
    }

    fn page(&self) -> Page {
        Page::Jobs
    }

    fn hints(&self) -> &'static str {
        "/ filter  1-4 status  s sort  S reverse  c clear  j/k nav  r refresh"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        self.focus = JobsFocus::Table;
        self.table.focus();
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        self.table.blur();
        self.query_input.blur();
        None
    }
}

impl JobsPage {
    /// Clone the page for rendering (to apply theme styles without mutating self).
    fn clone_for_render(&self) -> Self {
        Self {
            table: self.table.clone(),
            jobs: self.jobs.clone(),
            filtered_indices: self.filtered_indices.clone(),
            seed: self.seed,
            logs: self.logs.clone(),
            details_scroll: self.details_scroll,
            query_input: self.query_input.clone(),
            query: self.query.clone(),
            status_filter: self.status_filter,
            sort_column: self.sort_column,
            sort_direction: self.sort_direction,
            focus: self.focus,
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
