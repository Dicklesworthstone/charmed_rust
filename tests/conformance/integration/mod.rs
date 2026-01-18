//! Cross-crate integration tests
//!
//! This module contains tests that verify interactions between
//! multiple charmed_rust crates, such as:
//!
//! - bubbletea + lipgloss styling integration
//! - glamour + lipgloss theme application
//! - bubbles + bubbletea component rendering
//! - huh + bubbles form components
//! - harmonica + bubbles animation integration
//!
//! These tests verify that crates work together correctly without
//! unexpected interactions.

use bubbles::progress::Progress;
use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::textinput::TextInput;
use bubbles::viewport::Viewport;
use bubbletea::Model;
use glamour::{Renderer, Style as GlamourStyle};
use harmonica::{Spring, fps};
use huh::{Confirm, Field, Form, Group, Input, Note, Select, SelectOption};
use lipgloss::{Border, Position, Style};

// ============================================================================
// Lipgloss + Bubbletea Integration Tests
// ============================================================================

/// Test that lipgloss styles can be used in view output
fn test_lipgloss_style_in_view() -> Result<(), String> {
    // Create a styled string using lipgloss
    let style = Style::new()
        .bold()
        .foreground("#ff0000");

    let styled = style.render("Hello World");

    // Verify the output contains ANSI codes
    if !styled.contains("\x1b[") {
        return Err("Styled output should contain ANSI escape codes".to_string());
    }

    // Verify the text content is preserved
    if !styled.contains("Hello World") {
        return Err("Styled output should contain original text".to_string());
    }

    Ok(())
}

/// Test style composition with borders and padding
fn test_lipgloss_border_and_padding() -> Result<(), String> {
    let style = Style::new()
        .border(Border::rounded())
        .padding((1, 2))
        .width(20);

    let rendered = style.render("Test");

    // Should have newlines (from border/padding)
    if !rendered.contains('\n') {
        return Err("Bordered content should have multiple lines".to_string());
    }

    // Should have border characters
    if !rendered.contains('╭') && !rendered.contains('┌') {
        // Try rounded or normal corner chars
        if !rendered.contains('─') && !rendered.contains('-') {
            return Err("Border should contain border characters".to_string());
        }
    }

    Ok(())
}

/// Test join functions for layout
fn test_lipgloss_join_functions() -> Result<(), String> {
    let left = "Left";
    let right = "Right";

    // Horizontal join
    let joined = lipgloss::join_horizontal(Position::Top, &[left, right]);
    if !joined.contains("Left") || !joined.contains("Right") {
        return Err("Horizontal join should contain both strings".to_string());
    }

    // Vertical join
    let joined = lipgloss::join_vertical(Position::Left, &[left, right]);
    if !joined.contains("Left") || !joined.contains("Right") {
        return Err("Vertical join should contain both strings".to_string());
    }

    Ok(())
}

// ============================================================================
// Bubbles + Bubbletea Integration Tests
// ============================================================================

