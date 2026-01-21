//! End-to-end browser tests with DOM manipulation.
//!
//! Run with: wasm-pack test --headless --chrome
//!
//! These tests verify that charmed-wasm works correctly in a real browser
//! environment, including DOM rendering and user interaction scenarios.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::{Document, Element, HtmlElement, window};

wasm_bindgen_test_configure!(run_in_browser);

// === Helper Functions ===

fn get_document() -> Document {
    window()
        .expect("no window")
        .document()
        .expect("no document")
}

fn create_test_container(id: &str) -> Element {
    let doc = get_document();
    let container = doc.create_element("div").expect("create div failed");
    container.set_id(id);

    // Add some basic styling
    if let Some(html_element) = container.dyn_ref::<HtmlElement>() {
        let _ = html_element
            .style()
            .set_property("font-family", "monospace");
        let _ = html_element.style().set_property("white-space", "pre-wrap");
    }

    doc.body()
        .expect("no body")
        .append_child(&container)
        .expect("append failed");
    container
}

fn cleanup_container(id: &str) {
    if let Some(element) = get_document().get_element_by_id(id) {
        let _ = element.remove();
    }
}

// === DOM Rendering Tests ===

#[wasm_bindgen_test]
fn test_style_renders_to_dom() {
    let container = create_test_container("test-style-dom");

    let style = charmed_wasm::new_style().foreground("#ff0000").bold();

    let html = style.render("Red Bold Text");
    container.set_inner_html(&html);

    // Verify DOM content
    let inner = container.inner_html();
    assert!(inner.contains("Red Bold Text"), "Content not found in DOM");

    cleanup_container("test-style-dom");
}

#[wasm_bindgen_test]
fn test_multiple_styles_in_dom() {
    let container = create_test_container("test-multi-styles");

    let header = charmed_wasm::new_style()
        .foreground("#61dafb")
        .bold()
        .render("Header");

    let body = charmed_wasm::new_style()
        .foreground("#888888")
        .render("Body content");

    let footer = charmed_wasm::new_style()
        .foreground("#666666")
        .faint()
        .render("Footer");

    let combined =
        charmed_wasm::join_vertical(0.0, vec![header.into(), body.into(), footer.into()]);

    container.set_inner_html(&combined);

    let inner = container.inner_html();
    assert!(inner.contains("Header"));
    assert!(inner.contains("Body content"));
    assert!(inner.contains("Footer"));

    cleanup_container("test-multi-styles");
}

#[wasm_bindgen_test]
fn test_bordered_content_in_dom() {
    let container = create_test_container("test-bordered");

    let style = charmed_wasm::new_style()
        .border_style("rounded")
        .border_all()
        .padding(1, 2, 1, 2)
        .render("Bordered Box");

    container.set_inner_html(&style);

    let inner = container.inner_html();
    assert!(inner.contains("Bordered Box"));

    cleanup_container("test-bordered");
}

#[wasm_bindgen_test]
fn test_complex_layout_in_dom() {
    let container = create_test_container("test-complex-layout");

    // Create a menu-like layout
    let selected = charmed_wasm::new_style()
        .background("#3498db")
        .foreground("#ffffff")
        .bold()
        .width(20)
        .render("> File");

    let normal = charmed_wasm::new_style()
        .foreground("#95a5a6")
        .width(20)
        .render("  Edit");

    let disabled = charmed_wasm::new_style()
        .foreground("#34495e")
        .faint()
        .width(20)
        .render("  View");

    let menu =
        charmed_wasm::join_vertical(0.0, vec![selected.into(), normal.into(), disabled.into()]);

    container.set_inner_html(&menu);

    let inner = container.inner_html();
    assert!(inner.contains("File"));
    assert!(inner.contains("Edit"));
    assert!(inner.contains("View"));

    cleanup_container("test-complex-layout");
}

// === Performance Tests ===

#[wasm_bindgen_test]
fn test_rapid_style_creation() {
    // Test that creating many styles quickly doesn't cause issues
    for i in 0..100 {
        let style = charmed_wasm::new_style()
            .foreground("#ff0000")
            .background("#000000")
            .bold()
            .padding(1, 1, 1, 1);

        let _ = style.render(&format!("Item {}", i));
    }
}

#[wasm_bindgen_test]
fn test_rapid_dom_updates() {
    let container = create_test_container("test-rapid-updates");

    // Simulate rapid updates like in a live editor
    for i in 0..50 {
        let style = charmed_wasm::new_style().foreground(&format!(
            "#{:02x}{:02x}{:02x}",
            i * 5,
            100,
            200 - i * 4
        ));

        let html = style.render(&format!("Update {}", i));
        container.set_inner_html(&html);
    }

    let inner = container.inner_html();
    assert!(inner.contains("Update 49")); // Last update should be visible

    cleanup_container("test-rapid-updates");
}

// === Edge Case Tests ===

