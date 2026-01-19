# Go Wish to Rust API Mapping

> Mapping of Go Wish APIs to Rust russh equivalents.

---

## Server Configuration

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `wish.NewServer(options...)` | `Server::new(options)` or `ServerBuilder::new()` |
| `WithAddress(addr string)` | `with_address(addr)` / `.address(addr)` |
| `WithVersion(version string)` | `with_version(version)` / `.version(version)` |
| `WithBanner(banner string)` | `with_banner(banner)` / `.banner(banner)` |
| `WithHostKeyPath(path string)` | `with_host_key_path(path)` / `.host_key_path(path)` |
| `WithHostKeyPEM(pem []byte)` | `with_host_key_pem(pem)` / `.host_key_pem(pem)` |
| `WithIdleTimeout(d time.Duration)` | `with_idle_timeout(duration)` / `.idle_timeout(duration)` |
| `WithMaxTimeout(d time.Duration)` | `with_max_timeout(duration)` / `.max_timeout(duration)` |

## Authentication

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `WithPublicKeyAuth(fn func(Context, ssh.PublicKey) bool)` | `with_public_key_auth(handler)` / `.public_key_auth(handler)` |
| `WithPasswordAuth(fn func(Context, string) bool)` | `with_password_auth(handler)` / `.password_auth(handler)` |
| `WithKeyboardInteractiveAuth(fn)` | `with_keyboard_interactive_auth(handler)` / `.keyboard_interactive_auth(handler)` |

## Session API

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `Session.User() string` | `Session::user() -> &str` |
| `Session.RemoteAddr() net.Addr` | `Session::remote_addr() -> SocketAddr` |
| `Session.LocalAddr() net.Addr` | `Session::local_addr() -> SocketAddr` |
| `Session.Pty() (Pty, <-chan Window, bool)` | `Session::pty() -> (Option<&Pty>, bool)` |
| `Session.Command() []string` | `Session::command() -> &[String]` |
| `Session.Environ() []string` | `Session::environ() -> &HashMap<String, String>` |
| `Session.PublicKey() ssh.PublicKey` | `Session::public_key() -> Option<&PublicKey>` |
| `Session.Context() Context` | `Session::context() -> &Context` |
| `Session.Subsystem() string` | `Session::subsystem() -> Option<&str>` |
| `Session.Write([]byte) (int, error)` | `Session::write(&[u8]) -> io::Result<usize>` |
| `Session.Exit(code int) error` | `Session::exit(code: i32) -> io::Result<()>` |
| `Session.Close() error` | `Session::close() -> io::Result<()>` |

## Context API

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `Context.User() string` | `Context::user() -> &str` |
| `Context.RemoteAddr() net.Addr` | `Context::remote_addr() -> SocketAddr` |
| `Context.LocalAddr() net.Addr` | `Context::local_addr() -> SocketAddr` |
| `Context.ClientVersion() string` | `Context::client_version() -> &str` |
| `Context.SetValue(key, value any)` | `Context::set_value(key, value)` |
| `Context.Value(key any) any` | `Context::get_value(key) -> Option<String>` |

## Output Helpers

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `wish.Print(Session, args...)` | `print(session, args)` |
| `wish.Println(Session, args...)` | `println(session, args)` |
| `wish.Printf(Session, format, args...)` | `printf(session, format, args)` |
| `wish.Error(Session, args...)` | `error(session, args)` |
| `wish.Errorln(Session, args...)` | `errorln(session, args)` |
| `wish.Errorf(Session, format, args...)` | `errorf(session, format, args)` |
| `wish.Fatal(Session, args...)` | `fatal(session, args)` |
| `wish.Fatalln(Session, args...)` | `fatalln(session, args)` |
| `wish.Fatalf(Session, format, args...)` | `fatalf(session, format, args)` |
| `wish.WriteString(Session, s string)` | `write_string(session, s)` |

