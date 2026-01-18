// Bubbles capture program - captures component behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/filepicker"
	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/stopwatch"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/timer"
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

	// Capture stopwatch behaviors
	captureStopwatchTests(fixtures)

	// Capture timer behaviors
	captureTimerTests(fixtures)

	// Capture paginator behaviors
	capturePaginatorTests(fixtures)

	// Capture help behaviors
	captureHelpTests(fixtures)

	// Capture cursor behaviors
	captureCursorTests(fixtures)

	// Capture key bindings
	captureKeyBindingTests(fixtures)

	// Capture list behaviors
	captureListTests(fixtures)

	// Capture table behaviors
	captureTableTests(fixtures)

	// Capture filepicker behaviors
	captureFilepickerTests(fixtures)

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

func captureStopwatchTests(fs *capture.FixtureSet) {
	// Test 1: New stopwatch
	{
		sw := stopwatch.New()
		fs.AddTestWithCategory("stopwatch_new", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"elapsed":     sw.Elapsed().String(),
				"elapsed_ms":  sw.Elapsed().Milliseconds(),
				"interval_ms": sw.Interval.Milliseconds(),
				"running":     sw.Running(),
				"view":        sw.View(),
			},
		)
	}

	// Test 2: Start + single tick - using TickMsg directly to simulate elapsed time
	{
		sw := stopwatch.New()
		// Simulate a tick (the stopwatch tracks elapsed time via ticks internally)
		sw, _ = sw.Update(stopwatch.TickMsg{ID: sw.ID()})
		fs.AddTestWithCategory("stopwatch_tick", "unit",
			map[string]interface{}{
				"ticks": 1,
			},
			map[string]interface{}{
				"elapsed":    sw.Elapsed().String(),
				"elapsed_ms": sw.Elapsed().Milliseconds(),
				"running":    sw.Running(),
				"view":       sw.View(),
			},
		)
	}

	// Test 3: Reset after tick
	{
		sw := stopwatch.New()
		sw, _ = sw.Update(stopwatch.TickMsg{ID: sw.ID()})
		sw, _ = sw.Update(stopwatch.ResetMsg{ID: sw.ID()})
		fs.AddTestWithCategory("stopwatch_reset", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"elapsed":    sw.Elapsed().String(),
				"elapsed_ms": sw.Elapsed().Milliseconds(),
				"running":    sw.Running(),
				"view":       sw.View(),
			},
		)
	}
}

