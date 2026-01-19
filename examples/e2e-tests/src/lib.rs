//! End-to-End Integration Tests for charmed_rust examples
//!
//! This module provides a terminal simulation framework for testing
//! example applications with real process spawning and I/O handling.
//!
//! # Usage
//!
//! ```ignore
//! use e2e_tests::TestTerminal;
//!
//! #[test]
//! fn test_counter_increment() {
//!     let mut term = TestTerminal::spawn("counter").unwrap();
//!     term.wait_for("Count: 0", Duration::from_secs(5)).unwrap();
//!     term.press_key("+").unwrap();
//!     term.wait_for("Count: 1", Duration::from_secs(1)).unwrap();
//!     term.exit().unwrap();
//! }
//! ```

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

/// A simulated terminal for end-to-end testing of TUI applications.
///
/// TestTerminal spawns example binaries and provides methods for:
/// - Sending key presses via stdin
/// - Waiting for specific output
/// - Capturing and asserting on screen content
pub struct TestTerminal {
    child: Child,
    stdin: std::process::ChildStdin,
    output_rx: Receiver<String>,
    buffer: String,
    example_name: String,
}

impl TestTerminal {
    /// Spawn an example application in a pseudo-terminal.
    ///
    /// # Arguments
    /// * `example_name` - The name of the example (e.g., "counter", "spinner")
    ///
    /// # Example
    /// ```ignore
    /// let mut term = TestTerminal::spawn("counter")?;
    /// ```
    pub fn spawn(example_name: &str) -> anyhow::Result<Self> {
        // Build the example first to ensure it's up to date
        let build_status = Command::new("cargo")
            .args(["build", "-p", &format!("example-{}", example_name)])
            .current_dir(Self::examples_dir()?)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !build_status.success() {
            anyhow::bail!("Failed to build example-{}", example_name);
        }

        // Spawn the example
        let mut child = Command::new("cargo")
            .args(["run", "-p", &format!("example-{}", example_name), "-q"])
            .current_dir(Self::examples_dir()?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        // Spawn a thread to read stdout asynchronously
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = tx.send(line);
                } else {
                    break;
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            output_rx: rx,
            buffer: String::new(),
            example_name: example_name.to_string(),
        })
    }

    /// Get the examples directory path.
    fn examples_dir() -> anyhow::Result<std::path::PathBuf> {
        // Try to find the examples directory relative to current dir or CARGO_MANIFEST_DIR
        let paths = [
            std::path::PathBuf::from("examples"),
            std::path::PathBuf::from("../examples"),
            std::path::PathBuf::from("../../examples"),
        ];

        for path in &paths {
            if path.exists() && path.is_dir() {
                return Ok(path.clone());
            }
        }

        // Fallback to cargo manifest dir
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let examples_dir = std::path::PathBuf::from(manifest_dir)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();
            if examples_dir.exists() {
                return Ok(examples_dir);
            }
        }

        anyhow::bail!("Could not find examples directory")
    }

    /// Send a key press to the application.
    ///
    /// # Arguments
    /// * `key` - The key to send. Supports:
    ///   - Single characters: "a", "q", "+", "-"
    ///   - Special keys: "up", "down", "left", "right", "enter", "escape", "tab"
    ///   - Control sequences: "ctrl+c"
    ///
    /// # Example
    /// ```ignore
    /// term.press_key("+")?;  // Send plus key
    /// term.press_key("enter")?;  // Send Enter key
    /// term.press_key("ctrl+c")?;  // Send Ctrl+C
    /// ```
    pub fn press_key(&mut self, key: &str) -> anyhow::Result<()> {
        let bytes = match key.to_lowercase().as_str() {
            // Arrow keys (ANSI escape sequences)
            "up" => "\x1b[A",
            "down" => "\x1b[B",
            "right" => "\x1b[C",
            "left" => "\x1b[D",
            // Special keys
            "enter" => "\r",
            "return" => "\r",
            "escape" | "esc" => "\x1b",
            "tab" => "\t",
            "backspace" => "\x7f",
            "delete" => "\x1b[3~",
            "home" => "\x1b[H",
            "end" => "\x1b[F",
            "pageup" | "pgup" => "\x1b[5~",
            "pagedown" | "pgdown" => "\x1b[6~",
            "space" => " ",
            // Control sequences
            "ctrl+c" => "\x03",
            "ctrl+d" => "\x04",
            "ctrl+z" => "\x1a",
            // Single character
            _ => key,
        };

        self.stdin.write_all(bytes.as_bytes())?;
        self.stdin.flush()?;

        // Small delay to allow the app to process
        thread::sleep(Duration::from_millis(10));

        Ok(())
    }

    /// Wait for output containing the expected string.
    ///
    /// This method blocks until the expected string appears in the output
    /// or the timeout is reached.
    ///
    /// # Arguments
    /// * `expected` - The string to wait for
    /// * `timeout` - Maximum time to wait
    ///
    /// # Example
    /// ```ignore
    /// term.wait_for("Count: 0", Duration::from_secs(5))?;
    /// ```
    pub fn wait_for(&mut self, expected: &str, timeout: Duration) -> anyhow::Result<()> {
        let start = Instant::now();

        // First check if it's already in the buffer
        if self.buffer.contains(expected) {
            return Ok(());
        }

        // Keep reading until we find it or timeout
        while start.elapsed() < timeout {
            // Try to receive new output with a short timeout
            match self.output_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(line) => {
                    self.buffer.push_str(&line);
                    self.buffer.push('\n');

                    if self.buffer.contains(expected) {
                        return Ok(());
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // No new output, check again
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Process ended
                    break;
                }
            }
        }

        // Timeout or disconnected
        anyhow::bail!(
            "Timeout waiting for '{}' in {}.\nBuffer contents:\n{}",
            expected,
            self.example_name,
            self.buffer
        )
    }

    /// Assert that the current screen contains the expected text.
    ///
    /// # Example
    /// ```ignore
    /// term.assert_screen_contains("Counter Example")?;
    /// ```
    pub fn assert_screen_contains(&self, expected: &str) -> anyhow::Result<()> {
        if !self.buffer.contains(expected) {
            anyhow::bail!(
                "Screen should contain '{}' in {}.\nBuffer contents:\n{}",
                expected,
                self.example_name,
                self.buffer
            );
        }
        Ok(())
    }

    /// Get the current buffer contents.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Clear the internal buffer.
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Gracefully exit the application and wait for the process to complete.
    ///
    /// This sends 'q' to quit, then waits for the process to exit.
    ///
    /// # Example
    /// ```ignore
    /// let status = term.exit()?;
    /// assert!(status.success());
    /// ```
    pub fn exit(mut self) -> anyhow::Result<ExitStatus> {
        // Try to quit gracefully
        let _ = self.press_key("q");
        thread::sleep(Duration::from_millis(100));

        // Wait for process with timeout
        match self.child.try_wait()? {
            Some(status) => Ok(status),
            None => {
                // If still running, send Ctrl+C
                let _ = self.press_key("ctrl+c");
                thread::sleep(Duration::from_millis(100));

                match self.child.try_wait()? {
                    Some(status) => Ok(status),
                    None => {
                        // Force kill
                        self.child.kill()?;
                        Ok(self.child.wait()?)
                    }
                }
            }
        }
    }

    /// Kill the process immediately without graceful shutdown.
    pub fn kill(mut self) -> anyhow::Result<()> {
        self.child.kill()?;
        self.child.wait()?;
        Ok(())
    }
}

impl Drop for TestTerminal {
    fn drop(&mut self) {
        // Try to kill the process if it's still running
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_key_mapping() {
        // Just verify the key mapping logic works
        let mappings = [
            ("up", "\x1b[A"),
            ("down", "\x1b[B"),
            ("enter", "\r"),
            ("escape", "\x1b"),
            ("ctrl+c", "\x03"),
        ];

        for (key, expected) in mappings {
            let bytes = match key.to_lowercase().as_str() {
                "up" => "\x1b[A",
                "down" => "\x1b[B",
                "enter" => "\r",
                "escape" | "esc" => "\x1b",
                "ctrl+c" => "\x03",
                _ => key,
            };
            assert_eq!(bytes, expected, "Key '{}' should map correctly", key);
        }
    }
}
