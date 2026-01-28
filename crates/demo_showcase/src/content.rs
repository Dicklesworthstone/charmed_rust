//! Content & Copy Style Guide for `demo_showcase`.
//!
//! This module defines the text/content conventions that make the demo feel like
//! a real ops console / developer tool—not a generic TUI demo with lorem ipsum.
//!
//! # Product Identity
//!
//! **Name:** Charmed Control Center
//!
//! The demo presents itself as a premium operations console for managing
//! microservices, deployments, jobs, and infrastructure. All content should
//! reinforce this "DevOps dashboard" identity.
//!
//! # Tone & Voice
//!
//! - **Professional:** Enterprise-quality, trustworthy
//! - **Crisp:** Concise, no filler words
//! - **Actionable:** Focus on what the user can do
//! - **Confident:** Direct statements, not hedging language
//!
//! ## Good Examples
//!
//! - "12/15 services healthy"
//! - "Deployment succeeded"
//! - "Press r to refresh"
//! - "Uptime: 7d 5h 23m"
//!
//! ## Avoid
//!
//! - Lorem ipsum or placeholder text
//! - Overly casual language ("Oops!")
//! - Vague descriptions ("Something went wrong")
//! - Excessive punctuation or emojis
//!
//! # Entity Naming Conventions
//!
//! ## Services
//!
//! Use microservice-style names: `{function}-{type}` format.
//!
//! | Prefixes | Suffixes |
//! |----------|----------|
//! | api, auth, billing, cache | service, svc, worker, handler |
//! | config, data, email, event | processor, gateway, proxy |
//! | file, gateway, identity | server, api, engine, manager |
//! | inventory, job, kafka, log | |
//! | media, notification, order | |
//! | payment, queue, redis | |
//! | search, telemetry, user | |
//! | web, worker | |
//!
//! **Examples:** api-service, auth-handler, billing-worker, cache-proxy
//!
//! ## Jobs & Tasks
//!
//! Use descriptive templates with numeric suffixes for uniqueness:
//!
//! - database-backup-{n}
//! - log-rotation-{n}
//! - cache-warmup-{n}
//! - index-rebuild-{n}
//! - schema-migration-{n}
//! - security-scan-{n}
//! - test-suite-{n}
//!
//! ## Environments
//!
//! Standard deployment environment names (lowercase):
//!
//! - production, staging, development
//! - qa, sandbox, canary, preview, demo
//!
//! ## Authors/Users
//!
//! Common first names for realistic attribution:
//!
//! alice, bob, carol, david, eve, frank, grace, henry, etc.
//!
//! # Status Labels & Indicators
//!
//! ## Platform Health
//!
//! | State | Label | Description |
//! |-------|-------|-------------|
//! | All healthy | HEALTHY | All services operational |
//! | Some degraded | PARTIAL | Some services have issues |
//! | Any unhealthy | DEGRADED | Critical issues present |
//!
//! ## Service Health
//!
//! | State | Icon | Label |
//! |-------|------|-------|
//! | Healthy | ● | (green) |
//! | Degraded | ⚠ | (yellow) |
//! | Unhealthy | ✕ | (red) |
//! | Unknown | ℹ | (blue/gray) |
//!
//! ## Job Status
//!
//! | State | Icon | Label |
//! |-------|------|-------|
//! | Queued | (blue) | queued |
//! | Running | ◐ | {n}% |
//! | Completed | ● | done |
//! | Failed | ✕ | failed |
//! | Cancelled | ✕ | cancelled |
//!
//! ## Deployment Status
//!
//! | State | Label (chip) |
//! |-------|--------------|
//! | Pending | pending |
//! | In Progress | deploying |
//! | Succeeded | success |
//! | Failed | failed |
//! | Rolled Back | rolled back |
//!
//! # Metrics & Numbers
//!
//! ## Format Rules
//!
//! - Use units in labels: "Requests/s", "P95 Latency", "Error Rate"
//! - Large numbers: Use commas (1,234) or abbreviations (1.2k)
//! - Percentages: "45%" not "45 percent"
//! - Time durations: "7d 5h 23m" format
//! - Latency: Include unit suffix "ms"
//!
//! ## Metric Examples
//!
//! | Metric | Format | Example |
//! |--------|--------|---------|
//! | Requests per second | Number | 1,234 |
//! | Latency | Number + ms | 45ms |
//! | Error rate | Percentage | 0.12% |
//! | Uptime | Duration | 7d 5h 23m |
//! | Progress | Percentage | 75% |
//!
//! # Key Hints & Shortcuts
//!
//! ## Format
//!
//! `{key} {action}` with multiple hints separated by double spaces.
//!
//! - Keys: lowercase, no brackets (e.g., "r", "Enter", "Esc")
//! - Actions: lowercase verb phrase (e.g., "refresh", "select", "quit")
//! - Separator: two spaces between hint pairs
//!
//! ## Examples
//!
//! - "r refresh  s services  j jobs"
//! - "j/k nav  Enter select  / filter  Esc unfocus"
//! - "1-7 pages  [ sidebar  ? help  q quit"
//!
//! # Error & Warning Messages
//!
//! ## Principles
//!
//! 1. **Be specific:** Name what failed
//! 2. **Be brief:** One short sentence
//! 3. **Be actionable:** Suggest recovery when possible
//!
//! ## Good Examples
//!
//! - "High CPU usage on api-service"
//! - "Memory threshold exceeded on cache-worker"
//! - "Connection pool exhausted in payment-handler"
//! - "Process exited with non-zero status"
//! - "Export failed: permission denied"
//!
//! ## Avoid
//!
//! - "An error occurred" (too vague)
//! - "Something went wrong" (not actionable)
//! - Multi-sentence errors (keep it short)
//!
//! # Notification Copy
//!
//! ## Success
//!
//! - "Exported to {filename}"
//! - "Deployment succeeded"
//! - "Configuration saved"
//!
//! ## Warning
//!
//! - "High memory usage on {service}"
//! - "Latency increase on {service} endpoints"
//! - "Certificate expiring soon for {service}"
//!
//! ## Error
//!
//! - "Export failed: {reason}"
//! - "Connection timeout in {service}"
//! - "Health check failing for {service}"
//!
//! # Empty States
//!
//! Use guiding copy that explains what would normally appear:
//!
//! - "No recent deployments" (not "Nothing here")
//! - "No jobs in queue" (not "Empty")
//! - "No logs matching filter" (not "No results")
//!
//! # Page Icons (ASCII-safe)
//!
//! Each navigation page has a two-character icon:
//!
//! | Page | Icon | Meaning |
//! |------|------|---------|
//! | Dashboard | [] | Window/grid |
//! | Services | >_ | Terminal prompt |
//! | Jobs | >> | Process/arrow |
//! | Logs | # | Log/hash |
//! | Docs | ? | Help/question |
//! | Wizard | * | Star/magic |
//! | Settings | @ | Config/at |
//!
//! # Spacing & Punctuation
//!
//! - **Lists:** No trailing punctuation on items
//! - **Labels:** Title Case for navigation, lowercase for status chips
//! - **Counts:** "{n}/{total}" format (e.g., "12/15 services healthy")
//! - **Versions:** Semver format (e.g., "1.2.3")
//! - **SHAs:** 7-character truncation (e.g., "a1b2c3d")
//!
//! # Accessibility
//!
//! - All icons have ASCII fallbacks (●→*, ⚠→!, ✕→x, ℹ→i, ◐→~)
//! - Color is never the only indicator (always paired with text/icon)
//! - Key hints spell out actions, not just show keys

