use std::fmt;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, timeout};
use wish::{Handler, ServerBuilder, handler, println};

pub const TEST_USER: &str = "testuser";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
pub const LONG_TIMEOUT: Duration = Duration::from_secs(20);

pub fn ssh_available() -> bool {
    StdCommand::new("ssh").arg("-V").output().is_ok()
}

pub fn ssh_keygen_available() -> bool {
    StdCommand::new("ssh-keygen").arg("-h").output().is_ok()
}

pub struct TestServer {
    port: u16,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    pub async fn start(builder: ServerBuilder) -> Self {
        let port = pick_unused_port();
        let server = builder
            .address(format!("127.0.0.1:{port}"))
            .build()
            .expect("build wish server");

        let handle = tokio::spawn(async move {
            if let Err(err) = server.listen().await {
                eprintln!("wish server error: {err}");
            }
        });

        wait_for_port(port).await;

        Self { port, handle }
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub async fn stop(self) {
        self.handle.abort();
        let _ = self.handle.await;
    }
}

fn pick_unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind temp port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

async fn wait_for_port(port: u16) {
    let addr = format!("127.0.0.1:{port}");
    for _ in 0..100 {
        if TcpStream::connect(&addr).await.is_ok() {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("wish server did not start in time");
}

#[derive(Clone, Default)]
pub struct LogCapture {
    entries: Arc<Mutex<Vec<String>>>,
}

impl LogCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> Vec<String> {
        self.entries.lock().expect("log capture lock").clone()
    }
}

impl wish::middleware::logging::Logger for LogCapture {
    fn log(&self, format: &str, args: &[&dyn fmt::Display]) {
        let mut msg = format.to_string();
        for arg in args {
            if let Some(pos) = msg.find("{}") {
                msg.replace_range(pos..pos + 2, &arg.to_string());
            }
        }
        self.entries.lock().expect("log capture lock").push(msg);
    }
}

#[derive(Clone)]
pub struct SshClient {
    port: u16,
    user: String,
    identity_file: Option<PathBuf>,
}

impl SshClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            user: TEST_USER.to_string(),
            identity_file: None,
        }
    }

    #[allow(dead_code)] // Available for tests that need custom usernames
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = user.into();
        self
    }

    pub fn with_identity_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.identity_file = Some(path.into());
        self
    }

    pub async fn exec(&self, command: &str) -> io::Result<Output> {
        self.exec_with_options(
            command,
            &[
                "BatchMode=yes",
                "PreferredAuthentications=publickey,password,keyboard-interactive,none",
            ],
        )
        .await
    }

    pub async fn exec_with_options(
        &self,
        command: &str,
        extra_opts: &[&str],
    ) -> io::Result<Output> {
        let mut cmd = self.base_command();
        for opt in extra_opts {
            cmd.arg("-o").arg(opt);
        }
        cmd.arg(command);
        run_with_timeout(cmd, DEFAULT_TIMEOUT).await
    }

    pub async fn exec_with_password(&self, command: &str, password: &str) -> io::Result<Output> {
        let dir = tempfile::tempdir()?;
        let script_path = dir.path().join("askpass.sh");
        let script = "#!/bin/sh\nprintf '%s\\n' \"$WISH_TEST_PASSWORD\"\n";
        std::fs::write(&script_path, script)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&script_path, perms)?;
        }

        let mut cmd = self.base_command();
        cmd.arg("-o")
            .arg("BatchMode=no")
            .arg("-o")
            .arg("PreferredAuthentications=password")
            .arg("-o")
            .arg("PubkeyAuthentication=no")
            .arg("-o")
            .arg("NumberOfPasswordPrompts=1")
            .env("SSH_ASKPASS", &script_path)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("DISPLAY", "1")
            .env("WISH_TEST_PASSWORD", password)
            .arg(command);

        run_with_timeout(cmd, DEFAULT_TIMEOUT).await
    }

    pub fn spawn_interactive(&self) -> io::Result<tokio::process::Child> {
        let mut cmd = self.base_command();
        cmd.arg("-tt").arg("-o").arg("BatchMode=yes");
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd.spawn()
    }

    fn base_command(&self) -> TokioCommand {
        let mut cmd = TokioCommand::new("ssh");
        cmd.arg("-F")
            .arg("/dev/null")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("UserKnownHostsFile=/dev/null")
            .arg("-o")
            .arg("GlobalKnownHostsFile=/dev/null")
            .arg("-o")
            .arg("LogLevel=ERROR")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-p")
            .arg(self.port.to_string());

        if let Some(identity) = &self.identity_file {
            cmd.arg("-i").arg(identity);
            cmd.arg("-o").arg("IdentitiesOnly=yes");
            cmd.arg("-o").arg("IdentityAgent=none");
        }

        cmd.arg(format!("{}@127.0.0.1", self.user));
        cmd
    }
}

async fn run_with_timeout(mut cmd: TokioCommand, timeout_duration: Duration) -> io::Result<Output> {
    let output = timeout(timeout_duration, cmd.output())
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "ssh command timed out"))??;
    Ok(output)
}

pub async fn read_until_contains<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
    needle: &str,
    timeout_duration: Duration,
) -> io::Result<String> {
    use tokio::io::AsyncReadExt;

    let mut buffer = Vec::new();
    let mut scratch = [0u8; 1024];
    let deadline = tokio::time::Instant::now() + timeout_duration;

    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timeout waiting for output: {needle}"),
            ));
        }

        let read_result = timeout(deadline - now, reader.read(&mut scratch)).await;
        let read = match read_result {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => n,
            Ok(Err(err)) => return Err(err),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("timeout waiting for output: {needle}"),
                ));
            }
        };

        buffer.extend_from_slice(&scratch[..read]);
        let text = String::from_utf8_lossy(&buffer);
        let cleaned = strip_ansi(&text);
        if cleaned.contains(needle) {
            return Ok(cleaned);
        }
    }

    let text = String::from_utf8_lossy(&buffer);
    let cleaned = strip_ansi(&text);
    if cleaned.contains(needle) {
        Ok(cleaned)
    } else {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("output ended before finding: {needle}"),
        ))
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some(next) = chars.peek().copied() {
                if next == '[' {
                    chars.next();
                    for ctrl in chars.by_ref() {
                        if ('@'..='~').contains(&ctrl) {
                            break;
                        }
                    }
                    continue;
                } else if next == ']' {
                    chars.next();
                    let mut prev = '\0';
                    for ctrl in chars.by_ref() {
                        if ctrl == '\x07' {
                            break;
                        }
                        if prev == '\x1b' && ctrl == '\\' {
                            break;
                        }
                        prev = ctrl;
                    }
                    continue;
                }
            }
            continue;
        }

        out.push(ch);
    }

    out
}

pub fn handler_with_message(message: impl Into<String>) -> Handler {
    let message = message.into();
    handler(move |session| {
        let message = message.clone();
        async move {
            println(&session, &message);
            let _ = session.exit(0);
            let _ = session.close();
        }
    })
}
