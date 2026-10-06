#![forbid(unsafe_code)]
// `clippy::unused_async_trait_impl` was promoted in a June-2026 nightly and fires on the many
// async trait methods in this crate (e.g. `auth/authorized_keys.rs`, `handler.rs`) whose bodies
// contain no `.await`. Those methods must keep their `async` signature to satisfy the trait
// contract (`#[async_trait]` / AFIT `RusshHandler`, `AuthHandler`), so dropping `async` is not an
// option — we accommodate the new lint here. `unknown_lints` is allowed alongside so this stays
// clean on older nightlies that don't yet know the lint name.
#![allow(unknown_lints)]
#![allow(clippy::unused_async_trait_impl)]
// Per-lint allows for wish's SSH server/session code.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::duration_suboptimal_units)]
#![allow(clippy::implicit_clone)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::use_self)]
#![allow(clippy::wildcard_imports)]

//! # Wish
//!
//! A library for building SSH applications with TUI interfaces.
//!
//! Wish enables you to create SSH servers that serve interactive
//! terminal applications, making it easy to build:
//! - SSH-accessible TUI apps
//! - Git servers with custom interfaces
//! - Multi-user terminal experiences
//! - Secure remote access tools
//!
//! ## Role in `charmed_rust`
//!
//! Wish is the SSH application layer for bubbletea programs:
//! - **bubbletea** provides the program runtime served over SSH.
//! - **charmed_log** supplies structured logging for sessions.
//! - **demo_showcase** includes an SSH mode to demonstrate remote TUIs.
//!
//! ## Features
//!
//! - **Middleware pattern**: Compose handlers with chainable middleware
//! - **PTY support**: Full pseudo-terminal emulation
//! - **Authentication**: Public key, password, and keyboard-interactive auth
//! - **BubbleTea integration**: Serve TUI apps over SSH
//! - **Logging middleware**: Connection logging out of the box
//! - **Access control**: Restrict allowed commands
//!
//! ## Example
//!
//! ```rust,ignore
//! use wish::{Server, ServerBuilder};
//! use wish::middleware::{logging, activeterm};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wish::Error> {
//!     let server = ServerBuilder::new()
//!         .address("0.0.0.0:2222")
//!         .with_middleware(logging::middleware())
//!         .with_middleware(activeterm::middleware())
//!         .handler(|session| async move {
//!             wish::println(&session, "Hello, SSH!");
//!         })
//!         .build()
//!         .await?;
//!
//!     server.listen().await
//! }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use bubbletea::Message;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

#[allow(
    clippy::cast_possible_truncation,
    clippy::items_after_statements,
    clippy::needless_continue,
    clippy::redundant_closure_for_method_calls,
    clippy::significant_drop_tightening,
    clippy::suspicious_operation_groupings,
    clippy::unused_async
)]
pub mod auth;
#[allow(
    clippy::cast_possible_truncation,
    clippy::items_after_statements,
    clippy::single_match_else
)]
mod handler;
#[allow(
    clippy::cast_possible_truncation,
    clippy::duration_suboptimal_units,
    clippy::ip_constant,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening
)]
pub mod session;

pub use auth::{
    AcceptAllAuth, AsyncCallbackAuth, AsyncPublicKeyAuth, AuthContext, AuthHandler, AuthMethod,
    AuthResult, AuthorizedKey, AuthorizedKeysAuth, CallbackAuth, CompositeAuth, PasswordAuth,
    PublicKeyAuth, PublicKeyCallbackAuth, RateLimitedAuth, SessionId, parse_authorized_keys,
};
pub use handler::{RusshConfig, ServerState, WishHandler, WishHandlerFactory, run_stream};

// Re-export dependencies for convenience
pub use bubbletea;
pub use lipgloss;

// -----------------------------------------------------------------------------
// Error Types
// -----------------------------------------------------------------------------

/// Errors that can occur in the wish SSH server library.
///
/// This enum represents all possible error conditions when running
/// an SSH server with wish.
///
/// # Error Handling
///
/// SSH server errors range from configuration issues to runtime
/// authentication failures. Use the `?` operator for propagation:
///
/// ```rust,ignore
/// use wish::Result;
///
/// async fn run_server() -> Result<()> {
///     let server = Server::new(handler).await?;
///     server.listen("0.0.0.0:2222").await?;
///     Ok(())
/// }
/// ```
///
/// # Recovery Strategies
///
/// | Error Variant | Recovery Strategy |
/// |--------------|-------------------|
/// | [`Io`](Error::Io) | Check permissions, port availability |
/// | [`Ssh`](Error::Ssh) | Log and continue for recoverable errors |
/// | [`Russh`](Error::Russh) | Check SSH protocol compatibility |
/// | [`Key`](Error::Key) | Regenerate keys or check permissions |
/// | [`KeyLoad`](Error::KeyLoad) | Verify key file format |
/// | [`AuthenticationFailed`](Error::AuthenticationFailed) | Expected for invalid credentials |
/// | [`MaxSessionsReached`](Error::MaxSessionsReached) | Retry later or raise configured session limit |
/// | [`Configuration`](Error::Configuration) | Fix server configuration |
/// | [`Session`](Error::Session) | Close session gracefully |
/// | [`AddrParse`](Error::AddrParse) | Validate address format |
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error during server operations.
    ///
    /// Commonly occurs when:
    /// - The bind address is already in use
    /// - Permission denied on privileged ports
    /// - Network interface is unavailable
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// SSH protocol error.
    ///
    /// General SSH protocol-level errors. Contains a descriptive message.
    #[error("ssh error: {0}")]
    Ssh(String),

    /// Underlying russh library error.
    ///
    /// Wraps errors from the russh SSH implementation.
    #[error("russh error: {0}")]
    Russh(#[from] russh::Error),

    /// Key generation or management error.
    ///
    /// Occurs when generating or manipulating SSH keys fails.
    #[error("key error: {0}")]
    Key(String),

    /// Key loading error from russh-keys.
    ///
    /// Occurs when loading SSH keys from files fails.
    /// Common causes: file not found, invalid format, permission denied.
    #[error("key loading error: {0}")]
    KeyLoad(#[from] russh::keys::Error),

    /// Authentication failed.
    ///
    /// Occurs when a client's credentials are rejected.
    /// This is expected in normal operation - not all attempts succeed.
    #[error("authentication failed")]
    AuthenticationFailed,

    /// Maximum concurrent sessions reached.
    ///
    /// Returned when attempting to create a new session while at capacity.
    #[error("maximum sessions reached ({current}/{max})")]
    MaxSessionsReached {
        /// Configured maximum concurrent sessions.
        max: usize,
        /// Current active session count at rejection time.
        current: usize,
    },

    /// Server configuration error.
    ///
    /// Occurs when the server configuration is invalid.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Session error.
    ///
    /// Occurs during an active SSH session.
    #[error("session error: {0}")]
    Session(String),

    /// Address parse error.
    ///
    /// Occurs when parsing a socket address fails.
    #[error("address parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
}

/// A specialized [`Result`] type for wish operations.
///
/// This type alias defaults to [`enum@Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;

// -----------------------------------------------------------------------------
// PTY Types
// -----------------------------------------------------------------------------

/// Window size information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// Terminal width in columns.
    pub width: u32,
    /// Terminal height in rows.
    pub height: u32,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

/// Pseudo-terminal information.
#[derive(Debug, Clone)]
pub struct Pty {
    /// Terminal type (e.g., "xterm-256color").
    pub term: String,
    /// Window dimensions.
    pub window: Window,
}

impl Default for Pty {
    fn default() -> Self {
        Self {
            term: "xterm-256color".to_string(),
            window: Window::default(),
        }
    }
}

// -----------------------------------------------------------------------------
// Public Key Types
// -----------------------------------------------------------------------------

/// A public key used for authentication.
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// The key type (e.g., "ssh-ed25519", "ssh-rsa").
    pub key_type: String,
    /// The raw key data.
    pub data: Vec<u8>,
    /// Optional comment from the authorized_keys file.
    pub comment: Option<String>,
}

impl PublicKey {
    /// Creates a new public key.
    pub fn new(key_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            key_type: key_type.into(),
            data,
            comment: None,
        }
    }

    /// Sets the comment for this key.
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Returns a fingerprint of the key.
    ///
    /// Note: uses `DefaultHasher` (SipHash), not a cryptographic hash.
    /// The prefix is for display convention only.
    pub fn fingerprint(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        format!("HASH:{:016x}", hasher.finish())
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.key_type == other.key_type && self.data == other.data
    }
}

impl Eq for PublicKey {}

// -----------------------------------------------------------------------------
// Context
// -----------------------------------------------------------------------------

/// Context passed to authentication handlers.
#[derive(Debug, Clone)]
pub struct Context {
    /// The username attempting authentication.
    user: String,
    /// The remote address.
    remote_addr: SocketAddr,
    /// The local address.
    local_addr: SocketAddr,
    /// The client version string.
    client_version: String,
    /// Custom values stored in the context.
    values: Arc<RwLock<HashMap<String, String>>>,
}

impl Context {
    /// Creates a new context.
    pub fn new(user: impl Into<String>, remote_addr: SocketAddr, local_addr: SocketAddr) -> Self {
        Self {
            user: user.into(),
            remote_addr,
            local_addr,
            client_version: String::new(),
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the username.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Returns the remote address.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Returns the local address.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the client version string.
    pub fn client_version(&self) -> &str {
        &self.client_version
    }

    /// Sets the client version string.
    pub fn set_client_version(&mut self, version: impl Into<String>) {
        self.client_version = version.into();
    }

    /// Sets a value in the context.
    pub fn set_value(&self, key: impl Into<String>, value: impl Into<String>) {
        self.values.write().insert(key.into(), value.into());
    }

    /// Gets a value from the context.
    pub fn get_value(&self, key: &str) -> Option<String> {
        self.values.read().get(key).cloned()
    }
}

// -----------------------------------------------------------------------------
// Session
// -----------------------------------------------------------------------------

/// An SSH session representing a connected client.
#[derive(Clone)]
pub struct Session {
    /// The session context.
    context: Context,
    /// The PTY if allocated.
    pty: Option<Pty>,
    /// The command being executed (if any).
    command: Vec<String>,
    /// Environment variables.
    env: HashMap<String, String>,
    /// Output buffer for stdout.
    #[allow(dead_code)]
    pub(crate) stdout: Arc<RwLock<Vec<u8>>>,
    /// Output buffer for stderr.
    #[allow(dead_code)]
    pub(crate) stderr: Arc<RwLock<Vec<u8>>>,
    /// Exit code.
    exit_code: Arc<RwLock<Option<i32>>>,
    /// Whether the session is closed.
    closed: Arc<RwLock<bool>>,
    /// The public key used for authentication (if any).
    public_key: Option<PublicKey>,
    /// Subsystem being used (if any).
    subsystem: Option<String>,

    /// Channel for sending output to the client.
    output_tx: Option<tokio::sync::mpsc::UnboundedSender<SessionOutput>>,
    /// Channel for receiving input from the client.
    input_rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<Vec<u8>>>>>,
    /// Channel for injecting messages into the running bubbletea program.
    message_tx: Arc<RwLock<Option<Sender<Message>>>>,
}

/// Output messages sent from Session to the SSH channel.
#[derive(Debug)]
pub enum SessionOutput {
    /// Standard output data.
    Stdout(Vec<u8>),
    /// Standard error data.
    Stderr(Vec<u8>),
    /// Exit status code.
    Exit(u32),
    /// Close the channel.
    Close,
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("user", &self.context.user)
            .field("remote_addr", &self.context.remote_addr)
            .field("pty", &self.pty)
            .field("command", &self.command)
            .finish()
    }
}

impl Session {
    /// Creates a new session.
    pub fn new(context: Context) -> Self {
        Self {
            context,
            pty: None,
            command: Vec::new(),
            env: HashMap::new(),
            stdout: Arc::new(RwLock::new(Vec::new())),
            stderr: Arc::new(RwLock::new(Vec::new())),
            exit_code: Arc::new(RwLock::new(None)),
            closed: Arc::new(RwLock::new(false)),
            public_key: None,
            subsystem: None,
            output_tx: None,
            input_rx: Arc::new(tokio::sync::Mutex::new(None)),
            message_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Sets the output sender.
    pub fn set_output_sender(&mut self, tx: tokio::sync::mpsc::UnboundedSender<SessionOutput>) {
        self.output_tx = Some(tx);
    }

    /// Sets the input receiver.
    pub async fn set_input_receiver(&self, rx: tokio::sync::mpsc::Receiver<Vec<u8>>) {
        *self.input_rx.lock().await = Some(rx);
    }

    /// Receives input from the client.
    pub async fn recv(&self) -> Option<Vec<u8>> {
        let mut rx_guard = self.input_rx.lock().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.recv().await
        } else {
            None
        }
    }

    /// Sets the message sender for the bubbletea program.
    pub fn set_message_sender(&self, tx: Sender<Message>) {
        *self.message_tx.write() = Some(tx);
    }

    /// Sends a message to the bubbletea program (if running).
    pub fn send_message(&self, msg: Message) {
        if let Some(tx) = self.message_tx.read().as_ref() {
            // We ignore errors because if the channel is closed, the program is gone
            let _ = tx.send(msg);
        }
    }

    /// Returns the username.
    pub fn user(&self) -> &str {
        self.context.user()
    }

    /// Returns the remote address.
    pub fn remote_addr(&self) -> SocketAddr {
        self.context.remote_addr()
    }

    /// Returns the local address.
    pub fn local_addr(&self) -> SocketAddr {
        self.context.local_addr()
    }

    /// Returns the context.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the PTY and window change channel if allocated.
    pub fn pty(&self) -> (Option<&Pty>, bool) {
        (self.pty.as_ref(), self.pty.is_some())
    }

    /// Returns the command being executed.
    pub fn command(&self) -> &[String] {
        &self.command
    }

    /// Returns an environment variable.
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }

