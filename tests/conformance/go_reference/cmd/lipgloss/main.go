// Lipgloss capture program - captures styling and rendering behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("lipgloss", "1.1.0")

	// Capture basic style tests
	captureBasicStyleTests(fixtures)

	// Capture color tests
	captureColorTests(fixtures)

	// Capture border tests
	captureBorderTests(fixtures)

	// Capture padding and margin tests
	capturePaddingMarginTests(fixtures)

	// Capture dimension tests
	captureDimensionTests(fixtures)

	// Capture alignment tests
	captureAlignmentTests(fixtures)

	// Capture join tests
	captureJoinTests(fixtures)

	// Capture place tests
	capturePlaceTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureBasicStyleTests(fs *capture.FixtureSet) {
	// Test 1: Plain text (no styling)
	{
		style := lipgloss.NewStyle()
		rendered := style.Render("Hello")
		fs.AddTestWithCategory("style_plain", "unit",
			capture.StyleInput{
				Text: "Hello",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,
				Height:   1,
			},
		)
	}

	// Test 2: Bold text
	{
		style := lipgloss.NewStyle().Bold(true)
		rendered := style.Render("Bold")
		fs.AddTestWithCategory("style_bold", "unit",
			capture.StyleInput{
				Text: "Bold",
				Bold: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    4,
				Height:   1,
			},
		)
	}

	// Test 3: Italic text
	{
		style := lipgloss.NewStyle().Italic(true)
		rendered := style.Render("Italic")
		fs.AddTestWithCategory("style_italic", "unit",
			capture.StyleInput{
				Text:   "Italic",
				Italic: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    6,
				Height:   1,
			},
		)
	}

	// Test 4: Underline text
	{
		style := lipgloss.NewStyle().Underline(true)
		rendered := style.Render("Underline")
		fs.AddTestWithCategory("style_underline", "unit",
			capture.StyleInput{
				Text:      "Underline",
				Underline: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    9,
				Height:   1,
			},
		)
	}

	// Test 5: Strikethrough text
	{
		style := lipgloss.NewStyle().Strikethrough(true)
		rendered := style.Render("Strike")
		fs.AddTestWithCategory("style_strikethrough", "unit",
			capture.StyleInput{
				Text:          "Strike",
				Strikethrough: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    6,
				Height:   1,
			},
		)
	}

	// Test 6: Faint text
	{
		style := lipgloss.NewStyle().Faint(true)
		rendered := style.Render("Faint")
		fs.AddTestWithCategory("style_faint", "unit",
			capture.StyleInput{
				Text:  "Faint",
				Faint: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,
				Height:   1,
			},
		)
	}

	// Test 7: Blink text
	{
		style := lipgloss.NewStyle().Blink(true)
		rendered := style.Render("Blink")
		fs.AddTestWithCategory("style_blink", "unit",
			capture.StyleInput{
				Text:  "Blink",
				Blink: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,
				Height:   1,
			},
		)
	}

	// Test 8: Reverse text
	{
		style := lipgloss.NewStyle().Reverse(true)
		rendered := style.Render("Reverse")
		fs.AddTestWithCategory("style_reverse", "unit",
			capture.StyleInput{
				Text:    "Reverse",
				Reverse: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    7,
				Height:   1,
			},
		)
	}

	// Test 9: Combined styles
	{
		style := lipgloss.NewStyle().Bold(true).Italic(true).Underline(true)
		rendered := style.Render("Combined")
		fs.AddTestWithCategory("style_combined", "unit",
			capture.StyleInput{
				Text:      "Combined",
				Bold:      true,
				Italic:    true,
				Underline: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    8,
				Height:   1,
			},
		)
	}

	// Test 10: Multi-line text
	{
		style := lipgloss.NewStyle().Bold(true)
		rendered := style.Render("Line1\nLine2\nLine3")
		fs.AddTestWithCategory("style_multiline", "unit",
			capture.StyleInput{
				Text: "Line1\nLine2\nLine3",
				Bold: true,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,
				Height:   3,
			},
		)
	}
}

