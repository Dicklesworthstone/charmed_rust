//! GitHub README fetching functionality.
//!
//! This module is only available when the `github` feature is enabled.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use directories::ProjectDirs;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

/// Represents a GitHub repository reference.
#[derive(Debug, Clone, PartialEq)]
pub struct RepoRef {
    /// Repository owner (user or organization).
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Optional branch name.
    pub branch: Option<String>,
}

impl RepoRef {
    /// Parse a repository reference from various formats.
    ///
    /// Supported formats:
    /// - `owner/repo`
    /// - `github.com/owner/repo`
    /// - `https://github.com/owner/repo`
    /// - `git@github.com:owner/repo`
    /// - `owner/repo@branch`
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();

        // Extract branch if specified with @
        let (repo_part, branch) = if let Some(at_idx) = input.rfind('@') {
            // Make sure @ is not part of git@ prefix
            if input.starts_with("git@") && at_idx < 15 {
                (input, None)
            } else {
                let branch = &input[at_idx + 1..];
                let repo = &input[..at_idx];
                (repo, Some(branch.to_string()))
            }
        } else {
            (input, None)
        };

        // Try different formats
        let (owner, name) = if repo_part.starts_with("git@github.com:") {
            // SSH format: git@github.com:owner/repo.git
            let path = repo_part
                .strip_prefix("git@github.com:")
                .ok_or(ParseError::InvalidFormat)?;
            let path = path.strip_suffix(".git").unwrap_or(path);
            Self::parse_owner_repo(path)?
        } else if repo_part.starts_with("https://github.com/")
            || repo_part.starts_with("http://github.com/")
        {
            // HTTPS URL format
            let path = repo_part
                .strip_prefix("https://github.com/")
                .or_else(|| repo_part.strip_prefix("http://github.com/"))
                .ok_or(ParseError::InvalidFormat)?;
            let path = path.strip_suffix(".git").unwrap_or(path);
            // Remove any trailing path components (like /tree/branch)
            let path = path.split('/').take(2).collect::<Vec<_>>().join("/");
            Self::parse_owner_repo(&path)?
        } else if repo_part.starts_with("github.com/") {
            // github.com/owner/repo format
            let path = repo_part
                .strip_prefix("github.com/")
                .ok_or(ParseError::InvalidFormat)?;
            Self::parse_owner_repo(path)?
        } else {
            // Simple owner/repo format
            Self::parse_owner_repo(repo_part)?
        };

        Ok(Self {
            owner,
            name,
            branch,
        })
    }

    fn parse_owner_repo(path: &str) -> Result<(String, String), ParseError> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() < 2 {
            return Err(ParseError::MissingOwnerOrRepo);
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Returns the cache key for this repository.
    pub fn cache_key(&self) -> String {
        match &self.branch {
            Some(branch) => format!("{}_{}_{}", self.owner, self.name, branch),
            None => format!("{}_{}", self.owner, self.name),
        }
    }
}

/// Error parsing a repository reference.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Invalid repository format.
    InvalidFormat,
    /// Missing owner or repository name.
    MissingOwnerOrRepo,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid repository format"),
            Self::MissingOwnerOrRepo => write!(f, "missing owner or repository name"),
        }
    }
}

impl std::error::Error for ParseError {}

/// GitHub API response for README content.
#[derive(Debug, Deserialize)]
struct ReadmeResponse {
    content: String,
    encoding: String,
    #[serde(default)]
    name: String,
}

/// Error fetching README from GitHub.
#[derive(Debug)]
pub enum FetchError {
    /// HTTP request failed.
    Request(reqwest::Error),
    /// API returned an error status.
    ApiError { status: u16, message: String },
    /// Failed to decode content.
    DecodeError(String),
    /// Rate limit exceeded.
    RateLimited { reset_at: Option<u64> },
    /// Cache I/O error.
    CacheError(io::Error),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "request failed: {e}"),
            Self::ApiError { status, message } => write!(f, "API error ({status}): {message}"),
            Self::DecodeError(msg) => write!(f, "decode error: {msg}"),
            Self::RateLimited { reset_at: Some(ts) } => {
                write!(f, "rate limited, resets at timestamp {ts}")
            }
            Self::RateLimited { reset_at: None } => write!(f, "rate limited"),
            Self::CacheError(e) => write!(f, "cache error: {e}"),
        }
    }
}

impl std::error::Error for FetchError {}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl From<io::Error> for FetchError {
    fn from(e: io::Error) -> Self {
        Self::CacheError(e)
    }
}

/// Configuration for the GitHub fetcher.
#[derive(Debug, Clone)]
pub struct FetcherConfig {
    /// GitHub API token for authentication.
    pub token: Option<String>,
    /// Cache time-to-live in seconds.
    pub cache_ttl: Duration,
    /// Whether to skip cache and fetch fresh.
    pub force_refresh: bool,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            token: std::env::var("GITHUB_TOKEN").ok(),
            cache_ttl: Duration::from_secs(3600), // 1 hour default
            force_refresh: false,
        }
    }
}