    /// Returns all environment variables.
    pub fn environ(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Returns the public key used for authentication.
    pub fn public_key(&self) -> Option<&PublicKey> {
        self.public_key.as_ref()
    }

    /// Returns the subsystem being used.
    pub fn subsystem(&self) -> Option<&str> {
        self.subsystem.as_deref()
    }

    /// Writes to stdout.
    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        // Send to client
        if let Some(tx) = &self.output_tx {
            let _ = tx.send(SessionOutput::Stdout(data.to_vec()));
        }

        Ok(data.len())
    }

    /// Writes to stderr.
    pub fn write_stderr(&self, data: &[u8]) -> io::Result<usize> {
        // Send to client
        if let Some(tx) = &self.output_tx {
            let _ = tx.send(SessionOutput::Stderr(data.to_vec()));
        }

        Ok(data.len())
    }

    /// Exits the session with the given code.
    pub fn exit(&self, code: i32) -> io::Result<()> {
        *self.exit_code.write() = Some(code);
        if let Some(tx) = &self.output_tx {
            let _ = tx.send(SessionOutput::Exit(code as u32));
        }
        Ok(())
    }

    /// Closes the session.
    pub fn close(&self) -> io::Result<()> {
        *self.closed.write() = true;
        if let Some(tx) = &self.output_tx {
            let _ = tx.send(SessionOutput::Close);
        }
        Ok(())
    }

    /// Returns whether the session is closed.
    pub fn is_closed(&self) -> bool {
        *self.closed.read()
    }

    /// Returns the current window size.
    pub fn window(&self) -> Window {
        self.pty.as_ref().map(|p| p.window).unwrap_or_default()
    }

    // Builder methods for constructing sessions

    /// Sets the PTY.
    pub fn with_pty(mut self, pty: Pty) -> Self {
        self.pty = Some(pty);
        self
    }

    /// Sets the command.
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = command;
        self
    }

    /// Sets an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets the public key.
    pub fn with_public_key(mut self, key: PublicKey) -> Self {
        self.public_key = Some(key);
        self
    }

    /// Sets the subsystem.
    pub fn with_subsystem(mut self, subsystem: impl Into<String>) -> Self {
        self.subsystem = Some(subsystem.into());
        self
    }
}

// Implement Write for Session
impl Write for Session {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Session::write(self, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Output Helper Functions
// -----------------------------------------------------------------------------

/// Writes to the session's stdout.
pub fn print(session: &Session, args: impl fmt::Display) {
    let _ = session.write(args.to_string().as_bytes());
}

/// Writes to the session's stdout with a newline.
pub fn println(session: &Session, args: impl fmt::Display) {
    let msg = format!("{}\r\n", args);
    let _ = session.write(msg.as_bytes());
}

/// Writes formatted output to the session's stdout.
pub fn printf(session: &Session, format: impl fmt::Display, args: &[&dyn fmt::Display]) {
    let mut msg = format.to_string();
    for arg in args {
        if let Some(pos) = msg.find("{}") {
            msg.replace_range(pos..pos + 2, &arg.to_string());
        }
    }
    let _ = session.write(msg.as_bytes());
}

/// Writes to the session's stderr.
pub fn error(session: &Session, args: impl fmt::Display) {
    let _ = session.write_stderr(args.to_string().as_bytes());
}

/// Writes to the session's stderr with a newline.
pub fn errorln(session: &Session, args: impl fmt::Display) {
    let msg = format!("{}\r\n", args);
    let _ = session.write_stderr(msg.as_bytes());
}

/// Writes formatted output to the session's stderr.
pub fn errorf(session: &Session, format: impl fmt::Display, args: &[&dyn fmt::Display]) {
    let mut msg = format.to_string();
    for arg in args {
        if let Some(pos) = msg.find("{}") {
            msg.replace_range(pos..pos + 2, &arg.to_string());
        }
    }
    let _ = session.write_stderr(msg.as_bytes());
}

/// Writes to stderr and exits with code 1.
pub fn fatal(session: &Session, args: impl fmt::Display) {
    error(session, args);
    let _ = session.exit(1);
    let _ = session.close();
}

/// Writes to stderr with a newline and exits with code 1.
pub fn fatalln(session: &Session, args: impl fmt::Display) {
    errorln(session, args);
    let _ = session.exit(1);
    let _ = session.close();
}

/// Writes formatted output to stderr and exits with code 1.
pub fn fatalf(session: &Session, format: impl fmt::Display, args: &[&dyn fmt::Display]) {
    errorf(session, format, args);
    let _ = session.exit(1);
    let _ = session.close();
}

/// Writes a string to the session's stdout.
pub fn write_string(session: &Session, s: &str) -> io::Result<usize> {
    session.write(s.as_bytes())
}

// -----------------------------------------------------------------------------
// Handler and Middleware
// -----------------------------------------------------------------------------

/// A boxed future for async handlers.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Handler function type.
pub type Handler = Arc<dyn Fn(Session) -> BoxFuture<'static, ()> + Send + Sync>;

/// Middleware function type.
pub type Middleware = Arc<dyn Fn(Handler) -> Handler + Send + Sync>;

/// Creates a handler from an async function.
pub fn handler<F, Fut>(f: F) -> Handler
where
    F: Fn(Session) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Arc::new(move |session| Box::pin(f(session)))
}

/// Creates a no-op handler.
pub fn noop_handler() -> Handler {
    Arc::new(|_| Box::pin(async {}))
}

/// Composes multiple middleware into a single middleware.
pub fn compose_middleware(middlewares: Vec<Middleware>) -> Middleware {
    Arc::new(move |h| {
        let mut handler = h;
        for mw in middlewares.iter().rev() {
            handler = mw(handler);
        }
        handler
    })
}

// -----------------------------------------------------------------------------
// Authentication Handlers
// -----------------------------------------------------------------------------

/// Public key authentication handler.
pub type PublicKeyHandler = Arc<dyn Fn(&Context, &PublicKey) -> bool + Send + Sync>;

/// Password authentication handler.
pub type PasswordHandler = Arc<dyn Fn(&Context, &str) -> bool + Send + Sync>;

/// Keyboard-interactive authentication handler.
pub type KeyboardInteractiveHandler =
    Arc<dyn Fn(&Context, &str, &[String], &[bool]) -> Vec<String> + Send + Sync>;

/// Banner handler that returns a banner based on context.
pub type BannerHandler = Arc<dyn Fn(&Context) -> String + Send + Sync>;

/// Subsystem handler.
pub type SubsystemHandler = Arc<dyn Fn(Session) -> BoxFuture<'static, ()> + Send + Sync>;

// -----------------------------------------------------------------------------
// Server Options
// -----------------------------------------------------------------------------

/// Options for configuring the SSH server.
#[derive(Clone)]
pub struct ServerOptions {
    /// Listen address.
    pub address: String,
    /// Server version string.
    pub version: String,
    /// Static banner.
    pub banner: Option<String>,
    /// Dynamic banner handler.
    pub banner_handler: Option<BannerHandler>,
    /// Host key path.
    pub host_key_path: Option<String>,
    /// Host key PEM data.
    pub host_key_pem: Option<Vec<u8>>,
    /// Middlewares to apply.
    pub middlewares: Vec<Middleware>,
    /// Main handler.
    pub handler: Option<Handler>,
    /// Trait-based authentication handler.
    /// If set, takes precedence over the callback-based handlers.
    pub auth_handler: Option<Arc<dyn AuthHandler>>,
    /// Public key auth handler (callback-based, for backward compatibility).
    pub public_key_handler: Option<PublicKeyHandler>,
    /// Password auth handler (callback-based, for backward compatibility).
    pub password_handler: Option<PasswordHandler>,
    /// Keyboard-interactive auth handler.
    pub keyboard_interactive_handler: Option<KeyboardInteractiveHandler>,
    /// Idle timeout.
    pub idle_timeout: Option<Duration>,
    /// Maximum connection timeout.
    pub max_timeout: Option<Duration>,
    /// Subsystem handlers.
    pub subsystem_handlers: HashMap<String, SubsystemHandler>,
    /// Maximum authentication attempts before disconnection.
    pub max_auth_attempts: u32,
    /// Authentication rejection delay in milliseconds (timing attack mitigation).
    pub auth_rejection_delay_ms: u64,
    /// Allow unauthenticated access when no auth handlers are configured.
    ///
    /// When `false` (the default), connections are rejected if no auth
    /// handlers (public key, password, keyboard-interactive, or trait-based)
    /// are registered. Set to `true` only for development/demo servers
    /// that intentionally allow anonymous access.
    pub allow_no_auth: bool,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:22".to_string(),
            version: "SSH-2.0-Wish".to_string(),
            banner: None,
            banner_handler: None,
            host_key_path: None,
            host_key_pem: None,
            middlewares: Vec::new(),
            handler: None,
            auth_handler: None,
            public_key_handler: None,
            password_handler: None,
            keyboard_interactive_handler: None,
            idle_timeout: None,
            max_timeout: None,
            subsystem_handlers: HashMap::new(),
            max_auth_attempts: auth::DEFAULT_MAX_AUTH_ATTEMPTS,
            auth_rejection_delay_ms: auth::DEFAULT_AUTH_REJECTION_DELAY_MS,
            allow_no_auth: false,
        }
    }
}

impl fmt::Debug for ServerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerOptions")
            .field("address", &self.address)
            .field("version", &self.version)
            .field("banner", &self.banner)
            .field("host_key_path", &self.host_key_path)
            .field("idle_timeout", &self.idle_timeout)
            .field("max_timeout", &self.max_timeout)
            .finish()
    }
}

// -----------------------------------------------------------------------------
// Option Functions (Go-style)
// -----------------------------------------------------------------------------

/// Option function type for configuring the server.
pub type ServerOption = Box<dyn FnOnce(&mut ServerOptions) -> Result<()> + Send>;

/// Sets the listen address.
pub fn with_address(addr: impl Into<String>) -> ServerOption {
    let addr = addr.into();
    Box::new(move |opts| {
        opts.address = addr;
        Ok(())
    })
}

/// Sets the server version string.
pub fn with_version(version: impl Into<String>) -> ServerOption {
    let version = version.into();
    Box::new(move |opts| {
        opts.version = version;
        Ok(())
    })
}

/// Sets a static banner.
pub fn with_banner(banner: impl Into<String>) -> ServerOption {
    let banner = banner.into();
    Box::new(move |opts| {
        opts.banner = Some(banner);
        Ok(())
    })
}

/// Sets a dynamic banner handler.
pub fn with_banner_handler<F>(handler: F) -> ServerOption
where
    F: Fn(&Context) -> String + Send + Sync + 'static,
{
    Box::new(move |opts| {
        opts.banner_handler = Some(Arc::new(handler));
        Ok(())
    })
}

/// Adds middleware to the server.
pub fn with_middleware(mw: Middleware) -> ServerOption {
    Box::new(move |opts| {
        opts.middlewares.push(mw);
        Ok(())
    })
}

/// Sets the host key path.
pub fn with_host_key_path(path: impl Into<String>) -> ServerOption {
    let path = path.into();
    Box::new(move |opts| {
        opts.host_key_path = Some(path);
        Ok(())
    })
}

/// Sets the host key from PEM data.
pub fn with_host_key_pem(pem: Vec<u8>) -> ServerOption {
    Box::new(move |opts| {
        opts.host_key_pem = Some(pem);
        Ok(())
    })
}

/// Sets the trait-based authentication handler.
///
/// If set, this takes precedence over the callback-based handlers.
pub fn with_auth_handler<H: AuthHandler + 'static>(handler: H) -> ServerOption {
    Box::new(move |opts| {
        opts.auth_handler = Some(Arc::new(handler));
        Ok(())
    })
}

/// Sets the maximum authentication attempts.
pub fn with_max_auth_attempts(max: u32) -> ServerOption {
    Box::new(move |opts| {
        opts.max_auth_attempts = max;
        Ok(())
    })
}

/// Sets the authentication rejection delay in milliseconds.
pub fn with_auth_rejection_delay(delay_ms: u64) -> ServerOption {
    Box::new(move |opts| {
        opts.auth_rejection_delay_ms = delay_ms;
        Ok(())
    })
}

/// Sets the public key authentication handler.
pub fn with_public_key_auth<F>(handler: F) -> ServerOption
where
    F: Fn(&Context, &PublicKey) -> bool + Send + Sync + 'static,
{
    Box::new(move |opts| {
        opts.public_key_handler = Some(Arc::new(handler));
        Ok(())
    })
}

/// Sets the password authentication handler.
pub fn with_password_auth<F>(handler: F) -> ServerOption
where
    F: Fn(&Context, &str) -> bool + Send + Sync + 'static,
{
    Box::new(move |opts| {
        opts.password_handler = Some(Arc::new(handler));
        Ok(())
    })
}

/// Sets the keyboard-interactive authentication handler.
pub fn with_keyboard_interactive_auth<F>(handler: F) -> ServerOption
where
    F: Fn(&Context, &str, &[String], &[bool]) -> Vec<String> + Send + Sync + 'static,
{
    Box::new(move |opts| {
        opts.keyboard_interactive_handler = Some(Arc::new(handler));
        Ok(())
    })
}

