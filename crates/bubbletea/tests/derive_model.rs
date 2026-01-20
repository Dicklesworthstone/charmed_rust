//! Integration tests for the `#[derive(Model)]` macro.

// Allow these lints for Model trait conformance in test models
#![allow(clippy::unused_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::unnecessary_wraps)]

use bubbletea::{Cmd, Message};

// Test: Basic derive with all required methods
#[derive(bubbletea::Model)]
struct Counter {
    count: i32,
}

impl Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(&delta) = msg.downcast_ref::<i32>() {
            self.count += delta;
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

// Test: Struct with multiple fields and #[state] attribute
#[derive(bubbletea::Model)]
struct AppWithState {
    #[state]
    text: String,
    #[state]
    cursor: usize,
    #[allow(dead_code)] // Intentionally unused to test non-state fields
    internal: bool,
}

impl AppWithState {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(text) = msg.downcast_ref::<String>() {
            self.text = text.clone();
        }
        None
    }

    fn view(&self) -> String {
        format!("Text: {} (cursor: {})", self.text, self.cursor)
    }
}

// Test: Struct with generics
#[derive(bubbletea::Model)]
struct GenericModel<T: Clone + Send + 'static> {
    #[allow(dead_code)] // Field unused in view for this test
    value: T,
}

impl<T: Clone + Send + 'static> GenericModel<T> {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        "Generic view".to_string()
    }
}

// Test: Struct with lifetime (static bound required by Model trait)
#[derive(bubbletea::Model)]
struct StaticModel {
    data: &'static str,
}

impl StaticModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        self.data.to_string()
    }
}

// Test: Struct with where clause
#[derive(bubbletea::Model)]
struct WhereClauseModel<T>
where
    T: std::fmt::Display + Clone + Send + 'static,
{
    item: T,
}

// Test: Generic struct with #[state] attribute
#[derive(bubbletea::Model)]
struct GenericWithState<T>
where
    T: Clone + PartialEq + Send + 'static,
{
    #[state]
    value: T,
    #[allow(dead_code)]
    metadata: String,
}

impl<T> WhereClauseModel<T>
where
    T: std::fmt::Display + Clone + Send + 'static,
{
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("{}", self.item)
    }
}

impl<T> GenericWithState<T>
where
    T: Clone + PartialEq + Send + 'static,
{
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        "generic".to_string()
    }
}

#[test]
fn test_counter_model_trait() {
    let mut counter = Counter { count: 0 };

    // Test Model trait methods
    assert!(counter.init().is_none());
    assert_eq!(counter.view(), "Count: 0");

    // Update with a message
    counter.update(Message::new(5i32));
    assert_eq!(counter.count, 5);
    assert_eq!(counter.view(), "Count: 5");

    counter.update(Message::new(-3i32));
    assert_eq!(counter.count, 2);
}

#[test]
fn test_app_with_state_model_trait() {
    let mut app = AppWithState {
        text: String::new(),
        cursor: 0,
        internal: false,
    };

    assert!(app.init().is_none());
    assert_eq!(app.view(), "Text:  (cursor: 0)");

    app.update(Message::new("Hello".to_string()));
    assert_eq!(app.text, "Hello");
    assert_eq!(app.view(), "Text: Hello (cursor: 0)");
}

#[test]
fn test_generic_model_trait() {
    let model: GenericModel<i32> = GenericModel { value: 42 };

    assert!(model.init().is_none());
    assert_eq!(model.view(), "Generic view");
}

#[test]
fn test_static_model_trait() {
    let model = StaticModel {
        data: "static data",
    };

    assert!(model.init().is_none());
    assert_eq!(model.view(), "static data");
}

#[test]
fn test_where_clause_model_trait() {
    let model: WhereClauseModel<String> = WhereClauseModel {
        item: "formatted".to_string(),
    };

    assert!(model.init().is_none());
    assert_eq!(model.view(), "formatted");
}

#[test]
fn test_hidden_type_name_method() {
    // The derive macro generates a hidden __bubbletea_type_name method
    assert_eq!(Counter::__bubbletea_type_name(), "Counter");
    assert_eq!(AppWithState::__bubbletea_type_name(), "AppWithState");
}

