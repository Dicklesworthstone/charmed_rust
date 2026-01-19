//! Form Example
//!
//! This example demonstrates:
//! - Using huh to build interactive forms
//! - Text input with validation
//! - Select dropdown and multi-select
//! - Confirmation step before submission
//! - Field navigation with Tab/Shift+Tab
//!
//! Run with: `cargo run -p example-form`

#![forbid(unsafe_code)]

use bubbletea::Program;
use huh::{Confirm, Form, Group, Input, MultiSelect, Select, SelectOption, new_options};

fn main() -> anyhow::Result<()> {
    // Create the registration form with multiple groups
    let form = Form::new(vec![
        // Group 1: Personal Information
        Group::new(vec![
            Box::new(
                Input::new()
                    .key("name")
                    .title("Full Name")
                    .description("Enter your full name")
                    .placeholder("John Doe")
                    .validate(|s| {
                        if s.trim().len() < 2 {
                            Some("Name must be at least 2 characters".to_string())
                        } else {
                            None
                        }
                    }),
            ),
            Box::new(
                Input::new()
                    .key("email")
                    .title("Email Address")
                    .description("We'll use this to contact you")
                    .placeholder("john@example.com")
                    .validate(|s| {
                        if !s.contains('@') || !s.contains('.') {
                            Some("Please enter a valid email address".to_string())
                        } else {
                            None
                        }
                    }),
            ),
        ])
        .title("Personal Information")
        .description("Tell us about yourself"),
        // Group 2: Preferences
        Group::new(vec![
            Box::new(
                Select::<String>::new()
                    .key("country")
                    .title("Country")
                    .description("Select your country")
                    .options(new_options(vec![
                        "United States",
                        "Canada",
                        "United Kingdom",
                        "Germany",
                        "France",
                        "Japan",
                        "Australia",
                        "Other",
                    ])),
            ),
            Box::new(
                MultiSelect::<String>::new()
                    .key("interests")
                    .title("Interests")
                    .description("Select all that apply")
                    .options(vec![
                        SelectOption::new("Programming", "programming".to_string()),
                        SelectOption::new("Open Source", "opensource".to_string()),
                        SelectOption::new("Terminal Apps", "terminal".to_string()),
                        SelectOption::new("Web Development", "webdev".to_string()),
                        SelectOption::new("Systems Programming", "systems".to_string()),
                        SelectOption::new("DevOps", "devops".to_string()),
                    ]),
            ),
        ])
        .title("Preferences")
        .description("Help us personalize your experience"),
        // Group 3: Confirmation
        Group::new(vec![Box::new(
            Confirm::new()
                .key("confirm")
                .title("Ready to submit?")
                .description("Review your information above")
                .affirmative("Submit")
                .negative("Go Back"),
        )])
        .title("Confirmation"),
    ])
    .width(60);

    // Run the form
    Program::new(form).with_alt_screen().run()?;

    println!("\nForm completed! Thank you for registering.");
    Ok(())
}
