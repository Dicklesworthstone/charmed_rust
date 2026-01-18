//! Conformance tests for the bubbletea crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of the Elm-architecture TUI framework matches
//! the behavior of the original Go library.
//!
//! Test categories:
//! - KeyType: Key type enum values and string representations
//! - Key sequences: ANSI escape sequence parsing
//! - Mouse buttons: Mouse button enum values
//! - Mouse actions: Mouse action enum values
//! - Mouse events: Mouse event string serialization
//! - Mouse parsing: X10 and SGR mouse protocol escape sequence parsing
//! - Key strings: KeyMsg Display implementation
//! - Command model: batch/sequence command semantics

use crate::harness::{FixtureLoader, TestFixture};
use bubbletea::message::{BatchMsg, SequenceMsg};
use bubbletea::{
    Cmd, KeyMsg, KeyType, Message, MouseAction, MouseButton, MouseMsg, batch,
    parse_mouse_event_sequence, parse_sequence, sequence,
};
use serde::Deserialize;

/// Input for key type tests
#[derive(Debug, Deserialize)]
struct KeyTypeInput {
    key_type: String,
}

/// Expected output for key type tests
#[derive(Debug, Deserialize)]
struct KeyTypeOutput {
    string_name: String,
    value: i16,
}

/// Input for key sequence tests
#[derive(Debug, Deserialize)]
struct SequenceInput {
    sequence: String,
}

/// Expected output for key sequence tests
#[derive(Debug, Deserialize)]
struct SequenceOutput {
    alt: bool,
    key_type: i16,
}

/// Input for mouse button tests
#[derive(Debug, Deserialize)]
struct MouseButtonInput {
    button: String,
}

/// Expected output for mouse button tests
#[derive(Debug, Deserialize)]
struct MouseButtonOutput {
    value: u8,
}

/// Input for mouse action tests
#[derive(Debug, Deserialize)]
struct MouseActionInput {
    action: String,
}

/// Expected output for mouse action tests
#[derive(Debug, Deserialize)]
struct MouseActionOutput {
    value: u8,
}

