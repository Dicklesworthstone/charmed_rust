//! Authorized keys file parsing and authentication.
//!
//! Parses OpenSSH `authorized_keys` files and provides authentication
//! against the keys contained within.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use tracing::{debug, info};

use super::handler::{AuthContext, AuthHandler, AuthMethod, AuthResult};
use crate::PublicKey;

/// An authorized key parsed from an authorized_keys file.
#[derive(Debug, Clone)]
pub struct AuthorizedKey {
    /// The key type (e.g., "ssh-ed25519", "ssh-rsa").
    pub key_type: String,
    /// The base64-encoded key data.
    pub key_data: String,
    /// The decoded key bytes.
    pub key_bytes: Vec<u8>,
    /// Optional comment (usually email or identifier).
    pub comment: Option<String>,
    /// Optional key options (e.g., "command=", "no-pty").
    pub options: Vec<String>,
}

impl AuthorizedKey {
    /// Converts this to a PublicKey for comparison.
    pub fn to_public_key(&self) -> PublicKey {
        PublicKey::new(&self.key_type, self.key_bytes.clone())
            .with_comment(self.comment.clone().unwrap_or_default())
    }

    /// Checks if this key matches the given public key.
    pub fn matches(&self, key: &PublicKey) -> bool {
        self.key_type == key.key_type && self.key_bytes == key.data
    }
}

/// Parses an authorized_keys file and returns the keys.
///
/// # Format
///
/// The authorized_keys file format is:
/// ```text
/// [options] key-type base64-data [comment]
/// ```
///
/// For example:
/// ```text
/// ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA... user@example.com
/// no-pty,command="/usr/bin/git-shell" ssh-rsa AAAAB3... git@server
/// ```
///
/// # Arguments
///
/// * `content` - The content of the authorized_keys file.
///
/// # Returns
///
/// A vector of parsed authorized keys.
pub fn parse_authorized_keys(content: &str) -> Vec<AuthorizedKey> {
    let mut keys = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(key) = parse_authorized_key_line(line) {
            keys.push(key);
        } else {
            debug!(line = %line, "Failed to parse authorized_keys line");
        }
    }

    keys
}

/// Parses a single line from an authorized_keys file.
fn parse_authorized_key_line(line: &str) -> Option<AuthorizedKey> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    // Key types we recognize
    const KEY_TYPES: &[&str] = &[
        "ssh-ed25519",
        "ssh-rsa",
        "ecdsa-sha2-nistp256",
        "ecdsa-sha2-nistp384",
        "ecdsa-sha2-nistp521",
        "ssh-dss",
        "sk-ssh-ed25519@openssh.com",
        "sk-ecdsa-sha2-nistp256@openssh.com",
    ];

    // Find the key type in the line
    let mut parts = line.splitn(2, |c: char| c.is_whitespace());
    let first = parts.next()?;

    // Check if first part is a key type or options
    if KEY_TYPES.contains(&first) {
        // No options, first part is key type
        parse_key_parts(first, parts.next().unwrap_or(""), &[])
    } else {
        // First part is options, find key type in the rest
        let rest = parts.next()?;
        let options: Vec<String> = first.split(',').map(|s| s.to_string()).collect();

        let mut rest_parts = rest.splitn(2, |c: char| c.is_whitespace());
        let key_type = rest_parts.next()?;

        if !KEY_TYPES.contains(&key_type) {
            return None;
        }

        parse_key_parts(key_type, rest_parts.next().unwrap_or(""), &options)
    }
}

/// Parses the key type, data, and optional comment.
fn parse_key_parts(key_type: &str, rest: &str, options: &[String]) -> Option<AuthorizedKey> {
    let mut parts = rest.trim().splitn(2, |c: char| c.is_whitespace());
    let key_data = parts.next()?;

    if key_data.is_empty() {
        return None;
    }

    // Decode base64 key data
    let key_bytes = match base64_decode(key_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            debug!(error = %e, "Failed to decode base64 key data");
            return None;
        }
    };

    let comment = parts.next().map(|s| s.trim().to_string());

    Some(AuthorizedKey {
        key_type: key_type.to_string(),
        key_data: key_data.to_string(),
        key_bytes,
        comment,
        options: options.to_vec(),
    })
}

