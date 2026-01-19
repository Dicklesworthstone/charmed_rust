package bench

import (
	"fmt"
	"strings"
	"testing"

	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
)

// BenchItem implements list.Item for benchmarking
type BenchItem struct {
	title string
}

func (i BenchItem) Title() string       { return i.title }
func (i BenchItem) Description() string { return "" }
func (i BenchItem) FilterValue() string { return i.title }

func buildItems(count int) []list.Item {
	items := make([]list.Item, count)
	for i := 0; i < count; i++ {
		items[i] = BenchItem{title: fmt.Sprintf("Item %d", i)}
	}
	return items
}

func buildTableColumns() []table.Column {
	return []table.Column{
		{Title: "Name", Width: 18},
		{Title: "Status", Width: 12},
		{Title: "Region", Width: 12},
		{Title: "Score", Width: 8},
	}
}

func buildTableRows(count int) []table.Row {
	rows := make([]table.Row, count)
	for i := 0; i < count; i++ {
		status := "Online"
		if i%2 != 0 {
			status = "Offline"
		}
		rows[i] = table.Row{
			fmt.Sprintf("Person %d", i),
			status,
			fmt.Sprintf("Zone %d", i%8),
			fmt.Sprintf("%d", i*7),
		}
	}
	return rows
}

func buildViewportContent(lines int) string {
	var sb strings.Builder
	for i := 0; i < lines; i++ {
		sb.WriteString(fmt.Sprintf("Line %d: Some content here with more text\n", i))
	}
	return sb.String()
}

// List Benchmarks - matches bubbles/list group

func BenchmarkListCreate10(b *testing.B) {
	items := buildItems(10)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = list.New(items, list.NewDefaultDelegate(), 80, 20)
	}
}

func BenchmarkListCreate100(b *testing.B) {
	items := buildItems(100)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = list.New(items, list.NewDefaultDelegate(), 80, 20)
	}
}

func BenchmarkListCreate1000(b *testing.B) {
	items := buildItems(1000)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = list.New(items, list.NewDefaultDelegate(), 80, 20)
	}
}

func BenchmarkListView100(b *testing.B) {
	l := list.New(buildItems(100), list.NewDefaultDelegate(), 80, 20)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = l.View()
	}
}

func BenchmarkListNavigate100(b *testing.B) {
	for i := 0; i < b.N; i++ {
		l := list.New(buildItems(100), list.NewDefaultDelegate(), 80, 20)
		for j := 0; j < 10; j++ {
			l.CursorDown()
		}
		for j := 0; j < 5; j++ {
			l.CursorUp()
		}
		_ = l.SelectedItem()
	}
}

func BenchmarkListFilter100(b *testing.B) {
	for i := 0; i < b.N; i++ {
		l := list.New(buildItems(100), list.NewDefaultDelegate(), 80, 20)
		l.SetFilteringEnabled(true)
		// Note: Filtering in Go bubbles works differently
		_ = l.View()
	}
}

// Table Benchmarks - matches bubbles/table group

func BenchmarkTableView10(b *testing.B) {
	t := table.New(
		table.WithColumns(buildTableColumns()),
		table.WithRows(buildTableRows(10)),
		table.WithWidth(80),
		table.WithHeight(20),
		table.WithFocused(true),
	)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = t.View()
	}
}

func BenchmarkTableView100(b *testing.B) {
	t := table.New(
		table.WithColumns(buildTableColumns()),
		table.WithRows(buildTableRows(100)),
		table.WithWidth(80),
		table.WithHeight(20),
		table.WithFocused(true),
	)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = t.View()
	}
}

func BenchmarkTableView1000(b *testing.B) {
	t := table.New(
		table.WithColumns(buildTableColumns()),
		table.WithRows(buildTableRows(1000)),
		table.WithWidth(80),
		table.WithHeight(20),
		table.WithFocused(true),
	)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = t.View()
	}
}

