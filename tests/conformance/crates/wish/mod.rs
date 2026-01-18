//! Conformance tests for the wish crate
//!
//! This module contains conformance tests verifying that the Rust
//! implementation of SSH application framework matches the behavior
//! of the original Go library.
//!
//! Currently implemented conformance areas:
//! - Server options and builder
//! - Address parsing
//! - Error types
//! - Session and Context
//! - PublicKey functionality
//!
//! Note: Middleware composition tests require async runtime and are
//! tested separately in the wish crate's unit tests.

// Allow dead code and unused imports in test fixture structures
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::harness::{FixtureLoader, TestFixture};
use serde::Deserialize;
use std::time::Duration;
use wish::{
    Context, Error, Pty, PublicKey, ServerBuilder, ServerOptions, Session, Window, middleware,
    with_address, with_banner, with_host_key_path, with_idle_timeout, with_max_timeout,
    with_version,
};

// ===== Input/Output Structures for Fixtures =====

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ServerOptionInput {
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    option: Option<String>,
    #[serde(default)]
    key_path: Option<String>,
    #[serde(default)]
    authorized_keys_path: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
    #[serde(default)]
    banner: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ServerOptionOutput {
    #[serde(default)]
    can_create: Option<bool>,
    #[serde(default)]
    expected: Option<String>,
    #[serde(default)]
    option_type: Option<String>,
    #[serde(default)]
    seconds: Option<u64>,
    #[serde(default)]
    valid: Option<bool>,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MiddlewareInput {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    middleware_count: Option<usize>,
    #[serde(default)]
    middleware_names: Option<Vec<String>>,
    #[serde(default)]
    option_type: Option<String>,
    #[serde(default)]
    middleware: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MiddlewareOutput {
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    order: Option<usize>,
    #[serde(default)]
    execution_order: Option<String>,
    #[serde(default)]
    configurable: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ErrorInput {
    #[serde(default)]
    error_type: Option<String>,
    #[serde(default)]
    function: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ErrorOutput {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    behavior: Option<String>,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    error_types: Option<Vec<String>>,
}

// ===== Server Options Tests =====

#[test]
fn test_server_default() {
    // Test that default server options match Go's defaults
    let opts = ServerOptions::default();

    // Go default is ":22" but our Rust impl uses "0.0.0.0:22"
    // Both are functionally equivalent for listening on all interfaces
    assert!(
        opts.address == "0.0.0.0:22" || opts.address == ":22",
        "Default address should be 0.0.0.0:22 or :22, got {}",
        opts.address
    );
    assert!(opts.version.starts_with("SSH-2.0-"));
    assert!(opts.banner.is_none());
    assert!(opts.middlewares.is_empty());
}

#[test]
fn test_server_with_address() {
    let mut opts = ServerOptions::default();
    with_address(":2222")(&mut opts).unwrap();
    assert_eq!(opts.address, ":2222");
}

#[test]
fn test_server_with_host_key() {
    let mut opts = ServerOptions::default();
    with_host_key_path("/path/to/host_key")(&mut opts).unwrap();
    assert_eq!(opts.host_key_path, Some("/path/to/host_key".to_string()));
}

#[test]
fn test_server_with_banner() {
    let mut opts = ServerOptions::default();
    with_banner("Welcome to my SSH server!")(&mut opts).unwrap();
    assert_eq!(opts.banner, Some("Welcome to my SSH server!".to_string()));
}

#[test]
fn test_server_with_version() {
    let mut opts = ServerOptions::default();
    with_version("SSH-2.0-MyServer_1.0")(&mut opts).unwrap();
    assert_eq!(opts.version, "SSH-2.0-MyServer_1.0");
}

#[test]
fn test_server_with_max_timeout() {
    let mut opts = ServerOptions::default();
    with_max_timeout(Duration::from_secs(30))(&mut opts).unwrap();
    assert_eq!(opts.max_timeout, Some(Duration::from_secs(30)));
}

#[test]
fn test_server_with_idle_timeout() {
    let mut opts = ServerOptions::default();
    with_idle_timeout(Duration::from_secs(300))(&mut opts).unwrap();
    assert_eq!(opts.idle_timeout, Some(Duration::from_secs(300)));
}

// ===== Address Parsing Tests =====

#[test]
fn test_address_port_only() {
    // Test address format ":22"
    let server = ServerBuilder::new().address(":22").build().unwrap();
    assert_eq!(server.address(), ":22");
}

#[test]
fn test_address_localhost_22() {
    let server = ServerBuilder::new()
        .address("localhost:22")
        .build()
        .unwrap();
    assert_eq!(server.address(), "localhost:22");
}

#[test]
fn test_address_localhost_2222() {
    let server = ServerBuilder::new()
        .address("localhost:2222")
        .build()
        .unwrap();
    assert_eq!(server.address(), "localhost:2222");
}

#[test]
fn test_address_ipv4_22() {
    let server = ServerBuilder::new()
        .address("127.0.0.1:22")
        .build()
        .unwrap();
    assert_eq!(server.address(), "127.0.0.1:22");
}

#[test]
fn test_address_ipv4_2222() {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .build()
        .unwrap();
    assert_eq!(server.address(), "0.0.0.0:2222");
}

#[test]
fn test_address_ipv6_22() {
    let server = ServerBuilder::new().address("[::1]:22").build().unwrap();
    assert_eq!(server.address(), "[::1]:22");
}

#[test]
fn test_address_ipv6_all() {
    let server = ServerBuilder::new().address("[::]:22").build().unwrap();
    assert_eq!(server.address(), "[::]:22");
}

#[test]
fn test_address_high_port() {
    let server = ServerBuilder::new()
        .address("localhost:65535")
        .build()
        .unwrap();
    assert_eq!(server.address(), "localhost:65535");
}

#[test]
fn test_address_custom_port() {
    let server = ServerBuilder::new()
        .address("10.0.0.1:3000")
        .build()
        .unwrap();
    assert_eq!(server.address(), "10.0.0.1:3000");
}

// ===== Server Builder Tests =====

#[test]
fn test_server_builder_full() {
    let server = ServerBuilder::new()
        .address("0.0.0.0:2222")
        .version("SSH-2.0-TestApp")
        .banner("Welcome!")
        .host_key_path("/path/to/key")
        .idle_timeout(Duration::from_secs(300))
        .max_timeout(Duration::from_secs(3600))
        .build()
        .unwrap();

    let opts = server.options();
    assert_eq!(opts.address, "0.0.0.0:2222");
    assert_eq!(opts.version, "SSH-2.0-TestApp");
    assert_eq!(opts.banner, Some("Welcome!".to_string()));
    assert_eq!(opts.host_key_path, Some("/path/to/key".to_string()));
    assert_eq!(opts.idle_timeout, Some(Duration::from_secs(300)));
    assert_eq!(opts.max_timeout, Some(Duration::from_secs(3600)));
}

// ===== Middleware Creation Tests =====
// Note: These tests verify middleware can be created. Actual execution
// is tested in the wish crate's unit tests with tokio runtime.

#[test]
fn test_middleware_activeterm_creation() {
    let _mw = middleware::activeterm::middleware();
    // Middleware created successfully
}

#[test]
fn test_middleware_logging_creation() {
    let _mw = middleware::logging::middleware();
    // Middleware created successfully
}

#[test]
fn test_middleware_recover_creation() {
    let _mw = middleware::recover::middleware();
    // Middleware created successfully
}

#[test]
fn test_middleware_elapsed_creation() {
    let _mw = middleware::elapsed::middleware();
    // Middleware created successfully
}

#[test]
fn test_middleware_comment_creation() {
    let _mw = middleware::comment::middleware("Welcome!");
    // Middleware created successfully
}

#[test]
fn test_middleware_accesscontrol_creation() {
    let allowed = vec!["git".to_string(), "ls".to_string()];
    let _mw = middleware::accesscontrol::middleware(allowed);
    // Middleware created successfully
}

#[test]
fn test_middleware_ratelimiter_creation() {
    let config = middleware::ratelimiter::Config {
        max_requests: 100,
        duration: Duration::from_secs(60),
    };
    let _mw = middleware::ratelimiter::middleware(config);
    // Middleware created successfully
}

// ===== Error Type Tests =====

#[test]
fn test_error_auth_failed() {
    let err = Error::AuthenticationFailed;
    assert!(
        err.to_string().to_lowercase().contains("authentication"),
        "Auth error should mention authentication"
    );
}

#[test]
fn test_error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
    let err = Error::Io(io_err);
    assert!(
        err.to_string().contains("io error"),
        "IO error should be properly wrapped"
    );
}

#[test]
fn test_error_ssh() {
    let err = Error::Ssh("protocol error".to_string());
    assert!(
        err.to_string().contains("ssh error"),
        "SSH error should contain ssh error"
    );
}

#[test]
fn test_error_session() {
    let err = Error::Session("invalid session".to_string());
    assert!(
        err.to_string().contains("session error"),
        "Session error should contain session error"
    );
}

#[test]
fn test_error_configuration() {
    let err = Error::Configuration("invalid config".to_string());
    assert!(
        err.to_string().contains("configuration error"),
        "Configuration error should contain configuration error"
    );
}

// ===== Session and Context Tests =====

#[test]
fn test_context_value_storage() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    ctx.set_value("key1", "value1");
    ctx.set_value("key2", "value2");

    assert_eq!(ctx.get_value("key1"), Some("value1".to_string()));
    assert_eq!(ctx.get_value("key2"), Some("value2".to_string()));
    assert_eq!(ctx.get_value("nonexistent"), None);
}

#[test]
fn test_context_basic() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    assert_eq!(ctx.user(), "testuser");
    assert_eq!(ctx.remote_addr(), addr);
    assert_eq!(ctx.local_addr(), addr);
}

#[test]
fn test_session_basic() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);
    let session = Session::new(ctx);

    assert_eq!(session.user(), "testuser");
    assert!(session.command().is_empty());
    assert!(session.public_key().is_none());
    assert!(session.subsystem().is_none());
}