/// Simple base64 decoder (standard alphabet).
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn decode_char(c: u8) -> Result<u8, &'static str> {
        if c == b'=' {
            return Ok(0);
        }
        ALPHABET
            .iter()
            .position(|&x| x == c)
            .map(|p| p as u8)
            .ok_or("invalid base64 character")
    }

    let input = input.as_bytes();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    let mut iter = input.iter().filter(|&&c| c != b'\n' && c != b'\r');
    // Base64 decoding loop - processes 4 characters at a time with padding handling
    // Loop structure is intentional for handling partial groups and '=' padding
    #[allow(clippy::while_let_loop, clippy::redundant_guards)]
    loop {
        let a = match iter.next() {
            Some(&c) => decode_char(c)?,
            None => break,
        };
        let b = match iter.next() {
            Some(&c) => decode_char(c)?,
            None => return Err("invalid base64 length"),
        };
        let c = match iter.next() {
            Some(&c) if c == b'=' => {
                output.push((a << 2) | (b >> 4));
                // Consume the last '='
                iter.next();
                break;
            }
            Some(&c) => decode_char(c)?,
            None => return Err("invalid base64 length"),
        };
        let d = match iter.next() {
            Some(&ch) if ch == b'=' => {
                output.push((a << 2) | (b >> 4));
                output.push((b << 4) | (c >> 2));
                break;
            }
            Some(&ch) => decode_char(ch)?,
            None => return Err("invalid base64 length"),
        };

        output.push((a << 2) | (b >> 4));
        output.push((b << 4) | (c >> 2));
        output.push((c << 6) | d);
    }

    Ok(output)
}

/// Authorized keys file-based authentication.
///
/// Loads and caches keys from an OpenSSH `authorized_keys` file.
/// Supports hot-reloading when the file changes.
///
/// # Example
///
/// ```rust,ignore
/// use wish::auth::AuthorizedKeysAuth;
///
/// let auth = AuthorizedKeysAuth::new("/home/user/.ssh/authorized_keys")
///     .expect("Failed to load authorized_keys");
///
/// // Reload keys after file change
/// auth.reload().await.expect("Failed to reload");
/// ```
pub struct AuthorizedKeysAuth {
    /// Path to the authorized_keys file.
    keys_path: PathBuf,
    /// Cached keys per user (username -> keys).
    /// If empty, all users share the same keys.
    per_user: bool,
    /// Cached keys.
    cache: Arc<RwLock<HashMap<String, Vec<AuthorizedKey>>>>,
}

impl AuthorizedKeysAuth {
    /// Creates a new authorized keys auth handler.
    ///
    /// # Arguments
    ///
    /// * `keys_path` - Path to the authorized_keys file. Supports `~` expansion.
    ///
    /// # Returns
    ///
    /// The auth handler, or an error if the file cannot be loaded.
    pub fn new(keys_path: impl AsRef<Path>) -> io::Result<Self> {
        let path = expand_tilde(keys_path.as_ref());

        let auth = Self {
            keys_path: path.clone(),
            per_user: false,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load initial keys
        auth.load_keys_sync()?;

        Ok(auth)
    }

    /// Creates an auth handler that uses per-user authorized_keys files.
    ///
    /// The path should contain `%u` which will be replaced with the username.
    /// For example: `/home/%u/.ssh/authorized_keys`
    pub fn per_user(keys_path: impl AsRef<Path>) -> Self {
        let path = keys_path.as_ref().to_path_buf();

        Self {
            keys_path: path,
            per_user: true,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Reloads keys from the file.
    pub async fn reload(&self) -> io::Result<()> {
        self.load_keys_sync()
    }

    /// Loads keys synchronously (used for initial load and reload).
    fn load_keys_sync(&self) -> io::Result<()> {
        if self.per_user {
            // Per-user mode: clear cache, keys will be loaded on demand
            self.cache.write().clear();
            return Ok(());
        }

        // Global mode: load from the configured path
        let content = std::fs::read_to_string(&self.keys_path)?;
        let keys = parse_authorized_keys(&content);

        info!(
            path = %self.keys_path.display(),
            count = keys.len(),
            "Loaded authorized keys"
        );

        let mut cache = self.cache.write();
        cache.clear();
        cache.insert(String::new(), keys); // Empty string = global keys

        Ok(())
    }

    /// Gets the keys for a user, loading from file if needed.
    fn get_keys_for_user(&self, username: &str) -> Vec<AuthorizedKey> {
        if self.per_user {
            // Check cache first
            if let Some(keys) = self.cache.read().get(username) {
                return keys.clone();
            }

            // Load user-specific file
            let path = self.keys_path.to_string_lossy().replace("%u", username);
            let path = expand_tilde(Path::new(&path));

            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let keys = parse_authorized_keys(&content);
                    debug!(
                        username = %username,
                        path = %path.display(),
                        count = keys.len(),
                        "Loaded user authorized keys"
                    );
                    self.cache
                        .write()
                        .insert(username.to_string(), keys.clone());
                    keys
                }
                Err(e) => {
                    debug!(
                        username = %username,
                        path = %path.display(),
                        error = %e,
                        "Failed to load user authorized keys"
                    );
                    Vec::new()
                }
            }
        } else {
            // Global mode: return cached keys
            self.cache.read().get("").cloned().unwrap_or_default()
        }
    }

    /// Returns the number of cached keys.
    pub fn cached_key_count(&self) -> usize {
        self.cache.read().values().map(|v| v.len()).sum()
    }

    /// Returns the path to the authorized keys file.
    pub fn keys_path(&self) -> &Path {
        &self.keys_path
    }
}

#[async_trait]
impl AuthHandler for AuthorizedKeysAuth {
    async fn auth_publickey(&self, ctx: &AuthContext, key: &PublicKey) -> AuthResult {
        debug!(
            username = %ctx.username(),
            remote_addr = %ctx.remote_addr(),
            key_type = %key.key_type,
            "AuthorizedKeysAuth: auth attempt"
        );

        let authorized_keys = self.get_keys_for_user(ctx.username());

        for ak in &authorized_keys {
            if ak.matches(key) {
                info!(
                    username = %ctx.username(),
                    comment = ak.comment.as_deref().unwrap_or("<none>"),
                    "AuthorizedKeysAuth: accepted"
                );
                return AuthResult::Accept;
            }
        }

        debug!(
            username = %ctx.username(),
            key_count = authorized_keys.len(),
            "AuthorizedKeysAuth: no matching key"
        );
        AuthResult::Reject
    }