#[test]
fn test_state_tracking_methods() {
    let app = AppWithState {
        text: "hello".to_string(),
        cursor: 5,
        internal: false,
    };

    // Snapshot the initial state
    let snapshot = app.__snapshot_state();

    // No changes yet - should return false
    assert!(!app.__state_changed(&snapshot));

    // Modify the struct
    let mut app2 = AppWithState {
        text: "hello".to_string(),
        cursor: 5,
        internal: true, // internal is not tracked
    };

    // internal field change shouldn't trigger state changed (not tracked)
    assert!(!app2.__state_changed(&snapshot));

    // Change a tracked field
    app2.text = "world".to_string();
    assert!(app2.__state_changed(&snapshot));

    // Reset text, change cursor
    app2.text = "hello".to_string();
    app2.cursor = 10;
    assert!(app2.__state_changed(&snapshot));
}

#[test]
fn test_no_state_tracking_for_counter() {
    let counter = Counter { count: 0 };

    // Counter has no #[state] fields, so __state_changed always returns false
    // __snapshot_state() returns () for models with no tracked fields
    counter.__snapshot_state();
    assert!(!counter.__state_changed(&()));

    let counter2 = Counter { count: 100 };
    // Even with different values, returns false since no fields are tracked
    assert!(!counter2.__state_changed(&()));
}

#[test]
fn test_generic_state_tracking() {
    let model: GenericWithState<i32> = GenericWithState {
        value: 42,
        metadata: "test".to_string(),
    };

    // Snapshot the initial state
    let snapshot = model.__snapshot_state();
    assert!(!model.__state_changed(&snapshot));

    // Different value should trigger change
    let model2: GenericWithState<i32> = GenericWithState {
        value: 100,
        metadata: "test".to_string(),
    };
    assert!(model2.__state_changed(&snapshot));

    // Same value, different metadata should not trigger (metadata not tracked)
    let model3: GenericWithState<i32> = GenericWithState {
        value: 42,
        metadata: "different".to_string(),
    };
    assert!(!model3.__state_changed(&snapshot));
}

// =============================================================================
// E2E Tests: Realistic Application Patterns
// =============================================================================

// Counter Application E2E Test
mod counter_app_e2e {
    use bubbletea::{Cmd, Message};

    #[derive(Clone, Debug)]
    enum CounterMsg {
        Increment,
        Decrement,
        Reset,
    }

    #[derive(bubbletea::Model)]
    struct CounterApp {
        #[state]
        count: i32,
    }

    impl CounterApp {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(counter_msg) = msg.downcast_ref::<CounterMsg>() {
                match counter_msg {
                    CounterMsg::Increment => self.count += 1,
                    CounterMsg::Decrement => self.count -= 1,
                    CounterMsg::Reset => self.count = 0,
                }
            }
            None
        }

        fn view(&self) -> String {
            format!(
                "Count: {}\n\nPress +/- to change, r to reset, q to quit",
                self.count
            )
        }
    }

    #[test]
    fn test_counter_app_full_lifecycle() {
        let mut app = CounterApp { count: 0 };

        // Test init() from derive macro
        let init_cmd = app.init();
        assert!(init_cmd.is_none(), "Default init should return None");

        // Test initial view
        assert!(app.view().contains("Count: 0"));

        // Test update cycle
        app.update(Message::new(CounterMsg::Increment));
        assert_eq!(app.count, 1);

        app.update(Message::new(CounterMsg::Increment));
        app.update(Message::new(CounterMsg::Increment));
        assert_eq!(app.count, 3);

        app.update(Message::new(CounterMsg::Decrement));
        assert_eq!(app.count, 2);

        // Test view reflects state
        let view = app.view();
        assert!(view.contains('2'));

        // Test reset
        app.update(Message::new(CounterMsg::Reset));
        assert_eq!(app.count, 0);
    }

    #[test]
    fn test_counter_state_change_detection() {
        let app = CounterApp { count: 5 };
        let snapshot = app.__snapshot_state();

        // Same state, no change
        assert!(!app.__state_changed(&snapshot));

        // Different count, should detect change
        let app2 = CounterApp { count: 10 };
        assert!(app2.__state_changed(&snapshot));
    }
}