func captureColorTests(fs *capture.FixtureSet) {
	// Test 1: ANSI foreground color
	{
		fg := "1" // Red
		style := lipgloss.NewStyle().Foreground(lipgloss.Color(fg))
		rendered := style.Render("Red")
		fs.AddTestWithCategory("color_ansi_fg", "unit",
			capture.StyleInput{
				Text:       "Red",
				Foreground: &fg,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    3,
				Height:   1,
			},
		)
	}

	// Test 2: ANSI background color
	{
		bg := "4" // Blue
		style := lipgloss.NewStyle().Background(lipgloss.Color(bg))
		rendered := style.Render("Blue")
		fs.AddTestWithCategory("color_ansi_bg", "unit",
			capture.StyleInput{
				Text:       "Blue",
				Background: &bg,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    4,
				Height:   1,
			},
		)
	}

	// Test 3: Hex foreground color
	{
		fg := "#FF0000"
		style := lipgloss.NewStyle().Foreground(lipgloss.Color(fg))
		rendered := style.Render("HexRed")
		fs.AddTestWithCategory("color_hex_fg", "unit",
			capture.StyleInput{
				Text:       "HexRed",
				Foreground: &fg,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    6,
				Height:   1,
			},
		)
	}

	// Test 4: Combined foreground and background
	{
		fg := "#FFFFFF"
		bg := "#0000FF"
		style := lipgloss.NewStyle().
			Foreground(lipgloss.Color(fg)).
			Background(lipgloss.Color(bg))
		rendered := style.Render("Contrast")
		fs.AddTestWithCategory("color_fg_bg_combined", "unit",
			capture.StyleInput{
				Text:       "Contrast",
				Foreground: &fg,
				Background: &bg,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    8,
				Height:   1,
			},
		)
	}

	// Test 5: ANSI 256 color
	{
		fg := "202" // Orange in 256-color palette
		style := lipgloss.NewStyle().Foreground(lipgloss.Color(fg))
		rendered := style.Render("Orange")
		fs.AddTestWithCategory("color_ansi256", "unit",
			capture.StyleInput{
				Text:       "Orange",
				Foreground: &fg,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    6,
				Height:   1,
			},
		)
	}
}

