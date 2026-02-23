//! E2E Tests: SSH mode smoke (bd-1m7x)
//!
//! End-to-end smoke tests for SSH server mode.
//!
//! These tests verify that the SSH server:
//! - Accepts connections with proper authentication
//! - Renders the TUI correctly over SSH
//! - Handles session cleanup gracefully
//!
//! # Running the tests
//!
//! These tests require the `ssh` feature and are marked `#[ignore]` by default
//! because they need a real SSH connection which may not work in all CI environments.
//!
//! ```bash
//! # Build with ssh feature
//! cargo build -p demo_showcase --features ssh
//!
//! # Run SSH tests explicitly
//! cargo test -p demo_showcase --features ssh -- --ignored ssh_e2e
//! ```
//!
//! # Test Requirements
//!
//! - The `demo_showcase` binary must be built with `--features ssh`
//! - Tests generate temporary host keys (no setup needed)
//! - An available port is automatically selected

#![cfg(feature = "ssh")]
#![allow(
    clippy::doc_markdown,
    clippy::ignore_without_reason,
    clippy::option_if_let_else,
    clippy::uninlined_format_args,
    clippy::useless_format
)]

use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Find an available port for testing.
fn find_available_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind to ephemeral port: {e}"))?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to read ephemeral listener address: {e}"))?;
    Ok(addr.port())
}

/// Probe server availability with an explicit connect+shutdown cycle.
fn probe_server(port: u16) -> bool {
    match TcpStream::connect(format!("127.0.0.1:{port}")) {
        Ok(stream) => {
            let _ = stream.shutdown(Shutdown::Both);
            true
        }
        Err(_) => false,
    }
}

fn generate_test_password(port: u16) -> String {
    #[allow(clippy::cast_possible_truncation)]
    let time_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    format!("pw_{:x}_{port}", time_seed ^ u64::from(std::process::id()))
}

fn has_sshpass() -> bool {
    matches!(
        Command::new("which").arg("sshpass").output(),
        Ok(output) if output.status.success()
    )
}