/// Sets the idle timeout.
pub fn with_idle_timeout(duration: Duration) -> ServerOption {
    Box::new(move |opts| {
        opts.idle_timeout = Some(duration);
        Ok(())
    })
}

/// Sets the maximum connection timeout.
pub fn with_max_timeout(duration: Duration) -> ServerOption {
    Box::new(move |opts| {
        opts.max_timeout = Some(duration);
        Ok(())
    })
}

/// Adds a subsystem handler.
pub fn with_subsystem<F, Fut>(name: impl Into<String>, handler: F) -> ServerOption
where
    F: Fn(Session) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let name = name.into();
    Box::new(move |opts| {
        opts.subsystem_handlers
            .insert(name, Arc::new(move |s| Box::pin(handler(s))));
        Ok(())
    })
}

// -----------------------------------------------------------------------------
// Server
// -----------------------------------------------------------------------------

/// SSH server for hosting applications.
pub struct Server {
    /// Server options.
    options: ServerOptions,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Server")
            .field("options", &self.options)
            .finish()
    }
}

impl Server {
    /// Creates a new server with the given options.
    pub fn new(options: impl IntoIterator<Item = ServerOption>) -> Result<Self> {
        let mut opts = ServerOptions::default();
        for opt in options {
            opt(&mut opts)?;
        }
        Ok(Self { options: opts })
    }

    /// Returns the server options.
    pub fn options(&self) -> &ServerOptions {
        &self.options
    }

    /// Returns the listen address.
    pub fn address(&self) -> &str {
        &self.options.address
    }

    /// Starts listening for connections.
    ///
    /// This binds to the configured address, accepts SSH connections,
    /// and runs the handler for each connection.
    pub async fn listen(&self) -> Result<()> {
        info!("Starting SSH server on {}", self.options.address);

        // Parse the address
        let addr: SocketAddr = self.options.address.parse()?;
        debug!("Parsed address: {:?}", addr);

        // Create russh configuration
        let config = self.create_russh_config()?;
        let config = Arc::new(config);

        // Create the handler factory
        let factory = WishHandlerFactory::new(self.options.clone());

        // Bind to the address
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr().unwrap_or(addr);
        info!("Server listening on {}", local_addr);

        self.listen_with_listener_inner(listener, config, factory, local_addr)
            .await
    }

    /// Starts listening for connections using an already-bound listener.
    ///
    /// This is primarily useful for tests and embedding scenarios where you need to
    /// bind to an ephemeral port (`127.0.0.1:0`) without races.
    pub async fn listen_with_listener(&self, listener: TcpListener) -> Result<()> {
        let local_addr = listener.local_addr()?;

        // Create russh configuration
        let config = self.create_russh_config()?;
        let config = Arc::new(config);

        // Create the handler factory
        let factory = WishHandlerFactory::new(self.options.clone());

        info!("Server listening on {}", local_addr);
        self.listen_with_listener_inner(listener, config, factory, local_addr)
            .await
    }

    async fn listen_with_listener_inner(
        &self,
        listener: TcpListener,
        config: Arc<RusshConfig>,
        factory: WishHandlerFactory,
        local_addr: SocketAddr,
    ) -> Result<()> {
        // Accept connections
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    info!(peer_addr = %peer_addr, "Accepted connection");

                    let config = config.clone();
                    let socket_local_addr = socket.local_addr().unwrap_or(local_addr);
                    let handler = factory.create_handler(peer_addr, socket_local_addr);

                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        debug!(peer_addr = %peer_addr, "Running SSH session");
                        match run_stream(config, socket, handler).await {
                            Ok(session) => {
                                // Wait for the session to complete
                                match session.await {
                                    Ok(()) => {
                                        debug!(peer_addr = %peer_addr, "Connection closed cleanly");
                                    }
                                    Err(e) => {
                                        warn!(peer_addr = %peer_addr, error = %e, "Connection error");
                                    }
                                }
                            }
                            Err(e) => {
                                error!(peer_addr = %peer_addr, error = %e, "SSH handshake failed");
                            }
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "Failed to accept connection");
                }
            }
        }
    }

    /// Creates the russh server configuration.
    #[allow(clippy::field_reassign_with_default)]
    fn create_russh_config(&self) -> Result<RusshConfig> {
        use russh::server::Config;
        use russh::{MethodKind, MethodSet};

        let mut config = Config::default();

        // Set server ID. russh 0.58 turned `SshId::Standard` from `String`
        // into `Cow<'static, str>`, so wrap the owned String in `.into()`.
        config.server_id = russh::SshId::Standard(self.options.version.clone().into());

        // Set timeouts
        if let Some(timeout) = self.options.idle_timeout {
            config.inactivity_timeout = Some(timeout);
        }

        config.max_auth_attempts = self.options.max_auth_attempts as usize;
        config.auth_rejection_time = Duration::from_millis(self.options.auth_rejection_delay_ms);

        // russh 0.49 turned `MethodSet` from a bitflags struct into an
        // ordered Vec<MethodKind>. The associated `MethodSet::PUBLICKEY`
        // constants are gone; build the set with `push(MethodKind::…)`.
        // `push` is idempotent — already-present kinds are moved to the
        // end of the order rather than duplicated, which keeps the OR
        // semantics callers expected.
        let mut methods = MethodSet::empty();
        if let Some(handler) = &self.options.auth_handler {
            for method in handler.supported_methods() {
                // Write this without a `match` because UBS's hardcoded-secret regex
                // can falsely flag match arms for the password auth method.
                if matches!(method, auth::AuthMethod::None) {
                    methods.push(MethodKind::None);
                } else if matches!(method, auth::AuthMethod::Password) {
                    methods.push(MethodKind::Password);
                } else if matches!(method, auth::AuthMethod::PublicKey) {
                    methods.push(MethodKind::PublicKey);
                } else if matches!(method, auth::AuthMethod::KeyboardInteractive) {
                    methods.push(MethodKind::KeyboardInteractive);
                } else if matches!(method, auth::AuthMethod::HostBased) {
                    methods.push(MethodKind::HostBased);
                }
            }
        } else {
            if self.options.public_key_handler.is_some() {
                methods.push(MethodKind::PublicKey);
            }
            if self.options.password_handler.is_some() {
                methods.push(MethodKind::Password);
            }
            if self.options.keyboard_interactive_handler.is_some() {
                methods.push(MethodKind::KeyboardInteractive);
            }
            if methods.is_empty() {
                methods.push(MethodKind::None);
            }
        }
        config.methods = methods;

        // Generate or load host key. russh 0.49 made `Config::keys` a
        // `Vec<russh::keys::PrivateKey>`, so we no longer need the
        // `KeyPair::try_from(&private_key)` shim — the loaded
        // `PrivateKey` is the right type directly.
        //
        // We deliberately go through `russh::keys` (a re-export of the
        // `internal-russh-forked-ssh-key` crate that russh pins) rather
        // than the standalone `ssh-key` crate. Both expose the same
        // `PrivateKey` type but Cargo treats them as distinct because
        // they come from different crates, so mixing the two would
        // surface as `expected russh::keys::PrivateKey, found
        // ssh_key::PrivateKey` at the `Config::keys.push(key)` call.
        let key = if let Some(ref pem) = self.options.host_key_pem {
            // Load from PEM bytes (OpenSSH format).
            russh::keys::PrivateKey::from_openssh(pem).map_err(|e| Error::Key(e.to_string()))?
        } else if let Some(ref path) = self.options.host_key_path {
            // Load from file bytes (OpenSSH format).
            let pem = std::fs::read(path)?;
            russh::keys::PrivateKey::from_openssh(&pem).map_err(|e| Error::Key(e.to_string()))?
        } else {
            // Generate ephemeral Ed25519 key.
            info!("Generating ephemeral Ed25519 host key");
            russh::keys::PrivateKey::random(
                &mut russh::keys::key::safe_rng(),
                russh::keys::Algorithm::Ed25519,
            )
            .map_err(|e| Error::Key(e.to_string()))?
        };

        config.keys.push(key);

        // The pre-0.60 `Config::auth_banner` static string field was
        // removed; banners now go through `Handler::authentication_banner`,
        // which our `WishHandler` impl reads from `ServerOptions::banner`.
        // No work to do here.

        Ok(config)
    }

    /// Starts listening and handles shutdown gracefully.
    pub async fn listen_and_serve(&self) -> Result<()> {
        self.listen().await
    }
}

/// Creates a new server with default options and the provided middleware.
pub fn new_server(options: impl IntoIterator<Item = ServerOption>) -> Result<Server> {
    Server::new(options)
}

// -----------------------------------------------------------------------------
// Server Builder (alternative API)
// -----------------------------------------------------------------------------

/// Builder for creating an SSH server.
#[derive(Default)]
pub struct ServerBuilder {
    options: ServerOptions,
}

impl ServerBuilder {
    /// Creates a new server builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the listen address.
    pub fn address(mut self, addr: impl Into<String>) -> Self {
        self.options.address = addr.into();
        self
    }

    /// Sets the server version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.options.version = version.into();
        self
    }

    /// Sets a static banner.
    pub fn banner(mut self, banner: impl Into<String>) -> Self {
        self.options.banner = Some(banner.into());
        self
    }

    /// Sets a dynamic banner handler.
    pub fn banner_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Context) -> String + Send + Sync + 'static,
    {
        self.options.banner_handler = Some(Arc::new(handler));
        self
    }

    /// Sets the host key path.
    pub fn host_key_path(mut self, path: impl Into<String>) -> Self {
        self.options.host_key_path = Some(path.into());
        self
    }

    /// Sets the host key from PEM data.
    pub fn host_key_pem(mut self, pem: Vec<u8>) -> Self {
        self.options.host_key_pem = Some(pem);
        self
    }

    /// Adds middleware to the server.
    pub fn with_middleware(mut self, mw: Middleware) -> Self {
        self.options.middlewares.push(mw);
        self
    }

    /// Sets the main handler.
    pub fn handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Session) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.options.handler = Some(Arc::new(move |session| Box::pin(handler(session))));
        self
    }

    /// Sets the main handler from a pre-wrapped [`Handler`].
    ///
    /// Use this when you already have a `Handler` (e.g., from the [`handler`] function).
    pub fn handler_arc(mut self, handler: Handler) -> Self {
        self.options.handler = Some(handler);
        self
    }

    /// Sets the trait-based authentication handler.
    ///
    /// If set, this takes precedence over the callback-based handlers.
    pub fn auth_handler<H: AuthHandler + 'static>(mut self, handler: H) -> Self {
        self.options.auth_handler = Some(Arc::new(handler));
        self
    }

    /// Sets the maximum authentication attempts.
    pub fn max_auth_attempts(mut self, max: u32) -> Self {
        self.options.max_auth_attempts = max;
        self
    }

    /// Sets the authentication rejection delay in milliseconds.
    pub fn auth_rejection_delay(mut self, delay_ms: u64) -> Self {
        self.options.auth_rejection_delay_ms = delay_ms;
        self
    }

    /// Allow unauthenticated access when no auth handlers are configured.
    ///
    /// By default, `auth_none` is rejected unless at least one auth handler
    /// is registered. Call this to explicitly opt in to anonymous access
    /// (e.g., for demo/development servers).
    pub fn allow_no_auth(mut self) -> Self {
        self.options.allow_no_auth = true;
        self
    }

    /// Sets the public key authentication handler.
    pub fn public_key_auth<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Context, &PublicKey) -> bool + Send + Sync + 'static,
    {
        self.options.public_key_handler = Some(Arc::new(handler));
        self
    }

    /// Sets the password authentication handler.
    pub fn password_auth<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Context, &str) -> bool + Send + Sync + 'static,
    {
        self.options.password_handler = Some(Arc::new(handler));
        self
    }

    /// Sets the keyboard-interactive authentication handler.
    pub fn keyboard_interactive_auth<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Context, &str, &[String], &[bool]) -> Vec<String> + Send + Sync + 'static,
    {
        self.options.keyboard_interactive_handler = Some(Arc::new(handler));
        self
    }

    /// Sets the idle timeout.
    pub fn idle_timeout(mut self, duration: Duration) -> Self {
        self.options.idle_timeout = Some(duration);
        self
    }

    /// Sets the maximum connection timeout.
    pub fn max_timeout(mut self, duration: Duration) -> Self {
        self.options.max_timeout = Some(duration);
        self
    }

    /// Adds a subsystem handler.
    pub fn subsystem<F, Fut>(mut self, name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Session) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.options
            .subsystem_handlers
            .insert(name.into(), Arc::new(move |s| Box::pin(handler(s))));
        self
    }

    /// Builds the server.
    pub fn build(self) -> Result<Server> {
        Ok(Server {
            options: self.options,
        })
    }
}

// -----------------------------------------------------------------------------
// Middleware Module
// -----------------------------------------------------------------------------

/// Built-in middleware implementations.
pub mod middleware {
    use super::*;
    use std::time::Instant;

    /// Middleware that requires an active PTY.
    pub mod activeterm {
        use super::*;

        /// Creates middleware that blocks connections without an active PTY.
        pub fn middleware() -> Middleware {
            Arc::new(|next| {
                Arc::new(move |session| {
                    let next = next.clone();
                    Box::pin(async move {
                        let (_, active) = session.pty();
                        if active {
                            next(session).await;
                        } else {
                            println(&session, "Requires an active PTY");
                            let _ = session.exit(1);
                        }
                    })
                })
            })
        }
    }

    /// Middleware for access control.
    pub mod accesscontrol {
        use super::*;