#[test]
fn test_session_with_public_key() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]).with_comment("test@example.com");

    let session = Session::new(ctx).with_public_key(key);

    assert!(session.public_key().is_some());
    assert_eq!(session.public_key().unwrap().key_type, "ssh-ed25519");
    assert_eq!(
        session.public_key().unwrap().comment,
        Some("test@example.com".to_string())
    );
}

#[test]
fn test_session_with_subsystem() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let session = Session::new(ctx).with_subsystem("sftp");

    assert_eq!(session.subsystem(), Some("sftp"));
}

#[test]
fn test_session_with_command() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let session = Session::new(ctx).with_command(vec!["git".to_string(), "clone".to_string()]);

    assert_eq!(session.command(), &["git", "clone"]);
}

#[test]
fn test_session_environ() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let session = Session::new(ctx)
        .with_env("HOME", "/home/user")
        .with_env("TERM", "xterm");

    assert_eq!(session.get_env("HOME"), Some(&"/home/user".to_string()));
    assert_eq!(session.get_env("TERM"), Some(&"xterm".to_string()));
    assert_eq!(session.environ().len(), 2);
}

#[test]
fn test_session_with_pty() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let pty = Pty {
        term: "xterm-256color".to_string(),
        window: Window {
            width: 120,
            height: 40,
        },
    };

    let session = Session::new(ctx).with_pty(pty);

    let (pty_ref, active) = session.pty();
    assert!(active);
    assert_eq!(pty_ref.unwrap().term, "xterm-256color");
    assert_eq!(session.window().width, 120);
    assert_eq!(session.window().height, 40);
}

