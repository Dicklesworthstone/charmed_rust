//! FixtureLoader - Test fixture loading and management
//!
//! Provides infrastructure for loading and managing test fixtures including:
//! - Go reference outputs with versioning and staleness detection
//! - Test input data
//! - Expected results for conformance tests
//! - Caching for performance
//! - Schema validation for fixture format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when loading fixtures
#[derive(Debug, Error)]
pub enum FixtureError {
    /// Fixture file not found
    #[error("Fixture not found: {path}")]
    NotFound { path: String },

    /// IO error reading fixture
    #[error("IO error reading {path}: {error}")]
    Io { path: String, error: String },

    /// JSON parsing error
    #[error("Invalid JSON in fixture {path}: {error}")]
    InvalidJson { path: String, error: String },

    /// Fixture is stale (Go version changed)
    #[error("Fixture {path} is stale: captured with {captured}, current is {current}")]
    Stale {
        path: String,
        captured: String,
        current: String,
    },

    /// Test not found in fixture set
    #[error("Test '{test_name}' not found in crate '{crate_name}'")]
    TestNotFound {
        crate_name: String,
        test_name: String,
    },
}

/// Result type for fixture operations
pub type FixtureResult<T> = Result<T, FixtureError>;

/// Metadata about a fixture set's origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureMetadata {
    /// Name of the crate this fixture is for
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Go version used to capture this fixture
    pub go_version: String,
    /// Version of the Go library
    pub library_version: String,
    /// When the fixture was captured (ISO8601)
    pub captured_at: String,
    /// Platform the fixture was captured on (e.g., "linux-amd64")
    #[serde(default)]
    pub platform: Option<String>,
    /// Any notes about the fixture
    #[serde(default)]
    pub notes: Option<String>,
}

/// A complete set of fixtures for a crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureSet {
    /// Metadata about this fixture set
    pub metadata: FixtureMetadata,
    /// Individual test fixtures
    pub tests: Vec<TestFixture>,
}

/// A single test fixture with input and expected output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    /// Unique name for this test
    pub name: String,
    /// Test category (unit, integration, behavioral, etc.)
    #[serde(default)]
    pub category: Option<String>,
    /// Input data for the test (can be any JSON value)
    #[serde(default)]
    pub input: serde_json::Value,
    /// Expected output from the test
    pub expected_output: serde_json::Value,
    /// Notes about this specific test
    #[serde(default)]
    pub notes: Option<String>,
    /// Tags for filtering tests
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// If set, the test should be skipped with this reason
    #[serde(default)]
    pub skip_reason: Option<String>,
}

impl TestFixture {
    /// Get input as typed value
    pub fn input_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.input.clone())
    }

    /// Get expected output as typed value
    pub fn expected_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.expected_output.clone())
    }

    /// Get expected output as string (common case)
    pub fn expected_str(&self) -> Option<&str> {
        self.expected_output.as_str()
    }

    /// Check if fixture should be skipped
    pub fn should_skip(&self) -> Option<&str> {
        self.skip_reason.as_deref()
    }
}

/// Status of a fixture
#[derive(Debug, Clone)]
pub struct FixtureStatus {
    /// Whether the fixture file exists
    pub exists: bool,
    /// Whether the fixture is valid JSON
    pub valid: bool,
    /// Whether the fixture is stale (Go version changed)
    pub stale: bool,
    /// Path to the fixture
    pub path: PathBuf,
    /// Metadata if loaded successfully
    pub metadata: Option<FixtureMetadata>,
}

/// Cached fixture entry
enum CachedFixture {
    /// Fully loaded fixture set
    Loaded(FixtureSet),
    /// Path for lazy loading
    LazyPath(PathBuf),
}

/// Loader for test fixtures with caching and version tracking
///
/// Manages loading of test data, expected outputs, and Go reference
/// behaviors from the fixtures directory.
///
/// # Example
///
/// ```rust,ignore
/// let mut loader = FixtureLoader::new();
/// let fixtures = loader.load_crate("lipgloss")?;
/// for test in &fixtures.tests {
///     println!("Test: {} - {:?}", test.name, test.expected_output);
/// }
/// ```
pub struct FixtureLoader {
    /// Base path for fixtures
    base_path: PathBuf,
    /// Path to Go reference outputs
    go_outputs_path: PathBuf,
    /// Path to shared inputs
    inputs_path: PathBuf,
    /// Cache of loaded fixtures
    cache: HashMap<String, CachedFixture>,
    /// Expected Go library versions (for staleness detection)
    expected_versions: HashMap<String, String>,
    /// Whether caching is enabled
    caching_enabled: bool,
}

impl Default for FixtureLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl FixtureLoader {
    /// Create a new fixture loader with default paths
    pub fn new() -> Self {
        let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
        Self {
            go_outputs_path: base_path.join("go_outputs"),
            inputs_path: base_path.join("inputs"),
            base_path,
            cache: HashMap::new(),
            expected_versions: HashMap::new(),
            caching_enabled: true,
        }
    }

