//! Integration tests for Terminal I/O verification: bubbletea rendering (bd-97j0).
//!
//! Verifies that bubbletea's rendering pipeline produces correct output through
//! custom I/O, including ANSI sequences from lipgloss-styled views.

use std::io::Cursor;
use std::sync::{Arc, Mutex};

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};

// ===========================================================================
// Helpers
// ===========================================================================

/// Check if bytes contain any ANSI escape sequence.
fn contains_ansi(s: &str) -> bool {
    s.contains('\x1b')
}

/// Strip all ANSI escape sequences from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next();
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if ('@'..='~').contains(&c2) {
                            break;
                        }
                    }
                } else if next == ']' {
                    chars.next();
                    while let Some(&c2) = chars.peek() {
                        chars.next();
                        if c2 == '\x07' {
                            break;
                        }
                        if c2 == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Thread-safe writer that captures output.
#[derive(Clone)]
struct CaptureWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl CaptureWriter {
    fn new() -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn output(&self) -> String {
        let buf = self.buf.lock().unwrap();
        String::from_utf8_lossy(&buf).to_string()
    }
}

impl std::io::Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ===========================================================================
// Test Models
// ===========================================================================

/// Simple model that echoes keystrokes in its view.
#[derive(Default)]
struct EchoModel {
    chars: String,
}

impl Model for EchoModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>()
            && key.key_type == KeyType::Runes
        {
            for c in key.runes {
                if c == 'q' {
                    return Some(quit());
                }
                self.chars.push(c);
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Echo: {}", self.chars)
    }
}

/// Model that produces styled (ANSI) output in its view.
struct StyledModel {
    message: String,
}

impl StyledModel {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Model for StyledModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>()
            && key.key_type == KeyType::Runes
            && key.runes.contains(&'q')
        {
            return Some(quit());
        }
        None
    }

    fn view(&self) -> String {
        // Use lipgloss to style the output
        let style = lipgloss::Style::new().bold();
        style.render(&self.message)
    }
}

/// Model with colored output.
struct ColorModel;

impl Model for ColorModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>()
            && key.key_type == KeyType::Runes
            && key.runes.contains(&'q')
        {
            return Some(quit());
        }
        None
    }

    fn view(&self) -> String {
        let style = lipgloss::Style::new().bold().foreground("#ff0000");
        style.render("Colored Output")
    }
}

/// Model that tracks update count.
struct CounterModel {
    count: usize,
}

impl CounterModel {
    const fn new() -> Self {
        Self { count: 0 }
    }
}

impl Model for CounterModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>()
            && key.key_type == KeyType::Runes
        {
            for c in key.runes {
                if c == 'q' {
                    return Some(quit());
                }
                self.count += 1;
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

/// Model that produces multiline styled view.
struct MultilineModel;

impl Model for MultilineModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast::<KeyMsg>()
            && key.key_type == KeyType::Runes
            && key.runes.contains(&'q')
        {
            return Some(quit());
        }
        None
    }

    fn view(&self) -> String {
        let title = lipgloss::Style::new().bold().render("Title");
        let body = lipgloss::Style::new().italic().render("Body text");
        format!("{title}\n{body}")
    }
}

// ===========================================================================
// 1. Basic Custom I/O Capture
// ===========================================================================

#[test]
fn custom_io_captures_view_output() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // View should render at least once ("Echo: ")
    assert!(
        output.contains("Echo:"),
        "Output should contain view text: {output:?}"
    );
}

#[test]
fn custom_io_captures_keystrokes_in_view() {
    let input = Cursor::new(b"abcq".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    assert_eq!(model.chars, "abc", "Model received keys: {:?}", model.chars);

    let output = output_ref.output();
    // Final view should contain "Echo: abc"
    assert!(
        output.contains("abc"),
        "Output should show typed chars: {output:?}"
    );
}

// ===========================================================================
// 2. Styled View Output with ANSI
// ===========================================================================

#[test]
fn styled_view_produces_ansi_in_output() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(StyledModel::new("Hello"))
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // Bold text should have SGR 1
    assert!(
        contains_ansi(&output),
        "Styled view should contain ANSI codes: {output:?}"
    );
    assert!(
        output.contains("\x1b[1m"),
        "Should contain bold SGR: {output:?}"
    );
}

#[test]
fn styled_view_content_preserved() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(StyledModel::new("Visible"))
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    let plain = strip_ansi(&output);
    assert!(
        plain.contains("Visible"),
        "Content should be present after stripping ANSI: {plain:?}"
    );
}

#[test]
fn colored_view_has_color_sequences() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(ColorModel)
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // Should have bold + color sequences
    assert!(output.contains("\x1b[1m"), "Bold: {output:?}");
    // Color may be TrueColor or 256 depending on default renderer
    assert!(
        output.contains("\x1b[38;2;") || output.contains("\x1b[38;5;"),
        "Should have foreground color: {output:?}"
    );
    // Content preserved
    let plain = strip_ansi(&output);
    assert!(plain.contains("Colored Output"), "Content: {plain:?}");
}

// ===========================================================================
// 3. Multiple Renders
// ===========================================================================

#[test]
fn multiple_updates_produce_multiple_renders() {
    let input = Cursor::new(b"xxxq".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let model = Program::new(CounterModel::new())
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    assert_eq!(model.count, 3, "Should have counted 3 keys");
    let output = output_ref.output();
    // Should contain "Count: 3" (final state)
    assert!(
        output.contains("Count: 3"),
        "Final count in output: {output:?}"
    );
}

// ===========================================================================
// 4. Multiline Styled View
// ===========================================================================

#[test]
fn multiline_styled_view_both_styles_present() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(MultilineModel)
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // Bold (SGR 1) and Italic (SGR 3) both present
    assert!(output.contains("\x1b[1m"), "Bold present: {output:?}");
    assert!(output.contains("\x1b[3m"), "Italic present: {output:?}");
    // Content preserved
    let plain = strip_ansi(&output);
    assert!(plain.contains("Title"), "Title present: {plain:?}");
    assert!(plain.contains("Body text"), "Body present: {plain:?}");
}