/// Test viewport component renders correctly
fn test_viewport_rendering() -> Result<(), String> {
    let mut viewport = Viewport::new(40, 10);
    let content = (1..20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    viewport.set_content(&content);

    let view = viewport.view();

    // Should render limited lines (viewport height)
    let line_count = view.lines().count();
    if line_count > 10 {
        return Err(format!(
            "Viewport should limit visible lines to height (10), got {}",
            line_count
        ));
    }

    // Content should be present
    if !view.contains("Line") {
        return Err("Viewport view should contain content".to_string());
    }

    Ok(())
}

/// Test viewport scrolling
fn test_viewport_scrolling() -> Result<(), String> {
    let mut viewport = Viewport::new(40, 5);
    let content = (1..20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    viewport.set_content(&content);

    // Initially at top
    let view_before = viewport.view();
    assert!(view_before.contains("Line 1"), "Should start with Line 1");

    // Scroll down
    viewport.scroll_down(5);
    let view_after = viewport.view();

    // Should now show different content
    if view_after.contains("Line 1") && view_before == view_after {
        return Err("Viewport content should change after scrolling".to_string());
    }

    Ok(())
}

/// Test text input component
fn test_textinput_value_handling() -> Result<(), String> {
    let mut input = TextInput::new();
    input.set_value("test input");

    let value = input.value();
    if value != "test input" {
        return Err(format!("Expected 'test input', got '{}'", value));
    }

    // Test view renders
    let view = input.view();
    if view.is_empty() {
        return Err("TextInput view should not be empty".to_string());
    }

    Ok(())
}

/// Test progress bar rendering
fn test_progress_rendering() -> Result<(), String> {
    let progress = Progress::new();

    // Render at 50%
    let view = progress.view_as(0.5);

    if view.is_empty() {
        return Err("Progress view should not be empty".to_string());
    }

    // Should contain progress characters
    if view.len() < 10 {
        return Err("Progress bar should have reasonable width".to_string());
    }

    Ok(())
}

/// Test spinner component
fn test_spinner_rendering() -> Result<(), String> {
    let spinner = SpinnerModel::with_spinner(spinners::dot());

    let view = spinner.view();

    // Spinner should render a frame
    if view.is_empty() {
        return Err("Spinner view should not be empty".to_string());
    }

    Ok(())
}

// ============================================================================
// Glamour + Lipgloss Integration Tests
// ============================================================================

/// Test glamour markdown rendering
fn test_glamour_markdown_rendering() -> Result<(), String> {
    let markdown = "# Heading\n\nThis is **bold** text.";

    let renderer = Renderer::new()
        .with_style(GlamourStyle::Dark)
        .with_word_wrap(80);

    let output = renderer.render(markdown);

    // Output should contain text
    if !output.contains("Heading") {
        return Err("Rendered markdown should contain heading text".to_string());
    }

    if !output.contains("bold") {
        return Err("Rendered markdown should contain bold text".to_string());
    }

    Ok(())
}

/// Test glamour with different styles
fn test_glamour_style_variants() -> Result<(), String> {
    let markdown = "# Test\n\nParagraph.";

    // Test each style variant renders without error
    for style in [GlamourStyle::Dark, GlamourStyle::Light, GlamourStyle::Ascii, GlamourStyle::Pink] {
        let renderer = Renderer::new().with_style(style);
        let output = renderer.render(markdown);

        if !output.contains("Test") {
            return Err(format!("Style {:?} should render content", style));
        }
    }

    Ok(())
}

/// Test glamour code block rendering
fn test_glamour_code_blocks() -> Result<(), String> {
    let markdown = "```rust\nfn main() {}\n```";

    let renderer = Renderer::new().with_style(GlamourStyle::Dark);
    let output = renderer.render(markdown);

    // Code should be present
    if !output.contains("fn main") {
        return Err("Code block content should be preserved".to_string());
    }

    Ok(())
}

// ============================================================================
// Harmonica + Bubbles Integration Tests
// ============================================================================

/// Test spring physics for progress animation
fn test_harmonica_spring_animation() -> Result<(), String> {
    let delta_time = fps(60);
    let spring = Spring::new(delta_time, 6.0, 1.0);

    // Animate from 0 to 1
    let (pos, vel) = spring.update(0.0, 0.0, 1.0);

    // Position should move toward target
    if pos <= 0.0 {
        return Err("Spring should move toward target".to_string());
    }

    // Velocity should be positive (moving toward 1)
    if vel <= 0.0 {
        return Err("Spring velocity should be positive toward target".to_string());
    }

    Ok(())
}

/// Test fps utility for animation timing
fn test_harmonica_fps_values() -> Result<(), String> {
    let delta_60 = fps(60);
    let delta_30 = fps(30);

    // 60 FPS should be ~16.67ms
    if !(0.016..0.017).contains(&delta_60) {
        return Err(format!("fps(60) should be ~0.0167, got {}", delta_60));
    }

    // 30 FPS should be ~33.33ms
    if !(0.033..0.034).contains(&delta_30) {
        return Err(format!("fps(30) should be ~0.0333, got {}", delta_30));
    }

    Ok(())
}

// ============================================================================
// Huh Form Integration Tests
// ============================================================================

/// Test huh form creation with multiple field types
fn test_huh_form_creation() -> Result<(), String> {
    let form = Form::new(vec![
        Group::new(vec![
            Box::new(Input::new().title("Username").key("username")),
            Box::new(Input::new().title("Email").key("email")),
        ]),
        Group::new(vec![
            Box::new(Select::new()
                .title("Role")
                .key("role")
                .options(vec![
                    SelectOption::new("admin", "Administrator"),
                    SelectOption::new("user", "Regular User"),
                ])),
        ]),
        Group::new(vec![
            Box::new(Confirm::new().title("Accept Terms").key("terms")),
        ]),
    ]);

    // Form should render without error
    let view = form.view();
    if view.is_empty() {
        return Err("Form view should not be empty".to_string());
    }

    Ok(())
}

/// Test huh input field rendering
fn test_huh_input_rendering() -> Result<(), String> {
    let input = Input::new()
        .title("Name")
        .placeholder("Enter your name")
        .key("name");

    let view = input.view();

    // View should contain title
    if !view.contains("Name") {
        return Err("Input view should contain title".to_string());
    }

    Ok(())
}

/// Test huh select component
fn test_huh_select_rendering() -> Result<(), String> {
    let select = Select::new()
        .title("Choose Option")
        .options(vec![
            SelectOption::new("a", "Option A"),
            SelectOption::new("b", "Option B"),
        ])
        .key("choice");

    let view = select.view();

    // View should contain title
    if !view.contains("Choose Option") {
        return Err("Select view should contain title".to_string());
    }

    Ok(())
}

/// Test huh confirm component
fn test_huh_confirm_rendering() -> Result<(), String> {
    let confirm = Confirm::new()
        .title("Are you sure?")
        .key("confirm");

    let view = confirm.view();

    // View should contain title
    if !view.contains("Are you sure") {
        return Err("Confirm view should contain title".to_string());
    }

    Ok(())
}

/// Test huh note component
fn test_huh_note_rendering() -> Result<(), String> {
    let note = Note::new()
        .title("Important")
        .description("This is a note.");

    let view = note.view();

    // View should contain title
    if !view.contains("Important") {
        return Err("Note view should contain title".to_string());
    }

    Ok(())
}

/// Test huh theming
fn test_huh_theming() -> Result<(), String> {
    // Test that themed input renders
    let mut input = Input::new()
        .title("Test")
        .key("test");
    input.with_theme(&huh::theme_charm());

    let view = input.view();

    if !view.contains("Test") {
        return Err("Themed input should render".to_string());
    }

    Ok(())
}

// ============================================================================
// E2E Scenarios
// ============================================================================

/// E2E: Viewport with glamour-rendered markdown
fn test_e2e_markdown_viewer() -> Result<(), String> {
    let markdown = "# Welcome\n\nThis is a **test** document.\n\n- Item 1\n- Item 2\n- Item 3";

    // Render markdown with glamour
    let renderer = Renderer::new()
        .with_style(GlamourStyle::Dark)
        .with_word_wrap(60);
    let rendered = renderer.render(markdown);

    // Display in viewport
    let mut viewport = Viewport::new(60, 10);
    viewport.set_content(&rendered);

    let view = viewport.view();

    // Should contain rendered content
    if !view.contains("Welcome") {
        return Err("Markdown viewer should display heading".to_string());
    }

    Ok(())
}

/// E2E: Styled progress bar with lipgloss
fn test_e2e_styled_progress() -> Result<(), String> {
    let progress = Progress::new();

    // Render progress at various levels
    let view_0 = progress.view_as(0.0);
    let view_50 = progress.view_as(0.5);
    let view_100 = progress.view_as(1.0);

    // All should render without error
    if view_0.is_empty() || view_50.is_empty() || view_100.is_empty() {
        return Err("Progress bars should render at all percentages".to_string());
    }

    // Wrap in styled container
    let container = Style::new()
        .border(Border::rounded())
        .padding((0, 1))
        .width(50);

    let styled = container.render(&view_50);

    // Should have border
    if !styled.contains('\n') {
        return Err("Styled container should add border lines".to_string());
    }

    Ok(())
}

/// E2E: Combined spinner and text
fn test_e2e_loading_indicator() -> Result<(), String> {
    let spinner = SpinnerModel::with_spinner(spinners::dot());
    let message = "Loading...";

    // Combine spinner view with text
    let combined = format!("{} {}", spinner.view(), message);

    if !combined.contains("Loading") {
        return Err("Loading indicator should contain message".to_string());
    }

    // Style the entire thing
    let style = Style::new().foreground("#00ff00");
    let styled = style.render(&combined);

    if !styled.contains("Loading") {
        return Err("Styled loading indicator should preserve content".to_string());
    }

    Ok(())
}

/// E2E: Form with styled theme
fn test_e2e_styled_form() -> Result<(), String> {
    // Create a complete form - Form shows current group only
    let form = Form::new(vec![
        Group::new(vec![
            Box::new(Note::new()
                .title("Registration")
                .description("Please fill out the form below.")),
        ]),
        Group::new(vec![
            Box::new(Input::new().title("Username").key("username")),
            Box::new(Input::new().title("Password").key("password")),
        ]),
        Group::new(vec![
            Box::new(Confirm::new().title("Remember me?").key("remember")),
        ]),
    ]);

    let view = form.view();

    // Form shows current group (first group with Registration note)
    if !view.contains("Registration") {
        return Err("Form should contain Registration note (first group)".to_string());
    }

    // Test individual group rendering
    let group = Group::new(vec![
        Box::new(Input::new().title("Username").key("username")),
        Box::new(Input::new().title("Password").key("password")),
    ]);

    let group_view = group.view();
    if !group_view.contains("Username") {
        return Err("Group should contain Username field".to_string());
    }
    if !group_view.contains("Password") {
        return Err("Group should contain Password field".to_string());
    }

    Ok(())
}

/// E2E: Layout composition with lipgloss
fn test_e2e_layout_composition() -> Result<(), String> {
    // Create multiple UI elements
    let header_style = Style::new()
        .bold()
        .foreground("#ffffff")
        .background("#333333")
        .width(40)
        .align(Position::Center);

    let body_style = Style::new()
        .padding((1, 2))
        .width(40);

    let footer_style = Style::new()
        .foreground("#888888")
        .width(40)
        .align(Position::Center);

    let header = header_style.render("My Application");
    let body = body_style.render("Content goes here");
    let footer = footer_style.render("Press q to quit");

    // Compose vertically
    let composed = lipgloss::join_vertical(
        Position::Left,
        &[&header, &body, &footer],
    );

    // Should contain all sections
    if !composed.contains("My Application") {
        return Err("Layout should contain header".to_string());
    }
    if !composed.contains("Content goes here") {
        return Err("Layout should contain body".to_string());
    }
    if !composed.contains("Press q to quit") {
        return Err("Layout should contain footer".to_string());
    }

    Ok(())
}

// ============================================================================
// Test Runner
// ============================================================================

/// Run a single test and collect result
fn run_test<F>(name: &'static str, test_fn: F, results: &mut Vec<(&'static str, Result<(), String>)>)
where
    F: FnOnce() -> Result<(), String>,
{
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_fn));
    let result = match result {
        Ok(r) => r,
        Err(_) => Err("Test panicked".to_string()),
    };
    results.push((name, result));
}

