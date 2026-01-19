package bench

import (
	"strings"
	"testing"

	"github.com/charmbracelet/lipgloss"
)

const (
	sampleLine      = "The quick brown fox jumps over the lazy dog."
	sampleParagraph = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt."
)

// Style Creation Benchmarks - matches lipgloss/style_creation group

func BenchmarkStyleNew(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.NewStyle()
	}
}

func BenchmarkStyleNewWithAllProps(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.NewStyle().
			Foreground(lipgloss.Color("#ff0000")).
			Background(lipgloss.Color("#0000ff")).
			Bold(true).
			Italic(true).
			Underline(true).
			Padding(1, 2, 1, 2).
			Margin(1, 1, 1, 1).
			Border(lipgloss.RoundedBorder())
	}
}

// Color Benchmarks - matches lipgloss/colors group

func BenchmarkAnsiColorFrom(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.Color("196")
	}
}

func BenchmarkColorHexParse(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.Color("#FF8040")
	}
}

func BenchmarkColorAnsiParse(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.Color("196")
	}
}

func BenchmarkAdaptiveColorRender(b *testing.B) {
	adaptive := lipgloss.AdaptiveColor{Light: "#000000", Dark: "#ffffff"}
	style := lipgloss.NewStyle().Foreground(adaptive)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render("test")
	}
}

// Rendering Benchmarks - matches lipgloss/rendering group

func BenchmarkRenderShortSimple(b *testing.B) {
	style := lipgloss.NewStyle().Foreground(lipgloss.Color("#ff0000"))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(sampleLine)
	}
}

func BenchmarkRenderShortComplex(b *testing.B) {
	style := lipgloss.NewStyle().
		Foreground(lipgloss.Color("#ff0000")).
		Background(lipgloss.Color("#0000ff")).
		Bold(true).
		Padding(1, 2).
		Border(lipgloss.RoundedBorder())
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(sampleLine)
	}
}

func BenchmarkRenderMediumSimple(b *testing.B) {
	style := lipgloss.NewStyle().Foreground(lipgloss.Color("#ff0000"))
	medium := sampleParagraph + "\n" + sampleParagraph + "\n" + sampleLine
	b.SetBytes(int64(len(medium)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(medium)
	}
}

func BenchmarkRenderLongSimple(b *testing.B) {
	style := lipgloss.NewStyle().Foreground(lipgloss.Color("#ff0000"))
	var sb strings.Builder
	for j := 0; j < 80; j++ {
		sb.WriteString(sampleParagraph)
		sb.WriteString("\n")
	}
	long := sb.String()
	b.SetBytes(int64(len(long)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(long)
	}
}

// Layout Benchmarks - matches lipgloss/layout group

func BenchmarkJoinHorizontal10(b *testing.B) {
	items := make([]string, 10)
	for j := 0; j < 10; j++ {
		items[j] = sampleLine
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = lipgloss.JoinHorizontal(lipgloss.Top, items...)
	}
}

func BenchmarkJoinVertical10(b *testing.B) {
	items := make([]string, 10)
	for j := 0; j < 10; j++ {
		items[j] = sampleLine
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = lipgloss.JoinVertical(lipgloss.Left, items...)
	}
}

func BenchmarkPlace(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = lipgloss.Place(80, 24, lipgloss.Center, lipgloss.Center, sampleLine)
	}
}

// Border Benchmarks - matches lipgloss/borders group

func BenchmarkBorderNone(b *testing.B) {
	style := lipgloss.NewStyle()
	content := sampleLine + "\n" + sampleParagraph + "\n" + sampleLine
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(content)
	}
}

func BenchmarkBorderNormal(b *testing.B) {
	style := lipgloss.NewStyle().Border(lipgloss.NormalBorder())
	content := sampleLine + "\n" + sampleParagraph + "\n" + sampleLine
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(content)
	}
}

func BenchmarkBorderRounded(b *testing.B) {
	style := lipgloss.NewStyle().Border(lipgloss.RoundedBorder())
	content := sampleLine + "\n" + sampleParagraph + "\n" + sampleLine
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(content)
	}
}

func BenchmarkBorderDouble(b *testing.B) {
	style := lipgloss.NewStyle().Border(lipgloss.DoubleBorder())
	content := sampleLine + "\n" + sampleParagraph + "\n" + sampleLine
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = style.Render(content)
	}
}