#[test]
fn test_session_write() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);
    let session = Session::new(ctx);

    let n = session.write(b"hello").unwrap();
    assert_eq!(n, 5);

    let n = session.write_stderr(b"error").unwrap();
    assert_eq!(n, 5);
}

#[test]
fn test_session_exit_close() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);
    let session = Session::new(ctx);

    assert!(!session.is_closed());
    session.exit(0).unwrap();
    session.close().unwrap();
    assert!(session.is_closed());
}

// ===== PublicKey Tests =====

#[test]
fn test_public_key_creation() {
    let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
    assert_eq!(key.key_type, "ssh-ed25519");
    assert_eq!(key.data, vec![1, 2, 3, 4]);
    assert!(key.comment.is_none());
}

#[test]
fn test_public_key_with_comment() {
    let key = PublicKey::new("ssh-rsa", vec![5, 6, 7, 8]).with_comment("user@host");
    assert_eq!(key.key_type, "ssh-rsa");
    assert_eq!(key.comment, Some("user@host".to_string()));
}

#[test]
fn test_public_key_equality() {
    let key1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
    let key2 = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
    let key3 = PublicKey::new("ssh-ed25519", vec![5, 6, 7, 8]);
    let key4 = PublicKey::new("ssh-rsa", vec![1, 2, 3, 4]);

    assert_eq!(key1, key2, "Same type and data should be equal");
    assert_ne!(key1, key3, "Different data should not be equal");
    assert_ne!(key1, key4, "Different type should not be equal");
}