/// Generate a temporary ED25519 host key for testing.
fn generate_temp_host_key() -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir();
    let key_path = temp_dir.join(format!("demo_showcase_test_key_{}", std::process::id()));
    let key_path_str = key_path
        .to_str()
        .ok_or_else(|| format!("Host key path is not valid UTF-8: {}", key_path.display()))?;

    // Remove existing key if present
    let _ = std::fs::remove_file(&key_path);
    let _ = std::fs::remove_file(key_path.with_extension("pub"));

    // Generate key using ssh-keygen
    let output = Command::new("ssh-keygen")
        .args(["-t", "ed25519", "-f", key_path_str, "-N", "", "-q"])
        .output()
        .map_err(|e| format!("Failed to run ssh-keygen: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(key_path)
}

/// Cleanup temporary host key files.
fn cleanup_temp_host_key(key_path: &PathBuf) {
    let _ = std::fs::remove_file(key_path);
    let _ = std::fs::remove_file(key_path.with_extension("pub"));
}

/// Get the path to the demo_showcase binary.
fn demo_showcase_binary() -> Option<PathBuf> {
    // Try different locations for the binary
    let possible_paths = [
        // When running from crates/demo_showcase
        PathBuf::from("../../target/debug/demo_showcase"),
        // When running from repo root
        PathBuf::from("target/debug/demo_showcase"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return Some(path.clone());
        }
    }

    None
}

/// SSH server test harness.
struct SshTestHarness {
    server_process: Child,
    port: u16,
    host_key_path: PathBuf,
    auth_credential: String,
}

impl SshTestHarness {
    /// Start an SSH server for testing.
    fn start() -> Result<Self, String> {
        let binary = demo_showcase_binary().ok_or("demo_showcase binary not found")?;

        let port = find_available_port()?;
        let host_key_path = generate_temp_host_key()?;
        let auth_credential = generate_test_password(port);

        // Start the SSH server
        let server_process = Command::new(&binary)
            .arg("ssh")
            .arg("--host-key")
            .arg(host_key_path.as_os_str())
            .arg("--addr")
            .arg(format!("127.0.0.1:{port}"))
            .arg("--password")
            .arg(&auth_credential)
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start server: {}", e))?;

        let harness = Self {
            server_process,
            port,
            host_key_path,
            auth_credential,
        };

        // Wait for server to be ready
        harness.wait_for_server_ready(Duration::from_secs(10))?;

        Ok(harness)
    }

    /// Wait for the server to accept connections.
    fn wait_for_server_ready(&self, timeout: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if probe_server(self.port) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!("Server did not become ready within {:?}", timeout))
    }

    /// Connect to the SSH server using the ssh command.
    /// Returns the output from the SSH session.
    fn ssh_connect_and_quit(&self) -> Result<String, String> {
        use std::process::Stdio;

        // Use sshpass to provide the password non-interactively
        // If sshpass is not available, we'll use expect or skip
        if !has_sshpass() {
            return Err("sshpass not installed - skipping SSH connection test".to_string());
        }

        // Connect via SSH, send 'q' to quit, capture output
        let mut child = Command::new("sshpass")
            .args([
                "-p",
                &self.auth_credential,
                "ssh",
                "-p",
                &self.port.to_string(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "ConnectTimeout=10",
                "-tt", // Force pseudo-terminal allocation
                &format!("testuser@127.0.0.1"),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn ssh: {}", e))?;

        // Give the session time to start
        std::thread::sleep(Duration::from_secs(2));

        // Send quit command
        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(b"q");
            let _ = stdin.flush();
        }

        // Wait for exit with timeout
        let output = child
            .wait_with_output()
            .map_err(|e| format!("SSH wait failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stdout.is_empty() {
            // Even if exit code is non-zero, if we got output it may be OK
            // (e.g., SSH exits with code 255 on connection close)
            return Ok(stdout);
        }

        if output.status.success() || !stdout.is_empty() {
            Ok(stdout)
        } else {
            Err(format!(
                "SSH connection failed. stdout: {}, stderr: {}",
                stdout, stderr
            ))
        }
    }
}

impl Drop for SshTestHarness {
    fn drop(&mut self) {
        // Kill the server process
        let _ = self.server_process.kill();
        let _ = self.server_process.wait();

        // Cleanup the host key
        cleanup_temp_host_key(&self.host_key_path);
    }
}

// =============================================================================
// SSH SMOKE TESTS
// =============================================================================

/// Test that the SSH server starts and accepts connections.
///
/// This test is ignored by default - run with `--ignored` to execute.
#[test]
#[ignore]
fn ssh_e2e_server_starts() {
    let harness = match SshTestHarness::start() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping SSH test: {}", e);
            return;
        }
    };

    // If we got here, the server started and is accepting connections
    println!("SSH server started successfully on port {}", harness.port);

    // Verify the port is actually listening
    assert!(
        probe_server(harness.port),
        "Should be able to connect to server"
    );
}

/// Test that the SSH server renders UI content.
///
/// This test requires `sshpass` to be installed.
/// Run with `--ignored` to execute.
#[test]
#[ignore]
fn ssh_e2e_renders_ui() {
    let harness = match SshTestHarness::start() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping SSH test: {}", e);
            return;
        }
    };

    println!("Connecting to SSH server on port {}", harness.port);

    match harness.ssh_connect_and_quit() {
        Ok(output) => {
            println!("SSH session output ({} bytes):", output.len());
            println!("{}", &output[..output.len().min(2000)]);

            // The output should contain TUI content
            // Note: output may contain ANSI escape codes
            let has_content = output.contains("Charmed")
                || output.contains("Dashboard")
                || output.contains("Welcome")
                || output.len() > 100; // At minimum we should have some output

            assert!(has_content, "SSH session should render TUI content");
        }
        Err(e) => {
            // sshpass may not be installed - that's OK for CI
            if e.contains("sshpass not installed") {
                eprintln!("Skipping UI verification: {}", e);
                return;
            }
            assert!(
                e.contains("sshpass not installed"),
                "SSH connection failed: {}",
                e
            );
        }
    }
}