func captureBorderTests(fs *capture.FixtureSet) {
	// Test 1: Normal border
	{
		style := lipgloss.NewStyle().Border(lipgloss.NormalBorder())
		rendered := style.Render("Normal")
		fs.AddTestWithCategory("border_normal", "unit",
			capture.BorderInput{
				BorderType: "normal",
				Text:       "Normal",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 2: Rounded border
	{
		style := lipgloss.NewStyle().Border(lipgloss.RoundedBorder())
		rendered := style.Render("Rounded")
		fs.AddTestWithCategory("border_rounded", "unit",
			capture.BorderInput{
				BorderType: "rounded",
				Text:       "Rounded",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 3: Double border
	{
		style := lipgloss.NewStyle().Border(lipgloss.DoubleBorder())
		rendered := style.Render("Double")
		fs.AddTestWithCategory("border_double", "unit",
			capture.BorderInput{
				BorderType: "double",
				Text:       "Double",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 4: Thick border
	{
		style := lipgloss.NewStyle().Border(lipgloss.ThickBorder())
		rendered := style.Render("Thick")
		fs.AddTestWithCategory("border_thick", "unit",
			capture.BorderInput{
				BorderType: "thick",
				Text:       "Thick",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 5: Block border
	{
		style := lipgloss.NewStyle().Border(lipgloss.BlockBorder())
		rendered := style.Render("Block")
		fs.AddTestWithCategory("border_block", "unit",
			capture.BorderInput{
				BorderType: "block",
				Text:       "Block",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 6: Hidden border
	{
		style := lipgloss.NewStyle().Border(lipgloss.HiddenBorder())
		rendered := style.Render("Hidden")
		fs.AddTestWithCategory("border_hidden", "unit",
			capture.BorderInput{
				BorderType: "hidden",
				Text:       "Hidden",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 7: ASCII border
	{
		style := lipgloss.NewStyle().Border(lipgloss.ASCIIBorder())
		rendered := style.Render("ASCII")
		fs.AddTestWithCategory("border_ascii", "unit",
			capture.BorderInput{
				BorderType: "ascii",
				Text:       "ASCII",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 8: Border with color
	{
		fg := "#FF0000"
		style := lipgloss.NewStyle().
			Border(lipgloss.NormalBorder()).
			BorderForeground(lipgloss.Color(fg))
		rendered := style.Render("Colored")
		fs.AddTestWithCategory("border_colored", "unit",
			capture.BorderInput{
				BorderType: "normal",
				Text:       "Colored",
				Foreground: &fg,
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}

	// Test 9: Partial border (top and bottom only)
	{
		style := lipgloss.NewStyle().
			Border(lipgloss.NormalBorder()).
			BorderTop(true).
			BorderBottom(true).
			BorderLeft(false).
			BorderRight(false)
		rendered := style.Render("TopBot")
		fs.AddTestWithNotes("border_partial_top_bottom",
			capture.BorderInput{
				BorderType: "normal",
				Text:       "TopBot",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
			"Border on top and bottom edges only",
		)
	}

	// Test 10: Multi-line with border
	{
		style := lipgloss.NewStyle().Border(lipgloss.RoundedBorder())
		rendered := style.Render("Line1\nLine2")
		fs.AddTestWithCategory("border_multiline", "unit",
			capture.BorderInput{
				BorderType: "rounded",
				Text:       "Line1\nLine2",
			},
			capture.BorderOutput{
				Rendered: rendered,
			},
		)
	}
}

func capturePaddingMarginTests(fs *capture.FixtureSet) {
	// Test 1: Padding all sides
	{
		style := lipgloss.NewStyle().Padding(1)
		rendered := style.Render("Pad")
		fs.AddTestWithCategory("padding_all", "unit",
			capture.StyleInput{
				Text:    "Pad",
				Padding: []int{1, 1, 1, 1},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,  // 3 + 1 + 1
				Height:   3,  // 1 + 1 + 1
			},
		)
	}

	// Test 2: Padding horizontal and vertical
	{
		style := lipgloss.NewStyle().Padding(1, 2) // vertical=1, horizontal=2
		rendered := style.Render("Pad")
		fs.AddTestWithCategory("padding_vh", "unit",
			capture.StyleInput{
				Text:    "Pad",
				Padding: []int{1, 2, 1, 2},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    7,  // 3 + 2 + 2
				Height:   3,  // 1 + 1 + 1
			},
		)
	}

	// Test 3: Padding individual sides
	{
		style := lipgloss.NewStyle().
			PaddingTop(1).
			PaddingRight(2).
			PaddingBottom(3).
			PaddingLeft(4)
		rendered := style.Render("P")
		fs.AddTestWithCategory("padding_individual", "unit",
			capture.StyleInput{
				Text:    "P",
				Padding: []int{1, 2, 3, 4},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    7,  // 1 + 2 + 4
				Height:   5,  // 1 + 1 + 3
			},
		)
	}

	// Test 4: Margin all sides
	{
		style := lipgloss.NewStyle().Margin(1)
		rendered := style.Render("Mar")
		fs.AddTestWithCategory("margin_all", "unit",
			capture.StyleInput{
				Text:   "Mar",
				Margin: []int{1, 1, 1, 1},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,  // 3 + 1 + 1
				Height:   3,  // 1 + 1 + 1
			},
		)
	}

	// Test 5: Margin individual sides
	{
		style := lipgloss.NewStyle().
			MarginTop(1).
			MarginRight(2).
			MarginBottom(3).
			MarginLeft(4)
		rendered := style.Render("M")
		fs.AddTestWithCategory("margin_individual", "unit",
			capture.StyleInput{
				Text:   "M",
				Margin: []int{1, 2, 3, 4},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    7,  // 1 + 2 + 4
				Height:   5,  // 1 + 1 + 3
			},
		)
	}

	// Test 6: Combined padding and margin
	{
		style := lipgloss.NewStyle().Padding(1).Margin(1)
		rendered := style.Render("PM")
		fs.AddTestWithCategory("padding_margin_combined", "unit",
			capture.StyleInput{
				Text:    "PM",
				Padding: []int{1, 1, 1, 1},
				Margin:  []int{1, 1, 1, 1},
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    6,  // 2 + 1+1 + 1+1
				Height:   5,  // 1 + 1+1 + 1+1
			},
		)
	}
}

func captureDimensionTests(fs *capture.FixtureSet) {
	// Test 1: Fixed width
	{
		style := lipgloss.NewStyle().Width(10)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("dimension_width", "unit",
			capture.StyleInput{
				Text:  "Hi",
				Width: 10,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    10,
				Height:   1,
			},
		)
	}

	// Test 2: Fixed height
	{
		style := lipgloss.NewStyle().Height(3)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("dimension_height", "unit",
			capture.StyleInput{
				Text:   "Hi",
				Height: 3,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    2,
				Height:   3,
			},
		)
	}

	// Test 3: Fixed width and height
	{
		style := lipgloss.NewStyle().Width(10).Height(3)
		rendered := style.Render("Box")
		fs.AddTestWithCategory("dimension_both", "unit",
			capture.StyleInput{
				Text:   "Box",
				Width:  10,
				Height: 3,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    10,
				Height:   3,
			},
		)
	}

	// Test 4: Max width
	{
		style := lipgloss.NewStyle().MaxWidth(5)
		rendered := style.Render("Hello World")
		fs.AddTestWithCategory("dimension_maxwidth", "unit",
			map[string]interface{}{
				"text":      "Hello World",
				"max_width": 5,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    5,
				Height:   1,
			},
		)
	}

	// Test 5: Max height
	{
		style := lipgloss.NewStyle().MaxHeight(2)
		rendered := style.Render("L1\nL2\nL3\nL4")
		fs.AddTestWithCategory("dimension_maxheight", "unit",
			map[string]interface{}{
				"text":       "L1\nL2\nL3\nL4",
				"max_height": 2,
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    2,
				Height:   2,
			},
		)
	}
}

func captureAlignmentTests(fs *capture.FixtureSet) {
	// Test 1: Align left (default)
	{
		style := lipgloss.NewStyle().Width(10).Align(lipgloss.Left)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_left", "unit",
			map[string]interface{}{
				"text":             "Hi",
				"width":            10,
				"align_horizontal": "left",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    10,
				Height:   1,
			},
		)
	}

	// Test 2: Align center
	{
		style := lipgloss.NewStyle().Width(10).Align(lipgloss.Center)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_center", "unit",
			map[string]interface{}{
				"text":             "Hi",
				"width":            10,
				"align_horizontal": "center",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    10,
				Height:   1,
			},
		)
	}

	// Test 3: Align right
	{
		style := lipgloss.NewStyle().Width(10).Align(lipgloss.Right)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_right", "unit",
			map[string]interface{}{
				"text":             "Hi",
				"width":            10,
				"align_horizontal": "right",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    10,
				Height:   1,
			},
		)
	}

	// Test 4: Vertical align top
	{
		style := lipgloss.NewStyle().Height(3).AlignVertical(lipgloss.Top)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_vertical_top", "unit",
			map[string]interface{}{
				"text":           "Hi",
				"height":         3,
				"align_vertical": "top",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    2,
				Height:   3,
			},
		)
	}

	// Test 5: Vertical align center
	{
		style := lipgloss.NewStyle().Height(3).AlignVertical(lipgloss.Center)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_vertical_center", "unit",
			map[string]interface{}{
				"text":           "Hi",
				"height":         3,
				"align_vertical": "center",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    2,
				Height:   3,
			},
		)
	}

	// Test 6: Vertical align bottom
	{
		style := lipgloss.NewStyle().Height(3).AlignVertical(lipgloss.Bottom)
		rendered := style.Render("Hi")
		fs.AddTestWithCategory("align_vertical_bottom", "unit",
			map[string]interface{}{
				"text":           "Hi",
				"height":         3,
				"align_vertical": "bottom",
			},
			capture.StyleOutput{
				Rendered: rendered,
				Width:    2,
				Height:   3,
			},
		)
	}
}

func captureJoinTests(fs *capture.FixtureSet) {
	// Test 1: Join horizontal top
	{
		result := lipgloss.JoinHorizontal(lipgloss.Top, "A\nA", "B\nB\nB")
		fs.AddTestWithCategory("join_horizontal_top", "unit",
			map[string]interface{}{
				"blocks":   []string{"A\nA", "B\nB\nB"},
				"position": "top",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 2: Join horizontal center
	{
		result := lipgloss.JoinHorizontal(lipgloss.Center, "A\nA", "B\nB\nB")
		fs.AddTestWithCategory("join_horizontal_center", "unit",
			map[string]interface{}{
				"blocks":   []string{"A\nA", "B\nB\nB"},
				"position": "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 3: Join horizontal bottom
	{
		result := lipgloss.JoinHorizontal(lipgloss.Bottom, "A\nA", "B\nB\nB")
		fs.AddTestWithCategory("join_horizontal_bottom", "unit",
			map[string]interface{}{
				"blocks":   []string{"A\nA", "B\nB\nB"},
				"position": "bottom",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 4: Join vertical left
	{
		result := lipgloss.JoinVertical(lipgloss.Left, "Short", "LongerText")
		fs.AddTestWithCategory("join_vertical_left", "unit",
			map[string]interface{}{
				"blocks":   []string{"Short", "LongerText"},
				"position": "left",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 5: Join vertical center
	{
		result := lipgloss.JoinVertical(lipgloss.Center, "Short", "LongerText")
		fs.AddTestWithCategory("join_vertical_center", "unit",
			map[string]interface{}{
				"blocks":   []string{"Short", "LongerText"},
				"position": "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 6: Join vertical right
	{
		result := lipgloss.JoinVertical(lipgloss.Right, "Short", "LongerText")
		fs.AddTestWithCategory("join_vertical_right", "unit",
			map[string]interface{}{
				"blocks":   []string{"Short", "LongerText"},
				"position": "right",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 7: Join multiple blocks
	{
		result := lipgloss.JoinHorizontal(lipgloss.Top, "A", "B", "C", "D")
		fs.AddTestWithCategory("join_horizontal_multiple", "unit",
			map[string]interface{}{
				"blocks":   []string{"A", "B", "C", "D"},
				"position": "top",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 8: Join empty strings
	{
		result := lipgloss.JoinHorizontal(lipgloss.Top, "", "B")
		fs.AddTestWithCategory("join_horizontal_empty", "unit",
			map[string]interface{}{
				"blocks":   []string{"", "B"},
				"position": "top",
			},
			map[string]string{
				"result": result,
			},
		)
	}
}

func capturePlaceTests(fs *capture.FixtureSet) {
	// Test 1: Place horizontal left
	{
		result := lipgloss.PlaceHorizontal(10, lipgloss.Left, "Hi")
		fs.AddTestWithCategory("place_horizontal_left", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"width":    10,
				"position": "left",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 2: Place horizontal center
	{
		result := lipgloss.PlaceHorizontal(10, lipgloss.Center, "Hi")
		fs.AddTestWithCategory("place_horizontal_center", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"width":    10,
				"position": "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 3: Place horizontal right
	{
		result := lipgloss.PlaceHorizontal(10, lipgloss.Right, "Hi")
		fs.AddTestWithCategory("place_horizontal_right", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"width":    10,
				"position": "right",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 4: Place vertical top
	{
		result := lipgloss.PlaceVertical(3, lipgloss.Top, "Hi")
		fs.AddTestWithCategory("place_vertical_top", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"height":   3,
				"position": "top",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 5: Place vertical center
	{
		result := lipgloss.PlaceVertical(3, lipgloss.Center, "Hi")
		fs.AddTestWithCategory("place_vertical_center", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"height":   3,
				"position": "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 6: Place vertical bottom
	{
		result := lipgloss.PlaceVertical(3, lipgloss.Bottom, "Hi")
		fs.AddTestWithCategory("place_vertical_bottom", "unit",
			map[string]interface{}{
				"text":     "Hi",
				"height":   3,
				"position": "bottom",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 7: Place both dimensions
	{
		result := lipgloss.Place(10, 3, lipgloss.Center, lipgloss.Center, "Hi")
		fs.AddTestWithCategory("place_both_center", "unit",
			map[string]interface{}{
				"text":              "Hi",
				"width":             10,
				"height":            3,
				"horizontal_pos":    "center",
				"vertical_pos":      "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}

	// Test 8: Place multi-line
	{
		result := lipgloss.Place(10, 5, lipgloss.Center, lipgloss.Center, "A\nB")
		fs.AddTestWithCategory("place_multiline", "unit",
			map[string]interface{}{
				"text":           "A\nB",
				"width":          10,
				"height":         5,
				"horizontal_pos": "center",
				"vertical_pos":   "center",
			},
			map[string]string{
				"result": result,
			},
		)
	}
}
