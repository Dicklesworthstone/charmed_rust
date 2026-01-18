// Glamour capture program - captures markdown rendering behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/glamour"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("glamour", "0.8.0")

	// Capture basic markdown rendering
	captureBasicMarkdownTests(fixtures)

	// Capture heading tests
	captureHeadingTests(fixtures)

	// Capture text formatting tests
	captureTextFormattingTests(fixtures)

	// Capture list tests
	captureListTests(fixtures)

	// Capture code block tests
	captureCodeBlockTests(fixtures)

	// Capture link tests
	captureLinkTests(fixtures)

	// Capture blockquote tests
	captureBlockquoteTests(fixtures)

	// Capture horizontal rule tests
	captureHorizontalRuleTests(fixtures)

	// Capture table tests
	captureTableTests(fixtures)

	// Capture style preset tests
	captureStylePresetTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureBasicMarkdownTests(fs *capture.FixtureSet) {
	// Basic text with no formatting
	testCases := []struct {
		name  string
		input string
	}{
		{"plain_text", "Hello, World!"},
		{"plain_text_multiline", "Line 1\nLine 2\nLine 3"},
		{"empty", ""},
		{"whitespace_only", "   \n   \n   "},
		{"paragraph", "This is a paragraph of text that spans multiple words and should be rendered as a single paragraph."},
		{"two_paragraphs", "First paragraph.\n\nSecond paragraph."},
	}

	for _, tc := range testCases {
		out, err := glamour.Render(tc.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		if err != nil {
			result["error_msg"] = err.Error()
		}
		fs.AddTestWithCategory(fmt.Sprintf("basic_%s", tc.name), "unit",
			map[string]string{
				"input": tc.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureHeadingTests(fs *capture.FixtureSet) {
	headings := []struct {
		name  string
		input string
		level int
	}{
		{"h1", "# Heading 1", 1},
		{"h2", "## Heading 2", 2},
		{"h3", "### Heading 3", 3},
		{"h4", "#### Heading 4", 4},
		{"h5", "##### Heading 5", 5},
		{"h6", "###### Heading 6", 6},
		{"h1_alt", "Heading 1\n=========", 1},
		{"h2_alt", "Heading 2\n---------", 2},
	}

	for _, h := range headings {
		out, err := glamour.Render(h.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("heading_%s", h.name), "unit",
			map[string]interface{}{
				"input": h.input,
				"level": h.level,
				"style": "dark",
			},
			result,
		)
	}
}

func captureTextFormattingTests(fs *capture.FixtureSet) {
	formats := []struct {
		name  string
		input string
	}{
		{"bold_asterisk", "**bold text**"},
		{"bold_underscore", "__bold text__"},
		{"italic_asterisk", "*italic text*"},
		{"italic_underscore", "_italic text_"},
		{"bold_italic", "***bold and italic***"},
		{"strikethrough", "~~strikethrough~~"},
		{"inline_code", "`inline code`"},
		{"mixed", "Normal **bold** and *italic* and `code`"},
		{"nested_bold_italic", "**bold _and italic_**"},
	}

	for _, f := range formats {
		out, err := glamour.Render(f.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("format_%s", f.name), "unit",
			map[string]string{
				"input": f.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureListTests(fs *capture.FixtureSet) {
	lists := []struct {
		name  string
		input string
	}{
		{"unordered_dash", "- Item 1\n- Item 2\n- Item 3"},
		{"unordered_asterisk", "* Item 1\n* Item 2\n* Item 3"},
		{"unordered_plus", "+ Item 1\n+ Item 2\n+ Item 3"},
		{"ordered", "1. First\n2. Second\n3. Third"},
		{"ordered_all_ones", "1. First\n1. Second\n1. Third"},
		{"nested_unordered", "- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2"},
		{"nested_ordered", "1. First\n   1. Nested 1\n   2. Nested 2\n2. Second"},
		{"mixed_nested", "1. First\n   - Sub item\n   - Sub item\n2. Second"},
		{"task_list", "- [ ] Unchecked\n- [x] Checked\n- [ ] Another"},
	}

	for _, l := range lists {
		out, err := glamour.Render(l.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("list_%s", l.name), "unit",
			map[string]string{
				"input": l.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureCodeBlockTests(fs *capture.FixtureSet) {
	codeBlocks := []struct {
		name  string
		input string
	}{
		{"fenced_no_lang", "```\ncode here\n```"},
		{"fenced_go", "```go\nfunc main() {\n\tfmt.Println(\"Hello\")\n}\n```"},
		{"fenced_python", "```python\ndef hello():\n    print(\"Hello\")\n```"},
		{"fenced_rust", "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```"},
		{"fenced_json", "```json\n{\"key\": \"value\"}\n```"},
		{"indented", "    indented code\n    more code"},
	}

	for _, cb := range codeBlocks {
		out, err := glamour.Render(cb.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("code_%s", cb.name), "unit",
			map[string]string{
				"input": cb.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureLinkTests(fs *capture.FixtureSet) {
	links := []struct {
		name  string
		input string
	}{
		{"inline", "[Link text](https://example.com)"},
		{"inline_title", "[Link text](https://example.com \"Title\")"},
		{"reference", "[Link text][ref]\n\n[ref]: https://example.com"},
		{"autolink", "<https://example.com>"},
		{"autolink_email", "<user@example.com>"},
		{"image", "![Alt text](https://example.com/image.png)"},
		{"image_title", "![Alt text](https://example.com/image.png \"Title\")"},
	}

	for _, l := range links {
		out, err := glamour.Render(l.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("link_%s", l.name), "unit",
			map[string]string{
				"input": l.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureBlockquoteTests(fs *capture.FixtureSet) {
	quotes := []struct {
		name  string
		input string
	}{
		{"single_line", "> This is a quote"},
		{"multi_line", "> Line 1\n> Line 2\n> Line 3"},
		{"multi_paragraph", "> Paragraph 1\n>\n> Paragraph 2"},
		{"nested", "> Outer\n>> Inner\n> Back to outer"},
		{"with_formatting", "> **Bold** in quote\n> *Italic* in quote"},
	}

	for _, q := range quotes {
		out, err := glamour.Render(q.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("blockquote_%s", q.name), "unit",
			map[string]string{
				"input": q.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureHorizontalRuleTests(fs *capture.FixtureSet) {
	rules := []struct {
		name  string
		input string
	}{
		{"dashes", "---"},
		{"asterisks", "***"},
		{"underscores", "___"},
		{"dashes_spaced", "- - -"},
		{"many_dashes", "----------"},
		{"between_text", "Above\n\n---\n\nBelow"},
	}

	for _, r := range rules {
		out, err := glamour.Render(r.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("hr_%s", r.name), "unit",
			map[string]string{
				"input": r.input,
				"style": "dark",
			},
			result,
		)
	}
}

func captureTableTests(fs *capture.FixtureSet) {
	tables := []struct {
		name  string
		input string
	}{
		// Basic tables
		{"simple_2x2", "| A | B |\n|---|---|\n| 1 | 2 |"},
		{"simple_3x3", "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |"},
		{"headers_only", "| Header 1 | Header 2 |\n|----------|----------|"},

		// Alignment tests
		{"align_left", "| Left |\n|:-----|\n| text |"},
		{"align_center", "| Center |\n|:------:|\n| text |"},
		{"align_right", "| Right |\n|------:|\n| text |"},
		{"align_mixed", "| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |"},

		// Column width tests
		{"wide_content", "| Short | Very Long Column Content |\n|-------|-------------------------|\n| A | B |"},
		{"varying_widths", "| X | Medium | Very Very Long Content Here |\n|---|--------|----------------------------|\n| 1 | 2 | 3 |"},

		// Formatting in cells
		{"bold_in_cell", "| Normal | **Bold** |\n|--------|----------|\n| A | B |"},
		{"italic_in_cell", "| Normal | *Italic* |\n|--------|----------|\n| A | B |"},
		{"code_in_cell", "| Normal | `code` |\n|--------|--------|\n| A | B |"},
		{"mixed_formatting", "| **Bold** | *Italic* | `code` |\n|----------|----------|--------|\n| 1 | 2 | 3 |"},

		// Edge cases
		{"empty_cells", "| A | | C |\n|---|---|---|\n| | B | |"},
		{"single_column", "| Single |\n|--------|\n| Value |"},
		{"many_columns", "| A | B | C | D | E | F |\n|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 |"},
		{"many_rows", "| A |\n|---|\n| 1 |\n| 2 |\n| 3 |\n| 4 |\n| 5 |"},

		// Unicode content
		{"unicode_content", "| Emoji | CJK | Symbols |\n|-------|-----|----------|\n| 🎉 | 中文 | ★ |"},

		// Tables with surrounding content
		{"with_paragraph", "Some text before.\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\nSome text after."},
		{"with_heading", "# Heading\n\n| A | B |\n|---|---|\n| 1 | 2 |"},
	}

	for _, t := range tables {
		out, err := glamour.Render(t.input, "dark")
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("table_%s", t.name), "unit",
			map[string]string{
				"input": t.input,
				"style": "dark",
			},
			result,
		)
	}

	// Also test tables with different style presets
	tableMarkdown := "| Header 1 | Header 2 |\n|:---------|:---------|\n| Cell 1 | Cell 2 |"
	stylePresets := []string{"ascii", "light", "notty"}
	for _, preset := range stylePresets {
		out, err := glamour.Render(tableMarkdown, preset)
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		fs.AddTestWithCategory(fmt.Sprintf("table_style_%s", preset), "unit",
			map[string]string{
				"input": tableMarkdown,
				"style": preset,
			},
			result,
		)
	}
}

func captureStylePresetTests(fs *capture.FixtureSet) {
	// Test different style presets
	presets := []string{"dark", "light", "notty", "ascii", "dracula"}
	testMarkdown := "# Heading\n\nA **bold** paragraph with *italic* and `code`.\n\n- Item 1\n- Item 2"

	for _, preset := range presets {
		out, err := glamour.Render(testMarkdown, preset)
		result := map[string]interface{}{
			"output": out,
			"error":  err != nil,
		}
		if err != nil {
			result["error_msg"] = err.Error()
		}
		fs.AddTestWithCategory(fmt.Sprintf("style_preset_%s", preset), "unit",
			map[string]string{
				"input": testMarkdown,
				"style": preset,
			},
			result,
		)
	}
}