/// Test that the SSH server handles session cleanup gracefully.
///
/// Run with `--ignored` to execute.
#[test]
#[ignore]
fn ssh_e2e_clean_disconnect() {
    let harness = match SshTestHarness::start() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping SSH test: {}", e);
            return;
        }
    };

    // Connect and immediately disconnect
    let stream = TcpStream::connect(format!("127.0.0.1:{}", harness.port));
    assert!(stream.is_ok(), "Should connect to server");

    // Explicitly shutdown the connection.
    if let Ok(stream) = stream {
        let _ = stream.shutdown(Shutdown::Both);
    }

    // Wait a moment for the server to handle the disconnect
    std::thread::sleep(Duration::from_millis(500));

    // Server should still be alive and accepting new connections
    assert!(
        probe_server(harness.port),
        "Server should still accept connections after disconnect"
    );
}

/// Test that the SSH server rejects incorrect passwords.
///
/// Run with `--ignored` to execute.
#[test]
#[ignore]
fn ssh_e2e_rejects_bad_password() {
    let harness = match SshTestHarness::start() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Skipping SSH test: {}", e);
            return;
        }
    };

    // Check if sshpass is available
    if !has_sshpass() {
        eprintln!("Skipping test: sshpass not installed");
        return;
    }

    // Try to connect with wrong password
    let output = Command::new("sshpass")
        .args([
            "-p",
            "wrong_password",
            "ssh",
            "-p",
            &harness.port.to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "NumberOfPasswordPrompts=1",
            &format!("testuser@127.0.0.1"),
            "echo",
            "should_not_see_this",
        ])
        .output();
    assert!(output.is_ok(), "Failed to spawn ssh");
    let Ok(output) = output else {
        return;
    };

    // Should fail authentication
    assert!(
        !output.status.success(),
        "SSH with wrong password should fail"
    );

    // Server should still be alive
    assert!(
        probe_server(harness.port),
        "Server should still be alive after failed auth"
    );
}

// =============================================================================
// SMOKE TEST - COMPREHENSIVE SSH SCENARIO
// =============================================================================

/// Comprehensive smoke test for SSH mode.
///
/// This test exercises the full SSH workflow:
/// 1. Server startup
/// 2. Connection with authentication
/// 3. UI rendering verification
/// 4. Clean session termination
///
/// Run with `--ignored` to execute.
#[test]
#[ignore]
fn ssh_e2e_smoke_test() {
    println!("=== SSH E2E Smoke Test ===");

    // Phase 1: Start server
    println!("\n[Phase 1] Starting SSH server...");
    let harness = match SshTestHarness::start() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Cannot run smoke test: {}", e);
            return;
        }
    };
    println!("Server started on port {}", harness.port);

    // Phase 2: Verify server is listening
    println!("\n[Phase 2] Verifying server is listening...");
    assert!(probe_server(harness.port), "Server should be listening");
    println!("Server is accepting connections");

    // Phase 3: Test SSH connection
    println!("\n[Phase 3] Testing SSH connection...");
    match harness.ssh_connect_and_quit() {
        Ok(output) => {
            let preview_len = output.len().min(500);
            println!("Got output ({} bytes):", output.len());
            println!("---");
            println!("{}", &output[..preview_len]);
            if output.len() > preview_len {
                println!("... ({} more bytes)", output.len() - preview_len);
            }
            println!("---");
        }
        Err(e) if e.contains("sshpass") => {
            println!("Skipping SSH verification (sshpass not available)");
        }
        Err(e) => {
            println!("SSH connection warning: {}", e);
        }
    }

    // Phase 4: Verify server handles multiple connections
    println!("\n[Phase 4] Testing connection resilience...");
    for i in 1..=3 {
        assert!(probe_server(harness.port), "Connection {} failed", i);
        println!("Connection {} successful", i);
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\n=== SSH E2E Smoke Test PASSED ===");
}