/// Input for mouse event tests
#[derive(Debug, Deserialize)]
struct MouseEventInput {
    x: u16,
    y: u16,
    button: u8,
    action: u8,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

/// Expected output for mouse event tests
#[derive(Debug, Deserialize)]
struct MouseEventOutput {
    string: String,
}

/// Input for mouse parsing tests
#[derive(Debug, Deserialize)]
struct MouseParseInput {
    sequence: String,
}

/// Expected output for mouse parsing tests
#[derive(Debug, Deserialize)]
struct MouseParseOutput {
    x: u16,
    y: u16,
    button: u8,
    action: u8,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

/// Input for key string tests
#[derive(Debug, Deserialize)]
struct KeyStringInput {
    #[serde(rename = "type")]
    key_type: i16,
    runes: Vec<String>,
    alt: bool,
    paste: bool,
}

/// Expected output for key string tests
#[derive(Debug, Deserialize)]
struct KeyStringOutput {
    string: String,
}

/// Input for command model tests
#[derive(Debug, Deserialize)]
struct CommandListInput {
    commands: Vec<Option<i32>>,
}

/// Expected output for command model tests
#[derive(Debug, Deserialize)]
struct CommandListOutput {
    result: String,
    values: Vec<i32>,
}

/// Parse key type from Go name to Rust KeyType
fn parse_key_type(name: &str) -> Option<KeyType> {
    match name {
        "KeyNull" => Some(KeyType::Null),
        "KeyBreak" | "KeyCtrlC" => Some(KeyType::CtrlC),
        "KeyEnter" | "KeyCtrlM" => Some(KeyType::Enter),
        "KeyBackspace" => Some(KeyType::Backspace),
        "KeyTab" | "KeyCtrlI" => Some(KeyType::Tab),
        "KeyEsc" | "KeyEscape" => Some(KeyType::Esc),
        "KeyCtrlA" => Some(KeyType::CtrlA),
        "KeyCtrlB" => Some(KeyType::CtrlB),
        "KeyCtrlD" => Some(KeyType::CtrlD),
        "KeyCtrlE" => Some(KeyType::CtrlE),
        "KeyCtrlF" => Some(KeyType::CtrlF),
        "KeyCtrlG" => Some(KeyType::CtrlG),
        "KeyCtrlH" => Some(KeyType::CtrlH),
        "KeyCtrlJ" => Some(KeyType::CtrlJ),
        "KeyCtrlK" => Some(KeyType::CtrlK),
        "KeyCtrlL" => Some(KeyType::CtrlL),
        "KeyCtrlN" => Some(KeyType::CtrlN),
        "KeyCtrlO" => Some(KeyType::CtrlO),
        "KeyCtrlP" => Some(KeyType::CtrlP),
        "KeyCtrlQ" => Some(KeyType::CtrlQ),
        "KeyCtrlR" => Some(KeyType::CtrlR),
        "KeyCtrlS" => Some(KeyType::CtrlS),
        "KeyCtrlT" => Some(KeyType::CtrlT),
        "KeyCtrlU" => Some(KeyType::CtrlU),
        "KeyCtrlV" => Some(KeyType::CtrlV),
        "KeyCtrlW" => Some(KeyType::CtrlW),
        "KeyCtrlX" => Some(KeyType::CtrlX),
        "KeyCtrlY" => Some(KeyType::CtrlY),
        "KeyCtrlZ" => Some(KeyType::CtrlZ),
        "KeyRunes" => Some(KeyType::Runes),
        "KeyUp" => Some(KeyType::Up),
        "KeyDown" => Some(KeyType::Down),
        "KeyRight" => Some(KeyType::Right),
        "KeyLeft" => Some(KeyType::Left),
        "KeyShiftTab" => Some(KeyType::ShiftTab),
        "KeyHome" => Some(KeyType::Home),
        "KeyEnd" => Some(KeyType::End),
        "KeyPgUp" => Some(KeyType::PgUp),
        "KeyPgDown" => Some(KeyType::PgDown),
        "KeyDelete" => Some(KeyType::Delete),
        "KeyInsert" => Some(KeyType::Insert),
        "KeySpace" => Some(KeyType::Space),
        "KeyF1" => Some(KeyType::F1),
        "KeyF2" => Some(KeyType::F2),
        "KeyF3" => Some(KeyType::F3),
        "KeyF4" => Some(KeyType::F4),
        "KeyF5" => Some(KeyType::F5),
        "KeyF6" => Some(KeyType::F6),
        "KeyF7" => Some(KeyType::F7),
        "KeyF8" => Some(KeyType::F8),
        "KeyF9" => Some(KeyType::F9),
        "KeyF10" => Some(KeyType::F10),
        "KeyF11" => Some(KeyType::F11),
        "KeyF12" => Some(KeyType::F12),
        _ => None,
    }
}

/// Parse mouse button from Go name to Rust MouseButton
fn parse_mouse_button(name: &str) -> Option<MouseButton> {
    match name {
        "MouseButtonNone" => Some(MouseButton::None),
        "MouseButtonLeft" => Some(MouseButton::Left),
        "MouseButtonMiddle" => Some(MouseButton::Middle),
        "MouseButtonRight" => Some(MouseButton::Right),
        "MouseButtonWheelUp" => Some(MouseButton::WheelUp),
        "MouseButtonWheelDown" => Some(MouseButton::WheelDown),
        "MouseButtonWheelLeft" => Some(MouseButton::WheelLeft),
        "MouseButtonWheelRight" => Some(MouseButton::WheelRight),
        "MouseButtonBackward" => Some(MouseButton::Backward),
        "MouseButtonForward" => Some(MouseButton::Forward),
        _ => None,
    }
}

/// Parse mouse action from Go name to Rust MouseAction
fn parse_mouse_action(name: &str) -> Option<MouseAction> {
    match name {
        "MouseActionPress" => Some(MouseAction::Press),
        "MouseActionRelease" => Some(MouseAction::Release),
        "MouseActionMotion" => Some(MouseAction::Motion),
        _ => None,
    }
}

/// Convert numeric button to MouseButton
fn button_from_value(val: u8) -> MouseButton {
    match val {
        0 => MouseButton::None,
        1 => MouseButton::Left,
        2 => MouseButton::Middle,
        3 => MouseButton::Right,
        4 => MouseButton::WheelUp,
        5 => MouseButton::WheelDown,
        6 => MouseButton::WheelLeft,
        7 => MouseButton::WheelRight,
        8 => MouseButton::Backward,
        9 => MouseButton::Forward,
        _ => MouseButton::None,
    }
}

/// Convert numeric action to MouseAction
fn action_from_value(val: u8) -> MouseAction {
    match val {
        0 => MouseAction::Press,
        1 => MouseAction::Release,
        2 => MouseAction::Motion,
        _ => MouseAction::Press,
    }
}

/// Convert numeric key type value to KeyType
fn key_type_from_value(val: i16) -> Option<KeyType> {
    match val {
        0 => Some(KeyType::Null),
        1 => Some(KeyType::CtrlA),
        2 => Some(KeyType::CtrlB),
        3 => Some(KeyType::CtrlC),
        4 => Some(KeyType::CtrlD),
        5 => Some(KeyType::CtrlE),
        6 => Some(KeyType::CtrlF),
        7 => Some(KeyType::CtrlG),
        8 => Some(KeyType::CtrlH),
        9 => Some(KeyType::Tab),
        10 => Some(KeyType::CtrlJ),
        11 => Some(KeyType::CtrlK),
        12 => Some(KeyType::CtrlL),
        13 => Some(KeyType::Enter),
        14 => Some(KeyType::CtrlN),
        15 => Some(KeyType::CtrlO),
        16 => Some(KeyType::CtrlP),
        17 => Some(KeyType::CtrlQ),
        18 => Some(KeyType::CtrlR),
        19 => Some(KeyType::CtrlS),
        20 => Some(KeyType::CtrlT),
        21 => Some(KeyType::CtrlU),
        22 => Some(KeyType::CtrlV),
        23 => Some(KeyType::CtrlW),
        24 => Some(KeyType::CtrlX),
        25 => Some(KeyType::CtrlY),
        26 => Some(KeyType::CtrlZ),
        27 => Some(KeyType::Esc),
        127 => Some(KeyType::Backspace),
        -1 => Some(KeyType::Runes),
        -2 => Some(KeyType::Up),
        -3 => Some(KeyType::Down),
        -4 => Some(KeyType::Right),
        -5 => Some(KeyType::Left),
        -6 => Some(KeyType::ShiftTab),
        -7 => Some(KeyType::Home),
        -8 => Some(KeyType::End),
        -9 => Some(KeyType::PgUp),
        -10 => Some(KeyType::PgDown),
        -13 => Some(KeyType::Delete),
        -14 => Some(KeyType::Insert),
        -15 => Some(KeyType::Space),
        -16 => Some(KeyType::CtrlUp),
        -17 => Some(KeyType::CtrlDown),
        -18 => Some(KeyType::CtrlRight),
        -19 => Some(KeyType::CtrlLeft),
        -22 => Some(KeyType::ShiftUp),
        -23 => Some(KeyType::ShiftDown),
        -24 => Some(KeyType::ShiftRight),
        -25 => Some(KeyType::ShiftLeft),
        -34 => Some(KeyType::F1),
        -35 => Some(KeyType::F2),
        -36 => Some(KeyType::F3),
        -37 => Some(KeyType::F4),
        -38 => Some(KeyType::F5),
        -39 => Some(KeyType::F6),
        -40 => Some(KeyType::F7),
        -41 => Some(KeyType::F8),
        -42 => Some(KeyType::F9),
        -43 => Some(KeyType::F10),
        -44 => Some(KeyType::F11),
        -45 => Some(KeyType::F12),
        _ => None,
    }
}

/// Run a key type test
fn run_keytype_test(fixture: &TestFixture) -> Result<(), String> {
    let input: KeyTypeInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: KeyTypeOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let key_type = parse_key_type(&input.key_type)
        .ok_or_else(|| format!("Unknown key type: {}", input.key_type))?;

    // Check string representation
    let actual_name = key_type.to_string();
    if actual_name != expected.string_name {
        return Err(format!(
            "KeyType string mismatch for {}:\n  expected: {:?}\n  actual:   {:?}",
            input.key_type, expected.string_name, actual_name
        ));
    }

    // Check numeric value
    let actual_value = key_type as i16;
    if actual_value != expected.value {
        return Err(format!(
            "KeyType value mismatch for {}:\n  expected: {}\n  actual:   {}",
            input.key_type, expected.value, actual_value
        ));
    }

    Ok(())
}

/// Run a mouse button test
fn run_mouse_button_test(fixture: &TestFixture) -> Result<(), String> {
    let input: MouseButtonInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: MouseButtonOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let button = parse_mouse_button(&input.button)
        .ok_or_else(|| format!("Unknown mouse button: {}", input.button))?;

    // MouseButton doesn't have numeric repr, so compare by mapping
    let actual_value = match button {
        MouseButton::None => 0,
        MouseButton::Left => 1,
        MouseButton::Middle => 2,
        MouseButton::Right => 3,
        MouseButton::WheelUp => 4,
        MouseButton::WheelDown => 5,
        MouseButton::WheelLeft => 6,
        MouseButton::WheelRight => 7,
        MouseButton::Backward => 8,
        MouseButton::Forward => 9,
        MouseButton::Button10 => 10,
        MouseButton::Button11 => 11,
    };

    if actual_value != expected.value {
        return Err(format!(
            "MouseButton value mismatch for {}:\n  expected: {}\n  actual:   {}",
            input.button, expected.value, actual_value
        ));
    }

    Ok(())
}

/// Run a mouse action test
fn run_mouse_action_test(fixture: &TestFixture) -> Result<(), String> {
    let input: MouseActionInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: MouseActionOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let action = parse_mouse_action(&input.action)
        .ok_or_else(|| format!("Unknown mouse action: {}", input.action))?;

    // MouseAction doesn't have numeric repr, so compare by mapping
    let actual_value = match action {
        MouseAction::Press => 0,
        MouseAction::Release => 1,
        MouseAction::Motion => 2,
    };

    if actual_value != expected.value {
        return Err(format!(
            "MouseAction value mismatch for {}:\n  expected: {}\n  actual:   {}",
            input.action, expected.value, actual_value
        ));
    }

    Ok(())
}

/// Run a mouse event string test
fn run_mouse_event_test(fixture: &TestFixture) -> Result<(), String> {
    let input: MouseEventInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: MouseEventOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let mouse = MouseMsg {
        x: input.x,
        y: input.y,
        button: button_from_value(input.button),
        action: action_from_value(input.action),
        ctrl: input.ctrl,
        alt: input.alt,
        shift: input.shift,
    };

    let actual_string = mouse.to_string();

    if actual_string != expected.string {
        return Err(format!(
            "MouseEvent string mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.string, actual_string
        ));
    }

    Ok(())
}

/// Run a mouse parsing test (X10/SGR protocol parsing)
fn run_mouse_parse_test(fixture: &TestFixture) -> Result<(), String> {
    let input: MouseParseInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: MouseParseOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    // Parse the mouse escape sequence
    let result = parse_mouse_event_sequence(input.sequence.as_bytes())
        .map_err(|e| format!("Failed to parse mouse sequence: {}", e))?;

    // Compare all fields
    let expected_button = button_from_value(expected.button);
    let expected_action = action_from_value(expected.action);

    if result.x != expected.x {
        return Err(format!(
            "Mouse x coordinate mismatch:\n  expected: {}\n  actual:   {}",
            expected.x, result.x
        ));
    }

    if result.y != expected.y {
        return Err(format!(
            "Mouse y coordinate mismatch:\n  expected: {}\n  actual:   {}",
            expected.y, result.y
        ));
    }

    if result.button != expected_button {
        return Err(format!(
            "Mouse button mismatch:\n  expected: {:?} ({})\n  actual:   {:?}",
            expected_button, expected.button, result.button
        ));
    }

    if result.action != expected_action {
        return Err(format!(
            "Mouse action mismatch:\n  expected: {:?} ({})\n  actual:   {:?}",
            expected_action, expected.action, result.action
        ));
    }

    if result.ctrl != expected.ctrl {
        return Err(format!(
            "Mouse ctrl modifier mismatch:\n  expected: {}\n  actual:   {}",
            expected.ctrl, result.ctrl
        ));
    }

    if result.alt != expected.alt {
        return Err(format!(
            "Mouse alt modifier mismatch:\n  expected: {}\n  actual:   {}",
            expected.alt, result.alt
        ));
    }

    if result.shift != expected.shift {
        return Err(format!(
            "Mouse shift modifier mismatch:\n  expected: {}\n  actual:   {}",
            expected.shift, result.shift
        ));
    }

    Ok(())
}

/// Run a key string test
fn run_key_string_test(fixture: &TestFixture) -> Result<(), String> {
    let input: KeyStringInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: KeyStringOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    let key_type = key_type_from_value(input.key_type)
        .ok_or_else(|| format!("Unknown key type value: {}", input.key_type))?;

    // Convert runes from strings to chars
    let runes: Vec<char> = input
        .runes
        .iter()
        .filter_map(|s| s.chars().next())
        .collect();

    let key = KeyMsg {
        key_type,
        runes,
        alt: input.alt,
        paste: input.paste,
    };

    let actual_string = key.to_string();

    if actual_string != expected.string {
        return Err(format!(
            "KeyMsg string mismatch:\n  expected: {:?}\n  actual:   {:?}",
            expected.string, actual_string
        ));
    }

    Ok(())
}

/// Run a sequence test (ANSI escape sequence parsing)
fn run_sequence_test(fixture: &TestFixture) -> Result<(), String> {
    let input: SequenceInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {}", e))?;

    let expected: SequenceOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {}", e))?;

    // Parse the sequence using our new parse_sequence function
    let result = parse_sequence(input.sequence.as_bytes())
        .ok_or_else(|| format!("Sequence not recognized: {:?}", input.sequence))?;

    // Get expected key type
    let expected_key_type = key_type_from_value(expected.key_type)
        .ok_or_else(|| format!("Unknown expected key type: {}", expected.key_type))?;

    // Check key type matches
    if result.key_type != expected_key_type {
        return Err(format!(
            "Key type mismatch for sequence {:?}:\n  expected: {:?} ({})\n  actual:   {:?} ({})",
            input.sequence,
            expected_key_type,
            expected.key_type,
            result.key_type,
            result.key_type as i16
        ));
    }

    // Check alt modifier matches
    if result.alt != expected.alt {
        return Err(format!(
            "Alt modifier mismatch for sequence {:?}:\n  expected: {}\n  actual:   {}",
            input.sequence, expected.alt, result.alt
        ));
    }

    Ok(())
}

fn build_cmds(values: &[Option<i32>]) -> Vec<Option<Cmd>> {
    values
        .iter()
        .map(|value| value.map(|val| Cmd::new(move || Message::new(val))))
        .collect()
}

fn execute_cmds(cmds: Vec<Cmd>) -> Result<Vec<i32>, String> {
    let mut outputs = Vec::with_capacity(cmds.len());
    for cmd in cmds {
        let msg = cmd
            .execute()
            .ok_or_else(|| "Command did not return a message".to_string())?;
        let value = msg
            .downcast::<i32>()
            .ok_or_else(|| "Command message type mismatch".to_string())?;
        outputs.push(value);
    }
    Ok(outputs)
}

/// Run a batch command test
fn run_command_batch_test(fixture: &TestFixture) -> Result<(), String> {
    let input: CommandListInput = fixture
        .input_as()
        .map_err(|e| format!("Invalid input: {e}"))?;
    let expected: CommandListOutput = fixture
        .expected_as()
        .map_err(|e| format!("Invalid expected output: {e}"))?;

    let cmds = build_cmds(&input.commands);
    let result = batch(cmds);

    match expected.result.as_str() {
        "none" => {
            if result.is_some() {
                return Err("Expected no command, but got Some(cmd)".to_string());
            }
            if !expected.values.is_empty() {
                return Err("Expected empty values for none result".to_string());
            }
            Ok(())
        }
        "single" => {
            let cmd = result.ok_or_else(|| "Expected a command, got None".to_string())?;
            if expected.values.len() != 1 {
                return Err("Expected exactly one value for single result".to_string());
            }
            let msg = cmd
                .execute()
                .ok_or_else(|| "Command did not return a message".to_string())?;
            let value = msg
                .downcast::<i32>()
                .ok_or_else(|| "Command message type mismatch".to_string())?;
            if value != expected.values[0] {
                return Err(format!(
                    "Single command mismatch: expected {}, got {}",
                    expected.values[0], value
                ));
            }
            Ok(())
        }
        "batch" => {
            let cmd = result.ok_or_else(|| "Expected a command, got None".to_string())?;
            let msg = cmd
                .execute()
                .ok_or_else(|| "Batch command did not return a message".to_string())?;
            let batch_msg = msg
                .downcast::<BatchMsg>()
                .ok_or_else(|| "Expected BatchMsg from batch command".to_string())?;
            let outputs = execute_cmds(batch_msg.0)?;
            if outputs != expected.values {
                return Err(format!(
                    "Batch command outputs mismatch: expected {:?}, got {:?}",
                    expected.values, outputs
                ));
            }
            Ok(())
        }
        other => Err(format!("Unknown expected result type: {}", other)),
    }
}

/// Run a sequence command test
fn run_command_sequence_test(fixture: &TestFixture) -> Result<(), String> {
    let input: CommandListInput = fixture
        .input_as()
        .map_err(|e| format!("Invalid input: {e}"))?;
    let expected: CommandListOutput = fixture
        .expected_as()
        .map_err(|e| format!("Invalid expected output: {e}"))?;

    let cmds = build_cmds(&input.commands);
    let result = sequence(cmds);

    match expected.result.as_str() {
        "none" => {
            if result.is_some() {
                return Err("Expected no command, but got Some(cmd)".to_string());
            }
            if !expected.values.is_empty() {
                return Err("Expected empty values for none result".to_string());
            }
            Ok(())
        }
        "single" => {
            let cmd = result.ok_or_else(|| "Expected a command, got None".to_string())?;
            if expected.values.len() != 1 {
                return Err("Expected exactly one value for single result".to_string());
            }
            let msg = cmd
                .execute()
                .ok_or_else(|| "Command did not return a message".to_string())?;
            let value = msg
                .downcast::<i32>()
                .ok_or_else(|| "Command message type mismatch".to_string())?;
            if value != expected.values[0] {
                return Err(format!(
                    "Single command mismatch: expected {}, got {}",
                    expected.values[0], value
                ));
            }
            Ok(())
        }
        "sequence" => {
            let cmd = result.ok_or_else(|| "Expected a command, got None".to_string())?;
            let msg = cmd
                .execute()
                .ok_or_else(|| "Sequence command did not return a message".to_string())?;
            let sequence_msg = msg
                .downcast::<SequenceMsg>()
                .ok_or_else(|| "Expected SequenceMsg from sequence command".to_string())?;
            let outputs = execute_cmds(sequence_msg.0)?;
            if outputs != expected.values {
                return Err(format!(
                    "Sequence command outputs mismatch: expected {:?}, got {:?}",
                    expected.values, outputs
                ));
            }
            Ok(())
        }
        other => Err(format!("Unknown expected result type: {}", other)),
    }
}

/// Run a single test fixture
fn run_test(fixture: &TestFixture) -> Result<(), String> {
    // Skip if marked
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("SKIPPED: {}", reason));
    }

