use std::io::Write;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::io::AsyncWriteExt;

use super::common::{
    DEFAULT_TIMEOUT, LONG_TIMEOUT, SshClient, TestServer, read_until_contains, ssh_available,
};
use wish::{AcceptAllAuth, ServerBuilder};

#[tokio::test]
async fn test_pty_allocation_and_io() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_pty_allocation_and_io");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .handler(|session| async move {
                let (_, active) = session.pty();
                if active {
                    wish::println(&session, "PTY: yes");
                } else {
                    wish::println(&session, "PTY: no");
                }

                if let Some(data) = session.recv().await {
                    let _ = session.write(&data);
                }

                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let client = SshClient::new(server.port());
    let mut child = client.spawn_interactive().expect("ssh spawn");

    let mut stdout = child.stdout.take().expect("stdout");
    let mut stdin = child.stdin.take().expect("stdin");

    let _ = read_until_contains(&mut stdout, "PTY: yes", DEFAULT_TIMEOUT)
        .await
        .expect("pty output");

    stdin.write_all(b"ping\n").await.expect("write input");
    stdin.flush().await.expect("flush");

    let _ = read_until_contains(&mut stdout, "ping", DEFAULT_TIMEOUT)
        .await
        .expect("echo output");

    let status = tokio::time::timeout(DEFAULT_TIMEOUT, child.wait())
        .await
        .expect("wait timeout")
        .expect("wait failed");
    assert!(status.success(), "ssh exit status failed");

    server.stop().await;
}

#[tokio::test]
async fn test_window_resize() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_window_resize");
        return;
    }

    let started = Arc::new(AtomicBool::new(false));
    let last_size = Arc::new(Mutex::new(None));

    let model_started = started.clone();
    let model_last_size = last_size.clone();

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .with_middleware(wish::tea::middleware(move |_| {
                ResizeModel::new(model_started.clone(), model_last_size.clone())
            }))
            .handler(|session| async move {
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("openpty");

    let mut cmd = CommandBuilder::new("ssh");
    cmd.arg("-tt");
    cmd.arg("-F");
    cmd.arg("/dev/null");
    cmd.arg("-o");
    cmd.arg("StrictHostKeyChecking=no");
    cmd.arg("-o");
    cmd.arg("UserKnownHostsFile=/dev/null");
    cmd.arg("-o");
    cmd.arg("GlobalKnownHostsFile=/dev/null");
    cmd.arg("-o");
    cmd.arg("LogLevel=ERROR");
    cmd.arg("-o");
    cmd.arg("ConnectTimeout=5");
    cmd.arg("-p");
    cmd.arg(server.port().to_string());
    cmd.arg(format!("{}@127.0.0.1", super::common::TEST_USER));

    let mut child = pair.slave.spawn_command(cmd).expect("spawn ssh");
    drop(pair.slave);

    let mut writer = pair.master.take_writer().expect("pty writer");

    wait_for_flag(&started, LONG_TIMEOUT).await;

    pair.master
        .resize(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("resize pty");

    wait_for_resize(&last_size, (120, 40), LONG_TIMEOUT).await;

    writer.write_all(b"q").expect("write q");
    let _ = writer.flush();

    let _ = tokio::task::spawn_blocking(move || {
        let _ = child.kill();
        let _ = child.wait();
    })
    .await;

    server.stop().await;
}

async fn wait_for_flag(flag: &AtomicBool, timeout: std::time::Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if flag.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for program start");
}

async fn wait_for_resize(
    last_size: &Mutex<Option<(u16, u16)>>,
    expected: (u16, u16),
    timeout: std::time::Duration,
) {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if *last_size.lock().expect("size lock") == Some(expected) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for window resize");
}

#[derive(Clone)]
struct ResizeModel {
    started: Arc<AtomicBool>,
    last_size: Arc<Mutex<Option<(u16, u16)>>>,
}

impl ResizeModel {
    const fn new(started: Arc<AtomicBool>, last_size: Arc<Mutex<Option<(u16, u16)>>>) -> Self {
        Self { started, last_size }
    }
}

impl Model for ResizeModel {
    fn init(&self) -> Option<Cmd> {
        self.started.store(true, Ordering::SeqCst);
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            *self.last_size.lock().expect("size lock") = Some((size.width, size.height));
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(bubbletea::quit()),
                KeyType::Runes if key.runes == vec!['q'] => return Some(bubbletea::quit()),
                _ => {}
            }
        }

        None
    }

    fn view(&self) -> String {
        let size = *self.last_size.lock().expect("size lock");
        if let Some((width, height)) = size {
            format!("Size: {width}x{height}")
        } else {
            "Size: unknown".to_string()
        }
    }
}
