#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

//! Unit tests for wish types: Window, Pty, PublicKey, Context, Session.
//! These test the public API without requiring an actual SSH connection.

use std::net::SocketAddr;
use wish::{Context, Pty, PublicKey, Session, Window};

// =============================================================================
// Window
// =============================================================================

#[test]
fn window_default() {
    let w = Window::default();
    assert_eq!(w.width, 80);
    assert_eq!(w.height, 24);
}

#[test]
fn window_custom() {
    let w = Window {
        width: 120,
        height: 40,
    };
    assert_eq!(w.width, 120);
    assert_eq!(w.height, 40);
}

#[test]
fn window_copy() {
    let w = Window {
        width: 100,
        height: 50,
    };
    let w2 = w;
    assert_eq!(w.width, w2.width);
    assert_eq!(w.height, w2.height);
}

// =============================================================================
// Pty
// =============================================================================

#[test]
fn pty_default() {
    let p = Pty::default();
    assert_eq!(p.term, "xterm-256color");
    assert_eq!(p.window.width, 80);
    assert_eq!(p.window.height, 24);
}

#[test]
fn pty_custom_term() {
    let p = Pty {
        term: "screen".to_string(),
        window: Window {
            width: 132,
            height: 43,
        },
    };
    assert_eq!(p.term, "screen");
    assert_eq!(p.window.width, 132);
}

#[test]
fn pty_clone() {
    let p = Pty::default();
    let p2 = p.clone();
    assert_eq!(p.term, p2.term);
    assert_eq!(p.window.width, p2.window.width);
}

// =============================================================================
// PublicKey
// =============================================================================

#[test]
fn public_key_new() {
    let pk = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    assert_eq!(pk.key_type, "ssh-ed25519");
    assert_eq!(pk.data, vec![1, 2, 3]);
    assert!(pk.comment.is_none());
}

#[test]
fn public_key_with_comment() {
    let pk = PublicKey::new("ssh-rsa", vec![4, 5, 6]).with_comment("user@host");
    assert_eq!(pk.comment, Some("user@host".to_string()));
}

#[test]
fn public_key_fingerprint() {
    let pk = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let fp = pk.fingerprint();
    assert!(fp.starts_with("SHA256:"));
    assert_eq!(fp.len(), "SHA256:".len() + 16);
}

#[test]
fn public_key_fingerprint_deterministic() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let pk2 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    assert_eq!(pk1.fingerprint(), pk2.fingerprint());
}

#[test]
fn public_key_different_data_different_fingerprint() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let pk2 = PublicKey::new("ssh-ed25519", vec![4, 5, 6]);
    assert_ne!(pk1.fingerprint(), pk2.fingerprint());
}

#[test]
fn public_key_equality() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let pk2 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    assert_eq!(pk1, pk2);
}

#[test]
fn public_key_equality_ignores_comment() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]).with_comment("alice");
    let pk2 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]).with_comment("bob");
    assert_eq!(pk1, pk2);
}

#[test]
fn public_key_inequality_different_type() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let pk2 = PublicKey::new("ssh-rsa", vec![1, 2, 3]);
    assert_ne!(pk1, pk2);
}

#[test]
fn public_key_inequality_different_data() {
    let pk1 = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let pk2 = PublicKey::new("ssh-ed25519", vec![1, 2, 4]);
    assert_ne!(pk1, pk2);
}

// =============================================================================
// Context
// =============================================================================

fn test_addr() -> SocketAddr {
    "127.0.0.1:22".parse().unwrap()
}

fn test_remote() -> SocketAddr {
    "192.168.1.100:54321".parse().unwrap()
}

#[test]
fn context_new() {
    let ctx = Context::new("alice", test_remote(), test_addr());
    assert_eq!(ctx.user(), "alice");
    assert_eq!(ctx.remote_addr(), test_remote());
    assert_eq!(ctx.local_addr(), test_addr());
    assert_eq!(ctx.client_version(), "");
}

#[test]
fn context_client_version() {
    let mut ctx = Context::new("bob", test_remote(), test_addr());
    ctx.set_client_version("SSH-2.0-OpenSSH_9.0");
    assert_eq!(ctx.client_version(), "SSH-2.0-OpenSSH_9.0");
}

#[test]
fn context_custom_values() {
    let ctx = Context::new("charlie", test_remote(), test_addr());
    assert!(ctx.get_value("role").is_none());

    ctx.set_value("role", "admin");
    assert_eq!(ctx.get_value("role"), Some("admin".to_string()));

    ctx.set_value("role", "user");
    assert_eq!(ctx.get_value("role"), Some("user".to_string()));
}

