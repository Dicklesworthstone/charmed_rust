// Bubbletea capture program - captures key and mouse event parsing behaviors
package main

import (
	"charmed_conformance/internal/capture"
	"flag"
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	outputDir := flag.String("output", "output", "Output directory for fixtures")
	flag.Parse()

	fixtures := capture.NewFixtureSet("bubbletea", "1.3.4")

	// Capture key type tests
	captureKeyTypeTests(fixtures)

	// Capture key sequence tests
	captureKeySequenceTests(fixtures)

	// Capture mouse button tests
	captureMouseButtonTests(fixtures)

	// Capture mouse action tests
	captureMouseActionTests(fixtures)

	// Capture mouse event string tests
	captureMouseEventStringTests(fixtures)

	// Capture key string tests
	captureKeyStringTests(fixtures)

	if err := fixtures.WriteToFile(*outputDir); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func captureKeyTypeTests(fs *capture.FixtureSet) {
	// Capture key type constants
	keyTypes := []struct {
		name  string
		value tea.KeyType
	}{
		{"KeyNull", tea.KeyNull},
		{"KeyBreak", tea.KeyBreak},
		{"KeyEnter", tea.KeyEnter},
		{"KeyBackspace", tea.KeyBackspace},
		{"KeyTab", tea.KeyTab},
		{"KeyEsc", tea.KeyEsc},
		{"KeyEscape", tea.KeyEscape},
		{"KeyCtrlA", tea.KeyCtrlA},
		{"KeyCtrlB", tea.KeyCtrlB},
		{"KeyCtrlC", tea.KeyCtrlC},
		{"KeyCtrlD", tea.KeyCtrlD},
		{"KeyCtrlE", tea.KeyCtrlE},
		{"KeyCtrlF", tea.KeyCtrlF},
		{"KeyCtrlG", tea.KeyCtrlG},
		{"KeyCtrlH", tea.KeyCtrlH},
		{"KeyCtrlI", tea.KeyCtrlI},
		{"KeyCtrlJ", tea.KeyCtrlJ},
		{"KeyCtrlK", tea.KeyCtrlK},
		{"KeyCtrlL", tea.KeyCtrlL},
		{"KeyCtrlM", tea.KeyCtrlM},
		{"KeyCtrlN", tea.KeyCtrlN},
		{"KeyCtrlO", tea.KeyCtrlO},
		{"KeyCtrlP", tea.KeyCtrlP},
		{"KeyCtrlQ", tea.KeyCtrlQ},
		{"KeyCtrlR", tea.KeyCtrlR},
		{"KeyCtrlS", tea.KeyCtrlS},
		{"KeyCtrlT", tea.KeyCtrlT},
		{"KeyCtrlU", tea.KeyCtrlU},
		{"KeyCtrlV", tea.KeyCtrlV},
		{"KeyCtrlW", tea.KeyCtrlW},
		{"KeyCtrlX", tea.KeyCtrlX},
		{"KeyCtrlY", tea.KeyCtrlY},
		{"KeyCtrlZ", tea.KeyCtrlZ},
		{"KeyRunes", tea.KeyRunes},
		{"KeyUp", tea.KeyUp},
		{"KeyDown", tea.KeyDown},
		{"KeyRight", tea.KeyRight},
		{"KeyLeft", tea.KeyLeft},
		{"KeyShiftTab", tea.KeyShiftTab},
		{"KeyHome", tea.KeyHome},
		{"KeyEnd", tea.KeyEnd},
		{"KeyPgUp", tea.KeyPgUp},
		{"KeyPgDown", tea.KeyPgDown},
		{"KeyDelete", tea.KeyDelete},
		{"KeyInsert", tea.KeyInsert},
		{"KeySpace", tea.KeySpace},
		{"KeyF1", tea.KeyF1},
		{"KeyF2", tea.KeyF2},
		{"KeyF3", tea.KeyF3},
		{"KeyF4", tea.KeyF4},
		{"KeyF5", tea.KeyF5},
		{"KeyF6", tea.KeyF6},
		{"KeyF7", tea.KeyF7},
		{"KeyF8", tea.KeyF8},
		{"KeyF9", tea.KeyF9},
		{"KeyF10", tea.KeyF10},
		{"KeyF11", tea.KeyF11},
		{"KeyF12", tea.KeyF12},
	}

	for _, kt := range keyTypes {
		fs.AddTestWithCategory(fmt.Sprintf("keytype_%s", kt.name), "unit",
			map[string]string{
				"key_type": kt.name,
			},
			map[string]interface{}{
				"value":       int(kt.value),
				"string_name": kt.value.String(),
			},
		)
	}
}

func captureKeySequenceTests(fs *capture.FixtureSet) {
	// Key sequence parsing tests - these test ANSI escape sequence parsing
	sequences := []struct {
		name     string
		sequence string
		keyType  tea.KeyType
		alt      bool
	}{
		// Arrow keys
		{"arrow_up", "\x1b[A", tea.KeyUp, false},
		{"arrow_down", "\x1b[B", tea.KeyDown, false},
		{"arrow_right", "\x1b[C", tea.KeyRight, false},
		{"arrow_left", "\x1b[D", tea.KeyLeft, false},
		{"shift_up", "\x1b[1;2A", tea.KeyShiftUp, false},
		{"shift_down", "\x1b[1;2B", tea.KeyShiftDown, false},
		{"shift_right", "\x1b[1;2C", tea.KeyShiftRight, false},
		{"shift_left", "\x1b[1;2D", tea.KeyShiftLeft, false},
		{"alt_up", "\x1b[1;3A", tea.KeyUp, true},
		{"alt_down", "\x1b[1;3B", tea.KeyDown, true},
		{"alt_right", "\x1b[1;3C", tea.KeyRight, true},
		{"alt_left", "\x1b[1;3D", tea.KeyLeft, true},
		{"ctrl_up", "\x1b[1;5A", tea.KeyCtrlUp, false},
		{"ctrl_down", "\x1b[1;5B", tea.KeyCtrlDown, false},
		{"ctrl_right", "\x1b[1;5C", tea.KeyCtrlRight, false},
		{"ctrl_left", "\x1b[1;5D", tea.KeyCtrlLeft, false},

		// Navigation keys
		{"shift_tab", "\x1b[Z", tea.KeyShiftTab, false},
		{"insert", "\x1b[2~", tea.KeyInsert, false},
		{"delete", "\x1b[3~", tea.KeyDelete, false},
		{"page_up", "\x1b[5~", tea.KeyPgUp, false},
		{"page_down", "\x1b[6~", tea.KeyPgDown, false},
		{"home_1", "\x1b[1~", tea.KeyHome, false},
		{"home_2", "\x1b[H", tea.KeyHome, false},
		{"end_1", "\x1b[4~", tea.KeyEnd, false},
		{"end_2", "\x1b[F", tea.KeyEnd, false},

		// Function keys (vt100/xterm)
		{"f1_vt100", "\x1bOP", tea.KeyF1, false},
		{"f2_vt100", "\x1bOQ", tea.KeyF2, false},
		{"f3_vt100", "\x1bOR", tea.KeyF3, false},
		{"f4_vt100", "\x1bOS", tea.KeyF4, false},
		{"f5", "\x1b[15~", tea.KeyF5, false},
		{"f6", "\x1b[17~", tea.KeyF6, false},
		{"f7", "\x1b[18~", tea.KeyF7, false},
		{"f8", "\x1b[19~", tea.KeyF8, false},
		{"f9", "\x1b[20~", tea.KeyF9, false},
		{"f10", "\x1b[21~", tea.KeyF10, false},
		{"f11", "\x1b[23~", tea.KeyF11, false},
		{"f12", "\x1b[24~", tea.KeyF12, false},
	}

	for _, seq := range sequences {
		fs.AddTestWithCategory(fmt.Sprintf("sequence_%s", seq.name), "unit",
			map[string]interface{}{
				"sequence": seq.sequence,
			},
			map[string]interface{}{
				"key_type": int(seq.keyType),
				"alt":      seq.alt,
			},
		)
	}
}

func captureMouseButtonTests(fs *capture.FixtureSet) {
	buttons := []struct {
		name  string
		value tea.MouseButton
	}{
		{"MouseButtonNone", tea.MouseButtonNone},
		{"MouseButtonLeft", tea.MouseButtonLeft},
		{"MouseButtonMiddle", tea.MouseButtonMiddle},
		{"MouseButtonRight", tea.MouseButtonRight},
		{"MouseButtonWheelUp", tea.MouseButtonWheelUp},
		{"MouseButtonWheelDown", tea.MouseButtonWheelDown},
		{"MouseButtonWheelLeft", tea.MouseButtonWheelLeft},
		{"MouseButtonWheelRight", tea.MouseButtonWheelRight},
		{"MouseButtonBackward", tea.MouseButtonBackward},
		{"MouseButtonForward", tea.MouseButtonForward},
	}

	for _, btn := range buttons {
		fs.AddTestWithCategory(fmt.Sprintf("mouse_button_%s", btn.name), "unit",
			map[string]string{
				"button": btn.name,
			},
			map[string]interface{}{
				"value": int(btn.value),
			},
		)
	}
}

func captureMouseActionTests(fs *capture.FixtureSet) {
	actions := []struct {
		name  string
		value tea.MouseAction
	}{
		{"MouseActionPress", tea.MouseActionPress},
		{"MouseActionRelease", tea.MouseActionRelease},
		{"MouseActionMotion", tea.MouseActionMotion},
	}

	for _, act := range actions {
		fs.AddTestWithCategory(fmt.Sprintf("mouse_action_%s", act.name), "unit",
			map[string]string{
				"action": act.name,
			},
			map[string]interface{}{
				"value": int(act.value),
			},
		)
	}
}

func captureMouseEventStringTests(fs *capture.FixtureSet) {
	// Test mouse event string representations
	events := []struct {
		name   string
		event  tea.MouseEvent
		expect string
	}{
		{
			"left_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonLeft, Action: tea.MouseActionPress},
			"left press",
		},
		{
			"left_release",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonNone, Action: tea.MouseActionRelease},
			"release",
		},
		{
			"right_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonRight, Action: tea.MouseActionPress},
			"right press",
		},
		{
			"middle_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonMiddle, Action: tea.MouseActionPress},
			"middle press",
		},
		{
			"wheel_up",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonWheelUp, Action: tea.MouseActionPress},
			"wheel up",
		},
		{
			"wheel_down",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonWheelDown, Action: tea.MouseActionPress},
			"wheel down",
		},
		{
			"motion",
			tea.MouseEvent{X: 10, Y: 20, Button: tea.MouseButtonNone, Action: tea.MouseActionMotion},
			"motion",
		},
		{
			"ctrl_left_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonLeft, Action: tea.MouseActionPress, Ctrl: true},
			"ctrl+left press",
		},
		{
			"alt_left_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonLeft, Action: tea.MouseActionPress, Alt: true},
			"alt+left press",
		},
		{
			"shift_left_click",
			tea.MouseEvent{X: 0, Y: 0, Button: tea.MouseButtonLeft, Action: tea.MouseActionPress, Shift: true},
			"shift+left press",
		},
		{
			"ctrl_alt_shift_left_click",
			tea.MouseEvent{
				X: 0, Y: 0, Button: tea.MouseButtonLeft, Action: tea.MouseActionPress,
				Ctrl: true, Alt: true, Shift: true,
			},
			"ctrl+alt+shift+left press",
		},
	}

	for _, evt := range events {
		fs.AddTestWithCategory(fmt.Sprintf("mouse_event_%s", evt.name), "unit",
			map[string]interface{}{
				"x":      evt.event.X,
				"y":      evt.event.Y,
				"button": int(evt.event.Button),
				"action": int(evt.event.Action),
				"ctrl":   evt.event.Ctrl,
				"alt":    evt.event.Alt,
				"shift":  evt.event.Shift,
			},
			map[string]string{
				"string": evt.event.String(),
			},
		)
	}
}