    // Route to appropriate test runner based on test name
    let name = &fixture.name;

    if name.starts_with("keytype_") {
        run_keytype_test(fixture)
    } else if name.starts_with("sequence_") {
        run_sequence_test(fixture)
    } else if name.starts_with("mouse_button_") {
        run_mouse_button_test(fixture)
    } else if name.starts_with("mouse_action_") {
        run_mouse_action_test(fixture)
    } else if name.starts_with("mouse_event_") {
        run_mouse_event_test(fixture)
    } else if name.starts_with("mouse_parse_") {
        run_mouse_parse_test(fixture)
    } else if name.starts_with("key_string_") {
        run_key_string_test(fixture)
    } else if name.starts_with("command_batch_") {
        run_command_batch_test(fixture)
    } else if name.starts_with("command_sequence_") {
        run_command_sequence_test(fixture)
    } else {
        Err(format!("Unknown test type: {}", name))
    }
}

/// Run all bubbletea conformance tests
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let mut results = Vec::new();

    // Load fixtures
    let fixtures = match loader.load_crate("bubbletea") {
        Ok(f) => f,
        Err(e) => {
            results.push((
                "load_fixtures",
                Err(format!("Failed to load fixtures: {}", e)),
            ));
            return results;
        }
    };

    println!(
        "Loaded {} tests from bubbletea.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    // Run each test
    for test in &fixtures.tests {
        let result = run_test(test);
        // Store the test name by leaking since we need 'static lifetime
        let name: &'static str = Box::leak(test.name.clone().into_boxed_str());
        results.push((name, result));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner that loads fixtures and runs all conformance tests
    #[test]
    fn test_bubbletea_conformance() {
        let results = run_all_tests();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut failures = Vec::new();

        for (name, result) in &results {
            match result {
                Ok(()) => {
                    passed += 1;
                    println!("  PASS: {}", name);
                }
                Err(msg) if msg.starts_with("SKIPPED:") => {
                    skipped += 1;
                    println!("  SKIP: {} - {}", name, msg);
                }
                Err(msg) => {
                    failed += 1;
                    failures.push((name, msg));
                    println!("  FAIL: {} - {}", name, msg);
                }
            }
        }

        println!("\nBubbletea Conformance Results:");
        println!("  Passed:  {}", passed);
        println!("  Failed:  {}", failed);
        println!("  Skipped: {}", skipped);
        println!("  Total:   {}", results.len());

        if !failures.is_empty() {
            println!("\nFailures:");
            for (name, msg) in &failures {
                println!("  {}: {}", name, msg);
            }
            panic!(
                "Bubbletea conformance tests failed: {} of {} tests failed",
                failed,
                results.len()
            );
        }

        assert_eq!(failed, 0, "All conformance tests should pass");
    }

    /// Basic KeyType display test
    #[test]
    fn test_keytype_display() {
        assert_eq!(KeyType::Enter.to_string(), "enter");
        assert_eq!(KeyType::CtrlC.to_string(), "ctrl+c");
        assert_eq!(KeyType::Up.to_string(), "up");
        assert_eq!(KeyType::F1.to_string(), "f1");
    }

    /// Basic KeyMsg display test
    #[test]
    fn test_keymsg_display() {
        let key = KeyMsg::from_type(KeyType::Enter);
        assert_eq!(key.to_string(), "enter");

        let key = KeyMsg::from_char('a');
        assert_eq!(key.to_string(), "a");

        let key = KeyMsg::from_char('a').with_alt();
        assert_eq!(key.to_string(), "alt+a");
    }

    /// Basic MouseMsg display test
    #[test]
    fn test_mousemsg_display() {
        let mouse = MouseMsg {
            x: 0,
            y: 0,
            button: MouseButton::Left,
            action: MouseAction::Press,
            ctrl: false,
            alt: false,
            shift: false,
        };
        assert_eq!(mouse.to_string(), "left");

        let mouse = MouseMsg {
            x: 0,
            y: 0,
            button: MouseButton::Left,
            action: MouseAction::Press,
            ctrl: true,
            alt: false,
            shift: false,
        };
        assert_eq!(mouse.to_string(), "ctrl+left");
    }
}

// =============================================================================
// Program Lifecycle Conformance Tests
// =============================================================================
//
// These tests verify that the Program lifecycle matches Go bubbletea behavior:
// - Init is called exactly once at program start
// - View is called after init and after each update
// - Update receives messages correctly
// - Quit properly terminates the program
// - Final model state is preserved

#[cfg(test)]
mod lifecycle_tests {
    use bubbletea::message::QuitMsg;
    use bubbletea::simulator::{ProgramSimulator, TrackingModel};
    use bubbletea::{Cmd, Message, Model};
    use std::sync::atomic::Ordering;

    /// Test that init is called exactly once
    #[test]
    fn test_lifecycle_init_called_once() {
        let model = TrackingModel::new();
        let init_count = model.init_count.clone();

        let mut sim = ProgramSimulator::new(model);

        // Before init
        assert_eq!(init_count.load(Ordering::SeqCst), 0);

        // Explicit init
        sim.init();
        assert_eq!(init_count.load(Ordering::SeqCst), 1);

        // Second init should not increment
        sim.init();
        assert_eq!(init_count.load(Ordering::SeqCst), 1);

        // Further steps should not call init again
        sim.send(Message::new(1));
        sim.step();
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
    }

    /// Test that view is called after initialization
    #[test]
    fn test_lifecycle_view_after_init() {
        let model = TrackingModel::new();
        let view_count = model.view_count.clone();

        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // View should be called once after init (for initial render)
        assert_eq!(view_count.load(Ordering::SeqCst), 1);
        assert_eq!(sim.views().len(), 1);
        assert_eq!(sim.last_view(), Some("Value: 0"));
    }

    /// Test that view is called after each update
    #[test]
    fn test_lifecycle_view_after_update() {
        let model = TrackingModel::new();
        let view_count = model.view_count.clone();

        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // 1 view from init
        assert_eq!(view_count.load(Ordering::SeqCst), 1);

        sim.send(Message::new(5));
        sim.step();
        // 1 from init + 1 from update = 2
        assert_eq!(view_count.load(Ordering::SeqCst), 2);

        sim.send(Message::new(3));
        sim.step();
        // 1 from init + 2 from updates = 3
        assert_eq!(view_count.load(Ordering::SeqCst), 3);

        // Verify views are captured correctly
        assert_eq!(sim.views().len(), 3);
        assert_eq!(sim.views()[0], "Value: 0");
        assert_eq!(sim.views()[1], "Value: 5");
        assert_eq!(sim.views()[2], "Value: 8");
    }

    /// Test that quit properly terminates processing
    #[test]
    fn test_lifecycle_quit_terminates() {
        let model = TrackingModel::new();
        let update_count = model.update_count.clone();

        let mut sim = ProgramSimulator::new(model);
        sim.init();

        sim.send(Message::new(1));
        sim.send(Message::new(QuitMsg));
        sim.send(Message::new(2)); // This should NOT be processed

        let processed = sim.run_until_quit(10);

        assert!(sim.is_quit());
        assert_eq!(processed, 2); // First message + quit
        assert_eq!(sim.model().value, 1); // Only first increment was applied
        assert_eq!(update_count.load(Ordering::SeqCst), 1); // Only one update before quit
    }

    /// Test that final model state is preserved correctly
    #[test]
    fn test_lifecycle_final_model_state() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        sim.send(Message::new(10));
        sim.send(Message::new(20));
        sim.send(Message::new(30));
        sim.run_until_empty();

        assert_eq!(sim.model().value, 60);

        // into_model should return the final state
        let final_model = sim.into_model();
        assert_eq!(final_model.value, 60);
    }

    /// Test that implicit initialization works
    #[test]
    fn test_lifecycle_implicit_init() {
        let model = TrackingModel::new();
        let init_count = model.init_count.clone();

        let mut sim = ProgramSimulator::new(model);

        // step() should implicitly init if not initialized
        assert!(!sim.is_initialized());
        sim.send(Message::new(42));
        sim.step();

        assert!(sim.is_initialized());
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
        assert_eq!(sim.model().value, 42);
    }

    /// Test statistics tracking
    #[test]
    fn test_lifecycle_stats() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(1));
        sim.send(Message::new(2));
        sim.send(Message::new(3));
        sim.run_until_empty();

        let stats = sim.stats();
        assert_eq!(stats.init_calls, 1);
        assert_eq!(stats.update_calls, 3);
        assert_eq!(stats.view_calls, 4); // 1 init + 3 updates
        assert!(!stats.quit_requested);
    }

    /// Test that init can return a command
    #[test]
    fn test_lifecycle_init_returns_command() {
        // Create a model that returns a command from init
        struct InitCmdModel {
            received_init_msg: bool,
        }

        struct InitDoneMsg;

        impl Model for InitCmdModel {
            fn init(&self) -> Option<Cmd> {
                // Return a command that sends a message
                Some(Cmd::new(|| Message::new(InitDoneMsg)))
            }

            fn update(&mut self, msg: Message) -> Option<Cmd> {
                if msg.is::<InitDoneMsg>() {
                    self.received_init_msg = true;
                }
                None
            }

            fn view(&self) -> String {
                format!(
                    "init_msg={}",
                    if self.received_init_msg { "yes" } else { "no" }
                )
            }
        }

        let model = InitCmdModel {
            received_init_msg: false,
        };
        let mut sim = ProgramSimulator::new(model);

        // Init should return a command
        let cmd = sim.init();
        assert!(cmd.is_some());

        // Execute the command and send resulting message
        if let Some(cmd) = cmd {
            if let Some(msg) = cmd.execute() {
                sim.send(msg);
            }
        }

        // Process the message
        sim.step();

        assert!(sim.model().received_init_msg);
        assert_eq!(sim.last_view(), Some("init_msg=yes"));
    }

    /// Test run_until_empty processes all messages
    #[test]
    fn test_lifecycle_run_until_empty() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        for i in 1..=10 {
            sim.send(Message::new(i));
        }

        let processed = sim.run_until_empty();

        assert_eq!(processed, 10);
        assert_eq!(sim.model().value, 55); // 1+2+3+4+5+6+7+8+9+10 = 55
    }

    /// Test run_until_quit respects max_steps
    #[test]
    fn test_lifecycle_run_until_quit_max_steps() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        for i in 1..=100 {
            sim.send(Message::new(i));
        }

        // Only process 5 messages
        let processed = sim.run_until_quit(5);

        assert_eq!(processed, 5);
        assert_eq!(sim.model().value, 15); // 1+2+3+4+5 = 15
        assert!(!sim.is_quit()); // Not quit, just hit max steps
        assert_eq!(sim.pending_count(), 95); // 95 messages still pending
    }

    /// Test that commands returned from update are tracked in stats
    #[test]
    fn test_lifecycle_commands_returned() {
        struct CmdReturningModel {
            count: i32,
        }

        impl Model for CmdReturningModel {
            fn init(&self) -> Option<Cmd> {
                // Init returns a command
                Some(Cmd::new(|| Message::new(0)))
            }

            fn update(&mut self, msg: Message) -> Option<Cmd> {
                if let Some(n) = msg.downcast::<i32>() {
                    self.count += n;
                    if self.count < 3 {
                        // Return a command for first few updates
                        return Some(Cmd::new(|| Message::new(0)));
                    }
                }
                None
            }

            fn view(&self) -> String {
                format!("Count: {}", self.count)
            }
        }

        let model = CmdReturningModel { count: 0 };
        let mut sim = ProgramSimulator::new(model);

        let init_cmd = sim.init();
        assert!(init_cmd.is_some());
        assert_eq!(sim.stats().commands_returned, 1);

        sim.send(Message::new(1));
        let cmd = sim.step();
        assert!(cmd.is_some());
        assert_eq!(sim.stats().commands_returned, 2);
    }
}