#![allow(dead_code)] // Constants are reference documentation

// =============================================================================
// Product Identity
// =============================================================================

/// The product name displayed in headers and titles.
pub const PRODUCT_NAME: &str = "Charmed Control Center";

/// Short tagline for the product.
pub const PRODUCT_TAGLINE: &str = "Platform health at a glance";

// =============================================================================
// Status Labels
// =============================================================================

/// Platform health status labels (uppercase for badges).
pub mod platform_status {
    pub const HEALTHY: &str = "HEALTHY";
    pub const PARTIAL: &str = "PARTIAL";
    pub const DEGRADED: &str = "DEGRADED";
}

/// Deployment status labels (lowercase for chips).
pub mod deployment_status {
    pub const PENDING: &str = "pending";
    pub const DEPLOYING: &str = "deploying";
    pub const SUCCESS: &str = "success";
    pub const FAILED: &str = "failed";
    pub const ROLLED_BACK: &str = "rolled back";
}

/// Job status labels (lowercase for display).
pub mod job_status {
    pub const QUEUED: &str = "queued";
    pub const RUNNING: &str = "running";
    pub const DONE: &str = "done";
    pub const FAILED: &str = "failed";
    pub const CANCELLED: &str = "cancelled";
}

// =============================================================================
// Icons
// =============================================================================

