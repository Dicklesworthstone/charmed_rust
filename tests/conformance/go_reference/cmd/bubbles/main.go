// Bubbles capture program - captures component behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("bubbles", "0.20.0")

	// Capture viewport behaviors
	captureViewportTests(fixtures)

	// Capture textinput behaviors
	captureTextInputTests(fixtures)

	// Capture progress behaviors
	captureProgressTests(fixtures)

	// Capture spinner behaviors
	captureSpinnerTests(fixtures)

	// Capture paginator behaviors
	capturePaginatorTests(fixtures)

	// Capture help behaviors
	captureHelpTests(fixtures)

	// Capture cursor behaviors
	captureCursorTests(fixtures)

	// Capture key bindings
	captureKeyBindingTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureViewportTests(fs *capture.FixtureSet) {
	// Test 1: Basic viewport creation
	{
		vp := viewport.New(80, 24)
		fs.AddTestWithCategory("viewport_new", "unit",
			map[string]interface{}{
				"width":  80,
				"height": 24,
			},
			map[string]interface{}{
				"width":        vp.Width,
				"height":       vp.Height,
				"y_offset":     vp.YOffset,
				"y_position":   vp.YPosition,
				"at_top":       vp.AtTop(),
				"at_bottom":    vp.AtBottom(),
				"scroll_percent": vp.ScrollPercent(),
			},
		)
	}

	// Test 2: Viewport with content
	{
		vp := viewport.New(80, 5)
		content := "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10"
		vp.SetContent(content)
		fs.AddTestWithCategory("viewport_with_content", "unit",
			map[string]interface{}{
				"width":   80,
				"height":  5,
				"content": content,
			},
			map[string]interface{}{
				"total_lines":   10,
				"visible_lines": 5,
				"at_top":        vp.AtTop(),
				"at_bottom":     vp.AtBottom(),
				"scroll_percent": vp.ScrollPercent(),
				"view":          vp.View(),
			},
		)
	}

	// Test 3: Viewport scrolling
	{
		vp := viewport.New(80, 3)
		content := "Line 1\nLine 2\nLine 3\nLine 4\nLine 5"
		vp.SetContent(content)
		vp.LineDown(1)
		fs.AddTestWithCategory("viewport_scroll_down", "unit",
			map[string]interface{}{
				"width":      80,
				"height":     3,
				"content":    content,
				"scroll_by":  1,
			},
			map[string]interface{}{
				"y_offset":       vp.YOffset,
				"at_top":         vp.AtTop(),
				"at_bottom":      vp.AtBottom(),
				"scroll_percent": vp.ScrollPercent(),
				"view":           vp.View(),
			},
		)
	}

	// Test 4: Viewport scroll to bottom
	{
		vp := viewport.New(80, 3)
		content := "Line 1\nLine 2\nLine 3\nLine 4\nLine 5"
		vp.SetContent(content)
		vp.GotoBottom()
		fs.AddTestWithCategory("viewport_goto_bottom", "unit",
			map[string]interface{}{
				"width":   80,
				"height":  3,
				"content": content,
			},
			map[string]interface{}{
				"y_offset":       vp.YOffset,
				"at_top":         vp.AtTop(),
				"at_bottom":      vp.AtBottom(),
				"scroll_percent": vp.ScrollPercent(),
				"view":           vp.View(),
			},
		)
	}

	// Test 5: Viewport scroll to top
	{
		vp := viewport.New(80, 3)
		content := "Line 1\nLine 2\nLine 3\nLine 4\nLine 5"
		vp.SetContent(content)
		vp.GotoBottom()
		vp.GotoTop()
		fs.AddTestWithCategory("viewport_goto_top", "unit",
			map[string]interface{}{
				"width":   80,
				"height":  3,
				"content": content,
			},
			map[string]interface{}{
				"y_offset":       vp.YOffset,
				"at_top":         vp.AtTop(),
				"at_bottom":      vp.AtBottom(),
				"scroll_percent": vp.ScrollPercent(),
				"view":           vp.View(),
			},
		)
	}

	// Test 6: Half page down
	{
		vp := viewport.New(80, 4)
		content := "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8"
		vp.SetContent(content)
		vp.HalfViewDown()
		fs.AddTestWithCategory("viewport_half_page_down", "unit",
			map[string]interface{}{
				"width":   80,
				"height":  4,
				"content": content,
			},
			map[string]interface{}{
				"y_offset":       vp.YOffset,
				"scroll_percent": vp.ScrollPercent(),
			},
		)
	}

	// Test 7: ViewDown and ViewUp
	{
		vp := viewport.New(80, 3)
		content := "L1\nL2\nL3\nL4\nL5\nL6\nL7\nL8\nL9"
		vp.SetContent(content)
		vp.ViewDown()
		downOffset := vp.YOffset
		vp.ViewUp()
		upOffset := vp.YOffset
		fs.AddTestWithCategory("viewport_page_navigation", "unit",
			map[string]interface{}{
				"width":   80,
				"height":  3,
				"content": content,
			},
			map[string]interface{}{
				"after_view_down": downOffset,
				"after_view_up":   upOffset,
			},
		)
	}
}