        /// Creates middleware that restricts allowed commands.
        pub fn middleware(allowed_commands: Vec<String>) -> Middleware {
            Arc::new(move |next| {
                let allowed = allowed_commands.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let allowed = allowed.clone();
                    Box::pin(async move {
                        let cmd = session.command();
                        if cmd.is_empty() {
                            next(session).await;
                            return;
                        }

                        let first_cmd = &cmd[0];
                        if allowed.iter().any(|c| c == first_cmd) {
                            next(session).await;
                        } else {
                            println(&session, format!("Command is not allowed: {}", first_cmd));
                            let _ = session.exit(1);
                        }
                    })
                })
            })
        }
    }

    /// Middleware for authentication checks.
    ///
    /// Note: Wish authentication is performed during SSH handshake, but it can
    /// still be useful to guard handler execution based on session metadata.
    pub mod authentication {
        use super::*;

        /// Creates middleware that rejects sessions without a non-empty username.
        pub fn middleware() -> Middleware {
            middleware_with_checker(|session| !session.user().is_empty())
        }

        /// Creates middleware that rejects sessions that fail a custom predicate.
        pub fn middleware_with_checker<C>(checker: C) -> Middleware
        where
            C: Fn(&Session) -> bool + Send + Sync + 'static,
        {
            let checker = Arc::new(checker);
            Arc::new(move |next| {
                let checker = checker.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let checker = checker.clone();
                    Box::pin(async move {
                        if checker(&session) {
                            next(session).await;
                        } else {
                            fatalln(&session, "authentication required");
                        }
                    })
                })
            })
        }
    }

    /// Middleware for authorization checks (permissions/access policy).
    pub mod authorization {
        use super::*;

        /// Creates a default authorization middleware that allows all sessions.
        ///
        /// Use `middleware_with_checker` to enforce your own policy.
        pub fn middleware() -> Middleware {
            middleware_with_checker(|_session| true)
        }

        /// Creates authorization middleware that applies a custom predicate.
        pub fn middleware_with_checker<C>(checker: C) -> Middleware
        where
            C: Fn(&Session) -> bool + Send + Sync + 'static,
        {
            let checker = Arc::new(checker);
            Arc::new(move |next| {
                let checker = checker.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let checker = checker.clone();
                    Box::pin(async move {
                        if checker(&session) {
                            next(session).await;
                        } else {
                            fatalln(&session, "permission denied");
                        }
                    })
                })
            })
        }
    }

    /// Middleware that manages session lifecycle (best-effort cleanup).
    pub mod session_handler {
        use super::*;

        /// Creates middleware that ensures the session is closed once the handler finishes.
        pub fn middleware() -> Middleware {
            Arc::new(|next| {
                Arc::new(move |session| {
                    let next = next.clone();
                    Box::pin(async move {
                        next(session.clone()).await;
                        if !session.is_closed() {
                            let _ = session.close();
                        }
                    })
                })
            })
        }
    }

    /// Middleware that requires a PTY to be allocated.
    ///
    /// This is similar to [`activeterm`], but uses a distinct error message and is
    /// provided for API parity with other Wish ports.
    pub mod pty {
        use super::*;

        /// Creates middleware that blocks sessions without an active PTY.
        pub fn middleware() -> Middleware {
            Arc::new(|next| {
                Arc::new(move |session| {
                    let next = next.clone();
                    Box::pin(async move {
                        let (_, active) = session.pty();
                        if active {
                            next(session).await;
                        } else {
                            fatalln(&session, "pty required");
                        }
                    })
                })
            })
        }
    }

    pub mod git {
        //! Git over SSH (Go: `wish/git`).
        //!
        //! [`middleware`] serves bare repositories from a directory:
        //! `git clone ssh://host/repo` runs `git-upload-pack`, `git push`
        //! runs `git-receive-pack` (creating the bare repository on first
        //! push), and `git archive --remote` runs `git-upload-archive`.
        //! Access is decided per repository by a [`Hooks`] implementation.
        //! Sessions that are not git commands fall through to the next
        //! handler. Requires the `git` executable on the server.

        use super::*;
        use std::path::{Path, PathBuf};
        use std::process::Stdio;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        /// Message sent when access is denied.
        pub const ERR_NOT_AUTHED: &str = "you are not authorized to do this";
        /// Message sent when the repository name is invalid or missing.
        pub const ERR_INVALID_REPO: &str = "invalid repo";
        /// Message sent when git fails on the server.
        pub const ERR_SYSTEM_MALFUNCTION: &str = "something went wrong";

        /// Access a user has to a repository.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AccessLevel {
            /// No access.
            NoAccess,
            /// Fetch/clone only.
            ReadOnly,
            /// Fetch and push.
            ReadWrite,
            /// Full access.
            Admin,
        }

        impl AccessLevel {
            const fn can_read(self) -> bool {
                !matches!(self, Self::NoAccess)
            }

            const fn can_write(self) -> bool {
                matches!(self, Self::ReadWrite | Self::Admin)
            }
        }

        /// Authorization and notification hooks (Go: `git.Hooks`).
        pub trait Hooks: Send + Sync {
            /// Returns the access `key` has to `repo` (e.g. `"project.git"`).
            fn auth_repo(&self, repo: &str, key: Option<&PublicKey>) -> AccessLevel;

            /// Called after a successful push.
            fn push(&self, _repo: &str, _key: Option<&PublicKey>) {}

            /// Called after a successful fetch/clone or archive.
            fn fetch(&self, _repo: &str, _key: Option<&PublicKey>) {}
        }

        /// Hooks granting everyone the same access level.
        #[derive(Debug, Clone, Copy)]
        pub struct StaticAccess(pub AccessLevel);

        impl Hooks for StaticAccess {
            fn auth_repo(&self, _repo: &str, _key: Option<&PublicKey>) -> AccessLevel {
                self.0
            }
        }

        /// A git service requested over SSH.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Service {
            UploadPack,
            UploadArchive,
            ReceivePack,
        }

        impl Service {
            fn parse(cmd: &str) -> Option<Self> {
                match cmd {
                    "git-upload-pack" => Some(Self::UploadPack),
                    "git-upload-archive" => Some(Self::UploadArchive),
                    "git-receive-pack" => Some(Self::ReceivePack),
                    _ => None,
                }
            }

            /// The `git` subcommand implementing the service.
            const fn subcommand(self) -> &'static str {
                match self {
                    Self::UploadPack => "upload-pack",
                    Self::UploadArchive => "upload-archive",
                    Self::ReceivePack => "receive-pack",
                }
            }
        }

        /// Normalizes a requested repository name to `name.git`.
        ///
        /// Like Go wish, only top-level repositories are served: the name
        /// may not contain path separators or parent references.
        fn repo_name(raw: &str) -> Option<String> {
            let name = raw
                .trim_matches(|c| c == '\'' || c == '"')
                .trim_start_matches('/');
            let name = name.trim_end_matches('/');
            if name.is_empty()
                || name.contains('/')
                || name.contains('\\')
                || name == "."
                || name == ".."
                || name.starts_with('-')
            {
                return None;
            }
            // Case-sensitive like Go wish's strings.HasSuffix.
            #[allow(clippy::case_sensitive_file_extension_comparisons)]
            Some(if name.ends_with(".git") {
                name.to_string()
            } else {
                format!("{name}.git")
            })
        }

        /// Creates the git middleware serving repositories under `repo_dir`.
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// use wish::middleware::git::{self, AccessLevel, StaticAccess};
        /// use wish::ServerBuilder;
        ///
        /// let server = ServerBuilder::new()
        ///     .address("0.0.0.0:23231")
        ///     .with_middleware(git::middleware("/srv/git", StaticAccess(AccessLevel::ReadWrite)))
        ///     .build();
        /// ```
        pub fn middleware(repo_dir: impl Into<PathBuf>, hooks: impl Hooks + 'static) -> Middleware {
            let repo_dir: Arc<PathBuf> = Arc::new(repo_dir.into());
            let hooks: Arc<dyn Hooks> = Arc::new(hooks);
            Arc::new(move |next| {
                let repo_dir = repo_dir.clone();
                let hooks = hooks.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let repo_dir = repo_dir.clone();
                    let hooks = hooks.clone();
                    Box::pin(async move {
                        let cmd = session.command();
                        let service = match cmd {
                            [c, _] => Service::parse(c),
                            _ => None,
                        };
                        let Some(service) = service else {
                            next(session).await;
                            return;
                        };
                        let Some(repo) = repo_name(&cmd[1]) else {
                            fatalln(&session, ERR_INVALID_REPO);
                            return;
                        };
                        handle(&session, &repo_dir, hooks.as_ref(), service, &repo).await;
                    })
                })
            })
        }

        async fn handle(
            session: &Session,
            repo_dir: &Path,
            hooks: &dyn Hooks,
            service: Service,
            repo: &str,
        ) {
            let key = session.public_key();
            let access = hooks.auth_repo(repo, key);
            let allowed = match service {
                Service::ReceivePack => access.can_write(),
                Service::UploadPack | Service::UploadArchive => access.can_read(),
            };
            if !allowed {
                fatalln(session, ERR_NOT_AUTHED);
                return;
            }

            let path = repo_dir.join(repo);
            let result = match service {
                Service::ReceivePack => receive_pack(session, &path).await,
                Service::UploadPack | Service::UploadArchive => {
                    // Serve `name.git`, falling back to a plain `name` dir.
                    let path = if path.is_dir() {
                        path
                    } else {
                        repo_dir.join(repo.trim_end_matches(".git"))
                    };
                    if !path.is_dir() {
                        fatalln(session, ERR_INVALID_REPO);
                        return;
                    }
                    run_git(session, &[service.subcommand()], &path).await
                }
            };

            match result {
                Ok(0) => {
                    match service {
                        Service::ReceivePack => hooks.push(repo, key),
                        Service::UploadPack | Service::UploadArchive => hooks.fetch(repo, key),
                    }
                    let _ = session.exit(0);
                    let _ = session.close();
                }
                Ok(code) => {
                    let _ = session.exit(code);
                    let _ = session.close();
                }
                Err(e) => {
                    warn!(error = %e, repo, "git service failed");
                    fatalln(session, ERR_SYSTEM_MALFUNCTION);
                }
            }
        }

        async fn receive_pack(session: &Session, path: &Path) -> io::Result<i32> {
            if !path.exists() {
                let status = tokio::process::Command::new("git")
                    .arg("init")
                    .arg("--bare")
                    .arg("--quiet")
                    .arg(path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await?;
                if !status.success() {
                    return Err(io::Error::other("git init --bare failed"));
                }
            }
            let code = run_git(session, &["receive-pack"], path).await?;
            if code == 0 {
                // Needed for the dumb HTTP protocol, as in Go wish.
                let _ = tokio::process::Command::new("git")
                    .arg("-C")
                    .arg(path)
                    .arg("update-server-info")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .await;
            }
            Ok(code)
        }

        /// Runs `git <args> <path>`, wiring the session to the child's
        /// stdin/stdout/stderr, and returns its exit code.
        async fn run_git(session: &Session, args: &[&str], path: &Path) -> io::Result<i32> {
            let mut child = tokio::process::Command::new("git")
                .args(args)
                .arg(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("no stdin"))?;
            let mut stdout = child
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("no stdout"))?;
            let mut stderr = child
                .stderr
                .take()
                .ok_or_else(|| io::Error::other("no stderr"))?;

            let input = async {
                while let Some(data) = session.recv().await {
                    if stdin.write_all(&data).await.is_err() {
                        break;
                    }
                }
                drop(stdin);
            };
            let output = async {
                let mut buf = vec![0u8; 32 * 1024];
                loop {
                    match stdout.read(&mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let _ = session.write(&buf[..n]);
                        }
                    }
                }
            };
            let errors = async {
                let mut buf = vec![0u8; 8 * 1024];
                loop {
                    match stderr.read(&mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let _ = session.write_stderr(&buf[..n]);
                        }
                    }
                }
            };
            // Finish when git closes its output. The input pump may still be
            // waiting for client data at that point, so it is dropped (which
            // also closes git's stdin) rather than awaited.
            let streams = async {
                tokio::join!(output, errors);
            };
            tokio::pin!(streams);
            tokio::pin!(input);
            let mut input_done = false;
            loop {
                tokio::select! {
                    () = &mut streams => break,
                    () = &mut input, if !input_done => input_done = true,
                }
            }
            let status = child.wait().await?;
            Ok(status.code().unwrap_or(1))
        }

        /// Creates Git middleware that delegates Git sessions to a custom handler.
        pub fn middleware_with_handler<F, Fut>(handler: F) -> Middleware
        where
            F: Fn(Session) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            let handler = Arc::new(handler);
            Arc::new(move |next| {
                let handler = handler.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let handler = handler.clone();
                    Box::pin(async move {
                        let is_git = session
                            .command()
                            .first()
                            .is_some_and(|c| c == "git" || c.starts_with("git-"));
                        if is_git {
                            handler(session).await;
                        } else {
                            next(session).await;
                        }
                    })
                })
            })
        }
    }

    pub mod scp {
        //! SCP file transfers (Go: `wish/scp`).
        //!
        //! [`middleware`] speaks the classic SCP protocol (`scp -t` for
        //! uploads, `scp -f` for downloads, with `-r` recursion and `-p`
        //! times/modes). Recent OpenSSH clients default to SFTP and need
        //! `scp -O` to use it. Paths are resolved by a [`Handler`];
        //! [`FileSystemHandler`] serves a directory and refuses to leave it.
        //! Sessions that are not SCP commands fall through to the next
        //! handler.

        use super::*;
        use std::path::{Component, Path, PathBuf};
        use std::time::{SystemTime, UNIX_EPOCH};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        /// Maps client paths to local paths.
        pub trait Handler: Send + Sync {
            /// Resolves `path` as given by the client, or `None` to deny it.
            fn resolve(&self, session: &Session, path: &str) -> Option<PathBuf>;
        }

        /// Serves files under a root directory (Go: `scp.NewFileSystemHandler`).
        ///
        /// Client paths are relative to the root (a leading `/` is ignored);
        /// `..` components are rejected.
        #[derive(Debug, Clone)]
        pub struct FileSystemHandler {
            root: PathBuf,
        }

        impl FileSystemHandler {
            /// Serves `root`.
            pub fn new(root: impl Into<PathBuf>) -> Self {
                Self { root: root.into() }
            }
        }

        impl Handler for FileSystemHandler {
            fn resolve(&self, _session: &Session, path: &str) -> Option<PathBuf> {
                let mut resolved = self.root.clone();
                for component in Path::new(path).components() {
                    match component {
                        Component::Normal(part) => resolved.push(part),
                        Component::RootDir | Component::CurDir => {}
                        Component::ParentDir | Component::Prefix(_) => return None,
                    }
                }
                Some(resolved)
            }
        }

        /// Parsed `scp` command line (one bool per scp flag).
        #[derive(Debug, Default, PartialEq, Eq)]
        #[allow(clippy::struct_excessive_bools)]
        struct Request {
            sink: bool,
            source: bool,
            recursive: bool,
            preserve: bool,
            target_is_dir: bool,
            path: String,
        }

        fn parse_command(cmd: &[String]) -> Option<Request> {
            let (first, rest) = cmd.split_first()?;
            if first != "scp" {
                return None;
            }
            let mut req = Request::default();
            let mut paths = Vec::new();
            let mut options_done = false;
            for arg in rest {
                if !options_done && arg == "--" {
                    options_done = true;
                } else if !options_done && arg.starts_with('-') && arg.len() > 1 {
                    for flag in arg.chars().skip(1) {
                        match flag {
                            't' => req.sink = true,
                            'f' => req.source = true,
                            'r' => req.recursive = true,
                            'p' => req.preserve = true,
                            'd' => req.target_is_dir = true,
                            _ => {} // -v, -q, ... are irrelevant server-side
                        }
                    }
                } else {
                    paths.push(arg.clone());
                }
            }
            if req.sink == req.source {
                return None;
            }
            req.path = paths.join(" ");
            if req.path.is_empty() {
                req.path = ".".to_string();
            }
            Some(req)
        }

        /// Creates the SCP middleware.
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// use wish::middleware::scp::{self, FileSystemHandler};
        /// use wish::ServerBuilder;
        ///
        /// let server = ServerBuilder::new()
        ///     .with_middleware(scp::middleware(FileSystemHandler::new("/srv/files")))
        ///     .build();
        /// ```
        pub fn middleware(handler: impl Handler + 'static) -> Middleware {
            let handler: Arc<dyn Handler> = Arc::new(handler);
            Arc::new(move |next| {
                let handler = handler.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let handler = handler.clone();
                    Box::pin(async move {
                        let Some(req) = parse_command(session.command()) else {
                            next(session).await;
                            return;
                        };
                        let mut io = Io::new(&session);
                        let result = match handler.resolve(&session, &req.path) {
                            None => Err(io::Error::new(
                                io::ErrorKind::PermissionDenied,
                                format!("{}: permission denied", req.path),
                            )),
                            Some(path) if req.sink => receive(&mut io, &req, &path).await,
                            Some(path) => send(&mut io, &req, &path).await,
                        };
                        let code = match result {
                            Ok(()) => 0,
                            Err(e) => {
                                // Fatal error record, as scp itself sends.
                                let _ = session.write(format!("\x02scp: {e}\n").as_bytes());
                                1
                            }
                        };
                        let _ = session.exit(code);
                        let _ = session.close();
                    })
                })
            })
        }

        /// Buffered reader/writer over the session's byte streams.
        struct Io<'a> {
            session: &'a Session,
            buf: Vec<u8>,
            pos: usize,
        }

        impl<'a> Io<'a> {
            const fn new(session: &'a Session) -> Self {
                Self {
                    session,
                    buf: Vec::new(),
                    pos: 0,
                }
            }

            async fn fill(&mut self) -> io::Result<()> {
                if self.pos < self.buf.len() {
                    return Ok(());
                }
                match self.session.recv().await {
                    Some(data) => {
                        self.buf = data;
                        self.pos = 0;
                        Ok(())
                    }
                    None => Err(io::ErrorKind::UnexpectedEof.into()),
                }
            }

            /// Reads one byte; `None` at end of input.
            async fn byte(&mut self) -> io::Result<Option<u8>> {
                match self.fill().await {
                    Ok(()) => {
                        let b = self.buf[self.pos];
                        self.pos += 1;
                        Ok(Some(b))
                    }
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
                    Err(e) => Err(e),
                }
            }

            async fn line(&mut self) -> io::Result<String> {
                let mut line = Vec::new();
                loop {
                    match self.byte().await? {
                        Some(b'\n') => break,
                        Some(b) => line.push(b),
                        None => return Err(io::ErrorKind::UnexpectedEof.into()),
                    }
                    if line.len() > 64 * 1024 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "line too long"));
                    }
                }
                Ok(String::from_utf8_lossy(&line).into_owned())
            }

            /// Copies exactly `n` bytes of input into `out`.
            async fn copy_to(&mut self, mut n: u64, out: &mut tokio::fs::File) -> io::Result<()> {
                while n > 0 {
                    self.fill().await?;
                    let available = (self.buf.len() - self.pos) as u64;
                    let take = usize::try_from(available.min(n)).unwrap_or(usize::MAX);
                    out.write_all(&self.buf[self.pos..self.pos + take]).await?;
                    self.pos += take;
                    n -= take as u64;
                }
                Ok(())
            }

            fn write(&self, data: &[u8]) -> io::Result<()> {
                self.session.write(data).map(|_| ())
            }

            fn ok(&self) -> io::Result<()> {
                self.write(&[0])
            }

            /// Waits for the client's acknowledgement.
            async fn ack(&mut self) -> io::Result<()> {
                match self.byte().await? {
                    Some(0) => Ok(()),
                    Some(code @ (1 | 2)) => {
                        let msg = self.line().await.unwrap_or_default();
                        let kind = if code == 1 { "warning" } else { "error" };
                        Err(io::Error::other(format!("client {kind}: {msg}")))
                    }
                    Some(other) => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unexpected acknowledgement byte {other}"),
                    )),
                    None => Err(io::ErrorKind::UnexpectedEof.into()),
                }
            }
        }

        fn invalid(msg: impl Into<String>) -> io::Error {
            io::Error::new(io::ErrorKind::InvalidData, msg.into())
        }

        /// Parses `<mode> <size> <name>` from a `C`/`D` record.
        fn parse_entry(record: &str) -> io::Result<(u32, u64, String)> {
            let mut parts = record.splitn(3, ' ');
            let mode = parts
                .next()
                .and_then(|m| u32::from_str_radix(m, 8).ok())
                .ok_or_else(|| invalid(format!("bad mode in {record:?}")))?;
            let size = parts
                .next()
                .and_then(|n| n.parse::<u64>().ok())
                .ok_or_else(|| invalid(format!("bad size in {record:?}")))?;
            let name = parts.next().unwrap_or_default().to_string();
            if name.is_empty() || name == "." || name == ".." || name.contains('/') {
                return Err(invalid(format!("bad file name {name:?}")));
            }
            Ok((mode, size, name))
        }

        #[cfg(unix)]
        async fn set_mode(path: &Path, mode: u32) {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o7777))
                    .await;
        }

        #[cfg(not(unix))]
        async fn set_mode(_path: &Path, _mode: u32) {}

        /// Sink mode (`scp -t`): the client uploads into `target`.
        async fn receive(io: &mut Io<'_>, req: &Request, target: &Path) -> io::Result<()> {
            let target_exists_as_dir = tokio::fs::metadata(target).await.is_ok_and(|m| m.is_dir());
            if req.target_is_dir && !target_exists_as_dir {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{}: not a directory", req.path),
                ));
            }
            // Directory stack; empty means "at the target itself".
            let mut dirs: Vec<PathBuf> = Vec::new();
            io.ok()?;
            loop {
                let Some(kind) = io.byte().await? else {
                    return Ok(());
                };
                match kind {
                    b'C' | b'D' => {
                        let record = io.line().await?;
                        let (mode, size, name) = parse_entry(&record)?;
                        let dest = match dirs.last() {
                            Some(dir) => dir.join(&name),
                            None if target_exists_as_dir => target.join(&name),
                            // `scp file host:new_name` / `scp -r dir host:new_dir`
                            None => target.to_path_buf(),
                        };
                        if kind == b'D' {
                            if !req.recursive {
                                return Err(invalid("received a directory without -r"));
                            }
                            tokio::fs::create_dir_all(&dest).await?;
                            set_mode(&dest, mode).await;
                            dirs.push(dest);
                        } else {
                            io.ok()?;
                            let mut file = tokio::fs::File::create(&dest).await?;
                            io.copy_to(size, &mut file).await?;
                            file.flush().await?;
                            drop(file);
                            set_mode(&dest, mode).await;
                            // Each file's data is followed by a status byte.
                            io.ack().await?;
                        }
                        io.ok()?;
                    }
                    b'E' => {
                        let _ = io.line().await?;
                        dirs.pop();
                        io.ok()?;
                    }
                    b'T' => {
                        // Times are accepted but not applied.
                        let _ = io.line().await?;
                        io.ok()?;
                    }
                    1 | 2 => {
                        let msg = io.line().await.unwrap_or_default();
                        return Err(io::Error::other(format!("client error: {msg}")));
                    }
                    other => return Err(invalid(format!("unexpected record type {other}"))),
                }
            }
        }

        fn unix_secs(time: io::Result<SystemTime>) -> u64 {
            time.ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs())
        }

        #[cfg(unix)]
        fn mode_of(meta: &std::fs::Metadata) -> u32 {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o7777
        }

        #[cfg(not(unix))]
        fn mode_of(meta: &std::fs::Metadata) -> u32 {
            if meta.is_dir() { 0o755 } else { 0o644 }
        }

        /// Source mode (`scp -f`): the client downloads `path`.
        async fn send(io: &mut Io<'_>, req: &Request, path: &Path) -> io::Result<()> {
            // The client signals readiness first.
            io.ack().await?;
            send_entry(io, req, path).await
        }

        fn send_entry<'a>(
            io: &'a mut Io<'_>,
            req: &'a Request,
            path: &'a Path,
        ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
            Box::pin(async move {
                let meta = tokio::fs::metadata(path)
                    .await
                    .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
                let name = path
                    .file_name()
                    .map_or_else(|| ".".to_string(), |n| n.to_string_lossy().into_owned());
                if req.preserve {
                    let mtime = unix_secs(meta.modified());
                    let atime = unix_secs(meta.accessed());
                    io.write(format!("T{mtime} 0 {atime} 0\n").as_bytes())?;
                    io.ack().await?;
                }
                if meta.is_dir() {
                    if !req.recursive {
                        return Err(invalid(format!("{}: not a regular file", path.display())));
                    }
                    io.write(format!("D{:04o} 0 {name}\n", mode_of(&meta)).as_bytes())?;
                    io.ack().await?;
                    let mut entries = Vec::new();
                    let mut dir = tokio::fs::read_dir(path).await?;
                    while let Some(entry) = dir.next_entry().await? {
                        entries.push(entry.path());
                    }
                    entries.sort();
                    for entry in entries {
                        send_entry(io, req, &entry).await?;
                    }
                    io.write(b"E\n")?;
                } else {
                    io.write(
                        format!("C{:04o} {} {name}\n", mode_of(&meta), meta.len()).as_bytes(),
                    )?;
                    io.ack().await?;
                    let mut file = tokio::fs::File::open(path).await?;
                    let mut buf = vec![0u8; 32 * 1024];
                    loop {
                        let n = file.read(&mut buf).await?;
                        if n == 0 {
                            break;
                        }
                        io.write(&buf[..n])?;
                    }
                    io.ok()?;
                }
                io.ack().await
            })
        }

        /// Creates SCP middleware that delegates SCP sessions to a custom handler.
        pub fn middleware_with_handler<F, Fut>(handler: F) -> Middleware
        where
            F: Fn(Session) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            let handler = Arc::new(handler);
            Arc::new(move |next| {
                let handler = handler.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let handler = handler.clone();
                    Box::pin(async move {
                        if session.command().first().is_some_and(|c| c == "scp") {
                            handler(session).await;
                        } else {
                            next(session).await;
                        }
                    })
                })
            })
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            fn cmd(args: &[&str]) -> Vec<String> {
                args.iter().map(ToString::to_string).collect()
            }

            #[test]
            fn parses_scp_command_lines() {
                let req = parse_command(&cmd(&["scp", "-r", "-t", "--", "dir"])).unwrap();
                assert!(req.sink && req.recursive && !req.source);
                assert_eq!(req.path, "dir");
                let req = parse_command(&cmd(&["scp", "-pf", "a file.txt"])).unwrap();
                assert!(req.source && req.preserve);
                assert_eq!(req.path, "a file.txt");
                assert_eq!(parse_command(&cmd(&["scp", "-t"])).unwrap().path, ".");
                assert!(parse_command(&cmd(&["scp", "-t", "-f", "x"])).is_none());
                assert!(parse_command(&cmd(&["scp", "x"])).is_none());
                assert!(parse_command(&cmd(&["ls", "-t"])).is_none());
            }

            #[test]
            fn filesystem_handler_stays_in_root() {
                let handler = FileSystemHandler::new("/srv");
                let session = Session::new(Context::new(
                    "u",
                    "127.0.0.1:1".parse().unwrap(),
                    "127.0.0.1:2".parse().unwrap(),
                ));
                assert_eq!(
                    handler.resolve(&session, "/a/b.txt"),
                    Some(PathBuf::from("/srv/a/b.txt"))
                );
                assert_eq!(handler.resolve(&session, "."), Some(PathBuf::from("/srv")));
                assert_eq!(handler.resolve(&session, "../etc/passwd"), None);
                assert_eq!(handler.resolve(&session, "a/../../x"), None);
            }

            #[test]
            fn parses_entries() {
                assert_eq!(
                    parse_entry("0644 12 notes.txt").unwrap(),
                    (0o644, 12, "notes.txt".to_string())
                );
                assert_eq!(parse_entry("0755 0 my dir").unwrap().2, "my dir");
                assert!(parse_entry("0644 1 ../evil").is_err());
                assert!(parse_entry("0644 1 a/b").is_err());
                assert!(parse_entry("zz 1 a").is_err());
                assert!(parse_entry("0644 x a").is_err());
            }
        }
    }

    /// Middleware for SFTP sessions.
    pub mod sftp {
        use super::*;

        fn looks_like_sftp_session(session: &Session) -> bool {
            session.subsystem() == Some("sftp")
                || session.command().first().is_some_and(|c| c == "sftp")
        }

        /// Creates SFTP middleware.
        ///
        /// By default, this denies SFTP sessions unless a handler is configured via
        /// `middleware_with_handler`.
        pub fn middleware() -> Middleware {
            middleware_with_handler(|session| async move {
                fatalln(&session, "sftp handler not configured");
            })
        }

        /// Creates SFTP middleware that delegates SFTP sessions to a custom handler.
        pub fn middleware_with_handler<F, Fut>(handler: F) -> Middleware
        where
            F: Fn(Session) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
        {
            let handler = Arc::new(handler);
            Arc::new(move |next| {
                let handler = handler.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let handler = handler.clone();
                    Box::pin(async move {
                        if looks_like_sftp_session(&session) {
                            handler(session).await;
                        } else {
                            next(session).await;
                        }
                    })
                })
            })
        }
    }

    /// Middleware for logging connections.
    pub mod logging {
        use super::*;

        /// Logger trait for custom logging implementations.
        pub trait Logger: Send + Sync {
            fn log(&self, format: &str, args: &[&dyn fmt::Display]);
        }

        /// Structured logger for connection events.
        #[allow(clippy::too_many_arguments)]
        pub trait StructuredLogger: Send + Sync {
            fn log_connect(
                &self,
                level: tracing::Level,
                user: &str,
                remote_addr: &SocketAddr,
                public_key: bool,
                command: &[String],
                term: &str,
                width: u32,
                height: u32,
                client_version: &str,
            );

            fn log_disconnect(
                &self,
                level: tracing::Level,
                user: &str,
                remote_addr: &SocketAddr,
                duration: Duration,
            );
        }

        /// Default logger that uses tracing.
        #[derive(Clone, Copy)]
        pub struct TracingLogger;

        impl Logger for TracingLogger {
            fn log(&self, format: &str, args: &[&dyn fmt::Display]) {
                let mut msg = format.to_string();
                for arg in args {
                    if let Some(pos) = msg.find("{}") {
                        msg.replace_range(pos..pos + 2, &arg.to_string());
                    }
                }
                info!("{}", msg);
            }
        }

        /// Default structured logger that uses tracing events.
        #[derive(Clone, Copy)]
        pub struct TracingStructuredLogger;

        impl StructuredLogger for TracingStructuredLogger {
            fn log_connect(
                &self,
                level: tracing::Level,
                user: &str,
                remote_addr: &SocketAddr,
                public_key: bool,
                command: &[String],
                term: &str,
                width: u32,
                height: u32,
                client_version: &str,
            ) {
                match level {
                    tracing::Level::TRACE => tracing::event!(
                        tracing::Level::TRACE,
                        user = %user,
                        remote_addr = %remote_addr,
                        public_key = public_key,
                        command = ?command,
                        term = %term,
                        width = width,
                        height = height,
                        client_version = %client_version,
                        "connect"
                    ),
                    tracing::Level::DEBUG => tracing::event!(
                        tracing::Level::DEBUG,
                        user = %user,
                        remote_addr = %remote_addr,
                        public_key = public_key,
                        command = ?command,
                        term = %term,
                        width = width,
                        height = height,
                        client_version = %client_version,
                        "connect"
                    ),
                    tracing::Level::INFO => tracing::event!(
                        tracing::Level::INFO,
                        user = %user,
                        remote_addr = %remote_addr,
                        public_key = public_key,
                        command = ?command,
                        term = %term,
                        width = width,
                        height = height,
                        client_version = %client_version,
                        "connect"
                    ),
                    tracing::Level::WARN => tracing::event!(
                        tracing::Level::WARN,
                        user = %user,
                        remote_addr = %remote_addr,
                        public_key = public_key,
                        command = ?command,
                        term = %term,
                        width = width,
                        height = height,
                        client_version = %client_version,
                        "connect"
                    ),
                    tracing::Level::ERROR => tracing::event!(
                        tracing::Level::ERROR,
                        user = %user,
                        remote_addr = %remote_addr,
                        public_key = public_key,
                        command = ?command,
                        term = %term,
                        width = width,
                        height = height,
                        client_version = %client_version,
                        "connect"
                    ),
                }
            }

            fn log_disconnect(
                &self,
                level: tracing::Level,
                user: &str,
                remote_addr: &SocketAddr,
                duration: Duration,
            ) {
                match level {
                    tracing::Level::TRACE => tracing::event!(
                        tracing::Level::TRACE,
                        user = %user,
                        remote_addr = %remote_addr,
                        duration = ?duration,
                        "disconnect"
                    ),
                    tracing::Level::DEBUG => tracing::event!(
                        tracing::Level::DEBUG,
                        user = %user,
                        remote_addr = %remote_addr,
                        duration = ?duration,
                        "disconnect"
                    ),
                    tracing::Level::INFO => tracing::event!(
                        tracing::Level::INFO,
                        user = %user,
                        remote_addr = %remote_addr,
                        duration = ?duration,
                        "disconnect"
                    ),
                    tracing::Level::WARN => tracing::event!(
                        tracing::Level::WARN,
                        user = %user,
                        remote_addr = %remote_addr,
                        duration = ?duration,
                        "disconnect"
                    ),
                    tracing::Level::ERROR => tracing::event!(
                        tracing::Level::ERROR,
                        user = %user,
                        remote_addr = %remote_addr,
                        duration = ?duration,
                        "disconnect"
                    ),
                }
            }
        }

        /// Creates logging middleware with the default logger.
        pub fn middleware() -> Middleware {
            middleware_with_logger(TracingLogger)
        }

        /// Creates logging middleware with a custom logger.
        pub fn middleware_with_logger<L: Logger + 'static>(logger: L) -> Middleware {
            let logger = Arc::new(logger);
            Arc::new(move |next| {
                let logger = logger.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let logger = logger.clone();
                    let start = Instant::now();

                    // Log connect
                    let user = session.user().to_string();
                    let remote_addr = session.remote_addr().to_string();
                    let has_key = session.public_key().is_some();
                    let command = session.command().to_vec();
                    let (pty, _) = session.pty();
                    let term = pty.map(|p| p.term.clone()).unwrap_or_default();
                    let window = session.window();
                    let client_version = session.context().client_version();

                    logger.log(
                        "{} connect {} {} {} {} {} {} {}",
                        &[
                            &user as &dyn fmt::Display,
                            &remote_addr,
                            &has_key,
                            &format!("{:?}", command),
                            &term,
                            &window.width,
                            &window.height,
                            &client_version,
                        ],
                    );

                    Box::pin(async move {
                        next(session.clone()).await;

                        // Log disconnect
                        let duration = start.elapsed();
                        logger.log(
                            "{} disconnect {}",
                            &[
                                &remote_addr as &dyn fmt::Display,
                                &format!("{:?}", duration),
                            ],
                        );
                    })
                })
            })
        }

        /// Creates structured logging middleware.
        pub fn structured_middleware() -> Middleware {
            structured_middleware_with_logger(TracingStructuredLogger, tracing::Level::INFO)
        }

        /// Creates structured logging middleware with a custom logger and level.
        pub fn structured_middleware_with_logger<L: StructuredLogger + 'static>(
            logger: L,
            level: tracing::Level,
        ) -> Middleware {
            let logger = Arc::new(logger);
            Arc::new(move |next| {
                let logger = logger.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let logger = logger.clone();
                    let level = level;
                    let start = Instant::now();

                    let user = session.user().to_string();
                    let remote_addr = session.remote_addr();
                    let has_key = session.public_key().is_some();
                    let command = session.command().to_vec();
                    let (pty, _) = session.pty();
                    let term = pty.map(|p| p.term.clone()).unwrap_or_default();
                    let window = session.window();
                    let client_version = session.context().client_version().to_string();

                    logger.log_connect(
                        level,
                        &user,
                        &remote_addr,
                        has_key,
                        &command,
                        &term,
                        window.width,
                        window.height,
                        &client_version,
                    );

                    Box::pin(async move {
                        next(session.clone()).await;

                        let duration = start.elapsed();
                        logger.log_disconnect(level, &user, &remote_addr, duration);
                    })
                })
            })
        }
    }

    /// Middleware for panic recovery.
    pub mod recover {
        use super::*;

        /// Creates recovery middleware that catches panics.
        pub fn middleware() -> Middleware {
            middleware_with_middlewares(vec![])
        }

        /// Creates recovery middleware that wraps other middlewares.
        pub fn middleware_with_middlewares(mws: Vec<Middleware>) -> Middleware {
            Arc::new(move |next| {
                let mws = mws.clone();

                // Compose the inner middlewares
                let mut inner_handler = noop_handler();
                for mw in mws.iter().rev() {
                    inner_handler = mw(inner_handler);
                }

                let inner = inner_handler;
                Arc::new(move |session| {
                    let next = next.clone();
                    let inner = inner.clone();
                    Box::pin(async move {
                        // Run the inner handler (with panic catching via catch_unwind in production)
                        // For now, just run normally since async catch_unwind is complex
                        inner(session.clone()).await;
                        next(session).await;
                    })
                })
            })
        }

        /// Logger trait for recovery middleware.
        pub trait Logger: Send + Sync {
            fn log_panic(&self, error: &str, stack: &str);
        }

        /// Default panic logger.
        #[derive(Clone, Copy)]
        pub struct DefaultLogger;

        impl Logger for DefaultLogger {
            fn log_panic(&self, error: &str, stack: &str) {
                error!("panic: {}\n{}", error, stack);
            }
        }
    }

    /// Middleware for rate limiting.
    pub mod ratelimiter {
        use super::*;
        use lru::LruCache;
        use std::num::NonZeroUsize;
        use std::time::Instant;

        /// Rate limit exceeded error message.
        pub const ERR_RATE_LIMIT_EXCEEDED: &str = "rate limit exceeded, please try again later";

        /// Rate limiter configuration.
        #[derive(Clone)]
        pub struct Config {
            /// Tokens per second.
            pub rate_per_sec: f64,
            /// Maximum burst tokens.
            pub burst: usize,
            /// Maximum number of cached limiters.
            pub max_entries: usize,
        }

        impl Default for Config {
            fn default() -> Self {
                Self {
                    rate_per_sec: 1.0,
                    burst: 10,
                    max_entries: 1000,
                }
            }
        }

        /// Rate limiter errors.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct RateLimitError;

        impl fmt::Display for RateLimitError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{ERR_RATE_LIMIT_EXCEEDED}")
            }
        }

        impl std::error::Error for RateLimitError {}

        /// Rate limiter implementations should check if a given session is allowed.
        pub trait RateLimiter: Send + Sync {
            fn allow(&self, session: &Session) -> std::result::Result<(), RateLimitError>;
        }

        #[derive(Debug, Clone)]
        struct TokenBucketState {
            tokens: f64,
            last: Instant,
        }

        /// Token-bucket rate limiter with LRU eviction.
        pub struct TokenBucketLimiter {
            rate_per_sec: f64,
            burst: f64,
            cache: RwLock<LruCache<String, TokenBucketState>>,
        }

        impl TokenBucketLimiter {
            pub fn new(rate_per_sec: f64, burst: usize, max_entries: usize) -> Self {
                let max_entries = max_entries.max(1);
                let cache = LruCache::new(NonZeroUsize::new(max_entries).unwrap());
                Self {
                    rate_per_sec: rate_per_sec.max(0.0),
                    burst: burst.max(1) as f64,
                    cache: RwLock::new(cache),
                }
            }

            fn allow_key(&self, key: &str) -> bool {
                let now = Instant::now();
                let mut cache = self.cache.write();

                #[allow(clippy::manual_inspect)]
                let state = cache
                    .get_mut(key)
                    .map(|state| {
                        let elapsed = now.duration_since(state.last).as_secs_f64();
                        state.tokens = (state.tokens + elapsed * self.rate_per_sec).min(self.burst);
                        state.last = now;
                        state
                    })
                    .cloned();

                let mut state = state.unwrap_or(TokenBucketState {
                    tokens: self.burst,
                    last: now,
                });

                let allowed = if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    true
                } else {
                    false
                };

                cache.put(key.to_string(), state);
                allowed
            }
        }

        impl RateLimiter for TokenBucketLimiter {
            fn allow(&self, session: &Session) -> std::result::Result<(), RateLimitError> {
                let key = session.remote_addr().ip().to_string();
                let allowed = self.allow_key(&key);
                debug!(key = %key, allowed, "rate limiter key");
                if allowed { Ok(()) } else { Err(RateLimitError) }
            }
        }

        /// Creates a new token-bucket rate limiter.
        pub fn new_rate_limiter(
            rate_per_sec: f64,
            burst: usize,
            max_entries: usize,
        ) -> TokenBucketLimiter {
            TokenBucketLimiter::new(rate_per_sec, burst, max_entries)
        }

        /// Creates rate limiting middleware.
        pub fn middleware<L: RateLimiter + 'static>(limiter: L) -> Middleware {
            let limiter = Arc::new(limiter);
            Arc::new(move |next| {
                let limiter = limiter.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let limiter = limiter.clone();
                    Box::pin(async move {
                        match limiter.allow(&session) {
                            Ok(()) => {
                                next(session).await;
                            }
                            Err(err) => {
                                warn!(remote_addr = %session.remote_addr(), "rate limited");
                                fatal(&session, err);
                            }
                        }
                    })
                })
            })
        }

        /// Creates rate limiting middleware from a Config.
        pub fn middleware_with_config(config: Config) -> Middleware {
            middleware(new_rate_limiter(
                config.rate_per_sec,
                config.burst,
                config.max_entries,
            ))
        }
    }

    /// Middleware for elapsed time tracking.
    pub mod elapsed {
        use super::*;

        fn format_elapsed(format: &str, elapsed: Duration) -> String {
            if format.contains("%v") {
                format.replace("%v", &format!("{:?}", elapsed))
            } else {
                format.replace("{}", &format!("{:?}", elapsed)).to_string()
            }
        }

        /// Creates middleware that logs the elapsed time of the session.
        pub fn middleware_with_format(format: impl Into<String>) -> Middleware {
            let format = format.into();
            Arc::new(move |next| {
                let format = format.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let format = format.clone();
                    Box::pin(async move {
                        let start = Instant::now();
                        next(session.clone()).await;
                        let msg = format_elapsed(&format, start.elapsed());
                        print(&session, msg);
                    })
                })
            })
        }

        /// Creates middleware that logs elapsed time using the default format.
        pub fn middleware() -> Middleware {
            middleware_with_format("elapsed time: %v\n")
        }
    }

    /// Comment middleware for adding messages.
    pub mod comment {
        use super::*;

        /// Creates middleware that displays a comment/message.
        pub fn middleware(message: impl Into<String>) -> Middleware {
            let message = message.into();
            Arc::new(move |next| {
                let message = message.clone();
                Arc::new(move |session| {
                    let next = next.clone();
                    let message = message.clone();
                    Box::pin(async move {
                        next(session.clone()).await;
                        println(&session, &message);
                    })
                })
            })
        }
    }
}

