#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

//! External unit tests for glow file browser, utility rendering, and
//! Reader/Config edge cases.

use std::fs::{self, File};
use std::io::Write as _;
use std::path::Path;

use bubbletea::{KeyMsg, KeyType, Message, Model};
use glow::browser::{BrowserConfig, Entry, FileBrowser};
use glow::{Config, Reader, Stash};
use tempfile::TempDir;

// =============================================================================
// Test helpers
// =============================================================================

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn touch(dir: &Path, name: &str) {
    File::create(dir.join(name)).unwrap();
}

fn write_file(dir: &Path, name: &str, content: &str) {
    let mut f = File::create(dir.join(name)).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn key_msg(key_type: KeyType) -> Message {
    Message::new(KeyMsg {
        key_type,
        runes: Vec::new(),
        alt: false,
        paste: false,
    })
}

fn rune_msg(c: char) -> Message {
    Message::new(KeyMsg {
        key_type: KeyType::Runes,
        runes: vec![c],
        alt: false,
        paste: false,
    })
}

fn populated_browser(dir: &TempDir) -> FileBrowser {
    touch(dir.path(), "alpha.md");
    touch(dir.path(), "beta.md");
    touch(dir.path(), "gamma.md");
    fs::create_dir(dir.path().join("subdir")).unwrap();
    touch(&dir.path().join("subdir"), "nested.md");

    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();
    browser
}

// =============================================================================
// BrowserConfig
// =============================================================================

#[test]
fn browser_config_default_extensions() {
    let config = BrowserConfig::default();
    assert!(config.extensions.contains(&"md".to_string()));
    assert!(config.extensions.contains(&"markdown".to_string()));
    assert!(!config.show_hidden);
    assert!(!config.recursive);
    assert_eq!(config.max_depth, 5);
}

// =============================================================================
// Entry
// =============================================================================

#[test]
fn entry_directory_type() {
    let dir = make_dir();
    let entry = Entry::from_path(dir.path()).unwrap();
    assert!(entry.is_directory());
    assert!(!entry.is_markdown());
    assert_eq!(entry.size_display(), "-");
}

#[test]
fn entry_markdown_file() {
    let dir = make_dir();
    touch(dir.path(), "test.md");
    let entry = Entry::from_path(&dir.path().join("test.md")).unwrap();
    assert!(entry.is_markdown());
    assert!(!entry.is_directory());
}

#[test]
fn entry_non_markdown_file() {
    let dir = make_dir();
    touch(dir.path(), "test.rs");
    let entry = Entry::from_path(&dir.path().join("test.rs")).unwrap();
    assert!(!entry.is_markdown());
    assert!(!entry.is_directory());
}

#[test]
fn entry_mdown_extension() {
    let dir = make_dir();
    touch(dir.path(), "notes.mdown");
    let entry = Entry::from_path(&dir.path().join("notes.mdown")).unwrap();
    assert!(entry.is_markdown());
}

#[test]
fn entry_mkd_extension() {
    let dir = make_dir();
    touch(dir.path(), "notes.mkd");
    let entry = Entry::from_path(&dir.path().join("notes.mkd")).unwrap();
    assert!(entry.is_markdown());
}

#[test]
fn entry_size_display_bytes() {
    let dir = make_dir();
    write_file(dir.path(), "small.md", "hello");
    let entry = Entry::from_path(&dir.path().join("small.md")).unwrap();
    assert_eq!(entry.size_display(), "5B");
}

#[test]
fn entry_nonexistent_path() {
    let result = Entry::from_path(Path::new("/nonexistent/path/file.md"));
    assert!(result.is_err());
}

// =============================================================================
// FileBrowser: creation and scanning
// =============================================================================

#[test]
fn browser_with_invalid_directory() {
    let result = FileBrowser::with_directory("/nonexistent/path", BrowserConfig::default());
    assert!(result.is_err());
}

#[test]
fn browser_with_file_path_not_directory() {
    let dir = make_dir();
    touch(dir.path(), "file.txt");
    let result =
        FileBrowser::with_directory(dir.path().join("file.txt"), BrowserConfig::default());
    assert!(result.is_err());
}

#[test]
fn browser_empty_directory() {
    let dir = make_dir();
    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();
    assert!(browser.entries().is_empty());
    assert!(browser.selected_entry().is_none());
}

#[test]
fn browser_scan_excludes_non_markdown() {
    let dir = make_dir();
    touch(dir.path(), "readme.md");
    touch(dir.path(), "main.rs");
    touch(dir.path(), "data.json");

    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();

    let names: Vec<_> = browser.entries().iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"readme.md"));
    assert!(!names.contains(&"main.rs"));
    assert!(!names.contains(&"data.json"));
}