/// Fetches README files from GitHub repositories.
pub struct GitHubFetcher {
    client: Client,
    config: FetcherConfig,
    cache_dir: Option<PathBuf>,
}

impl GitHubFetcher {
    /// Create a new fetcher with the given configuration.
    pub fn new(config: FetcherConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client");

        let cache_dir = ProjectDirs::from("com", "charmbracelet", "glow")
            .map(|dirs| dirs.cache_dir().to_path_buf());

        Self {
            client,
            config,
            cache_dir,
        }
    }

    /// Fetch the README for a repository.
    pub fn fetch_readme(&self, repo: &RepoRef) -> Result<String, FetchError> {
        // Check cache first (unless force refresh)
        if !self.config.force_refresh {
            if let Some(cached) = self.get_cached(repo)? {
                return Ok(cached);
            }
        }

        // Build API URL
        let url = match &repo.branch {
            Some(branch) => format!(
                "https://api.github.com/repos/{}/{}/readme?ref={}",
                repo.owner, repo.name, branch
            ),
            None => format!(
                "https://api.github.com/repos/{}/{}/readme",
                repo.owner, repo.name
            ),
        };

        // Build request
        let mut request = self
            .client
            .get(&url)
            .header(USER_AGENT, "glow-cli/1.0")
            .header(ACCEPT, "application/vnd.github.v3+json");

        if let Some(token) = &self.config.token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        // Execute request
        let response = request.send()?;
        let status = response.status();

        // Handle rate limiting
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            let reset_at = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(FetchError::RateLimited { reset_at });
        }

        // Handle errors
        if !status.is_success() {
            let message = response.text().unwrap_or_default();
            return Err(FetchError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        // Parse response
        let readme: ReadmeResponse = response.json()?;

        // Decode content (GitHub returns base64-encoded content)
        let content = if readme.encoding == "base64" {
            let cleaned: String = readme.content.chars().filter(|c| !c.is_whitespace()).collect();
            let decoded = base64_decode(&cleaned)
                .map_err(|e| FetchError::DecodeError(format!("base64 decode failed: {e}")))?;
            String::from_utf8(decoded)
                .map_err(|e| FetchError::DecodeError(format!("UTF-8 decode failed: {e}")))?
        } else {
            readme.content
        };

        // Cache the result
        self.set_cached(repo, &content)?;

        Ok(content)
    }

    fn get_cached(&self, repo: &RepoRef) -> Result<Option<String>, FetchError> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(None),
        };

        let cache_file = cache_dir.join(format!("{}.md", repo.cache_key()));
        if !cache_file.exists() {
            return Ok(None);
        }

        // Check TTL
        let metadata = fs::metadata(&cache_file)?;
        let modified = metadata.modified()?;
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::MAX);

        if age > self.config.cache_ttl {
            return Ok(None);
        }

        Ok(Some(fs::read_to_string(cache_file)?))
    }

    fn set_cached(&self, repo: &RepoRef, content: &str) -> Result<(), FetchError> {
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir,
            None => return Ok(()),
        };

        fs::create_dir_all(cache_dir)?;
        let cache_file = cache_dir.join(format!("{}.md", repo.cache_key()));
        fs::write(cache_file, content)?;

        Ok(())
    }
}

/// Simple base64 decoder (avoids adding another dependency).
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1,
        -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in input.chars() {
        if c == '=' {
            break;
        }

        let byte = c as usize;
        if byte >= 128 {
            return Err(format!("invalid character: {c}"));
        }

        let value = DECODE_TABLE[byte];
        if value < 0 {
            return Err(format!("invalid character: {c}"));
        }

        buffer = (buffer << 6) | (value as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_ref_parse_simple() {
        let repo = RepoRef::parse("owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.branch, None);
    }

    #[test]
    fn test_repo_ref_parse_with_branch() {
        let repo = RepoRef::parse("owner/repo@main").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
        assert_eq!(repo.branch, Some("main".to_string()));
    }

    #[test]
    fn test_repo_ref_parse_https_url() {
        let repo = RepoRef::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_repo_ref_parse_https_url_with_git() {
        let repo = RepoRef::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_repo_ref_parse_github_com() {
        let repo = RepoRef::parse("github.com/owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_repo_ref_parse_ssh() {
        let repo = RepoRef::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.name, "repo");
    }

    #[test]
    fn test_repo_ref_parse_invalid() {
        assert!(RepoRef::parse("invalid").is_err());
        assert!(RepoRef::parse("").is_err());
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ="; // "Hello World"
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");
    }

    #[test]
    fn test_cache_key() {
        let repo = RepoRef {
            owner: "owner".to_string(),
            name: "repo".to_string(),
            branch: None,
        };
        assert_eq!(repo.cache_key(), "owner_repo");

        let repo_with_branch = RepoRef {
            owner: "owner".to_string(),
            name: "repo".to_string(),
            branch: Some("main".to_string()),
        };
        assert_eq!(repo_with_branch.cache_key(), "owner_repo_main");
    }
}