func captureTextInputTests(fs *capture.FixtureSet) {
	// Test 1: Basic text input
	{
		ti := textinput.New()
		ti.Placeholder = "Enter text..."
		fs.AddTestWithCategory("textinput_new", "unit",
			map[string]interface{}{
				"placeholder": "Enter text...",
			},
			map[string]interface{}{
				"value":       ti.Value(),
				"placeholder": ti.Placeholder,
				"cursor_pos":  ti.Position(),
				"focused":     ti.Focused(),
			},
		)
	}

	// Test 2: Text input with value
	{
		ti := textinput.New()
		ti.SetValue("Hello World")
		fs.AddTestWithCategory("textinput_with_value", "unit",
			map[string]interface{}{
				"value": "Hello World",
			},
			map[string]interface{}{
				"value":      ti.Value(),
				"cursor_pos": ti.Position(),
				"length":     len(ti.Value()),
			},
		)
	}

	// Test 3: Text input with character limit
	{
		ti := textinput.New()
		ti.CharLimit = 10
		ti.SetValue("Hello World Extra")
		fs.AddTestWithCategory("textinput_char_limit", "unit",
			map[string]interface{}{
				"char_limit": 10,
				"input":      "Hello World Extra",
			},
			map[string]interface{}{
				"value":      ti.Value(),
				"length":     len(ti.Value()),
				"char_limit": ti.CharLimit,
			},
		)
	}

	// Test 4: Text input with width
	{
		ti := textinput.New()
		ti.Width = 20
		ti.SetValue("Test")
		fs.AddTestWithCategory("textinput_width", "unit",
			map[string]interface{}{
				"width": 20,
				"value": "Test",
			},
			map[string]interface{}{
				"width": ti.Width,
				"value": ti.Value(),
			},
		)
	}

	// Test 5: Text input cursor movement
	{
		ti := textinput.New()
		ti.SetValue("Hello World")
		ti.SetCursor(5)
		fs.AddTestWithCategory("textinput_cursor_set", "unit",
			map[string]interface{}{
				"value":      "Hello World",
				"cursor_pos": 5,
			},
			map[string]interface{}{
				"value":      ti.Value(),
				"cursor_pos": ti.Position(),
			},
		)
	}

	// Test 6: Text input cursor at start
	{
		ti := textinput.New()
		ti.SetValue("Hello")
		ti.CursorStart()
		fs.AddTestWithCategory("textinput_cursor_start", "unit",
			map[string]interface{}{
				"value": "Hello",
			},
			map[string]interface{}{
				"cursor_pos": ti.Position(),
			},
		)
	}

	// Test 7: Text input cursor at end
	{
		ti := textinput.New()
		ti.SetValue("Hello")
		ti.CursorEnd()
		fs.AddTestWithCategory("textinput_cursor_end", "unit",
			map[string]interface{}{
				"value": "Hello",
			},
			map[string]interface{}{
				"cursor_pos": ti.Position(),
			},
		)
	}

	// Test 8: Password echo mode
	{
		ti := textinput.New()
		ti.EchoMode = textinput.EchoPassword
		ti.SetValue("secret")
		fs.AddTestWithCategory("textinput_password", "unit",
			map[string]interface{}{
				"value":     "secret",
				"echo_mode": "password",
			},
			map[string]interface{}{
				"value":         ti.Value(),
				"echo_mode":     int(ti.EchoMode),
				"echo_char":     string(ti.EchoCharacter),
			},
		)
	}

	// Test 9: Echo none mode
	{
		ti := textinput.New()
		ti.EchoMode = textinput.EchoNone
		ti.SetValue("hidden")
		fs.AddTestWithCategory("textinput_echo_none", "unit",
			map[string]interface{}{
				"value":     "hidden",
				"echo_mode": "none",
			},
			map[string]interface{}{
				"value":     ti.Value(),
				"echo_mode": int(ti.EchoMode),
			},
		)
	}

	// Test 10: Focus and blur
	{
		ti := textinput.New()
		ti.Focus()
		focusedState := ti.Focused()
		ti.Blur()
		blurredState := ti.Focused()
		fs.AddTestWithCategory("textinput_focus_blur", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"after_focus": focusedState,
				"after_blur":  blurredState,
			},
		)
	}
}

