//! Wizard page - multi-step service deployment workflow.
//!
//! This page demonstrates huh form integration by guiding users through
//! a realistic "Deploy a Service" workflow with validation and progress.
//!
//! ## Error States (bd-2fty)
//!
//! The wizard simulates realistic backend errors to demonstrate polished
//! error handling:
//!
//! - **Permission denied** - For production deployments (simulated 15% chance)
//! - **Network timeout** - Transient failures (simulated 10% chance)
//! - **Conflict** - Service name already exists (deterministic for "api-*" names)
//!
//! Recovery flows allow retrying or backing out safely.

use bubbletea::{Cmd, KeyMsg, KeyType, Message};
// Note: huh components would be used in a more complete implementation
// For now, we implement a custom form UI to demonstrate the patterns
use lipgloss::Style;

use super::PageModel;
use crate::messages::{Notification, NotificationMsg, Page, WizardDeploymentConfig, WizardMsg};
use crate::theme::Theme;

/// Service type options for deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceType {
    #[default]
    WebService,
    BackgroundWorker,
    ScheduledJob,
}

impl ServiceType {
    /// Get the display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::WebService => "Web Service",
            Self::BackgroundWorker => "Background Worker",
            Self::ScheduledJob => "Scheduled Job",
        }
    }

    /// Get the description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::WebService => "HTTP server with port binding",
            Self::BackgroundWorker => "Queue processor, no external ports",
            Self::ScheduledJob => "Cron-style recurring task",
        }
    }
}

/// Environment target for deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    Development,
    #[default]
    Staging,
    Production,
}

impl Environment {
    /// Get the display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

/// Environment variable options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub name: &'static str,
    pub description: &'static str,
}

impl Default for EnvVar {
    fn default() -> Self {
        Self {
            name: "",
            description: "",
        }
    }
}

/// Available environment variables.
const ENV_VARS: &[EnvVar] = &[
    EnvVar {
        name: "DATABASE_URL",
        description: "PostgreSQL connection string",
    },
    EnvVar {
        name: "REDIS_URL",
        description: "Redis cache connection",
    },
    EnvVar {
        name: "API_KEY",
        description: "Internal service API key",
    },
    EnvVar {
        name: "LOG_LEVEL",
        description: "Logging verbosity",
    },
    EnvVar {
        name: "METRICS_ENDPOINT",
        description: "Prometheus push gateway",
    },
    EnvVar {
        name: "SENTRY_DSN",
        description: "Error tracking endpoint",
    },
];

/// Simulated backend error types (bd-2fty).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulatedError {
    /// Permission denied (e.g., production deployment without approval).
    PermissionDenied,
    /// Network/timeout error (transient, can retry).
    NetworkTimeout,
    /// Conflict - resource already exists.
    Conflict(String),
}

impl SimulatedError {
    /// Get a user-friendly error message.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::PermissionDenied => {
                "Permission denied: production deployments require admin approval".to_string()
            }
            Self::NetworkTimeout => {
                "Network timeout: failed to reach deployment service (retryable)".to_string()
            }
            Self::Conflict(name) => {
                format!("Conflict: service '{name}' already exists in this environment")
            }
        }
    }

    /// Whether this error is retryable.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::PermissionDenied => false,
            Self::NetworkTimeout => true,
            Self::Conflict(_) => false,
        }
    }

    /// Get recovery hint for the user.
    #[must_use]
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "Press b to go back and change environment",
            Self::NetworkTimeout => "Press Enter to retry or b to go back",
            Self::Conflict(_) => "Press b to go back and change service name",
        }
    }
}

/// Deployment status.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DeploymentStatus {
    #[default]
    NotStarted,
    InProgress(usize),
    Complete,
    /// Failed with a simulated error.
    Failed(SimulatedError),
    /// Failed with a generic error message (legacy compatibility).
    FailedGeneric(String),
}

/// Wizard state containing all form data.
#[derive(Debug, Clone)]
pub struct WizardState {
    /// Current step (0-indexed).
    pub step: usize,

    /// Step 1: Service type.
    pub service_type: ServiceType,

    /// Step 2: Basic configuration.
    pub name: String,
    pub description: String,
    pub environment: Environment,

    /// Step 3: Type-specific configuration.
    pub port: String,
    pub health_check: String,
    pub replicas: usize,
    pub queue_name: String,
    pub concurrency: usize,
    pub schedule: String,
    pub timeout: String,
    pub run_on_deploy: bool,

    /// Step 4: Environment variables.
    pub env_vars: Vec<usize>,

    /// Step 5: Confirmation.
    pub confirmed: bool,

    /// Step 6: Deployment.
    pub deployment_status: DeploymentStatus,
    #[allow(dead_code)]
    pub deployment_progress: usize,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            step: 0,
            service_type: ServiceType::WebService,
            name: String::new(),
            description: String::new(),
            environment: Environment::Staging,
            port: "8080".to_string(),
            health_check: "/health".to_string(),
            replicas: 2,
            queue_name: "default".to_string(),
            concurrency: 4,
            schedule: "0 * * * *".to_string(),
            timeout: "5m".to_string(),
            run_on_deploy: false,
            env_vars: Vec::new(),
            confirmed: false,
            deployment_status: DeploymentStatus::NotStarted,
            deployment_progress: 0,
        }
    }
}

/// Total number of wizard steps.
const TOTAL_STEPS: usize = 6;

/// Step names for the progress indicator.
const STEP_NAMES: &[&str] = &["Type", "Config", "Options", "Variables", "Review", "Deploy"];

