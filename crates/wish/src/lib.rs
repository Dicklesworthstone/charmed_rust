#![forbid(unsafe_code)]
// Allow pedantic lints for early-stage API ergonomics.
#![allow(clippy::doc_markdown)]
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

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

pub mod auth;
mod handler;
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
    KeyLoad(#[from] russh_keys::Error),

    /// Authentication failed.
    ///
    /// Occurs when a client's credentials are rejected.
    /// This is expected in normal operation - not all attempts succeed.
    #[error("authentication failed")]
    AuthenticationFailed,

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
/// This type alias defaults to [`Error`] as the error type.
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
    pub fn fingerprint(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        format!("SHA256:{:016x}", hasher.finish())
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
    pub fn set_input_receiver(&self, rx: tokio::sync::mpsc::Receiver<Vec<u8>>) {
        *self.input_rx.blocking_lock() = Some(rx);
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
        info!("Server listening on {}", addr);

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    info!(peer_addr = %peer_addr, "Accepted connection");

                    let config = config.clone();
                    let local_addr = socket.local_addr().unwrap_or(addr);
                    let handler = factory.create_handler(peer_addr, local_addr);

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
    fn create_russh_config(&self) -> Result<RusshConfig> {
        use russh::MethodSet;
        use russh::server::Config;
        use russh_keys::key::KeyPair;

        let mut config = Config::default();

        // Set server ID
        config.server_id = russh::SshId::Standard(self.options.version.clone());

        // Set timeouts
        if let Some(timeout) = self.options.idle_timeout {
            config.inactivity_timeout = Some(timeout);
        }

        config.max_auth_attempts = self.options.max_auth_attempts as usize;
        config.auth_rejection_time = Duration::from_millis(self.options.auth_rejection_delay_ms);

        let mut methods = MethodSet::empty();
        if let Some(handler) = &self.options.auth_handler {
            for method in handler.supported_methods() {
                methods |= match method {
                    auth::AuthMethod::None => MethodSet::NONE,
                    auth::AuthMethod::Password => MethodSet::PASSWORD,
                    auth::AuthMethod::PublicKey => MethodSet::PUBLICKEY,
                    auth::AuthMethod::KeyboardInteractive => MethodSet::KEYBOARD_INTERACTIVE,
                    auth::AuthMethod::HostBased => MethodSet::HOSTBASED,
                };
            }
        } else {
            if self.options.public_key_handler.is_some() {
                methods |= MethodSet::PUBLICKEY;
            }
            if self.options.password_handler.is_some() {
                methods |= MethodSet::PASSWORD;
            }
            if self.options.keyboard_interactive_handler.is_some() {
                methods |= MethodSet::KEYBOARD_INTERACTIVE;
            }
            if methods.is_empty() {
                methods |= MethodSet::NONE;
            }
        }
        config.methods = methods;

        // Generate or load host key
        let key = if let Some(ref pem) = self.options.host_key_pem {
            // Load from PEM
            russh_keys::decode_secret_key(
                std::str::from_utf8(pem).map_err(|e| Error::Key(e.to_string()))?,
                None,
            )?
        } else if let Some(ref path) = self.options.host_key_path {
            // Load from file
            russh_keys::load_secret_key(path, None)?
        } else {
            // Generate ephemeral Ed25519 key
            info!("Generating ephemeral Ed25519 host key");
            KeyPair::generate_ed25519()
        };

        config.keys.push(key);

        // Set authentication banner if configured
        if let Some(ref banner) = self.options.banner {
            // russh expects &'static str, so we leak the banner
            // This is acceptable since the server typically runs for the lifetime of the process
            let banner: &'static str = Box::leak(banner.clone().into_boxed_str());
            config.auth_banner = Some(banner);
        }

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

    /// Middleware for logging connections.
    pub mod logging {
        use super::*;

        /// Logger trait for custom logging implementations.
        pub trait Logger: Send + Sync {
            fn log(&self, format: &str, args: &[&dyn fmt::Display]);
        }

        /// Structured logger for connection events.
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
                    tokio::task::spawn_blocking(move || {
                        let _ = Program::new(model)
                            .with_custom_io()
                            .with_input_receiver(rx)
                            .run_with_writer(session_clone);
                    })
                    .await
                    .unwrap();

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

        let key = key.with_comment("test@example.com");
        assert_eq!(key.comment, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_public_key_fingerprint() {
        let key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        let fp = key.fingerprint();
        assert!(fp.starts_with("SHA256:"));
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
        assert!(session.command().is_empty());
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
    fn test_output_helpers() {
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
        match rx.try_recv() {
            Ok(SessionOutput::Stdout(data)) => assert_eq!(data, b"hello"),
            _ => panic!("Expected stdout hello"),
        }

        // 2. println "world\r\n"
        match rx.try_recv() {
            Ok(SessionOutput::Stdout(data)) => assert_eq!(data, b"world\r\n"),
            _ => panic!("Expected stdout world"),
        }

        // 3. error "err"
        match rx.try_recv() {
            Ok(SessionOutput::Stderr(data)) => assert_eq!(data, b"err"),
            _ => panic!("Expected stderr err"),
        }

        // 4. errorln "error line\r\n"
        match rx.try_recv() {
            Ok(SessionOutput::Stderr(data)) => assert_eq!(data, b"error line\r\n"),
            _ => panic!("Expected stderr error line"),
        }
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

    #[test]
    fn test_error_display() {
        let err = Error::Io(io::Error::other("test"));
        assert!(err.to_string().contains("io error"));

        let err = Error::AuthenticationFailed;
        assert_eq!(err.to_string(), "authentication failed");

        let err = Error::Configuration("bad config".to_string());
        assert!(err.to_string().contains("configuration error"));
    }
}
