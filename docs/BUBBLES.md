# Bubbles - TUI Components

## Overview

Bubbles is a collection of pre-built, reusable TUI components for use with Bubbletea. Each component follows the Model-Update-View architecture and can be composed together to build complex terminal interfaces.

## Components

### Foundation Components (No Dependencies)

#### 1. key - Keybinding Definitions

```rust
/// A key binding with associated help text.
pub struct Binding {
    keys: Vec<String>,
    help: Help,
    enabled: bool,
}

pub struct Help {
    pub key: String,
    pub desc: String,
}

impl Binding {
    pub fn new() -> BindingBuilder;
    pub fn keys(&self) -> &[String];
    pub fn enabled(&self) -> bool;
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn help(&self) -> &Help;
    pub fn unbind(&mut self);
}

/// Check if any binding matches a key.
pub fn matches<K: AsRef<str>>(key: K, bindings: &[&Binding]) -> bool;
```

#### 2. runeutil - Input Sanitization

```rust
pub struct Sanitizer {
    replace_tabs: Option<String>,
    replace_newlines: Option<String>,
}

impl Sanitizer {
    pub fn new() -> SanitizerBuilder;
    pub fn sanitize(&self, input: &[char]) -> Vec<char>;
}

impl SanitizerBuilder {
    pub fn replace_tabs(self, replacement: &str) -> Self;
    pub fn replace_newlines(self, replacement: &str) -> Self;
    pub fn build(self) -> Sanitizer;
}
```

### Display Components

#### 3. cursor - Cursor Blinking

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Blink,
    Static,
    Hide,
}

pub struct Cursor {
    mode: Mode,
    style: lipgloss::Style,
    text_style: lipgloss::Style,
    char: String,
    blink: bool,
    id: u64,
}

pub struct BlinkMsg {
    id: u64,
}