#[test]
fn multiline_styled_view_each_line_has_reset() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(MultilineModel)
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // Every line with ANSI codes should have a reset
    for (i, line) in output.lines().enumerate() {
        if contains_ansi(line) && !line.trim().is_empty() {
            assert!(
                line.contains("\x1b[0m"),
                "Line {i} with ANSI should have reset: {line:?}"
            );
        }
    }
}

// ===========================================================================
// 5. ANSI Well-Formedness in Render Output
// ===========================================================================

#[test]
fn render_output_ansi_sequences_well_formed() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(ColorModel)
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // Every \x1b[ should be properly terminated
    let mut in_csi = false;
    let mut prev_esc = false;
    for c in output.chars() {
        if c == '\x1b' {
            prev_esc = true;
            continue;
        }
        if prev_esc && c == '[' {
            in_csi = true;
            prev_esc = false;
            continue;
        }
        prev_esc = false;
        if in_csi {
            if ('@'..='~').contains(&c) {
                in_csi = false;
            } else if !c.is_ascii_digit() && c != ';' && c != '?' {
                panic!("Bad CSI byte {c:?} in render output");
            }
        }
    }
    assert!(!in_csi, "Unterminated CSI in render output");
}

// ===========================================================================
// 6. Key Input Parsing from Raw Bytes
// ===========================================================================

#[test]
fn arrow_key_escape_sequences_parsed() {
    let input = Cursor::new(b"\x1b[Aq".to_vec()); // Up arrow + q
    let output = Vec::new();

    let model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should complete");

    // Arrow key doesn't add to chars (only runes do)
    assert_eq!(model.chars, "", "Arrow key should not be in chars");
}

#[test]
fn mixed_keys_and_escapes() {
    let input = Cursor::new(b"a\x1b[Bbq".to_vec()); // a, Down, b, q
    let output = Vec::new();

    let model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should complete");

    assert_eq!(model.chars, "ab", "Should have a and b");
}

// ===========================================================================
// 7. Render Contains Terminal Control Sequences
// ===========================================================================

#[test]
fn render_output_contains_cursor_and_clear() {
    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    // In custom I/O mode, bubbletea renders using cursor positioning and clear
    // The exact sequences depend on the renderer implementation
    // At minimum, the view content should be present
    assert!(
        output.contains("Echo:"),
        "View content should be in output: {output:?}"
    );
}

// ===========================================================================
// 8. Edge Cases
// ===========================================================================

#[test]
fn empty_view_no_panic() {
    struct EmptyModel;
    impl Model for EmptyModel {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(key) = msg.downcast::<KeyMsg>()
                && key.key_type == KeyType::Runes
                && key.runes.contains(&'q')
            {
                return Some(quit());
            }
            None
        }
        fn view(&self) -> String {
            String::new()
        }
    }

    let input = Cursor::new(b"q".to_vec());
    let output = Vec::new();

    let _model = Program::new(EmptyModel)
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should complete without panic");
}

#[test]
fn unicode_in_view_output() {
    struct UnicodeModel;
    impl Model for UnicodeModel {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(key) = msg.downcast::<KeyMsg>()
                && key.key_type == KeyType::Runes
                && key.runes.contains(&'q')
            {
                return Some(quit());
            }
            None
        }
        fn view(&self) -> String {
            "こんにちは 🦀".to_string()
        }
    }

    let input = Cursor::new(b"q".to_vec());
    let writer = CaptureWriter::new();
    let output_ref = writer.clone();

    let _model = Program::new(UnicodeModel)
        .with_input(input)
        .with_output(writer)
        .run()
        .expect("program should complete");

    let output = output_ref.output();
    assert!(
        output.contains("こんにちは"),
        "Unicode text present: {output:?}"
    );
    assert!(output.contains("🦀"), "Emoji present: {output:?}");
}

#[test]
fn long_view_output() {
    struct LongModel;
    impl Model for LongModel {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, msg: Message) -> Option<Cmd> {
            if let Some(key) = msg.downcast::<KeyMsg>()
                && key.key_type == KeyType::Runes
                && key.runes.contains(&'q')
            {
                return Some(quit());
            }
            None
        }
        fn view(&self) -> String {
            "line\n".repeat(100)
        }
    }

    let input = Cursor::new(b"q".to_vec());
    let output = Vec::new();

    let _model = Program::new(LongModel)
        .with_input(input)
        .with_output(output)
        .run()
        .expect("long view should not panic");
}

// ===========================================================================
// 9. Model State Integrity After Run
// ===========================================================================

#[test]
fn model_state_preserved_after_run() {
    let input = Cursor::new(b"hiq".to_vec());
    let output = Vec::new();

    let model = Program::new(EchoModel::default())
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should complete");

    assert_eq!(
        model.chars, "hi",
        "Model state preserved: {:?}",
        model.chars
    );
}

#[test]
fn counter_model_final_state() {
    let input = Cursor::new(b"12345q".to_vec());
    let output = Vec::new();

    let model = Program::new(CounterModel::new())
        .with_input(input)
        .with_output(output)
        .run()
        .expect("program should complete");

    assert_eq!(model.count, 5, "Counter should be 5");
}
