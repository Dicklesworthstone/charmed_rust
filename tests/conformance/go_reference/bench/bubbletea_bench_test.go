package bench

import (
	"fmt"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
)

// BenchMsg is a simple message for benchmarking
type BenchMsg int

const (
	Increment BenchMsg = iota
	Decrement
	NoOp
)

// BenchModel is a minimal model for benchmarking
type BenchModel struct {
	count int64
}

func (m BenchModel) Init() tea.Cmd {
	return nil
}

func (m BenchModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case BenchMsg:
		switch msg {
		case Increment:
			m.count++
		case Decrement:
			m.count--
		case NoOp:
		}
	}
	return m, nil
}

func (m BenchModel) View() string {
	return fmt.Sprintf("Count: %d", m.count)
}

// ComplexModel represents a list with selection
type ComplexModel struct {
	items    []string
	selected int
}

func (m ComplexModel) Init() tea.Cmd {
	return nil
}

func (m ComplexModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	return m, nil
}

func (m ComplexModel) View() string {
	var b strings.Builder
	b.Grow(len(m.items) * 24)
	for i, item := range m.items {
		if i == m.selected {
			b.WriteString("> ")
		} else {
			b.WriteString("  ")
		}
		b.WriteString(item)
		b.WriteString("\n")
	}
	return b.String()
}

// Message Dispatch Benchmarks - matches bubbletea/message_dispatch group

func BenchmarkSingleMessage(b *testing.B) {
	for i := 0; i < b.N; i++ {
		var m tea.Model = BenchModel{count: 0}
		m, _ = m.Update(Increment)
		_ = m.(BenchModel).count
	}
}

func BenchmarkMessages1000(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		var m tea.Model = BenchModel{count: 0}
		for j := 0; j < 1000; j++ {
			m, _ = m.Update(Increment)
		}
		_ = m.(BenchModel).count
	}
}

func BenchmarkMessages1000Mixed(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		var m tea.Model = BenchModel{count: 0}
		for j := 0; j < 500; j++ {
			m, _ = m.Update(Increment)
			m, _ = m.Update(Decrement)
		}
		_ = m.(BenchModel).count
	}
}

// View Rendering Benchmarks - matches bubbletea/view_rendering group

func BenchmarkSimpleView(b *testing.B) {
	m := BenchModel{count: 42}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = m.View()
	}
}

func BenchmarkListView100Items(b *testing.B) {
	items := make([]string, 100)
	for j := 0; j < 100; j++ {
		items[j] = fmt.Sprintf("Item %d", j)
	}
	m := ComplexModel{items: items, selected: 50}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = m.View()
	}
}

// Command Benchmarks - matches bubbletea/commands group

func BenchmarkCmdNone(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = tea.Cmd(nil)
	}
}

func BenchmarkCmdBatch10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmds := make([]tea.Cmd, 10)
		for j := range cmds {
			cmds[j] = func() tea.Msg { return NoOp }
		}
		_ = tea.Batch(cmds...)
	}
}

func BenchmarkCmdSequence10(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmds := make([]tea.Cmd, 10)
		for j := range cmds {
			cmds[j] = func() tea.Msg { return NoOp }
		}
		_ = tea.Sequence(cmds...)
	}
}

// Event Loop Simulation Benchmarks - matches bubbletea/event_loop group

func BenchmarkFrameCycle(b *testing.B) {
	for i := 0; i < b.N; i++ {
		var m tea.Model = BenchModel{count: 0}
		m, _ = m.Update(Increment)
		_ = m.View()
	}
}

func BenchmarkFrames60fps1sec(b *testing.B) {
	for i := 0; i < b.N; i++ {
		var m tea.Model = BenchModel{count: 0}
		for j := 0; j < 60; j++ {
			m, _ = m.Update(Increment)
			_ = m.View()
		}
	}
}

// Key Parsing Benchmarks - matches bubbletea/key_parsing group
// Note: These require parsing ANSI sequences which is internal to bubbletea
// We benchmark the overall key handling instead

type KeyTestModel struct {
	keys int
}

func (m KeyTestModel) Init() tea.Cmd { return nil }
func (m KeyTestModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg.(type) {
	case tea.KeyMsg:
		m.keys++
	}
	return m, nil
}
func (m KeyTestModel) View() string { return "" }

func BenchmarkKeyMsgCreate(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = tea.KeyMsg{
			Type:  tea.KeyRunes,
			Runes: []rune{'a'},
		}
	}
}

func BenchmarkKeyMsgCtrl(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = tea.KeyMsg{
			Type: tea.KeyCtrlC,
		}
	}
}

func BenchmarkKeyMsgSpecial(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = tea.KeyMsg{
			Type: tea.KeyTab,
			Alt:  true,
		}
	}
}