impl Cursor {
    pub fn new() -> Self;
    pub fn focus(&mut self) -> Option<Cmd>;
    pub fn blur(&mut self);
    pub fn blink_cmd(&self) -> Option<Cmd>;
    pub fn set_mode(&mut self, mode: Mode);
    pub fn mode(&self) -> Mode;
    pub fn set_char(&mut self, c: &str);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

#### 4. spinner - Loading Animations

```rust
pub struct Spinner {
    pub frames: Vec<String>,
    pub fps: u32,
}

pub mod spinners {
    pub const LINE: Spinner;
    pub const DOT: Spinner;
    pub const MINI_DOT: Spinner;
    pub const JUMP: Spinner;
    pub const PULSE: Spinner;
    pub const POINTS: Spinner;
    pub const GLOBE: Spinner;
    pub const MOON: Spinner;
    pub const MONKEY: Spinner;
    pub const METER: Spinner;
    pub const HAMBURGER: Spinner;
    pub const ELLIPSIS: Spinner;
}

pub struct SpinnerModel {
    spinner: Spinner,
    frame: usize,
    id: u64,
    style: lipgloss::Style,
}

pub struct TickMsg {
    id: u64,
}

impl SpinnerModel {
    pub fn new() -> Self;
    pub fn with_spinner(spinner: Spinner) -> Self;
    pub fn tick(&self) -> Cmd;
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

#### 5. progress - Progress Bar with Animation

```rust
pub struct Progress {
    percent: f64,
    target_percent: f64,
    width: u16,
    full_char: char,
    empty_char: char,
    show_percentage: bool,
    spring: harmonica::Spring,
    // Gradient colors (optional)
    gradient_start: Option<String>,
    gradient_end: Option<String>,
    solid_fill: Option<String>,
    empty_color: Option<String>,
}

pub struct FrameMsg;

impl Progress {
    pub fn new() -> ProgressBuilder;
    pub fn set_percent(&mut self, percent: f64) -> Option<Cmd>;
    pub fn incr_percent(&mut self, delta: f64) -> Option<Cmd>;
    pub fn decr_percent(&mut self, delta: f64) -> Option<Cmd>;
    pub fn percent(&self) -> f64;
    pub fn is_animating(&self) -> bool;
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
    pub fn view_as(&self, percent: f64) -> String;
}

impl ProgressBuilder {
    pub fn width(self, w: u16) -> Self;
    pub fn with_gradient(self, start: &str, end: &str) -> Self;
    pub fn with_solid_fill(self, color: &str) -> Self;
    pub fn without_percentage(self) -> Self;
    pub fn with_spring_options(self, freq: f64, damping: f64) -> Self;
    pub fn build(self) -> Progress;
}
```

#### 6. paginator - Pagination Control

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Arabic,  // "1/5"
    Dots,    // "●○○○○"
}

pub struct Paginator {
    page: usize,
    per_page: usize,
    total_pages: usize,
    ptype: Type,
    active_dot: String,
    inactive_dot: String,
    arabic_format: String,
    keymap: KeyMap,
}

pub struct KeyMap {
    pub prev_page: Binding,
    pub next_page: Binding,
}

impl Paginator {
    pub fn new() -> Self;
    pub fn set_total_pages(&mut self, items: usize);
    pub fn items_on_page(&self, total_items: usize) -> usize;
    pub fn get_slice_bounds(&self, length: usize) -> (usize, usize);
    pub fn page(&self) -> usize;
    pub fn total_pages(&self) -> usize;
    pub fn on_first_page(&self) -> bool;
    pub fn on_last_page(&self) -> bool;
    pub fn prev_page(&mut self);
    pub fn next_page(&mut self);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### Time Components

#### 7. timer - Countdown Timer

```rust
pub struct Timer {
    timeout: Duration,
    remaining: Duration,
    interval: Duration,
    running: bool,
    id: u64,
}

pub struct TickMsg {
    id: u64,
    timeout: bool,
}

pub struct TimeoutMsg;

pub struct StartStopMsg {
    id: u64,
    running: bool,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self;
    pub fn with_interval(timeout: Duration, interval: Duration) -> Self;
    pub fn init(&self) -> Option<Cmd>;
    pub fn start(&mut self) -> Option<Cmd>;
    pub fn stop(&mut self) -> Option<Cmd>;
    pub fn toggle(&mut self) -> Option<Cmd>;
    pub fn reset(&mut self);
    pub fn running(&self) -> bool;
    pub fn timed_out(&self) -> bool;
    pub fn remaining(&self) -> Duration;
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

#### 8. stopwatch - Elapsed Time Tracking

```rust
pub struct Stopwatch {
    elapsed: Duration,
    interval: Duration,
    running: bool,
    id: u64,
}

pub struct TickMsg {
    id: u64,
}

pub struct StartStopMsg {
    id: u64,
    running: bool,
}

pub struct ResetMsg {
    id: u64,
}

impl Stopwatch {
    pub fn new() -> Self;
    pub fn with_interval(interval: Duration) -> Self;
    pub fn init(&self) -> Option<Cmd>;
    pub fn start(&mut self) -> Option<Cmd>;
    pub fn stop(&mut self) -> Option<Cmd>;
    pub fn toggle(&mut self) -> Option<Cmd>;
    pub fn reset(&mut self) -> Option<Cmd>;
    pub fn running(&self) -> bool;
    pub fn elapsed(&self) -> Duration;
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### Layout Components

#### 9. viewport - Scrollable Content

```rust
pub struct Viewport {
    width: u16,
    height: u16,
    y_offset: usize,
    x_offset: usize,
    lines: Vec<String>,
    keymap: KeyMap,
    mouse_wheel_enabled: bool,
    mouse_wheel_delta: usize,
    style: lipgloss::Style,
}

pub struct KeyMap {
    pub page_down: Binding,
    pub page_up: Binding,
    pub half_page_down: Binding,
    pub half_page_up: Binding,
    pub down: Binding,
    pub up: Binding,
    pub left: Binding,
    pub right: Binding,
    pub goto_top: Binding,
    pub goto_bottom: Binding,
}

impl Viewport {
    pub fn new(width: u16, height: u16) -> Self;
    pub fn set_content(&mut self, content: &str);
    pub fn content(&self) -> String;
    pub fn set_y_offset(&mut self, offset: usize);
    pub fn y_offset(&self) -> usize;
    pub fn scroll_percent(&self) -> f64;
    pub fn at_top(&self) -> bool;
    pub fn at_bottom(&self) -> bool;
    pub fn past_bottom(&self) -> bool;
    pub fn scroll_up(&mut self, lines: usize);
    pub fn scroll_down(&mut self, lines: usize);
    pub fn half_page_up(&mut self);
    pub fn half_page_down(&mut self);
    pub fn page_up(&mut self);
    pub fn page_down(&mut self);
    pub fn goto_top(&mut self);
    pub fn goto_bottom(&mut self);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### Help Components

#### 10. help - Help Text Display

```rust
pub trait KeyMap {
    fn short_help(&self) -> Vec<Binding>;
    fn full_help(&self) -> Vec<Vec<Binding>>;
}

pub struct Help {
    width: u16,
    show_all: bool,
    short_separator: String,
    full_separator: String,
    ellipsis: String,
    styles: Styles,
}

pub struct Styles {
    pub short_key: lipgloss::Style,
    pub short_desc: lipgloss::Style,
    pub short_separator: lipgloss::Style,
    pub full_key: lipgloss::Style,
    pub full_desc: lipgloss::Style,
    pub full_separator: lipgloss::Style,
    pub ellipsis: lipgloss::Style,
}

impl Help {
    pub fn new() -> Self;
    pub fn with_width(width: u16) -> Self;
    pub fn set_show_all(&mut self, show: bool);
    pub fn short_help_view(&self, bindings: &[Binding]) -> String;
    pub fn full_help_view(&self, groups: &[Vec<Binding>]) -> String;
    pub fn view<K: KeyMap>(&self, keymap: &K) -> String;
}
```

### Input Components

#### 11. textinput - Single-Line Text Input

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EchoMode {
    Normal,
    Password,
    None,
}

pub struct TextInput {
    value: Vec<char>,
    cursor_pos: usize,
    offset: usize,
    width: u16,
    placeholder: String,
    echo_mode: EchoMode,
    echo_char: char,
    cursor: Cursor,
    char_limit: Option<usize>,
    suggestions: Vec<String>,
    current_suggestion: Option<usize>,
    show_suggestions: bool,
    validate: Option<ValidateFn>,
    err: Option<String>,
    keymap: KeyMap,
    style: Styles,
    sanitizer: Sanitizer,
}

pub type ValidateFn = Box<dyn Fn(&str) -> Result<(), String>>;

pub struct KeyMap {
    pub char_forward: Binding,
    pub char_backward: Binding,
    pub word_forward: Binding,
    pub word_backward: Binding,
    pub delete_char_backward: Binding,
    pub delete_char_forward: Binding,
    pub delete_word_backward: Binding,
    pub delete_word_forward: Binding,
    pub delete_before_cursor: Binding,
    pub delete_after_cursor: Binding,
    pub line_start: Binding,
    pub line_end: Binding,
    pub paste: Binding,
    pub accept_suggestion: Binding,
    pub next_suggestion: Binding,
    pub prev_suggestion: Binding,
}

impl TextInput {
    pub fn new() -> Self;
    pub fn set_value(&mut self, value: &str);
    pub fn value(&self) -> String;
    pub fn cursor(&self) -> usize;
    pub fn set_cursor(&mut self, pos: usize);
    pub fn cursor_start(&mut self);
    pub fn cursor_end(&mut self);
    pub fn focus(&mut self) -> Option<Cmd>;
    pub fn blur(&mut self);
    pub fn focused(&self) -> bool;
    pub fn reset(&mut self);
    pub fn set_suggestions(&mut self, suggestions: Vec<String>);
    pub fn available_suggestions(&self) -> Vec<&str>;
    pub fn current_suggestion(&self) -> Option<&str>;
    pub fn err(&self) -> Option<&str>;
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

#### 12. textarea - Multi-Line Text Editor

```rust
pub struct TextArea {
    value: Vec<Vec<char>>,  // Lines of characters
    row: usize,
    col: usize,
    width: u16,
    height: u16,
    max_height: u16,
    max_width: u16,
    viewport: Viewport,
    cursor: Cursor,
    show_line_numbers: bool,
    line_number_style: lipgloss::Style,
    end_of_buffer_char: char,
    placeholder: String,
    char_limit: Option<usize>,
    keymap: KeyMap,
    style: Styles,
    sanitizer: Sanitizer,
    // Wrap cache
    wrap_cache: WrapCache,
}

pub struct KeyMap {
    pub char_forward: Binding,
    pub char_backward: Binding,
    pub word_forward: Binding,
    pub word_backward: Binding,
    pub line_next: Binding,
    pub line_previous: Binding,
    pub delete_char_backward: Binding,
    pub delete_char_forward: Binding,
    pub delete_word_backward: Binding,
    pub delete_word_forward: Binding,
    pub delete_before_cursor: Binding,
    pub delete_after_cursor: Binding,
    pub insert_newline: Binding,
    pub line_start: Binding,
    pub line_end: Binding,
    pub input_begin: Binding,
    pub input_end: Binding,
    pub paste: Binding,
    pub uppercase_word: Binding,
    pub lowercase_word: Binding,
    pub capitalize_word: Binding,
    pub transpose_char_backward: Binding,
}

pub struct LineInfo {
    pub width: u16,
    pub char_width: u16,
    pub height: u16,
    pub start_col: usize,
    pub col_offset: usize,
    pub row_offset: usize,
    pub char_offset: usize,
}

impl TextArea {
    pub fn new() -> Self;
    pub fn set_value(&mut self, value: &str);
    pub fn insert_string(&mut self, s: &str);
    pub fn insert_rune(&mut self, r: char);
    pub fn value(&self) -> String;
    pub fn length(&self) -> usize;
    pub fn line(&self, n: usize) -> Option<&[char]>;
    pub fn line_count(&self) -> usize;
    pub fn cursor_down(&mut self);
    pub fn cursor_up(&mut self);
    pub fn focus(&mut self) -> Option<Cmd>;
    pub fn blur(&mut self);
    pub fn focused(&self) -> bool;
    pub fn reset(&mut self);
    pub fn line_info(&self) -> LineInfo;
    pub fn set_width(&mut self, width: u16);
    pub fn set_height(&mut self, height: u16);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### Table Component

#### 13. table - Data Table with Keyboard Navigation

```rust
pub struct Column {
    pub title: String,
    pub width: u16,
}

pub type Row = Vec<String>;

pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Row>,
    cursor: usize,
    focused: bool,
    viewport_start: usize,
    height: u16,
    width: u16,
    keymap: KeyMap,
    styles: Styles,
}

pub struct KeyMap {
    pub line_up: Binding,
    pub line_down: Binding,
    pub page_up: Binding,
    pub page_down: Binding,
    pub goto_top: Binding,
    pub goto_bottom: Binding,
}

pub struct Styles {
    pub header: lipgloss::Style,
    pub cell: lipgloss::Style,
    pub selected: lipgloss::Style,
}

impl Table {
    pub fn new() -> TableBuilder;
    pub fn set_columns(&mut self, columns: Vec<Column>);
    pub fn set_rows(&mut self, rows: Vec<Row>);
    pub fn columns(&self) -> &[Column];
    pub fn rows(&self) -> &[Row];
    pub fn selected_row(&self) -> Option<&Row>;
    pub fn cursor(&self) -> usize;
    pub fn set_cursor(&mut self, cursor: usize);
    pub fn move_up(&mut self, n: usize);
    pub fn move_down(&mut self, n: usize);
    pub fn goto_top(&mut self);
    pub fn goto_bottom(&mut self);
    pub fn focus(&mut self);
    pub fn blur(&mut self);
    pub fn focused(&self) -> bool;
    pub fn set_height(&mut self, height: u16);
    pub fn set_width(&mut self, width: u16);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### List Component

#### 14. list - Feature-Rich List Selection

```rust
pub trait Item: Send + 'static {
    fn filter_value(&self) -> String;
}

pub trait ItemDelegate<I: Item>: Send + 'static {
    fn height(&self) -> u16;
    fn spacing(&self) -> u16;
    fn update(&mut self, msg: Message, item: &mut I) -> Option<Cmd>;
    fn render(&self, item: &I, index: usize, selected: bool, width: u16) -> String;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterState {
    Unfiltered,
    Filtering,
    FilterApplied,
}

pub struct Rank {
    pub index: usize,
    pub match_indices: Vec<usize>,
}

pub type FilterFn<I> = Box<dyn Fn(&str, &[I]) -> Vec<Rank> + Send + 'static>;

pub struct List<I: Item, D: ItemDelegate<I>> {
    items: Vec<I>,
    filtered_items: Vec<usize>,
    delegate: D,
    cursor: usize,
    width: u16,
    height: u16,

    // Filter state
    filter_state: FilterState,
    filter_input: TextInput,
    filter_fn: FilterFn<I>,

    // Components
    paginator: Paginator,
    spinner: SpinnerModel,
    help: Help,

    // Display
    title: String,
    show_title: bool,
    show_filter: bool,
    show_status_bar: bool,
    show_pagination: bool,
    show_help: bool,
    infinite_scroll: bool,

    // Status
    status_message: Option<String>,
    status_message_lifetime: Duration,

    keymap: KeyMap,
    styles: Styles,
}

pub struct KeyMap {
    // Cursor movement
    pub cursor_up: Binding,
    pub cursor_down: Binding,
    pub next_page: Binding,
    pub prev_page: Binding,
    pub goto_start: Binding,
    pub goto_end: Binding,

    // Filtering
    pub filter: Binding,
    pub clear_filter: Binding,
    pub cancel_while_filtering: Binding,
    pub accept_while_filtering: Binding,

    // Other
    pub show_full_help: Binding,
    pub close_full_help: Binding,
    pub quit: Binding,
    pub force_quit: Binding,
}

impl<I: Item, D: ItemDelegate<I>> List<I, D> {
    pub fn new(items: Vec<I>, delegate: D, width: u16, height: u16) -> Self;

    // Items
    pub fn set_items(&mut self, items: Vec<I>);
    pub fn items(&self) -> &[I];
    pub fn visible_items(&self) -> Vec<&I>;
    pub fn index(&self) -> usize;
    pub fn selected_item(&self) -> Option<&I>;
    pub fn select(&mut self, index: usize);

    // Navigation
    pub fn cursor_up(&mut self);
    pub fn cursor_down(&mut self);
    pub fn next_page(&mut self);
    pub fn prev_page(&mut self);

    // Filtering
    pub fn filter_state(&self) -> FilterState;
    pub fn filter_value(&self) -> &str;
    pub fn set_filter_value(&mut self, value: &str);
    pub fn reset_filter(&mut self);
    pub fn set_filter_fn(&mut self, f: FilterFn<I>);

    // Spinner
    pub fn start_spinner(&mut self) -> Option<Cmd>;
    pub fn stop_spinner(&mut self);
    pub fn spinner_visible(&self) -> bool;

    // Status
    pub fn new_status_message(&mut self, msg: &str) -> Option<Cmd>;
    pub fn status_message(&self) -> Option<&str>;

    // Dimensions
    pub fn set_width(&mut self, width: u16);
    pub fn set_height(&mut self, height: u16);
    pub fn width(&self) -> u16;
    pub fn height(&self) -> u16;

    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

### File System Component

#### 15. filepicker - File Selection

```rust
pub struct FilePicker {
    current_dir: PathBuf,
    files: Vec<DirEntry>,
    cursor: usize,
    selected_file: Option<PathBuf>,
    height: u16,
    show_hidden: bool,
    dir_allowed: bool,
    file_allowed: bool,
    allowed_types: Vec<String>,
    auto_height: bool,
    keymap: KeyMap,
    styles: Styles,
    // Navigation history
    stack: Vec<PathBuf>,
}

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mode: String,
}

pub struct KeyMap {
    pub goto_top: Binding,
    pub goto_last: Binding,
    pub down: Binding,
    pub up: Binding,
    pub page_up: Binding,
    pub page_down: Binding,
    pub back: Binding,
    pub open: Binding,
    pub select: Binding,
}

pub struct Styles {
    pub cursor: lipgloss::Style,
    pub symlink: lipgloss::Style,
    pub directory: lipgloss::Style,
    pub file: lipgloss::Style,
    pub permission: lipgloss::Style,
    pub selected: lipgloss::Style,
    pub disabled_cursor: lipgloss::Style,
    pub disabled_selected: lipgloss::Style,
    pub file_size: lipgloss::Style,
    pub empty_directory: lipgloss::Style,
}

pub struct ReadDirMsg {
    pub path: PathBuf,
    pub entries: Result<Vec<DirEntry>, io::Error>,
}

impl FilePicker {
    pub fn new() -> Self;
    pub fn init(&self) -> Option<Cmd>;
    pub fn path(&self) -> &Path;
    pub fn current_directory(&self) -> &Path;
    pub fn selected_file(&self) -> Option<&Path>;
    pub fn did_select_file(&self, msg: &Message) -> Option<&Path>;
    pub fn did_select_disabled_file(&self, msg: &Message) -> Option<&Path>;
    pub fn set_height(&mut self, height: u16);
    pub fn set_show_hidden(&mut self, show: bool);
    pub fn set_allowed_types(&mut self, types: Vec<String>);
    pub fn update(&mut self, msg: Message) -> Option<Cmd>;
    pub fn view(&self) -> String;
}
```

## Module Structure

```
crates/bubbles/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── key.rs         # Keybinding utilities
    ├── runeutil.rs    # Input sanitization
    ├── cursor.rs      # Cursor blinking
    ├── spinner.rs     # Loading animations
    ├── progress.rs    # Progress bar
    ├── paginator.rs   # Pagination control
    ├── timer.rs       # Countdown timer
    ├── stopwatch.rs   # Elapsed time
    ├── viewport.rs    # Scrollable content
    ├── help.rs        # Help display
    ├── textinput.rs   # Single-line input
    ├── textarea.rs    # Multi-line editor
    ├── table.rs       # Data table
    ├── list.rs        # List selection
    └── filepicker.rs  # File browser
```

## Dependencies

```toml
[dependencies]
bubbletea = { path = "../bubbletea" }
lipgloss = { path = "../lipgloss" }
harmonica = { path = "../harmonica" }
unicode-segmentation = "1.10"
unicode-width = "0.1"
parking_lot = "0.12"

[dev-dependencies]
```

## Implementation Notes

### ID Tagging Pattern

Components like spinner, timer, and cursor use ID tagging to route messages:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

impl SpinnerModel {
    pub fn new() -> Self {
        Self {
            id: next_id(),
            // ...
        }
    }

    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if tick.id != self.id {
                return None; // Message not for us
            }
            // Handle tick...
        }
        None
    }
}
```

### Viewport Rendering

Efficient visible window calculation:

```rust
impl Viewport {
    pub fn view(&self) -> String {
        let start = self.y_offset;
        let end = (start + self.height as usize).min(self.lines.len());

        self.lines[start..end]
            .iter()
            .map(|line| {
                // Handle horizontal offset and width truncation
                let visible = &line[self.x_offset..];
                truncate(visible, self.width as usize)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

### Command-Based State Transitions

Timer/stopwatch use commands for state changes:

```rust
impl Timer {
    pub fn start(&mut self) -> Option<Cmd> {
        if self.running {
            return None;
        }
        let id = self.id;
        Some(Cmd::new(move || Message::new(StartStopMsg { id, running: true })))
    }

    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(ssm) = msg.downcast_ref::<StartStopMsg>() {
            if ssm.id == self.id {
                self.running = ssm.running;
                if self.running {
                    return self.tick_cmd();
                }
            }
        }
        // ...
        None
    }
}
```

### Memoization for Expensive Operations

TextArea uses wrap cache:

```rust
struct WrapCache {
    width: u16,
    cache: HashMap<usize, Vec<String>>,  // line index -> wrapped lines
}

impl WrapCache {
    fn get_or_compute(&mut self, line_idx: usize, line: &[char], width: u16) -> &[String] {
        if self.width != width {
            self.cache.clear();
            self.width = width;
        }

        self.cache.entry(line_idx).or_insert_with(|| {
            wrap_line(line, width)
        })
    }

