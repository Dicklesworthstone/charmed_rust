//! End-to-end tests for the derive macro with realistic model implementations.
//!
//! These tests verify the macro works correctly in practice with real TUI application patterns.

// Allow these lints for Model trait conformance in test models
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]

use bubbletea::{Cmd, Message};
use std::fmt::Write;

// ============================================================================
// Test 1: Full Counter Application Lifecycle
// ============================================================================

/// Message types for the counter app
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CounterMsg {
    Increment,
    Decrement,
    Reset,
    SetValue(i32),
}

/// Counter app demonstrating basic derive usage with state tracking
#[derive(bubbletea::Model)]
struct CounterApp {
    #[state]
    count: i32,
    #[state]
    last_action: String,
    // Non-tracked internal state
    update_count: usize,
}

impl CounterApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.update_count += 1;

        if let Some(counter_msg) = msg.downcast_ref::<CounterMsg>() {
            match counter_msg {
                CounterMsg::Increment => {
                    self.count += 1;
                    self.last_action = "incremented".to_string();
                }
                CounterMsg::Decrement => {
                    self.count -= 1;
                    self.last_action = "decremented".to_string();
                }
                CounterMsg::Reset => {
                    self.count = 0;
                    self.last_action = "reset".to_string();
                }
                CounterMsg::SetValue(v) => {
                    self.count = *v;
                    self.last_action = format!("set to {v}");
                }
            }
        }
        None
    }

    fn view(&self) -> String {
        format!(
            "Counter: {}\nLast action: {}\nTotal updates: {}",
            self.count, self.last_action, self.update_count
        )
    }
}

#[test]
fn test_counter_app_full_lifecycle() {
    let mut app = CounterApp {
        count: 0,
        last_action: "initialized".to_string(),
        update_count: 0,
    };

    // Test init
    assert!(app.init().is_none());
    assert_eq!(app.count, 0);

    // Take initial snapshot
    let snapshot = app.__snapshot_state();

    // Test increment cycle
    app.update(Message::new(CounterMsg::Increment));
    assert_eq!(app.count, 1);
    assert_eq!(app.last_action, "incremented");
    assert!(app.__state_changed(&snapshot)); // State should have changed

    // Multiple operations
    app.update(Message::new(CounterMsg::Increment));
    app.update(Message::new(CounterMsg::Increment));
    assert_eq!(app.count, 3);

    app.update(Message::new(CounterMsg::Decrement));
    assert_eq!(app.count, 2);

    // Reset
    app.update(Message::new(CounterMsg::Reset));
    assert_eq!(app.count, 0);
    assert_eq!(app.last_action, "reset");

    // Set specific value
    app.update(Message::new(CounterMsg::SetValue(42)));
    assert_eq!(app.count, 42);

    // View should render correctly
    let view = app.view();
    assert!(view.contains("42"));
    assert!(view.contains("set to 42"));
    assert!(view.contains("Total updates: 6"));
}

#[test]
fn test_counter_state_tracking_precision() {
    let mut app = CounterApp {
        count: 10,
        last_action: "start".to_string(),
        update_count: 0,
    };

    let snapshot1 = app.__snapshot_state();

    // Non-tracked field change should NOT trigger state change
    app.update_count = 999;
    assert!(
        !app.__state_changed(&snapshot1),
        "Non-tracked field should not trigger state change"
    );

    // Tracked field change SHOULD trigger
    app.count = 11;
    assert!(
        app.__state_changed(&snapshot1),
        "Tracked field should trigger state change"
    );

    // Snapshot again and verify
    let snapshot2 = app.__snapshot_state();
    assert!(
        !app.__state_changed(&snapshot2),
        "Same state should not be different"
    );
}

// ============================================================================
// Test 2: Todo List Application
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum TodoMsg {
    AddItem(String),
    ToggleItem(usize),
    DeleteItem(usize),
    ClearCompleted,
    SelectNext,
    SelectPrev,
}

#[derive(bubbletea::Model)]
struct TodoApp {
    #[state]
    items: Vec<TodoItem>,
    #[state]
    selected_idx: usize,
    #[state]
    filter_completed: bool,
    // Non-tracked
    next_id: usize,
    #[allow(dead_code)]
    input_buffer: String,
}

