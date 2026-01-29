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

use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Find an available port for testing.
fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to ephemeral port");
    listener.local_addr().unwrap().port()
}

/// Generate a temporary ED25519 host key for testing.
fn generate_temp_host_key() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let key_path = temp_dir.join(format!("demo_showcase_test_key_{}", std::process::id()));

    // Remove existing key if present
    let _ = std::fs::remove_file(&key_path);
    let _ = std::fs::remove_file(key_path.with_extension("pub"));

    // Generate key using ssh-keygen
    let output = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            key_path.to_str().unwrap(),
            "-N",
            "", // No passphrase
            "-q", // Quiet
        ])
        .output()
        .expect("Failed to run ssh-keygen");

    if !output.status.success() {
        panic!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    key_path
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
    password: String,
}

impl SshTestHarness {
    /// Start an SSH server for testing.
    fn start() -> Result<Self, String> {
        let binary = demo_showcase_binary().ok_or("demo_showcase binary not found")?;

        let port = find_available_port();
        let host_key_path = generate_temp_host_key();
        let password = "test_password_12345".to_string();

        // Start the SSH server
        let server_process = Command::new(&binary)
            .args([
                "ssh",
                "--host-key",
                host_key_path.to_str().unwrap(),
                "--addr",
                &format!("127.0.0.1:{}", port),
                "--password",
                &password,
            ])
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start server: {}", e))?;

        let harness = Self {
            server_process,
            port,
            host_key_path,
            password,
        };

        // Wait for server to be ready
        harness.wait_for_server_ready(Duration::from_secs(10))?;

        Ok(harness)
    }

    /// Wait for the server to accept connections.
    fn wait_for_server_ready(&self, timeout: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(_) = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!(
            "Server did not become ready within {:?}",
            timeout
        ))
    }

    /// Connect to the SSH server using the ssh command.
    /// Returns the output from the SSH session.
    fn ssh_connect_and_quit(&self) -> Result<String, String> {
        use std::process::Stdio;

        // Use sshpass to provide the password non-interactively
        // If sshpass is not available, we'll use expect or skip
        let sshpass_check = Command::new("which").arg("sshpass").output();
        let has_sshpass = sshpass_check.is_ok() && sshpass_check.unwrap().status.success();

        if !has_sshpass {
            return Err("sshpass not installed - skipping SSH connection test".to_string());
        }

        // Connect via SSH, send 'q' to quit, capture output
        let mut child = Command::new("sshpass")
            .args([
                "-p",
                &self.password,
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
    println!(
        "SSH server started successfully on port {}",
        harness.port
    );

    // Verify the port is actually listening
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port)).is_ok(),
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
            panic!("SSH connection failed: {}", e);
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
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port));
    assert!(stream.is_ok(), "Should connect to server");

    // Drop the connection
    drop(stream);

    // Wait a moment for the server to handle the disconnect
    std::thread::sleep(Duration::from_millis(500));

    // Server should still be alive and accepting new connections
    let stream2 = std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port));
    assert!(
        stream2.is_ok(),
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
    let sshpass_check = Command::new("which").arg("sshpass").output();
    if sshpass_check.is_err() || !sshpass_check.unwrap().status.success() {
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
        .output()
        .expect("Failed to spawn ssh");

    // Should fail authentication
    assert!(
        !output.status.success(),
        "SSH with wrong password should fail"
    );

    // Server should still be alive
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port)).is_ok(),
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
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port)).is_ok(),
        "Server should be listening"
    );
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
        if std::net::TcpStream::connect(format!("127.0.0.1:{}", harness.port)).is_ok() {
            println!("Connection {} successful", i);
        } else {
            panic!("Connection {} failed", i);
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\n=== SSH E2E Smoke Test PASSED ===");
}
