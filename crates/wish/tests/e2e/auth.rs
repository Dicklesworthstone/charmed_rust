use std::io;
use std::path::{Path, PathBuf};

use super::common::{
    SshClient, TEST_USER, TestServer, handler_with_message, ssh_available, ssh_keygen_available,
};
use wish::{AuthorizedKeysAuth, PasswordAuth, ServerBuilder};

#[tokio::test]
async fn test_password_auth() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_password_auth");
        return;
    }

    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(PasswordAuth::new().add_user(TEST_USER, "secret"))
            .handler_arc(handler_with_message("password ok")),
    )
    .await;

    let client = SshClient::new(server.port());

    let output = client
        .exec_with_options(
            "echo should-fail",
            &["BatchMode=yes", "PreferredAuthentications=password"],
        )
        .await
        .expect("ssh exec");
    assert!(!output.status.success(), "ssh should fail without password");

    let output = client
        .exec_with_password("echo ok", "secret")
        .await
        .expect("ssh exec with password");
    assert!(output.status.success(), "ssh password auth failed");

    server.stop().await;
}

#[tokio::test]
async fn test_public_key_auth() {
    if !ssh_available() {
        eprintln!("ssh not available; skipping test_public_key_auth");
        return;
    }

    if !ssh_keygen_available() {
        eprintln!("ssh-keygen not available; skipping test_public_key_auth");
        return;
    }

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let key_path = generate_test_keypair(temp_dir.path()).expect("generate keypair");
    let pub_key_path = key_path.with_extension("pub");

    let auth = AuthorizedKeysAuth::new(&pub_key_path).expect("authorized keys");
    let server = TestServer::start(
        ServerBuilder::new()
            .auth_handler(auth)
            .handler_arc(handler_with_message("pubkey ok")),
    )
    .await;

    let client = SshClient::new(server.port()).with_identity_file(key_path);
    let output = client
        .exec_with_options(
            "echo ok",
            &[
                "BatchMode=yes",
                "PreferredAuthentications=publickey",
                "PubkeyAuthentication=yes",
            ],
        )
        .await
        .expect("ssh exec with pubkey");

    assert!(output.status.success(), "ssh pubkey auth failed");

    server.stop().await;
}

fn generate_test_keypair(dir: &Path) -> io::Result<PathBuf> {
    let key_path = dir.join("id_ed25519");
    let status = std::process::Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-N")
        .arg("")
        .arg("-f")
        .arg(&key_path)
        .arg("-C")
        .arg("wish-test")
        .status()?;

    if !status.success() {
        return Err(io::Error::other("ssh-keygen failed"));
    }

    Ok(key_path)
}