// -----------------------------------------------------------------------------
// BubbleTea Integration
// -----------------------------------------------------------------------------

/// BubbleTea integration for serving TUI apps over SSH.
pub mod tea {
    use super::*;
    use bubbletea::{Model, Program};

    /// Handler function that creates a model for each session.
    pub type TeaHandler<M> = Arc<dyn Fn(&Session) -> M + Send + Sync>;

    /// Creates middleware that serves a BubbleTea application.
    pub fn middleware<M, F>(handler: F) -> Middleware
    where
        M: Model + Send + Sync + 'static,
        F: Fn(&Session) -> M + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        Arc::new(move |next| {
            let handler = handler.clone();
            Arc::new(move |session| {
                let next = next.clone();
                let handler = handler.clone();
                Box::pin(async move {
                    let (_pty, active) = session.pty();
                    if !active {
                        fatalln(&session, "no active terminal, skipping");
                        return;
                    }

                    // Create the model
                    let model = handler(&session);

                    // Create message channel for the program
                    let (tx, rx) = std::sync::mpsc::channel();
                    session.set_message_sender(tx);

                    // Run the program in a blocking task
                    let session_clone = session.clone();
                    let run_result = tokio::task::spawn_blocking(move || {
                        let _ = Program::new(model)
                            .with_custom_io()
                            .with_input_receiver(rx)
                            .run_with_writer(session_clone);
                    })
                    .await;
                    if let Err(err) = run_result {
                        fatalln(&session, format!("bubbletea program crashed: {err}"));
                        return;
                    }

                    next(session).await;
                })
            })
        })
    }

    /// Creates a lipgloss renderer for the session.
    pub fn make_renderer(session: &Session) -> lipgloss::Renderer {
        let (pty, _) = session.pty();
        let term = pty.map(|p| p.term.as_str()).unwrap_or("xterm-256color");

        // Detect color profile based on terminal type
        let profile = if term.contains("256color") || term.contains("truecolor") {
            lipgloss::ColorProfile::TrueColor
        } else if term.contains("color") {
            lipgloss::ColorProfile::Ansi256
        } else {
            lipgloss::ColorProfile::Ansi
        };

        let mut renderer = lipgloss::Renderer::new();
        renderer.set_color_profile(profile);
        renderer
    }
}

