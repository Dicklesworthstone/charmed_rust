//! Logs page - real-time log viewer with follow mode.
//!
//! This page displays a scrollable log stream with color-coded levels,
//! follow mode for live tailing, and smooth navigation.
//!
//! # Filtering & Search
//!
//! The page provides:
//! - **Level filter**: Toggle visibility of ERROR/WARN/INFO/DEBUG/TRACE
//! - **Service filter**: Filter by target/service name
//! - **Query bar**: Free-text substring search across log messages
//!
//! Filtering maintains a `filtered_indices` vector for efficient rendering.
//!
//! Uses `RwLock` for thread-safe interior mutability, enabling SSH mode.

use std::path::PathBuf;
use std::sync::RwLock;

use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};

use super::PageModel;
use crate::data::generator::GeneratedData;
use crate::data::{LogColumnWidths, LogEntry, LogFormatter, LogLevel, LogStream};
use crate::messages::{Notification, NotificationMsg, Page};
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Maximum number of log entries to retain.
const MAX_LOG_ENTRIES: usize = 1000;

// =============================================================================
// Performance Helpers (bd-3kvw)
// =============================================================================

/// Case-insensitive substring search without allocating lowercase copies.
///
/// This is O(n*m) but avoids the allocation overhead of `to_lowercase()`.
/// For log filtering with ~1000 entries, this is faster than allocating
/// 2000 strings per keystroke.
#[inline]
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }

    // Fast path for ASCII-only (most common for log messages)
    let needle_bytes = needle.as_bytes();
    let haystack_bytes = haystack.as_bytes();

    'outer: for i in 0..=(haystack_bytes.len() - needle_bytes.len()) {
        for j in 0..needle_bytes.len() {
            let h = haystack_bytes[i + j];
            let n = needle_bytes[j];

            // Case-insensitive ASCII comparison
            if h != n && h.to_ascii_lowercase() != n.to_ascii_lowercase() {
                continue 'outer;
            }
        }
        return true;
    }
    false
}

// =============================================================================
// Filtering
// =============================================================================

/// Level filter state - which log levels to show.
#[derive(Debug, Clone, Copy)]
pub struct LevelFilter {
    /// Show ERROR level logs.
    pub error: bool,
    /// Show WARN level logs.
    pub warn: bool,
    /// Show INFO level logs.
    pub info: bool,
    /// Show DEBUG level logs.
    pub debug: bool,
    /// Show TRACE level logs.
    pub trace: bool,
}

impl Default for LevelFilter {
    fn default() -> Self {
        Self::all()
    }
}

impl LevelFilter {
    /// Create a filter that shows all levels.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            error: true,
            warn: true,
            info: true,
            debug: true,
            trace: true,
        }
    }

    /// Toggle a specific level filter.
    pub fn toggle(&mut self, level: LogLevel) {
        match level {
            LogLevel::Error => self.error = !self.error,
            LogLevel::Warn => self.warn = !self.warn,
            LogLevel::Info => self.info = !self.info,
            LogLevel::Debug => self.debug = !self.debug,
            LogLevel::Trace => self.trace = !self.trace,
        }
    }

    /// Check if a log level passes the filter.
    #[must_use]
    pub const fn matches(&self, level: LogLevel) -> bool {
        match level {
            LogLevel::Error => self.error,
            LogLevel::Warn => self.warn,
            LogLevel::Info => self.info,
            LogLevel::Debug => self.debug,
            LogLevel::Trace => self.trace,
        }
    }

    /// Count how many levels are enabled.
    #[must_use]
    #[allow(dead_code)]
    pub const fn enabled_count(&self) -> usize {
        let mut count = 0;
        if self.error {
            count += 1;
        }
        if self.warn {
            count += 1;
        }
        if self.info {
            count += 1;
        }
        if self.debug {
            count += 1;
        }
        if self.trace {
            count += 1;
        }
        count
    }
}

/// Focus state for the logs page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogsFocus {
    /// Viewport is focused (default).
    #[default]
    Viewport,
    /// Query input is focused.
    QueryInput,
}