impl TodoApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(todo_msg) = msg.downcast_ref::<TodoMsg>() {
            match todo_msg {
                TodoMsg::AddItem(text) => {
                    self.items.push(TodoItem {
                        id: self.next_id,
                        text: text.clone(),
                        completed: false,
                    });
                    self.next_id += 1;
                }
                TodoMsg::ToggleItem(id) => {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == *id) {
                        item.completed = !item.completed;
                    }
                }
                TodoMsg::DeleteItem(id) => {
                    self.items.retain(|i| i.id != *id);
                    if self.selected_idx > 0 && self.selected_idx >= self.items.len() {
                        self.selected_idx = self.items.len().saturating_sub(1);
                    }
                }
                TodoMsg::ClearCompleted => {
                    self.items.retain(|i| !i.completed);
                    self.selected_idx = 0;
                }
                TodoMsg::SelectNext => {
                    if self.selected_idx < self.items.len().saturating_sub(1) {
                        self.selected_idx += 1;
                    }
                }
                TodoMsg::SelectPrev => {
                    self.selected_idx = self.selected_idx.saturating_sub(1);
                }
            }
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::from("== Todo List ==\n\n");

        if self.items.is_empty() {
            output.push_str("  No items\n");
        } else {
            for (idx, item) in self.items.iter().enumerate() {
                let cursor = if idx == self.selected_idx { ">" } else { " " };
                let checkbox = if item.completed { "[x]" } else { "[ ]" };
                let _ = writeln!(output, "{cursor} {checkbox} {}", item.text);
            }
        }

        let completed = self.items.iter().filter(|i| i.completed).count();
        let total = self.items.len();
        let _ = write!(output, "\n{completed}/{total} completed");

        output
    }

    fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.completed).count()
    }

    fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| !i.completed).count()
    }
}

#[test]
fn test_todo_app_full_workflow() {
    let mut app = TodoApp {
        items: vec![],
        selected_idx: 0,
        filter_completed: false,
        next_id: 1,
        input_buffer: String::new(),
    };

    // Add items
    app.update(Message::new(TodoMsg::AddItem("Buy groceries".to_string())));
    app.update(Message::new(TodoMsg::AddItem("Write tests".to_string())));
    app.update(Message::new(TodoMsg::AddItem("Review PR".to_string())));

    assert_eq!(app.items.len(), 3);
    assert_eq!(app.completed_count(), 0);
    assert_eq!(app.pending_count(), 3);

    // Toggle first item
    app.update(Message::new(TodoMsg::ToggleItem(1)));
    assert!(app.items[0].completed);
    assert_eq!(app.completed_count(), 1);

    // Navigation
    app.update(Message::new(TodoMsg::SelectNext));
    assert_eq!(app.selected_idx, 1);

    app.update(Message::new(TodoMsg::SelectNext));
    assert_eq!(app.selected_idx, 2);

    // Boundary - can't go past end
    app.update(Message::new(TodoMsg::SelectNext));
    assert_eq!(app.selected_idx, 2);

    // Delete item
    app.update(Message::new(TodoMsg::DeleteItem(2))); // Delete second item
    assert_eq!(app.items.len(), 2);
    assert_eq!(app.selected_idx, 1); // Adjusted

    // Clear completed
    app.update(Message::new(TodoMsg::ClearCompleted));
    assert_eq!(app.items.len(), 1);
    assert_eq!(app.completed_count(), 0);

    // View renders correctly
    let view = app.view();
    assert!(view.contains("Todo List"));
    assert!(view.contains("Review PR"));
}

#[test]
fn test_todo_state_tracking() {
    let mut app = TodoApp {
        items: vec![TodoItem {
            id: 1,
            text: "Test".to_string(),
            completed: false,
        }],
        selected_idx: 0,
        filter_completed: false,
        next_id: 2,
        input_buffer: String::new(),
    };

    let snapshot = app.__snapshot_state();

    // Changing non-tracked next_id should not trigger change
    app.next_id = 100;
    assert!(!app.__state_changed(&snapshot));

    // Changing tracked items should trigger
    app.items[0].completed = true;
    assert!(app.__state_changed(&snapshot));
}

// ============================================================================
// Test 3: Form with Validation
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum FormMsg {
    SetName(String),
    SetEmail(String),
    SetAge(String),
    Submit,
    ClearErrors,
}

#[derive(bubbletea::Model)]
struct RegistrationForm {
    #[state]
    name: String,
    #[state]
    email: String,
    #[state]
    age: Option<u32>,
    #[state]
    errors: Vec<String>,
    #[state]
    submitted: bool,
    // Non-tracked
    dirty: bool,
}

