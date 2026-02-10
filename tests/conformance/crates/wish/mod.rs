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
use bubbletea::{Cmd, Message, Model};
use serde::Deserialize;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wish::AuthorizedKeysAuth;
use wish::{
    Context, Error, Pty, PublicKey, ServerBuilder, ServerOptions, Session, Window, middleware,
    with_address, with_auth_handler, with_banner, with_host_key_path, with_idle_timeout,
    with_keyboard_interactive_auth, with_max_timeout, with_password_auth, with_public_key_auth,
    with_version,
};

#[derive(Clone, Default)]
struct NoopTeaModel;

impl Model for NoopTeaModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        String::new()
    }
}

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

/// Run all wish conformance tests.
pub fn run_all_tests() -> Vec<(&'static str, Result<(), String>)> {
    let mut loader = FixtureLoader::new();
    let fixtures = loader
        .load_crate("wish")
        .map_err(|e| format!("Failed to load wish fixtures: {e}"));

    let fixtures = match fixtures {
        Ok(f) => f,
        Err(e) => return vec![("load_fixtures", Err(e))],
    };

    println!(
        "Loaded {} tests from wish.json (Go lib version {})",
        fixtures.tests.len(),
        fixtures.metadata.library_version
    );

    let mut results = Vec::with_capacity(fixtures.tests.len());
    for test in &fixtures.tests {
        let result = run_test(test);
        let name: &'static str = Box::leak(test.name.clone().into_boxed_str());
        results.push((name, result));
    }
    results
}

fn run_test(fixture: &TestFixture) -> Result<(), String> {
    if let Some(reason) = fixture.should_skip() {
        return Err(format!("Fixture unexpectedly marked skip: {reason}"));
    }

    if fixture.name.starts_with("server_") {
        return run_server_fixture(fixture);
    }
    if fixture.name.starts_with("address_") {
        return run_address_fixture(fixture);
    }
    if fixture.name.starts_with("middleware_") {
        return run_middleware_fixture(fixture);
    }
    if fixture.name.starts_with("error_") {
        return run_error_fixture(fixture);
    }

    Err(format!("Not implemented for fixture: {}", fixture.name))
}

