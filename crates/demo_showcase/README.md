# Demo Showcase

A comprehensive demonstration of charmed_rust capabilities—showcasing bubbletea, lipgloss, bubbles, glamour, huh, harmonica, and charmed_log in a single multi-page TUI application.

## Quick Start

```bash
# Run the showcase
cargo run -p demo_showcase

# Run with a specific theme
cargo run -p demo_showcase -- --theme dracula

# Run with deterministic data (same seed = same demo data)
cargo run -p demo_showcase -- --seed 42

# Run headless self-check (for CI)
cargo run -p demo_showcase -- --self-check
```

## CLI Options

| Flag | Environment Variable | Description |
|------|---------------------|-------------|
| `-t, --theme <THEME>` | `DEMO_THEME` | Theme preset: `dark`, `light`, `dracula`, `nord`, `catppuccin-*` |
| `--theme-file <PATH>` | `DEMO_THEME_FILE` | Load custom theme from TOML/JSON/YAML file |
| `-s, --seed <SEED>` | `DEMO_SEED` | Seed for deterministic data generation |
| `--no-animations` | `DEMO_NO_ANIMATIONS` | Disable animations (also respects `REDUCE_MOTION`) |
| `--no-mouse` | `DEMO_NO_MOUSE` | Disable mouse support |
| `--no-color` | `NO_COLOR` | Force ASCII mode (respects NO_COLOR spec) |
| `--force-color` | — | Force color output (overrides NO_COLOR) |
| `--no-alt-screen` | `DEMO_NO_ALT_SCREEN` | Run in main terminal buffer |
| `--self-check` | — | Run headless validation and exit |
| `--files-root <PATH>` | `DEMO_FILES_ROOT` | Root directory for file browser |
| `-v, --verbose` | — | Enable verbose logging (repeat for more) |

## Pages

The showcase includes 7 interactive pages:

| # | Page | Description | Key Features |
|---|------|-------------|--------------|
| 1 | **Dashboard** | Overview with status cards | Real-time stats, recent jobs list |
| 2 | **Services** | Placeholder for service status | — |
| 3 | **Jobs** | Background task monitoring | Filterable table, job actions (n/⏎/x/R) |
| 4 | **Logs** | Real-time log viewer | Follow mode, level filters, export |
| 5 | **Docs** | Markdown documentation | Syntax highlighting, split-view, search |
| 6 | **Wizard** | Multi-step form demo | huh integration, form validation |
| 7 | **Settings** | App configuration | Theme switching, toggle controls |

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `1-7` | Navigate to page |
| `[` | Toggle sidebar |
| `?` | Show help overlay |
| `q` | Quit |
| `t` | Cycle theme |

### Help Overlay

| Key | Action |
|-----|--------|
| `j/k` | Scroll down/up |
| `g/G` | Go to top/bottom |
| `q/?/Esc` | Close |

### Jobs Page

| Key | Action |
|-----|--------|
| `n` | Create new job |
| `⏎` | Start queued job |
| `x` | Cancel job (running/queued) |
| `R` | Retry job (failed/cancelled) |
| `/` | Filter by query |
| `1-4` | Toggle status filters |
| `s/S` | Sort / reverse sort |
| `j/k` | Navigate rows |
| `r` | Refresh data |

### Logs Page

| Key | Action |
|-----|--------|
| `y` | Copy viewport to file |
| `Y` | Copy all logs to file |
| `e` | Export full log buffer |
| `X` | Clear log buffer |
| `/` | Filter by query |
| `1-5` | Toggle level filters (E/W/I/D/T) |
| `f` | Toggle follow mode |
| `j/k` | Scroll |
| `g/G` | Go to top/bottom |

### Docs Page

| Key | Action |
|-----|--------|
| `Tab` | Toggle split view |
| `/` | Start search |
| `n/N` | Next/prev match |
| `h` | Toggle syntax highlighting |
| `l` | Toggle line numbers |
| `j/k` | Scroll |

### Settings Page

| Key | Action |
|-----|--------|
| `Tab` | Next section |
| `j/k` | Navigate options |
| `⏎/Space` | Toggle/select option |

## Testing

### Unit Tests

```bash
# Run all demo_showcase tests
cargo test -p demo_showcase

# Run specific page tests
cargo test -p demo_showcase -- jobs
cargo test -p demo_showcase -- logs
cargo test -p demo_showcase -- docs
```

### Self-Check Mode

The `--self-check` flag runs a headless validation that:
- Creates the app without a TTY
- Renders all pages
- Verifies no panics occur

```bash
cargo run -p demo_showcase -- --self-check
```