/// Integration with the conformance trait system
pub mod integration {
    use super::*;
    use crate::harness::{ConformanceTest, TestCategory, TestContext, TestResult};

    /// Bubbletea conformance test
    pub struct BubbleteaTest {
        name: String,
    }

    impl BubbleteaTest {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ConformanceTest for BubbleteaTest {
        fn name(&self) -> &str {
            &self.name
        }

        fn crate_name(&self) -> &str {
            "bubbletea"
        }

        fn category(&self) -> TestCategory {
            TestCategory::Unit
        }

        fn run(&self, _ctx: &mut TestContext) -> TestResult {
            let mut loader = FixtureLoader::new();

            let fixture = match loader.get_test("bubbletea", &self.name) {
                Ok(f) => f.clone(),
                Err(e) => {
                    return TestResult::Fail {
                        reason: format!("Failed to load fixture: {}", e),
                    };
                }
            };

            match run_test(&fixture) {
                Ok(()) => TestResult::Pass,
                Err(msg) if msg.starts_with("SKIPPED:") => TestResult::Skipped {
                    reason: msg.replace("SKIPPED: ", ""),
                },
                Err(msg) => TestResult::Fail { reason: msg },
            }
        }
    }

    /// Get all bubbletea conformance tests as trait objects
    pub fn all_tests() -> Vec<Box<dyn ConformanceTest>> {
        let mut loader = FixtureLoader::new();
        let fixtures = match loader.load_crate("bubbletea") {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        fixtures
            .tests
            .iter()
            .map(|t| Box::new(BubbleteaTest::new(&t.name)) as Box<dyn ConformanceTest>)
            .collect()
    }
}
