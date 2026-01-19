package bench

import (
	"fmt"
	"strings"
	"testing"

	"github.com/charmbracelet/glamour"
)

const smallDoc = `# Hello World

This is a simple markdown document.

- Item 1
- Item 2
- Item 3
`

const mediumDoc = `# Medium Document

This is a medium-sized markdown document with more content.

## Section 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

## Section 2

- First item
- Second item
- Third item

### Subsection

` + "```go\nfunc main() {\n    fmt.Println(\"Hello\")\n}\n```" + `

## Section 3

More text here with **bold** and *italic* formatting.

| Column 1 | Column 2 |
|----------|----------|
| A        | B        |
| C        | D        |
`

// generateLargeDoc creates a large markdown document
func generateLargeDoc() string {
	var sb strings.Builder
	sb.WriteString("# Large Document\n\n")
	for i := 0; i < 50; i++ {
		sb.WriteString(fmt.Sprintf("## Section %d\n\n", i))
		sb.WriteString("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ")
		sb.WriteString("Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n")
		sb.WriteString("- Item 1\n- Item 2\n- Item 3\n\n")
	}
	return sb.String()
}

// Full Render Benchmarks - matches glamour/render group

func BenchmarkRenderSmall(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	b.SetBytes(int64(len(smallDoc)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(smallDoc)
	}
}

func BenchmarkRenderMedium(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	b.SetBytes(int64(len(mediumDoc)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(mediumDoc)
	}
}

func BenchmarkRenderLarge(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	largeDoc := generateLargeDoc()
	b.SetBytes(int64(len(largeDoc)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(largeDoc)
	}
}

// Element Benchmarks - matches glamour/elements group

func BenchmarkHeaders(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	var sb strings.Builder
	for n := 1; n <= 6; n++ {
		sb.WriteString(strings.Repeat("#", n))
		sb.WriteString(fmt.Sprintf(" Header Level %d\n\n", n))
	}
	headers := strings.Repeat(sb.String(), 100)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(headers)
	}
}

func BenchmarkUnorderedList100(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	var sb strings.Builder
	for i := 0; i < 100; i++ {
		sb.WriteString(fmt.Sprintf("- Item %d\n", i))
	}
	list := sb.String()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(list)
	}
}

func BenchmarkNestedList(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	var sb strings.Builder
	for i := 0; i < 50; i++ {
		sb.WriteString(fmt.Sprintf("- Item %d\n", i))
		sb.WriteString(fmt.Sprintf("  - Nested %d\n", i))
		sb.WriteString(fmt.Sprintf("    - Deep %d\n", i))
	}
	nestedList := sb.String()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(nestedList)
	}
}

func BenchmarkCodeBlocks50(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	codeBlock := "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n"
	codeBlocks := strings.Repeat(codeBlock, 50)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(codeBlocks)
	}
}

func BenchmarkLinksEmphasis100(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	var sb strings.Builder
	for i := 0; i < 100; i++ {
		sb.WriteString(fmt.Sprintf("[Link %d](https://example.com/%d) and **bold** and *italic*\n", i, i))
	}
	links := sb.String()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(links)
	}
}

func BenchmarkTables50(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	table := "| Col 1 | Col 2 | Col 3 |\n|-------|-------|-------|\n| A | B | C |\n"
	tables := strings.Repeat(table, 50)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(tables)
	}
}

// Config Impact Benchmarks - matches glamour/config group

func BenchmarkDefaultDark(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(mediumDoc)
	}
}

func BenchmarkLightStyle(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("light"))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(mediumDoc)
	}
}

func BenchmarkASCIIStyle(b *testing.B) {
	r, _ := glamour.NewTermRenderer(glamour.WithStandardStyle("ascii"))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _ = r.Render(mediumDoc)
	}
}

// Renderer creation benchmark
func BenchmarkRendererCreate(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_, _ = glamour.NewTermRenderer(glamour.WithStandardStyle("dark"))
	}
}