Output on success:
```
Running self-check...
✓ App creates successfully
✓ View renders (10 chars)
✓ Jobs page renders
✓ Logs page renders
✓ Docs page renders
✓ Dashboard page renders
✓ Self-check passed (4 pages validated)
```

### E2E Tests

E2E tests use the `E2ETestRunner` harness for scenario-based testing:

```bash
# Run E2E smoke tour
cargo test -p demo_showcase -- e2e_smoke_tour

# Run with verbose output
cargo test -p demo_showcase -- e2e_smoke_tour --nocapture
```

Artifacts are written to `target/demo_showcase_e2e/<scenario>/<run_id>/`:
- `summary.txt` — Test results and timeline
- `frames/` — Captured terminal frames
- `logs/` — Exported log files

### Environment Variables for Testing

| Variable | Purpose |
|----------|---------|
| `DEMO_SHOWCASE_E2E=1` | Switch to E2E artifact directory |
| `DEMO_SEED=42` | Use deterministic data generation |
| `NO_COLOR=1` | Test ASCII mode |
| `REDUCE_MOTION=1` | Test reduced motion |

## Feature Coverage Matrix

This table shows which charmed_rust features are demonstrated and tested:

| Crate | Feature | UI Location | Test Coverage |
|-------|---------|-------------|---------------|
| **bubbletea** | Elm Architecture | All pages | Unit + E2E |
| | Commands (Cmd) | Job actions, exports | Unit |
| | Tick/timer | Animations, spinners | Unit |
| | Key handling | All pages | Unit + E2E |
| | Mouse support | Table, viewport | Manual |
| | Alt screen | Default mode | E2E |
| | Batch/sequence | Job actions | Unit |
| **lipgloss** | Style builder | All rendering | Unit |
| | Borders | Boxes, modals | Visual |
| | Colors (24-bit) | Themes | Unit |
| | Adaptive colors | Light/dark themes | E2E |
| | Padding/margin | Layout | Visual |
| | Width/height | Responsive layout | E2E |
| **bubbles** | Viewport | Logs, Docs | Unit + E2E |
| | Table | Jobs | Unit + E2E |
| | TextInput | Filters, Wizard | Unit |
| | Spinner | Loading states | Visual |
| | Progress | Job progress | Unit |
| | List | File picker | Unit |
| | Paginator | Tables | Unit |
| | Help | Help overlay | E2E |
| | FilePicker | Files page | Unit |
| | Timer | Animations | Unit |
| **glamour** | Markdown render | Docs page | Unit + E2E |
| | Syntax highlighting | Code blocks | Toggle test |
| | Theme support | Docs styling | E2E |
| | Table render | Docs content | Visual |
| **huh** | Form fields | Wizard page | Unit |
| | Validation | Form submission | Unit |
| | Multi-step | Wizard flow | E2E |
| **harmonica** | Spring animation | Transitions | Unit |
| | Projectile | Effects | Unit |
| **charmed_log** | Styled logging | Log viewer | Unit |
| | Level filtering | Logs page | Unit + E2E |
| | Structured fields | Log details | Unit |

### Manual-Only Checks

Some features require manual testing:

- **Mouse drag** — Table column resize, viewport scroll
- **SSH mode** — `cargo run -p demo_showcase -- ssh` (requires setup)
- **Terminal resize** — Responsive layout reflow
- **Theme file loading** — Custom TOML/JSON/YAML themes
- **High-DPI rendering** — Visual inspection on HiDPI displays

## Architecture

```
demo_showcase/
├── src/
│   ├── app.rs          # Main App model, routing, chrome
│   ├── pages/          # Page models (Dashboard, Jobs, Logs, etc.)
│   ├── components/     # Reusable UI components
│   ├── data/           # Domain models, simulation, actions
│   ├── theme.rs        # Theme system with presets
│   ├── keymap.rs       # Keybinding definitions
│   ├── messages.rs     # Message types for routing
│   ├── config.rs       # CLI config and env mapping
│   └── test_support.rs # E2E testing harness
└── assets/
    ├── docs/           # Embedded markdown documentation
    └── fixtures/       # Test fixtures and sample data
```

## Exports

The showcase supports exporting content:

| Action | Output Location | Format |
|--------|-----------------|--------|
| Copy viewport (`y`) | `demo_showcase_exports/viewport_*.txt` | Plain text |
| Copy all logs (`Y`) | `demo_showcase_exports/logs_all_*.txt` | Plain text |
| Export logs (`e`) | `demo_showcase_exports/logs_export_*.txt` | Plain text |
| Export view (global) | `demo_<page>_<timestamp>.txt/html` | Plain or HTML |

In E2E mode (`DEMO_SHOWCASE_E2E=1`), exports go to `target/demo_showcase_e2e/logs/`.
