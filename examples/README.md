# charmed_rust Examples

This directory contains example applications demonstrating how to use the charmed_rust TUI framework.

## Directory Structure

```
examples/
├── basic/              # Simple, single-concept examples
│   ├── counter/        # Basic counter with keyboard input
│   ├── spinner/        # Animated spinner with bubbles component
│   └── textinput/      # Text input with focus and submission
├── intermediate/       # Multi-component examples
│   ├── todo-list/      # Todo list with state management
│   ├── viewport/       # Scrollable content viewer
│   └── progress/       # Async progress bar with tick commands
├── advanced/          # Complex applications
│   ├── form/           # Multi-step form with validation
│   ├── markdown-viewer/# Markdown rendering with glamour
│   └── multi-component/# Dashboard with focus management
└── showcase/          # Full-featured demos (coming soon)
```

## Running Examples

From this directory, run any example using:

```bash
cargo run -p example-counter
```

Or from the project root:

```bash
cd examples && cargo run -p example-counter
```

## Examples Index

### Basic

| Example | Description | Crates Used |
|---------|-------------|-------------|
| counter | Simple increment/decrement counter | bubbletea |
| spinner | Animated loading spinner | bubbletea, bubbles, lipgloss |
| textinput | Text input with form submission | bubbletea, bubbles, lipgloss |

### Intermediate

| Example | Description | Crates Used |
|---------|-------------|-------------|
| [todo-list](intermediate/todo-list/) | Todo list with add/delete/toggle | bubbletea, lipgloss |
| [viewport](intermediate/viewport/) | Scrollable content viewer | bubbletea, bubbles, lipgloss |
| [progress](intermediate/progress/) | Async progress bar with tick commands | bubbletea, bubbles, lipgloss |

### Advanced

| Example | Description | Crates Used |
|---------|-------------|-------------|
| [form](advanced/form/) | Multi-step form with validation | bubbletea, huh, lipgloss |
| [markdown-viewer](advanced/markdown-viewer/) | Markdown rendering with scrolling | bubbletea, bubbles, glamour |
| [multi-component](advanced/multi-component/) | Dashboard with focus management | bubbletea, bubbles, lipgloss |

## Adding New Examples

1. Create a new directory under the appropriate category
2. Add a `Cargo.toml` using workspace dependencies
3. Implement your example in `src/main.rs`
4. Update this README

## See Also

- [Example Audit](../docs/example-audit.md) - Full catalog of Go examples being ported
- [bubbletea crate examples](../crates/bubbletea/examples/) - In-crate examples
