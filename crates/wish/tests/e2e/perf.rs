use std::io::Write;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Instant;

use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

use super::common::{LONG_TIMEOUT, SshClient, TEST_USER, TestServer, ssh_available};
use wish::{AcceptAllAuth, ServerBuilder};

#[tokio::test]
#[ignore = "expensive performance test - run manually"]
async fn test_connection_throughput() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_connection_throughput");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .handler(|session| async move {
                wish::println(&session, "ok");
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let start = Instant::now();
    let connections = 50;
    let mut handles = Vec::new();

    for _ in 0..connections {
        let port = server.port();
        handles.push(tokio::spawn(async move {
            let client = SshClient::new(port);
            let output = client.exec("echo ok").await.expect("ssh exec");
            assert!(output.status.success(), "connection failed");
        }));
    }

    for handle in handles {
        handle.await.expect("join");
    }

    let elapsed = start.elapsed();
    let rate = f64::from(connections) / elapsed.as_secs_f64();
    eprintln!("connection rate: {rate:.2} conn/sec");
    assert!(rate > 5.0, "connection rate too low");

    server.stop().await;
}

/// Stress test: rapid sequential open/close cycles.
///
/// Tests server stability under rapid connection churn.
#[tokio::test]
#[ignore = "expensive performance test - run manually"]
async fn test_rapid_open_close_cycles() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_rapid_open_close_cycles");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .handler(|session| async move {
                wish::println(&session, "ok");
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let cycles = 30;
    let start = Instant::now();
    let client = SshClient::new(server.port());

    for i in 0..cycles {
        let output = client.exec("echo cycle").await.expect("ssh exec");
        assert!(
            output.status.success(),
            "cycle {i} failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let elapsed = start.elapsed();
    let rate = f64::from(cycles) / elapsed.as_secs_f64();
    eprintln!(
        "sequential open/close: {} cycles in {:.2}s ({:.2} cycles/sec)",
        cycles,
        elapsed.as_secs_f64(),
        rate
    );
    assert!(rate > 1.0, "open/close rate too low: {rate:.2}/sec");

    server.stop().await;
}

/// Stress test: rapid PTY window resize events.
///
/// Tests server stability when receiving many resize events in quick succession.
#[tokio::test]
#[ignore = "expensive performance test - run manually"]
async fn test_rapid_pty_resize() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_rapid_pty_resize");
        return;
    }

    let resize_count = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(AtomicBool::new(false));

    let model_resize_count = resize_count.clone();
    let model_started = started.clone();

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .with_middleware(wish::tea::middleware(move |_| {
                ResizeCounterModel::new(model_started.clone(), model_resize_count.clone())
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
    cmd.arg(format!("{TEST_USER}@127.0.0.1"));

    let mut child = pair.slave.spawn_command(cmd).expect("spawn ssh");
    drop(pair.slave);

    let mut writer = pair.master.take_writer().expect("pty writer");

    // Wait for program to start
    wait_for_flag(&started, LONG_TIMEOUT).await;

    // Send rapid resize events
    let resize_count_target: u16 = 20;
    for i in 0..resize_count_target {
        let rows = 24 + (i % 10);
        let cols = 80 + (i % 20);
        pair.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("resize pty");

        // Small delay to allow resize to propagate
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Allow final resizes to process
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let final_count = resize_count.load(Ordering::SeqCst);
    eprintln!("rapid resize: sent {resize_count_target} resize events, received {final_count}");

    // We should receive at least some resize events (may be coalesced)
    assert!(
        final_count >= 1,
        "expected at least 1 resize event, got {final_count}"
    );

    // Quit the application
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

#[derive(Clone)]
struct ResizeCounterModel {
    started: Arc<AtomicBool>,
    resize_count: Arc<AtomicUsize>,
    last_size: Arc<Mutex<Option<(u16, u16)>>>,
}

impl ResizeCounterModel {
    fn new(started: Arc<AtomicBool>, resize_count: Arc<AtomicUsize>) -> Self {
        Self {
            started,
            resize_count,
            last_size: Arc::new(Mutex::new(None)),
        }
    }
}

impl Model for ResizeCounterModel {
    fn init(&self) -> Option<Cmd> {
        self.started.store(true, Ordering::SeqCst);
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            let new_size = (size.width, size.height);
            let mut last = self.last_size.lock().expect("size lock");
            if *last != Some(new_size) {
                self.resize_count.fetch_add(1, Ordering::SeqCst);
                *last = Some(new_size);
            }
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
        let count = self.resize_count.load(Ordering::SeqCst);
        format!("Resize count: {count}")
    }
}

/// Stress test: multiple concurrent PTY sessions.
///
/// Tests server stability with multiple simultaneous interactive sessions.
#[tokio::test]
#[ignore = "expensive performance test - run manually"]
async fn test_concurrent_pty_sessions() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_concurrent_pty_sessions");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .handler(|session| async move {
                let (_, active) = session.pty();
                if active {
                    wish::println(&session, "PTY active");
                }
                // Wait for input before closing
                if let Some(data) = session.recv().await {
                    wish::println(&session, format!("got: {}", data.len()));
                }
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let concurrent = 5;
    let mut handles = Vec::new();

    for i in 0..concurrent {
        let port = server.port();
        handles.push(tokio::spawn(async move {
            use super::common::{DEFAULT_TIMEOUT, read_until_contains};
            use tokio::io::AsyncWriteExt;

            let client = SshClient::new(port);
            let mut child = client.spawn_interactive().expect("ssh spawn");

            let mut stdout = child.stdout.take().expect("stdout");
            let mut stdin = child.stdin.take().expect("stdin");

            let _ = read_until_contains(&mut stdout, "PTY active", DEFAULT_TIMEOUT)
                .await
                .expect("pty output");

            stdin.write_all(b"x\n").await.expect("write input");
            stdin.flush().await.expect("flush");

            let status = tokio::time::timeout(DEFAULT_TIMEOUT, child.wait())
                .await
                .expect("wait timeout")
                .expect("wait failed");
            assert!(status.success(), "session {i} failed");
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        handle.await.unwrap_or_else(|_| panic!("join session {i}"));
    }

    eprintln!("concurrent PTY sessions: {concurrent} sessions completed successfully");

    server.stop().await;
}