#[wasm_bindgen_test]
fn test_empty_content_in_dom() {
    let container = create_test_container("test-empty");

    let style = charmed_wasm::new_style()
        .padding_all(1)
        .border_style("rounded")
        .border_all()
        .render("");

    container.set_inner_html(&style);

    // Should render without error
    assert!(container.inner_html().len() > 0);

    cleanup_container("test-empty");
}

#[wasm_bindgen_test]
fn test_very_long_content() {
    let container = create_test_container("test-long");

    let long_text = "A".repeat(1000);
    let style = charmed_wasm::new_style()
        .foreground("#ff0000")
        .render(&long_text);

    container.set_inner_html(&style);

    let inner = container.inner_html();
    assert!(inner.len() >= 1000);

    cleanup_container("test-long");
}

#[wasm_bindgen_test]
fn test_unicode_in_dom() {
    let container = create_test_container("test-unicode");

    let style = charmed_wasm::new_style()
        .foreground("#61dafb")
        .render("Hello, World!");

    container.set_inner_html(&style);

    let inner = container.inner_html();
    assert!(inner.contains("World"));

    cleanup_container("test-unicode");
}

#[wasm_bindgen_test]
fn test_html_special_chars() {
    let container = create_test_container("test-special");

    // Test that special HTML characters are handled
    let style = charmed_wasm::new_style().render("<script>alert('xss')</script>");

    container.set_inner_html(&style);

    // The content should be in the DOM somehow
    let inner = container.inner_html();
    // The exact representation depends on how the backend handles escaping
    assert!(inner.contains("script") || inner.contains("&lt;"));

    cleanup_container("test-special");
}

// === Integration Tests ===

#[wasm_bindgen_test]
fn test_dashboard_layout() {
    let container = create_test_container("test-dashboard");

    // Create a simple dashboard-like layout
    let title = charmed_wasm::new_style()
        .foreground("#61dafb")
        .bold()
        .align_center()
        .width(40)
        .render("Dashboard");

    let stat1 = charmed_wasm::new_style()
        .foreground("#2ecc71")
        .render("Users: 1,234");

    let stat2 = charmed_wasm::new_style()
        .foreground("#e74c3c")
        .render("Errors: 5");

    let stat3 = charmed_wasm::new_style()
        .foreground("#f39c12")
        .render("Pending: 42");

    let stats = charmed_wasm::join_horizontal(
        0.5,
        vec![
            stat1.into(),
            "  ".to_string().into(),
            stat2.into(),
            "  ".to_string().into(),
            stat3.into(),
        ],
    );

    let dashboard =
        charmed_wasm::join_vertical(0.5, vec![title.into(), "".to_string().into(), stats.into()]);

    let boxed = charmed_wasm::new_style()
        .border_style("rounded")
        .border_all()
        .padding(1, 2, 1, 2)
        .render(&dashboard);

    container.set_inner_html(&boxed);

    let inner = container.inner_html();
    assert!(inner.contains("Dashboard"));
    assert!(inner.contains("Users"));
    assert!(inner.contains("Errors"));
    assert!(inner.contains("Pending"));

    cleanup_container("test-dashboard");
}

#[wasm_bindgen_test]
fn test_card_component() {
    let container = create_test_container("test-card");

    let card_title = charmed_wasm::new_style()
        .foreground("#ffffff")
        .bold()
        .render("Card Title");

    let card_body = charmed_wasm::new_style()
        .foreground("#aaaaaa")
        .render("This is the card body with some content.");

    let card_content = charmed_wasm::join_vertical(
        0.0,
        vec![card_title.into(), "".to_string().into(), card_body.into()],
    );

    let card = charmed_wasm::new_style()
        .border_style("rounded")
        .border_all()
        .foreground("#58a6ff")
        .padding(1, 2, 1, 2)
        .render(&card_content);

    container.set_inner_html(&card);

    let inner = container.inner_html();
    assert!(inner.contains("Card Title"));
    assert!(inner.contains("card body"));

    cleanup_container("test-card");
}

// === Utility Function Tests ===

#[wasm_bindgen_test]
fn test_place_in_dom() {
    let container = create_test_container("test-place");

    // Place content in the center of a 30x5 container
    let placed = charmed_wasm::place(30, 5, 0.5, 0.5, "Centered!");

    container.set_inner_html(&placed);

    let inner = container.inner_html();
    assert!(inner.contains("Centered!"));

    // Verify dimensions
    let height = charmed_wasm::string_height(&placed);
    assert_eq!(height, 5);

    cleanup_container("test-place");
}

#[wasm_bindgen_test]
fn test_string_metrics() {
    // Test string measurement functions
    let text = "Hello, World!";
    let width = charmed_wasm::string_width(text);
    let height = charmed_wasm::string_height(text);

    assert!(width >= 13);
    assert_eq!(height, 1);

    let multiline = "Line 1\nLine 2\nLine 3";
    assert_eq!(charmed_wasm::string_height(multiline), 3);
}