#[test]
fn browser_directories_sorted_first() {
    let dir = make_dir();
    touch(dir.path(), "zebra.md");
    fs::create_dir(dir.path().join("aaa_dir")).unwrap();
    touch(dir.path(), "alpha.md");

    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();

    assert!(browser.entries()[0].is_directory());
    assert_eq!(browser.entries()[0].name, "aaa_dir");
}

#[test]
fn browser_recursive_scan() {
    let dir = make_dir();
    touch(dir.path(), "top.md");
    fs::create_dir(dir.path().join("sub")).unwrap();
    touch(&dir.path().join("sub"), "deep.md");

    let config = BrowserConfig {
        recursive: true,
        ..Default::default()
    };
    let mut browser = FileBrowser::with_directory(dir.path(), config).unwrap();
    browser.scan().unwrap();

    let names: Vec<_> = browser.entries().iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"top.md"));
    assert!(names.contains(&"deep.md"));
    // Recursive mode doesn't show directories
    assert!(!names.contains(&"sub"));
}

#[test]
fn browser_hidden_files_excluded_by_default() {
    let dir = make_dir();
    touch(dir.path(), ".hidden.md");
    touch(dir.path(), "visible.md");

    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();

    let names: Vec<_> = browser.entries().iter().map(|e| e.name.as_str()).collect();
    assert!(!names.contains(&".hidden.md"));
    assert!(names.contains(&"visible.md"));
}

// =============================================================================
// FileBrowser: navigation
// =============================================================================

#[test]
fn browser_move_up_at_top_stays() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.move_up();
    assert_eq!(browser.selected_entry().map(|e| e.name.as_str()), browser.entries().first().map(|e| e.name.as_str()));
}

#[test]
fn browser_move_down_at_bottom_stays() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    let count = browser.filtered_entries().len();
    for _ in 0..count + 5 {
        browser.move_down();
    }
    // Should be at last entry
    assert!(browser.selected_entry().is_some());
}

#[test]
fn browser_page_up_from_zero() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.page_up();
    assert!(browser.selected_entry().is_some());
}

#[test]
fn browser_page_down_past_end() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.page_down();
    browser.page_down();
    browser.page_down();
    assert!(browser.selected_entry().is_some());
}

#[test]
fn browser_move_to_top_and_bottom() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    let last_name = browser.filtered_entries().last().unwrap().name.clone();

    browser.move_to_bottom();
    assert_eq!(browser.selected_entry().unwrap().name, last_name);

    browser.move_to_top();
    assert_eq!(
        browser.selected_entry().unwrap().name,
        browser.entries().first().unwrap().name
    );
}

// =============================================================================
// FileBrowser: filter
// =============================================================================

#[test]
fn browser_filter_narrows_results() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    let initial = browser.filtered_entries().len();

    browser.set_filter("alpha");
    assert!(browser.filtered_entries().len() < initial);
    assert_eq!(browser.filtered_entries()[0].name, "alpha.md");
}

#[test]
fn browser_filter_case_insensitive() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.set_filter("ALPHA");
    assert_eq!(browser.filtered_entries().len(), 1);
}

#[test]
fn browser_filter_no_match() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.set_filter("zzzzz_no_match");
    assert!(browser.filtered_entries().is_empty());
    assert!(browser.selected_entry().is_none());
}

#[test]
fn browser_clear_filter_restores_all() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    let initial = browser.filtered_entries().len();
    browser.set_filter("alpha");
    browser.clear_filter();
    assert_eq!(browser.filtered_entries().len(), initial);
}

#[test]
fn browser_filter_input_and_backspace() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.filter_input('a');
    assert_eq!(browser.filter(), "a");
    browser.filter_input('l');
    assert_eq!(browser.filter(), "al");
    browser.filter_backspace();
    assert_eq!(browser.filter(), "a");
    browser.filter_backspace();
    assert_eq!(browser.filter(), "");
}

