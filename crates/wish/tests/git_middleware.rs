//! Git-over-SSH middleware tests driven by an in-process russh client.
//!
//! These exercise the real `git` executable on the server side (skipped when
//! git is not installed) and speak the git wire protocol directly, so no
//! `ssh` client binary is needed.

#[allow(dead_code)]
mod common;

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use common::TestServer;
use russh::client;
use russh::keys::PublicKeyOrCertificate;
use russh::{ChannelMsg, Disconnect};
use wish::middleware::git::{self, AccessLevel, Hooks, StaticAccess};
use wish::{PublicKey, ServerBuilder};

struct Client;

impl client::Handler for Client {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        _key: &PublicKeyOrCertificate,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        std::future::ready(Ok(true))
    }
}

/// Output of one exec channel.
#[derive(Debug, Default)]
struct ExecResult {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: Option<u32>,
}

impl ExecResult {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// Runs `command`, sends `input` followed by EOF, and collects the output.
async fn exec(port: u16, command: &str, input: &[u8]) -> ExecResult {
    let config = Arc::new(client::Config::default());
    let mut session = client::connect(config, ("127.0.0.1", port), Client)
        .await
        .expect("connect");
    let auth = session.authenticate_none("tester").await.expect("auth");
    assert!(auth.success(), "none auth should be accepted");
    let mut channel = session.channel_open_session().await.expect("open session");
    channel.exec(true, command).await.expect("exec");
    if !input.is_empty() {
        channel.data(input).await.expect("send input");
    }
    channel.eof().await.expect("eof");

    let mut result = ExecResult::default();
    let collect = async {
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => result.stdout.extend_from_slice(&data),
                ChannelMsg::ExtendedData { data, .. } => result.stderr.extend_from_slice(&data),
                ChannelMsg::ExitStatus { exit_status } => result.exit = Some(exit_status),
                ChannelMsg::Close => break,
                _ => {}
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(20), collect)
        .await
        .expect("exec timed out");
    let _ = session
        .disconnect(Disconnect::ByApplication, "", "en")
        .await;
    result
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?} failed");
}

/// Creates `<repos>/<name>.git` containing one commit on `main`.
fn seed_repo(repos: &Path, name: &str) {
    let work = tempfile::tempdir().expect("worktree");
    git(work.path(), &["init", "--quiet", "-b", "main"]);
    std::fs::write(work.path().join("README.md"), "# hello\n").unwrap();
    git(work.path(), &["add", "."]);
    git(work.path(), &["commit", "--quiet", "-m", "init"]);
    let dest = repos.join(format!("{name}.git"));
    git(
        work.path(),
        &["clone", "--quiet", "--bare", ".", dest.to_str().unwrap()],
    );
}

async fn start(repos: &Path, hooks: impl Hooks + 'static) -> TestServer {
    let builder = ServerBuilder::new()
        .allow_no_auth()
        .with_middleware(git::middleware(repos.to_path_buf(), hooks))
        .handler(|session| async move {
            wish::println(&session, "fallthrough");
            let _ = session.exit(0);
        });
    TestServer::start(builder).await
}

/// A flush-only pkt-line request ("0000") ends a fetch/push negotiation
/// without transferring anything.
const FLUSH: &[u8] = b"0000";