impl RegistrationForm {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(form_msg) = msg.downcast_ref::<FormMsg>() {
            match form_msg {
                FormMsg::SetName(n) => {
                    self.name.clone_from(n);
                    self.dirty = true;
                }
                FormMsg::SetEmail(e) => {
                    self.email.clone_from(e);
                    self.dirty = true;
                }
                FormMsg::SetAge(a) => {
                    self.age = a.parse().ok();
                    self.dirty = true;
                }
                FormMsg::Submit => {
                    let validation_errors = self.validate();
                    if validation_errors.is_empty() {
                        self.submitted = true;
                        self.errors.clear();
                    } else {
                        self.errors = validation_errors;
                    }
                }
                FormMsg::ClearErrors => {
                    self.errors.clear();
                }
            }
        }
        None
    }

    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.trim().len() < 2 {
            errors.push("Name must be at least 2 characters".to_string());
        }

        if !self.email.contains('@') || !self.email.contains('.') {
            errors.push("Invalid email format".to_string());
        }

        if let Some(age) = self.age {
            if !(13..=120).contains(&age) {
                errors.push("Age must be between 13 and 120".to_string());
            }
        } else {
            errors.push("Age is required".to_string());
        }

        errors
    }

    fn view(&self) -> String {
        let mut output = String::from("== Registration Form ==\n\n");

        let _ = writeln!(output, "Name:  {}", self.name);
        let _ = writeln!(output, "Email: {}", self.email);
        let _ = writeln!(
            output,
            "Age:   {}",
            self.age.map(|a| a.to_string()).unwrap_or_default()
        );

        if !self.errors.is_empty() {
            output.push_str("\nErrors:\n");
            for err in &self.errors {
                let _ = writeln!(output, "  - {err}");
            }
        }

        if self.submitted {
            output.push_str("\n[Form submitted successfully!]");
        }

        output
    }
}

#[test]
fn test_form_validation_flow() {
    let mut form = RegistrationForm {
        name: String::new(),
        email: String::new(),
        age: None,
        errors: vec![],
        submitted: false,
        dirty: false,
    };

    // Submit empty form
    form.update(Message::new(FormMsg::Submit));
    assert!(!form.submitted);
    assert_eq!(form.errors.len(), 3); // name, email, age

    // Fill in invalid data
    form.update(Message::new(FormMsg::SetName("J".to_string()))); // Too short
    form.update(Message::new(FormMsg::SetEmail("invalid".to_string()))); // No @
    form.update(Message::new(FormMsg::SetAge("5".to_string()))); // Too young

    form.update(Message::new(FormMsg::Submit));
    assert!(!form.submitted);
    assert_eq!(form.errors.len(), 3);

    // Fill in valid data
    form.update(Message::new(FormMsg::SetName("John Doe".to_string())));
    form.update(Message::new(FormMsg::SetEmail(
        "john@example.com".to_string(),
    )));
    form.update(Message::new(FormMsg::SetAge("25".to_string())));

    form.update(Message::new(FormMsg::Submit));
    assert!(form.submitted);
    assert!(form.errors.is_empty());

    // View shows success
    let view = form.view();
    assert!(view.contains("John Doe"));
    assert!(view.contains("submitted successfully"));
}

#[test]
fn test_form_state_tracking() {
    let mut form = RegistrationForm {
        name: "Test".to_string(),
        email: "test@test.com".to_string(),
        age: Some(25),
        errors: vec![],
        submitted: false,
        dirty: false,
    };

    let snapshot = form.__snapshot_state();

    // Non-tracked dirty flag
    form.dirty = true;
    assert!(!form.__state_changed(&snapshot));

    // Tracked name
    form.name = "Changed".to_string();
    assert!(form.__state_changed(&snapshot));
}

// ============================================================================
// Test 4: Nested/Complex Model
// ============================================================================

#[derive(Clone, PartialEq)]
struct MenuItem {
    label: String,
    enabled: bool,
}

#[derive(Clone, PartialEq)]
struct StatusBar {
    message: String,
    level: String,
}

#[derive(Debug, Clone)]
enum DashboardMsg {
    SelectMenu(usize),
    SetStatus(String, String),
    ToggleMenuItem(usize),
}

#[derive(bubbletea::Model)]
struct Dashboard {
    #[state]
    menu_items: Vec<MenuItem>,
    #[state]
    selected_menu: usize,
    #[state]
    status: StatusBar,
    #[state]
    content: String,
    // Non-tracked cache
    render_cache: Option<String>,
}

impl Dashboard {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.render_cache = None; // Invalidate cache

        if let Some(dash_msg) = msg.downcast_ref::<DashboardMsg>() {
            match dash_msg {
                DashboardMsg::SelectMenu(idx) => {
                    if *idx < self.menu_items.len() {
                        self.selected_menu = *idx;
                        self.content = format!("Viewing: {}", self.menu_items[*idx].label);
                    }
                }
                DashboardMsg::SetStatus(msg, level) => {
                    self.status = StatusBar {
                        message: msg.clone(),
                        level: level.clone(),
                    };
                }
                DashboardMsg::ToggleMenuItem(idx) => {
                    if let Some(item) = self.menu_items.get_mut(*idx) {
                        item.enabled = !item.enabled;
                    }
                }
            }
        }
        None
    }

    fn view(&self) -> String {
        let mut output = String::from("== Dashboard ==\n\n");

        output.push_str("Menu:\n");
        for (idx, item) in self.menu_items.iter().enumerate() {
            let cursor = if idx == self.selected_menu { ">" } else { " " };
            let status = if item.enabled { "+" } else { "-" };
            let _ = writeln!(output, "{cursor} [{status}] {}", item.label);
        }

        let _ = writeln!(output, "\nContent: {}", self.content);
        let _ = write!(
            output,
            "Status: [{}] {}",
            self.status.level, self.status.message
        );

        output
    }
}