#[test]
fn browser_filter_backspace_on_empty() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.filter_backspace();
    assert_eq!(browser.filter(), "");
}

// =============================================================================
// FileBrowser: filter mode
// =============================================================================

#[test]
fn browser_filter_mode_toggle() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    assert!(!browser.is_filter_mode());
    browser.enter_filter_mode();
    assert!(browser.is_filter_mode());
    browser.exit_filter_mode();
    assert!(!browser.is_filter_mode());
}

// =============================================================================
// FileBrowser: Model trait
// =============================================================================

#[test]
fn browser_model_init_returns_cmd() {
    let browser = FileBrowser::new(BrowserConfig::default());
    assert!(browser.init().is_some());
}

#[test]
fn browser_model_update_arrow_keys() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    let first = browser.selected_entry().unwrap().name.clone();

    browser.update(key_msg(KeyType::Down));
    assert_ne!(browser.selected_entry().unwrap().name, first);

    browser.update(key_msg(KeyType::Up));
    assert_eq!(browser.selected_entry().unwrap().name, first);
}

#[test]
fn browser_model_update_vim_keys() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(rune_msg('j'));
    let after_j = browser.selected_entry().unwrap().name.clone();

    browser.update(rune_msg('k'));
    let after_k = browser.selected_entry().unwrap().name.clone();

    assert_ne!(after_j, after_k);
}

#[test]
fn browser_model_update_g_and_big_g() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(rune_msg('G'));
    let last = browser.selected_entry().unwrap().name.clone();

    browser.update(rune_msg('g'));
    let first = browser.selected_entry().unwrap().name.clone();

    assert_ne!(first, last);
}

#[test]
fn browser_model_update_slash_enters_filter_mode() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    assert!(!browser.is_filter_mode());

    browser.update(rune_msg('/'));
    assert!(browser.is_filter_mode());
}

#[test]
fn browser_model_filter_mode_esc_clears() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(rune_msg('/'));
    browser.update(rune_msg('a'));
    assert!(browser.is_filter_mode());

    browser.update(key_msg(KeyType::Esc));
    assert!(!browser.is_filter_mode());
    assert_eq!(browser.filter(), "");
}

#[test]
fn browser_model_filter_mode_enter_exits() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(rune_msg('/'));
    browser.update(rune_msg('a'));
    browser.update(key_msg(KeyType::Enter));

    assert!(!browser.is_filter_mode());
    assert_eq!(browser.filter(), "a"); // filter preserved
}

#[test]
fn browser_model_filter_mode_backspace() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(rune_msg('/'));
    browser.update(rune_msg('x'));
    browser.update(rune_msg('y'));
    browser.update(key_msg(KeyType::Backspace));

    assert_eq!(browser.filter(), "x");
}

#[test]
fn browser_model_page_keys() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);

    browser.update(key_msg(KeyType::End));
    let at_end = browser.selected_entry().unwrap().name.clone();

    browser.update(key_msg(KeyType::Home));
    let at_home = browser.selected_entry().unwrap().name.clone();

    assert_ne!(at_end, at_home);
}

// =============================================================================
// FileBrowser: view rendering
// =============================================================================

#[test]
fn browser_view_contains_entries() {
    let dir = make_dir();
    let browser = populated_browser(&dir);
    let view = browser.view();
    assert!(view.contains("alpha.md"));
    assert!(view.contains("beta.md"));
}

#[test]
fn browser_view_empty_dir_message() {
    let dir = make_dir();
    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();
    let view = browser.view();
    assert!(view.contains("No markdown files found"));
}

#[test]
fn browser_view_no_matches_message() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.set_filter("zzzzz");
    let view = browser.view();
    assert!(view.contains("No matches"));
}

#[test]
fn browser_view_filter_mode_shows_cursor() {
    let dir = make_dir();
    let mut browser = populated_browser(&dir);
    browser.enter_filter_mode();
    browser.filter_input('t');
    let view = browser.view();
    assert!(view.contains("/t_")); // cursor indicator
}

#[test]
fn browser_view_contains_help() {
    let dir = make_dir();
    let browser = populated_browser(&dir);
    let view = browser.view();
    assert!(view.contains("quit"));
    assert!(view.contains("filter"));
}