    fn invalidate_line(&mut self, line_idx: usize) {
        self.cache.remove(&line_idx);
    }

    fn invalidate_from(&mut self, line_idx: usize) {
        self.cache.retain(|&k, _| k < line_idx);
    }
}
```

## Port Order

Based on dependencies, implement in this order:

1. **Phase 1 - Foundation**
   - key (no dependencies)
   - runeutil (no dependencies)

2. **Phase 2 - Simple Components**
   - cursor (lipgloss)
   - spinner (lipgloss)
   - paginator (key)
   - timer (none)
   - stopwatch (none)

3. **Phase 3 - Display Components**
   - progress (harmonica, lipgloss)
   - viewport (key, lipgloss)
   - help (key, lipgloss)

4. **Phase 4 - Input Components**
   - textinput (cursor, key, runeutil)
   - textarea (cursor, key, runeutil, viewport)

5. **Phase 5 - Complex Components**
   - table (viewport, help, key)
   - list (spinner, paginator, textinput, help, key)
   - filepicker (key)

## Testing Strategy

Each component should have:

1. **Unit tests** for core functionality
2. **Integration tests** with bubbletea message flow
3. **Property tests** for edge cases (empty input, overflow, etc.)

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_tick() {
        let mut spinner = SpinnerModel::new();
        let frame0 = spinner.view();

        // Simulate tick
        let msg = Message::new(TickMsg { id: spinner.id });
        spinner.update(msg);
        let frame1 = spinner.view();

        assert_ne!(frame0, frame1);
    }

    #[test]
    fn test_spinner_ignores_other_ids() {
        let mut spinner = SpinnerModel::new();
        let frame0 = spinner.view();

        // Tick with wrong ID
        let msg = Message::new(TickMsg { id: 9999 });
        spinner.update(msg);
        let frame1 = spinner.view();

        assert_eq!(frame0, frame1);
    }
}
```

## Model Trait Usage Guides

These guides focus on wiring each component into a Bubble Tea update loop. Every section lists the messages the component understands, the commands it returns, and a minimal working example.

### cursor

#### Overview
Blinking cursor primitive used by text inputs. It reacts to focus/blur and blink messages.

#### Quick Start
```rust
use bubbles::cursor::Cursor;

let mut cursor = Cursor::new();
cursor.set_char("_");
cursor.focus();
```

#### Messages
- `InitialBlinkMsg`
- `BlinkMsg { id, tag }`
- `BlinkCanceledMsg`
- `bubbletea::FocusMsg` / `bubbletea::BlurMsg`

#### Commands
- `blink_cmd()` to kick off blinking
- `Cursor::focus()` / `Cursor::set_mode(Mode::Blink)` return a blink command
- `Cursor::update(...)` returns the next blink command when a `BlinkMsg` is handled

#### Configuration
- `blink_speed`, `set_mode(Mode)`, `set_char(...)`
- `focus()` / `blur()` to enable or disable blinking

#### Styling
- `style` for the cursor block
- `text_style` for the hidden/blinked-off state

#### Full Example
```rust
use bubbletea::{Cmd, Message, Model, Program};
use bubbles::cursor::Cursor;

struct App {
    cursor: Cursor,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.cursor.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.cursor.update(msg)
    }

    fn view(&self) -> String {
        self.cursor.view()
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let mut cursor = Cursor::new();
    cursor.set_char("_");
    cursor.focus();

    let app = App { cursor };
    Program::new(app).run().map(|_| ())
}
```

### spinner

#### Overview
Animated loading indicator with multiple preset styles.

#### Quick Start
```rust
use bubbles::spinner::{SpinnerModel, spinners};

let spinner = SpinnerModel::with_spinner(spinners::dot());
```

#### Messages
- `TickMsg { id, tag }`

#### Commands
- `SpinnerModel::init()` returns the first tick command
- `SpinnerModel::update(...)` returns the next tick command

#### Configuration
- `SpinnerModel::with_spinner(spinner)`
- `SpinnerModel::spinner(...)` to swap styles
- `style` for lipgloss styling

#### Styling
- `style` applies to the rendered frame

#### Full Example
```rust
use bubbletea::Program;
use bubbles::spinner::{SpinnerModel, spinners};

fn main() -> Result<(), bubbletea::Error> {
    let spinner = SpinnerModel::with_spinner(spinners::dot());
    Program::new(spinner).run().map(|_| ())
}
```

### progress

#### Overview
Animated progress bar with optional gradient fills.

#### Quick Start
```rust
use bubbles::progress::Progress;

let mut progress = Progress::new().width(40);
let _ = progress.set_percent(0.75);
```

#### Messages
- `FrameMsg { id, tag }`

#### Commands
- `set_percent` / `incr_percent` / `decr_percent` return a frame command
- `Progress::update(...)` returns the next frame command while animating

#### Configuration
- `width(...)`, `fill_chars(...)`, `solid_fill(...)`
- `with_gradient()` or `with_gradient_colors(...)`
- `with_scaled_gradient(...)`
- `without_percentage()`
- `set_spring_options(frequency, damping)`

#### Styling
- `percentage_style` for the percentage text
- `full_color` / `empty_color` (solid) or gradient colors

#### Full Example
```rust
use bubbletea::{Cmd, Message, Model, Program};
use bubbles::progress::Progress;

struct StartMsg;

struct App {
    progress: Progress,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        Some(Cmd::new(|| Message::new(StartMsg)))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if msg.is::<StartMsg>() {
            return self.progress.set_percent(0.75);
        }
        self.progress.update(msg)
    }

    fn view(&self) -> String {
        self.progress.view()
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let progress = Progress::new().width(40);
    let app = App { progress };
    Program::new(app).run().map(|_| ())
}
```

### paginator

#### Overview
Pagination control for lists/tables with arabic or dot display.

#### Quick Start
```rust
use bubbles::paginator::{Paginator, Type};

let paginator = Paginator::new()
    .display_type(Type::Dots)
    .per_page(10)
    .total_pages(5);
```

#### Messages
- `KeyMsg` (uses `key_map.next_page` / `key_map.prev_page`)

#### Commands
- None (pure state updates)

#### Configuration
- `display_type(Type)`
- `per_page(...)`, `total_pages(...)`
- `active_dot`, `inactive_dot`, `arabic_format`
- `key_map` for navigation

#### Styling
- Rendered dots/format strings; use lipgloss in your view if needed

#### Full Example
```rust
use bubbletea::Program;
use bubbles::paginator::{Paginator, Type};

fn main() -> Result<(), bubbletea::Error> {
    let paginator = Paginator::new()
        .display_type(Type::Dots)
        .per_page(10)
        .total_pages(5);

    Program::new(paginator).run().map(|_| ())
}
```

### timer

#### Overview
Countdown timer that ticks down and emits a timeout message.

#### Quick Start
```rust
use bubbles::timer::Timer;
use std::time::Duration;

let timer = Timer::new(Duration::from_secs(60));
```

#### Messages
- `StartStopMsg { id, running }`
- `TickMsg { id, timeout, tag }`
- `TimeoutMsg { id }`

#### Commands
- `init()` returns the first tick command
- `start()` / `stop()` / `toggle()` return a StartStop command
- `update(...)` returns the next tick command, and batches a `TimeoutMsg` when time runs out

#### Configuration
- `Timer::new(timeout)`
- `Timer::with_interval(timeout, interval)`

#### Styling
- None (render with your own lipgloss styles)

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::timer::{Timer, TimeoutMsg};
use std::time::Duration;

struct App {
    timer: Timer,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.timer.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if msg.is::<TimeoutMsg>() {
            return Some(quit());
        }
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
        }
        self.timer.update(msg)
    }