// Todo Application E2E Test
mod todo_app_e2e {
    use bubbletea::{Cmd, Message};

    #[derive(Clone, Debug, PartialEq)]
    struct TodoItem {
        text: String,
        completed: bool,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)] // Realistic enum - not all variants used in tests
    enum TodoMsg {
        Add(String),
        Toggle(usize),
        Delete(usize),
        SelectNext,
        SelectPrev,
        Loaded(Vec<TodoItem>),
    }

    #[derive(bubbletea::Model)]
    struct TodoApp {
        #[state]
        items: Vec<TodoItem>,
        #[state]
        selected: usize,
        #[state]
        input_mode: bool,
        input_buffer: String, // Not tracked - doesn't trigger re-render
    }

    impl TodoApp {
        fn init(&self) -> Option<Cmd> {
            // Default init returns None
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(todo_msg) = msg.downcast_ref::<TodoMsg>() {
                match todo_msg {
                    TodoMsg::Add(text) => {
                        self.items.push(TodoItem {
                            text: text.clone(),
                            completed: false,
                        });
                        self.input_mode = false;
                    }
                    TodoMsg::Toggle(idx) => {
                        if let Some(item) = self.items.get_mut(*idx) {
                            item.completed = !item.completed;
                        }
                    }
                    TodoMsg::Delete(idx) => {
                        if *idx < self.items.len() {
                            self.items.remove(*idx);
                        }
                    }
                    TodoMsg::SelectNext => {
                        if self.selected < self.items.len().saturating_sub(1) {
                            self.selected += 1;
                        }
                    }
                    TodoMsg::SelectPrev => {
                        self.selected = self.selected.saturating_sub(1);
                    }
                    TodoMsg::Loaded(items) => {
                        self.items = items.clone();
                    }
                }
            }
            None
        }

        fn view(&self) -> String {
            use std::fmt::Write;
            let mut output = String::from("Todo List\n\n");

            for (i, item) in self.items.iter().enumerate() {
                let checkbox = if item.completed { "[x]" } else { "[ ]" };
                let cursor = if i == self.selected { ">" } else { " " };
                let _ = writeln!(output, "{cursor} {checkbox} {}", item.text);
            }

            output
        }
    }

    #[test]
    fn test_todo_app_operations() {
        let mut app = TodoApp {
            items: vec![],
            selected: 0,
            input_mode: false,
            input_buffer: String::new(),
        };

        // Init should return None
        let init_cmd = app.init();
        assert!(init_cmd.is_none());

        // Add items
        app.update(Message::new(TodoMsg::Add("Task 1".to_string())));
        app.update(Message::new(TodoMsg::Add("Task 2".to_string())));
        app.update(Message::new(TodoMsg::Add("Task 3".to_string())));

        assert_eq!(app.items.len(), 3);

        // Toggle completion
        app.update(Message::new(TodoMsg::Toggle(0)));
        assert!(app.items[0].completed);
        assert!(!app.items[1].completed);

        // Select navigation
        app.update(Message::new(TodoMsg::SelectNext));
        assert_eq!(app.selected, 1);
        app.update(Message::new(TodoMsg::SelectNext));
        assert_eq!(app.selected, 2);
        // Boundary check - can't go past end
        app.update(Message::new(TodoMsg::SelectNext));
        assert_eq!(app.selected, 2);

        // Delete item
        app.update(Message::new(TodoMsg::Delete(1)));
        assert_eq!(app.items.len(), 2);
        assert_eq!(app.items[1].text, "Task 3");

        // Test view renders correctly
        let view = app.view();
        assert!(view.contains("Task 1"));
        assert!(view.contains("[x]")); // Completed item
    }

    #[test]
    fn test_todo_loaded_data() {
        let mut app = TodoApp {
            items: vec![],
            selected: 0,
            input_mode: false,
            input_buffer: String::new(),
        };

        // Simulate loaded data
        app.update(Message::new(TodoMsg::Loaded(vec![
            TodoItem {
                text: "Loaded 1".into(),
                completed: false,
            },
            TodoItem {
                text: "Loaded 2".into(),
                completed: true,
            },
        ])));

        assert_eq!(app.items.len(), 2);
        assert!(!app.items[0].completed);
        assert!(app.items[1].completed);
    }

    #[test]
    fn test_todo_state_tracking() {
        let app = TodoApp {
            items: vec![TodoItem {
                text: "Test".into(),
                completed: false,
            }],
            selected: 0,
            input_mode: false,
            input_buffer: "typing...".to_string(),
        };

        let snapshot = app.__snapshot_state();

        // Same tracked state, different input_buffer should NOT trigger change
        let mut app2 = app.clone();
        app2.input_buffer = "different buffer".to_string();
        assert!(!app2.__state_changed(&snapshot));

        // Different items SHOULD trigger change
        let mut app3 = app;
        app3.items.push(TodoItem {
            text: "New".into(),
            completed: false,
        });
        assert!(app3.__state_changed(&snapshot));
    }

    impl Clone for TodoApp {
        fn clone(&self) -> Self {
            Self {
                items: self.items.clone(),
                selected: self.selected,
                input_mode: self.input_mode,
                input_buffer: self.input_buffer.clone(),
            }
        }
    }
}

