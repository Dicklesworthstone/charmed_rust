# Configuration

Glow can be configured through both the programmatic API and a configuration file.

## Programmatic Configuration

### Config Builder

```rust
use glow::Config;

let config = Config::new()
    .style("dark")      // Theme style
    .width(80)          // Wrap width
    .pager(true);       // Enable pager
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `style` | `String` | `"dark"` | Color theme |
| `width` | `Option<usize>` | `None` | Word wrap width (None = terminal width) |
| `pager` | `bool` | `true` | Enable pager mode |

## Configuration File

Glow looks for configuration in these locations (in order):

1. `$GLOW_CONFIG` environment variable
2. `$XDG_CONFIG_HOME/glow/config.yml`
3. `~/.config/glow/config.yml`
4. `~/.glow.yml`

### Example Configuration

```yaml
# ~/.config/glow/config.yml

# Style theme
style: dark

# Word wrap width (0 = terminal width)
width: 100

# Enable pager mode
pager: true

# Enable mouse support
mouse: true

# Only show local files (disable GitHub)
local_only: false

# Custom styles directory
styles_dir: ~/.config/glow/styles
```

## Styles

### Built-in Styles

| Style | Description |
|-------|-------------|
| `dark` | Dark background with bright text |
| `light` | Light background with dark text |
| `ascii` | ASCII-only characters (no Unicode) |
| `pink` | Pink accent colors |
| `auto` | Auto-detect from terminal |
| `no-tty` | Plain output (no ANSI codes) |

### Style Selection

```rust
// Programmatic
let config = Config::new().style("light");

// CLI
glow --style light README.md

// Config file
style: light
```

### Auto Style

The `auto` style detects your terminal's background color and chooses between light and dark themes automatically.

```yaml
style: auto
```

## Width Configuration

The width setting controls word wrapping:

```rust
// Fixed width
let config = Config::new().width(80);

// Terminal width (default)
let config = Config::new();  // width is None
```

CLI usage:

```bash
# Fixed width
glow --width 80 README.md

# Use terminal width
glow README.md
```

## Pager Mode

When enabled, long documents are displayed in a scrollable pager:

```rust
// Enable pager (default)
let config = Config::new().pager(true);

// Disable pager
let config = Config::new().pager(false);
```

CLI usage:

```bash
# With pager (default)
glow README.md

# Without pager
glow --no-pager README.md
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `GLOW_CONFIG` | Path to configuration file |
| `GLOW_STYLE` | Default style theme |
| `NO_COLOR` | Disable colors (forces `no-tty` style) |
| `TERM` | Used for `auto` style detection |

## Configuration Precedence

1. CLI arguments (highest priority)
2. Environment variables
3. Configuration file
4. Default values (lowest priority)

## Example Configurations

### Minimal (Dark Mode)

```yaml
style: dark
pager: true
```

### Reading E-books

```yaml
style: light
width: 60
pager: true
mouse: true
```

### CI/Scripts

```yaml
style: no-tty
pager: false
```

### Wide Terminal

```yaml
style: dark
width: 120
pager: true
```