#[tokio::test(flavor = "multi_thread")]
async fn upload_pack_advertises_refs() {
    if !git_available() {
        eprintln!("skipping: git not installed");
        return;
    }
    let repos = tempfile::tempdir().unwrap();
    seed_repo(repos.path(), "demo");
    let server = start(repos.path(), StaticAccess(AccessLevel::ReadOnly)).await;

    // Both "demo" and "/demo.git" resolve to demo.git.
    for name in ["'demo'", "'/demo.git'"] {
        let out = exec(server.port(), &format!("git-upload-pack {name}"), FLUSH).await;
        assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
        assert!(out.stdout().contains("refs/heads/main"), "{}", out.stdout());
    }
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn receive_pack_creates_bare_repo_on_first_push() {
    if !git_available() {
        eprintln!("skipping: git not installed");
        return;
    }
    let repos = tempfile::tempdir().unwrap();
    let server = start(repos.path(), StaticAccess(AccessLevel::ReadWrite)).await;

    let out = exec(server.port(), "git-receive-pack 'fresh'", FLUSH).await;
    assert_eq!(out.exit, Some(0), "stderr: {}", out.stderr());
    // An empty repository advertises the capabilities pseudo-ref.
    assert!(out.stdout().contains("capabilities^{}"), "{}", out.stdout());
    assert!(repos.path().join("fresh.git").join("HEAD").is_file());
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn access_levels_are_enforced() {
    if !git_available() {
        eprintln!("skipping: git not installed");
        return;
    }
    let repos = tempfile::tempdir().unwrap();
    seed_repo(repos.path(), "demo");

    let read_only = start(repos.path(), StaticAccess(AccessLevel::ReadOnly)).await;
    let push = exec(read_only.port(), "git-receive-pack 'demo'", FLUSH).await;
    assert_eq!(push.exit, Some(1));
    assert!(
        push.stderr().contains(git::ERR_NOT_AUTHED),
        "{}",
        push.stderr()
    );
    read_only.stop().await;

    let none = start(repos.path(), StaticAccess(AccessLevel::NoAccess)).await;
    let fetch = exec(none.port(), "git-upload-pack 'demo'", FLUSH).await;
    assert_eq!(fetch.exit, Some(1));
    assert!(
        fetch.stderr().contains(git::ERR_NOT_AUTHED),
        "{}",
        fetch.stderr()
    );
    none.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_and_missing_repos_are_rejected() {
    if !git_available() {
        eprintln!("skipping: git not installed");
        return;
    }
    let repos = tempfile::tempdir().unwrap();
    let server = start(repos.path(), StaticAccess(AccessLevel::Admin)).await;

    for command in [
        "git-upload-pack '../etc'",
        "git-upload-pack 'a/b'",
        "git-upload-pack 'missing'",
    ] {
        let out = exec(server.port(), command, FLUSH).await;
        assert_eq!(out.exit, Some(1), "{command}");
        assert!(
            out.stderr().contains(git::ERR_INVALID_REPO),
            "{command}: {}",
            out.stderr()
        );
    }
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn non_git_sessions_fall_through() {
    let repos = tempfile::tempdir().unwrap();
    let server = start(repos.path(), StaticAccess(AccessLevel::Admin)).await;
    let out = exec(server.port(), "echo hi", b"").await;
    assert!(out.stdout().contains("fallthrough"), "{}", out.stdout());
    server.stop().await;
}

#[derive(Default)]
struct CountingHooks {
    fetches: AtomicUsize,
    pushes: AtomicUsize,
}

/// Shares the counters with the test.
struct Counting(Arc<CountingHooks>);

impl Hooks for Counting {
    fn auth_repo(&self, _repo: &str, _key: Option<&PublicKey>) -> AccessLevel {
        AccessLevel::ReadWrite
    }

    fn push(&self, _repo: &str, _key: Option<&PublicKey>) {
        self.0.pushes.fetch_add(1, Ordering::SeqCst);
    }

    fn fetch(&self, _repo: &str, _key: Option<&PublicKey>) {
        self.0.fetches.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn hooks_are_notified() {
    if !git_available() {
        eprintln!("skipping: git not installed");
        return;
    }
    let repos = tempfile::tempdir().unwrap();
    seed_repo(repos.path(), "demo");
    let hooks = Arc::new(CountingHooks::default());
    let server = start(repos.path(), Counting(Arc::clone(&hooks))).await;

    exec(server.port(), "git-upload-pack 'demo'", FLUSH).await;
    exec(server.port(), "git-receive-pack 'demo'", FLUSH).await;
    assert_eq!(hooks.fetches.load(Ordering::SeqCst), 1);
    assert_eq!(hooks.pushes.load(Ordering::SeqCst), 1);
    server.stop().await;
}