func captureProgressTests(fs *capture.FixtureSet) {
	// Test 1: Basic progress bar
	{
		p := progress.New(progress.WithDefaultGradient())
		view := p.ViewAs(0.5)
		fs.AddTestWithCategory("progress_basic", "unit",
			map[string]interface{}{
				"percent": 0.5,
			},
			map[string]interface{}{
				"view_length":       len(view),
				"percent":           0.5,
				"is_animated":       p.IsAnimating(),
			},
		)
	}

	// Test 2: Progress at 0%
	{
		p := progress.New()
		view := p.ViewAs(0.0)
		fs.AddTestWithCategory("progress_zero", "unit",
			map[string]interface{}{
				"percent": 0.0,
			},
			map[string]interface{}{
				"view_length": len(view),
			},
		)
	}

	// Test 3: Progress at 100%
	{
		p := progress.New()
		view := p.ViewAs(1.0)
		fs.AddTestWithCategory("progress_full", "unit",
			map[string]interface{}{
				"percent": 1.0,
			},
			map[string]interface{}{
				"view_length": len(view),
			},
		)
	}

	// Test 4: Progress with custom width
	{
		p := progress.New(progress.WithWidth(50))
		view := p.ViewAs(0.75)
		fs.AddTestWithCategory("progress_custom_width", "unit",
			map[string]interface{}{
				"width":   50,
				"percent": 0.75,
			},
			map[string]interface{}{
				"view_length": len(view),
			},
		)
	}

	// Test 5: Progress without percentage
	{
		p := progress.New(progress.WithoutPercentage())
		view := p.ViewAs(0.5)
		fs.AddTestWithCategory("progress_no_percent", "unit",
			map[string]interface{}{
				"show_percentage": false,
				"percent":         0.5,
			},
			map[string]interface{}{
				"view":        view,
				"view_length": len(view),
			},
		)
	}

	// Test 6: Progress with solid fill
	{
		p := progress.New(progress.WithSolidFill("blue"))
		view := p.ViewAs(0.6)
		fs.AddTestWithCategory("progress_solid_fill", "unit",
			map[string]interface{}{
				"fill_color": "blue",
				"percent":    0.6,
			},
			map[string]interface{}{
				"view_length": len(view),
			},
		)
	}
}

func captureSpinnerTests(fs *capture.FixtureSet) {
	// Test spinner types
	spinnerTypes := []struct {
		name    string
		spinner spinner.Spinner
	}{
		{"Line", spinner.Line},
		{"Dot", spinner.Dot},
		{"MiniDot", spinner.MiniDot},
		{"Jump", spinner.Jump},
		{"Pulse", spinner.Pulse},
		{"Points", spinner.Points},
		{"Globe", spinner.Globe},
		{"Moon", spinner.Moon},
		{"Monkey", spinner.Monkey},
		{"Meter", spinner.Meter},
		{"Hamburger", spinner.Hamburger},
	}

	for _, st := range spinnerTypes {
		fs.AddTestWithCategory(fmt.Sprintf("spinner_%s", strings.ToLower(st.name)), "unit",
			map[string]interface{}{
				"spinner_type": st.name,
			},
			map[string]interface{}{
				"frames":      st.spinner.Frames,
				"frame_count": len(st.spinner.Frames),
				"fps":         st.spinner.FPS.Milliseconds(),
			},
		)
	}

	// Test spinner model
	{
		s := spinner.New()
		s.Spinner = spinner.Dot
		view := s.View()
		fs.AddTestWithCategory("spinner_model_view", "unit",
			map[string]interface{}{
				"spinner_type": "Dot",
			},
			map[string]interface{}{
				"view":       view,
				"view_bytes": len(view),
			},
		)
	}
}