    fn view(&self) -> String {
        format!("Remaining: {:?}\nPress q to quit", self.timer.remaining())
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let timer = Timer::new(Duration::from_secs(10));
    let app = App { timer };
    Program::new(app).run().map(|_| ())
}
```

### stopwatch

#### Overview
Counts up from zero at a configured interval.

#### Quick Start
```rust
use bubbles::stopwatch::Stopwatch;

let stopwatch = Stopwatch::new();
```

#### Messages
- `StartStopMsg { id, running }`
- `TickMsg { id, tag }`
- `ResetMsg { id }`

#### Commands
- `init()` returns a start/tick sequence
- `start()` / `stop()` / `toggle()` / `reset()` return commands
- `update(...)` returns the next tick command while running

#### Configuration
- `Stopwatch::new()`
- `Stopwatch::with_interval(interval)`

#### Styling
- None (render with your own lipgloss styles)

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::stopwatch::Stopwatch;

struct App {
    sw: Stopwatch,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.sw.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
            if key.key_type == KeyType::Runes && key.runes == vec![' '] {
                return self.sw.toggle();
            }
            if key.key_type == KeyType::Runes && key.runes == vec!['r'] {
                return self.sw.reset();
            }
        }
        self.sw.update(msg)
    }

    fn view(&self) -> String {
        format!(
            "Elapsed: {:?}\nspace start/stop, r reset, q quit",
            self.sw.elapsed()
        )
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let sw = Stopwatch::new();
    let app = App { sw };
    Program::new(app).run().map(|_| ())
}
```

### viewport

#### Overview
Scrollable window over long content, with keyboard and mouse-wheel support.

#### Quick Start
```rust
use bubbles::viewport::Viewport;

let mut viewport = Viewport::new(40, 10);
viewport.set_content("Line 1\nLine 2\nLine 3");
```

#### Messages
- `KeyMsg` (page up/down, arrows, etc.)
- `MouseMsg` (wheel scrolling, if enabled)

#### Commands
- None (pure state updates)

#### Configuration
- `Viewport::new(width, height)`
- `set_content(...)`, `set_y_offset(...)`, `set_x_offset(...)`
- `key_map`, `mouse_wheel_enabled`, `mouse_wheel_delta`, `style`

#### Styling
- `style` for the rendered window

#### Full Example
```rust
use bubbletea::{Cmd, Message, Model, Program};
use bubbles::viewport::Viewport;

struct App {
    viewport: Viewport,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.viewport.update(&msg);
        None
    }

    fn view(&self) -> String {
        self.viewport.view()
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let content = (0..100).map(|i| format!("Line {}\n", i)).collect::<String>();
    let mut viewport = Viewport::new(40, 10);
    viewport.set_content(&content);

    let app = App { viewport };
    Program::new(app).run().map(|_| ())
}
```

### help

#### Overview
Renders short or full help text for key bindings.

#### Quick Start
```rust
use bubbles::help::Help;
use bubbles::key::Binding;

let help = Help::new();
let quit = Binding::new().keys(&["q"]).help("q", "quit");
let view = help.short_help_view(&[&quit]);
```

#### Messages
- `ToggleFullHelpMsg`
- `SetWidthMsg(usize)`
- `SetBindingsMsg(Vec<Binding>)`

#### Commands
- None

#### Configuration
- `width(...)`, `show_all(...)`
- `with_bindings(...)`, `set_bindings(...)`
- separators and `ellipsis` strings

#### Styling
- `Styles` fields (`short_key`, `short_desc`, `full_key`, `full_desc`, etc.)

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::help::{Help, ToggleFullHelpMsg};
use bubbles::key::Binding;

struct App {
    help: Help,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
            if key.key_type == KeyType::Runes && key.runes == vec!['?'] {
                let _ = self.help.update(Message::new(ToggleFullHelpMsg));
            }
        }
        None
    }

    fn view(&self) -> String {
        let bindings: Vec<&Binding> = self.help.bindings().iter().collect();
        self.help.view(&bindings)
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let bindings = vec![
        Binding::new().keys(&["q"]).help("q", "quit"),
        Binding::new().keys(&["?"]).help("?", "toggle help"),
    ];

    let help = Help::new().with_bindings(bindings);
    let app = App { help };
    Program::new(app).run().map(|_| ())
}
```

### textinput

#### Overview
Single-line text input with cursor, validation, and suggestions.

#### Quick Start
```rust
use bubbles::textinput::TextInput;