#[test]
fn test_dashboard_complex_state() {
    let mut dash = Dashboard {
        menu_items: vec![
            MenuItem {
                label: "Home".to_string(),
                enabled: true,
            },
            MenuItem {
                label: "Settings".to_string(),
                enabled: true,
            },
            MenuItem {
                label: "Profile".to_string(),
                enabled: false,
            },
        ],
        selected_menu: 0,
        status: StatusBar {
            message: "Ready".to_string(),
            level: "info".to_string(),
        },
        content: "Welcome!".to_string(),
        render_cache: None,
    };

    let snapshot = dash.__snapshot_state();

    // Select different menu
    dash.update(Message::new(DashboardMsg::SelectMenu(1)));
    assert_eq!(dash.selected_menu, 1);
    assert!(dash.content.contains("Settings"));
    assert!(dash.__state_changed(&snapshot));

    // Toggle menu item
    let snapshot2 = dash.__snapshot_state();
    dash.update(Message::new(DashboardMsg::ToggleMenuItem(2)));
    assert!(dash.menu_items[2].enabled);
    assert!(dash.__state_changed(&snapshot2));

    // Update status
    dash.update(Message::new(DashboardMsg::SetStatus(
        "Operation complete".to_string(),
        "success".to_string(),
    )));
    assert_eq!(dash.status.message, "Operation complete");

    // Non-tracked cache should not trigger change
    let snapshot3 = dash.__snapshot_state();
    dash.render_cache = Some("cached".to_string());
    assert!(!dash.__state_changed(&snapshot3));

    // View renders all components
    let view = dash.view();
    assert!(view.contains("Dashboard"));
    assert!(view.contains("Home"));
    assert!(view.contains("Settings"));
    assert!(view.contains("success"));
}

// ============================================================================
// Test 5: Custom Equality with State
// ============================================================================

// Custom eq function must take references per the macro's eq callback signature
#[allow(clippy::trivially_copy_pass_by_ref)]
fn float_approx_eq(a: &f64, b: &f64) -> bool {
    (a - b).abs() < 0.01
}

#[derive(bubbletea::Model)]
struct ProgressTracker {
    #[state(eq = "float_approx_eq")]
    progress: f64,
    #[state]
    label: String,
    #[state(skip)]
    last_update_ms: u64,
}

impl ProgressTracker {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&delta) = msg.downcast_ref::<f64>() {
            self.progress = (self.progress + delta).clamp(0.0, 1.0);
            self.last_update_ms = 12345; // Simulated timestamp
        }
        None
    }

    fn view(&self) -> String {
        let bar_width = 20;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let filled = ((self.progress * bar_width as f64) as usize).min(bar_width);
        let empty = bar_width - filled;
        format!(
            "{}: [{}{}] {:.0}%",
            self.label,
            "=".repeat(filled),
            " ".repeat(empty),
            self.progress * 100.0
        )
    }
}

#[test]
fn test_custom_equality_state_tracking() {
    let mut tracker = ProgressTracker {
        progress: 0.5,
        label: "Download".to_string(),
        last_update_ms: 0,
    };

    let snapshot = tracker.__snapshot_state();

    // Small change within tolerance - should NOT trigger state change
    tracker.progress = 0.505;
    assert!(
        !tracker.__state_changed(&snapshot),
        "Small progress change should be ignored"
    );

    // Larger change - SHOULD trigger state change
    tracker.progress = 0.6;
    assert!(
        tracker.__state_changed(&snapshot),
        "Large progress change should trigger"
    );

    // Skipped field should never trigger change
    let snapshot2 = tracker.__snapshot_state();
    tracker.last_update_ms = 99999;
    assert!(
        !tracker.__state_changed(&snapshot2),
        "Skipped field should not trigger change"
    );

    // But label change should still work
    tracker.label = "Upload".to_string();
    assert!(
        tracker.__state_changed(&snapshot2),
        "Label change should trigger"
    );
}

#[test]
fn test_progress_view_rendering() {
    let tracker = ProgressTracker {
        progress: 0.75,
        label: "Installing".to_string(),
        last_update_ms: 0,
    };

    let view = tracker.view();
    assert!(view.contains("Installing"));
    assert!(view.contains("75%"));
    assert!(view.contains('='));
}
