# Go Examples Audit for charmed_rust Port

This document catalogs all examples from the Charm Go ecosystem, categorized by complexity and dependencies, to guide the porting strategy for charmed_rust.

## Source Repositories

| Repository | URL | Example Count |
|------------|-----|---------------|
| bubbletea | github.com/charmbracelet/bubbletea/examples | 48 |
| lipgloss | github.com/charmbracelet/lipgloss/examples | 5 |
| glamour | github.com/charmbracelet/glamour/examples | 5 |
| huh | github.com/charmbracelet/huh/examples | 23 |
| **Total** | | **81** |

## Complexity Categories

### Basic (Single file, 1-2 components)
Simple examples that demonstrate core concepts with minimal dependencies.

### Intermediate (Multi-component, state management)
Examples that combine multiple bubbles or require more complex state handling.

### Advanced (Complex state, multiple views, external I/O)
Full applications with navigation, async operations, or complex interactions.

---

## Bubbletea Examples (48)

### Basic Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| simple | Minimal Bubble Tea app | bubbletea | 1 |
| spinner | Loading spinner | bubbletea, bubbles | 1 |
| spinners | Spinner variants showcase | bubbletea, bubbles | 1 |
| textinput | Single-line text input | bubbletea, bubbles | 1 |
| textarea | Multi-line text input | bubbletea, bubbles | 1 |
| timer | Simple countdown timer | bubbletea | 1 |
| stopwatch | Stopwatch application | bubbletea | 1 |
| result | Choice menu selection | bubbletea | 1 |
| send-msg | Custom message types | bubbletea | 1 |
| set-window-title | Terminal title control | bubbletea | 1 |
| window-size | Window size handling | bubbletea | 1 |
| focus-blur | Focus state management | bubbletea | 2 |
| mouse | Mouse event handling | bubbletea | 2 |
| debounce | Input throttling | bubbletea | 2 |
| sequence | Command sequencing | bubbletea | 2 |
| prevent-quit | Quit prevention | bubbletea | 2 |
| suspend | Process suspension | bubbletea | 2 |

### Intermediate Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| altscreen-toggle | Alt screen switching | bubbletea | 2 |
| fullscreen | Fullscreen mode | bubbletea | 2 |
| list-simple | Basic list component | bubbletea, bubbles | 2 |
| list-default | Standard list usage | bubbletea, bubbles | 2 |
| paginator | Paginated list | bubbletea, bubbles | 2 |
| progress-static | Static progress bar | bubbletea, bubbles | 2 |
| progress-animated | Animated progress | bubbletea, bubbles | 2 |
| table | Table component | bubbletea, bubbles | 2 |
| tabs | Tabbed navigation | bubbletea, lipgloss | 2 |
| help | Help display | bubbletea, bubbles | 2 |
| textinputs | Multiple text inputs | bubbletea, bubbles | 2 |
| split-editors | Split textarea view | bubbletea, bubbles | 2 |
| composable-views | Component composition | bubbletea, bubbles | 2 |
| views | Multi-view navigation | bubbletea | 2 |
| eyes | Interactive animation | bubbletea, lipgloss | 3 |
| cellbuffer | Cell buffer usage | bubbletea | 3 |
| realtime | Go channel communication | bubbletea | 3 |
| pipe | Shell pipe I/O | bubbletea | 3 |

### Advanced Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| list-fancy | Customized list | bubbletea, bubbles, lipgloss | 3 |
| table-resize | Resizable table | bubbletea, bubbles | 3 |
| credit-card-form | Multi-step form | bubbletea, bubbles | 3 |
| file-picker | File browser | bubbletea, bubbles | 3 |
| pager | Less-like pager | bubbletea, bubbles | 3 |
| glamour | Markdown viewer | bubbletea, bubbles, glamour | 3 |
| autocomplete | Autocomplete UI | bubbletea, bubbles | 3 |
| chat | Chat application | bubbletea, bubbles | 3 |
| http | HTTP requests | bubbletea (+ http client) | 3 |
| progress-download | Download progress | bubbletea, bubbles (+ http) | 3 |
| exec | External command exec | bubbletea | 3 |
| package-manager | Package manager UI | bubbletea, bubbles, lipgloss | 3 |
| tui-daemon-combo | TUI + daemon mode | bubbletea | 3 |

---

## Lipgloss Examples (5)