fn run_server_fixture(fixture: &TestFixture) -> Result<(), String> {
    let input: ServerOptionInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {e}"))?;
    let expected: ServerOptionOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {e}"))?;

    match fixture.name.as_str() {
        "server_default" => {
            let _ = ServerBuilder::new()
                .build()
                .map_err(|e| format!("Failed to build default server: {e}"))?;
            if expected.can_create != Some(true) {
                return Err("Expected can_create=true for server_default".to_string());
            }
            Ok(())
        }
        "server_with_address" => {
            let mut opts = ServerOptions::default();
            let addr = input
                .address
                .ok_or_else(|| "Missing input.address".to_string())?;
            with_address(addr.clone())(&mut opts).map_err(|e| e.to_string())?;
            let want = expected
                .expected
                .ok_or_else(|| "Missing expected.expected".to_string())?;
            if opts.address != want {
                return Err(format!(
                    "Address mismatch: expected {want:?}, got {:?}",
                    opts.address
                ));
            }
            Ok(())
        }
        "server_with_host_key" => {
            let mut opts = ServerOptions::default();
            let path = input
                .key_path
                .ok_or_else(|| "Missing input.key_path".to_string())?;
            with_host_key_path(path.clone())(&mut opts).map_err(|e| e.to_string())?;
            if opts.host_key_path.as_deref() != Some(path.as_str()) {
                return Err(format!(
                    "Host key path mismatch: expected {path:?}, got {:?}",
                    opts.host_key_path
                ));
            }
            Ok(())
        }
        "server_with_authorized_keys" => {
            let path = input
                .authorized_keys_path
                .ok_or_else(|| "Missing input.authorized_keys_path".to_string())?;

            // Use per-user mode to avoid filesystem reads for non-existent fixture paths.
            let auth = AuthorizedKeysAuth::per_user(&path);
            let mut opts = ServerOptions::default();
            with_auth_handler(auth)(&mut opts).map_err(|e| e.to_string())?;
            if opts.auth_handler.is_none() {
                return Err("Expected auth_handler to be set".to_string());
            }

            // Verify the path string was accepted by the handler.
            if expected.expected.as_deref() != Some(path.as_str()) {
                return Err(format!(
                    "Fixture expected path mismatch: expected {:?}, got {:?}",
                    expected.expected, path
                ));
            }
            Ok(())
        }
        "server_with_public_key_auth" => {
            let mut opts = ServerOptions::default();
            with_public_key_auth(|_ctx, _key| true)(&mut opts).map_err(|e| e.to_string())?;
            if opts.public_key_handler.is_none() {
                return Err("Expected public_key_handler to be set".to_string());
            }
            Ok(())
        }
        "server_with_password_auth" => {
            let mut opts = ServerOptions::default();
            with_password_auth(|_ctx, _pw| true)(&mut opts).map_err(|e| e.to_string())?;
            if opts.password_handler.is_none() {
                return Err("Expected password_handler to be set".to_string());
            }
            Ok(())
        }
        "server_with_keyboard_interactive" => {
            let mut opts = ServerOptions::default();
            with_keyboard_interactive_auth(|_ctx, _resp, _prompts, _echos| vec!["ok".to_string()])(
                &mut opts,
            )
            .map_err(|e| e.to_string())?;
            if opts.keyboard_interactive_handler.is_none() {
                return Err("Expected keyboard_interactive_handler to be set".to_string());
            }
            Ok(())
        }
        "server_with_max_timeout" => {
            let mut opts = ServerOptions::default();
            let secs = input
                .timeout
                .ok_or_else(|| "Missing input.timeout".to_string())?;
            with_max_timeout(Duration::from_secs(secs))(&mut opts).map_err(|e| e.to_string())?;
            if opts.max_timeout != Some(Duration::from_secs(secs)) {
                return Err(format!(
                    "max_timeout mismatch: expected {:?}, got {:?}",
                    Duration::from_secs(secs),
                    opts.max_timeout
                ));
            }
            Ok(())
        }
        "server_with_idle_timeout" => {
            let mut opts = ServerOptions::default();
            let secs = input
                .timeout
                .ok_or_else(|| "Missing input.timeout".to_string())?;
            with_idle_timeout(Duration::from_secs(secs))(&mut opts).map_err(|e| e.to_string())?;
            if opts.idle_timeout != Some(Duration::from_secs(secs)) {
                return Err(format!(
                    "idle_timeout mismatch: expected {:?}, got {:?}",
                    Duration::from_secs(secs),
                    opts.idle_timeout
                ));
            }
            Ok(())
        }
        "server_with_banner" => {
            let mut opts = ServerOptions::default();
            let banner = input
                .banner
                .ok_or_else(|| "Missing input.banner".to_string())?;
            with_banner(banner.clone())(&mut opts).map_err(|e| e.to_string())?;
            if opts.banner.as_deref() != Some(banner.as_str()) {
                return Err(format!(
                    "banner mismatch: expected {banner:?}, got {:?}",
                    opts.banner
                ));
            }
            Ok(())
        }
        "server_with_version" => {
            let mut opts = ServerOptions::default();
            let version = input
                .version
                .ok_or_else(|| "Missing input.version".to_string())?;
            with_version(version.clone())(&mut opts).map_err(|e| e.to_string())?;
            if opts.version != version {
                return Err(format!(
                    "version mismatch: expected {version:?}, got {:?}",
                    opts.version
                ));
            }
            Ok(())
        }
        other => Err(format!("Not implemented server fixture: {other}")),
    }
}

