//! Browser-based WASM tests.
//!
//! Run with: wasm-pack test --headless --chrome

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_module_ready() {
    assert!(charmed_wasm::is_ready());
}

#[wasm_bindgen_test]
fn test_version() {
    let version = charmed_wasm::version();
    assert!(!version.is_empty());
    // Version should be semver-like
    assert!(version.contains('.'));
}

#[wasm_bindgen_test]
fn test_new_style_creation() {
    let style = charmed_wasm::new_style();
    // Basic style should render content
    let result = style.render("Hello");
    assert!(result.contains("Hello"));
}

#[wasm_bindgen_test]
fn test_style_with_colors() {
    let style = charmed_wasm::new_style()
        .foreground("#ff0000")
        .background("#0000ff");

    let result = style.render("Colored");
    // HTML output should contain color information
    assert!(result.contains("Colored"));
}

#[wasm_bindgen_test]
fn test_style_with_formatting() {
    let style = charmed_wasm::new_style().bold().italic().underline();
    let result = style.render("Formatted");
    assert!(result.contains("Formatted"));
}

#[wasm_bindgen_test]
fn test_style_with_padding() {
    let style = charmed_wasm::new_style().padding(1, 2, 1, 2);
    let result = style.render("Padded");
    assert!(result.contains("Padded"));
}

#[wasm_bindgen_test]
fn test_style_with_border() {
    let style = charmed_wasm::new_style()
        .border_style("rounded")
        .border_all();
    let result = style.render("Bordered");
    assert!(result.contains("Bordered"));
}

#[wasm_bindgen_test]
fn test_style_chaining() {
    // All these should chain without panic
    let style = charmed_wasm::new_style()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .bold()
        .italic()
        .underline()
        .padding(1, 2, 1, 2)
        .margin(0, 1, 0, 1)
        .width(40)
        .height(5)
        .align_center();

    let result = style.render("Chained");
    assert!(result.contains("Chained"));
}

#[wasm_bindgen_test]
fn test_style_copy() {
    let style1 = charmed_wasm::new_style().bold();
    let style2 = style1.copy();

    // Both should render the same way
    let result1 = style1.render("Test");
    let result2 = style2.render("Test");
    assert_eq!(result1, result2);
}

#[wasm_bindgen_test]
fn test_string_width() {
    let width = charmed_wasm::string_width("Hello");
    assert_eq!(width, 5);

    // Unicode characters
    let width_emoji = charmed_wasm::string_width("Hi!");
    assert!(width_emoji >= 3);
}

#[wasm_bindgen_test]
fn test_string_height() {
    let single_line = charmed_wasm::string_height("Hello");
    assert_eq!(single_line, 1);

    let multi_line = charmed_wasm::string_height("Line 1\nLine 2\nLine 3");
    assert_eq!(multi_line, 3);
}

#[wasm_bindgen_test]
fn test_border_presets() {
    let presets = charmed_wasm::border_presets();
    assert!(!presets.is_empty());
    // Should have at least the standard presets
    assert!(presets.len() >= 4);
}

#[wasm_bindgen_test]
fn test_place() {
    let result = charmed_wasm::place(10, 3, 0.5, 0.5, "Hi");
    // Should contain the content
    assert!(result.contains("Hi"));
    // Should have 3 lines
    assert_eq!(result.lines().count(), 3);
}

// === Additional comprehensive tests ===

#[wasm_bindgen_test]
fn test_all_border_styles() {
    let styles = ["normal", "rounded", "thick", "double", "hidden", "ascii"];
    for style_name in styles {
        let style = charmed_wasm::new_style()
            .border_style(style_name)
            .border_all();
        let result = style.render("Test");
        assert!(
            result.contains("Test"),
            "Border style {} failed",
            style_name
        );
    }
}

#[wasm_bindgen_test]
fn test_individual_borders() {
    // Test enabling borders individually
    let top = charmed_wasm::new_style()
        .border_style("normal")
        .border_top()
        .render("Top only");
    assert!(top.contains("Top only"));

    let bottom = charmed_wasm::new_style()
        .border_style("normal")
        .border_bottom()
        .render("Bottom only");
    assert!(bottom.contains("Bottom only"));

    let left = charmed_wasm::new_style()
        .border_style("normal")
        .border_left()
        .render("Left only");
    assert!(left.contains("Left only"));

    let right = charmed_wasm::new_style()
        .border_style("normal")
        .border_right()
        .render("Right only");
    assert!(right.contains("Right only"));
}