// Form Application E2E Test
mod form_app_e2e {
    use bubbletea::{Cmd, Message};

    #[derive(Clone, Debug)]
    #[allow(dead_code)] // Realistic enum - not all variants used in tests
    enum FormMsg {
        SetName(String),
        SetEmail(String),
        SetPassword(String),
        Submit,
        ValidationError(Vec<String>),
        SubmitSuccess,
    }

    #[derive(bubbletea::Model)]
    struct RegistrationForm {
        #[state]
        name: String,
        #[state]
        email: String,
        #[state]
        password: String,
        #[state]
        errors: Vec<String>,
        #[state]
        submitted: bool,
    }

    impl RegistrationForm {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(form_msg) = msg.downcast_ref::<FormMsg>() {
                match form_msg {
                    FormMsg::SetName(n) => self.name = n.clone(),
                    FormMsg::SetEmail(e) => self.email = e.clone(),
                    FormMsg::SetPassword(p) => self.password = p.clone(),
                    FormMsg::Submit => {
                        let errors = self.validate();
                        if errors.is_empty() {
                            // In real app, would return async command
                            self.submitted = true;
                        } else {
                            self.errors = errors;
                        }
                    }
                    FormMsg::ValidationError(e) => self.errors = e.clone(),
                    FormMsg::SubmitSuccess => self.submitted = true,
                }
            }
            None
        }

        fn validate(&self) -> Vec<String> {
            let mut errors = Vec::new();
            if self.name.len() < 2 {
                errors.push("Name too short".into());
            }
            if !self.email.contains('@') {
                errors.push("Invalid email".into());
            }
            if self.password.len() < 8 {
                errors.push("Password too short".into());
            }
            errors
        }

        fn view(&self) -> String {
            use std::fmt::Write;
            let mut output = String::from("Registration Form\n\n");
            let _ = writeln!(output, "Name: {}", self.name);
            let _ = writeln!(output, "Email: {}", self.email);
            output.push_str("Password: ****\n");

            if !self.errors.is_empty() {
                output.push_str("\nErrors:\n");
                for err in &self.errors {
                    let _ = writeln!(output, "- {err}");
                }
            }

            if self.submitted {
                output.push_str("\nSubmitted successfully!");
            }

            output
        }
    }

    #[test]
    fn test_form_validation_flow() {
        let mut form = RegistrationForm {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            errors: vec![],
            submitted: false,
        };

        // Submit empty form - should fail validation
        form.update(Message::new(FormMsg::Submit));
        assert_eq!(form.errors.len(), 3, "Should have 3 validation errors");
        assert!(form.errors.iter().any(|e| e.contains("Name")));
        assert!(form.errors.iter().any(|e| e.contains("email")));
        assert!(form.errors.iter().any(|e| e.contains("Password")));
        assert!(!form.submitted);

        // Fill in valid data
        form.update(Message::new(FormMsg::SetName("John Doe".into())));
        form.update(Message::new(FormMsg::SetEmail("john@example.com".into())));
        form.update(Message::new(FormMsg::SetPassword("securepass123".into())));
        form.errors.clear();

        // Submit valid form
        form.update(Message::new(FormMsg::Submit));
        assert!(form.errors.is_empty(), "Valid form should have no errors");
        assert!(form.submitted, "Form should be marked as submitted");

        // Verify view
        let view = form.view();
        assert!(view.contains("John Doe"));
        assert!(view.contains("john@example.com"));
        assert!(view.contains("Submitted successfully"));
    }

    #[test]
    fn test_form_partial_validation() {
        let mut form = RegistrationForm {
            name: "Jo".to_string(),        // Valid (2+ chars)
            email: "invalid".to_string(),  // Invalid (no @)
            password: "short".to_string(), // Invalid (< 8 chars)
            errors: vec![],
            submitted: false,
        };

        form.update(Message::new(FormMsg::Submit));
        assert_eq!(form.errors.len(), 2, "Should have 2 validation errors");
        assert!(!form.errors.iter().any(|e| e.contains("Name")));
        assert!(form.errors.iter().any(|e| e.contains("email")));
        assert!(form.errors.iter().any(|e| e.contains("Password")));
    }
}