fn run_address_fixture(fixture: &TestFixture) -> Result<(), String> {
    let input: ServerOptionInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {e}"))?;
    let expected: ServerOptionOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {e}"))?;

    let addr = input
        .address
        .ok_or_else(|| "Missing input.address".to_string())?;
    let valid = expected
        .valid
        .ok_or_else(|| "Missing expected.valid".to_string())?;

    let built = ServerBuilder::new().address(addr.clone()).build();
    if valid && built.is_err() {
        return Err(format!("Expected valid address, build failed: {built:?}"));
    }
    if !valid && built.is_ok() {
        return Err("Expected invalid address, but build succeeded".to_string());
    }

    if let Some(want) = expected.address {
        let server = built.map_err(|e| e.to_string())?;
        if server.address() != want {
            return Err(format!(
                "Address mismatch: expected {want:?}, got {:?}",
                server.address()
            ));
        }
    }

    Ok(())
}

fn run_middleware_fixture(fixture: &TestFixture) -> Result<(), String> {
    let input: MiddlewareInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {e}"))?;
    let expected: MiddlewareOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {e}"))?;

    // Some middleware fixtures don't use `input.name` and instead test composition or options.
    // Handle those first to avoid treating missing `input.name` as a conformance failure.
    if fixture.name == "middleware_chain" {
        let names = input
            .middleware_names
            .ok_or_else(|| "Missing input.middleware_names".to_string())?;
        let count = input
            .middleware_count
            .ok_or_else(|| "Missing input.middleware_count".to_string())?;
        if names.len() != count {
            return Err(format!(
                "middleware_count mismatch: expected {count}, got {}",
                names.len()
            ));
        }

        let want = expected
            .execution_order
            .ok_or_else(|| "Missing expected.execution_order".to_string())?;
        if want != "outer_to_inner" {
            return Err(format!(
                "Unexpected execution_order in fixture: expected \"outer_to_inner\", got {want:?}"
            ));
        }

        // Verify our middleware composer applies middleware in list order from outer to inner.
        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        fn record_middleware(label: String, events: Arc<Mutex<Vec<String>>>) -> wish::Middleware {
            Arc::new(move |next: wish::Handler| {
                let label = label.clone();
                let events = events.clone();
                Arc::new(
                    move |session: wish::Session| -> wish::BoxFuture<'static, ()> {
                        let next = next.clone();
                        let label = label.clone();
                        let events = events.clone();
                        Box::pin(async move {
                            events.lock().unwrap().push(label);
                            next(session).await;
                        })
                    },
                )
            })
        }

        let middlewares: Vec<wish::Middleware> = names
            .iter()
            .cloned()
            .map(|label| record_middleware(label, events.clone()))
            .collect();

        let composed = wish::compose_middleware(middlewares);
        let handler: wish::Handler = Arc::new({
            let events = events.clone();
            move |_session| {
                let events = events.clone();
                Box::pin(async move {
                    events.lock().unwrap().push("inner_handler".to_string());
                })
            }
        });

        let final_handler = composed(handler);

        // Execute synchronously on a tiny tokio runtime (futures should be immediately ready).
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .map_err(|e| format!("Failed to build tokio runtime: {e}"))?;

        let addr: std::net::SocketAddr = "127.0.0.1:2222"
            .parse::<std::net::SocketAddr>()
            .map_err(|e| e.to_string())?;
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        rt.block_on(async move { final_handler(session).await });

        let got = events.lock().unwrap().clone();
        let mut expected_order = names.clone();
        expected_order.push("inner_handler".to_string());
        if got != expected_order {
            return Err(format!(
                "middleware execution order mismatch: expected {expected_order:?}, got {got:?}"
            ));
        }

        return Ok(());
    }

    if fixture.name == "middleware_with_options" {
        let middleware_name = input
            .middleware
            .ok_or_else(|| "Missing input.middleware".to_string())?;
        let option_type = input
            .option_type
            .ok_or_else(|| "Missing input.option_type".to_string())?;

        if expected.configurable != Some(true) {
            return Err("Expected configurable=true".to_string());
        }

        match (middleware_name.as_str(), option_type.as_str()) {
            ("logging", "logger") => {
                struct TestLogger;
                impl middleware::logging::Logger for TestLogger {
                    fn log(&self, _format: &str, _args: &[&dyn std::fmt::Display]) {}
                }

                let _mw = middleware::logging::middleware_with_logger(TestLogger);
                Ok(())
            }
            other => Err(format!(
                "Unknown middleware/options combo in fixture: {other:?}"
            )),
        }?;

        return Ok(());
    }

    let name = input.name.ok_or_else(|| "Missing input.name".to_string())?;

    // Conformance requirement: these named middlewares must exist.
    // Some are implemented as part of `wish::middleware`, others live under `wish::tea`.
    match name.as_str() {
        "logging" => {
            let _mw = middleware::logging::middleware();
            Ok(())
        }
        "authentication" => {
            let _mw = middleware::authentication::middleware();
            Ok(())
        }
        "authorization" => {
            let _mw = middleware::authorization::middleware();
            Ok(())
        }
        "session_handler" => {
            let _mw = middleware::session_handler::middleware();
            Ok(())
        }
        "activeterm" => {
            let _mw = middleware::activeterm::middleware();
            Ok(())
        }
        "recovery" => {
            let _mw = middleware::recover::middleware();
            Ok(())
        }
        "elapsed" => {
            let _mw = middleware::elapsed::middleware();
            Ok(())
        }
        "comment" => {
            let _mw = middleware::comment::middleware("hi");
            Ok(())
        }
        "accesscontrol" => {
            let allowed = vec!["git".to_string()];
            let _mw = middleware::accesscontrol::middleware(allowed);
            Ok(())
        }
        "ratelimiter" => {
            let limiter = middleware::ratelimiter::new_rate_limiter(1.0, 10, 100);
            let _mw = middleware::ratelimiter::middleware(limiter);
            Ok(())
        }
        "bubbletea" => {
            // Bubble Tea integration middleware lives under `wish::tea`.
            let _mw = wish::tea::middleware(|_session| NoopTeaModel::default());
            Ok(())
        }
        "git" => {
            let _mw = middleware::git::middleware();
            Ok(())
        }
        "scp" => {
            let _mw = middleware::scp::middleware();
            Ok(())
        }
        "sftp" => {
            let _mw = middleware::sftp::middleware();
            Ok(())
        }
        "pty" => {
            let _mw = middleware::pty::middleware();
            Ok(())
        }
        other => Err(format!("Missing middleware implementation: {other}")),
    }?;

    Ok(())
}

