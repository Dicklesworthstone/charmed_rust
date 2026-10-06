//! SCP middleware tests driven by an in-process russh client speaking the
//! SCP protocol (the same bytes `scp -O` exchanges).

#[allow(dead_code)]
mod common;

use common::{TestServer, exec};
use wish::ServerBuilder;
use wish::middleware::scp::{self, FileSystemHandler};

async fn start(root: &std::path::Path) -> TestServer {
    let builder = ServerBuilder::new()
        .allow_no_auth()
        .with_middleware(scp::middleware(FileSystemHandler::new(root)))
        .handler(|session| async move {
            wish::println(&session, "fallthrough");
            let _ = session.exit(0);
        });
    TestServer::start(builder).await
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_single_file_to_new_name() {
    let root = tempfile::tempdir().unwrap();
    let server = start(root.path()).await;

    // Record, acked; 5 data bytes + status byte, acked.
    let out = exec(
        server.port(),
        "scp -t /notes.txt",
        b"C0640 5 ignored.txt\nhello\0",
    )
    .await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    assert_eq!(out.stdout, b"\0\0\0", "one ack each: start, record, data");
    assert_eq!(
        std::fs::read(root.path().join("notes.txt")).unwrap(),
        b"hello"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(root.path().join("notes.txt"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o640);
    }
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_into_existing_directory_and_large_file() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("inbox")).unwrap();
    let server = start(root.path()).await;

    let payload: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
    let mut input = format!("C0644 {} big.bin\n", payload.len()).into_bytes();
    input.extend_from_slice(&payload);
    input.push(0);
    let out = exec(server.port(), "scp -t -- inbox", &input).await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    assert_eq!(
        std::fs::read(root.path().join("inbox/big.bin")).unwrap(),
        payload
    );
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_directory_recursively() {
    let root = tempfile::tempdir().unwrap();
    let server = start(root.path()).await;

    let input = b"D0755 0 project\nT1700000000 0 1700000000 0\nC0644 3 a.txt\nabc\0D0755 0 sub\nC0644 2 b.txt\nhi\0E\nE\n";
    let out = exec(server.port(), "scp -r -t /", input).await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    assert_eq!(
        std::fs::read(root.path().join("project/a.txt")).unwrap(),
        b"abc"
    );
    assert_eq!(
        std::fs::read(root.path().join("project/sub/b.txt")).unwrap(),
        b"hi"
    );
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn download_file() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("down.txt"), b"world").unwrap();
    let server = start(root.path()).await;

    // Client acks: ready, record, data.
    let out = exec(server.port(), "scp -f /down.txt", b"\0\0\0").await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    let stdout = out.stdout();
    assert!(stdout.starts_with("C0"), "{stdout:?}");
    assert!(stdout.ends_with(" 5 down.txt\nworld\0"), "{stdout:?}");
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn download_directory_with_times() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("docs")).unwrap();
    std::fs::write(root.path().join("docs/one.md"), b"1").unwrap();
    let server = start(root.path()).await;

    let out = exec(server.port(), "scp -rpf docs", &[0u8; 16]).await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    let stdout = out.stdout();
    let records: Vec<&str> = stdout
        .split('\n')
        .filter(|l| l.starts_with(['T', 'D', 'C', 'E']))
        .collect();
    assert!(records[0].starts_with('T'), "{records:?}");
    assert!(
        records[1].starts_with('D') && records[1].ends_with(" 0 docs"),
        "{records:?}"
    );
    assert!(stdout.contains(" 1 one.md\n1\0"), "{stdout:?}");
    assert!(stdout.ends_with("E\n"), "{stdout:?}");
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_escapes_and_bad_requests() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("dir")).unwrap();
    let server = start(root.path()).await;

    let out = exec(server.port(), "scp -t ../outside.txt", b"C0644 1 x\nx\0").await;
    assert_eq!(out.exit, Some(1));
    assert!(
        out.stdout().contains("permission denied"),
        "{:?}",
        out.stdout()
    );

    let out = exec(server.port(), "scp -t /", b"C0644 1 ../evil\nx\0").await;
    assert_eq!(out.exit, Some(1));
    assert!(!root.path().parent().unwrap().join("evil").exists());

    let out = exec(server.port(), "scp -f dir", b"\0").await;
    assert_eq!(out.exit, Some(1), "directories need -r");
    assert!(
        out.stdout().contains("not a regular file"),
        "{:?}",
        out.stdout()
    );

    let out = exec(server.port(), "scp -f missing.txt", b"\0").await;
    assert_eq!(out.exit, Some(1));
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn non_scp_sessions_fall_through() {
    let root = tempfile::tempdir().unwrap();
    let server = start(root.path()).await;
    let out = exec(server.port(), "ls", b"").await;
    assert!(out.stdout().contains("fallthrough"), "{}", out.stdout());
    server.stop().await;
}