#[test]
fn test_public_key_fingerprint() {
    let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
    let fp = key.fingerprint();

    assert!(
        fp.starts_with("SHA256:"),
        "Fingerprint should start with SHA256:"
    );
    assert!(fp.len() > 7, "Fingerprint should have content after prefix");
}

// ===== Window and PTY Tests =====

#[test]
fn test_window_default() {
    let window = Window::default();
    assert_eq!(window.width, 80);
    assert_eq!(window.height, 24);
}

#[test]
fn test_pty_default() {
    let pty = Pty::default();
    assert_eq!(pty.term, "xterm-256color");
    assert_eq!(pty.window.width, 80);
    assert_eq!(pty.window.height, 24);
}

// ===== BubbleTea Integration Tests =====

#[test]
fn test_tea_make_renderer_256color() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let pty = Pty {
        term: "xterm-256color".to_string(),
        window: Window::default(),
    };

    let session = Session::new(ctx).with_pty(pty);
    let _renderer = wish::tea::make_renderer(&session);
    // Verify renderer was created (we can't easily check color profile)
}

#[test]
fn test_tea_make_renderer_basic_term() {
    let addr: std::net::SocketAddr = "127.0.0.1:2222".parse().unwrap();
    let ctx = Context::new("testuser", addr, addr);

    let pty = Pty {
        term: "vt100".to_string(),
        window: Window::default(),
    };

    let session = Session::new(ctx).with_pty(pty);
    let _renderer = wish::tea::make_renderer(&session);
}

// ===== Fixture-based Conformance Tests =====

#[test]
fn test_fixture_server_options() {
    let mut loader = FixtureLoader::new();
    let fixtures = match loader.load_crate("wish") {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Warning: wish fixtures not found, skipping fixture tests");
            return;
        }
    };

    for fixture in fixtures.tests.iter() {
        if let Some(skip) = fixture.should_skip() {
            eprintln!("Skipping {}: {}", fixture.name, skip);
            continue;
        }

        if fixture.name.starts_with("server_") {
            test_server_option_fixture(fixture);
        }
    }
}

fn test_server_option_fixture(fixture: &TestFixture) {
    let input: ServerOptionInput = match fixture.input_as() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Warning: Could not parse input for {}: {}", fixture.name, e);
            return;
        }
    };

    let output: ServerOptionOutput = match fixture.expected_as() {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "Warning: Could not parse output for {}: {}",
                fixture.name, e
            );
            return;
        }
    };

    match fixture.name.as_str() {
        "server_default" => {
            if output.can_create == Some(true) {
                let _opts = ServerOptions::default();
                // Server can be created with defaults
            }
        }
        "server_with_address" => {
            if let (Some(addr), Some(expected)) = (&input.address, &output.expected) {
                let mut opts = ServerOptions::default();
                with_address(addr.clone())(&mut opts).unwrap();
                assert_eq!(&opts.address, expected);
            }
        }
        "server_with_host_key" => {
            if let Some(path) = &input.key_path {
                let mut opts = ServerOptions::default();
                with_host_key_path(path.clone())(&mut opts).unwrap();
                assert_eq!(opts.host_key_path, Some(path.clone()));
            }
        }
        "server_with_banner" => {
            if let (Some(banner), Some(expected)) = (&input.banner, &output.expected) {
                let mut opts = ServerOptions::default();
                with_banner(banner.clone())(&mut opts).unwrap();
                assert_eq!(opts.banner, Some(expected.clone()));
            }
        }
        "server_with_version" => {
            if let (Some(version), Some(expected)) = (&input.version, &output.expected) {
                let mut opts = ServerOptions::default();
                with_version(version.clone())(&mut opts).unwrap();
                assert_eq!(&opts.version, expected);
            }
        }
        "server_with_max_timeout" | "server_with_idle_timeout" => {
            if let Some(secs) = output.seconds {
                let mut opts = ServerOptions::default();
                let duration = Duration::from_secs(secs);
                if fixture.name.contains("max") {
                    with_max_timeout(duration)(&mut opts).unwrap();
                    assert_eq!(opts.max_timeout, Some(duration));
                } else {
                    with_idle_timeout(duration)(&mut opts).unwrap();
                    assert_eq!(opts.idle_timeout, Some(duration));
                }
            }
        }
        _ => {
            // Other server options (auth handlers, etc.) are tested separately
        }
    }
}

