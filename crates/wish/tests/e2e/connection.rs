use super::common::{
    DEFAULT_TIMEOUT, LogCapture, SshClient, TestServer, handler_with_message, read_until_contains,
    ssh_available,
};
use wish::{AcceptAllAuth, ServerBuilder, middleware};

fn basic_builder() -> ServerBuilder {
    ServerBuilder::new()
        .auth_handler(AcceptAllAuth::new())
        .handler(|session| async move {
            wish::println(&session, "ok");
            let _ = session.exit(0);
            let _ = session.close();
        })
}

#[tokio::test]
async fn test_ssh_connection() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_ssh_connection");
        return;
    }

    let server = TestServer::start(basic_builder()).await;
    let client = SshClient::new(server.port());

    let output = client.exec("echo hello").await.expect("ssh exec");
    assert!(output.status.success(), "ssh exit status failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok"), "unexpected output: {stdout}");

    server.stop().await;
}

#[tokio::test]
async fn test_concurrent_connections() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_concurrent_connections");
        return;
    }

    let server = TestServer::start(basic_builder()).await;
    let port = server.port();

    let mut handles = Vec::new();
    for idx in 0..10 {
        handles.push(tokio::spawn(async move {
            let client = SshClient::new(port);
            let output = client
                .exec(&format!("echo conn{idx}"))
                .await
                .expect("ssh exec");
            assert!(output.status.success(), "conn {idx} failed");
        }));
    }

    for handle in handles {
        handle.await.expect("join");
    }

    server.stop().await;
}

#[tokio::test]
async fn test_graceful_shutdown() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_graceful_shutdown");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .handler(|session| async move {
                wish::println(&session, "bye");
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let client = SshClient::new(server.port());
    let mut child = client.spawn_interactive().await.expect("ssh spawn");

    let mut stdout = child.stdout.take().expect("stdout");
    let _ = read_until_contains(&mut stdout, "bye", DEFAULT_TIMEOUT)
        .await
        .expect("read output");

    let status = tokio::time::timeout(DEFAULT_TIMEOUT, child.wait())
        .await
        .expect("wait timeout")
        .expect("wait failed");
    assert!(status.success(), "ssh exit status not successful");

    server.stop().await;
}

#[tokio::test]
async fn test_log_capture() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_log_capture");
        return;
    }

    let logger = LogCapture::new();
    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .with_middleware(middleware::logging::middleware_with_logger(logger.clone()))
            .handler(handler_with_message("logged")),
    )
    .await;

    let client = SshClient::new(server.port());
    let output = client.exec("echo logged").await.expect("ssh exec");
    assert!(output.status.success(), "ssh exit status failed");

    let logs = logger.entries();
    assert!(
        logs.iter().any(|entry| entry.contains("connect")),
        "expected connect log entry, got: {logs:?}"
    );

    server.stop().await;
}
