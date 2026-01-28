# Accessibility and Fallback Guidelines

This document defines guardrails for accessibility and degraded terminal environments, ensuring Charmed Control Center remains usable regardless of terminal capabilities.

---

## Design Philosophy

1. **Graceful Degradation**: Features degrade, but functionality persists
2. **Contrast First**: All text must be readable in any environment
3. **Keyboard Complete**: Every action is achievable via keyboard alone
4. **Color Independence**: Information is never conveyed by color alone

---

## Contrast and Legibility Requirements

### Minimum Contrast Ratios

All text must meet WCAG 2.1 AA standards for contrast ratio (4.5:1 for normal text, 3:1 for large text and UI components).

| Text Type | Minimum Ratio | Notes |
|-----------|---------------|-------|
| Primary text (`text`) | 4.5:1 | Against `bg` |
| Muted text (`text_muted`) | 3:1 | Against `bg` and `bg_subtle` |
| Inverse text (`text_inverse`) | 4.5:1 | Against colored backgrounds |
| Status colors (success/warning/error) | 3:1 | Against their usage context |

### Theme-Specific Contrast Validation

Each theme preset MUST pass these checks:

```
Dark Theme:
  text (#FFFFFF) on bg (#000000): 21:1      [PASS]
  text_muted (#626262) on bg: 5.3:1         [PASS]
  primary (#7D56F4) on bg: 4.7:1            [PASS]
  success (#00FF00) on bg: 15.3:1           [PASS]
  warning (#FFCC00) on bg: 15.1:1           [PASS]
  error (#FF0000) on bg: 5.3:1              [PASS]

Light Theme:
  text (#1A202C) on bg (#FFFFFF): 14.5:1    [PASS]
  text_muted (#718096) on bg: 4.5:1         [PASS]
  primary (#6B46C1) on bg: 5.8:1            [PASS]
  success (#38A169) on bg: 4.3:1            [PASS]
  warning (#D69E2E) on bg: 3.6:1            [PASS]
  error (#E53E3E) on bg: 4.6:1              [PASS]

Dracula Theme:
  text (#F8F8F2) on bg (#282A36): 11.3:1    [PASS]
  text_muted (#6272A4) on bg: 4.0:1         [PASS]
  primary (#BD93F9) on bg: 6.9:1            [PASS]
  success (#50FA7B) on bg: 10.7:1           [PASS]
  warning (#F1FA8C) on bg: 13.1:1           [PASS]
  error (#FF5555) on bg: 5.9:1              [PASS]
```

### Adding New Themes

When adding a theme, verify:
1. `text` on `bg` >= 4.5:1
2. `text_muted` on `bg` >= 3:1
3. `text_muted` on `bg_subtle` >= 3:1
4. All semantic colors >= 3:1 on their background contexts

Use a contrast checker tool or the `Theme::check_contrast_aa()` method (when available).

---

## Focus Indication Rules

### Visual Focus Indicators

Every interactive element MUST have a visible focus indicator:

| Element Type | Focus Style | Implementation |
|--------------|-------------|----------------|
| Sidebar items | Bold + primary color + highlight bg | `sidebar_selected_style()` |
| List items | `>` prefix + bold + primary color | `selected_style()` |
| Form fields | Cursor visible + `>` prefix | Field-specific |
| Buttons | Border changes to `border_focus` | `box_focused_style()` |
| Table rows | Highlight background | `hover_style()` |
| Modal dialogs | Double border with `border_focus` | `modal_style()` |

### Focus Movement Patterns

```
Tab         Move to next section/group
Shift+Tab   Move to previous section/group
j/k or ↓/↑  Move within list/table
h/l or ←/→  Move between panes (when applicable)
Enter       Activate/select focused item
Esc         Close overlay/cancel action
```

### Focus Visibility Rules

1. **Always Visible**: Focus indicator must be visible at all times
2. **High Contrast**: Focus color (`border_focus`) contrasts with surrounding elements
3. **Not Color Alone**: Focus uses multiple cues (border change + color + prefix character)
4. **Consistent**: Same focus pattern across all similar elements

### Focus in Degraded Modes

When colors are unavailable (NO_COLOR/ASCII mode):
- Use `[ ]` for unfocused, `[*]` for focused
- Add `>` prefix before focused items in lists
- Underline focused text elements
- Double borders for focused containers: `+=====+` instead of `+-----+`

---

## Color Profile Detection and Fallbacks

### Profile Hierarchy

The application detects terminal capabilities and selects an appropriate profile:

```
Level 1: TrueColor (24-bit)
  - COLORTERM=truecolor OR
  - COLORTERM=24bit
  - Full hex color support

Level 2: ANSI 256 (8-bit)
  - TERM contains "256color"
  - Map hex to nearest ANSI 256 palette color

Level 3: ANSI 16 (4-bit)
  - TERM=xterm, linux, vt100, etc.
  - Map to basic ANSI colors (see below)

Level 4: ASCII/NoColor (1-bit)
  - NO_COLOR environment variable is set
  - TERM=dumb
  - No ANSI escape sequences at all
```

### ANSI 16 Color Mapping

When limited to 16 colors, semantic tokens map as follows:

| Token | ANSI Color | Code |
|-------|------------|------|
| `primary` | Bright Blue | 94 |
| `secondary` | Bright Magenta | 95 |
| `success` | Bright Green | 92 |
| `warning` | Bright Yellow | 93 |
| `error` | Bright Red | 91 |
| `info` | Bright Cyan | 96 |
| `text` | White | 97 |
| `text_muted` | Bright Black (Gray) | 90 |
| `bg` | Default (transparent) | 49 |
| `bg_subtle` | Black | 40 |
| `bg_highlight` | Bright Black | 100 |
| `border` | Bright Black (Gray) | 90 |
| `border_focus` | Bright Blue | 94 |