/// Run all integration tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut results = Vec::new();

    // Lipgloss + Bubbletea
    run_test("lipgloss_style_in_view", test_lipgloss_style_in_view, &mut results);
    run_test("lipgloss_border_and_padding", test_lipgloss_border_and_padding, &mut results);
    run_test("lipgloss_join_functions", test_lipgloss_join_functions, &mut results);

    // Bubbles + Bubbletea
    run_test("viewport_rendering", test_viewport_rendering, &mut results);
    run_test("viewport_scrolling", test_viewport_scrolling, &mut results);
    run_test("textinput_value_handling", test_textinput_value_handling, &mut results);
    run_test("progress_rendering", test_progress_rendering, &mut results);
    run_test("spinner_rendering", test_spinner_rendering, &mut results);

    // Glamour + Lipgloss
    run_test("glamour_markdown_rendering", test_glamour_markdown_rendering, &mut results);
    run_test("glamour_style_variants", test_glamour_style_variants, &mut results);
    run_test("glamour_code_blocks", test_glamour_code_blocks, &mut results);

    // Harmonica + Bubbles
    run_test("harmonica_spring_animation", test_harmonica_spring_animation, &mut results);
    run_test("harmonica_fps_values", test_harmonica_fps_values, &mut results);

    // Huh Forms
    run_test("huh_form_creation", test_huh_form_creation, &mut results);
    run_test("huh_input_rendering", test_huh_input_rendering, &mut results);
    run_test("huh_select_rendering", test_huh_select_rendering, &mut results);
    run_test("huh_confirm_rendering", test_huh_confirm_rendering, &mut results);
    run_test("huh_note_rendering", test_huh_note_rendering, &mut results);
    run_test("huh_theming", test_huh_theming, &mut results);

    // E2E Scenarios
    run_test("e2e_markdown_viewer", test_e2e_markdown_viewer, &mut results);
    run_test("e2e_styled_progress", test_e2e_styled_progress, &mut results);
    run_test("e2e_loading_indicator", test_e2e_loading_indicator, &mut results);
    run_test("e2e_styled_form", test_e2e_styled_form, &mut results);
    run_test("e2e_layout_composition", test_e2e_layout_composition, &mut results);

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that executes all integration tests
    #[test]
    fn test_cross_crate_integration() {
        let results = run_all_tests();

        let mut passed = 0;
        let mut failed = 0;
        let mut failures = Vec::new();

        for (name, result) in &results {
            match result {
                Ok(()) => {
                    passed += 1;
                    println!("  PASS: {}", name);
                }
                Err(msg) => {
                    failed += 1;
                    failures.push((name, msg));
                    println!("  FAIL: {} - {}", name, msg);
                }
            }
        }

        println!("\nIntegration Test Results:");
        println!("  Passed: {}", passed);
        println!("  Failed: {}", failed);
        println!("  Total:  {}", results.len());

        if !failures.is_empty() {
            println!("\nFailures:");
            for (name, msg) in &failures {
                println!("  {}: {}", name, msg);
            }
            panic!(
                "Integration tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All integration tests should pass");
    }
}