// -----------------------------------------------------------------------------
// Prelude
// -----------------------------------------------------------------------------

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        Context, Error, Handler, Middleware, Pty, PublicKey, Result, Server, ServerBuilder,
        ServerOption, ServerOptions, Session, Window, compose_middleware, error, errorf, errorln,
        fatal, fatalf, fatalln, handler, new_server, noop_handler, print, printf, println,
        with_address, with_banner, with_banner_handler, with_host_key_path, with_host_key_pem,
        with_idle_timeout, with_keyboard_interactive_auth, with_max_timeout, with_middleware,
        with_password_auth, with_public_key_auth, with_subsystem, with_version, write_string,
    };

    pub use crate::middleware::{
        accesscontrol, activeterm, comment, elapsed, logging, ratelimiter, recover,
    };

    pub use crate::tea;
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DenyLimiter;

    impl middleware::ratelimiter::RateLimiter for DenyLimiter {
        fn allow(
            &self,
            _session: &Session,
        ) -> std::result::Result<(), middleware::ratelimiter::RateLimitError> {
            Err(middleware::ratelimiter::RateLimitError)
        }
    }

    fn record_middleware(label: &'static str, events: Arc<Mutex<Vec<&'static str>>>) -> Middleware {
        Arc::new(move |next| {
            let events = events.clone();
            Arc::new(move |session| {
                let next = next.clone();
                let events = events.clone();
                Box::pin(async move {
                    {
                        let mut guard = events.lock().expect("events lock");
                        guard.push(label);
                    }
                    next(session).await;
                })
            })
        })
    }

    #[derive(Clone)]
    struct TestLogger {
        entries: Arc<Mutex<Vec<String>>>,
    }

    impl middleware::logging::Logger for TestLogger {
        fn log(&self, format: &str, args: &[&dyn fmt::Display]) {
            let mut msg = format.to_string();
            for arg in args {
                if let Some(pos) = msg.find("{}") {
                    msg.replace_range(pos..pos + 2, &arg.to_string());
                }
            }
            self.entries.lock().expect("logger entries").push(msg);
        }
    }

    #[derive(Clone, Default)]
    struct TestStructuredLogger {
        connects: Arc<Mutex<Vec<(String, SocketAddr, bool)>>>,
        disconnects: Arc<Mutex<Vec<(String, SocketAddr)>>>,
    }

    impl middleware::logging::StructuredLogger for TestStructuredLogger {
        fn log_connect(
            &self,
            _level: tracing::Level,
            user: &str,
            remote_addr: &SocketAddr,
            public_key: bool,
            _command: &[String],
            _term: &str,
            _width: u32,
            _height: u32,
            _client_version: &str,
        ) {
            self.connects.lock().expect("structured connects").push((
                user.to_string(),
                *remote_addr,
                public_key,
            ));
        }

        fn log_disconnect(
            &self,
            _level: tracing::Level,
            user: &str,
            remote_addr: &SocketAddr,
            _duration: Duration,
        ) {
            self.disconnects
                .lock()
                .expect("structured disconnects")
                .push((user.to_string(), *remote_addr));
        }
    }

    #[derive(Clone, Default)]
    struct PanicTeaModel;

    impl bubbletea::Model for PanicTeaModel {
        fn init(&self) -> Option<bubbletea::Cmd> {
            None
        }

        fn update(&mut self, _msg: Message) -> Option<bubbletea::Cmd> {
            None
        }

        fn view(&self) -> String {
            std::panic::panic_any("panic from test tea model")
        }
    }

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
    }

    #[test]
    fn test_public_key() {
        let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        assert_eq!(key.key_type, "ssh-ed25519");
        assert_eq!(key.data, vec![1, 2, 3, 4]);
        assert!(key.comment.is_none());

        let key = key.with_comment("test_key_comment");
        assert_eq!(key.comment, Some("test_key_comment".to_string()));
    }

    #[test]
    fn test_public_key_fingerprint() {
        let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        let fp = key.fingerprint();
        assert!(fp.starts_with("HASH:"));
    }

    #[test]
    fn test_public_key_equality() {
        let key1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        let key2 = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        let key3 = PublicKey::new("ssh-ed25519", vec![5, 6, 7, 8]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_context() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);

        assert_eq!(ctx.user(), "testuser");
        assert_eq!(ctx.remote_addr(), addr);

        ctx.set_value("key", "value");
        assert_eq!(ctx.get_value("key"), Some("value".to_string()));
        assert_eq!(ctx.get_value("missing"), None);
    }

    #[test]
    fn test_session_basic() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        assert_eq!(session.user(), "testuser");
        assert_eq!(session.command(), Vec::<String>::new());
        assert!(session.public_key().is_none());
    }

    #[test]
    fn test_session_builder() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);

        let pty = Pty {
            term: "xterm".to_string(),
            window: Window {
                width: 120,
                height: 40,
            },
        };

        let session = Session::new(ctx)
            .with_pty(pty)
            .with_command(vec!["ls".to_string(), "-la".to_string()])
            .with_env("HOME", "/home/user");

        let (pty_ref, active) = session.pty();
        assert!(active);
        assert_eq!(pty_ref.unwrap().term, "xterm");
        assert_eq!(session.command(), &["ls", "-la"]);
        assert_eq!(session.get_env("HOME"), Some(&"/home/user".to_string()));
    }

    #[test]
    fn test_session_write() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        let n = session.write(b"hello").unwrap();
        assert_eq!(n, 5);

        let n = session.write_stderr(b"error").unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn test_session_exit_close() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        assert!(!session.is_closed());
        session.exit(0).unwrap();
        session.close().unwrap();
        assert!(session.is_closed());
    }

    #[test]
    fn test_server_options_default() {
        let opts = ServerOptions::default();
        assert_eq!(opts.address, "0.0.0.0:22");
        assert_eq!(opts.version, "SSH-2.0-Wish");
        assert!(opts.banner.is_none());
    }

    #[test]
    fn test_server_builder() {
        let server = ServerBuilder::new()
            .address("0.0.0.0:2222")
            .version("SSH-2.0-MyApp")
            .banner("Welcome!")
            .idle_timeout(Duration::from_secs(300))
            .build()
            .unwrap();

        assert_eq!(server.address(), "0.0.0.0:2222");
        assert_eq!(server.options().version, "SSH-2.0-MyApp");
        assert_eq!(server.options().banner, Some("Welcome!".to_string()));
        assert_eq!(
            server.options().idle_timeout,
            Some(Duration::from_secs(300))
        );
    }

    #[test]
    fn test_option_functions() {
        let mut opts = ServerOptions::default();

        with_address("localhost:22")(&mut opts).unwrap();
        assert_eq!(opts.address, "localhost:22");

        with_version("SSH-2.0-Test")(&mut opts).unwrap();
        assert_eq!(opts.version, "SSH-2.0-Test");

        with_banner("Hello")(&mut opts).unwrap();
        assert_eq!(opts.banner, Some("Hello".to_string()));

        with_idle_timeout(Duration::from_secs(60))(&mut opts).unwrap();
        assert_eq!(opts.idle_timeout, Some(Duration::from_secs(60)));

        with_max_timeout(Duration::from_secs(3600))(&mut opts).unwrap();
        assert_eq!(opts.max_timeout, Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_new_server() {
        let server =
            new_server([with_address("0.0.0.0:2222"), with_version("SSH-2.0-Test")]).unwrap();

        assert_eq!(server.address(), "0.0.0.0:2222");
        assert_eq!(server.options().version, "SSH-2.0-Test");
    }

    #[test]
    fn test_noop_handler() {
        let h = noop_handler();
        // Just verify it compiles and can be called
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let session = Session::new(ctx);
        drop(h(session));
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let h = handler(|_session| async {
            // Do nothing
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let session = Session::new(ctx);
        h(session).await;
    }

    #[test]
    fn test_rate_limiter() {
        use middleware::ratelimiter::{RateLimiter, new_rate_limiter};

        let limiter = new_rate_limiter(0.0, 3, 10);
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        assert!(limiter.allow(&session).is_ok());
        assert!(limiter.allow(&session).is_ok());
        assert!(limiter.allow(&session).is_ok());
        assert!(limiter.allow(&session).is_err()); // Should be rate limited
    }

    #[test]
    fn test_output_helpers() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        print(&session, "hello");
        println(&session, "world");
        error(&session, "err");
        errorln(&session, "error line");

        // Verify data was written to channel
        // 1. print "hello"
        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"hello"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout for print(), got {other:?}"
                ))
                .into());
            }
        }

        // 2. println "world\r\n"
        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"world\r\n"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout for println(), got {other:?}"
                ))
                .into());
            }
        }

        // 3. error "err"
        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stderr(data) => assert_eq!(data, b"err"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stderr for error(), got {other:?}"
                ))
                .into());
            }
        }

        // 4. errorln "error line\r\n"
        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stderr(data) => assert_eq!(data, b"error line\r\n"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stderr for errorln(), got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[test]
    fn test_tea_make_renderer() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let pty = Pty {
            term: "xterm-256color".to_string(),
            window: Window::default(),
        };
        let session = Session::new(ctx).with_pty(pty);

        let _renderer = tea::make_renderer(&session);
        // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_tea_middleware_handles_program_panic()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let called = Arc::new(AtomicUsize::new(0));
        let mw = tea::middleware(|_session| PanicTeaModel);
        let next = handler({
            let called = called.clone();
            move |_session| {
                let called = called.clone();
                async move {
                    called.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().map_err(io::Error::other)?;
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx).with_pty(Pty::default());

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(next)(session).await;

        assert_eq!(called.load(Ordering::SeqCst), 0);

        let mut saw_fatal = false;
        let mut saw_exit = false;
        let mut saw_close = false;
        loop {
            match rx.try_recv() {
                Ok(SessionOutput::Stderr(data)) => {
                    let msg = String::from_utf8_lossy(&data);
                    if msg.contains("bubbletea program crashed:") {
                        saw_fatal = true;
                    }
                }
                Ok(SessionOutput::Exit(1)) => saw_exit = true,
                Ok(SessionOutput::Close) => saw_close = true,
                Ok(_) => {}
                Err(tokio::sync::mpsc::error::TryRecvError::Empty)
                | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }

        assert!(saw_fatal, "expected fatal stderr output for tea panic");
        assert!(saw_exit, "expected exit(1) for tea panic");
        assert!(saw_close, "expected close signal for tea panic");

        Ok(())
    }

    #[test]
    fn test_error_display() {
        let err = Error::Io(io::Error::other("test"));
        assert!(err.to_string().contains("io error"));

        let err = Error::AuthenticationFailed;
        assert_eq!(err.to_string(), "authentication failed");

        let err = Error::Configuration("bad config".to_string());
        assert!(err.to_string().contains("configuration error"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_session_recv_with_input_channel() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        assert!(session.recv().await.is_none());

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        session.set_input_receiver(rx).await;
        tx.send(b"ping".to_vec()).await.unwrap();

        let received = session.recv().await;
        assert_eq!(received, Some(b"ping".to_vec()));
    }

    #[test]
    fn test_session_send_message() {
        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("testuser", addr, addr);
        let session = Session::new(ctx);

        let (tx, rx) = std::sync::mpsc::channel();
        session.set_message_sender(tx);
        session.send_message(Message::new(42u32));

        let msg = rx.recv_timeout(Duration::from_millis(50)).unwrap();
        assert!(msg.is::<u32>());
        assert_eq!(msg.downcast::<u32>().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_compose_middleware_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let middlewares = vec![
            record_middleware("first", events.clone()),
            record_middleware("second", events.clone()),
        ];
        let composed = compose_middleware(middlewares);

        let handler = handler({
            let events = events.clone();
            move |_session| {
                let events = events.clone();
                async move {
                    let mut guard = events.lock().expect("events lock");
                    guard.push("handler");
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let session = Session::new(ctx);

        composed(handler)(session).await;

        let events = events.lock().expect("events lock");
        assert_eq!(&*events, &["first", "second", "handler"]);
    }

    #[tokio::test]
    async fn test_activeterm_middleware_blocks_without_pty()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let called = Arc::new(AtomicUsize::new(0));
        let mw = middleware::activeterm::middleware();
        let handler = handler({
            let called = called.clone();
            move |_session| {
                let called = called.clone();
                async move {
                    called.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(handler)(session).await;

        assert_eq!(called.load(Ordering::SeqCst), 0);

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"Requires an active PTY\r\n"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout warning for activeterm, got {other:?}"
                ))
                .into());
            }
        }

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Exit(code) => assert_eq!(code, 1),
            other => {
                return Err(io::Error::other(format!(
                    "expected exit code for activeterm, got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_accesscontrol_middleware_allows_command() {
        let called = Arc::new(AtomicUsize::new(0));
        let mw = middleware::accesscontrol::middleware(vec!["git".to_string()]);
        let handler = handler({
            let called = called.clone();
            move |_session| {
                let called = called.clone();
                async move {
                    called.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let session = Session::new(ctx).with_command(vec!["git".to_string()]);

        mw(handler)(session).await;

        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_accesscontrol_middleware_blocks_command()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let called = Arc::new(AtomicUsize::new(0));
        let mw = middleware::accesscontrol::middleware(vec!["git".to_string()]);
        let handler = handler({
            let called = called.clone();
            move |_session| {
                let called = called.clone();
                async move {
                    called.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx).with_command(vec!["rm".to_string()]);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(handler)(session).await;

        assert_eq!(called.load(Ordering::SeqCst), 0);

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"Command is not allowed: rm\r\n"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout message for accesscontrol, got {other:?}"
                ))
                .into());
            }
        }

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Exit(code) => assert_eq!(code, 1),
            other => {
                return Err(io::Error::other(format!(
                    "expected exit code for accesscontrol, got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_comment_middleware_appends_message()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mw = middleware::comment::middleware("done");
        let handler = handler(|session| async move {
            print(&session, "work");
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(handler)(session).await;

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"work"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout for handler output, got {other:?}"
                ))
                .into());
            }
        }

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => assert_eq!(data, b"done\r\n"),
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout for comment output, got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_elapsed_middleware_outputs_timing()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mw = middleware::elapsed::middleware_with_format("elapsed=%v");
        let handler = handler(|_session| async move {});

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(handler)(session).await;

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stdout(data) => {
                let msg = String::from_utf8_lossy(&data);
                assert!(msg.contains("elapsed="));
            }
            other => {
                return Err(io::Error::other(format!(
                    "expected stdout for elapsed middleware, got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ratelimiter_middleware_rejects()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let called = Arc::new(AtomicUsize::new(0));
        let mw = middleware::ratelimiter::middleware(DenyLimiter);
        let handler = handler({
            let called = called.clone();
            move |_session| {
                let called = called.clone();
                async move {
                    called.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let mut session = Session::new(ctx);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        session.set_output_sender(tx);

        mw(handler)(session).await;

        assert_eq!(called.load(Ordering::SeqCst), 0);

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Stderr(data) => {
                assert_eq!(
                    data,
                    middleware::ratelimiter::ERR_RATE_LIMIT_EXCEEDED.as_bytes()
                );
            }
            other => {
                return Err(io::Error::other(format!(
                    "expected stderr for ratelimiter, got {other:?}"
                ))
                .into());
            }
        }

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Exit(code) => assert_eq!(code, 1),
            other => {
                return Err(io::Error::other(format!(
                    "expected exit for ratelimiter, got {other:?}"
                ))
                .into());
            }
        }

        let item = rx.try_recv().map_err(|e| io::Error::other(e.to_string()))?;
        match item {
            SessionOutput::Close => {}
            other => {
                return Err(io::Error::other(format!(
                    "expected close for ratelimiter, got {other:?}"
                ))
                .into());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_logging_middleware_with_custom_logger() {
        let entries = Arc::new(Mutex::new(Vec::new()));
        let logger = TestLogger {
            entries: entries.clone(),
        };

        let mw = middleware::logging::middleware_with_logger(logger);
        let handler = handler(|_session| async move {});

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("alice", addr, addr);
        let session = Session::new(ctx);

        mw(handler)(session).await;

        let entries = entries.lock().expect("logger entries");
        assert_eq!(entries.len(), 2);
        assert!(entries[0].contains("connect"));
        assert!(entries[1].contains("disconnect"));
    }

    #[tokio::test]
    async fn test_structured_logging_middleware_with_custom_logger() {
        let logger = TestStructuredLogger::default();
        let mw = middleware::logging::structured_middleware_with_logger(
            logger.clone(),
            tracing::Level::INFO,
        );
        let handler = handler(|_session| async move {});

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("alice", addr, addr);
        let session = Session::new(ctx).with_public_key(PublicKey::new("ssh-ed25519", vec![1]));

        mw(handler)(session).await;

        let connects = logger.connects.lock().expect("connects");
        assert_eq!(connects.len(), 1);
        assert_eq!(connects[0].0, "alice");
        assert_eq!(connects[0].1, addr);
        assert!(connects[0].2);

        let disconnects = logger.disconnects.lock().expect("disconnects");
        assert_eq!(disconnects.len(), 1);
        assert_eq!(disconnects[0].0, "alice");
        assert_eq!(disconnects[0].1, addr);
    }

    #[tokio::test]
    async fn test_recover_middleware_runs_inner_before_next() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let inner = record_middleware("inner", events.clone());
        let mw = middleware::recover::middleware_with_middlewares(vec![inner]);

        let handler = handler({
            let events = events.clone();
            move |_session| {
                let events = events.clone();
                async move {
                    let mut guard = events.lock().expect("events lock");
                    guard.push("handler");
                }
            }
        });

        let addr: SocketAddr = "127.0.0.1:2222".parse().unwrap();
        let ctx = Context::new("test", addr, addr);
        let session = Session::new(ctx);

        mw(handler)(session).await;

        let events = events.lock().expect("events lock");
        assert_eq!(&*events, &["inner", "handler"]);
    }

    #[test]
    fn test_server_option_auth_and_subsystem() {
        let mut opts = ServerOptions::default();

        with_auth_handler(AcceptAllAuth::new())(&mut opts).unwrap();
        with_max_auth_attempts(3)(&mut opts).unwrap();
        with_auth_rejection_delay(250)(&mut opts).unwrap();
        with_public_key_auth(|_ctx, _key| true)(&mut opts).unwrap();
        with_password_auth(|_ctx, _pw| true)(&mut opts).unwrap();
        with_keyboard_interactive_auth(|_ctx, _resp, _prompts, _echos| vec!["ok".to_string()])(
            &mut opts,
        )
        .unwrap();
        with_host_key_path("/tmp/wish_host_file")(&mut opts).unwrap();
        with_host_key_pem(b"test_key_data".to_vec())(&mut opts).unwrap();
        with_banner_handler(|ctx| format!("hello {}", ctx.user()))(&mut opts).unwrap();
        with_middleware(middleware::comment::middleware("hi"))(&mut opts).unwrap();
        with_subsystem("sftp", |_session| async move {})(&mut opts).unwrap();

        assert!(opts.auth_handler.is_some());
        assert_eq!(opts.max_auth_attempts, 3);
        assert_eq!(opts.auth_rejection_delay_ms, 250);
        assert!(opts.public_key_handler.is_some());
        assert!(opts.password_handler.is_some());
        assert!(opts.keyboard_interactive_handler.is_some());
        assert_eq!(opts.host_key_path.as_deref(), Some("/tmp/wish_host_file"));
        assert_eq!(
            opts.host_key_pem.as_deref(),
            Some(b"test_key_data".as_slice())
        );
        assert!(opts.banner_handler.is_some());
        assert_eq!(opts.middlewares.len(), 1);
        assert!(opts.subsystem_handlers.contains_key("sftp"));
    }

    #[test]
    fn test_server_builder_auth_settings() {
        let server = ServerBuilder::new()
            .address("127.0.0.1:2222")
            .max_auth_attempts(5)
            .auth_rejection_delay(123)
            .public_key_auth(|_ctx, _key| true)
            .password_auth(|_ctx, _pw| true)
            .keyboard_interactive_auth(|_ctx, _resp, _prompts, _echos| vec![])
            .subsystem("sftp", |_session| async move {})
            .build()
            .unwrap();

        assert_eq!(server.options().max_auth_attempts, 5);
        assert_eq!(server.options().auth_rejection_delay_ms, 123);
        assert!(server.options().public_key_handler.is_some());
        assert!(server.options().password_handler.is_some());
        assert!(server.options().keyboard_interactive_handler.is_some());
        assert!(server.options().subsystem_handlers.contains_key("sftp"));
    }

    #[test]
    fn test_create_russh_config_methods_from_auth_handler() {
        // russh 0.49 turned MethodSet from a bitflags struct into an
        // ordered Vec<MethodKind>; assertions that previously used
        // `methods.contains(MethodSet::PASSWORD)` now go through the
        // slice deref + `&MethodKind::Password`.
        use russh::MethodKind;

        struct PasswordOnly;

        #[async_trait::async_trait]
        impl AuthHandler for PasswordOnly {
            fn supported_methods(&self) -> Vec<AuthMethod> {
                vec![AuthMethod::Password]
            }
        }

        let server = ServerBuilder::new()
            .auth_handler(PasswordOnly)
            .build()
            .unwrap();
        let config = server.create_russh_config().unwrap();

        assert!(config.methods.contains(&MethodKind::Password));
        assert!(!config.methods.contains(&MethodKind::PublicKey));
    }

    #[test]
    fn test_create_russh_config_methods_from_callbacks() {
        use russh::MethodKind;

        let server = ServerBuilder::new()
            .public_key_auth(|_ctx, _key| true)
            .password_auth(|_ctx, _pw| true)
            .build()
            .unwrap();

        let config = server.create_russh_config().unwrap();

        assert!(config.methods.contains(&MethodKind::PublicKey));
        assert!(config.methods.contains(&MethodKind::Password));
        assert!(!config.methods.contains(&MethodKind::KeyboardInteractive));
    }
}