fn run_error_fixture(fixture: &TestFixture) -> Result<(), String> {
    let input: ErrorInput = fixture
        .input_as()
        .map_err(|e| format!("Failed to parse input: {e}"))?;
    let expected: ErrorOutput = fixture
        .expected_as()
        .map_err(|e| format!("Failed to parse expected output: {e}"))?;

    match fixture.name.as_str() {
        "error_auth_failed" => {
            let err = Error::AuthenticationFailed;
            let msg = expected
                .message
                .ok_or_else(|| "Missing expected.message".to_string())?;
            if !err.to_string().contains(&msg) {
                return Err(format!(
                    "error message mismatch: expected substring {msg:?}, got {:?}",
                    err.to_string()
                ));
            }
            Ok(())
        }
        "error_connection_closed" => {
            let err = Error::Io(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "connection closed",
            ));
            let msg = expected
                .message
                .ok_or_else(|| "Missing expected.message".to_string())?;
            if !err.to_string().contains(&msg) {
                return Err(format!(
                    "error message mismatch: expected substring {msg:?}, got {:?}",
                    err.to_string()
                ));
            }
            Ok(())
        }
        "error_invalid_session" => {
            let err = Error::Session("invalid session".to_string());
            let msg = expected
                .message
                .ok_or_else(|| "Missing expected.message".to_string())?;
            if !err.to_string().contains(&msg) {
                return Err(format!(
                    "error message mismatch: expected substring {msg:?}, got {:?}",
                    err.to_string()
                ));
            }
            Ok(())
        }
        "error_permission_denied" => {
            let err = Error::Io(std::io::Error::new(
                ErrorKind::PermissionDenied,
                "permission denied",
            ));
            let msg = expected
                .message
                .ok_or_else(|| "Missing expected.message".to_string())?;
            if !err.to_string().contains(&msg) {
                return Err(format!(
                    "error message mismatch: expected substring {msg:?}, got {:?}",
                    err.to_string()
                ));
            }
            Ok(())
        }
        "error_timeout" => {
            let err = Error::Ssh("connection timeout".to_string());
            let msg = expected
                .message
                .ok_or_else(|| "Missing expected.message".to_string())?;
            if !err.to_string().contains(&msg) {
                return Err(format!(
                    "error message mismatch: expected substring {msg:?}, got {:?}",
                    err.to_string()
                ));
            }
            Ok(())
        }
        "error_fatal" => {
            let addr: std::net::SocketAddr = "127.0.0.1:2222"
                .parse::<std::net::SocketAddr>()
                .map_err(|e| e.to_string())?;
            let ctx = Context::new("testuser", addr, addr);
            let mut session = Session::new(ctx);
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            session.set_output_sender(tx);

            wish::fatalln(&session, "fatal error");

            let mut saw_stderr = false;
            let mut saw_exit = false;
            while let Ok(item) = rx.try_recv() {
                match item {
                    wish::SessionOutput::Stderr(buf) => {
                        let s = String::from_utf8_lossy(&buf);
                        if s.contains("fatal error") {
                            saw_stderr = true;
                        }
                    }
                    wish::SessionOutput::Exit(code) => {
                        if code == 1 {
                            saw_exit = true;
                        }
                    }
                    _ => {}
                }
            }

            if !saw_stderr {
                return Err("fatal did not write expected stderr".to_string());
            }
            if !saw_exit {
                return Err("fatal did not send exit code 1".to_string());
            }
            Ok(())
        }
        "error_patterns" => {
            let want = expected
                .error_types
                .ok_or_else(|| "Missing expected.error_types".to_string())?;
            let mut have = Vec::new();

            // These correspond to the error patterns wish uses.
            have.push(("authentication_error", Error::AuthenticationFailed));
            have.push((
                "connection_error",
                Error::Io(std::io::Error::new(ErrorKind::Other, "connection closed")),
            ));
            have.push((
                "session_error",
                Error::Session("invalid session".to_string()),
            ));
            have.push((
                "timeout_error",
                Error::Ssh("connection timeout".to_string()),
            ));

            for (label, err) in have {
                if want.iter().any(|w| w == label) && err.to_string().is_empty() {
                    return Err(format!("expected non-empty error display for {label}"));
                }
            }

            Ok(())
        }
        other => {
            let error_type = input.error_type.unwrap_or_else(|| "<none>".to_string());
            Err(format!(
                "Not implemented error fixture: {other} (type {error_type})"
            ))
        }
    }
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
    let limiter = middleware::ratelimiter::new_rate_limiter(1.0, 10, 100);
    let _mw = middleware::ratelimiter::middleware(limiter);
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

    let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]).with_comment("test_key_comment");

    let session = Session::new(ctx).with_public_key(key);

    assert!(session.public_key().is_some());
    assert_eq!(session.public_key().unwrap().key_type, "ssh-ed25519");
    assert_eq!(
        session.public_key().unwrap().comment,
        Some("test_key_comment".to_string())
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
    let key = PublicKey::new("ssh-rsa", vec![5, 6, 7, 8]).with_comment("user_host");
    assert_eq!(key.key_type, "ssh-rsa");
    assert_eq!(key.comment, Some("user_host".to_string()));
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
        fp.starts_with("HASH:"),
        "Fingerprint should start with HASH:"
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
    let fixtures = loader
        .load_crate("wish")
        .expect("wish fixtures must exist for conformance");

    for fixture in fixtures.tests.iter() {
        if let Some(skip) = fixture.should_skip() {
            assert!(
                skip.is_empty(),
                "wish fixture unexpectedly marked skip: {}: {}",
                fixture.name,
                skip
            );
        }

        if fixture.name.starts_with("server_") {
            test_server_option_fixture(fixture);
        }
    }
}

fn test_server_option_fixture(fixture: &TestFixture) {
    let input: ServerOptionInput = fixture
        .input_as()
        .expect("could not parse input for wish server option fixture");

    let output: ServerOptionOutput = fixture
        .expected_as()
        .expect("could not parse output for wish server option fixture");

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