/// Wizard page for service deployment workflow.
pub struct WizardPage {
    /// Current wizard state.
    state: WizardState,
    /// Current field index within the step.
    field_index: usize,
    /// Validation error message.
    error: Option<String>,
    /// Whether the page is focused.
    focused: bool,
    /// Field-level error hints (field_index -> hint).
    field_errors: Vec<Option<String>>,
    /// Counter for pseudo-random error simulation (deterministic based on name hash).
    error_seed: u64,
    /// Number of retry attempts for current deployment.
    retry_count: u32,
    /// Next notification ID (for generating unique IDs).
    next_notification_id: u64,
}

impl WizardPage {
    /// Create a new wizard page.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: WizardState::default(),
            field_index: 0,
            error: None,
            focused: false,
            field_errors: vec![None; 10], // Pre-allocate for common field counts
            error_seed: 0,
            retry_count: 0,
            next_notification_id: 1000, // Start at 1000 to avoid collisions
        }
    }

    /// Reset the wizard to initial state.
    pub fn reset(&mut self) {
        self.state = WizardState::default();
        self.field_index = 0;
        self.error = None;
        self.field_errors = vec![None; 10];
        self.retry_count = 0;
        // Keep error_seed for reproducible behavior within session
    }

    /// Compute a deterministic error seed from the service name.
    fn compute_error_seed(&mut self) {
        // Simple hash for deterministic pseudo-random behavior
        let mut hash: u64 = 5381;
        for byte in self.state.name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte));
        }
        self.error_seed = hash;
    }

    /// Check if a simulated error should occur based on deployment context.
    ///
    /// Returns `Some(SimulatedError)` if an error should be simulated.
    fn check_simulated_error(&self) -> Option<SimulatedError> {
        // Conflict: service names starting with "api-" are always in conflict
        if self.state.name.starts_with("api-") || self.state.name == "api" {
            return Some(SimulatedError::Conflict(self.state.name.clone()));
        }

        // Use error_seed for deterministic "random" behavior
        let seed = self.error_seed.wrapping_add(u64::from(self.retry_count));

        // Permission denied: 15% chance for production, never for other envs
        if self.state.environment == Environment::Production && (seed % 7) == 0 {
            return Some(SimulatedError::PermissionDenied);
        }

        // Network timeout: 10% chance on first attempt, less on retries
        let timeout_threshold = if self.retry_count == 0 { 10 } else { 20 };
        if (seed % timeout_threshold) == 1 {
            return Some(SimulatedError::NetworkTimeout);
        }

        None
    }

    /// Clear field-level error for the current field.
    fn clear_field_error(&mut self) {
        if self.field_index < self.field_errors.len() {
            self.field_errors[self.field_index] = None;
        }
    }

    /// Set field-level error for a specific field.
    fn set_field_error(&mut self, field_index: usize, message: String) {
        if field_index >= self.field_errors.len() {
            self.field_errors.resize(field_index + 1, None);
        }
        self.field_errors[field_index] = Some(message);
    }

    /// Get field-level error for a specific field.
    fn get_field_error(&self, field_index: usize) -> Option<&String> {
        self.field_errors.get(field_index).and_then(|e| e.as_ref())
    }

    /// Move to the next step if validation passes.
    fn next_step(&mut self) {
        if let Some(err) = self.validate_current_step() {
            self.error = Some(err);
            return;
        }
        self.error = None;

        if self.state.step < TOTAL_STEPS - 1 {
            self.state.step += 1;
            self.field_index = 0;
        }
    }

    /// Move to the previous step.
    fn prev_step(&mut self) {
        self.error = None;
        if self.state.step > 0 {
            self.state.step -= 1;
            self.field_index = 0;
        }
    }

    /// Validate the current step.
    fn validate_current_step(&self) -> Option<String> {
        match self.state.step {
            0 => None, // Service type always valid (has default)
            1 => {
                // Basic configuration
                if self.state.name.trim().is_empty() {
                    return Some("Service name is required".to_string());
                }
                if self.state.name.len() < 3 {
                    return Some("Service name must be at least 3 characters".to_string());
                }
                if self.state.name.len() > 40 {
                    return Some("Service name must be 40 characters or less".to_string());
                }
                if !self
                    .state
                    .name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-')
                {
                    return Some("Service name must be alphanumeric with hyphens".to_string());
                }
                None
            }
            2 => {
                // Type-specific options
                match self.state.service_type {
                    ServiceType::WebService => {
                        if let Ok(port) = self.state.port.parse::<u16>() {
                            if port < 1024 {
                                return Some("Port must be 1024 or higher".to_string());
                            }
                        } else {
                            return Some("Invalid port number".to_string());
                        }
                        if !self.state.health_check.starts_with('/') {
                            return Some("Health check path must start with /".to_string());
                        }
                    }
                    ServiceType::BackgroundWorker => {
                        if self.state.queue_name.trim().is_empty() {
                            return Some("Queue name is required".to_string());
                        }
                    }
                    ServiceType::ScheduledJob => {
                        if self.state.schedule.trim().is_empty() {
                            return Some("Schedule is required".to_string());
                        }
                        // Basic cron validation: should have 5 space-separated parts
                        if self.state.schedule.split_whitespace().count() != 5 {
                            return Some("Invalid cron expression (need 5 fields)".to_string());
                        }
                    }
                }
                None
            }
            3 => None, // Environment variables are optional
            4 => {
                // Review step - must confirm
                if !self.state.confirmed {
                    return Some("Please confirm to proceed".to_string());
                }
                None
            }
            _ => None,
        }
    }

    /// Get the number of fields in the current step.
    fn field_count(&self) -> usize {
        match self.state.step {
            0 => 1, // Service type select
            1 => 3, // Name, description, environment
            2 => match self.state.service_type {
                ServiceType::WebService => 3,       // Port, health check, replicas
                ServiceType::BackgroundWorker => 3, // Queue name, concurrency, retries
                ServiceType::ScheduledJob => 3,     // Schedule, timeout, run on deploy
            },
            3 => 1, // MultiSelect for env vars
            4 => 1, // Confirm
            5 => 0, // Deployment (no interactive fields)
            _ => 0,
        }
    }

    /// Move to the next field within the current step.
    fn next_field(&mut self) {
        let count = self.field_count();
        if count > 0 && self.field_index < count - 1 {
            self.field_index += 1;
        }
    }

    /// Move to the previous field within the current step.
    fn prev_field(&mut self) {
        if self.field_index > 0 {
            self.field_index -= 1;
        }
    }

    /// Handle keyboard input.
    fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        // During deployment in progress, only allow viewing
        if self.state.step == 5
            && matches!(
                self.state.deployment_status,
                DeploymentStatus::InProgress(_)
            )
        {
            return None;
        }

        // Handle back navigation from failed state
        let in_failed_state = matches!(
            self.state.deployment_status,
            DeploymentStatus::Failed(_) | DeploymentStatus::FailedGeneric(_)
        );

        match key.key_type {
            KeyType::Enter => self.handle_enter(),
            KeyType::Esc => {
                if in_failed_state && self.state.step == 5 {
                    // From failed state, go back to review step
                    self.go_back_from_failure()
                } else if self.state.step > 0 {
                    self.prev_step();
                    None
                } else {
                    None
                }
            }
            KeyType::Tab => {
                self.next_field();
                None
            }
            KeyType::Up => {
                self.handle_up();
                None
            }
            KeyType::Down => {
                self.handle_down();
                None
            }
            KeyType::Runes => {
                match key.runes.as_slice() {
                    ['j'] => self.handle_down(),
                    ['k'] => self.handle_up(),
                    [' '] => self.handle_space(),
                    ['b'] => {
                        if in_failed_state && self.state.step == 5 {
                            return self.go_back_from_failure();
                        } else if self.state.step > 0 {
                            self.prev_step();
                        }
                    }
                    [c] if c.is_alphanumeric() || *c == '-' || *c == '/' || *c == '*' => {
                        self.handle_char(*c);
                    }
                    _ => {}
                }
                None
            }
            KeyType::Backspace => {
                self.handle_backspace();
                None
            }
            _ => None,
        }
    }

    /// Go back from a failed deployment state to fix the issue.
    fn go_back_from_failure(&mut self) -> Option<Cmd> {
        let error = match &self.state.deployment_status {
            DeploymentStatus::Failed(err) => Some(err.clone()),
            _ => None,
        };

        // Determine which step to go back to based on error type
        let target_step = match &error {
            Some(SimulatedError::PermissionDenied) => 1, // Go to config to change env
            Some(SimulatedError::Conflict(_)) => 1,      // Go to config to change name
            Some(SimulatedError::NetworkTimeout) => 4,   // Go to review to retry
            None => 4,                                   // Default to review
        };

        self.state.step = target_step;
        self.state.deployment_status = DeploymentStatus::NotStarted;
        self.state.confirmed = false;
        self.field_index = match &error {
            Some(SimulatedError::PermissionDenied) => 2, // Focus on environment field
            Some(SimulatedError::Conflict(_)) => 0,      // Focus on name field
            _ => 0,
        };
        self.error = None;
        self.retry_count = 0;

        // Emit navigation notification
        let id = self.next_notification_id;
        self.next_notification_id += 1;

        let message = match &error {
            Some(SimulatedError::PermissionDenied) => {
                "Returned to configuration - change environment to proceed"
            }
            Some(SimulatedError::Conflict(_)) => {
                "Returned to configuration - change service name to proceed"
            }
            _ => "Returned to review - make changes and try again",
        };

        let notification = Notification::info(id, message);
        Some(Cmd::new(move || {
            NotificationMsg::Show(notification).into_message()
        }))
    }

    /// Handle Enter key.
    ///
    /// Returns a Cmd when deployment starts to emit messages.
    fn handle_enter(&mut self) -> Option<Cmd> {
        match self.state.step {
            4 => {
                // On review step, Enter confirms
                self.state.confirmed = true;
                self.next_step();
                None
            }
            5 => {
                // On deploy step, handle various states
                match &self.state.deployment_status {
                    DeploymentStatus::NotStarted => self.start_deployment(),
                    DeploymentStatus::Complete => {
                        // Reset wizard on completion
                        self.reset();
                        None
                    }
                    DeploymentStatus::Failed(err) if err.is_retryable() => {
                        // Retry deployment for retryable errors
                        self.retry_deployment()
                    }
                    DeploymentStatus::Failed(_) | DeploymentStatus::FailedGeneric(_) => {
                        // Non-retryable: Enter does nothing, must go back
                        None
                    }
                    DeploymentStatus::InProgress(_) => {
                        // Still deploying, ignore Enter
                        None
                    }
                }
            }
            _ => {
                // Move to next step
                self.next_step();
                None
            }
        }
    }

    /// Retry a failed deployment.
    fn retry_deployment(&mut self) -> Option<Cmd> {
        self.retry_count += 1;
        self.state.deployment_status = DeploymentStatus::NotStarted;
        self.error = None;

        // Emit notification about retry
        let retry_count = self.retry_count;
        let id = self.next_notification_id;
        self.next_notification_id += 1;

        let notification = Notification::info(id, format!("Retrying deployment (attempt #{})", retry_count + 1));
        let notification_cmd = Cmd::new(move || NotificationMsg::Show(notification).into_message());

        // Start the new deployment attempt
        let start_cmd = self.start_deployment();

        match start_cmd {
            Some(cmd) => Some(Cmd::batch(vec![Some(notification_cmd), Some(cmd)])),
            None => Some(notification_cmd),
        }
    }

    /// Handle Up key.
    fn handle_up(&mut self) {
        match self.state.step {
            0 => {
                // Cycle service type
                self.state.service_type = match self.state.service_type {
                    ServiceType::WebService => ServiceType::ScheduledJob,
                    ServiceType::BackgroundWorker => ServiceType::WebService,
                    ServiceType::ScheduledJob => ServiceType::BackgroundWorker,
                };
            }
            1 => {
                // Cycle fields or environment
                if self.field_index == 2 {
                    self.state.environment = match self.state.environment {
                        Environment::Development => Environment::Production,
                        Environment::Staging => Environment::Development,
                        Environment::Production => Environment::Staging,
                    };
                } else {
                    self.prev_field();
                }
            }
            2 => {
                // Handle type-specific options
                self.handle_type_options_up();
            }
            3 => {
                // Navigate env var selection (handled by MultiSelect logic)
            }
            _ => {}
        }
    }

    /// Handle Down key.
    fn handle_down(&mut self) {
        match self.state.step {
            0 => {
                // Cycle service type
                self.state.service_type = match self.state.service_type {
                    ServiceType::WebService => ServiceType::BackgroundWorker,
                    ServiceType::BackgroundWorker => ServiceType::ScheduledJob,
                    ServiceType::ScheduledJob => ServiceType::WebService,
                };
            }
            1 => {
                // Cycle fields or environment
                if self.field_index == 2 {
                    self.state.environment = match self.state.environment {
                        Environment::Development => Environment::Staging,
                        Environment::Staging => Environment::Production,
                        Environment::Production => Environment::Development,
                    };
                } else {
                    self.next_field();
                }
            }
            2 => {
                // Handle type-specific options
                self.handle_type_options_down();
            }
            3 => {
                // Navigate env var selection
            }
            _ => {}
        }
    }

    /// Handle type-specific options Up.
    fn handle_type_options_up(&mut self) {
        match self.state.service_type {
            ServiceType::WebService => {
                if self.field_index == 2 {
                    self.state.replicas = self.state.replicas.saturating_sub(1).max(1);
                } else {
                    self.prev_field();
                }
            }
            ServiceType::BackgroundWorker => {
                if self.field_index == 1 {
                    self.state.concurrency = self.state.concurrency.saturating_sub(1).max(1);
                } else {
                    self.prev_field();
                }
            }
            ServiceType::ScheduledJob => {
                self.prev_field();
            }
        }
    }

    /// Handle type-specific options Down.
    fn handle_type_options_down(&mut self) {
        match self.state.service_type {
            ServiceType::WebService => {
                if self.field_index == 2 {
                    self.state.replicas = (self.state.replicas + 1).min(10);
                } else {
                    self.next_field();
                }
            }
            ServiceType::BackgroundWorker => {
                if self.field_index == 1 {
                    self.state.concurrency = (self.state.concurrency + 1).min(32);
                } else {
                    self.next_field();
                }
            }
            ServiceType::ScheduledJob => {
                self.next_field();
            }
        }
    }

    /// Handle Space key (toggle).
    fn handle_space(&mut self) {
        match self.state.step {
            2 if self.state.service_type == ServiceType::ScheduledJob && self.field_index == 2 => {
                self.state.run_on_deploy = !self.state.run_on_deploy;
            }
            3 => {
                // Toggle env var selection
                if let Some(idx) = self.env_var_cursor_index() {
                    if self.state.env_vars.contains(&idx) {
                        self.state.env_vars.retain(|&i| i != idx);
                    } else {
                        self.state.env_vars.push(idx);
                    }
                }
            }
            4 => {
                // Toggle confirmation
                self.state.confirmed = !self.state.confirmed;
            }
            _ => {}
        }
    }

    /// Get the env var index at current cursor.
    fn env_var_cursor_index(&self) -> Option<usize> {
        if self.state.step == 3 && self.field_index < ENV_VARS.len() {
            Some(self.field_index)
        } else {
            None
        }
    }

    /// Handle character input.
    fn handle_char(&mut self, c: char) {
        match self.state.step {
            1 => match self.field_index {
                0 => self.state.name.push(c),
                1 => self.state.description.push(c),
                _ => {}
            },
            2 => match self.state.service_type {
                ServiceType::WebService => match self.field_index {
                    0 if c.is_ascii_digit() => self.state.port.push(c),
                    1 => self.state.health_check.push(c),
                    _ => {}
                },
                ServiceType::BackgroundWorker => {
                    if self.field_index == 0 {
                        self.state.queue_name.push(c);
                    }
                }
                ServiceType::ScheduledJob => {
                    if self.field_index == 0 {
                        self.state.schedule.push(c);
                    }
                }
            },
            _ => {}
        }
        self.error = None;
    }

    /// Handle Backspace.
    fn handle_backspace(&mut self) {
        match self.state.step {
            1 => match self.field_index {
                0 => {
                    self.state.name.pop();
                }
                1 => {
                    self.state.description.pop();
                }
                _ => {}
            },
            2 => match self.state.service_type {
                ServiceType::WebService => match self.field_index {
                    0 => {
                        self.state.port.pop();
                    }
                    1 => {
                        self.state.health_check.pop();
                    }
                    _ => {}
                },
                ServiceType::BackgroundWorker => {
                    if self.field_index == 0 {
                        self.state.queue_name.pop();
                    }
                }
                ServiceType::ScheduledJob => {
                    if self.field_index == 0 {
                        self.state.schedule.pop();
                    }
                }
            },
            _ => {}
        }
        self.error = None;
    }

    /// Render the step indicator.
    fn render_step_indicator(&self, theme: &Theme, _width: usize) -> String {
        let mut parts: Vec<String> = Vec::new();

        for (i, name) in STEP_NAMES.iter().enumerate() {
            let indicator = if i < self.state.step {
                theme.success_style().render("*")
            } else if i == self.state.step {
                theme.info_style().render("@")
            } else {
                theme.muted_style().render("o")
            };

            let label = if i == self.state.step {
                theme.info_style().render(name)
            } else if i < self.state.step {
                theme.success_style().render(name)
            } else {
                theme.muted_style().render(name)
            };

            parts.push(format!("{indicator} {label}"));
        }

        let indicator = parts.join("  >  ");
        let title = theme.title_style().render("Deploy Service");
        let step_label =
            theme
                .muted_style()
                .render(&format!("Step {}/{}", self.state.step + 1, TOTAL_STEPS));

        format!("{title}  {step_label}\n\n{indicator}")
    }

    /// Render the current step content.
    fn render_step_content(&self, theme: &Theme, width: usize, _height: usize) -> String {
        match self.state.step {
            0 => self.render_step_type(theme, width),
            1 => self.render_step_config(theme, width),
            2 => self.render_step_options(theme, width),
            3 => self.render_step_env_vars(theme, width),
            4 => self.render_step_review(theme, width),
            5 => self.render_step_deploy(theme, width),
            _ => String::new(),
        }
    }

    /// Render Step 1: Service Type Selection.
    fn render_step_type(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        lines.push(theme.heading_style().render("What type of service?"));
        lines.push(String::new());

        for st in [
            ServiceType::WebService,
            ServiceType::BackgroundWorker,
            ServiceType::ScheduledJob,
        ] {
            let selected = st == self.state.service_type;
            let indicator = if selected { ">" } else { " " };
            let marker = if selected { "(*)" } else { "( )" };

            let name_style = if selected {
                theme.info_style()
            } else {
                Style::new().foreground(theme.text)
            };

            let name = name_style.render(st.name());
            let desc = theme.muted_style().render(st.description());

            lines.push(format!(" {indicator} {marker} {name}"));
            lines.push(format!("       {desc}"));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Render Step 2: Basic Configuration.
    fn render_step_config(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        lines.push(theme.heading_style().render("Basic Configuration"));
        lines.push(String::new());

        // Name field
        let name_label = if self.field_index == 0 {
            theme.info_style().render("> Name:")
        } else {
            theme.muted_style().render("  Name:")
        };
        let name_value = if self.field_index == 0 {
            format!("{}_", self.state.name)
        } else {
            self.state.name.clone()
        };
        lines.push(format!("{name_label} {name_value}"));

        // Description field
        let desc_label = if self.field_index == 1 {
            theme.info_style().render("> Description:")
        } else {
            theme.muted_style().render("  Description:")
        };
        let desc_value = if self.field_index == 1 {
            format!("{}_", self.state.description)
        } else if self.state.description.is_empty() {
            theme.muted_style().render("(optional)").to_string()
        } else {
            self.state.description.clone()
        };
        lines.push(format!("{desc_label} {desc_value}"));

        // Environment field
        let env_label = if self.field_index == 2 {
            theme.info_style().render("> Environment:")
        } else {
            theme.muted_style().render("  Environment:")
        };
        let env_style = match self.state.environment {
            Environment::Production => theme.warning_style(),
            _ => Style::new().foreground(theme.text),
        };
        let env_value = env_style.render(self.state.environment.name());
        lines.push(format!("{env_label} {env_value}"));

        if self.state.environment == Environment::Production {
            lines.push(String::new());
            lines.push(
                theme
                    .warning_style()
                    .render("  ! Production deployment requires extra confirmation"),
            );
        }

        lines.join("\n")
    }

    /// Render Step 3: Type-Specific Options.
    fn render_step_options(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        let title = format!("{} Options", self.state.service_type.name());
        lines.push(theme.heading_style().render(&title));
        lines.push(String::new());

        match self.state.service_type {
            ServiceType::WebService => {
                // Port
                let port_label = if self.field_index == 0 {
                    theme.info_style().render("> Port:")
                } else {
                    theme.muted_style().render("  Port:")
                };
                let port_value = if self.field_index == 0 {
                    format!("{}_", self.state.port)
                } else {
                    self.state.port.clone()
                };
                lines.push(format!("{port_label} {port_value}"));

                // Health check
                let health_label = if self.field_index == 1 {
                    theme.info_style().render("> Health Check:")
                } else {
                    theme.muted_style().render("  Health Check:")
                };
                let health_value = if self.field_index == 1 {
                    format!("{}_", self.state.health_check)
                } else {
                    self.state.health_check.clone()
                };
                lines.push(format!("{health_label} {health_value}"));

                // Replicas
                let replicas_label = if self.field_index == 2 {
                    theme.info_style().render("> Replicas:")
                } else {
                    theme.muted_style().render("  Replicas:")
                };
                lines.push(format!("{replicas_label} {}", self.state.replicas));
            }
            ServiceType::BackgroundWorker => {
                // Queue name
                let queue_label = if self.field_index == 0 {
                    theme.info_style().render("> Queue:")
                } else {
                    theme.muted_style().render("  Queue:")
                };
                let queue_value = if self.field_index == 0 {
                    format!("{}_", self.state.queue_name)
                } else {
                    self.state.queue_name.clone()
                };
                lines.push(format!("{queue_label} {queue_value}"));

                // Concurrency
                let conc_label = if self.field_index == 1 {
                    theme.info_style().render("> Concurrency:")
                } else {
                    theme.muted_style().render("  Concurrency:")
                };
                lines.push(format!("{conc_label} {}", self.state.concurrency));

                // Max retries (fixed at 3)
                let retries_label = if self.field_index == 2 {
                    theme.info_style().render("> Max Retries:")
                } else {
                    theme.muted_style().render("  Max Retries:")
                };
                lines.push(format!("{retries_label} 3"));
            }
            ServiceType::ScheduledJob => {
                // Schedule
                let sched_label = if self.field_index == 0 {
                    theme.info_style().render("> Schedule:")
                } else {
                    theme.muted_style().render("  Schedule:")
                };
                let sched_value = if self.field_index == 0 {
                    format!("{}_", self.state.schedule)
                } else {
                    self.state.schedule.clone()
                };
                lines.push(format!("{sched_label} {sched_value}"));
                lines.push(
                    theme
                        .muted_style()
                        .render("    (cron format: min hour day month weekday)")
                        .to_string(),
                );

                // Timeout
                let timeout_label = if self.field_index == 1 {
                    theme.info_style().render("> Timeout:")
                } else {
                    theme.muted_style().render("  Timeout:")
                };
                lines.push(format!("{timeout_label} {}", self.state.timeout));

                // Run on deploy
                let run_label = if self.field_index == 2 {
                    theme.info_style().render("> Run on Deploy:")
                } else {
                    theme.muted_style().render("  Run on Deploy:")
                };
                let run_value = if self.state.run_on_deploy {
                    "[x] Yes"
                } else {
                    "[ ] No"
                };
                lines.push(format!("{run_label} {run_value}"));
            }
        }

        lines.join("\n")
    }

    /// Render Step 4: Environment Variables.
    fn render_step_env_vars(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        lines.push(theme.heading_style().render("Environment Variables"));
        lines.push(
            theme
                .muted_style()
                .render("Select variables to inject (Space to toggle)")
                .to_string(),
        );
        lines.push(String::new());

        for (i, var) in ENV_VARS.iter().enumerate() {
            let selected = self.state.env_vars.contains(&i);
            let cursor = if i == self.field_index { ">" } else { " " };
            let checkbox = if selected { "[x]" } else { "[ ]" };

            let name_style = if i == self.field_index {
                theme.info_style()
            } else {
                Style::new().foreground(theme.text)
            };

            let name = name_style.render(var.name);
            let desc = theme.muted_style().render(var.description);

            lines.push(format!(" {cursor} {checkbox} {name}"));
            lines.push(format!("       {desc}"));
        }

        lines.join("\n")
    }

    /// Render Step 5: Review.
    fn render_step_review(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        lines.push(theme.heading_style().render("Review Deployment"));
        lines.push(String::new());

        // Summary box
        lines.push(format!(
            "  Service: {}",
            theme.info_style().render(&self.state.name)
        ));
        lines.push(format!(
            "  Type:    {}",
            theme.muted_style().render(self.state.service_type.name())
        ));

        let env_style = if self.state.environment == Environment::Production {
            theme.warning_style()
        } else {
            theme.muted_style()
        };
        lines.push(format!(
            "  Env:     {}",
            env_style.render(self.state.environment.name())
        ));

        lines.push(String::new());
        lines.push(theme.heading_style().render("Configuration:").to_string());

        match self.state.service_type {
            ServiceType::WebService => {
                lines.push(format!("  Port:         {}", self.state.port));
                lines.push(format!("  Health Check: {}", self.state.health_check));
                lines.push(format!("  Replicas:     {}", self.state.replicas));
            }
            ServiceType::BackgroundWorker => {
                lines.push(format!("  Queue:       {}", self.state.queue_name));
                lines.push(format!("  Concurrency: {}", self.state.concurrency));
                lines.push("  Max Retries: 3".to_string());
            }
            ServiceType::ScheduledJob => {
                lines.push(format!("  Schedule:      {}", self.state.schedule));
                lines.push(format!("  Timeout:       {}", self.state.timeout));
                lines.push(format!(
                    "  Run on Deploy: {}",
                    if self.state.run_on_deploy {
                        "Yes"
                    } else {
                        "No"
                    }
                ));
            }
        }

        lines.push(String::new());

        if !self.state.env_vars.is_empty() {
            lines.push(
                theme
                    .heading_style()
                    .render(&format!(
                        "Environment Variables ({}):",
                        self.state.env_vars.len()
                    ))
                    .to_string(),
            );
            for &idx in &self.state.env_vars {
                if let Some(var) = ENV_VARS.get(idx) {
                    lines.push(format!("  - {}", var.name));
                }
            }
            lines.push(String::new());
        }

        // Confirmation
        let confirm_style = if self.state.confirmed {
            theme.success_style()
        } else {
            theme.muted_style()
        };
        let checkbox = if self.state.confirmed { "[x]" } else { "[ ]" };
        lines.push(format!(
            "> {} {}",
            checkbox,
            confirm_style.render("I confirm this deployment")
        ));

        if self.state.environment == Environment::Production {
            lines.push(String::new());
            lines.push(
                theme
                    .warning_style()
                    .render("! This will deploy to PRODUCTION")
                    .to_string(),
            );
        }

        lines.join("\n")
    }

    /// Render Step 6: Deployment Progress.
    fn render_step_deploy(&self, theme: &Theme, _width: usize) -> String {
        let mut lines = Vec::new();

        lines.push(theme.heading_style().render("Deploying..."));
        lines.push(String::new());

        let steps = [
            "Validating configuration",
            "Creating container image",
            "Provisioning resources",
            "Starting service",
            "Running health checks",
        ];

        match &self.state.deployment_status {
            DeploymentStatus::NotStarted => {
                lines.push(
                    theme
                        .muted_style()
                        .render("Press Enter to begin deployment")
                        .to_string(),
                );
            }
            DeploymentStatus::InProgress(current) => {
                for (i, step) in steps.iter().enumerate() {
                    let icon = if i < *current {
                        theme.success_style().render("*")
                    } else if i == *current {
                        theme.info_style().render("@")
                    } else {
                        theme.muted_style().render("o")
                    };

                    let label = if i <= *current {
                        Style::new().foreground(theme.text).render(step)
                    } else {
                        theme.muted_style().render(step)
                    };

                    lines.push(format!("  {icon} {label}"));
                }
            }
            DeploymentStatus::Complete => {
                for step in &steps {
                    let icon = theme.success_style().render("*");
                    lines.push(format!("  {icon} {step}"));
                }
                lines.push(String::new());
                lines.push(
                    theme
                        .success_style()
                        .render("Deployment complete!")
                        .to_string(),
                );
                lines.push(String::new());
                lines.push(format!(
                    "Service '{}' is now running.",
                    theme.info_style().render(&self.state.name)
                ));
                lines.push(String::new());
                lines.push(
                    theme
                        .muted_style()
                        .render("Press Enter to deploy another service")
                        .to_string(),
                );
            }
            DeploymentStatus::Failed(err) => {
                for (i, step) in steps.iter().enumerate() {
                    let icon = if i < 2 {
                        theme.success_style().render("*")
                    } else if i == 2 {
                        theme.error_style().render("x")
                    } else {
                        theme.muted_style().render("o")
                    };
                    lines.push(format!("  {icon} {step}"));
                }
                lines.push(String::new());
                lines.push(theme.error_style().render("Deployment failed!").to_string());
                lines.push(format!("Error: {err}"));
            }
        }

        lines.join("\n")
    }

    /// Render error message if any.
    fn render_error(&self, theme: &Theme) -> Option<String> {
        self.error
            .as_ref()
            .map(|e| theme.error_style().render(&format!("! {e}")).to_string())
    }

    /// Create a deployment config from the current wizard state.
    fn deployment_config(&self) -> WizardDeploymentConfig {
        let env_var_names: Vec<String> = self
            .state
            .env_vars
            .iter()
            .filter_map(|&idx| ENV_VARS.get(idx))
            .map(|v| v.name.to_string())
            .collect();

        WizardDeploymentConfig {
            service_name: self.state.name.clone(),
            service_type: self.state.service_type.name().to_string(),
            environment: self.state.environment.name().to_string(),
            env_vars: env_var_names,
        }
    }

    /// Start the deployment and emit the DeploymentStarted message.
    ///
    /// Returns a Cmd that emits `WizardMsg::DeploymentStarted`.
    pub fn start_deployment(&mut self) -> Option<Cmd> {
        if self.state.deployment_status != DeploymentStatus::NotStarted {
            return None;
        }

        // Compute error seed for deterministic error simulation
        self.compute_error_seed();

        self.state.deployment_status = DeploymentStatus::InProgress(0);
        let config = self.deployment_config();

        Some(Cmd::new(move || {
            WizardMsg::DeploymentStarted(config).into_message()
        }))
    }

    /// Tick the deployment progress (called from app tick).
    ///
    /// Returns a Cmd if deployment state changed (progress or completion).
    pub fn tick_deployment(&mut self) -> Option<Cmd> {
        if let DeploymentStatus::InProgress(step) = self.state.deployment_status {
            if step < 4 {
                self.state.deployment_status = DeploymentStatus::InProgress(step + 1);
                Some(Cmd::new(move || {
                    WizardMsg::DeploymentProgress(step as u8 + 1).into_message()
                }))
            } else {
                self.state.deployment_status = DeploymentStatus::Complete;
                let config = self.deployment_config();
                Some(Cmd::new(move || {
                    WizardMsg::DeploymentCompleted(config).into_message()
                }))
            }
        } else {
            None
        }
    }

    /// Check if deployment is in progress.
    #[must_use]
    pub const fn is_deploying(&self) -> bool {
        matches!(
            self.state.deployment_status,
            DeploymentStatus::InProgress(_)
        )
    }
}

impl Default for WizardPage {
    fn default() -> Self {
        Self::new()
    }
}

impl PageModel for WizardPage {
    fn update(&mut self, msg: &Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            return self.handle_key(key);
        }
        None
    }

    fn view(&self, width: usize, height: usize, theme: &Theme) -> String {
        let indicator = self.render_step_indicator(theme, width);
        let content = self.render_step_content(theme, width, height);
        let error = self.render_error(theme).unwrap_or_default();

        let separator = theme.muted_style().render(&"-".repeat(width.min(60)));

        let mut result = format!("{indicator}\n{separator}\n\n{content}");

        if !error.is_empty() {
            result.push_str(&format!("\n\n{error}"));
        }

        result
    }

    fn page(&self) -> Page {
        Page::Wizard
    }

    fn hints(&self) -> &'static str {
        match self.state.step {
            0 => "j/k select  Enter continue  Esc back",
            1 | 2 => "j/k fields  Enter continue  b back",
            3 => "j/k navigate  Space toggle  Enter continue  b back",
            4 => "Space confirm  Enter deploy  b back",
            5 => match self.state.deployment_status {
                DeploymentStatus::NotStarted => "Enter start",
                DeploymentStatus::InProgress(_) => "deploying...",
                DeploymentStatus::Complete => "Enter new wizard",
                DeploymentStatus::Failed(_) => "Enter retry  b back",
            },
            _ => "j/k navigate  Enter select",
        }
    }

    fn on_enter(&mut self) -> Option<Cmd> {
        self.focused = true;
        None
    }

    fn on_leave(&mut self) -> Option<Cmd> {
        self.focused = false;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_initial_state() {
        let page = WizardPage::new();
        assert_eq!(page.state.step, 0);
        assert_eq!(page.state.service_type, ServiceType::WebService);
    }

    #[test]
    fn wizard_validates_empty_name() {
        let mut page = WizardPage::new();
        page.state.step = 1;
        page.state.name = String::new();

        let err = page.validate_current_step();
        assert!(err.is_some());
        assert!(err.unwrap().contains("required"));
    }

    #[test]
    fn wizard_validates_short_name() {
        let mut page = WizardPage::new();
        page.state.step = 1;
        page.state.name = "ab".to_string();

        let err = page.validate_current_step();
        assert!(err.is_some());
        assert!(err.unwrap().contains("3 characters"));
    }

    #[test]
    fn wizard_validates_port_range() {
        let mut page = WizardPage::new();
        page.state.step = 2;
        page.state.service_type = ServiceType::WebService;
        page.state.port = "80".to_string();

        let err = page.validate_current_step();
        assert!(err.is_some());
        assert!(err.unwrap().contains("1024"));
    }

    #[test]
    fn wizard_reset_clears_state() {
        let mut page = WizardPage::new();
        page.state.step = 3;
        page.state.name = "my-service".to_string();
        page.state.env_vars = vec![0, 1, 2];

        page.reset();

        assert_eq!(page.state.step, 0);
        assert!(page.state.name.is_empty());
        assert!(page.state.env_vars.is_empty());
    }

    #[test]
    fn wizard_deployment_progress() {
        let mut page = WizardPage::new();
        page.state.deployment_status = DeploymentStatus::InProgress(0);

        page.tick_deployment();
        assert_eq!(
            page.state.deployment_status,
            DeploymentStatus::InProgress(1)
        );

        page.tick_deployment();
        page.tick_deployment();
        page.tick_deployment();
        page.tick_deployment();

        assert_eq!(page.state.deployment_status, DeploymentStatus::Complete);
    }

    #[test]
    fn wizard_start_deployment_returns_cmd() {
        let mut page = WizardPage::new();
        page.state.name = "test-service".to_string();
        page.state.deployment_status = DeploymentStatus::NotStarted;

        // Start deployment should return a command
        let cmd = page.start_deployment();
        assert!(cmd.is_some());

        // Status should now be InProgress
        assert!(matches!(
            page.state.deployment_status,
            DeploymentStatus::InProgress(0)
        ));

        // Starting again should return None
        let cmd2 = page.start_deployment();
        assert!(cmd2.is_none());
    }

    #[test]
    fn wizard_tick_returns_cmds() {
        let mut page = WizardPage::new();
        page.state.name = "test-service".to_string();

        // Starting returns cmd
        let start_cmd = page.start_deployment();
        assert!(start_cmd.is_some());

        // Each tick returns progress cmd
        for _ in 0..4 {
            let tick_cmd = page.tick_deployment();
            assert!(tick_cmd.is_some());
        }

        // Final tick returns completion cmd and sets Complete
        let final_cmd = page.tick_deployment();
        assert!(final_cmd.is_some());
        assert_eq!(page.state.deployment_status, DeploymentStatus::Complete);

        // Ticking after complete returns None
        let noop = page.tick_deployment();
        assert!(noop.is_none());
    }

    #[test]
    fn wizard_deployment_config_captures_state() {
        let mut page = WizardPage::new();
        page.state.name = "my-api".to_string();
        page.state.service_type = ServiceType::WebService;
        page.state.environment = Environment::Production;
        page.state.env_vars = vec![0, 2]; // DATABASE_URL and API_KEY

        let config = page.deployment_config();

        assert_eq!(config.service_name, "my-api");
        assert_eq!(config.service_type, "Web Service");
        assert_eq!(config.environment, "production");
        assert_eq!(config.env_vars.len(), 2);
        assert!(config.env_vars.contains(&"DATABASE_URL".to_string()));
        assert!(config.env_vars.contains(&"API_KEY".to_string()));
    }
}
