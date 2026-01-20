use super::common::{DEFAULT_TIMEOUT, SshClient, TestServer, read_until_contains, ssh_available};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use tokio::io::AsyncWriteExt;
use wish::{AcceptAllAuth, ServerBuilder};

#[derive(Clone)]
struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(bubbletea::quit()),
                KeyType::Runes if key.runes == vec!['q'] => return Some(bubbletea::quit()),
                KeyType::Runes if key.runes == vec!['+'] => {
                    self.count += 1;
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

#[tokio::test]
async fn test_bubbletea_rendering() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_bubbletea_rendering");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(AcceptAllAuth::new())
            .with_middleware(wish::tea::middleware(|_| Counter::new()))
            .handler(|session| async move {
                let _ = session.exit(0);
                let _ = session.close();
            }),
    )
    .await;

    let client = SshClient::new(server.port());
    let mut child = client.spawn_interactive().await.expect("ssh spawn");

    let mut stdout = child.stdout.take().expect("stdout");
    let mut stdin = child.stdin.take().expect("stdin");

    let _ = read_until_contains(&mut stdout, "Count: 0", DEFAULT_TIMEOUT)
        .await
        .expect("initial render");

    stdin.write_all(b"+").await.expect("write +");
    stdin.flush().await.expect("flush");

    let _ = read_until_contains(&mut stdout, "Count: 1", DEFAULT_TIMEOUT)
        .await
        .expect("updated render");

    stdin.write_all(b"q").await.expect("write q");
    stdin.flush().await.expect("flush");

    let status = tokio::time::timeout(DEFAULT_TIMEOUT, child.wait())
        .await
        .expect("wait timeout")
        .expect("wait failed");
    assert!(status.success() || status.code() == Some(0));

    server.stop().await;
}