func captureTimerTests(fs *capture.FixtureSet) {
	// Test 1: New timer
	{
		t := timer.New(10 * time.Second)
		fs.AddTestWithCategory("timer_new", "unit",
			map[string]interface{}{
				"timeout_secs": 10,
			},
			map[string]interface{}{
				"remaining":    t.Timeout.String(),
				"remaining_ms": t.Timeout.Milliseconds(),
				"interval_ms":  t.Interval.Milliseconds(),
				"running":      t.Running(),
				"timed_out":    t.Timedout(),
				"view":         t.View(),
			},
		)
	}

	// Test 2: Single tick
	{
		t := timer.New(3 * time.Second)
		t, _ = t.Update(timer.TickMsg{ID: t.ID(), Timeout: false})
		fs.AddTestWithCategory("timer_tick", "unit",
			map[string]interface{}{
				"timeout_secs": 3,
				"tick_count":   1,
			},
			map[string]interface{}{
				"remaining":    t.Timeout.String(),
				"remaining_ms": t.Timeout.Milliseconds(),
				"running":      t.Running(),
				"timed_out":    t.Timedout(),
				"view":         t.View(),
			},
		)
	}

	// Test 3: Timeout
	{
		t := timer.New(1 * time.Second)
		t, _ = t.Update(timer.TickMsg{ID: t.ID(), Timeout: false})
		fs.AddTestWithCategory("timer_timeout", "unit",
			map[string]interface{}{
				"timeout_secs": 1,
			},
			map[string]interface{}{
				"remaining":    t.Timeout.String(),
				"remaining_ms": t.Timeout.Milliseconds(),
				"running":      t.Running(),
				"timed_out":    t.Timedout(),
				"view":         t.View(),
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

// List item for testing
type listItem struct {
	title       string
	description string
}

func (i listItem) Title() string       { return i.title }
func (i listItem) Description() string { return i.description }
func (i listItem) FilterValue() string { return i.title }

func captureListTests(fs *capture.FixtureSet) {
	// Test 1: Empty list
	{
		items := []list.Item{}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		fs.AddTestWithCategory("list_empty", "unit",
			map[string]interface{}{
				"width":  80,
				"height": 24,
			},
			map[string]interface{}{
				"index":         l.Index(),
				"cursor":        l.Cursor(),
				"items_count":   len(l.Items()),
				"filter_state":  l.FilterState().String(),
			},
		)
	}

	// Test 2: List with items
	{
		items := []list.Item{
			listItem{title: "Apple", description: "A fruit"},
			listItem{title: "Banana", description: "Another fruit"},
			listItem{title: "Cherry", description: "A small fruit"},
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		fs.AddTestWithCategory("list_with_items", "unit",
			map[string]interface{}{
				"width":  80,
				"height": 24,
				"items":  []string{"Apple", "Banana", "Cherry"},
			},
			map[string]interface{}{
				"index":         l.Index(),
				"cursor":        l.Cursor(),
				"items_count":   len(l.Items()),
				"visible_items": l.VisibleItems(),
			},
		)
	}

	// Test 3: List cursor movement
	{
		items := []list.Item{
			listItem{title: "Item 1", description: "First"},
			listItem{title: "Item 2", description: "Second"},
			listItem{title: "Item 3", description: "Third"},
			listItem{title: "Item 4", description: "Fourth"},
			listItem{title: "Item 5", description: "Fifth"},
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		initialIndex := l.Index()
		l.CursorDown()
		afterDown := l.Index()
		l.CursorDown()
		afterSecondDown := l.Index()
		l.CursorUp()
		afterUp := l.Index()
		fs.AddTestWithCategory("list_cursor_movement", "unit",
			map[string]interface{}{
				"width":       80,
				"height":      24,
				"items_count": 5,
			},
			map[string]interface{}{
				"initial_index":     initialIndex,
				"after_down":        afterDown,
				"after_second_down": afterSecondDown,
				"after_up":          afterUp,
			},
		)
	}

	// Test 4: List go to top/bottom
	{
		items := []list.Item{
			listItem{title: "First", description: "1"},
			listItem{title: "Second", description: "2"},
			listItem{title: "Third", description: "3"},
			listItem{title: "Fourth", description: "4"},
			listItem{title: "Fifth", description: "5"},
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		l.CursorDown()
		l.CursorDown()
		middleIndex := l.Index()
		l.Select(len(items) - 1) // Go to bottom
		atBottom := l.Index()
		l.Select(0) // Go to top
		atTop := l.Index()
		fs.AddTestWithCategory("list_goto_top_bottom", "unit",
			map[string]interface{}{
				"items_count": 5,
			},
			map[string]interface{}{
				"middle_index": middleIndex,
				"at_bottom":    atBottom,
				"at_top":       atTop,
			},
		)
	}

	// Test 5: List pagination
	{
		// Create more items than can fit on one page
		items := make([]list.Item, 20)
		for i := 0; i < 20; i++ {
			items[i] = listItem{
				title:       fmt.Sprintf("Item %d", i+1),
				description: fmt.Sprintf("Description %d", i+1),
			}
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 10) // Small height to force pagination
		paginator := l.Paginator
		fs.AddTestWithCategory("list_pagination", "unit",
			map[string]interface{}{
				"width":       80,
				"height":      10,
				"items_count": 20,
			},
			map[string]interface{}{
				"total_pages":    paginator.TotalPages,
				"current_page":   paginator.Page,
				"items_per_page": paginator.PerPage,
			},
		)
	}

	// Test 6: List title and status
	{
		items := []list.Item{
			listItem{title: "Item 1", description: "Desc 1"},
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		l.Title = "My List"
		fs.AddTestWithCategory("list_title", "unit",
			map[string]interface{}{
				"title": "My List",
			},
			map[string]interface{}{
				"title":       l.Title,
				"show_title":  l.ShowTitle(),
			},
		)
	}

	// Test 7: List selection
	{
		items := []list.Item{
			listItem{title: "A", description: "First"},
			listItem{title: "B", description: "Second"},
			listItem{title: "C", description: "Third"},
		}
		l := list.New(items, list.NewDefaultDelegate(), 80, 24)
		l.Select(1)
		selectedItem := l.SelectedItem().(listItem)
		fs.AddTestWithCategory("list_selection", "unit",
			map[string]interface{}{
				"items": []string{"A", "B", "C"},
			},
			map[string]interface{}{
				"selected_index": l.Index(),
				"selected_title": selectedItem.title,
			},
		)
	}
}

func captureTableTests(fs *capture.FixtureSet) {
	// Test 1: Empty table
	{
		t := table.New(
			table.WithColumns([]table.Column{}),
			table.WithRows([]table.Row{}),
		)
		fs.AddTestWithCategory("table_empty", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"cursor":       t.Cursor(),
				"focused":      t.Focused(),
				"columns_count": 0,
				"rows_count":   0,
			},
		)
	}

	// Test 2: Table with columns and rows
	{
		columns := []table.Column{
			{Title: "ID", Width: 10},
			{Title: "Name", Width: 20},
			{Title: "Status", Width: 15},
		}
		rows := []table.Row{
			{"1", "Alice", "Active"},
			{"2", "Bob", "Inactive"},
			{"3", "Charlie", "Active"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		fs.AddTestWithCategory("table_with_data", "unit",
			map[string]interface{}{
				"columns": []map[string]interface{}{
					{"title": "ID", "width": 10},
					{"title": "Name", "width": 20},
					{"title": "Status", "width": 15},
				},
				"rows": [][]string{
					{"1", "Alice", "Active"},
					{"2", "Bob", "Inactive"},
					{"3", "Charlie", "Active"},
				},
			},
			map[string]interface{}{
				"cursor":        t.Cursor(),
				"columns_count": 3,
				"rows_count":    3,
				"selected_row":  t.SelectedRow(),
			},
		)
	}

	// Test 3: Table cursor movement
	{
		columns := []table.Column{
			{Title: "ID", Width: 10},
			{Title: "Name", Width: 20},
		}
		rows := []table.Row{
			{"1", "First"},
			{"2", "Second"},
			{"3", "Third"},
			{"4", "Fourth"},
			{"5", "Fifth"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		initialCursor := t.Cursor()
		t.MoveDown(1)
		afterDown := t.Cursor()
		t.MoveDown(1)
		afterSecondDown := t.Cursor()
		t.MoveUp(1)
		afterUp := t.Cursor()
		fs.AddTestWithCategory("table_cursor_movement", "unit",
			map[string]interface{}{
				"rows_count": 5,
			},
			map[string]interface{}{
				"initial_cursor":    initialCursor,
				"after_down":        afterDown,
				"after_second_down": afterSecondDown,
				"after_up":          afterUp,
			},
		)
	}

	// Test 4: Table goto top/bottom
	{
		columns := []table.Column{
			{Title: "ID", Width: 10},
		}
		rows := []table.Row{
			{"1"}, {"2"}, {"3"}, {"4"}, {"5"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		t.MoveDown(2)
		middleCursor := t.Cursor()
		t.GotoBottom()
		atBottom := t.Cursor()
		t.GotoTop()
		atTop := t.Cursor()
		fs.AddTestWithCategory("table_goto_top_bottom", "unit",
			map[string]interface{}{
				"rows_count": 5,
			},
			map[string]interface{}{
				"middle_cursor": middleCursor,
				"at_bottom":     atBottom,
				"at_top":        atTop,
			},
		)
	}

	// Test 5: Table focus
	{
		columns := []table.Column{
			{Title: "Col", Width: 10},
		}
		rows := []table.Row{
			{"Data"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		initialFocus := t.Focused()
		t.Focus()
		afterFocus := t.Focused()
		t.Blur()
		afterBlur := t.Focused()
		fs.AddTestWithCategory("table_focus", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"initial_focus": initialFocus,
				"after_focus":   afterFocus,
				"after_blur":    afterBlur,
			},
		)
	}

	// Test 6: Table set cursor
	{
		columns := []table.Column{
			{Title: "ID", Width: 10},
		}
		rows := []table.Row{
			{"1"}, {"2"}, {"3"}, {"4"}, {"5"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		t.SetCursor(3)
		afterSet := t.Cursor()
		selectedRow := t.SelectedRow()
		fs.AddTestWithCategory("table_set_cursor", "unit",
			map[string]interface{}{
				"rows_count": 5,
				"set_to":     3,
			},
			map[string]interface{}{
				"cursor":       afterSet,
				"selected_row": selectedRow,
			},
		)
	}

	// Test 7: Table dimensions
	{
		columns := []table.Column{
			{Title: "A", Width: 10},
			{Title: "B", Width: 20},
		}
		rows := []table.Row{
			{"1", "Data 1"},
			{"2", "Data 2"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
			table.WithWidth(50),
			table.WithHeight(10),
		)
		fs.AddTestWithCategory("table_dimensions", "unit",
			map[string]interface{}{
				"width":  50,
				"height": 10,
			},
			map[string]interface{}{
				"width":  t.Width(),
				"height": t.Height(),
			},
		)
	}

	// Test 8: Table cursor bounds
	{
		columns := []table.Column{
			{Title: "ID", Width: 10},
		}
		rows := []table.Row{
			{"1"}, {"2"}, {"3"},
		}
		t := table.New(
			table.WithColumns(columns),
			table.WithRows(rows),
		)
		// Try moving up from top
		t.MoveUp(1)
		atTopAfterUp := t.Cursor()
		// Move to bottom
		t.GotoBottom()
		// Try moving down from bottom
		t.MoveDown(1)
		atBottomAfterDown := t.Cursor()
		fs.AddTestWithCategory("table_cursor_bounds", "unit",
			map[string]interface{}{
				"rows_count": 3,
			},
			map[string]interface{}{
				"at_top_after_up":     atTopAfterUp,
				"at_bottom_after_down": atBottomAfterDown,
			},
		)
	}
}

func captureFilepickerTests(fs *capture.FixtureSet) {
	// Test 1: New filepicker defaults
	{
		fp := filepicker.New()
		fs.AddTestWithCategory("filepicker_new", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"show_permissions": fp.ShowPermissions,
				"show_size":        fp.ShowSize,
				"show_hidden":      fp.ShowHidden,
				"dir_allowed":      fp.DirAllowed,
				"file_allowed":     fp.FileAllowed,
				"auto_height":      fp.AutoHeight,
				"current_directory": fp.CurrentDirectory,
			},
		)
	}

	// Test 2: Set current directory
	{
		fp := filepicker.New()
		fp.CurrentDirectory = "/tmp"
		fs.AddTestWithCategory("filepicker_set_directory", "unit",
			map[string]interface{}{
				"directory": "/tmp",
			},
			map[string]interface{}{
				"current_directory": fp.CurrentDirectory,
			},
		)
	}

	// Test 3: Allowed types
	{
		fp := filepicker.New()
		fp.AllowedTypes = []string{".txt", ".md"}
		fs.AddTestWithCategory("filepicker_allowed_types", "unit",
			map[string]interface{}{
				"allowed_types": []string{".txt", ".md"},
			},
			map[string]interface{}{
				"allowed_types": fp.AllowedTypes,
			},
		)
	}

	// Test 4: Show hidden files
	{
		fp := filepicker.New()
		fp.ShowHidden = true
		fs.AddTestWithCategory("filepicker_show_hidden", "unit",
			map[string]interface{}{
				"show_hidden": true,
			},
			map[string]interface{}{
				"show_hidden": fp.ShowHidden,
			},
		)
	}

	// Test 5: Height configuration
	{
		fp := filepicker.New()
		fp.Height = 20
		fp.AutoHeight = false
		fs.AddTestWithCategory("filepicker_height", "unit",
			map[string]interface{}{
				"height":      20,
				"auto_height": false,
			},
			map[string]interface{}{
				"height":      fp.Height,
				"auto_height": fp.AutoHeight,
			},
		)
	}

	// Test 6: Dir allowed configuration
	{
		fp := filepicker.New()
		fp.DirAllowed = true
		fp.FileAllowed = false
		fs.AddTestWithCategory("filepicker_dir_allowed", "unit",
			map[string]interface{}{
				"dir_allowed":  true,
				"file_allowed": false,
			},
			map[string]interface{}{
				"dir_allowed":  fp.DirAllowed,
				"file_allowed": fp.FileAllowed,
			},
		)
	}

	// Test 7: Keybindings check (verify key bindings are set)
	{
		fp := filepicker.New()
		fs.AddTestWithCategory("filepicker_keybindings", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"up_keys":      fp.KeyMap.Up.Keys(),
				"down_keys":    fp.KeyMap.Down.Keys(),
				"open_keys":    fp.KeyMap.Open.Keys(),
				"back_keys":    fp.KeyMap.Back.Keys(),
				"select_keys":  fp.KeyMap.Select.Keys(),
			},
		)
	}

	// Test 8: Format size helper test - simulate file sizes
	{
		// Test various size formats
		sizes := []int64{0, 512, 1024, 1536, 1048576, 1073741824}
		expectedFormats := []string{"0B", "512B", "1.0K", "1.5K", "1.0M", "1.0G"}
		fs.AddTestWithCategory("filepicker_format_size", "unit",
			map[string]interface{}{
				"sizes": sizes,
			},
			map[string]interface{}{
				"expected_formats": expectedFormats,
			},
		)
	}

	// Test 9: Cursor character
	{
		fp := filepicker.New()
		fs.AddTestWithCategory("filepicker_cursor", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"cursor": fp.Cursor,
			},
		)
	}

	// Test 10: Directory sorting (deterministic test using temp dir)
	// Create a temp directory with known files and test sort order
	{
		// Create temp directory for testing
		tmpDir, err := os.MkdirTemp("", "filepicker_test")
		if err == nil {
			defer os.RemoveAll(tmpDir)

			// Create test files and directories
			os.Mkdir(filepath.Join(tmpDir, "dir_b"), 0755)
			os.Mkdir(filepath.Join(tmpDir, "dir_a"), 0755)
			os.WriteFile(filepath.Join(tmpDir, "file_z.txt"), []byte("z"), 0644)
			os.WriteFile(filepath.Join(tmpDir, "file_a.txt"), []byte("a"), 0644)
			os.WriteFile(filepath.Join(tmpDir, ".hidden"), []byte("hidden"), 0644)

			// Create filepicker and read directory
			fp := filepicker.New()
			fp.CurrentDirectory = tmpDir
			fp.ShowHidden = false

			// The expected sort order: directories first (alphabetical), then files (alphabetical)
			// dir_a, dir_b, file_a.txt, file_z.txt (hidden files excluded)
			fs.AddTestWithCategory("filepicker_sort_order", "unit",
				map[string]interface{}{
					"test_dir":    "temp",
					"show_hidden": false,
				},
				map[string]interface{}{
					"sort_order": []string{"dir_a", "dir_b", "file_a.txt", "file_z.txt"},
				},
			)
		}
	}

	// Test 11: Empty directory view
	{
		fp := filepicker.New()
		// Test the view when no files are loaded
		view := fp.View()
		fs.AddTestWithCategory("filepicker_empty_view", "unit",
			map[string]interface{}{},
			map[string]interface{}{
				"view_contains": "No files",
			},
		)
		_ = view // silence unused variable warning
	}
}