/// Unicode status icons.
pub mod icons {
    pub const SUCCESS: &str = "●";
    pub const WARNING: &str = "⚠";
    pub const ERROR: &str = "✕";
    pub const INFO: &str = "ℹ";
    pub const RUNNING: &str = "◐";
}

/// ASCII fallback icons for terminals without Unicode support.
pub mod ascii_icons {
    pub const SUCCESS: &str = "*";
    pub const WARNING: &str = "!";
    pub const ERROR: &str = "x";
    pub const INFO: &str = "i";
    pub const RUNNING: &str = "~";
}

// =============================================================================
// Format Helpers
// =============================================================================

/// Format an uptime duration as a human-readable string.
///
/// # Examples
///
/// ```ignore
/// format_uptime(90061) // => "1d 1h 1m"
/// format_uptime(3661)  // => "1h 1m"
/// format_uptime(61)    // => "1m"
/// ```
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

/// Format a count as a fraction string.
///
/// # Examples
///
/// ```ignore
/// format_fraction(12, 15) // => "12/15"
/// ```
pub fn format_fraction(count: usize, total: usize) -> String {
    format!("{count}/{total}")
}

/// Truncate a SHA to 7 characters.
///
/// # Examples
///
/// ```ignore
/// truncate_sha("a1b2c3d4e5f6") // => "a1b2c3d"
/// ```
pub fn truncate_sha(sha: &str) -> &str {
    if sha.len() > 7 { &sha[..7] } else { sha }
}

// =============================================================================
// Message Templates
// =============================================================================

/// Alert message templates with `{service}` placeholder.
pub mod alert_templates {
    pub const HIGH_CPU: &str = "High CPU usage on {service}";
    pub const HIGH_MEMORY: &str = "Memory threshold exceeded on {service}";
    pub const LOW_DISK: &str = "Disk space low on {service}";
    pub const POOL_EXHAUSTED: &str = "Connection pool exhausted in {service}";
    pub const ERROR_SPIKE: &str = "Error rate spike detected in {service}";
    pub const LATENCY_INCREASE: &str = "Latency increase on {service} endpoints";
    pub const CERT_EXPIRING: &str = "Certificate expiring soon for {service}";
    pub const RATE_LIMITED: &str = "Rate limiting triggered on {service}";
    pub const DB_TIMEOUT: &str = "Database connection timeout in {service}";
    pub const QUEUE_CRITICAL: &str = "Queue depth critical on {service}";
    pub const HEALTH_FAILING: &str = "Health check failing for {service}";
    pub const ROLLBACK: &str = "Deployment rollback triggered for {service}";
}

/// Empty state messages.
pub mod empty_states {
    pub const NO_DEPLOYMENTS: &str = "No recent deployments";
    pub const NO_JOBS: &str = "No recent jobs";
    pub const NO_ALERTS: &str = "No active alerts";
    pub const NO_LOGS: &str = "No logs matching filter";
    pub const NO_RESULTS: &str = "No matching results";
}

/// Notification messages.
pub mod notifications {
    pub const EXPORT_SUCCESS: &str = "Exported to";
    pub const EXPORT_FAILED: &str = "Export failed:";
    pub const CONFIG_SAVED: &str = "Configuration saved";
    pub const REFRESH_COMPLETE: &str = "Data refreshed";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_uptime_days() {
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn format_uptime_hours() {
        assert_eq!(format_uptime(3661), "1h 1m");
    }

    #[test]
    fn format_uptime_minutes() {
        assert_eq!(format_uptime(61), "1m");
    }

    #[test]
    fn format_fraction_works() {
        assert_eq!(format_fraction(12, 15), "12/15");
    }

    #[test]
    fn truncate_sha_long() {
        assert_eq!(truncate_sha("a1b2c3d4e5f6g7h8"), "a1b2c3d");
    }

    #[test]
    fn truncate_sha_short() {
        assert_eq!(truncate_sha("abc"), "abc");
    }
}