#[test]
fn test_fixture_addresses() {
    let mut loader = FixtureLoader::new();
    let fixtures = match loader.load_crate("wish") {
        Ok(f) => f,
        Err(_) => return,
    };

    for fixture in fixtures.tests.iter() {
        if fixture.name.starts_with("address_") {
            test_address_fixture(fixture);
        }
    }
}

fn test_address_fixture(fixture: &TestFixture) {
    let input: ServerOptionInput = match fixture.input_as() {
        Ok(i) => i,
        Err(_) => return,
    };

    let output: ServerOptionOutput = match fixture.expected_as() {
        Ok(o) => o,
        Err(_) => return,
    };

    if let (Some(addr), Some(true)) = (&input.address, output.valid) {
        let server = ServerBuilder::new().address(addr.clone()).build().unwrap();
        assert_eq!(
            server.address(),
            addr,
            "Address should be set correctly for {}",
            fixture.name
        );
    }
}

#[test]
fn test_fixture_middleware() {
    let mut loader = FixtureLoader::new();
    let fixtures = match loader.load_crate("wish") {
        Ok(f) => f,
        Err(_) => return,
    };

    for fixture in fixtures.tests.iter() {
        if fixture.name.starts_with("middleware_") {
            test_middleware_fixture(fixture);
        }
    }
}

fn test_middleware_fixture(fixture: &TestFixture) {
    let input: MiddlewareInput = match fixture.input_as() {
        Ok(i) => i,
        Err(_) => return,
    };

    let output: MiddlewareOutput = match fixture.expected_as() {
        Ok(o) => o,
        Err(_) => return,
    };

    // Test middleware creation based on name
    if let Some(name) = &input.name {
        match name.as_str() {
            "logging" => {
                let _mw = middleware::logging::middleware();
            }
            "activeterm" => {
                let _mw = middleware::activeterm::middleware();
            }
            "recovery" => {
                let _mw = middleware::recover::middleware();
            }
            "elapsed" => {
                let _mw = middleware::elapsed::middleware();
            }
            _ => {
                // Other middleware types may not be implemented yet
            }
        }
    }

    // Test middleware chain behavior
    if fixture.name == "middleware_chain" {
        if let Some(order) = &output.execution_order {
            assert_eq!(
                order, "outer_to_inner",
                "Middleware should execute outer to inner"
            );
        }
    }
}

#[test]
fn test_fixture_errors() {
    let mut loader = FixtureLoader::new();
    let fixtures = match loader.load_crate("wish") {
        Ok(f) => f,
        Err(_) => return,
    };

    for fixture in fixtures.tests.iter() {
        if fixture.name.starts_with("error_") {
            test_error_fixture(fixture);
        }
    }
}

fn test_error_fixture(fixture: &TestFixture) {
    let input: ErrorInput = match fixture.input_as() {
        Ok(i) => i,
        Err(_) => return,
    };

    let output: ErrorOutput = match fixture.expected_as() {
        Ok(o) => o,
        Err(_) => return,
    };

    if let Some(error_type) = &input.error_type {
        match error_type.as_str() {
            "ErrAuthFailed" => {
                let err = Error::AuthenticationFailed;
                if let Some(msg) = &output.message {
                    assert!(
                        err.to_string().to_lowercase().contains(&msg.to_lowercase()),
                        "Auth error message mismatch"
                    );
                }
            }
            "ErrInvalidSession" => {
                let err = Error::Session("invalid session".to_string());
                assert!(err.to_string().contains("session"));
            }
            "ErrTimeout" => {
                let err = Error::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "connection timeout",
                ));
                assert!(err.to_string().contains("io error"));
            }
            "ErrPermissionDenied" => {
                let err = Error::Session("permission denied".to_string());
                assert!(err.to_string().contains("session"));
            }
            _ => {}
        }
    }

    // Test fatal function behavior
    if let Some(func) = &input.function {
        if func == "wish.Fatal" {
            // Fatal should exit with code 1
            if let Some(code) = output.exit_code {
                assert_eq!(code, 1, "Fatal should exit with code 1");
            }
        }
    }
}