let mut input = TextInput::new();
input.focus();
input.placeholder = "Search...".to_string();
```

#### Messages
- `KeyMsg`
- `PasteMsg(String)` / `PasteErrMsg(String)`
- Cursor blink messages from `cursor` (`InitialBlinkMsg`, `BlinkMsg`, `BlinkCanceledMsg`)

#### Commands
- `focus()` returns a cursor blink command
- `update(...)` may return cursor blink commands

#### Configuration
- `prompt`, `placeholder`, `echo_mode`, `echo_character`
- `char_limit`, `width`, `show_suggestions`
- `key_map`, `set_value(...)`, `set_suggestions(...)`, `set_validate(...)`

#### Styling
- `prompt_style`, `text_style`, `placeholder_style`, `completion_style`
- `cursor` styles (via `cursor.style` / `cursor.text_style`)

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::textinput::TextInput;

struct App {
    input: TextInput,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.input.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Enter {
                return Some(quit());
            }
        }
        self.input.update(msg)
    }

    fn view(&self) -> String {
        format!("{}\n(enter to quit)", self.input.view())
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let mut input = TextInput::new();
    input.prompt = "> ".to_string();
    input.focus();

    let app = App { input };
    Program::new(app).run().map(|_| ())
}
```

### textarea

#### Overview
Multi-line text editor with cursor, scrolling, and line numbers.