## Middleware

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `type Middleware = func(Handler) Handler` | `type Middleware = Arc<dyn Fn(Handler) -> Handler>` |
| `activeterm.Middleware()` | `middleware::activeterm::middleware()` |
| `accesscontrol.Middleware([]string)` | `middleware::accesscontrol::middleware(vec![...])` |
| `logging.Middleware()` | `middleware::logging::middleware()` |
| `logging.MiddlewareWithLogger(Logger)` | `middleware::logging::middleware_with_logger(logger)` |
| `recover.Middleware()` | `middleware::recover::middleware()` |
| `comment.Middleware(string)` | `middleware::comment::middleware(message)` |
| `elapsed.Middleware()` | `middleware::elapsed::middleware()` |

## BubbleTea Integration

| Go Wish | Rust Equivalent |
|---------|-----------------|
| `bubbletea.Handler(func(*tea.Program))` | `tea::middleware(handler)` |
| `bubbletea.MakeRenderer(Session)` | `tea::make_renderer(session)` |
| `bubbletea.WithAltScreen()` | Part of Program options |
| `bubbletea.WithMouseCellMotion()` | Part of Program options |

---

## russh Handler Trait Callbacks

The russh `Handler` trait requires implementing these async methods:

### Authentication

```rust
async fn auth_publickey(&mut self, user: &str, public_key: &PublicKey) -> Result<Auth, Self::Error>
async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error>
async fn auth_keyboard_interactive(&mut self, user: &str, submethods: &str, response: Option<Response>) -> Result<Auth, Self::Error>
```

### Channel Lifecycle

```rust
async fn channel_open_session(&mut self, channel: Channel<Msg>, session: &mut Session) -> Result<bool, Self::Error>
async fn channel_close(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>
async fn channel_eof(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>
```

### PTY and Shell

```rust
async fn pty_request(&mut self, channel: ChannelId, term: &str, col_width: u32, row_height: u32, pix_width: u32, pix_height: u32, modes: &[(Pty, u32)], session: &mut Session) -> Result<(), Self::Error>
async fn shell_request(&mut self, channel: ChannelId, session: &mut Session) -> Result<(), Self::Error>
async fn exec_request(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error>
async fn subsystem_request(&mut self, channel: ChannelId, name: &str, session: &mut Session) -> Result<(), Self::Error>
async fn window_change_request(&mut self, channel: ChannelId, col_width: u32, row_height: u32, pix_width: u32, pix_height: u32, session: &mut Session) -> Result<(), Self::Error>
```

### Data Transfer

```rust
async fn data(&mut self, channel: ChannelId, data: &[u8], session: &mut Session) -> Result<(), Self::Error>
async fn extended_data(&mut self, channel: ChannelId, code: u32, data: &[u8], session: &mut Session) -> Result<(), Self::Error>
```

---

## Feature Parity Checklist

### Core Server
- [x] Server configuration options
- [x] Builder pattern API
- [ ] Host key loading from file
- [ ] Host key generation
- [ ] TCP listener and accept loop
- [ ] Connection handling via russh

### Authentication
- [x] Public key auth handler signature
- [x] Password auth handler signature
- [x] Keyboard-interactive handler signature
- [ ] Wiring to russh Handler trait
- [ ] Constant-time rejection (automatic via russh)

### Session Management
- [x] Session struct with context
- [x] PTY information
- [x] Window dimensions
- [x] Environment variables
- [x] Command tracking
- [ ] Input/output bridging with russh

### Middleware
- [x] activeterm middleware
- [x] accesscontrol middleware
- [x] logging middleware
- [x] recover middleware
- [x] comment middleware
- [x] elapsed middleware
- [x] ratelimiter middleware
- [ ] Async middleware adaptation

### BubbleTea Integration
- [x] tea::middleware structure
- [x] tea::make_renderer
- [ ] Async input stream adapter
- [ ] Window resize → WindowSizeMsg
- [ ] Program lifecycle management

### Output
- [x] print/println/printf
- [x] error/errorln/errorf
- [x] fatal/fatalln/fatalf
- [x] write_string
