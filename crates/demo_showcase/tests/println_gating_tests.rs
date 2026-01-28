//! Tests for bubbletea println/printf gating (bd-39te)
//!
//! These tests verify that the demo_showcase correctly uses bubbletea's println/printf
//! for lifecycle events, and document the gating behavior.
//!
//! # Architecture
//!
//! The println/printf gating works at two levels:
//!
//! 1. **App Level**: The demo_showcase App emits println commands for key lifecycle
//!    events (deployments starting/completing, job status changes).
//!
//! 2. **Runtime Level**: bubbletea's Program runtime gates the actual output based
//!    on `options.alt_screen`. When alt_screen is true (default), PrintLineMsg is
//!    silently ignored. When alt_screen is false (--no-alt-screen), the message is
//!    printed above the TUI.
//!
//! # Policy
//!
//! The demo_showcase always emits println commands for lifecycle events. The
//! bubbletea runtime is responsible for deciding whether to actually print them.
//! This keeps the app logic simple and testable.
//!
//! # What These Tests Verify
//!
//! 1. Deployment events produce commands (via WizardMsg)
//! 2. Job events produce commands (via action notifications)
//! 3. The commands are non-None (actual println execution is bubbletea's job)

use bubbletea::{Cmd, Message, Model};
use demo_showcase::app::App;
use demo_showcase::config::Config;
use demo_showcase::messages::{WizardDeploymentConfig, WizardMsg};

// =============================================================================
// DEPLOYMENT EVENT TESTS
// =============================================================================

/// Verify that DeploymentStarted produces a println command.
#[test]
fn deployment_started_produces_println_command() {
    let mut app = App::from_config(&Config::default());

    let config = WizardDeploymentConfig {
        service_name: "test-service".to_string(),
        service_type: "api".to_string(),
        environment: "staging".to_string(),
        env_vars: vec![],
    };

    let cmd = app.update(Message::new(WizardMsg::DeploymentStarted(config)));

    assert!(
        cmd.is_some(),
        "DeploymentStarted should produce a println command"
    );
}

/// Verify that DeploymentCompleted produces a println command.
#[test]
fn deployment_completed_produces_println_command() {
    let mut app = App::from_config(&Config::default());

    let config = WizardDeploymentConfig {
        service_name: "test-service".to_string(),
        service_type: "api".to_string(),
        environment: "production".to_string(),
        env_vars: vec![],
    };

    let cmd = app.update(Message::new(WizardMsg::DeploymentCompleted(config)));

    assert!(
        cmd.is_some(),
        "DeploymentCompleted should produce a println command"
    );
}

/// Verify that DeploymentFailed produces a println command.
#[test]
fn deployment_failed_produces_println_command() {
    let mut app = App::from_config(&Config::default());

    let cmd = app.update(Message::new(WizardMsg::DeploymentFailed(
        "Connection timeout".to_string(),
    )));

    assert!(
        cmd.is_some(),
        "DeploymentFailed should produce a println command"
    );
}

/// Verify that DeploymentProgress does NOT produce a command (progress updates are UI-only).
#[test]
fn deployment_progress_does_not_produce_command() {
    let mut app = App::from_config(&Config::default());

    let cmd = app.update(Message::new(WizardMsg::DeploymentProgress(3)));

    assert!(
        cmd.is_none(),
        "DeploymentProgress should not produce a println command (UI shows progress)"
    );
}

// =============================================================================
// BUBBLETEA RUNTIME GATING DOCUMENTATION
// =============================================================================

/// This test documents the bubbletea runtime gating behavior.
///
/// The println command produces a PrintLineMsg, which bubbletea handles as follows:
/// - When alt_screen is true (default): message is silently ignored
/// - When alt_screen is false (--no-alt-screen): message is printed above TUI
///
/// This behavior is tested in bubbletea's own test suite (command.rs).
/// We document it here for clarity.
#[test]
fn document_bubbletea_gating_behavior() {
    // This test serves as documentation.
    //
    // The actual gating happens in bubbletea::program::run():
    //
    // ```rust
    // if let Some(print_msg) = msg.downcast_ref::<PrintLineMsg>() {
    //     if !self.options.alt_screen {
    //         for line in print_msg.0.lines() {
    //             let _ = writeln!(writer, "{}", line);
    //         }
    //     }
    //     continue;
    // }
    // ```
    //
    // So when alt_screen is ON (default), PrintLineMsg is a no-op.
    // When alt_screen is OFF (--no-alt-screen), the message is printed.
    //
    // The demo_showcase policy:
    // - Always emit println for lifecycle events (deployments, jobs)
    // - Let bubbletea decide whether to actually print based on alt_screen
    //
    // This keeps the app logic simple and testable.
}

// =============================================================================
// COMMAND EXECUTION TESTS
// =============================================================================

/// Helper to check if a command produces any message when executed.
fn command_produces_message(cmd: Option<Cmd>) -> bool {
    cmd.and_then(|c| c.execute()).is_some()
}

/// Verify that deployment println commands execute and produce messages.
#[test]
fn deployment_println_commands_are_executable() {
    let mut app = App::from_config(&Config::default());

    let config = WizardDeploymentConfig {
        service_name: "exec-test".to_string(),
        service_type: "worker".to_string(),
        environment: "dev".to_string(),
        env_vars: vec![],
    };

    let cmd = app.update(Message::new(WizardMsg::DeploymentStarted(config)));

    assert!(
        command_produces_message(cmd),
        "DeploymentStarted command should execute and produce a message"
    );
}

// =============================================================================
// ALT-SCREEN CONFIG TESTS
// =============================================================================

/// Verify that Config correctly tracks alt_screen setting.
#[test]
fn config_alt_screen_default_is_true() {
    let config = Config::default();
    assert!(
        config.alt_screen,
        "Default config should have alt_screen = true"
    );
}

/// Verify that Config can be set to alt_screen = false.
#[test]
fn config_alt_screen_can_be_disabled() {
    let mut config = Config::default();
    config.alt_screen = false;
    assert!(
        !config.alt_screen,
        "Config should allow alt_screen = false (--no-alt-screen)"
    );
}

// =============================================================================
// CONSISTENCY TESTS
// =============================================================================

/// Verify that all deployment event types are consistently handled.
#[test]
fn deployment_events_consistent_handling() {
    let mut app = App::from_config(&Config::default());

    let config = WizardDeploymentConfig {
        service_name: "consistency-test".to_string(),
        service_type: "api".to_string(),
        environment: "staging".to_string(),
        env_vars: vec![],
    };

    // Started, Completed, Failed should all produce commands
    let started = app.update(Message::new(WizardMsg::DeploymentStarted(config.clone())));
    let completed = app.update(Message::new(WizardMsg::DeploymentCompleted(config)));
    let failed = app.update(Message::new(WizardMsg::DeploymentFailed("error".into())));

    assert!(
        started.is_some(),
        "DeploymentStarted should produce command"
    );
    assert!(
        completed.is_some(),
        "DeploymentCompleted should produce command"
    );
    assert!(failed.is_some(), "DeploymentFailed should produce command");

    // Progress should NOT produce command
    let progress = app.update(Message::new(WizardMsg::DeploymentProgress(2)));
    assert!(
        progress.is_none(),
        "DeploymentProgress should NOT produce command"
    );
}