#### Quick Start
```rust
use bubbles::textarea::TextArea;

let mut textarea = TextArea::new();
textarea.focus();
textarea.set_height(10);
```

#### Messages
- `KeyMsg`
- `PasteMsg(String)` / `PasteErrMsg(String)`
- Cursor blink messages from `cursor`
- Viewport scroll messages (`KeyMsg`, `MouseMsg`) via embedded viewport

#### Commands
- `focus()` returns a cursor blink command
- `update(...)` may return cursor blink commands

#### Configuration
- `prompt`, `placeholder`, `show_line_numbers`
- `max_height`, `max_width`, `char_limit`
- `key_map`, `set_value(...)`

#### Styling
- `focused_style` / `blurred_style` (prompt, text, cursor, line numbers)
- `cursor` styles

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::textarea::TextArea;

struct App {
    textarea: TextArea,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.textarea.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
        }
        self.textarea.update(msg)
    }

    fn view(&self) -> String {
        format!("{}\n(q to quit)", self.textarea.view())
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let mut textarea = TextArea::new();
    textarea.focus();

    let app = App { textarea };
    Program::new(app).run().map(|_| ())
}
```

### table

#### Overview
Keyboard-navigable table for tabular data.

#### Quick Start
```rust
use bubbles::table::{Column, Table};