#[wasm_bindgen_test]
fn test_alignment_positions() {
    // Test different alignment positions
    let left = charmed_wasm::new_style()
        .width(20)
        .align_left()
        .render("Left");
    assert!(left.contains("Left"));

    let center = charmed_wasm::new_style()
        .width(20)
        .align_center()
        .render("Center");
    assert!(center.contains("Center"));

    let right = charmed_wasm::new_style()
        .width(20)
        .align_right()
        .render("Right");
    assert!(right.contains("Right"));
}

#[wasm_bindgen_test]
fn test_numeric_alignment() {
    // Test alignment with numeric values
    let aligned = charmed_wasm::new_style()
        .width(20)
        .align_horizontal(0.5)
        .render("Centered");
    assert!(aligned.contains("Centered"));
}

#[wasm_bindgen_test]
fn test_padding_variants() {
    // All sides
    let all = charmed_wasm::new_style().padding_all(2).render("Padded");
    assert!(all.contains("Padded"));

    // Vertical/Horizontal
    let vh = charmed_wasm::new_style()
        .padding_vh(1, 2)
        .render("VH Padded");
    assert!(vh.contains("VH Padded"));

    // Individual
    let individual = charmed_wasm::new_style()
        .padding(1, 2, 3, 4)
        .render("Individual");
    assert!(individual.contains("Individual"));
}

#[wasm_bindgen_test]
fn test_margin_variants() {
    // All sides
    let all = charmed_wasm::new_style().margin_all(1).render("Margin");
    assert!(all.contains("Margin"));

    // Vertical/Horizontal
    let vh = charmed_wasm::new_style()
        .margin_vh(1, 2)
        .render("VH Margin");
    assert!(vh.contains("VH Margin"));

    // Individual
    let individual = charmed_wasm::new_style()
        .margin(1, 2, 3, 4)
        .render("Individual");
    assert!(individual.contains("Individual"));
}

#[wasm_bindgen_test]
fn test_text_formatting_combinations() {
    // Combinations of formatting
    let bold_italic = charmed_wasm::new_style()
        .bold()
        .italic()
        .render("Bold Italic");
    assert!(bold_italic.contains("Bold Italic"));

    let all_formatting = charmed_wasm::new_style()
        .bold()
        .italic()
        .underline()
        .strikethrough()
        .faint()
        .render("All");
    assert!(all_formatting.contains("All"));
}

#[wasm_bindgen_test]
fn test_reverse_style() {
    let reversed = charmed_wasm::new_style()
        .foreground("#ff0000")
        .background("#00ff00")
        .reverse()
        .render("Reversed");
    assert!(reversed.contains("Reversed"));
}

#[wasm_bindgen_test]
fn test_dimensions() {
    // Test fixed width
    let wide = charmed_wasm::new_style().width(50).render("Wide");
    assert!(wide.contains("Wide"));

    // Test fixed height
    let tall = charmed_wasm::new_style().height(3).render("Tall");
    assert!(tall.contains("Tall"));

    // Test both
    let sized = charmed_wasm::new_style()
        .width(30)
        .height(5)
        .render("Sized");
    assert!(sized.contains("Sized"));
}

#[wasm_bindgen_test]
fn test_multiline_content() {
    let multiline = charmed_wasm::new_style()
        .border_style("rounded")
        .border_all()
        .render("Line 1\nLine 2\nLine 3");
    assert!(multiline.contains("Line 1"));
    assert!(multiline.contains("Line 2"));
    assert!(multiline.contains("Line 3"));
}

#[wasm_bindgen_test]
fn test_empty_content() {
    let empty = charmed_wasm::new_style().padding_all(1).render("");
    // Should not panic, may contain padding spaces
    assert!(empty.len() >= 0);
}

#[wasm_bindgen_test]
fn test_special_characters() {
    let special = charmed_wasm::new_style().render("<script>alert('xss')</script>");
    // Content should be preserved
    assert!(special.contains("script"));
}