    fn supported_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::PublicKey]
    }
}

/// Expands `~` to the user's home directory.
fn expand_tilde(path: &Path) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    expand_tilde_with_home(path, home.as_deref())
}

/// Expands `~` to the given home directory (for testability).
fn expand_tilde_with_home(path: &Path, home: Option<&Path>) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix("~/")
        && let Some(home_dir) = home
    {
        return home_dir.join(stripped);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let line = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG1cILnhxkg+kMsGsVJP7hQnfKSPPIP/8GSXTE2n/8SE user@example.com";
        let keys = parse_authorized_keys(line);

        assert_eq!(keys.len(), 1);
        let key = &keys[0];
        assert_eq!(key.key_type, "ssh-ed25519");
        assert_eq!(key.comment, Some("user@example.com".to_string()));
        assert!(key.options.is_empty());
    }

    #[test]
    fn test_parse_key_with_options() {
        let line = "no-pty,command=\"/bin/git-shell\" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG1cILnhxkg+kMsGsVJP7hQnfKSPPIP/8GSXTE2n/8SE git@server";
        let keys = parse_authorized_keys(line);

        assert_eq!(keys.len(), 1);
        let key = &keys[0];
        assert_eq!(key.key_type, "ssh-ed25519");
        assert!(key.options.contains(&"no-pty".to_string()));
        assert!(key.options.iter().any(|o| o.starts_with("command=")));
    }

    #[test]
    fn test_parse_multiple_keys() {
        let content = r#"
# Comment line
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG1cILnhxkg+kMsGsVJP7hQnfKSPPIP/8GSXTE2n/8SE user1@example.com
ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAAgQC1 user2@example.com

# Another comment
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHUFrQ== user3@example.com
        "#;

        let keys = parse_authorized_keys(content);
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_parse_empty_and_comments() {
        let content = r#"
# Only comments

# and empty lines

        "#;

        let keys = parse_authorized_keys(content);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_base64_decode() {
        // "Hello" in base64 is "SGVsbG8="
        let decoded = base64_decode("SGVsbG8=").unwrap();
        assert_eq!(decoded, b"Hello");

        // "Hi" in base64 is "SGk="
        let decoded = base64_decode("SGk=").unwrap();
        assert_eq!(decoded, b"Hi");

        // "A" in base64 is "QQ=="
        let decoded = base64_decode("QQ==").unwrap();
        assert_eq!(decoded, b"A");
    }

    #[test]
    fn test_authorized_key_matches() {
        let ak = AuthorizedKey {
            key_type: "ssh-ed25519".to_string(),
            key_data: "AAAA".to_string(),
            key_bytes: vec![1, 2, 3, 4],
            comment: None,
            options: vec![],
        };

        let matching_key = PublicKey::new("ssh-ed25519", vec![1, 2, 3, 4]);
        assert!(ak.matches(&matching_key));

        let wrong_type = PublicKey::new("ssh-rsa", vec![1, 2, 3, 4]);
        assert!(!ak.matches(&wrong_type));

        let wrong_data = PublicKey::new("ssh-ed25519", vec![5, 6, 7, 8]);
        assert!(!ak.matches(&wrong_data));
    }

    #[test]
    fn test_expand_tilde() {
        let home = Path::new("/home/testuser");

        // Test with tilde and home set
        let expanded = expand_tilde_with_home(Path::new("~/.ssh/authorized_keys"), Some(home));
        assert_eq!(
            expanded,
            PathBuf::from("/home/testuser/.ssh/authorized_keys")
        );

        // Test without tilde
        let expanded = expand_tilde_with_home(Path::new("/etc/ssh/keys"), Some(home));
        assert_eq!(expanded, PathBuf::from("/etc/ssh/keys"));

        // Test with tilde but no home
        let expanded = expand_tilde_with_home(Path::new("~/.ssh/authorized_keys"), None);
        assert_eq!(expanded, PathBuf::from("~/.ssh/authorized_keys"));
    }
}