    /// Create a fixture loader with a custom base path
    pub fn with_base_path(base_path: impl AsRef<Path>) -> Self {
        let base_path = base_path.as_ref().to_path_buf();
        Self {
            go_outputs_path: base_path.join("go_outputs"),
            inputs_path: base_path.join("inputs"),
            base_path,
            cache: HashMap::new(),
            expected_versions: HashMap::new(),
            caching_enabled: true,
        }
    }

    /// Configure expected Go library versions for staleness detection
    pub fn with_go_versions(mut self, versions: HashMap<String, String>) -> Self {
        self.expected_versions = versions;
        self
    }

    /// Enable or disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.caching_enabled = enabled;
        self
    }

    /// Get the path to Go outputs directory
    pub fn go_outputs_path(&self) -> &Path {
        &self.go_outputs_path
    }

    /// Get the path to inputs directory
    pub fn inputs_path(&self) -> &Path {
        &self.inputs_path
    }

    /// Get the path to a Go reference output file (JSON format)
    pub fn go_output_path(&self, crate_name: &str) -> PathBuf {
        self.go_outputs_path.join(format!("{crate_name}.json"))
    }

    /// Get the path to an input fixture file
    pub fn input_path(&self, filename: &str) -> PathBuf {
        self.inputs_path.join(filename)
    }

    /// Load a fixture set for a crate
    pub fn load_crate(&mut self, crate_name: &str) -> FixtureResult<&FixtureSet> {
        // Check cache first - if already loaded, return early
        if self.caching_enabled {
            if let Some(CachedFixture::Loaded(_)) = self.cache.get(crate_name) {
                // Already cached, extract reference below
                return self.get_cached_fixture(crate_name);
            }
        }

        let path = self.go_output_path(crate_name);
        let path_str = path.display().to_string();

        if !path.exists() {
            return Err(FixtureError::NotFound { path: path_str });
        }

        let content = fs::read_to_string(&path).map_err(|e| FixtureError::Io {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

        let fixture_set: FixtureSet =
            serde_json::from_str(&content).map_err(|e| FixtureError::InvalidJson {
                path: path_str.clone(),
                error: e.to_string(),
            })?;

        // Check for staleness
        if let Some(expected) = self.expected_versions.get(crate_name) {
            if fixture_set.metadata.library_version != *expected {
                return Err(FixtureError::Stale {
                    path: path_str,
                    captured: fixture_set.metadata.library_version.clone(),
                    current: expected.clone(),
                });
            }
        }

        // Cache the fixture set
        self.cache
            .insert(crate_name.to_string(), CachedFixture::Loaded(fixture_set));

        // Return reference to cached value
        self.get_cached_fixture(crate_name)
    }

    /// Helper to get a cached fixture set
    fn get_cached_fixture(&self, crate_name: &str) -> FixtureResult<&FixtureSet> {
        if let Some(CachedFixture::Loaded(set)) = self.cache.get(crate_name) {
            Ok(set)
        } else {
            unreachable!("Fixture should be loaded at this point")
        }
    }

    /// Get a specific test fixture by name
    pub fn get_test(&mut self, crate_name: &str, test_name: &str) -> FixtureResult<&TestFixture> {
        self.load_crate(crate_name)?;

        if let Some(CachedFixture::Loaded(set)) = self.cache.get(crate_name) {
            set.tests
                .iter()
                .find(|t| t.name == test_name)
                .ok_or_else(|| FixtureError::TestNotFound {
                    crate_name: crate_name.to_string(),
                    test_name: test_name.to_string(),
                })
        } else {
            Err(FixtureError::TestNotFound {
                crate_name: crate_name.to_string(),
                test_name: test_name.to_string(),
            })
        }
    }

    /// Load a Go reference output as a string (legacy format)
    pub fn load_go_output(&self, crate_name: &str, test_name: &str) -> Option<String> {
        let path = self
            .go_outputs_path
            .join(crate_name)
            .join(format!("{test_name}.txt"));
        fs::read_to_string(path).ok()
    }

    /// Load an input fixture as a string
    pub fn load_input(&self, filename: &str) -> Option<String> {
        let path = self.input_path(filename);
        fs::read_to_string(path).ok()
    }

    /// Load a JSON fixture and deserialize it
    pub fn load_json<T: serde::de::DeserializeOwned>(&self, filename: &str) -> FixtureResult<T> {
        let path = self.input_path(filename);
        let path_str = path.display().to_string();

        if !path.exists() {
            return Err(FixtureError::NotFound { path: path_str });
        }

        let content = fs::read_to_string(&path).map_err(|e| FixtureError::Io {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

        serde_json::from_str(&content).map_err(|e| FixtureError::InvalidJson {
            path: path_str,
            error: e.to_string(),
        })
    }

    /// Load bytes from a fixture file
    pub fn load_bytes(&self, relative_path: &str) -> FixtureResult<Vec<u8>> {
        let path = self.base_path.join(relative_path);
        let path_str = path.display().to_string();

        if !path.exists() {
            return Err(FixtureError::NotFound { path: path_str });
        }

        fs::read(&path).map_err(|e| FixtureError::Io {
            path: path_str,
            error: e.to_string(),
        })
    }

    /// Check fixture status without fully loading
    pub fn status(&self, crate_name: &str) -> FixtureStatus {
        let path = self.go_output_path(crate_name);
        let exists = path.exists();

        if !exists {
            return FixtureStatus {
                exists: false,
                valid: false,
                stale: false,
                path,
                metadata: None,
            };
        }

        // Try to read and validate
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                return FixtureStatus {
                    exists: true,
                    valid: false,
                    stale: false,
                    path,
                    metadata: None,
                };
            }
        };

        let fixture_set: Result<FixtureSet, _> = serde_json::from_str(&content);
        match fixture_set {
            Ok(set) => {
                let stale = self
                    .expected_versions
                    .get(crate_name)
                    .is_some_and(|expected| set.metadata.library_version != *expected);

                FixtureStatus {
                    exists: true,
                    valid: true,
                    stale,
                    path,
                    metadata: Some(set.metadata),
                }
            }
            Err(_) => FixtureStatus {
                exists: true,
                valid: false,
                stale: false,
                path,
                metadata: None,
            },
        }
    }

    /// Check if a fixture exists
    pub fn fixture_exists(&self, crate_name: &str) -> bool {
        self.go_output_path(crate_name).exists()
    }

    /// List available crates with fixtures
    pub fn list_crates(&self) -> Vec<String> {
        if !self.go_outputs_path.exists() {
            return Vec::new();
        }

        fs::read_dir(&self.go_outputs_path)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        let path = e.path();
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all fixture files in a subdirectory
    pub fn list_fixtures(&self, subdir: &str) -> Vec<PathBuf> {
        let path = self.base_path.join(subdir);
        if !path.exists() {
            return Vec::new();
        }

        fs::read_dir(path)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| p.is_file())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all stale fixtures
    pub fn stale_fixtures(&self) -> Vec<FixtureStatus> {
        self.list_crates()
            .into_iter()
            .map(|name| self.status(&name))
            .filter(|s| s.stale)
            .collect()
    }

    /// Clear the fixture cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Preload all fixtures for a crate into cache
    pub fn preload(&mut self, crate_name: &str) -> FixtureResult<()> {
        self.load_crate(crate_name)?;
        Ok(())
    }

    /// Mark a fixture path for lazy loading
    pub fn mark_lazy(&mut self, crate_name: &str) {
        let path = self.go_output_path(crate_name);
        self.cache
            .insert(crate_name.to_string(), CachedFixture::LazyPath(path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_loader_creation() {
        let loader = FixtureLoader::new();
        assert!(loader.base_path.ends_with("fixtures"));
    }

    #[test]
    fn test_custom_base_path() {
        let loader = FixtureLoader::with_base_path("/tmp/test_fixtures");
        assert_eq!(loader.base_path, PathBuf::from("/tmp/test_fixtures"));
    }

    #[test]
    fn test_go_output_path() {
        let loader = FixtureLoader::new();
        let path = loader.go_output_path("lipgloss");
        assert!(path.ends_with("go_outputs/lipgloss.json"));
    }

    #[test]
    fn test_fixture_status_missing() {
        let loader = FixtureLoader::new();
        let status = loader.status("nonexistent_crate_xyz");
        assert!(!status.exists);
        assert!(!status.valid);
    }

    #[test]
    fn test_test_fixture_helpers() {
        let fixture = TestFixture {
            name: "test".to_string(),
            category: Some("unit".to_string()),
            input: serde_json::json!({"value": 42}),
            expected_output: serde_json::json!("expected"),
            notes: None,
            tags: None,
            skip_reason: None,
        };

        assert_eq!(fixture.expected_str(), Some("expected"));
        assert!(fixture.should_skip().is_none());
    }

    #[test]
    fn test_test_fixture_skip() {
        let fixture = TestFixture {
            name: "test".to_string(),
            category: None,
            input: serde_json::Value::Null,
            expected_output: serde_json::Value::Null,
            notes: None,
            tags: None,
            skip_reason: Some("Platform specific".to_string()),
        };

        assert_eq!(fixture.should_skip(), Some("Platform specific"));
    }

    #[test]
    fn test_caching_toggle() {
        let loader = FixtureLoader::new().with_caching(false);
        assert!(!loader.caching_enabled);
    }

    #[test]
    fn test_go_versions() {
        let mut versions = HashMap::new();
        versions.insert("lipgloss".to_string(), "0.11.0".to_string());

        let loader = FixtureLoader::new().with_go_versions(versions.clone());
        assert_eq!(loader.expected_versions, versions);
    }
}