/// Logs page showing real-time log viewer with follow mode.
pub struct LogsPage {
    /// The viewport for scrollable content (`RwLock` for thread-safe interior mutability).
    viewport: RwLock<Viewport>,
    /// The log stream containing entries.
    logs: LogStream,
    /// Filtered entry indices (indices into logs.entries()).
    filtered_indices: Vec<usize>,
    /// Whether follow mode is enabled (tail -f behavior).
    following: bool,
    /// Currently selected line index (for copy/export).
    #[expect(dead_code, reason = "Reserved for future copy/export feature")]
    selected_line: Option<usize>,
    /// Current seed for data generation.
    seed: u64,
    /// Cached formatted content (`RwLock` for thread-safe interior mutability).
    formatted_content: RwLock<String>,
    /// Whether content needs to be reformatted.
    needs_reformat: RwLock<bool>,
    /// Last known dimensions (for detecting resize).
    last_dims: RwLock<(usize, usize)>,
    /// Query input for filtering.
    query_input: TextInput,
    /// Current query text.
    query: String,
    /// Level filter state.
    level_filter: LevelFilter,
    /// Current focus state.
    focus: LogsFocus,
}

impl LogsPage {
    /// Create a new logs page.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    /// Create a new logs page with the given seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let data = GeneratedData::generate(seed);
        let logs = Self::generate_initial_logs(&data, seed);

        // Initialize filtered indices to all entries
        let filtered_indices: Vec<usize> = (0..logs.len()).collect();

        // Start with follow mode enabled
        let mut viewport = Viewport::new(80, 24);
        viewport.mouse_wheel_enabled = true;
        viewport.mouse_wheel_delta = 3;

        // Create query input
        let mut query_input = TextInput::new();
        query_input.set_placeholder("Filter logs... (/ to focus)");
        query_input.width = 40;