let columns = vec![Column::new("ID", 6), Column::new("Name", 20)];
let rows = vec![vec!["1".into(), "Alice".into()]];
let table = Table::new().columns(columns).rows(rows).focused(true);
```

#### Messages
- `KeyMsg` (navigation via `key_map`)

#### Commands
- None

#### Configuration
- `columns(...)`, `rows(...)`
- `width(...)`, `height(...)`
- `focused(true/false)`
- `styles` and `key_map`

#### Styling
- `styles.header`, `styles.cell`, `styles.selected`

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::table::{Column, Table};

struct App {
    table: Table,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.table.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
        }
        self.table.update(msg)
    }

    fn view(&self) -> String {
        self.table.view()
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let columns = vec![
        Column::new("ID", 6),
        Column::new("Name", 20),
        Column::new("Status", 10),
    ];
    let rows = vec![
        vec!["1".into(), "Alice".into(), "Active".into()],
        vec!["2".into(), "Bob".into(), "Inactive".into()],
    ];

    let table = Table::new()
        .columns(columns)
        .rows(rows)
        .width(40)
        .height(10)
        .focused(true);

    let app = App { table };
    Program::new(app).run().map(|_| ())
}
```

### list

#### Overview
Filterable, paginated list with optional spinner and status bar.

#### Quick Start
```rust
use bubbles::list::{DefaultDelegate, Item, List};

#[derive(Clone)]
struct MyItem(String);

impl Item for MyItem {
    fn filter_value(&self) -> &str {
        &self.0
    }
}

let items = vec![MyItem("Apple".into()), MyItem("Banana".into())];
let list = List::new(items, DefaultDelegate::new(), 40, 10);
```