#[test]
fn context_multiple_values() {
    let ctx = Context::new("dave", test_remote(), test_addr());
    ctx.set_value("key1", "val1");
    ctx.set_value("key2", "val2");
    assert_eq!(ctx.get_value("key1"), Some("val1".to_string()));
    assert_eq!(ctx.get_value("key2"), Some("val2".to_string()));
}

#[test]
fn context_clone_shares_values() {
    let ctx = Context::new("eve", test_remote(), test_addr());
    ctx.set_value("shared", "yes");
    let ctx2 = ctx.clone();
    // Arc<RwLock<HashMap>> is shared
    assert_eq!(ctx2.get_value("shared"), Some("yes".to_string()));
}

// =============================================================================
// Session
// =============================================================================

fn test_session() -> Session {
    let ctx = Context::new("testuser", test_remote(), test_addr());
    Session::new(ctx)
}

#[test]
fn session_user() {
    let s = test_session();
    assert_eq!(s.user(), "testuser");
}

#[test]
fn session_addresses() {
    let s = test_session();
    assert_eq!(s.remote_addr(), test_remote());
    assert_eq!(s.local_addr(), test_addr());
}

#[test]
fn session_no_pty_by_default() {
    let s = test_session();
    let (pty, has_pty) = s.pty();
    assert!(pty.is_none());
    assert!(!has_pty);
}

#[test]
fn session_with_pty() {
    let s = test_session().with_pty(Pty::default());
    let (pty, has_pty) = s.pty();
    assert!(pty.is_some());
    assert!(has_pty);
    assert_eq!(pty.unwrap().term, "xterm-256color");
}

#[test]
fn session_window_default_without_pty() {
    let s = test_session();
    let w = s.window();
    assert_eq!(w.width, 80);
    assert_eq!(w.height, 24);
}

#[test]
fn session_window_from_pty() {
    let pty = Pty {
        term: "xterm".to_string(),
        window: Window {
            width: 120,
            height: 40,
        },
    };
    let s = test_session().with_pty(pty);
    let w = s.window();
    assert_eq!(w.width, 120);
    assert_eq!(w.height, 40);
}

#[test]
fn session_command() {
    let s = test_session().with_command(vec!["ls".to_string(), "-la".to_string()]);
    assert_eq!(s.command(), &["ls", "-la"]);
}

#[test]
fn session_empty_command() {
    let s = test_session();
    assert!(s.command().is_empty());
}

#[test]
fn session_env() {
    let s = test_session()
        .with_env("TERM", "xterm")
        .with_env("LANG", "en_US.UTF-8");
    assert_eq!(s.get_env("TERM"), Some(&"xterm".to_string()));
    assert_eq!(s.get_env("LANG"), Some(&"en_US.UTF-8".to_string()));
    assert!(s.get_env("MISSING").is_none());
    assert_eq!(s.environ().len(), 2);
}

#[test]
fn session_public_key() {
    let pk = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let s = test_session().with_public_key(pk.clone());
    assert_eq!(s.public_key(), Some(&pk));
}

#[test]
fn session_no_public_key() {
    let s = test_session();
    assert!(s.public_key().is_none());
}

#[test]
fn session_subsystem() {
    let s = test_session().with_subsystem("sftp");
    assert_eq!(s.subsystem(), Some("sftp"));
}

#[test]
fn session_no_subsystem() {
    let s = test_session();
    assert!(s.subsystem().is_none());
}

#[test]
fn session_close() {
    let s = test_session();
    assert!(!s.is_closed());
    s.close().unwrap();
    assert!(s.is_closed());
}

#[test]
fn session_exit() {
    let s = test_session();
    s.exit(0).unwrap();
    // No panic, exit code stored internally
}

#[test]
fn session_write_without_sender() {
    let s = test_session();
    // Write without output sender — should succeed (data goes nowhere)
    let result = s.write(b"hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn session_write_stderr_without_sender() {
    let s = test_session();
    let result = s.write_stderr(b"error");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn session_context_accessor() {
    let s = test_session();
    assert_eq!(s.context().user(), "testuser");
}

#[test]
fn session_debug_format() {
    let s = test_session();
    let debug = format!("{s:?}");
    assert!(debug.contains("testuser"));
    assert!(debug.contains("Session"));
}

#[test]
fn session_builder_chain() {
    let pk = PublicKey::new("ssh-ed25519", vec![1, 2, 3]);
    let s = test_session()
        .with_pty(Pty::default())
        .with_command(vec!["bash".to_string()])
        .with_env("HOME", "/home/test")
        .with_public_key(pk)
        .with_subsystem("sftp");

    assert!(s.pty().1);
    assert_eq!(s.command(), &["bash"]);
    assert_eq!(s.get_env("HOME"), Some(&"/home/test".to_string()));
    assert!(s.public_key().is_some());
    assert_eq!(s.subsystem(), Some("sftp"));
}