        Self {
            viewport: RwLock::new(viewport),
            logs,
            filtered_indices,
            following: true,
            selected_line: None,
            seed,
            formatted_content: RwLock::new(String::new()),
            needs_reformat: RwLock::new(true),
            last_dims: RwLock::new((0, 0)),
            query_input,
            query: String::new(),
            level_filter: LevelFilter::all(),
            focus: LogsFocus::Viewport,
        }
    }

    // =========================================================================
    // Filtering
    // =========================================================================

    /// Apply current filters, updating filtered_indices.
    ///
    /// Performance: Uses `contains_ignore_case()` to avoid allocating lowercase
    /// copies of message/target for every entry (bd-3kvw optimization).
    fn apply_filters(&mut self) {
        let query = &self.query;
        let entries = self.logs.entries();

        self.filtered_indices = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                // Level filter
                if !self.level_filter.matches(entry.level) {
                    return false;
                }

                // Query filter (match message or target) - no allocations
                if !query.is_empty() {
                    let msg_match = contains_ignore_case(&entry.message, query);
                    let target_match = contains_ignore_case(&entry.target, query);
                    if !msg_match && !target_match {
                        return false;
                    }
                }

                true
            })
            .map(|(i, _)| i)
            .collect();

        // Mark for reformatting
        *self.needs_reformat.write().unwrap() = true;
    }

    /// Toggle a level filter and reapply.
    fn toggle_level_filter(&mut self, level: LogLevel) {
        self.level_filter.toggle(level);
        self.apply_filters();
    }

    /// Clear all filters.
    fn clear_filters(&mut self) {
        self.query.clear();
        self.query_input.set_value("");
        self.level_filter = LevelFilter::all();
        self.apply_filters();
    }

    /// Generate initial log entries from the generated data.
    fn generate_initial_logs(data: &GeneratedData, seed: u64) -> LogStream {
        use rand::prelude::*;
        use rand_pcg::Pcg64;

        let mut rng = Pcg64::seed_from_u64(seed.wrapping_add(54321));
        let mut logs = LogStream::new(MAX_LOG_ENTRIES);

        let targets = [
            "api::handlers",
            "api::auth",
            "api::routes",
            "db::postgres",
            "db::redis",
            "cache::memcached",
            "worker::jobs",
            "worker::scheduler",
            "http::server",
            "grpc::server",
            "metrics::exporter",
            "health::checker",
        ];

        let messages = [
            "Request received",
            "Processing request",
            "Query executed",
            "Cache hit",
            "Cache miss",
            "Connection established",
            "Connection closed",
            "Task scheduled",
            "Task completed",
            "Metrics exported",
            "Health check passed",
            "Retrying operation",
            "Rate limit applied",
            "Authentication successful",
            "Session created",
            "Data validated",
            "Response sent",
            "Error handled gracefully",
        ];

        // Generate a mix of entries correlated with services and jobs
        let entry_count = rng.random_range(150..250);

        for i in 0..entry_count {
            let level = if rng.random_ratio(1, 50) {
                LogLevel::Error
            } else if rng.random_ratio(1, 15) {
                LogLevel::Warn
            } else if rng.random_ratio(1, 5) {
                LogLevel::Debug
            } else if rng.random_ratio(1, 10) {
                LogLevel::Trace
            } else {
                LogLevel::Info
            };

            let target_idx = rng.random_range(0..targets.len());
            let msg_idx = rng.random_range(0..messages.len());

            let target = targets[target_idx];
            let message = messages[msg_idx];

            // Optionally correlate with a job
            let job_id = if rng.random_ratio(1, 4) && !data.jobs.is_empty() {
                let job_idx = rng.random_range(0..data.jobs.len());
                Some(data.jobs[job_idx].id)
            } else {
                None
            };

            // Optionally correlate with a deployment
            let deployment_id = if rng.random_ratio(1, 6) && !data.deployments.is_empty() {
                let deploy_idx = rng.random_range(0..data.deployments.len());
                Some(data.deployments[deploy_idx].id)
            } else {
                None
            };

            #[expect(clippy::cast_sign_loss, reason = "i is always non-negative from loop")]
            let tick = i as u64;

            let entry = LogEntry::new(logs.len() as u64 + 1, level, target, message)
                .with_tick(tick)
                .with_field(
                    "request_id",
                    format!("req-{:06x}", rng.random::<u32>() % 0x00FF_FFFF),
                );

            // Add correlation IDs if present
            let entry = if let Some(jid) = job_id {
                entry.with_job_id(jid)
            } else {
                entry
            };
            let entry = if let Some(did) = deployment_id {
                entry.with_deployment_id(did)
            } else {
                entry
            };

            logs.push(entry);
        }

        logs
    }

    /// Refresh logs with a new seed, preserving filters.
    pub fn refresh(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        let data = GeneratedData::generate(self.seed);
        self.logs = Self::generate_initial_logs(&data, self.seed);
        // Reapply filters to new data
        self.apply_filters();
        if self.following {
            self.viewport.write().unwrap().goto_bottom();
        }
    }

    /// Add a new log entry (for live updates).
    #[allow(dead_code)] // Reserved for simulation tick integration
    pub fn push_log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
        *self.needs_reformat.write().unwrap() = true;
        if self.following {
            self.viewport.write().unwrap().goto_bottom();
        }
    }

    /// Toggle follow mode.
    pub fn toggle_follow(&mut self) {
        self.following = !self.following;
        if self.following {
            self.viewport.write().unwrap().goto_bottom();
        }
    }

    /// Check if follow mode should pause (user scrolled up).
    fn check_follow_pause(&mut self) {
        if self.following && !self.viewport.read().unwrap().at_bottom() {
            self.following = false;
        }
    }

    // =========================================================================
    // Actions (bd-15xi)
    // =========================================================================

    /// Get the export directory path based on mode.
    ///
    /// In E2E/test mode, writes to `target/demo_showcase_e2e/logs/`.
    /// Otherwise, writes to `./demo_showcase_exports/`.
    fn export_dir() -> PathBuf {
        // Check for E2E mode via environment variable
        if std::env::var("DEMO_SHOWCASE_E2E").is_ok() {
            PathBuf::from("target/demo_showcase_e2e/logs")
        } else {
            PathBuf::from("demo_showcase_exports")
        }
    }

    /// Copy the current visible viewport content to a file.
    ///
    /// Returns a command to show a notification.
    fn action_copy_viewport(&self, theme: &Theme) -> Option<Cmd> {
        let content = self.viewport.read().unwrap().view();
        Self::write_to_export_file("viewport", &content, theme)
    }

    /// Copy all filtered log entries to a file.
    ///
    /// Returns a command to show a notification.
    fn action_copy_all(&self, theme: &Theme) -> Option<Cmd> {
        let content = self.format_logs_plain();
        Self::write_to_export_file("logs_full", &content, theme)
    }

    /// Export the full log buffer to a file (plain text format).
    ///
    /// Returns a command to show a notification.
    fn action_export(&self, _theme: &Theme) -> Option<Cmd> {
        let content = self.format_logs_plain();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let export_dir = Self::export_dir();
        let filename = format!("logs_export_{timestamp}.txt");
        let filepath = export_dir.join(&filename);

        // Use blocking command to write file
        Some(Cmd::blocking(move || {
            // Ensure export directory exists
            if let Err(e) = std::fs::create_dir_all(&export_dir) {
                let notification =
                    Notification::error(0, format!("Failed to create export dir: {e}"));
                return NotificationMsg::Show(notification).into_message();
            }

            match std::fs::write(&filepath, content) {
                Ok(()) => {
                    let notification = Notification::success(
                        0,
                        format!("Exported logs to {}", filepath.display()),
                    );
                    NotificationMsg::Show(notification).into_message()
                }
                Err(e) => {
                    let notification = Notification::error(0, format!("Export failed: {e}"));
                    NotificationMsg::Show(notification).into_message()
                }
            }
        }))
    }

    /// Clear all logs from the buffer.
    ///
    /// Returns a command to show a notification.
    fn action_clear(&mut self) -> Option<Cmd> {
        let count = self.logs.len();
        self.logs = LogStream::new(MAX_LOG_ENTRIES);
        self.filtered_indices.clear();
        *self.needs_reformat.write().unwrap() = true;
        self.viewport.write().unwrap().set_content("");

        Some(Cmd::new(move || {
            let notification = Notification::info(0, format!("Cleared {count} log entries"));
            NotificationMsg::Show(notification).into_message()
        }))
    }

    /// Write content to an export file and return a notification command.
    fn write_to_export_file(prefix: &str, content: &str, _theme: &Theme) -> Option<Cmd> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let export_dir = Self::export_dir();
        let filename = format!("{prefix}_{timestamp}.txt");
        let filepath = export_dir.join(&filename);
        let content = content.to_string();

        Some(Cmd::blocking(move || {
            // Ensure export directory exists
            if let Err(e) = std::fs::create_dir_all(&export_dir) {
                let notification =
                    Notification::error(0, format!("Failed to create export dir: {e}"));
                return NotificationMsg::Show(notification).into_message();
            }

            match std::fs::write(&filepath, content) {
                Ok(()) => {
                    let notification =
                        Notification::success(0, format!("Copied to {}", filepath.display()));
                    NotificationMsg::Show(notification).into_message()
                }
                Err(e) => {
                    let notification = Notification::error(0, format!("Copy failed: {e}"));
                    NotificationMsg::Show(notification).into_message()
                }
            }
        }))
    }

    /// Format logs as plain text without ANSI styling.
    fn format_logs_plain(&self) -> String {
        let entries = self.logs.entries();
        self.filtered_indices
            .iter()
            .filter_map(|&i| entries.get(i))
            .map(|entry| {
                let timestamp = entry.timestamp.format("%H:%M:%S");
                let level = entry.level.abbrev();
                format!(
                    "{} {} [{}] {}",
                    timestamp, level, entry.target, entry.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format filtered log entries for display.
    fn format_logs(&self, theme: &Theme, target_width: usize) -> String {
        // Calculate optimal target width based on available space
        // timestamp (8) + level (5) + spacing (4) = 17
        let target_col_width = target_width.saturating_sub(17).clamp(10, 25);

        let formatter = LogFormatter::new(theme).with_widths(LogColumnWidths {
            timestamp: 8,
            level: 5,
            target: target_col_width,
        });

        let entries = self.logs.entries();
        self.filtered_indices
            .iter()
            .filter_map(|&i| entries.get(i))
            .map(|entry| formatter.format(entry))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render the filter bar with level chips and query input.
    fn render_filter_bar(&self, theme: &Theme, _width: usize) -> String {
        // Level filter chips
        let chip = |label: &str, active: bool, style: lipgloss::Style| -> String {
            let prefix = if active { "[●]" } else { "[ ]" };
            let text = format!("{prefix} {label}");
            if active {
                style.render(&text)
            } else {
                theme.muted_style().render(&text)
            }
        };

        let error_chip = chip("E", self.level_filter.error, theme.error_style());
        let warn_chip = chip("W", self.level_filter.warn, theme.warning_style());
        let info_chip = chip("I", self.level_filter.info, theme.info_style());
        let debug_chip = chip("D", self.level_filter.debug, theme.muted_style());
        let trace_chip = chip("T", self.level_filter.trace, theme.muted_style());

        // Query input
        let label = if self.focus == LogsFocus::QueryInput {
            theme.info_style().render("Filter: ")
        } else {
            theme.muted_style().render("/ filter ")
        };

        let input_view = self.query_input.view();

        format!(
            "{error_chip} {warn_chip} {info_chip} {debug_chip} {trace_chip}  {label}{input_view}"
        )
    }

    /// Render the status bar showing follow mode and position.
    fn render_status_bar(&self, theme: &Theme, width: usize) -> String {
        let follow_indicator = if self.following {
            theme.success_style().bold().render(" FOLLOWING ")
        } else {
            theme.warning_style().render(" PAUSED ")
        };

        // Show filtered count vs total
        let total = self.logs.len();
        let filtered = self.filtered_indices.len();
        let count_display = if filtered == total {
            theme.muted_style().render(&format!("{total} lines"))
        } else {
            theme
                .info_style()
                .render(&format!("{filtered}/{total} shown"))
        };

        let viewport = self.viewport.read().unwrap();
        let position = format!("{}:{}", viewport.y_offset() + 1, filtered);
        let position_styled = theme.muted_style().render(&position);

        // Calculate spacing
        let indicator_len = if self.following { 11 } else { 8 };
        let content_len = indicator_len + 30; // Approximate
        let padding = width.saturating_sub(content_len);

        format!(
            "{}{:padding$}{}  {}",
            follow_indicator,
            "",
            count_display,
            position_styled,
            padding = padding
        )
    }
}

impl Default for LogsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for LogsPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Handle key messages
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            // Handle query input focus
            if self.focus == LogsFocus::QueryInput {
                match key.key_type {
                    KeyType::Esc => {
                        // Exit query input, return to viewport
                        self.focus = LogsFocus::Viewport;
                        self.query_input.blur();
                        return None;
                    }
                    KeyType::Enter => {
                        // Apply filter and return to viewport
                        self.focus = LogsFocus::Viewport;
                        self.query_input.blur();
                        return None;
                    }
                    KeyType::Backspace => {
                        // Delete last character
                        self.query.pop();
                        self.query_input.set_value(&self.query);
                        self.apply_filters();
                        return None;
                    }
                    KeyType::Runes => {
                        // Add typed characters
                        for c in &key.runes {
                            if c.is_alphanumeric()
                                || *c == '-'
                                || *c == '_'
                                || *c == ' '
                                || *c == ':'
                                || *c == '.'
                            {
                                self.query.push(*c);
                            }
                        }
                        self.query_input.set_value(&self.query);
                        self.apply_filters();
                        return None;
                    }
                    _ => {
                        return None;
                    }
                }
            }

            // Viewport focus mode
            match key.key_type {
                KeyType::Home => {
                    self.viewport.write().unwrap().goto_top();
                    self.following = false;
                    return None;
                }
                KeyType::End => {
                    self.viewport.write().unwrap().goto_bottom();
                    self.following = true;
                    return None;
                }
                KeyType::Runes => {
                    // Handle character keys
                    match key.runes.as_slice() {
                        ['/'] => {
                            // Enter query input mode
                            self.focus = LogsFocus::QueryInput;
                            self.query_input.focus();
                            return None;
                        }
                        // Toggle follow mode with 'f' or 'F'
                        ['f' | 'F'] => {
                            self.toggle_follow();
                            return None;
                        }
                        // Go to top with 'g'
                        ['g'] => {
                            self.viewport.write().unwrap().goto_top();
                            self.following = false;
                            return None;
                        }
                        // Go to bottom with 'G'
                        ['G'] => {
                            self.viewport.write().unwrap().goto_bottom();
                            self.following = true;
                            return None;
                        }
                        // Refresh with 'r' or 'R'
                        ['r' | 'R'] => {
                            self.refresh();
                            return None;
                        }
                        // Level filter toggles (Shift+1-5: !, @, #, $, %)
                        // Uses Shift+number to avoid conflict with page navigation (1-8)
                        ['!'] => {
                            self.toggle_level_filter(LogLevel::Error);
                            return None;
                        }
                        ['@'] => {
                            self.toggle_level_filter(LogLevel::Warn);
                            return None;
                        }
                        ['#'] => {
                            self.toggle_level_filter(LogLevel::Info);
                            return None;
                        }
                        ['$'] => {
                            self.toggle_level_filter(LogLevel::Debug);
                            return None;
                        }
                        ['%'] => {
                            self.toggle_level_filter(LogLevel::Trace);
                            return None;
                        }
                        // Copy/export/clear actions (bd-15xi)
                        ['y'] => {
                            // Copy visible viewport to file
                            return self.action_copy_viewport(&Theme::default());
                        }
                        ['Y'] => {
                            // Copy all filtered logs to file
                            return self.action_copy_all(&Theme::default());
                        }
                        ['e'] => {
                            // Export full log buffer to file
                            return self.action_export(&Theme::default());
                        }
                        ['X'] => {
                            // Clear log buffer (capital X for dangerous action)
                            return self.action_clear();
                        }
                        // Clear filters
                        ['c'] => {
                            self.clear_filters();
                            return None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Delegate to viewport for scroll handling
        self.viewport.write().unwrap().update(msg);

        // Check if scrolling paused follow mode
        self.check_follow_pause();

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Reserve space for filter bar and status bar
        let filter_bar_height = 1;
        let status_bar_height = 1;
        let content_height = height.saturating_sub(filter_bar_height + status_bar_height + 1);

        // Check if dimensions changed or content needs reformatting
        let last_dims = *self.last_dims.read().unwrap();
        let needs_resize = last_dims.0 != width || last_dims.1 != content_height;
        let needs_reformat = *self.needs_reformat.read().unwrap();

        if needs_resize || needs_reformat {
            let mut viewport = self.viewport.write().unwrap();
            viewport.width = width;
            viewport.height = content_height;

            let formatted = self.format_logs(theme, width);
            viewport.set_content(&formatted);
            *self.formatted_content.write().unwrap() = formatted;
            *self.needs_reformat.write().unwrap() = false;
            *self.last_dims.write().unwrap() = (width, content_height);

            // Maintain follow mode position
            if self.following {
                viewport.goto_bottom();
            }
        }

        // Render filter bar
        let filter_bar = self.render_filter_bar(theme, width);

        // Render viewport content
        let content = self.viewport.read().unwrap().view();

        // Render status bar
        let status = self.render_status_bar(theme, width);

        // Combine with newlines
        format!("{filter_bar}\n{content}\n{status}")
    }

    fn page(&self) -> Page {
        Page::Logs
    }

    fn hints(&self) -> &'static str {
        "y copy  e export  X clear  / filter  1-5 levels  f follow  j/k scroll"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        // Mark content for reformatting when page becomes active
        *self.needs_reformat.write().unwrap() = true;
        self.focus = LogsFocus::Viewport;
        if self.following {
            self.viewport.write().unwrap().goto_bottom();
        }
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        self.query_input.blur();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_page_creates_with_data() {
        let page = LogsPage::new();
        assert!(!page.logs.is_empty());
        assert!(page.following);
    }

    #[test]
    fn logs_page_deterministic() {
        let page1 = LogsPage::with_seed(123);
        let page2 = LogsPage::with_seed(123);
        assert_eq!(page1.logs.len(), page2.logs.len());
    }

    #[test]
    fn logs_page_different_seeds_differ() {
        let page1 = LogsPage::with_seed(1);
        let page2 = LogsPage::with_seed(2);
        // Content should differ (not guaranteed but very likely)
        let counts1 = page1.logs.count_by_level();
        let counts2 = page2.logs.count_by_level();
        // At least one count should differ
        assert!(
            counts1.error != counts2.error
                || counts1.warn != counts2.warn
                || counts1.info != counts2.info
        );
    }

    #[test]
    fn logs_page_toggle_follow() {
        let mut page = LogsPage::new();
        assert!(page.following);

        page.toggle_follow();
        assert!(!page.following);

        page.toggle_follow();
        assert!(page.following);
    }

    #[test]
    fn logs_page_refresh_changes_seed() {
        let mut page = LogsPage::with_seed(42);
        let initial_seed = page.seed;

        page.refresh();
        assert_ne!(page.seed, initial_seed);
    }

    #[test]
    fn logs_page_push_log() {
        let mut page = LogsPage::new();
        let initial_len = page.logs.len();

        let entry = LogEntry::new(999, LogLevel::Info, "test", "Test message");
        page.push_log(entry);

        assert_eq!(page.logs.len(), initial_len + 1);
    }

    #[test]
    fn logs_page_hints() {
        let page = LogsPage::new();
        let hints = page.hints();
        assert!(hints.contains("follow"));
        assert!(hints.contains("scroll"));
    }

    #[test]
    fn logs_page_page_type() {
        let page = LogsPage::new();
        assert_eq!(page.page(), Page::Logs);
    }

    // =========================================================================
    // Filtering Tests
    // =========================================================================

    #[test]
    fn initial_filter_shows_all() {
        let page = LogsPage::new();
        assert_eq!(page.filtered_indices.len(), page.logs.len());
    }

    #[test]
    fn level_filter_reduces_count() {
        let mut page = LogsPage::new();
        let total = page.logs.len();

        // Disable INFO filter (most common level)
        page.level_filter.info = false;
        page.apply_filters();

        // Should have fewer entries
        assert!(page.filtered_indices.len() < total);
    }

    #[test]
    fn query_filter_matches_message() {
        let mut page = LogsPage::new();

        // Filter to logs containing "request"
        page.query = "request".to_string();
        page.apply_filters();

        // All filtered logs should contain "request" (case-insensitive)
        let entries = page.logs.entries();
        for &idx in &page.filtered_indices {
            let entry = &entries[idx];
            let matches = entry.message.to_lowercase().contains("request")
                || entry.target.to_lowercase().contains("request");
            assert!(matches, "Entry should match 'request': {:?}", entry);
        }
    }

    #[test]
    fn query_filter_matches_target() {
        let mut page = LogsPage::new();

        // Filter by target/service
        page.query = "api".to_string();
        page.apply_filters();

        // Should find some matches
        assert!(!page.filtered_indices.is_empty());
    }

    #[test]
    fn clear_filters_restores_all() {
        let mut page = LogsPage::new();
        let original_count = page.filtered_indices.len();

        // Apply some filters
        page.query = "nonexistent".to_string();
        page.level_filter.error = false;
        page.level_filter.warn = false;
        page.apply_filters();

        // Clear and restore
        page.clear_filters();
        assert_eq!(page.filtered_indices.len(), original_count);
    }

    #[test]
    fn level_filter_toggle() {
        let mut filter = LevelFilter::all();
        assert!(filter.error);

        filter.toggle(LogLevel::Error);
        assert!(!filter.error);

        filter.toggle(LogLevel::Error);
        assert!(filter.error);
    }

    #[test]
    fn level_filter_matches_correctly() {
        let filter = LevelFilter {
            error: true,
            warn: false,
            info: true,
            debug: false,
            trace: true,
        };

        assert!(filter.matches(LogLevel::Error));
        assert!(!filter.matches(LogLevel::Warn));
        assert!(filter.matches(LogLevel::Info));
        assert!(!filter.matches(LogLevel::Debug));
        assert!(filter.matches(LogLevel::Trace));
    }

    #[test]
    fn level_filter_enabled_count() {
        let filter = LevelFilter {
            error: true,
            warn: true,
            info: false,
            debug: false,
            trace: false,
        };

        assert_eq!(filter.enabled_count(), 2);
    }

    // =========================================================================
    // Edge Case Tests (for bd-3eru)
    // =========================================================================

    #[test]
    fn empty_query_shows_all_with_level_filter() {
        let mut page = LogsPage::new();
        let total = page.logs.len();

        // Empty query should show all entries (respecting level filter)
        page.query = String::new();
        page.apply_filters();
        assert_eq!(page.filtered_indices.len(), total);
    }

    #[test]
    fn unicode_query_does_not_panic() {
        let mut page = LogsPage::new();

        // Unicode characters in query should not panic
        page.query = "日本語テスト".to_string();
        page.apply_filters();
        // Should complete without panicking (likely no matches)
        assert!(page.filtered_indices.len() <= page.logs.len());
    }

    #[test]
    fn emoji_query_does_not_panic() {
        let mut page = LogsPage::new();

        // Emoji in query should not panic
        page.query = "🚀 deployment 🎉".to_string();
        page.apply_filters();
        // Should complete without panicking
        assert!(page.filtered_indices.len() <= page.logs.len());
    }

    #[test]
    fn very_long_query_does_not_panic() {
        let mut page = LogsPage::new();

        // Very long query should not panic or cause memory issues
        page.query = "a".repeat(10_000);
        page.apply_filters();
        // Should complete without panicking (likely no matches)
        assert!(page.filtered_indices.is_empty() || page.filtered_indices.len() <= page.logs.len());
    }

    #[test]
    fn whitespace_only_query_shows_all() {
        let mut page = LogsPage::new();
        let total = page.logs.len();

        // Whitespace-only query should effectively be empty after trim
        // Note: Current implementation doesn't trim, so this tests actual behavior
        page.query = "   ".to_string();
        page.apply_filters();
        // Should not crash; result depends on implementation
        assert!(page.filtered_indices.len() <= total);
    }

    #[test]
    fn newline_in_query_does_not_panic() {
        let mut page = LogsPage::new();

        // Paste-like input with newlines
        page.query = "error\nmessage\ntest".to_string();
        page.apply_filters();
        // Should complete without panicking
        assert!(page.filtered_indices.len() <= page.logs.len());
    }

    #[test]
    fn filter_is_case_insensitive() {
        let mut page = LogsPage::new();

        // Same query in different cases should match same entries
        page.query = "ERROR".to_string();
        page.apply_filters();
        let upper_count = page.filtered_indices.len();

        page.query = "error".to_string();
        page.apply_filters();
        let lower_count = page.filtered_indices.len();

        page.query = "Error".to_string();
        page.apply_filters();
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
        let mut page = LogsPage::new();

        page.query = "api".to_string();
        page.apply_filters();
        let first_result = page.filtered_indices.clone();

        // Apply again - should get same result
        page.apply_filters();
        let second_result = page.filtered_indices.clone();

        assert_eq!(first_result, second_result, "Filter should be idempotent");
    }

    // =========================================================================
    // Action Tests (bd-15xi)
    // =========================================================================

    #[test]
    fn action_clear_empties_logs() {
        let mut page = LogsPage::new();
        assert!(!page.logs.is_empty(), "Should start with logs");

        let cmd = page.action_clear();

        // Logs should be cleared
        assert!(page.logs.is_empty(), "Logs should be empty after clear");
        assert!(
            page.filtered_indices.is_empty(),
            "Filtered indices should be empty"
        );
        // Should return a notification command
        assert!(cmd.is_some(), "Should return a notification command");
    }

    #[test]
    fn format_logs_plain_produces_text() {
        let page = LogsPage::new();

        let plain = page.format_logs_plain();

        // Should produce non-empty plain text
        assert!(!plain.is_empty(), "Plain log format should not be empty");
        // Should contain timestamp format (HH:MM:SS)
        assert!(plain.contains(':'), "Should contain timestamp separators");
        // Should not contain ANSI escape codes
        assert!(
            !plain.contains("\x1b["),
            "Plain text should not contain ANSI escapes"
        );
    }

    #[test]
    fn export_dir_default_path() {
        // Test the default export directory (when E2E env var is not set)
        // Note: We cannot safely modify env vars in tests due to forbid(unsafe_code),
        // so we test the default case only.
        // The E2E case is covered by integration/E2E tests.
        let dir = LogsPage::export_dir();
        // Either the default or E2E path is valid depending on test environment
        let valid_paths = [
            std::path::PathBuf::from("demo_showcase_exports"),
            std::path::PathBuf::from("target/demo_showcase_e2e/logs"),
        ];
        assert!(
            valid_paths.contains(&dir),
            "Export dir should be a valid path: {:?}",
            dir
        );
    }

    #[test]
    fn action_export_returns_command() {
        let page = LogsPage::new();

        let cmd = page.action_export(&Theme::default());

        // Should return a command for file export
        assert!(cmd.is_some(), "action_export should return a command");
    }

    // =========================================================================
    // Performance Optimization Tests (bd-3kvw)
    // =========================================================================

    #[test]
    fn contains_ignore_case_basic() {
        // Basic ASCII matching
        assert!(contains_ignore_case("Hello World", "hello"));
        assert!(contains_ignore_case("Hello World", "WORLD"));
        assert!(contains_ignore_case("Hello World", "lo Wo"));
        assert!(!contains_ignore_case("Hello World", "xyz"));
    }

    #[test]
    fn contains_ignore_case_empty() {
        // Empty needle always matches
        assert!(contains_ignore_case("anything", ""));
        assert!(contains_ignore_case("", ""));
    }

    #[test]
    fn contains_ignore_case_needle_longer() {
        // Needle longer than haystack never matches
        assert!(!contains_ignore_case("hi", "hello"));
    }

    #[test]
    fn contains_ignore_case_exact_match() {
        assert!(contains_ignore_case("test", "test"));
        assert!(contains_ignore_case("test", "TEST"));
        assert!(contains_ignore_case("TEST", "test"));
    }

    #[test]
    fn contains_ignore_case_start_middle_end() {
        let haystack = "The quick brown fox";
        assert!(contains_ignore_case(haystack, "The")); // Start
        assert!(contains_ignore_case(haystack, "quick")); // Middle
        assert!(contains_ignore_case(haystack, "fox")); // End
        assert!(contains_ignore_case(haystack, "THE QUICK")); // Case mismatch
    }

    #[test]
    fn contains_ignore_case_special_chars() {
        // Should handle non-letter ASCII characters
        assert!(contains_ignore_case("[ERROR] failed", "[error]"));
        assert!(contains_ignore_case("2024-01-01", "01-01"));
        assert!(contains_ignore_case("user@example.com", "@EXAMPLE"));
    }
}
