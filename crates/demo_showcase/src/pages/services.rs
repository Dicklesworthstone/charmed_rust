//! Services page.
//!
//! A lightweight "service catalog" view used by the showcase to demonstrate:
//! - Filtering
//! - List navigation
//! - A details panel layout
//!
//! This intentionally keeps its state machine small and local to the page.

#![forbid(unsafe_code)]

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::{Position, Style};

use super::PageModel;
use crate::messages::{Notification, NotificationMsg, Page};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceStatus {
    Running,
    Degraded,
    Stopped,
}

impl ServiceStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::Degraded => "DEGRADED",
            Self::Stopped => "STOPPED",
        }
    }
}

#[derive(Debug, Clone)]
struct Service {
    name: &'static str,
    kind: &'static str,
    version: &'static str,
    status: ServiceStatus,
    description: &'static str,
}

/// Services page model.
pub struct ServicesPage {
    services: Vec<Service>,
    cursor: usize,
    filtering: bool,
    filter: String,
    /// Next notification ID (shared with the app's ID space).
    next_notification_id: u64,
}

impl ServicesPage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: vec![
                Service {
                    name: "api-gateway",
                    kind: "web",
                    version: "v1.12.0",
                    status: ServiceStatus::Running,
                    description: "Routes external traffic to internal services.",
                },
                Service {
                    name: "billing-worker",
                    kind: "worker",
                    version: "v3.4.2",
                    status: ServiceStatus::Degraded,
                    description: "Processes subscription renewals and invoices.",
                },
                Service {
                    name: "search-indexer",
                    kind: "worker",
                    version: "v0.9.7",
                    status: ServiceStatus::Running,
                    description: "Builds the search index from change streams.",
                },
                Service {
                    name: "reporting-cron",
                    kind: "cron",
                    version: "v2.0.1",
                    status: ServiceStatus::Stopped,
                    description: "Generates daily reports and email digests.",
                },
                Service {
                    name: "docs-site",
                    kind: "web",
                    version: "v0.3.0",
                    status: ServiceStatus::Running,
                    description: "Internal documentation and runbooks.",
                },
            ],
            cursor: 0,
            filtering: false,
            filter: String::new(),
            next_notification_id: 2000,
        }
    }

    fn filtered_indices(&self) -> Vec<usize> {
        if self.filter.trim().is_empty() {
            return (0..self.services.len()).collect();
        }
        let term = self.filter.to_ascii_lowercase();
        self.services
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.name.to_ascii_lowercase().contains(&term)
                    || s.kind.to_ascii_lowercase().contains(&term)
                    || s.description.to_ascii_lowercase().contains(&term)
                    || s.status.label().to_ascii_lowercase().contains(&term)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn clamp_cursor(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.cursor = 0;
            return;
        }
        self.cursor = self.cursor.min(filtered_len - 1);
    }

    fn selected(&self, filtered: &[usize]) -> Option<&Service> {
        filtered
            .get(self.cursor)
            .and_then(|&idx| self.services.get(idx))
    }

    fn status_style(theme: &Theme, status: ServiceStatus) -> Style {
        match status {
            ServiceStatus::Running => theme.success_style().bold(),
            ServiceStatus::Degraded => theme.warning_style().bold(),
            ServiceStatus::Stopped => theme.error_style().bold(),
        }
    }

    fn render_list(&self, theme: &Theme, width: usize, height: usize) -> String {
        let filtered = self.filtered_indices();
        let mut out = String::new();

        let title = theme.title_style().render("Services");
        out.push_str(&title);
        out.push('\n');

        // Filter bar (1 line)
        let filter_bar = if self.filtering {
            theme
                .muted_style()
                .render(&format!("Filter: {}_", self.filter))
        } else if self.filter.is_empty() {
            theme
                .muted_style()
                .render("Press / to filter. Enter to show details.")
        } else {
            theme
                .muted_style()
                .render(&format!("Filter: {}  (Esc to clear)", self.filter))
        };
        out.push_str(&filter_bar);
        out.push('\n');

        out.push_str(&theme.muted_style().render(&"-".repeat(width.min(60))));
        out.push('\n');

        // List rows
        let available_rows = height.saturating_sub(4).max(1);
        let start = self.cursor.saturating_sub(available_rows / 2);
        let end = (start + available_rows).min(filtered.len());

        if filtered.is_empty() {
            out.push_str(&theme.muted_style().italic().render("No services match."));
            return out;
        }

        for (row, &idx) in filtered[start..end].iter().enumerate() {
            let svc = &self.services[idx];
            let selected = start + row == self.cursor;
            let indicator = if selected { ">" } else { " " };

            let status = Self::status_style(theme, svc.status).render(svc.status.label());
            let name = theme.heading_style().render(svc.name);
            let meta = theme
                .muted_style()
                .render(&format!("{} {} {}", svc.kind, svc.version, ""));

            let line = format!("{indicator} {status}  {name}  {meta}");
            if selected {
                out.push_str(&theme.box_style().padding((0, 1)).render(&line));
            } else {
                out.push_str(&line);
            }
            out.push('\n');
        }

        out
    }

    fn render_details(&self, theme: &Theme, width: usize, height: usize) -> String {
        let filtered = self.filtered_indices();
        let Some(svc) = self.selected(&filtered) else {
            return theme
                .muted_style()
                .italic()
                .render("Select a service to see details.");
        };

        let status = Self::status_style(theme, svc.status).render(svc.status.label());
        let header = format!(
            "{}\n{}",
            theme.title_style().render(svc.name),
            theme
                .muted_style()
                .render(&format!("{}  {}  {}", svc.kind, svc.version, status))
        );

        let body = format!(
            "{}\n\n{}\n{}\n{}\n",
            header,
            theme.heading_style().render("Description"),
            theme.muted_style().render(svc.description),
            theme
                .muted_style()
                .render("Actions: Enter notify  r refresh  / filter"),
        );

        let safe_w = width.max(1);
        let safe_h = height.max(1);
        let boxed = theme.box_style().padding(1).render(&body);
        lipgloss::place(safe_w, safe_h, Position::Left, Position::Top, &boxed)
    }

    fn notify_selected(&mut self) -> Option<Cmd> {
        let filtered = self.filtered_indices();
        let (name, status_label) = {
            let svc = self.selected(&filtered)?;
            (svc.name, svc.status.label())
        };
        let id = self.next_notification_id;
        self.next_notification_id += 1;
        let msg = format!("Selected service: {name} ({status_label})");
        let notif = Notification::info(id, msg);
        Some(Cmd::new(move || {
            NotificationMsg::Show(notif).into_message()
        }))
    }

    fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        if self.filtering {
            match key.key_type {
                KeyType::Esc => {
                    self.filtering = false;
                    self.filter.clear();
                    self.cursor = 0;
                    return None;
                }
                KeyType::Enter => {
                    self.filtering = false;
                    let filtered_len = self.filtered_indices().len();
                    self.clamp_cursor(filtered_len);
                    return None;
                }
                KeyType::Backspace => {
                    self.filter.pop();
                    let filtered_len = self.filtered_indices().len();
                    self.clamp_cursor(filtered_len);
                    return None;
                }
                KeyType::Runes => {
                    for c in &key.runes {
                        // Keep filter simple and predictable.
                        if c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_' {
                            self.filter.push(*c);
                        }
                    }
                    let filtered_len = self.filtered_indices().len();
                    self.clamp_cursor(filtered_len);
                    return None;
                }
                _ => return None,
            }
        }

        match key.key_type {
            KeyType::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                None
            }
            KeyType::Down => {
                let filtered_len = self.filtered_indices().len();
                if filtered_len > 0 {
                    self.cursor = (self.cursor + 1).min(filtered_len - 1);
                }
                None
            }
            KeyType::Enter => self.notify_selected(),
            KeyType::Esc => {
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.cursor = 0;
                }
                None
            }
            KeyType::Runes => match key.runes.as_slice() {
                ['/'] => {
                    self.filtering = true;
                    None
                }
                ['j'] => {
                    let filtered_len = self.filtered_indices().len();
                    if filtered_len > 0 {
                        self.cursor = (self.cursor + 1).min(filtered_len - 1);
                    }
                    None
                }
                ['k'] => {
                    self.cursor = self.cursor.saturating_sub(1);
                    None
                }
                ['r' | 'R'] => {
                    // Showcase: treat refresh as a local action and toast.
                    let id = self.next_notification_id;
                    self.next_notification_id += 1;
                    let notif = Notification::success(id, "Refreshed services");
                    Some(Cmd::new(move || {
                        NotificationMsg::Show(notif).into_message()
                    }))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl Default for ServicesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for ServicesPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        // Services page only reacts to keyboard input for now.
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            return self.handle_key(key);
        }
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        // Split into list + details.
        let left_w = (width.saturating_mul(3) / 5).max(30).min(width);
        let right_w = width.saturating_sub(left_w + 1).max(20);

        let left = self.render_list(theme, left_w, height);
        let right = self.render_details(theme, right_w, height);

        lipgloss::join_horizontal(Position::Top, &[&left, &right])
    }

    fn page(&self) -> Page {
        Page::Services
    }

    fn hints(&self) -> &'static str {
        "/ filter  j/k move  Enter notify  Esc clear"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_page_renders() {
        let page = ServicesPage::new();
        let theme = Theme::default();
        let view = page.view(120, 40, &theme);
        assert!(!view.is_empty());
        assert!(view.contains("Services"));
    }
}