| Example | Description | Complexity | Priority |
|---------|-------------|------------|----------|
| layout | Layout primitives | Basic | 1 |
| list | Styled lists | Basic | 1 |
| table | Styled tables | Intermediate | 2 |
| tree | Tree structures | Intermediate | 2 |
| ssh | SSH styling | Advanced | 3 |

---

## Glamour Examples (5)

| Example | Description | Complexity | Priority |
|---------|-------------|------------|----------|
| helloworld | Basic markdown render | Basic | 1 |
| stdin-stdout | Pipe markdown | Basic | 1 |
| stdin-stdout-custom-styles | Custom theme | Intermediate | 2 |
| custom_renderer | Custom rendering | Intermediate | 2 |
| artichokes | Complex styling | Advanced | 3 |

---

## Huh Examples (23)

### Basic Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| readme | Basic form showcase | huh | 1 |
| burger | Food ordering form | huh | 1 |
| theme | Theme customization | huh, lipgloss | 1 |
| help | Help text display | huh | 1 |
| hide | Field hiding | huh | 2 |
| skip | Field skipping | huh | 2 |
| timer | Timed forms | huh | 2 |

### Intermediate Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| conditional | Conditional fields | huh | 2 |
| dynamic | Dynamic forms | huh | 2 |
| layout | Form layouts | huh, lipgloss | 2 |
| multiple-groups | Grouped fields | huh | 2 |
| scroll | Scrollable forms | huh | 2 |
| filepicker | File selection | huh, bubbles | 2 |
| filepicker-picking | File picker variants | huh, bubbles | 2 |
| accessibility | A11y features | huh | 2 |
| accessibility-secure-input | Secure a11y input | huh | 2 |
| stickers | Visual embellishments | huh, lipgloss | 3 |

### Advanced Examples

| Example | Description | Crates Required | Priority |
|---------|-------------|-----------------|----------|
| bubbletea | Bubble Tea integration | huh, bubbletea | 3 |
| bubbletea-options | Advanced BT options | huh, bubbletea | 3 |
| gh | GitHub CLI style | huh (complex) | 3 |
| git | Git commit wizard | huh (complex) | 3 |
| gum | Gum compatibility | huh, gum | 3 |
| ssh-form | SSH-based forms | huh, wish | 3 |

---

## Recommended Porting Order

### Phase 1: Foundation (Priority 1)
Focus on core patterns and simple demonstrations.

1. **bubbletea/simple** - Baseline app structure
2. **bubbletea/spinner** - Async tick patterns
3. **bubbletea/textinput** - User input handling
4. **bubbletea/timer** - Time-based updates
5. **lipgloss/layout** - Styling basics
6. **glamour/helloworld** - Markdown rendering

### Phase 2: Components (Priority 2)
Demonstrate component library usage.

1. **bubbletea/list-simple** - List navigation
2. **bubbletea/table** - Tabular data
3. **bubbletea/progress-static** - Progress indication
4. **bubbletea/composable-views** - Component composition
5. **huh/readme** - Form basics

### Phase 3: Applications (Priority 3)
Full-featured example applications.

1. **bubbletea/glamour** - Markdown viewer
2. **bubbletea/file-picker** - File browser
3. **bubbletea/chat** - Multi-component app
4. **huh/git** - Complex wizard

---

## Go Patterns Requiring Rust Adaptation

### Pattern: tea.Cmd returning nil
**Go**: `return nil` for no command
**Rust**: `Option<Cmd>` with `None`

### Pattern: tea.Batch for concurrent commands
**Go**: `tea.Batch(cmd1, cmd2)`
**Rust**: `batch(vec![Some(cmd1), Some(cmd2)])`

### Pattern: Interface embedding for models
**Go**: Embed interfaces for composition
**Rust**: Trait composition via generics or enums

### Pattern: Context for cancellation
**Go**: `context.Context` for async cancellation
**Rust**: `tokio::select!` or cancellation tokens

### Pattern: Channel-based communication
**Go**: Goroutines with channels
**Rust**: `tokio::sync::mpsc` or crossbeam channels

---

## Summary Statistics

| Category | Count | Percentage |
|----------|-------|------------|
| Basic | ~30 | 37% |
| Intermediate | ~35 | 43% |
| Advanced | ~16 | 20% |
| **Total** | **81** | 100% |

**Recommended first batch**: 6 examples (simple, spinner, textinput, timer, layout, helloworld)

---

*Generated: 2026-01-19*
*Task: charmed_rust-jb3*
