//! Logs page - real-time log viewer with follow mode.
//!
//! This page displays a scrollable log stream with color-coded levels,
//! follow mode for live tailing, and smooth navigation.

use std::cell::RefCell;

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};

use super::PageModel;
use crate::data::generator::GeneratedData;
use crate::data::{LogColumnWidths, LogEntry, LogFormatter, LogLevel, LogStream};
use crate::messages::Page;
use crate::theme::Theme;

/// Default seed for deterministic data generation.
const DEFAULT_SEED: u64 = 42;

/// Maximum number of log entries to retain.
const MAX_LOG_ENTRIES: usize = 1000;

/// Logs page showing real-time log viewer with follow mode.
pub struct LogsPage {
    /// The viewport for scrollable content (`RefCell` for interior mutability in view).
    viewport: RefCell<Viewport>,
    /// The log stream containing entries.
    logs: LogStream,
    /// Whether follow mode is enabled (tail -f behavior).
    following: bool,
    /// Currently selected line index (for copy/export).
    #[expect(dead_code, reason = "Reserved for future copy/export feature")]
    selected_line: Option<usize>,
    /// Current seed for data generation.
    seed: u64,
    /// Cached formatted content (`RefCell` for interior mutability in view).
    formatted_content: RefCell<String>,
    /// Whether content needs to be reformatted.
    needs_reformat: RefCell<bool>,
    /// Last known dimensions (for detecting resize).
    last_dims: RefCell<(usize, usize)>,
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

        // Start with follow mode enabled
        let mut viewport = Viewport::new(80, 24);
        viewport.mouse_wheel_enabled = true;
        viewport.mouse_wheel_delta = 3;

        Self {
            viewport: RefCell::new(viewport),
            logs,
            following: true,
            selected_line: None,
            seed,
            formatted_content: RefCell::new(String::new()),
            needs_reformat: RefCell::new(true),
            last_dims: RefCell::new((0, 0)),
        }
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

    /// Refresh logs with a new seed.
    pub fn refresh(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        let data = GeneratedData::generate(self.seed);
        self.logs = Self::generate_initial_logs(&data, self.seed);
        *self.needs_reformat.borrow_mut() = true;
        if self.following {
            self.viewport.borrow_mut().goto_bottom();
        }
    }

    /// Add a new log entry (for live updates).
    #[expect(dead_code, reason = "Reserved for simulation tick integration")]
    pub fn push_log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
        *self.needs_reformat.borrow_mut() = true;
        if self.following {
            self.viewport.borrow_mut().goto_bottom();
        }
    }

    /// Toggle follow mode.
    pub fn toggle_follow(&mut self) {
        self.following = !self.following;
        if self.following {
            self.viewport.borrow_mut().goto_bottom();
        }
    }

    /// Check if follow mode should pause (user scrolled up).
    fn check_follow_pause(&mut self) {
        if self.following && !self.viewport.borrow().at_bottom() {
            self.following = false;
        }
    }

    /// Format all log entries for display.
    fn format_logs(&self, theme: &Theme, target_width: usize) -> String {
        // Calculate optimal target width based on available space
        // timestamp (8) + level (5) + spacing (4) = 17
        let target_col_width = target_width.saturating_sub(17).clamp(10, 25);

        let formatter = LogFormatter::new(theme).with_widths(LogColumnWidths {
            timestamp: 8,
            level: 5,
            target: target_col_width,
        });

        self.logs
            .entries()
            .iter()
            .map(|entry| formatter.format(entry))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Render the status bar showing follow mode and position.
    fn render_status_bar(&self, theme: &Theme, width: usize) -> String {
        let follow_indicator = if self.following {
            theme.success_style().bold().render(" FOLLOWING ")
        } else {
            theme.warning_style().render(" PAUSED ")
        };

        let counts = self.logs.count_by_level();
        let stats = format!(
            "E:{} W:{} I:{} D:{}",
            counts.error, counts.warn, counts.info, counts.debug
        );
        let stats_styled = theme.muted_style().render(&stats);

        let viewport = self.viewport.borrow();
        let position = format!(
            "{}/{}",
            viewport.y_offset() + viewport.visible_line_count(),
            self.logs.len()
        );
        let position_styled = theme.muted_style().render(&position);

        // Calculate spacing
        let indicator_len = if self.following { 11 } else { 8 };
        let stats_len = stats.len();
        let position_len = position.len();
        let content_len = indicator_len + stats_len + position_len + 4;
        let padding = width.saturating_sub(content_len);

        format!(
            "{}{:padding$}{}  {}",
            follow_indicator,
            "",
            stats_styled,
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
            // Handle special keys
            match key.key_type {
                KeyType::Home => {
                    self.viewport.borrow_mut().goto_top();
                    self.following = false;
                    return None;
                }
                KeyType::End => {
                    self.viewport.borrow_mut().goto_bottom();
                    self.following = true;
                    return None;
                }
                KeyType::Runes => {
                    // Handle character keys
                    match key.runes.as_slice() {
                        // Toggle follow mode with 'f' or 'F'
                        ['f' | 'F'] => {
                            self.toggle_follow();
                            return None;
                        }
                        // Go to top with 'g'
                        ['g'] => {
                            self.viewport.borrow_mut().goto_top();
                            self.following = false;
                            return None;
                        }
                        // Go to bottom with 'G'
                        ['G'] => {
                            self.viewport.borrow_mut().goto_bottom();
                            self.following = true;
                            return None;
                        }
                        // Refresh with 'r' or 'R'
                        ['r' | 'R'] => {
                            self.refresh();
                            return None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Delegate to viewport for scroll handling
        self.viewport.borrow_mut().update(msg);

        // Check if scrolling paused follow mode
        self.check_follow_pause();

        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Reserve space for status bar
        let content_height = height.saturating_sub(1);

        // Check if dimensions changed or content needs reformatting
        let last_dims = *self.last_dims.borrow();
        let needs_resize = last_dims.0 != width || last_dims.1 != content_height;
        let needs_reformat = *self.needs_reformat.borrow();

        if needs_resize || needs_reformat {
            let mut viewport = self.viewport.borrow_mut();
            viewport.width = width;
            viewport.height = content_height;

            let formatted = self.format_logs(theme, width);
            viewport.set_content(&formatted);
            *self.formatted_content.borrow_mut() = formatted;
            *self.needs_reformat.borrow_mut() = false;
            *self.last_dims.borrow_mut() = (width, content_height);

            // Maintain follow mode position
            if self.following {
                viewport.goto_bottom();
            }
        }

        // Render viewport content
        let content = self.viewport.borrow().view();

        // Render status bar
        let status = self.render_status_bar(theme, width);

        // Combine with newline
        format!("{content}\n{status}")
    }

    fn page(&self) -> Page {
        Page::Logs
    }

    fn hints(&self) -> &'static str {
        "j/k scroll  f follow  g/G top/bottom  r refresh"
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        // Mark content for reformatting when page becomes active
        *self.needs_reformat.borrow_mut() = true;
        if self.following {
            self.viewport.borrow_mut().goto_bottom();
        }
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
}
