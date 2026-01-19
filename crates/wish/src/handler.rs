//! russh Handler implementation for Wish SSH server.
//!
//! This module implements the `russh::server::Handler` trait to bridge
//! the russh SSH server with Wish's session and middleware system.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use russh::server::{Auth, Handler as RusshHandler, Msg, Session as RusshSession};
use russh::{Channel, ChannelId};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, trace, warn};
use bubbletea::{Message, KeyMsg, parse_sequence, WindowSizeMsg};

use crate::{
    Context, Error, Handler, Pty, PublicKey, ServerOptions, Session, SessionOutput, Window,
    compose_middleware, noop_handler,
};

// Re-export russh server types for use by Server
pub use russh::server::{Config as RusshConfig, run_stream};

/// Shared state for all connections to a server.
pub struct ServerState {
    /// Server options.
    pub options: ServerOptions,
    /// Composed handler (middleware + main handler).
    pub handler: Handler,
    /// Connection counter for generating IDs.
    pub connection_counter: RwLock<u64>,
}

impl ServerState {
    /// Creates new server state from options.
    pub fn new(options: ServerOptions) -> Self {
        // Compose middleware with the main handler
        let base_handler = options.handler.clone().unwrap_or_else(noop_handler);
        let handler = if options.middlewares.is_empty() {
            base_handler
        } else {
            let composed = compose_middleware(options.middlewares.clone());
            composed(base_handler)
        };

        Self {
            options,
            handler,
            connection_counter: RwLock::new(0),
        }
    }

    /// Returns the next connection ID.
    pub fn next_connection_id(&self) -> u64 {
        let mut counter = self.connection_counter.write();
        *counter += 1;
        *counter
    }
}

/// Per-channel state tracking.
struct ChannelState {
    /// The wish Session for this channel.
    session: Session,
    /// Input sender for data from client.
    input_tx: mpsc::Sender<Vec<u8>>,
    /// Whether shell/exec has started.
    started: bool,
}

/// Handler for a single SSH connection.
///
/// Implements `russh::server::Handler` to handle SSH protocol events
/// and bridge them to Wish's session/middleware system.
pub struct WishHandler {
    /// Connection ID for logging.
    connection_id: u64,
    /// Remote address.
    remote_addr: SocketAddr,
    /// Local address.
    local_addr: SocketAddr,
    /// User after authentication.
    user: Option<String>,
    /// Public key if auth'd via key.
    public_key: Option<russh_keys::key::PublicKey>,
    /// PTY info if allocated.
    pty: Option<Pty>,
    /// Current window dimensions.
    window: Window,
    /// Server-level shared state.
    server_state: Arc<ServerState>,
    /// Active channels.
    channels: HashMap<ChannelId, ChannelState>,
    /// Shutdown signal receiver.
    #[allow(dead_code)]
    shutdown_rx: broadcast::Receiver<()>,
}

impl WishHandler {
    /// Creates a new handler for a connection.
    pub fn new(
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
        server_state: Arc<ServerState>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        let connection_id = server_state.next_connection_id();
        debug!(
            connection_id,
            remote_addr = %remote_addr,
            "New connection handler created"
        );

        Self {
            connection_id,
            remote_addr,
            local_addr,
            user: None,
            public_key: None,
            pty: None,
            window: Window::default(),
            server_state,
            channels: HashMap::new(),
            shutdown_rx,
        }
    }

    /// Creates a Context from current connection state.
    fn make_context(&self, user: &str) -> Context {
        let ctx = Context::new(user, self.remote_addr, self.local_addr);
        ctx.set_value("connection_id", self.connection_id.to_string());
        ctx
    }

    /// Converts a russh public key to our PublicKey type.
    fn convert_public_key(key: &russh_keys::key::PublicKey) -> PublicKey {
        let key_name = key.name();
        let key_type = match key_name {
            "ssh-ed25519" => "ssh-ed25519",
            "rsa-sha2-256" | "rsa-sha2-512" | "ssh-rsa" => "ssh-rsa",
            "ecdsa-sha2-nistp256" => "ecdsa-sha2-nistp256",
            "ecdsa-sha2-nistp384" => "ecdsa-sha2-nistp384",
            "ecdsa-sha2-nistp521" => "ecdsa-sha2-nistp521",
            other => other,
        };

        // Get key bytes - use fingerprint as placeholder since we can't easily serialize
        let fingerprint = key.fingerprint();
        PublicKey::new(key_type, fingerprint.as_bytes().to_vec())
    }
}

#[async_trait]
impl RusshHandler for WishHandler {
    type Error = Error;