func capturePaginatorTests(fs *capture.FixtureSet) {
	// Test 1: Basic paginator (dot style)
	{
		p := paginator.New()
		p.Type = paginator.Dots
		p.SetTotalPages(5)
		fs.AddTestWithCategory("paginator_dots", "unit",
			map[string]interface{}{
				"type":        "dots",
				"total_pages": 5,
			},
			map[string]interface{}{
				"page":        p.Page,
				"total_pages": p.TotalPages,
				"on_first":    p.OnFirstPage(),
				"on_last":     p.OnLastPage(),
				"view":        p.View(),
			},
		)
	}

	// Test 2: Arabic numerals paginator
	{
		p := paginator.New()
		p.Type = paginator.Arabic
		p.SetTotalPages(10)
		fs.AddTestWithCategory("paginator_arabic", "unit",
			map[string]interface{}{
				"type":        "arabic",
				"total_pages": 10,
			},
			map[string]interface{}{
				"page":        p.Page,
				"total_pages": p.TotalPages,
				"view":        p.View(),
			},
		)
	}

	// Test 3: Paginator navigation
	{
		p := paginator.New()
		p.SetTotalPages(5)
		p.Page = 0
		p.NextPage()
		afterNext := p.Page
		p.PrevPage()
		afterPrev := p.Page
		fs.AddTestWithCategory("paginator_navigation", "unit",
			map[string]interface{}{
				"total_pages":  5,
				"start_page":   0,
			},
			map[string]interface{}{
				"after_next": afterNext,
				"after_prev": afterPrev,
			},
		)
	}

	// Test 4: Paginator at boundaries
	{
		p := paginator.New()
		p.SetTotalPages(3)
		p.Page = 0
		p.PrevPage() // Should not go below 0
		atStart := p.Page
		p.Page = 2
		p.NextPage() // Should not go above total
		atEnd := p.Page
		fs.AddTestWithCategory("paginator_boundaries", "unit",
			map[string]interface{}{
				"total_pages": 3,
			},
			map[string]interface{}{
				"at_start_after_prev": atStart,
				"at_end_after_next":   atEnd,
				"on_first":            p.OnFirstPage(),
				"on_last":             p.OnLastPage(),
			},
		)
	}

	// Test 5: Items per page
	{
		p := paginator.New()
		p.SetTotalPages(3)
		p.PerPage = 10
		items := 25
		p.SetTotalPages(items / p.PerPage)
		if items%p.PerPage > 0 {
			p.SetTotalPages((items / p.PerPage) + 1)
		}
		fs.AddTestWithCategory("paginator_items_per_page", "unit",
			map[string]interface{}{
				"total_items": items,
				"per_page":    10,
			},
			map[string]interface{}{
				"total_pages": p.TotalPages,
				"per_page":    p.PerPage,
			},
		)
	}
}

func captureHelpTests(fs *capture.FixtureSet) {
	// Test 1: Basic help model
	{
		h := help.New()
		keys := testKeyMap{}
		shortView := h.ShortHelpView(keys.ShortHelp())
		fullView := h.FullHelpView(keys.FullHelp())
		fs.AddTestWithCategory("help_basic", "unit",
			map[string]interface{}{
				"keys": []string{"up", "down", "enter", "quit"},
			},
			map[string]interface{}{
				"short_view":        shortView,
				"full_view":         fullView,
				"short_view_length": len(shortView),
				"full_view_length":  len(fullView),
			},
		)
	}

	// Test 2: Help with custom width
	{
		h := help.New()
		h.Width = 40
		keys := testKeyMap{}
		shortView := h.ShortHelpView(keys.ShortHelp())
		fs.AddTestWithCategory("help_custom_width", "unit",
			map[string]interface{}{
				"width": 40,
			},
			map[string]interface{}{
				"short_view": shortView,
				"width":      h.Width,
			},
		)
	}

	// Test 3: Empty help
	{
		h := help.New()
		emptyKeys := emptyKeyMap{}
		shortView := h.ShortHelpView(emptyKeys.ShortHelp())
		fs.AddTestWithCategory("help_empty", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"short_view":        shortView,
				"short_view_length": len(shortView),
			},
		)
	}
}

