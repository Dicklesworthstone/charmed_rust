//! End-to-end integration tests for the glow CLI.
//!
//! These tests verify the complete CLI workflow from command invocation to output.
//! They test real-world usage scenarios including file rendering, error handling,
//! and various command-line options.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// Get a Command for the glow binary.
#[allow(deprecated)]
fn glow_cmd() -> Command {
    Command::cargo_bin("glow").unwrap()
}

// =============================================================================
// Basic Usage Tests
// =============================================================================

mod basic_usage {
    use super::*;

    #[test]
    fn test_render_basic_markdown_file() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Hello World"));
    }

    #[test]
    fn test_render_complex_markdown_file() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/complex.md")
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Complex Markdown Document"))
            .stdout(predicate::str::contains("Code Blocks"));
    }

    #[test]
    fn test_render_empty_file() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/empty.md")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_render_with_dark_style() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("dark")
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Hello World"));
    }

    #[test]
    fn test_render_with_light_style() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("light")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_render_with_ascii_style() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("ascii")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_render_with_custom_width() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--width")
            .arg("60")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_short_style_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("-s")
            .arg("pink")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_short_width_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("-w")
            .arg("80")
            .arg("--no-pager")
            .assert()
            .success();
    }
}

// =============================================================================
// Stdin Input Tests
// =============================================================================

mod stdin_input {
    use super::*;

    #[test]
    fn test_stdin_with_dash() {
        let mut cmd = glow_cmd();
        cmd.arg("-")
            .arg("--no-pager")
            .write_stdin("# From Stdin\n\nHello!")
            .assert()
            .success()
            .stdout(predicate::str::contains("From Stdin"));
    }

    #[test]
    fn test_stdin_empty() {
        let mut cmd = glow_cmd();
        cmd.arg("-")
            .arg("--no-pager")
            .write_stdin("")
            .assert()
            .success();
    }

    #[test]
    fn test_stdin_with_code_block() {
        let markdown = r#"# Code Example

```python
print("hello")
```
"#;
        let mut cmd = glow_cmd();
        cmd.arg("-")
            .arg("--no-pager")
            .write_stdin(markdown)
            .assert()
            .success()
            .stdout(predicate::str::contains("Code Example"));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_file_not_found() {
        let mut cmd = glow_cmd();
        cmd.arg("nonexistent-file.md")
            .arg("--no-pager")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error"));
    }

    #[test]
    fn test_directory_instead_of_file() {
        let dir = TempDir::new().unwrap();
        let mut cmd = glow_cmd();
        cmd.arg(dir.path()).arg("--no-pager").assert().failure();
    }

    #[test]
    fn test_invalid_style() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("invalid-style-name")
            .arg("--no-pager")
            .assert()
            .failure()
            .stderr(predicate::str::contains("unknown style"));
    }

    #[test]
    fn test_invalid_width_not_number() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--width")
            .arg("abc")
            .arg("--no-pager")
            .assert()
            .failure();
    }
}

// =============================================================================
// Temp File Tests
// =============================================================================

mod temp_file_tests {
    use super::*;

    #[test]
    fn test_render_temp_file() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "# Temp File\n\nThis is temporary.").unwrap();

        let mut cmd = glow_cmd();
        cmd.arg(temp.path())
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Temp File"));
    }

    #[test]
    fn test_render_unicode_content() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(
            temp,
            "# Unicode Test\n\nEmoji: \u{1F600} Symbols: \u{2764} \u{2605}"
        )
        .unwrap();

        let mut cmd = glow_cmd();
        cmd.arg(temp.path())
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Unicode Test"));
    }

    #[test]
    fn test_render_long_lines() {
        let mut temp = NamedTempFile::new().unwrap();
        let long_line = "word ".repeat(100);
        writeln!(temp, "# Long Line Test\n\n{}", long_line).unwrap();

        let mut cmd = glow_cmd();
        cmd.arg(temp.path())
            .arg("--width")
            .arg("80")
            .arg("--no-pager")
            .assert()
            .success()
            .stdout(predicate::str::contains("Long Line Test"));
    }
}

// =============================================================================
// Help and Version Tests
// =============================================================================

mod help_version {
    use super::*;

    #[test]
    fn test_help_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Terminal-based markdown reader"))
            .stdout(predicate::str::contains("--style"))
            .stdout(predicate::str::contains("--width"));
    }

    #[test]
    fn test_help_short_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("-h")
            .assert()
            .success()
            .stdout(predicate::str::contains("glow"));
    }

    #[test]
    fn test_version_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("glow"));
    }

    #[test]
    fn test_version_short_flag() {
        let mut cmd = glow_cmd();
        cmd.arg("-V").assert().success();
    }
}

// =============================================================================
// No Arguments Behavior
// =============================================================================

mod no_arguments {
    use super::*;

    #[test]
    fn test_no_args_shows_help() {
        let mut cmd = glow_cmd();
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Usage"));
    }
}

// =============================================================================
// Style Variations
// =============================================================================

mod style_variations {
    use super::*;

    #[test]
    fn test_style_no_tty_hyphen() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("no-tty")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_style_auto() {
        let mut cmd = glow_cmd();
        cmd.arg("tests/fixtures/basic.md")
            .arg("--style")
            .arg("auto")
            .arg("--no-pager")
            .assert()
            .success();
    }

    #[test]
    fn test_all_supported_styles() {
        let styles = ["dark", "light", "ascii", "pink", "auto", "no-tty"];

        for style in styles {
            let mut cmd = glow_cmd();
            cmd.arg("tests/fixtures/basic.md")
                .arg("--style")
                .arg(style)
                .arg("--no-pager")
                .assert()
                .success();
        }
    }
}