// Complex Nested Model Test
mod nested_model_e2e {
    use bubbletea::{Cmd, Message};

    #[derive(Clone, PartialEq)]
    struct SidebarModel {
        items: Vec<String>,
        selected: usize,
    }

    #[derive(Clone, PartialEq)]
    struct ContentModel {
        text: String,
    }

    #[derive(Clone, PartialEq)]
    struct FooterModel {
        status: String,
    }

    #[derive(Clone, Debug)]
    enum DashboardMsg {
        SelectSidebar(usize),
        SetContent(String),
        SetStatus(String),
    }

    #[derive(bubbletea::Model)]
    struct Dashboard {
        #[state]
        sidebar: SidebarModel,
        #[state]
        main_content: ContentModel,
        #[state]
        footer: FooterModel,
    }

    impl Dashboard {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(dashboard_msg) = msg.downcast_ref::<DashboardMsg>() {
                match dashboard_msg {
                    DashboardMsg::SelectSidebar(idx) => {
                        if *idx < self.sidebar.items.len() {
                            self.sidebar.selected = *idx;
                        }
                    }
                    DashboardMsg::SetContent(text) => {
                        self.main_content.text = text.clone();
                    }
                    DashboardMsg::SetStatus(status) => {
                        self.footer.status = status.clone();
                    }
                }
            }
            None
        }

        fn view(&self) -> String {
            use std::fmt::Write;
            let mut output = String::new();

            // Sidebar
            output.push_str("| Sidebar |\n");
            for (i, item) in self.sidebar.items.iter().enumerate() {
                let marker = if i == self.sidebar.selected { ">" } else { " " };
                let _ = writeln!(output, "{marker} {item}");
            }

            // Main content
            output.push_str("\n| Content |\n");
            output.push_str(&self.main_content.text);
            output.push('\n');

            // Footer
            let _ = write!(output, "\n| Status: {} |", self.footer.status);

            output
        }
    }

    #[test]
    fn test_complex_nested_model() {
        let mut dashboard = Dashboard {
            sidebar: SidebarModel {
                items: vec!["Home".into(), "Settings".into(), "Profile".into()],
                selected: 0,
            },
            main_content: ContentModel {
                text: "Welcome to the dashboard!".into(),
            },
            footer: FooterModel {
                status: "Ready".into(),
            },
        };

        // Should compile and work with nested state
        let _init = dashboard.init();
        let view = dashboard.view();

        assert!(view.contains("Home"));
        assert!(view.contains("Welcome"));
        assert!(view.contains("Ready"));

        // Test updates to nested models
        dashboard.update(Message::new(DashboardMsg::SelectSidebar(1)));
        assert_eq!(dashboard.sidebar.selected, 1);

        dashboard.update(Message::new(DashboardMsg::SetContent(
            "Settings page".into(),
        )));
        assert_eq!(dashboard.main_content.text, "Settings page");

        dashboard.update(Message::new(DashboardMsg::SetStatus("Modified".into())));
        let view = dashboard.view();
        assert!(view.contains("Modified"));
    }

    #[test]
    fn test_nested_state_change_detection() {
        let dashboard = Dashboard {
            sidebar: SidebarModel {
                items: vec!["Home".into()],
                selected: 0,
            },
            main_content: ContentModel {
                text: "Test".into(),
            },
            footer: FooterModel {
                status: "Ready".into(),
            },
        };

        let snapshot = dashboard.__snapshot_state();

        // Same state, no change
        assert!(!dashboard.__state_changed(&snapshot));

        // Change nested sidebar selection
        let mut dashboard2 = Dashboard {
            sidebar: SidebarModel {
                items: vec!["Home".into()],
                selected: 1, // Changed!
            },
            main_content: ContentModel {
                text: "Test".into(),
            },
            footer: FooterModel {
                status: "Ready".into(),
            },
        };

        assert!(dashboard2.__state_changed(&snapshot));

        // Change only footer
        dashboard2.sidebar.selected = 0;
        dashboard2.footer.status = "Changed".into();
        assert!(dashboard2.__state_changed(&snapshot));
    }
}