func captureCursorTests(fs *capture.FixtureSet) {
	// Test cursor modes
	modes := []struct {
		name string
		mode cursor.Mode
	}{
		{"CursorBlink", cursor.CursorBlink},
		{"CursorStatic", cursor.CursorStatic},
		{"CursorHide", cursor.CursorHide},
	}

	for _, m := range modes {
		fs.AddTestWithCategory(fmt.Sprintf("cursor_mode_%s", strings.ToLower(m.name)), "unit",
			map[string]interface{}{
				"mode": m.name,
			},
			map[string]interface{}{
				"mode_value": int(m.mode),
				"mode_string": m.mode.String(),
			},
		)
	}

	// Test cursor model
	{
		c := cursor.New()
		c.SetMode(cursor.CursorBlink)
		fs.AddTestWithCategory("cursor_model", "unit",
			map[string]interface{}{
				"mode": "CursorBlink",
			},
			map[string]interface{}{
				"mode": int(c.Mode()),
			},
		)
	}
}

func captureKeyBindingTests(fs *capture.FixtureSet) {
	// Test 1: Simple key binding
	{
		kb := key.NewBinding(
			key.WithKeys("q"),
			key.WithHelp("q", "quit"),
		)
		fs.AddTestWithCategory("keybinding_simple", "unit",
			map[string]interface{}{
				"keys": []string{"q"},
				"help": "quit",
			},
			map[string]interface{}{
				"keys":    kb.Keys(),
				"help":    kb.Help().Key,
				"enabled": kb.Enabled(),
			},
		)
	}

	// Test 2: Multi-key binding
	{
		kb := key.NewBinding(
			key.WithKeys("up", "k"),
			key.WithHelp("up/k", "move up"),
		)
		fs.AddTestWithCategory("keybinding_multi", "unit",
			map[string]interface{}{
				"keys": []string{"up", "k"},
				"help": "move up",
			},
			map[string]interface{}{
				"keys":    kb.Keys(),
				"help":    kb.Help().Key,
				"enabled": kb.Enabled(),
			},
		)
	}

	// Test 3: Disabled key binding
	{
		kb := key.NewBinding(
			key.WithKeys("x"),
			key.WithHelp("x", "disabled"),
			key.WithDisabled(),
		)
		fs.AddTestWithCategory("keybinding_disabled", "unit",
			map[string]interface{}{
				"keys":     []string{"x"},
				"disabled": true,
			},
			map[string]interface{}{
				"keys":    kb.Keys(),
				"enabled": kb.Enabled(),
			},
		)
	}

	// Test 4: Key binding enable/disable
	{
		kb := key.NewBinding(
			key.WithKeys("y"),
		)
		before := kb.Enabled()
		kb.SetEnabled(false)
		afterDisable := kb.Enabled()
		kb.SetEnabled(true)
		afterEnable := kb.Enabled()
		fs.AddTestWithCategory("keybinding_toggle", "unit",
			map[string]interface{}{
				"keys": []string{"y"},
			},
			map[string]interface{}{
				"initial_enabled":       before,
				"after_disable":         afterDisable,
				"after_enable":          afterEnable,
			},
		)
	}
}

// Test key map for help tests
type testKeyMap struct{}

func (k testKeyMap) ShortHelp() []key.Binding {
	return []key.Binding{
		key.NewBinding(key.WithKeys("up", "k"), key.WithHelp("up/k", "up")),
		key.NewBinding(key.WithKeys("down", "j"), key.WithHelp("down/j", "down")),
	}
}

func (k testKeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{
			key.NewBinding(key.WithKeys("up", "k"), key.WithHelp("up/k", "up")),
			key.NewBinding(key.WithKeys("down", "j"), key.WithHelp("down/j", "down")),
		},
		{
			key.NewBinding(key.WithKeys("enter"), key.WithHelp("enter", "select")),
			key.NewBinding(key.WithKeys("q"), key.WithHelp("q", "quit")),
		},
	}
}

// Empty key map for testing
type emptyKeyMap struct{}

func (k emptyKeyMap) ShortHelp() []key.Binding {
	return []key.Binding{}
}

func (k emptyKeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{}
}