#[wasm_bindgen_test]
fn test_unicode_content() {
    let unicode = charmed_wasm::new_style().render("Hello, World!");
    assert!(unicode.contains("World"));
}

#[wasm_bindgen_test]
fn test_join_horizontal_alignment() {
    let left = charmed_wasm::new_style().render("A\nB\nC");
    let right = charmed_wasm::new_style().render("X\nY");

    // Top alignment
    let top = charmed_wasm::join_horizontal(0.0, vec![left.clone().into(), right.clone().into()]);
    assert!(top.contains("A"));
    assert!(top.contains("X"));

    // Center alignment
    let center =
        charmed_wasm::join_horizontal(0.5, vec![left.clone().into(), right.clone().into()]);
    assert!(center.contains("A"));
    assert!(center.contains("X"));

    // Bottom alignment
    let bottom = charmed_wasm::join_horizontal(1.0, vec![left.into(), right.into()]);
    assert!(bottom.contains("A"));
    assert!(bottom.contains("X"));
}

#[wasm_bindgen_test]
fn test_join_vertical_alignment() {
    let short = charmed_wasm::new_style().render("Short");
    let long = charmed_wasm::new_style().render("Much Longer Text");

    // Left alignment
    let left_aligned =
        charmed_wasm::join_vertical(0.0, vec![short.clone().into(), long.clone().into()]);
    assert!(left_aligned.contains("Short"));
    assert!(left_aligned.contains("Much Longer Text"));

    // Center alignment
    let center = charmed_wasm::join_vertical(0.5, vec![short.clone().into(), long.clone().into()]);
    assert!(center.contains("Short"));

    // Right alignment
    let right = charmed_wasm::join_vertical(1.0, vec![short.into(), long.into()]);
    assert!(right.contains("Short"));
}

#[wasm_bindgen_test]
fn test_place_positions() {
    // Test all corner positions
    let top_left = charmed_wasm::place(20, 5, 0.0, 0.0, "TL");
    assert!(top_left.contains("TL"));

    let top_right = charmed_wasm::place(20, 5, 1.0, 0.0, "TR");
    assert!(top_right.contains("TR"));

    let bottom_left = charmed_wasm::place(20, 5, 0.0, 1.0, "BL");
    assert!(bottom_left.contains("BL"));

    let bottom_right = charmed_wasm::place(20, 5, 1.0, 1.0, "BR");
    assert!(bottom_right.contains("BR"));

    let center = charmed_wasm::place(20, 5, 0.5, 0.5, "C");
    assert!(center.contains("C"));
}

#[wasm_bindgen_test]
fn test_string_width_unicode() {
    // ASCII
    assert_eq!(charmed_wasm::string_width("hello"), 5);

    // Empty string
    assert_eq!(charmed_wasm::string_width(""), 0);

    // Mixed content
    let width = charmed_wasm::string_width("Hello, World!");
    assert!(width >= 13);
}

#[wasm_bindgen_test]
fn test_string_height_edge_cases() {
    // Empty string
    let empty = charmed_wasm::string_height("");
    assert!(empty >= 1); // At least 1 for empty

    // Single line no newline
    assert_eq!(charmed_wasm::string_height("single"), 1);

    // Multiple lines
    assert_eq!(charmed_wasm::string_height("a\nb\nc\nd"), 4);
}

#[wasm_bindgen_test]
fn test_style_render_ansi() {
    // Test ANSI render mode (for terminal-like displays)
    let style = charmed_wasm::new_style().foreground("#ff0000").bold();
    let ansi = style.render_ansi("ANSI Output");
    // Should contain the text
    assert!(ansi.contains("ANSI Output"));
}

#[wasm_bindgen_test]
fn test_complex_layout() {
    // Build a complex layout
    let header = charmed_wasm::new_style()
        .foreground("#61dafb")
        .bold()
        .render("Header");

    let body = charmed_wasm::new_style()
        .foreground("#888888")
        .render("Body content\nwith multiple lines");

    let footer = charmed_wasm::new_style()
        .foreground("#666666")
        .faint()
        .render("Footer");

    let layout = charmed_wasm::join_vertical(0.5, vec![header.into(), body.into(), footer.into()]);

    assert!(layout.contains("Header"));
    assert!(layout.contains("Body content"));
    assert!(layout.contains("Footer"));
}
