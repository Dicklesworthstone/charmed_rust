use std::time::Instant;

use super::common::{SshClient, TestServer, ssh_available};
use wish::{AcceptAllAuth, ServerBuilder};

#[tokio::test]
#[ignore]
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
    let rate = connections as f64 / elapsed.as_secs_f64();
    eprintln!("connection rate: {:.2} conn/sec", rate);
    assert!(rate > 5.0, "connection rate too low");

    server.stop().await;
}
