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

use bubbles::paginator::Paginator;
use bubbles::table::{Column, Row, Styles, Table};
use bubbles::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, println};
use lipgloss::Style;

use super::PageModel;
use crate::data::actions::{
    ActionResult, IdGenerator, NotificationSeverity, cancel_job, create_job, retry_job, start_job,
};
use crate::data::generator::GeneratedData;
use crate::data::{Job, JobKind, JobStatus, LogEntry, LogLevel, LogStream};
use crate::messages::{Notification, NotificationMsg, Page};
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
    #[allow(dead_code)]
    pub const fn all_enabled(&self) -> bool {
        self.running && self.completed && self.failed && self.queued
    }

    /// Check if no filters are enabled.
    #[must_use]
    #[allow(dead_code)]
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

/// Default items per page for pagination.
const DEFAULT_ITEMS_PER_PAGE: usize = 10;

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
    /// ID generator for creating new jobs and log entries.
    id_gen: IdGenerator,
    /// Paginator for navigating through pages of jobs.
    paginator: Paginator,
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

        // Initialize ID generator starting after the highest existing ID
        let max_id = jobs.iter().map(|j| j.id).max().unwrap_or(0);
        let id_gen = IdGenerator::new(max_id + 100);

        // Initialize paginator
        let mut paginator = Paginator::new().per_page(DEFAULT_ITEMS_PER_PAGE);
        paginator.set_total_pages_from_items(filtered_indices.len());

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
            id_gen,
            paginator,
        }
    }

    // =========================================================================
    // Filtering & Sorting
    // =========================================================================

    /// Apply current filters and sorting, updating filtered_indices.
    fn apply_filter_and_sort(&mut self) {
        // Save current selection to restore after filter (best-effort)
        let prev_selected_job_id = self.selected_job().map(|j| j.id);

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

        // Update paginator total pages
        self.paginator
            .set_total_pages_from_items(self.filtered_indices.len());

        // Try to restore selection (or reset to page containing it)
        if let Some(job_id) = prev_selected_job_id {
            self.try_restore_selection(job_id);
        }

        // Update table rows
        self.update_table_rows();
    }

    /// Try to restore selection to a specific job ID after filtering.
    /// Adjusts the page if the job is on a different page.
    fn try_restore_selection(&mut self, job_id: u64) {
        // Find the position of the job in filtered_indices
        if let Some(pos) = self
            .filtered_indices
            .iter()
            .position(|&idx| self.jobs[idx].id == job_id)
        {
            // Calculate which page this job is on
            let per_page = self.paginator.get_per_page();
            let target_page = pos / per_page;
            self.paginator.set_page(target_page);

            // Set table cursor to the position within the page
            let _pos_in_page = pos % per_page;
            // We'll set the cursor after update_table_rows is called
            // For now, just ensure the page is correct
        }
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

    /// Update table rows from filtered indices, respecting pagination.
    fn update_table_rows(&mut self) {
        // Get the slice bounds for the current page
        let (start, end) = self.paginator.get_slice_bounds(self.filtered_indices.len());
        let page_indices = &self.filtered_indices[start..end];

        let rows = Self::indices_to_rows(&self.jobs, page_indices);
        let row_count = rows.len();
        self.table.set_rows(rows);

        // Clamp table cursor to valid range
        if self.table.cursor() >= row_count && row_count > 0 {
            self.table.goto_top();
        }
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

        // Enhanced progress display based on status
        let progress_str = Self::format_progress_cell(job);

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

    /// Format the progress cell based on job status.
    /// Uses spinners for indeterminate states and progress bars for determinate.
    fn format_progress_cell(job: &Job) -> String {
        match job.status {
            JobStatus::Queued => {
                // Indeterminate: show spinner-like indicator
                "◌ queued".to_string()
            }
            JobStatus::Running => {
                // Determinate with progress and ETA hint
                let eta = Self::estimate_eta(job);
                if let Some(eta_str) = eta {
                    format!("{}% {}", job.progress, eta_str)
                } else {
                    format!("{}%", job.progress)
                }
            }
            JobStatus::Completed => "✓ done".to_string(),
            JobStatus::Failed => "✕ error".to_string(),
            JobStatus::Cancelled => "⊘ cancel".to_string(),
        }
    }

    /// Estimate remaining time for a running job.
    /// Returns a short string like "~2m" or None if can't estimate.
    fn estimate_eta(job: &Job) -> Option<String> {
        if job.status != JobStatus::Running || job.progress == 0 {
            return None;
        }

        let started = job.started_at?;
        let elapsed = chrono::Utc::now().signed_duration_since(started);
        let elapsed_secs = elapsed.num_seconds();

        if elapsed_secs <= 0 {
            return None;
        }

        // Estimate total time based on progress rate
        #[allow(clippy::cast_precision_loss)]
        let rate = job.progress as f64 / elapsed_secs as f64; // percent per second

        if rate <= 0.0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let remaining_percent = (100 - job.progress) as f64;
        let eta_secs = (remaining_percent / rate) as i64;

        if eta_secs < 60 {
            Some(format!("~{}s", eta_secs))
        } else if eta_secs < 3600 {
            Some(format!("~{}m", eta_secs / 60))
        } else {
            Some(format!("~{}h", eta_secs / 3600))
        }
    }

    /// Check if a running job appears to be slow/stuck.
    /// Returns true if progress is < 10% after 30+ seconds.
    fn is_job_slow(job: &Job) -> bool {
        if job.status != JobStatus::Running {
            return false;
        }

        let Some(started) = job.started_at else {
            return false;
        };

        let elapsed = chrono::Utc::now().signed_duration_since(started);

        // Job is "slow" if running for 30+ seconds with <10% progress
        elapsed.num_seconds() >= 30 && job.progress < 10
    }

    /// Get the currently selected job (using filtered indices and pagination).
    #[must_use]
    pub fn selected_job(&self) -> Option<&Job> {
        // Map table cursor to global filtered index, then to actual job
        let (start, _) = self.paginator.get_slice_bounds(self.filtered_indices.len());
        let global_idx = start + self.table.cursor();
        self.filtered_indices
            .get(global_idx)
            .and_then(|&i| self.jobs.get(i))
    }

    /// Refresh data with the current seed, preserving filters.
    pub fn refresh(&mut self) {
        let data = GeneratedData::generate(self.seed);
        self.jobs = data.jobs;
        self.logs = Self::generate_job_logs(&self.jobs, self.seed);
        self.details_scroll = 0;
        // Reinitialize ID generator
        let max_id = self.jobs.iter().map(|j| j.id).max().unwrap_or(0);
        self.id_gen.set_next(max_id + 100);
        // Reapply current filters
        self.apply_filter_and_sort();
        // Reset to first page AFTER filter/sort (to override any selection restoration)
        self.paginator.set_page(0);
        self.update_table_rows();
    }

    // =========================================================================
    // Job Actions (bd-1x3q)
    // =========================================================================

    /// Get the index of the currently selected job in the jobs vector.
    fn selected_job_index(&self) -> Option<usize> {
        let (start, _) = self.paginator.get_slice_bounds(self.filtered_indices.len());
        let global_idx = start + self.table.cursor();
        self.filtered_indices.get(global_idx).copied()
    }

    /// Create a new job with a random kind and add it to the list.
    ///
    /// Returns a command to display a notification.
    fn action_create_job(&mut self) -> Option<Cmd> {
        use rand::prelude::*;
        use rand_pcg::Pcg64;

        // Generate a random job kind and name
        let mut rng = Pcg64::seed_from_u64(self.id_gen.peek().wrapping_mul(31337));
        let kinds = [
            JobKind::Build,
            JobKind::Test,
            JobKind::Migration,
            JobKind::Backup,
            JobKind::Cron,
            JobKind::Task,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];

        let names = [
            "Deploy frontend",
            "Run test suite",
            "Migrate database",
            "Backup snapshots",
            "Sync data",
            "Process queue",
            "Generate report",
            "Validate configs",
        ];
        let name = names[rng.random_range(0..names.len())];

        let (job, result) = create_job(&mut self.id_gen, name, kind);
        self.jobs.push(job);
        self.apply_filter_and_sort();

        // Scroll to show the new job (it might be at the bottom or top depending on sort)
        self.table.goto_bottom();

        Self::action_result_to_cmd(result)
    }

    /// Start the currently selected job if it's queued.
    fn action_start_job(&mut self) -> Option<Cmd> {
        let idx = self.selected_job_index()?;
        let job = self.jobs.get_mut(idx)?;

        let result = start_job(job, &mut self.id_gen)?;
        self.update_table_rows();

        Self::action_result_to_cmd(result)
    }

    /// Cancel the currently selected job if it's running or queued.
    fn action_cancel_job(&mut self) -> Option<Cmd> {
        let idx = self.selected_job_index()?;
        let job = self.jobs.get_mut(idx)?;

        let result = cancel_job(job, &mut self.id_gen)?;
        self.update_table_rows();

        Self::action_result_to_cmd(result)
    }

    /// Retry the currently selected job if it's failed or cancelled.
    fn action_retry_job(&mut self) -> Option<Cmd> {
        let idx = self.selected_job_index()?;
        let job = self.jobs.get_mut(idx)?;

        let result = retry_job(job, &mut self.id_gen)?;
        self.update_table_rows();

        Self::action_result_to_cmd(result)
    }

    /// Convert an `ActionResult` to a command that shows notifications.
    fn action_result_to_cmd(result: ActionResult) -> Option<Cmd> {
        if result.notifications.is_empty() {
            return None;
        }

        // Convert action notifications to app notifications via messages
        let cmds: Vec<Option<Cmd>> = result
            .notifications
            .into_iter()
            .flat_map(|notif| {
                let level = match notif.severity {
                    NotificationSeverity::Info => crate::components::StatusLevel::Info,
                    NotificationSeverity::Success => crate::components::StatusLevel::Success,
                    NotificationSeverity::Warning => crate::components::StatusLevel::Warning,
                    NotificationSeverity::Error => crate::components::StatusLevel::Error,
                };

                let message = if let Some(ref detail) = notif.message {
                    format!("{}: {}", notif.title, detail)
                } else {
                    notif.title.clone()
                };

                // Create a notification message
                let notification = Notification::new(0, message.clone(), level);
                let notification_cmd = Some(Cmd::new(move || {
                    NotificationMsg::Show(notification).into_message()
                }));

                // bd-7iul: Emit println for job lifecycle events (visible in no-alt-screen mode)
                // Only emit for Info/Success (job started, completed) to avoid noise
                let println_cmd = match notif.severity {
                    NotificationSeverity::Info | NotificationSeverity::Success => {
                        Some(println(format!("[job] {}", notif.title)))
                    }
                    _ => None,
                };

                [notification_cmd, println_cmd]
            })
            .collect();

        bubbletea::batch(cmds)
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
            "Sort: {}{}",
            self.sort_column.name(),
            self.sort_direction.icon()
        ));

        // Paginator display
        let page_display = if self.paginator.get_total_pages() > 1 {
            format!(
                "  {}",
                theme.info_style().render(&format!(
                    "Page {}/{}",
                    self.paginator.page() + 1,
                    self.paginator.get_total_pages()
                ))
            )
        } else {
            String::new()
        };

        format!(
            "{count_display}  {running_chip} {completed_chip} {failed_chip} {queued_chip}  {sort_indicator}{page_display}"
        )
    }

    /// Render the query bar.
    fn render_query_bar(&self, theme: &Theme, _width: usize) -> String {
        let label = if self.focus == JobsFocus::QueryInput {
            theme.info_style().render("Filter: ")
        } else {
            theme.muted_style().render("/ filter ")
        };

        let input_view = self.query_input.view();

        format!("{label}{input_view}")
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
        let inline_progress = Self::render_inline_progress(job, theme);
        lines.push(format!(
            "  Kind:     {}",
            theme.muted_style().render(job.kind.name())
        ));
        lines.push(format!("  Progress: {inline_progress}"));
        lines.push(format!(
            "  Duration: {}",
            theme.muted_style().render(&duration)
        ));

        // Show slow warning for running jobs
        if Self::is_job_slow(job) {
            lines.push(format!(
                "  Status:   {}",
                theme
                    .warning_style()
                    .render("⚠ Job may be stuck or experiencing delays")
            ));
        }

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

    /// Render a polished progress bar with Unicode characters.
    fn render_progress_bar(percent: u8, width: usize, theme: &Theme) -> String {
        let clamped = percent.min(100);
        let bar_width = width.saturating_sub(5); // Reserve space for percentage

        if bar_width == 0 {
            return format!("{clamped}%");
        }

        let fill_width = (usize::from(clamped) * bar_width) / 100;
        let empty_width = bar_width.saturating_sub(fill_width);

        // Use Unicode block characters for a modern look
        let fill = "█".repeat(fill_width);
        let empty = "░".repeat(empty_width);

        let percent_str = format!("{clamped:>3}%");

        if clamped >= 100 {
            let bar = theme.success_style().render(&fill);
            format!("{bar} {}", theme.success_style().render(&percent_str))
        } else if clamped >= 75 {
            // Near complete - success tint
            let bar = format!(
                "{}{}",
                theme.success_style().render(&fill),
                theme.muted_style().render(&empty)
            );
            format!("{bar} {}", theme.info_style().render(&percent_str))
        } else if clamped > 0 {
            // In progress - info color
            let bar = format!(
                "{}{}",
                theme.info_style().render(&fill),
                theme.muted_style().render(&empty)
            );
            format!("{bar} {}", theme.muted_style().render(&percent_str))
        } else {
            // Not started - all empty
            let bar = theme.muted_style().render(&empty);
            format!("{bar} {}", theme.muted_style().render(&percent_str))
        }
    }

    /// Render a compact inline progress indicator for the details pane.
    fn render_inline_progress(job: &Job, theme: &Theme) -> String {
        match job.status {
            JobStatus::Queued => {
                format!(
                    "{} {}",
                    theme.muted_style().render("◌"),
                    theme.muted_style().render("Waiting in queue...")
                )
            }
            JobStatus::Running => {
                let is_slow = Self::is_job_slow(job);
                let progress_bar = Self::render_progress_bar(job.progress, 20, theme);

                if is_slow {
                    format!(
                        "{} {}",
                        progress_bar,
                        theme.warning_style().render("⚠ slow")
                    )
                } else if let Some(eta) = Self::estimate_eta(job) {
                    format!("{} {}", progress_bar, theme.muted_style().render(&eta))
                } else {
                    progress_bar
                }
            }
            JobStatus::Completed => {
                format!(
                    "{} {}",
                    theme.success_style().render("✓"),
                    theme.success_style().render("Completed successfully")
                )
            }
            JobStatus::Failed => {
                let error_hint = job.error.as_deref().unwrap_or("Unknown error");
                format!(
                    "{} {}",
                    theme.error_style().render("✕"),
                    theme.error_style().render(error_hint)
                )
            }
            JobStatus::Cancelled => {
                format!(
                    "{} {}",
                    theme.warning_style().render("⊘"),
                    theme.warning_style().render("Cancelled by user")
                )
            }
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
                        self.query_input.blur();
                        self.table.focus();
                        return None;
                    }
                    KeyType::Enter => {
                        // Apply filter and return to table
                        self.focus = JobsFocus::Table;
                        self.query_input.blur();
                        self.table.focus();
                        return None;
                    }
                    KeyType::Backspace => {
                        // Delete last character
                        self.query.pop();
                        self.query_input.set_value(&self.query);
                        self.apply_filter_and_sort();
                        return None;
                    }
                    KeyType::Runes => {
                        // Add typed characters
                        for c in &key.runes {
                            if c.is_alphanumeric()
                                || *c == '-'
                                || *c == '_'
                                || *c == ' '
                                || *c == '#'
                            {
                                self.query.push(*c);
                            }
                        }
                        self.query_input.set_value(&self.query);
                        self.apply_filter_and_sort();
                        return None;
                    }
                    _ => {
                        return None;
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
                    ['n'] | ['N'] => {
                        // Create a new job
                        return self.action_create_job();
                    }
                    ['x'] => {
                        // Cancel the selected job
                        return self.action_cancel_job();
                    }
                    ['R'] => {
                        // Retry the selected job
                        return self.action_retry_job();
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
                    ['['] | ['h'] => {
                        // Previous page
                        if !self.paginator.on_first_page() {
                            self.paginator.prev_page();
                            self.update_table_rows();
                        }
                        return None;
                    }
                    [']'] | ['l'] => {
                        // Next page
                        if !self.paginator.on_last_page() {
                            self.paginator.next_page();
                            self.update_table_rows();
                        }
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
                    // Navigate to previous page
                    if !self.paginator.on_first_page() {
                        self.paginator.prev_page();
                        self.update_table_rows();
                    }
                    return None;
                }
                KeyType::PgDown => {
                    // Navigate to next page
                    if !self.paginator.on_last_page() {
                        self.paginator.next_page();
                        self.update_table_rows();
                    }
                    return None;
                }
                KeyType::Enter => {
                    // Start the selected job if it's queued
                    return self.action_start_job();
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
        "n new  ⏎ start  x cancel  R retry  / filter  s sort  j/k nav  [/] page  r refresh"
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
            id_gen: self.id_gen.clone(),
            paginator: self.paginator.clone(),
        }
    }

    /// Get the current page number (0-indexed).
    #[must_use]
    pub fn current_page(&self) -> usize {
        self.paginator.page()
    }

    /// Get the total number of pages.
    #[must_use]
    pub fn total_pages(&self) -> usize {
        self.paginator.get_total_pages()
    }

    /// Navigate to the next page.
    pub fn next_page(&mut self) {
        if !self.paginator.on_last_page() {
            self.paginator.next_page();
            self.update_table_rows();
        }
    }

    /// Navigate to the previous page.
    pub fn prev_page(&mut self) {
        if !self.paginator.on_first_page() {
            self.paginator.prev_page();
            self.update_table_rows();
        }
    }

    /// Jump to a specific page (0-indexed).
    pub fn goto_page(&mut self, page: usize) {
        self.paginator.set_page(page);
        self.update_table_rows();
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
        // Progress column now shows status-aware format:
        // - Running: "50%" or "50% ~2m"
        // - Queued: "◌ queued"
        // - Completed: "✓ done"
        // - Failed: "✕ error"
        // - Cancelled: "⊘ cancel"
        assert!(!row[4].is_empty()); // Progress cell is not empty
    }

    // =========================================================================
    // Filtering Tests
    // =========================================================================

    #[test]
    fn initial_filter_shows_all() {
        let page = JobsPage::new();
        assert_eq!(page.filtered_indices.len(), page.jobs.len());
    }

    #[test]
    fn status_filter_reduces_count() {
        let mut page = JobsPage::new();

        // Disable completed filter
        page.status_filter.completed = false;
        page.apply_filter_and_sort();

        let completed_count = page
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Completed)
            .count();
        assert_eq!(
            page.filtered_indices.len(),
            page.jobs.len() - completed_count
        );
    }

    #[test]
    fn query_filter_matches_name() {
        let mut page = JobsPage::new();

        // Filter to jobs containing "backup"
        page.query = "backup".to_string();
        page.apply_filter_and_sort();

        // All filtered jobs should contain "backup" (case-insensitive)
        for &idx in &page.filtered_indices {
            assert!(page.jobs[idx].name.to_lowercase().contains("backup"));
        }
    }

    #[test]
    fn query_filter_matches_id() {
        let mut page = JobsPage::new();
        let first_job_id = page.jobs[0].id;

        // Filter by ID
        page.query = format!("#{}", first_job_id);
        page.apply_filter_and_sort();

        // Should find the job with that ID
        assert!(page.filtered_indices.contains(&0));
    }

    #[test]
    fn clear_filters_restores_all() {
        let mut page = JobsPage::new();
        let original_count = page.filtered_indices.len();

        // Apply some filters
        page.query = "nonexistent".to_string();
        page.status_filter.running = false;
        page.apply_filter_and_sort();
        assert!(page.filtered_indices.len() < original_count);

        // Clear and restore
        page.clear_filters();
        assert_eq!(page.filtered_indices.len(), original_count);
    }

    #[test]
    fn sort_column_cycles() {
        let col = SortColumn::StartTime;
        assert_eq!(col.next(), SortColumn::Name);
        assert_eq!(col.next().next(), SortColumn::Status);
        assert_eq!(col.next().next().next(), SortColumn::Progress);
        assert_eq!(col.next().next().next().next(), SortColumn::StartTime);
    }

    #[test]
    fn sort_direction_toggles() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn status_filter_toggle() {
        let mut filter = StatusFilter::all();
        assert!(filter.running);

        filter.toggle(JobStatus::Running);
        assert!(!filter.running);

        filter.toggle(JobStatus::Running);
        assert!(filter.running);
    }

    #[test]
    fn status_filter_matches_correctly() {
        let filter = StatusFilter {
            running: true,
            completed: false,
            failed: true,
            queued: false,
        };

        assert!(filter.matches(JobStatus::Running));
        assert!(!filter.matches(JobStatus::Completed));
        assert!(filter.matches(JobStatus::Failed));
        assert!(filter.matches(JobStatus::Cancelled)); // Grouped with Failed
        assert!(!filter.matches(JobStatus::Queued));
    }

    // =========================================================================
    // Edge Case Tests (for bd-3eru)
    // =========================================================================

    #[test]
    fn empty_query_shows_all_with_status_filter() {
        let mut page = JobsPage::new();
        let total = page.jobs.len();

        // Empty query should show all jobs (respecting status filter)
        page.query = String::new();
        page.apply_filter_and_sort();
        assert_eq!(page.filtered_indices.len(), total);
    }

    #[test]
    fn unicode_query_does_not_panic() {
        let mut page = JobsPage::new();

        // Unicode characters in query should not panic
        page.query = "日本語テスト".to_string();
        page.apply_filter_and_sort();
        // Should complete without panicking (likely no matches)
        assert!(page.filtered_indices.len() <= page.jobs.len());
    }

    #[test]
    fn emoji_query_does_not_panic() {
        let mut page = JobsPage::new();

        // Emoji in query should not panic
        page.query = "🚀 deployment 🎉".to_string();
        page.apply_filter_and_sort();
        // Should complete without panicking
        assert!(page.filtered_indices.len() <= page.jobs.len());
    }

    #[test]
    fn very_long_query_does_not_panic() {
        let mut page = JobsPage::new();

        // Very long query should not panic or cause memory issues
        page.query = "a".repeat(10_000);
        page.apply_filter_and_sort();
        // Should complete without panicking (likely no matches)
        assert!(page.filtered_indices.is_empty() || page.filtered_indices.len() <= page.jobs.len());
    }

    #[test]
    fn whitespace_only_query() {
        let mut page = JobsPage::new();
        let total = page.jobs.len();

        // Whitespace-only query
        page.query = "   ".to_string();
        page.apply_filter_and_sort();
        // Should not crash; current impl doesn't trim so this tests actual behavior
        assert!(page.filtered_indices.len() <= total);
    }

    #[test]
    fn newline_in_query_does_not_panic() {
        let mut page = JobsPage::new();

        // Paste-like input with newlines
        page.query = "backup\njob\ntest".to_string();
        page.apply_filter_and_sort();
        // Should complete without panicking
        assert!(page.filtered_indices.len() <= page.jobs.len());
    }

    #[test]
    fn filter_is_case_insensitive() {
        let mut page = JobsPage::new();

        // Same query in different cases should match same entries
        page.query = "BACKUP".to_string();
        page.apply_filter_and_sort();
        let upper_count = page.filtered_indices.len();

        page.query = "backup".to_string();
        page.apply_filter_and_sort();
        let lower_count = page.filtered_indices.len();

        page.query = "Backup".to_string();
        page.apply_filter_and_sort();
        let mixed_count = page.filtered_indices.len();

        assert_eq!(
            upper_count, lower_count,
            "Case should not affect match count"
        );
        assert_eq!(
            lower_count, mixed_count,
            "Case should not affect match count"
        );
    }

    #[test]
    fn filter_is_idempotent() {
        let mut page = JobsPage::new();

        page.query = "backup".to_string();
        page.apply_filter_and_sort();
        let first_result = page.filtered_indices.clone();

        // Apply again - should get same result
        page.apply_filter_and_sort();
        let second_result = page.filtered_indices.clone();

        assert_eq!(first_result, second_result, "Filter should be idempotent");
    }

    #[test]
    fn sorting_is_stable() {
        let mut page = JobsPage::new();

        // Sort by name ascending
        page.sort_column = SortColumn::Name;
        page.sort_direction = SortDirection::Ascending;
        page.apply_filter_and_sort();
        let first_sort = page.filtered_indices.clone();

        // Sort again - should get same order
        page.apply_filter_and_sort();
        let second_sort = page.filtered_indices.clone();

        assert_eq!(first_sort, second_sort, "Sorting should be stable");
    }

    #[test]
    fn combined_filters_compose() {
        let mut page = JobsPage::new();

        // Disable all status filters
        page.status_filter = StatusFilter {
            running: false,
            completed: false,
            failed: false,
            queued: false,
        };
        page.apply_filter_and_sort();

        // With no status enabled, should show no jobs
        assert!(
            page.filtered_indices.is_empty(),
            "No status enabled should show no jobs"
        );

        // Enable one status
        page.status_filter.running = true;
        page.apply_filter_and_sort();

        // Now add a query that further restricts
        let running_count = page.filtered_indices.len();
        page.query = "xyzzy_nonexistent".to_string();
        page.apply_filter_and_sort();

        // Query should further restrict (unless it matches all running jobs)
        assert!(page.filtered_indices.len() <= running_count);
    }

    // =========================================================================
    // Job Actions Tests (bd-1x3q)
    // =========================================================================

    #[test]
    fn action_create_job_adds_new_job() {
        let mut page = JobsPage::new();
        let initial_count = page.jobs.len();

        let cmd = page.action_create_job();

        // A new job should be added
        assert_eq!(page.jobs.len(), initial_count + 1);
        // The new job should be queued
        assert_eq!(page.jobs.last().unwrap().status, JobStatus::Queued);
        // Should return a command (for showing notification)
        assert!(cmd.is_some());
    }

    #[test]
    fn action_start_job_starts_queued_job() {
        let mut page = JobsPage::new();

        // Find a queued job
        let queued_idx = page.jobs.iter().position(|j| j.status == JobStatus::Queued);

        if let Some(idx) = queued_idx {
            // Navigate to the queued job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_start_job();

                // Job should now be running
                assert_eq!(page.jobs[idx].status, JobStatus::Running);
                // started_at should be set
                assert!(page.jobs[idx].started_at.is_some());
                // Should return a notification command
                assert!(cmd.is_some());
            }
        }
    }

    #[test]
    fn action_start_job_returns_none_if_not_queued() {
        let mut page = JobsPage::new();

        // Find a running job
        let running_idx = page
            .jobs
            .iter()
            .position(|j| j.status == JobStatus::Running);

        if let Some(idx) = running_idx {
            // Navigate to the running job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_start_job();
                // Should return None because job is already running
                assert!(cmd.is_none());
            }
        }
    }

    #[test]
    fn action_cancel_job_cancels_running_job() {
        let mut page = JobsPage::new();

        // Find a running job
        let running_idx = page
            .jobs
            .iter()
            .position(|j| j.status == JobStatus::Running);

        if let Some(idx) = running_idx {
            // Navigate to the running job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_cancel_job();

                // Job should now be cancelled
                assert_eq!(page.jobs[idx].status, JobStatus::Cancelled);
                // ended_at should be set
                assert!(page.jobs[idx].ended_at.is_some());
                // Should return a notification command
                assert!(cmd.is_some());
            }
        }
    }

    #[test]
    fn action_cancel_job_returns_none_if_already_terminal() {
        let mut page = JobsPage::new();

        // Find a completed job
        let completed_idx = page
            .jobs
            .iter()
            .position(|j| j.status == JobStatus::Completed);

        if let Some(idx) = completed_idx {
            // Navigate to the completed job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_cancel_job();
                // Should return None because job is already in terminal state
                assert!(cmd.is_none());
            }
        }
    }

    #[test]
    fn action_retry_job_retries_failed_job() {
        let mut page = JobsPage::new();

        // Find a failed job
        let failed_idx = page.jobs.iter().position(|j| j.status == JobStatus::Failed);

        if let Some(idx) = failed_idx {
            // Navigate to the failed job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_retry_job();

                // Job should now be queued for retry
                assert_eq!(page.jobs[idx].status, JobStatus::Queued);
                // progress should be reset
                assert_eq!(page.jobs[idx].progress, 0);
                // started_at and ended_at should be cleared
                assert!(page.jobs[idx].started_at.is_none());
                assert!(page.jobs[idx].ended_at.is_none());
                // error should be cleared
                assert!(page.jobs[idx].error.is_none());
                // Should return a notification command
                assert!(cmd.is_some());
            }
        }
    }

    #[test]
    fn action_retry_job_returns_none_if_not_retriable() {
        let mut page = JobsPage::new();

        // Find a running job
        let running_idx = page
            .jobs
            .iter()
            .position(|j| j.status == JobStatus::Running);

        if let Some(idx) = running_idx {
            // Navigate to the running job
            while page.selected_job_index() != Some(idx)
                && page.table.cursor() < page.filtered_indices.len()
            {
                page.table.move_down(1);
            }

            if page.selected_job_index() == Some(idx) {
                let cmd = page.action_retry_job();
                // Should return None because job is not in a retriable state
                assert!(cmd.is_none());
            }
        }
    }

    #[test]
    fn action_create_and_start_workflow() {
        let mut page = JobsPage::new();
        let initial_count = page.jobs.len();

        // Create a new job
        page.action_create_job();
        assert_eq!(page.jobs.len(), initial_count + 1);

        // The new job should be at the end and queued
        let new_job_idx = page.jobs.len() - 1;
        assert_eq!(page.jobs[new_job_idx].status, JobStatus::Queued);

        // Test the action API directly
        let job = &mut page.jobs[new_job_idx];
        let original_id = job.id;

        // Start the job
        let result = start_job(job, &mut page.id_gen);
        assert!(result.is_some());
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.id, original_id);
    }

    // =========================================================================
    // Progress Visualization Tests (bd-3mxk)
    // =========================================================================

    #[test]
    fn format_progress_cell_queued() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Queued,
            progress: 0,
            created_at: chrono::Utc::now(),
            started_at: None,
            ended_at: None,
            error: None,
        };

        let cell = JobsPage::format_progress_cell(&job);
        assert!(
            cell.contains("queued"),
            "Queued job should show 'queued': {}",
            cell
        );
    }

    #[test]
    fn format_progress_cell_running() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 50,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now() - chrono::Duration::seconds(30)),
            ended_at: None,
            error: None,
        };

        let cell = JobsPage::format_progress_cell(&job);
        assert!(
            cell.contains("50%"),
            "Running job should show percentage: {}",
            cell
        );
    }

    #[test]
    fn format_progress_cell_completed() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Completed,
            progress: 100,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: None,
        };

        let cell = JobsPage::format_progress_cell(&job);
        assert!(
            cell.contains("done"),
            "Completed job should show 'done': {}",
            cell
        );
    }

    #[test]
    fn format_progress_cell_failed() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Failed,
            progress: 30,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: Some("Something went wrong".to_string()),
        };

        let cell = JobsPage::format_progress_cell(&job);
        assert!(
            cell.contains("error"),
            "Failed job should show 'error': {}",
            cell
        );
    }

    #[test]
    fn format_progress_cell_cancelled() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Cancelled,
            progress: 20,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: None,
        };

        let cell = JobsPage::format_progress_cell(&job);
        assert!(
            cell.contains("cancel"),
            "Cancelled job should show 'cancel': {}",
            cell
        );
    }

    #[test]
    fn estimate_eta_zero_progress() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 0,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: None,
            error: None,
        };

        let eta = JobsPage::estimate_eta(&job);
        assert!(eta.is_none(), "Zero progress should have no ETA");
    }

    #[test]
    fn estimate_eta_with_progress() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 50,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now() - chrono::Duration::seconds(60)),
            ended_at: None,
            error: None,
        };

        let eta = JobsPage::estimate_eta(&job);
        assert!(eta.is_some(), "Running job with progress should have ETA");
        let eta_str = eta.unwrap();
        assert!(
            eta_str.starts_with('~'),
            "ETA should start with ~: {}",
            eta_str
        );
    }

    #[test]
    fn estimate_eta_not_running() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Completed,
            progress: 100,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: None,
        };

        let eta = JobsPage::estimate_eta(&job);
        assert!(eta.is_none(), "Completed job should have no ETA");
    }

    #[test]
    fn is_job_slow_not_running() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Queued,
            progress: 0,
            created_at: chrono::Utc::now(),
            started_at: None,
            ended_at: None,
            error: None,
        };

        assert!(
            !JobsPage::is_job_slow(&job),
            "Queued job should not be slow"
        );
    }

    #[test]
    fn is_job_slow_fast_progress() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 50,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now() - chrono::Duration::seconds(60)),
            ended_at: None,
            error: None,
        };

        assert!(
            !JobsPage::is_job_slow(&job),
            "Job with good progress should not be slow"
        );
    }

    #[test]
    fn is_job_slow_stuck() {
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 5,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now() - chrono::Duration::seconds(60)),
            ended_at: None,
            error: None,
        };

        assert!(
            JobsPage::is_job_slow(&job),
            "Job with <10% progress after 30s should be slow"
        );
    }

    #[test]
    fn render_progress_bar_zero() {
        let theme = Theme::dark();
        let bar = JobsPage::render_progress_bar(0, 25, &theme);
        assert!(
            bar.contains("0%"),
            "Zero progress bar should show 0%: {}",
            bar
        );
    }

    #[test]
    fn render_progress_bar_fifty() {
        let theme = Theme::dark();
        let bar = JobsPage::render_progress_bar(50, 25, &theme);
        assert!(
            bar.contains("50%"),
            "50% progress bar should show 50%: {}",
            bar
        );
    }

    #[test]
    fn render_progress_bar_hundred() {
        let theme = Theme::dark();
        let bar = JobsPage::render_progress_bar(100, 25, &theme);
        assert!(
            bar.contains("100%"),
            "100% progress bar should show 100%: {}",
            bar
        );
    }

    #[test]
    fn render_progress_bar_over_hundred() {
        let theme = Theme::dark();
        let bar = JobsPage::render_progress_bar(150, 25, &theme);
        assert!(
            bar.contains("100%"),
            "Over 100% should clamp to 100%: {}",
            bar
        );
    }

    #[test]
    fn render_progress_bar_narrow_width() {
        let theme = Theme::dark();
        let bar = JobsPage::render_progress_bar(50, 5, &theme);
        // Very narrow bar should still show percentage
        assert!(
            bar.contains("50"),
            "Narrow bar should show percentage: {}",
            bar
        );
    }

    #[test]
    fn render_inline_progress_queued() {
        let theme = Theme::dark();
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Queued,
            progress: 0,
            created_at: chrono::Utc::now(),
            started_at: None,
            ended_at: None,
            error: None,
        };

        let inline = JobsPage::render_inline_progress(&job, &theme);
        assert!(
            inline.contains("queue") || inline.contains("◌"),
            "Queued should show queue indicator: {}",
            inline
        );
    }

    #[test]
    fn render_inline_progress_running() {
        let theme = Theme::dark();
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Running,
            progress: 50,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now() - chrono::Duration::seconds(30)),
            ended_at: None,
            error: None,
        };

        let inline = JobsPage::render_inline_progress(&job, &theme);
        assert!(
            inline.contains("50%") || inline.contains("█") || inline.contains("░"),
            "Running should show progress bar: {}",
            inline
        );
    }

    #[test]
    fn render_inline_progress_completed() {
        let theme = Theme::dark();
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Completed,
            progress: 100,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: None,
        };

        let inline = JobsPage::render_inline_progress(&job, &theme);
        assert!(
            inline.contains("✓") || inline.contains("Completed"),
            "Completed should show success: {}",
            inline
        );
    }

    #[test]
    fn render_inline_progress_failed() {
        let theme = Theme::dark();
        let job = Job {
            id: 1,
            name: "Test".to_string(),
            kind: JobKind::Build,
            status: JobStatus::Failed,
            progress: 30,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            ended_at: Some(chrono::Utc::now()),
            error: Some("Database error".to_string()),
        };

        let inline = JobsPage::render_inline_progress(&job, &theme);
        assert!(
            inline.contains("✕") || inline.contains("Database error"),
            "Failed should show error: {}",
            inline
        );
    }

    // =========================================================================
    // Paginator Integration Tests (bd-1tdh)
    // =========================================================================

    #[test]
    fn paginator_initializes_correctly() {
        let page = JobsPage::new();
        // With 20 jobs and 10 per page, should have 2 pages
        assert_eq!(page.total_pages(), 2);
        assert_eq!(page.current_page(), 0);
    }

    #[test]
    fn paginator_navigation_works() {
        let mut page = JobsPage::new();
        assert_eq!(page.current_page(), 0);

        // Navigate to next page
        page.next_page();
        assert_eq!(page.current_page(), 1);

        // Should not go past last page
        page.next_page();
        assert_eq!(page.current_page(), 1); // Still on page 1 (last page)

        // Navigate back
        page.prev_page();
        assert_eq!(page.current_page(), 0);

        // Should not go before first page
        page.prev_page();
        assert_eq!(page.current_page(), 0);
    }

    #[test]
    fn paginator_goto_page_works() {
        let mut page = JobsPage::new();

        page.goto_page(1);
        assert_eq!(page.current_page(), 1);

        page.goto_page(0);
        assert_eq!(page.current_page(), 0);

        // Out of bounds should clamp
        page.goto_page(100);
        assert_eq!(page.current_page(), 1); // Clamped to last page
    }

    #[test]
    fn filter_updates_paginator_total_pages() {
        let mut page = JobsPage::new();
        let initial_pages = page.total_pages();

        // Filter to fewer jobs
        page.query = "nonexistent_job_name".to_string();
        page.apply_filter_and_sort();

        // With no matches, should have 1 page (minimum)
        assert_eq!(page.total_pages(), 1);

        // Clear filter
        page.clear_filters();
        assert_eq!(page.total_pages(), initial_pages);
    }

    #[test]
    fn filter_resets_to_valid_page() {
        let mut page = JobsPage::new();

        // Go to second page
        page.goto_page(1);
        assert_eq!(page.current_page(), 1);

        // Apply a filter that reduces to fewer pages
        page.query = "unique_nonexistent".to_string();
        page.apply_filter_and_sort();

        // Page should be clamped to valid range
        assert!(page.current_page() < page.total_pages());
    }

    #[test]
    fn selected_job_respects_pagination() {
        let mut page = JobsPage::new();

        // First page, first job
        let first_page_job = page.selected_job().map(|j| j.id);
        assert!(first_page_job.is_some());

        // Go to second page
        page.next_page();
        let second_page_job = page.selected_job().map(|j| j.id);
        assert!(second_page_job.is_some());

        // Jobs should be different (different pages)
        assert_ne!(first_page_job, second_page_job);
    }

    #[test]
    fn table_cursor_stays_in_page_bounds() {
        let mut page = JobsPage::new();

        // Navigate within the page
        page.table.move_down(5);
        let cursor = page.table.cursor();
        assert!(cursor < DEFAULT_ITEMS_PER_PAGE);

        // The selected job should be valid
        assert!(page.selected_job().is_some());
    }

    #[test]
    fn refresh_resets_to_first_page() {
        let mut page = JobsPage::new();

        // Go to second page
        page.next_page();
        assert_eq!(page.current_page(), 1);

        // Refresh
        page.refresh();
        assert_eq!(page.current_page(), 0);
    }

    #[test]
    fn create_job_updates_pagination() {
        let mut page = JobsPage::new();
        let initial_total = page.jobs.len();
        let initial_pages = page.total_pages();

        // Create several jobs to potentially add a new page
        for _ in 0..DEFAULT_ITEMS_PER_PAGE {
            page.action_create_job();
        }

        assert_eq!(page.jobs.len(), initial_total + DEFAULT_ITEMS_PER_PAGE);
        assert!(page.total_pages() >= initial_pages);
    }
}