func captureKeyStringTests(fs *capture.FixtureSet) {
	// Test key string representations
	keys := []struct {
		name   string
		key    tea.Key
		expect string
	}{
		{
			"enter",
			tea.Key{Type: tea.KeyEnter},
			"enter",
		},
		{
			"backspace",
			tea.Key{Type: tea.KeyBackspace},
			"backspace",
		},
		{
			"tab",
			tea.Key{Type: tea.KeyTab},
			"tab",
		},
		{
			"escape",
			tea.Key{Type: tea.KeyEscape},
			"esc",
		},
		{
			"ctrl_c",
			tea.Key{Type: tea.KeyCtrlC},
			"ctrl+c",
		},
		{
			"ctrl_d",
			tea.Key{Type: tea.KeyCtrlD},
			"ctrl+d",
		},
		{
			"up",
			tea.Key{Type: tea.KeyUp},
			"up",
		},
		{
			"down",
			tea.Key{Type: tea.KeyDown},
			"down",
		},
		{
			"left",
			tea.Key{Type: tea.KeyLeft},
			"left",
		},
		{
			"right",
			tea.Key{Type: tea.KeyRight},
			"right",
		},
		{
			"f1",
			tea.Key{Type: tea.KeyF1},
			"f1",
		},
		{
			"rune_a",
			tea.Key{Type: tea.KeyRunes, Runes: []rune{'a'}},
			"a",
		},
		{
			"rune_hello",
			tea.Key{Type: tea.KeyRunes, Runes: []rune{'h', 'e', 'l', 'l', 'o'}},
			"hello",
		},
		{
			"alt_a",
			tea.Key{Type: tea.KeyRunes, Runes: []rune{'a'}, Alt: true},
			"alt+a",
		},
		{
			"alt_up",
			tea.Key{Type: tea.KeyUp, Alt: true},
			"alt+up",
		},
		{
			"space",
			tea.Key{Type: tea.KeySpace},
			" ",
		},
		{
			"paste_text",
			tea.Key{Type: tea.KeyRunes, Runes: []rune{'t', 'e', 's', 't'}, Paste: true},
			"[test]",
		},
	}

	for _, k := range keys {
		runeStrs := make([]string, len(k.key.Runes))
		for i, r := range k.key.Runes {
			runeStrs[i] = string(r)
		}

		fs.AddTestWithCategory(fmt.Sprintf("key_string_%s", k.name), "unit",
			map[string]interface{}{
				"type":  int(k.key.Type),
				"runes": runeStrs,
				"alt":   k.key.Alt,
				"paste": k.key.Paste,
			},
			map[string]string{
				"string": k.key.String(),
			},
		)
	}
}