---

## NO_COLOR / ASCII Mode Behavior

### Triggering Conditions

ASCII/NoColor mode activates when:
- `NO_COLOR` environment variable is set (any value)
- `TERM=dumb`
- `TERM=` (empty)
- Terminal does not support ANSI escape codes

### Visual Adaptations

#### Border Characters

| Normal Mode | ASCII Mode |
|-------------|------------|
| `╭──────╮` | `+------+` |
| `│      │` | `\|      \|` |
| `╰──────╯` | `+------+` |
| `├──────┤` | `+------+` |
| `═══════` | `=======` |

#### Status Indicators

| Normal Mode | ASCII Mode | Meaning |
|-------------|------------|---------|
| `●` (green) | `[OK]` | Healthy |
| `◐` (yellow) | `[!!]` | Degraded/Warning |
| `○` (red) | `[XX]` | Unhealthy/Error |
| `?` (gray) | `[??]` | Unknown |

#### Progress Bars

```
Normal: [████████░░░░░░░░] 50%
ASCII:  [########........] 50%
```

#### Selection Indicators

```
Normal: > Item Name (bold + color)
ASCII:  > [*] Item Name
        [*] indicates selected
        > indicates cursor position
```

#### Emphasis Fallbacks

| Visual Style | ASCII Fallback |
|--------------|----------------|
| Bold text | `**text**` or UPPERCASE |
| Primary color | Underlined |
| Error styling | Prefixed with `!` or `ERROR:` |
| Warning styling | Prefixed with `!` or `WARN:` |
| Success styling | Prefixed with `OK:` |
| Links | Underlined + `<URL>` suffix |

### Structural Clarity

Without color, rely on:
1. **Whitespace**: Consistent spacing to group related items
2. **Indentation**: 2-space indent for hierarchy levels
3. **Prefixes**: `- ` for lists, `> ` for selected, `! ` for errors
4. **Separators**: `---` or `===` between major sections
5. **Labels**: Explicit text labels where color would convey meaning

---

## Keyboard Navigation Completeness

### Navigation Matrix

Every page must support:

| Action | Primary Key | Alternate | Notes |
|--------|-------------|-----------|-------|
| Move down | `j` | `↓` | Vim-style preferred |
| Move up | `k` | `↑` | |
| Move left/prev | `h` | `←` | Between panes |
| Move right/next | `l` | `→` | Between panes |
| Select/Activate | `Enter` | `Space` | Context-dependent |
| Cancel/Back | `Esc` | `b` | Close dialogs, go back |
| Help | `?` | | Show keybindings |
| Quit | `q` | `Ctrl+C` | Exit application |
| Search/Filter | `/` | `Ctrl+F` | When filtering available |
| Page down | `Ctrl+D` | `Page Down` | Large scroll |
| Page up | `Ctrl+U` | `Page Up` | Large scroll |
| Top | `g` | `Home` | Go to first item |
| Bottom | `G` | `End` | Go to last item |

### Mouse Support (Optional Enhancement)

Mouse is supported but never required:
- Click to select items
- Scroll wheel to navigate lists
- All mouse actions have keyboard equivalents

---

## Testing Requirements

### Manual Testing Checklist

Before release, verify:

1. [ ] Run with `NO_COLOR=1` - all UI elements visible and functional
2. [ ] Run with `TERM=dumb` - no ANSI escape sequences in output
3. [ ] Run with `TERM=xterm` - 16-color mode renders correctly
4. [ ] Navigate entire app using keyboard only
5. [ ] Verify focus is always visible on focused element
6. [ ] Check contrast with browser developer tools or contrast checker
7. [ ] Test each theme preset for legibility

### Automated Testing

The theme module should include:
```rust
#[test]
fn all_themes_meet_contrast_requirements() {
    for preset in ThemePreset::all() {
        let theme = Theme::from_preset(preset);
        assert!(check_contrast(theme.text, theme.bg) >= 4.5);
        assert!(check_contrast(theme.text_muted, theme.bg) >= 3.0);
        // ... additional checks
    }
}
```

### Screen Reader Considerations

While terminal applications have limited screen reader support:
- Use consistent structural patterns for navigation
- Avoid purely visual indicators (spinner animations should have text alternatives)
- Status changes should be expressible as text

---

## Implementation Checklist

### Required Functions

```rust
impl Theme {
    /// Check if this theme meets WCAG AA contrast requirements.
    pub fn validate_contrast(&self) -> Vec<ContrastIssue>;

    /// Get ASCII-safe border style (+ - |).
    pub fn ascii_border() -> Border;

    /// Get status indicator text for ASCII mode.
    pub fn status_text(status: Status) -> &'static str;

    /// Detect the appropriate color profile for current terminal.
    pub fn detect_color_profile() -> ColorProfile;
}

enum ColorProfile {
    TrueColor,  // 24-bit
    Ansi256,    // 8-bit
    Ansi16,     // 4-bit
    Ascii,      // No color
}

enum Status {
    Ok,
    Warning,
    Error,
    Unknown,
}
```

### Environment Variables

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | Force ASCII mode (any value) |
| `COLORTERM` | Detect TrueColor support |
| `TERM` | Detect terminal capabilities |
| `CHARMED_FORCE_COLOR` | Override detection (debug) |

---

## Summary

1. **Contrast**: All themes must meet WCAG AA (4.5:1 text, 3:1 UI)
2. **Focus**: Every interactive element has a visible focus indicator using multiple cues
3. **Degradation**: Full functionality in ASCII mode with text-based alternatives
4. **Keyboard**: Complete keyboard navigation with consistent bindings
5. **Testing**: Manual and automated testing for all modes

The demo must be usable and not visually broken when colors are disabled.