func BenchmarkTableNavigate(b *testing.B) {
	t := table.New(
		table.WithColumns(buildTableColumns()),
		table.WithRows(buildTableRows(200)),
		table.WithWidth(80),
		table.WithHeight(20),
		table.WithFocused(true),
	)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		t.MoveDown(10)
		t.MoveUp(5)
		t.GotoBottom()
		t.GotoTop()
		_ = t.SelectedRow()
	}
}

func BenchmarkTableSetColumnsRows(b *testing.B) {
	columns := buildTableColumns()
	rows := buildTableRows(150)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		t := table.New(
			table.WithWidth(80),
			table.WithHeight(20),
		)
		t.SetColumns(columns)
		t.SetRows(rows)
		_ = t.View()
	}
}

// Viewport Benchmarks - matches bubbles/viewport group

func BenchmarkViewportRender100(b *testing.B) {
	content := buildViewportContent(100)
	vp := viewport.New(80, 24)
	vp.SetContent(content)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = vp.View()
	}
}

func BenchmarkViewportRender1000(b *testing.B) {
	content := buildViewportContent(1000)
	vp := viewport.New(80, 24)
	vp.SetContent(content)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = vp.View()
	}
}

func BenchmarkViewportRender10000(b *testing.B) {
	content := buildViewportContent(10000)
	vp := viewport.New(80, 24)
	vp.SetContent(content)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = vp.View()
	}
}

func BenchmarkViewportScrollOps(b *testing.B) {
	content := buildViewportContent(2000)
	for i := 0; i < b.N; i++ {
		vp := viewport.New(80, 24)
		vp.SetContent(content)
		vp.LineDown(5)
		vp.LineUp(2)
		vp.HalfViewDown()
		vp.HalfViewUp()
		_ = vp.View()
	}
}

// TextInput Benchmarks - matches bubbles/textinput group

func BenchmarkTextInputCreate(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = textinput.New()
	}
}

func BenchmarkTextInputViewWithText(b *testing.B) {
	ti := textinput.New()
	ti.SetValue("Hello, World!")
	ti.Focus()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = ti.View()
	}
}

func BenchmarkTextInputInsertChars(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ti := textinput.New()
		ti.Focus()
		for _, c := range "abcde" {
			ti.SetValue(ti.Value() + string(c))
		}
		_ = ti.Value()
	}
}

func BenchmarkTextInputCursorMovement(b *testing.B) {
	for i := 0; i < b.N; i++ {
		ti := textinput.New()
		ti.SetValue(strings.Repeat("x", 1000))
		ti.Focus()
		ti.CursorStart()
		ti.CursorEnd()
		_ = ti.Position()
	}
}

// Paginator Benchmarks - matches bubbles/paginator group

func BenchmarkPaginatorViewArabic(b *testing.B) {
	p := paginator.New()
	p.SetTotalPages(100)
	p.PerPage = 10
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = p.View()
	}
}

func BenchmarkPaginatorViewDots(b *testing.B) {
	p := paginator.New()
	p.Type = paginator.Dots
	p.SetTotalPages(10)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = p.View()
	}
}

// Spinner and Progress Benchmarks - matches bubbles/animated group

func BenchmarkSpinnerView(b *testing.B) {
	s := spinner.New()
	s.Spinner = spinner.Dot
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = s.View()
	}
}

func BenchmarkSpinnerUpdate(b *testing.B) {
	for i := 0; i < b.N; i++ {
		s := spinner.New()
		s.Spinner = spinner.Dot
		s, _ = s.Update(s.Tick())
		_ = s.View()
	}
}

func BenchmarkProgressView50(b *testing.B) {
	p := progress.New(progress.WithDefaultGradient())
	p.Width = 40
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = p.ViewAs(0.5)
	}
}

func BenchmarkProgressSetPercent(b *testing.B) {
	for i := 0; i < b.N; i++ {
		p := progress.New(progress.WithDefaultGradient())
		p.Width = 40
		p.SetPercent(0.75)
		_ = p.View()
	}
}