    /// Handle public key authentication.
    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &russh_keys::key::PublicKey,
    ) -> std::result::Result<Auth, Self::Error> {
        debug!(
            connection_id = self.connection_id,
            user = user,
            key_type = public_key.name(),
            "Public key auth attempt"
        );

        // Check if we have a public key handler
        if let Some(handler) = &self.server_state.options.public_key_handler {
            let ctx = self.make_context(user);
            let pk = Self::convert_public_key(public_key);

            if handler(&ctx, &pk) {
                info!(
                    connection_id = self.connection_id,
                    user = user,
                    "Public key auth accepted"
                );
                self.user = Some(user.to_string());
                self.public_key = Some(public_key.clone());
                return Ok(Auth::Accept);
            }
        }

        // If no handler or handler rejected, try other methods
        debug!(
            connection_id = self.connection_id,
            user = user,
            "Public key auth rejected"
        );
        Ok(Auth::Reject {
            proceed_with_methods: None,
        })
    }

    /// Handle password authentication.
    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> std::result::Result<Auth, Self::Error> {
        debug!(
            connection_id = self.connection_id,
            user = user,
            "Password auth attempt"
        );

        // Check if we have a password handler
        if let Some(handler) = &self.server_state.options.password_handler {
            let ctx = self.make_context(user);

            if handler(&ctx, password) {
                info!(
                    connection_id = self.connection_id,
                    user = user,
                    "Password auth accepted"
                );
                self.user = Some(user.to_string());
                return Ok(Auth::Accept);
            }
        }

        debug!(
            connection_id = self.connection_id,
            user = user,
            "Password auth rejected"
        );
        Ok(Auth::Reject {
            proceed_with_methods: None,
        })
    }

    /// Handle "none" authentication (for servers that accept all).
    async fn auth_none(&mut self, user: &str) -> std::result::Result<Auth, Self::Error> {
        // Accept if no auth handlers are configured
        let has_auth = self.server_state.options.public_key_handler.is_some()
            || self.server_state.options.password_handler.is_some()
            || self
                .server_state
                .options
                .keyboard_interactive_handler
                .is_some();

        if !has_auth {
            info!(
                connection_id = self.connection_id,
                user = user,
                "No auth configured, accepting connection"
            );
            self.user = Some(user.to_string());
            return Ok(Auth::Accept);
        }

        Ok(Auth::Reject {
            proceed_with_methods: None,
        })
    }

    /// Handle new session channel.
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut RusshSession,
    ) -> std::result::Result<bool, Self::Error> {
        let channel_id = channel.id();
        debug!(
            connection_id = self.connection_id,
            channel = ?channel_id,
            "Session channel opened"
        );

        // Create channel state
        let (input_tx, _input_rx) = mpsc::channel(1024);
        let (output_tx, mut output_rx) = mpsc::unbounded_channel::<SessionOutput>();

        let user = self.user.clone().unwrap_or_default();
        let ctx = self.make_context(&user);
        let mut wish_session = Session::new(ctx);
        wish_session.set_output_sender(output_tx);

        // Get session handle for sending exit status from spawned task
        let handle = session.handle();

        // Spawn output pump
        let connection_id = self.connection_id;
        tokio::spawn(async move {
            debug!(connection_id, channel = ?channel_id, "Starting output pump");
            while let Some(msg) = output_rx.recv().await {
                match msg {
                    SessionOutput::Stdout(data) => {
                        let _ = channel.data(&data[..]).await;
                    }
                    SessionOutput::Stderr(data) => {
                        let _ = channel.extended_data(1, &data[..]).await;
                    }
                    SessionOutput::Exit(code) => {
                        let _ = handle.exit_status_request(channel_id, code).await;
                        let _ = channel.close().await;
                        break;
                    }
                    SessionOutput::Close => {
                        let _ = channel.close().await;
                        break;
                    }
                }
            }
            debug!(connection_id, channel = ?channel_id, "Output pump finished");
        });

        // Add public key if authenticated via key
        if let Some(ref pk) = self.public_key {
            wish_session = wish_session.with_public_key(Self::convert_public_key(pk));
        }

        // Store channel reference in the session for later use
        wish_session
            .context()
            .set_value("channel_id", format!("{channel_id:?}"));

        self.channels.insert(
            channel_id,
            ChannelState {
                session: wish_session,
                input_tx,
                started: false,
            },
        );

        Ok(true)
    }

    /// Handle PTY request.
    async fn pty_request(
        &mut self,
        channel: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            term = term,
            width = col_width,
            height = row_height,
            "PTY request"
        );

        let pty = Pty {
            term: term.to_string(),
            window: Window {
                width: col_width,
                height: row_height,
            },
        };
        self.pty = Some(pty.clone());
        self.window = Window {
            width: col_width,
            height: row_height,
        };

        // Update channel session with PTY
        if let Some(state) = self.channels.get_mut(&channel) {
            state.session = state.session.clone().with_pty(pty);
        }

        session.channel_success(channel);
        Ok(())
    }

    /// Handle shell request.
    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            "Shell request"
        );

        if let Some(state) = self.channels.get_mut(&channel) {
            if state.started {
                warn!(
                    connection_id = self.connection_id,
                    channel = ?channel,
                    "Shell already started"
                );
                session.channel_failure(channel);
                return Ok(());
            }

            state.started = true;
            let wish_session = state.session.clone();
            let handler = self.server_state.handler.clone();
            let connection_id = self.connection_id;

            // Spawn the handler task
            tokio::spawn(async move {
                debug!(connection_id, "Starting handler");
                handler(wish_session).await;
                debug!(connection_id, "Handler completed");
            });

            session.channel_success(channel);
        } else {
            session.channel_failure(channel);
        }

        Ok(())
    }

    /// Handle exec request (command execution).
    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data).to_string();
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            command = %command,
            "Exec request"
        );

        if let Some(state) = self.channels.get_mut(&channel) {
            if state.started {
                session.channel_failure(channel);
                return Ok(());
            }

            // Parse command into args
            let args: Vec<String> = command.split_whitespace().map(String::from).collect();
            state.session = state.session.clone().with_command(args);
            state.started = true;

            let wish_session = state.session.clone();
            let handler = self.server_state.handler.clone();
            let connection_id = self.connection_id;

            tokio::spawn(async move {
                debug!(connection_id, "Starting exec handler");
                handler(wish_session).await;
                debug!(connection_id, "Exec handler completed");
            });

            session.channel_success(channel);
        } else {
            session.channel_failure(channel);
        }

        Ok(())
    }

    /// Handle environment variable request.
    async fn env_request(
        &mut self,
        channel: ChannelId,
        variable_name: &str,
        variable_value: &str,
        session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        trace!(
            connection_id = self.connection_id,
            channel = ?channel,
            name = variable_name,
            value = variable_value,
            "Environment variable request"
        );

        if let Some(state) = self.channels.get_mut(&channel) {
            state.session = state
                .session
                .clone()
                .with_env(variable_name, variable_value);
        }

        session.channel_success(channel);
        Ok(())
    }

    /// Handle subsystem request.
    async fn subsystem_request(
        &mut self,
        channel: ChannelId,
        name: &str,
        session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            subsystem = name,
            "Subsystem request"
        );

        // Check if we have a handler for this subsystem
        if let Some(handler) = self.server_state.options.subsystem_handlers.get(name) {
            if let Some(state) = self.channels.get_mut(&channel) {
                if state.started {
                    session.channel_failure(channel);
                    return Ok(());
                }

                state.session = state.session.clone().with_subsystem(name);
                state.started = true;

                let wish_session = state.session.clone();
                let handler = handler.clone();
                let connection_id = self.connection_id;
                let subsystem_name = name.to_string();

                tokio::spawn(async move {
                    debug!(
                        connection_id,
                        subsystem = %subsystem_name,
                        "Starting subsystem handler"
                    );
                    handler(wish_session).await;
                    debug!(connection_id, "Subsystem handler completed");
                });

                session.channel_success(channel);
                return Ok(());
            }
        }

        session.channel_failure(channel);
        Ok(())
    }

    /// Handle window change request.
    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        trace!(
            connection_id = self.connection_id,
            channel = ?channel,
            width = col_width,
            height = row_height,
            "Window change request"
        );

        self.window = Window {
            width: col_width,
            height: row_height,
        };

        // Update PTY window
        if let Some(ref mut pty) = self.pty {
            pty.window = self.window;
        }

        // Send WindowSizeMsg to bubbletea Program if running
        if let Some(state) = self.channels.get(&channel) {
            state.session.send_message(Message::new(WindowSizeMsg {
                width: col_width as u16,
                height: row_height as u16,
            }));
        }

        Ok(())
    }

    /// Handle data from client.
    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        trace!(
            connection_id = self.connection_id,
            channel = ?channel,
            len = data.len(),
            "Data received"
        );

        if let Some(state) = self.channels.get(&channel) {
            // Forward raw data to input_tx (legacy/stream support)
            let _ = state.input_tx.send(data.to_vec()).await;

            // Parse for bubbletea
            if let Some(key) = parse_sequence(data) {
                state.session.send_message(Message::new(key));
            } else {
                // Parse as UTF-8 chars
                if let Ok(s) = std::str::from_utf8(data) {
                    for c in s.chars() {
                        let key = KeyMsg::from_char(c);
                        state.session.send_message(Message::new(key));
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle channel EOF.
    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        _session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            "Channel EOF"
        );
        Ok(())
    }

    /// Handle channel close.
    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _session: &mut RusshSession,
    ) -> std::result::Result<(), Self::Error> {
        debug!(
            connection_id = self.connection_id,
            channel = ?channel,
            "Channel closed"
        );

        self.channels.remove(&channel);
        Ok(())
    }
}

/// Factory for creating WishHandler instances.
pub struct WishHandlerFactory {
    server_state: Arc<ServerState>,
    shutdown_tx: broadcast::Sender<()>,
}

impl WishHandlerFactory {
    /// Creates a new handler factory.
    pub fn new(options: ServerOptions) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            server_state: Arc::new(ServerState::new(options)),
            shutdown_tx,
        }
    }

    /// Creates a handler for a new connection.
    pub fn create_handler(&self, remote_addr: SocketAddr, local_addr: SocketAddr) -> WishHandler {
        WishHandler::new(
            remote_addr,
            local_addr,
            self.server_state.clone(),
            self.shutdown_tx.subscribe(),
        )
    }

    /// Signals all handlers to shut down.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_state_new() {
        let options = ServerOptions::default();
        let state = ServerState::new(options);
        assert_eq!(state.next_connection_id(), 1);
        assert_eq!(state.next_connection_id(), 2);
    }
}
