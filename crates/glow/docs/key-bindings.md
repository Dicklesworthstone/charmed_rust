# Key Bindings

Glow provides vim-style and standard key bindings for navigation.

## Navigation

| Key | Alternative | Action |
|-----|-------------|--------|
| `j` | `Down` | Scroll down one line |
| `k` | `Up` | Scroll up one line |
| `d` | `Page Down` | Scroll down half page |
| `u` | `Page Up` | Scroll up half page |
| `Ctrl+d` | | Scroll down full page |
| `Ctrl+u` | | Scroll up full page |
| `g` | `Home` | Go to top |
| `G` | `End` | Go to bottom |

## Search

| Key | Action |
|-----|--------|
| `/` | Start search |
| `n` | Next search result |
| `N` | Previous search result |
| `Esc` | Cancel search |

## File Browser

| Key | Action |
|-----|--------|
| `Enter` | Open selected file |
| `Backspace` | Go to parent directory |
| `Tab` | Toggle preview |
| `s` | Add to stash |
| `S` | View stash |

## General

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Esc` | Cancel/Close |
| `?` | Show help |
| `Ctrl+c` | Force quit |

## Mouse Support

When mouse mode is enabled:

| Action | Effect |
|--------|--------|
| Scroll wheel | Scroll content |
| Click | Select item (file browser) |
| Double-click | Open file |

## Customizing Key Bindings

Key bindings can be customized in the configuration file:

```yaml
# ~/.config/glow/config.yml
keybindings:
  quit: ["q", "Ctrl+c"]
  scroll_down: ["j", "Down"]
  scroll_up: ["k", "Up"]
  page_down: ["d", "Page Down", "Ctrl+d"]
  page_up: ["u", "Page Up", "Ctrl+u"]
  top: ["g", "Home"]
  bottom: ["G", "End"]
  search: ["/"]
  next_match: ["n"]
  prev_match: ["N"]
```

## Modal Key Bindings

### Normal Mode

Default mode for viewing content.

### Search Mode

Active when search is initiated with `/`:

| Key | Action |
|-----|--------|
| `Enter` | Execute search |
| `Esc` | Cancel search |
| `Ctrl+u` | Clear search input |

### File Browser Mode

Active when browsing files:

| Key | Action |
|-----|--------|
| `j/Down` | Next item |
| `k/Up` | Previous item |
| `Enter` | Open/Enter |
| `Backspace` | Parent directory |
| `h` | Go to home |
| `.` | Toggle hidden files |

## Tips

### Efficient Navigation

- Use `g` and `G` to quickly jump to top/bottom
- Use `/` search for finding specific content
- Use `n`/`N` to jump between search matches

### File Browser Tips

- Press `Tab` to toggle preview pane
- Press `s` to quickly stash files for later
- Press `.` to see hidden files (dotfiles)
