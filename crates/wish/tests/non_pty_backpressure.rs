//! Regression tests for `charmed_rust#99`:
//! - Non-PTY sessions whose handler exits (e.g. default bubbletea middleware)
//!   or ignores stdin must NOT freeze the SSH connection when the client sends
//!   more than the input channel capacity.
//! - Active non-PTY stream readers must still receive all bytes losslessly.

#[allow(dead_code)]
mod common;

use std::sync::Arc;
use std::time::Duration;

use common::TestServer;
use russh::client;
use russh::keys::PublicKeyOrCertificate;
use russh::{ChannelMsg, Disconnect};
use wish::ServerBuilder;

struct AcceptAll;

impl client::Handler for AcceptAll {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> impl std::future::Future<Output = Result<bool, Self::Error>> + Send {
        std::future::ready(Ok(true))
    }
}

async fn connect_no_auth(port: u16) -> client::Handle<AcceptAll> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(config, ("127.0.0.1", port), AcceptAll)
        .await
        .expect("connect to test server");
    let auth = handle.authenticate_none("tester").await.expect("auth_none");
    assert!(auth.success(), "auth_none should succeed");
    handle
}

/// Exact #99 reproducer: open a non-PTY session against a handler that exits
/// immediately without reading stdin (like `bubbletea::middleware` when no PTY
/// is allocated) and send >1,024 1-byte packets. The SSH connection must
/// remain responsive and deliver stdout + exit status + close promptly.
#[tokio::test(flavor = "multi_thread")]
async fn non_pty_flood_after_handler_exit_does_not_freeze_connection() {
    let server = TestServer::start(
        ServerBuilder::new()
            .allow_no_auth()
            .with_middleware(wish::middleware::activeterm::middleware())
            .handler(|session| async move {
                wish::println(&session, "should not run without pty");
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let session = connect_no_auth(server.port()).await;
    let channel = session.channel_open_session().await.expect("open channel");
    channel
        .exec(true, "ignored-stdin")
        .await
        .expect("exec request");

    // Blast 2,000 separate 1-byte SSH data packets (well above both 256 and 1,024).
    let (mut read_half, write_half) = channel.split();
    let sender = tokio::spawn(async move {
        for _ in 0..2_000 {
            if write_half.data(&b"x"[..]).await.is_err() {
                break;
            }
        }
        let _ = write_half.eof().await;
    });

    let mut stdout = Vec::new();
    let mut exit_status = None;
    let collect = async {
        while let Some(msg) = read_half.wait().await {
            match msg {
                ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                ChannelMsg::ExitStatus { exit_status: code } => exit_status = Some(code),
                ChannelMsg::Close => break,
                _ => {}
            }
        }
    };

    tokio::time::timeout(Duration::from_secs(5), collect)
        .await
        .expect("SSH connection must not freeze under non-PTY input flood (#99)");
    let _ = tokio::time::timeout(Duration::from_secs(2), sender).await;

    assert_eq!(exit_status, Some(1));
    assert!(
        String::from_utf8_lossy(&stdout).contains("PTY"),
        "expected PTY rejection message, got: {:?}",
        String::from_utf8_lossy(&stdout)
    );

    let _ = session
        .disconnect(Disconnect::ByApplication, "", "en")
        .await;
    server.stop().await;
}

/// Secondary #99 reproducer: handler stays alive (without reading stdin) while
/// the client floods >1,024 1-byte packets. The connection must not wedge; the
/// handler's output and exit status must still arrive on time.
#[tokio::test(flavor = "multi_thread")]
async fn non_pty_flood_while_handler_ignores_stdin_does_not_freeze() {
    let server = TestServer::start(ServerBuilder::new().allow_no_auth().handler(
        |session| async move {
            tokio::time::sleep(Duration::from_millis(400)).await;
            wish::println(&session, "completed-without-stdin");
            let _ = session.exit(0);
            let _ = session.close();
        },
    ))
    .await;

    let session = connect_no_auth(server.port()).await;
    let channel = session.channel_open_session().await.expect("open channel");
    channel
        .exec(true, "slow-no-stdin")
        .await
        .expect("exec request");

    let (mut read_half, write_half) = channel.split();
    let sender = tokio::spawn(async move {
        for _ in 0..2_000 {
            if write_half.data(&b"z"[..]).await.is_err() {
                break;
            }
        }
        let _ = write_half.eof().await;
    });

    let mut stdout = Vec::new();
    let mut exit_status = None;
    let collect = async {
        while let Some(msg) = read_half.wait().await {
            match msg {
                ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                ChannelMsg::ExitStatus { exit_status: code } => exit_status = Some(code),
                ChannelMsg::Close => break,
                _ => {}
            }
        }
    };

    tokio::time::timeout(Duration::from_secs(5), collect)
        .await
        .expect("slow non-reading handler must not freeze SSH connection (#99)");
    let _ = tokio::time::timeout(Duration::from_secs(2), sender).await;

    assert_eq!(exit_status, Some(0));
    assert!(
        String::from_utf8_lossy(&stdout).contains("completed-without-stdin"),
        "stdout: {:?}",
        String::from_utf8_lossy(&stdout)
    );

    let _ = session
        .disconnect(Disconnect::ByApplication, "", "en")
        .await;
    server.stop().await;
}

/// Active non-PTY streaming reader (like git-receive-pack or scp -t) receives
/// every byte across >1,000 packets with zero loss.
#[tokio::test(flavor = "multi_thread")]
async fn non_pty_active_reader_receives_all_bytes_losslessly() {
    let server = TestServer::start(ServerBuilder::new().allow_no_auth().handler(
        |session| async move {
            let mut total = 0usize;
            let mut checksum: u64 = 0;
            while let Some(chunk) = session.recv().await {
                total += chunk.len();
                for &b in &chunk {
                    checksum = checksum.wrapping_add(u64::from(b));
                }
                tokio::task::yield_now().await;
            }
            let _ = session
                .write_async(format!("{total}:{checksum}\n").as_bytes())
                .await;
            let _ = session.exit(0);
            let _ = session.close();
        },
    ))
    .await;

    let session = connect_no_auth(server.port()).await;
    let channel = session.channel_open_session().await.expect("open channel");
    channel.exec(true, "stream-sum").await.expect("exec");

    let chunk = vec![42u8; 256];
    let chunks = 1_500usize;
    let expected_total = chunk.len() * chunks;
    let expected_checksum: u64 = (expected_total as u64) * 42;

    let (mut read_half, write_half) = channel.split();
    let sender = tokio::spawn(async move {
        for _ in 0..chunks {
            write_half.data(&chunk[..]).await.expect("send chunk");
        }
        write_half.eof().await.expect("send eof");
    });

    let mut stdout = Vec::new();
    let mut exit_status = None;
    let collect = async {
        while let Some(msg) = read_half.wait().await {
            match msg {
                ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                ChannelMsg::ExitStatus { exit_status: code } => exit_status = Some(code),
                ChannelMsg::Close => break,
                _ => {}
            }
        }
    };

    tokio::time::timeout(Duration::from_secs(10), collect)
        .await
        .expect("streaming reader timed out");
    sender.await.expect("sender task");

    assert_eq!(exit_status, Some(0));
    assert_eq!(
        String::from_utf8_lossy(&stdout).trim(),
        format!("{expected_total}:{expected_checksum}")
    );

    let _ = session
        .disconnect(Disconnect::ByApplication, "", "en")
        .await;
    server.stop().await;
}