#### Messages
- `KeyMsg` (navigation/filtering)
- `StatusMessageTimeoutMsg`
- `spinner::TickMsg` (when spinner is visible)
- When filtering, messages are forwarded to the embedded `TextInput`

#### Commands
- `new_status_message(...)` returns a timeout command
- `filter_input.update(...)` may return cursor blink commands
- `spinner.update(...)` returns the next tick command

#### Configuration
- `title`, `show_title`, `show_filter`, `show_status_bar`, `show_help`
- `filtering_enabled`, `infinite_scrolling`
- `set_width(...)`, `set_height(...)`
- `key_map`, `styles`, `status_message_lifetime`

#### Styling
- `styles.title`, `styles.status_bar`, `styles.pagination`, `styles.help`, etc.

#### Full Example
```rust
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, quit};
use bubbles::list::{DefaultDelegate, Item, List};

#[derive(Clone)]
struct MyItem(String);

impl Item for MyItem {
    fn filter_value(&self) -> &str {
        &self.0
    }
}

struct App {
    list: List<MyItem, DefaultDelegate>,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.list.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == vec!['q'] {
                return Some(quit());
            }
        }
        self.list.update(msg)
    }

    fn view(&self) -> String {
        self.list.view()
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let items = vec![MyItem("Apple".into()), MyItem("Banana".into())];
    let list = List::new(items, DefaultDelegate::new(), 40, 10);

    let app = App { list };
    Program::new(app).run().map(|_| ())
}
```

### filepicker

#### Overview
File system browser for selecting files and directories.

#### Quick Start
```rust
use bubbles::filepicker::FilePicker;

let picker = FilePicker::new();
```

#### Messages
- `ReadDirMsg { id, entries }`
- `ReadDirErrMsg { id, error }`
- `WindowSizeMsg` (auto height)
- `KeyMsg` (navigation, open, select)

#### Commands
- `init()` returns a directory read command
- `update(...)` returns a new read command when changing directories

#### Configuration
- `current_directory` (via `set_current_directory`)
- `height`, `auto_height`
- `allowed_types`, `dir_allowed`, `file_allowed`
- `show_hidden`, `show_permissions`, `show_size`
- `cursor_char`, `styles`, `key_map`

#### Styling
- `styles.cursor`, `styles.directory`, `styles.file`, `styles.selected`, etc.

#### Full Example
```rust
use bubbletea::{Cmd, Message, Model, Program, quit};
use bubbles::filepicker::FilePicker;

struct App {
    picker: FilePicker,
    selected: Option<String>,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        self.picker.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(path) = self.picker.did_select_file(&msg) {
            self.selected = Some(path.to_string_lossy().to_string());
            return Some(quit());
        }
        self.picker.update(msg)
    }

    fn view(&self) -> String {
        let mut out = self.picker.view();
        if let Some(path) = &self.selected {
            out.push_str(&format!("\nSelected: {path}"));
        }
        out
    }
}

fn main() -> Result<(), bubbletea::Error> {
    let picker = FilePicker::new();
    let app = App {
        picker,
        selected: None,
    };
    Program::new(app).run().map(|_| ())
}
```

## Migration Guide: Non-Model to Model Usage

If you previously used components by manually calling `update` and `view` from your own code, you can migrate to the Model pattern in two steps:

1. **Embed the component in your model**
   ```rust
   struct App { list: List<MyItem, DefaultDelegate> }
   ```

2. **Forward messages and commands**
   ```rust
   fn update(&mut self, msg: Message) -> Option<Cmd> {
       self.list.update(msg)
   }
   ```

When a component returns a `Cmd`, return it from your app so the event loop keeps running (e.g., cursor blink, spinner ticks, timer ticks).

## FAQ

**Q: My text input/textarea doesn't respond to keys.**
A: Call `focus()` before running and forward `Message` values to `update`.

**Q: My spinner/progress doesn't animate.**
A: Make sure you return the command from `init()` or from `update(...)` so tick/frame messages are scheduled.

**Q: My file picker stays empty.**
A: Call `init()` so it returns a read-dir command, or trigger a read command after changing directories.

**Q: Help view never toggles.**
A: You must send `ToggleFullHelpMsg` from your own `update` logic when a key is pressed.

## Performance Considerations

- Prefer pagination (`Paginator` or `List`) over rendering massive lists every frame.
- Keep viewport sizes bounded; rendering scales with width * height.
- Reuse components and styles instead of rebuilding them on every update.
- Avoid excessive styling in tight loops; precompute where possible.
- For timers/spinners, keep tick rates reasonable to avoid unnecessary redraws.