// =============================================================================
// FileBrowser: toggle hidden
// =============================================================================

#[test]
fn browser_toggle_hidden() {
    let dir = make_dir();
    touch(dir.path(), ".secret.md");
    touch(dir.path(), "visible.md");

    let mut browser =
        FileBrowser::with_directory(dir.path(), BrowserConfig::default()).unwrap();
    browser.scan().unwrap();
    let without_hidden = browser.entries().len();

    browser.toggle_hidden().unwrap();
    let with_hidden = browser.entries().len();

    assert!(with_hidden > without_hidden);
}

// =============================================================================
// FileBrowser: builder methods
// =============================================================================

#[test]
fn browser_height_builder() {
    let browser = FileBrowser::new(BrowserConfig::default()).height(10);
    // Can't directly observe height but shouldn't panic
    let _ = browser.view();
}

#[test]
fn browser_focused_builder() {
    let browser = FileBrowser::new(BrowserConfig::default()).focused(false);
    let _ = browser.view();
}

// =============================================================================
// Reader: edge cases
// =============================================================================

#[test]
fn reader_render_file_from_disk() {
    let dir = make_dir();
    write_file(dir.path(), "test.md", "# Hello\n\nWorld.\n");

    let reader = Reader::new(Config::new().style("ascii").width(80));
    let result = reader.read_file(dir.path().join("test.md"));
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Hello"));
    assert!(output.contains("World"));
}

#[test]
fn reader_all_valid_styles() {
    let styles = ["dark", "light", "ascii", "pink", "auto", "no-tty", "notty", "no_tty"];
    for style in &styles {
        let reader = Reader::new(Config::new().style(*style));
        let result = reader.render_markdown("# Test");
        assert!(result.is_ok(), "style '{style}' should work");
    }
}

#[test]
fn reader_invalid_style_errors() {
    let reader = Reader::new(Config::new().style("nonexistent"));
    let result = reader.render_markdown("# Test");
    assert!(result.is_err());
}

#[test]
fn reader_width_option() {
    let reader = Reader::new(Config::new().style("ascii").width(40));
    let long_text = "This is a paragraph with quite a few words that should wrap at narrow widths.";
    let result = reader.render_markdown(long_text).unwrap();
    // With width 40 and ascii style, lines should be relatively short
    for line in result.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            assert!(
                trimmed.len() <= 50,
                "line too long at width 40: '{}' ({} chars)",
                trimmed,
                trimmed.len()
            );
        }
    }
}

#[test]
fn reader_preserve_newlines_option() {
    let config = Config::new()
        .style("ascii")
        .preserve_newlines(true);
    let reader = Reader::new(config);
    let result = reader.render_markdown("Line 1\nLine 2");
    assert!(result.is_ok());
}

#[test]
fn reader_line_numbers_option() {
    let config = Config::new()
        .style("ascii")
        .line_numbers(true);
    let reader = Reader::new(config);
    let result = reader.render_markdown("```\ncode\n```");
    assert!(result.is_ok());
}

// =============================================================================
// Config: builder chain completeness
// =============================================================================

#[test]
fn config_full_chain() {
    let config = Config::new()
        .pager(false)
        .width(120)
        .style("light")
        .line_numbers(true)
        .preserve_newlines(true);

    let reader = Reader::new(config);
    // Config fields are private; just verify it renders
    let result = reader.render_markdown("# Test");
    assert!(result.is_ok());
}

// =============================================================================
// Stash: edge cases
// =============================================================================

#[test]
fn stash_add_duplicates_allowed() {
    let mut stash = Stash::new();
    stash.add("file.md");
    stash.add("file.md");
    assert_eq!(stash.documents().len(), 2);
}

#[test]
fn stash_add_empty_string() {
    let mut stash = Stash::new();
    stash.add("");
    assert_eq!(stash.documents().len(), 1);
    assert_eq!(stash.documents()[0], "");
}

#[test]
fn stash_preserves_insertion_order() {
    let mut stash = Stash::new();
    stash.add("c.md");
    stash.add("a.md");
    stash.add("b.md");
    assert_eq!(stash.documents(), &["c.md", "a.md", "b.md"]);
}