// Async Command Pattern Test (without actually running async)
mod async_pattern_e2e {
    use bubbletea::{Cmd, Message};

    #[derive(Clone, Debug)]
    enum LoaderMsg {
        StartLoading,
        DataLoaded(String),
        Error(String),
    }

    #[derive(bubbletea::Model)]
    struct DataLoader {
        #[state]
        data: Option<String>,
        #[state]
        loading: bool,
        #[state]
        error: Option<String>,
    }

    impl DataLoader {
        fn init(&self) -> Option<Cmd> {
            // Return command to start loading
            Some(Cmd::new(|| Message::new(LoaderMsg::StartLoading)))
        }

        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(loader_msg) = msg.downcast_ref::<LoaderMsg>() {
                match loader_msg {
                    LoaderMsg::StartLoading => {
                        self.loading = true;
                        self.error = None;
                        // In real app: return async command to fetch data
                        None
                    }
                    LoaderMsg::DataLoaded(data) => {
                        self.loading = false;
                        self.data = Some(data.clone());
                        None
                    }
                    LoaderMsg::Error(err) => {
                        self.loading = false;
                        self.error = Some(err.clone());
                        None
                    }
                }
            } else {
                None
            }
        }

        fn view(&self) -> String {
            if self.loading {
                "Loading...".to_string()
            } else if let Some(ref err) = self.error {
                format!("Error: {err}")
            } else if let Some(ref data) = self.data {
                format!("Data: {data}")
            } else {
                "No data".to_string()
            }
        }
    }

    #[test]
    fn test_async_loading_pattern() {
        let mut loader = DataLoader {
            data: None,
            loading: false,
            error: None,
        };

        // Init should return a command
        let init_cmd = loader.init();
        assert!(init_cmd.is_some());

        // Simulate loading started
        loader.update(Message::new(LoaderMsg::StartLoading));
        assert!(loader.loading);
        assert!(loader.error.is_none());
        assert_eq!(loader.view(), "Loading...");

        // Simulate data loaded
        loader.update(Message::new(LoaderMsg::DataLoaded("API Response".into())));
        assert!(!loader.loading);
        assert_eq!(loader.data, Some("API Response".into()));
        assert!(loader.view().contains("API Response"));
    }

    #[test]
    fn test_async_error_pattern() {
        let mut loader = DataLoader {
            data: None,
            loading: false,
            error: None,
        };

        // Start loading
        loader.update(Message::new(LoaderMsg::StartLoading));
        assert!(loader.loading);

        // Simulate error
        loader.update(Message::new(LoaderMsg::Error("Network error".into())));
        assert!(!loader.loading);
        assert!(loader.data.is_none());
        assert_eq!(loader.error, Some("Network error".into()));
        assert!(loader.view().contains("Error: Network error"));
    }

    #[test]
    fn test_loader_state_tracking() {
        let loader = DataLoader {
            data: Some("cached".into()),
            loading: false,
            error: None,
        };

        let snapshot = loader.__snapshot_state();

        // Same state
        assert!(!loader.__state_changed(&snapshot));

        // Loading state change
        let loader2 = DataLoader {
            data: Some("cached".into()),
            loading: true,
            error: None,
        };
        assert!(loader2.__state_changed(&snapshot));
    }
}
