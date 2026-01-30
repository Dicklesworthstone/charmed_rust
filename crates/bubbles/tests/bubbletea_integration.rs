//! Integration tests for bubbles components within the bubbletea event loop.
//!
//! These tests verify that components work correctly when composed in a parent App
//! and driven by the bubbletea runtime (simulated).

#![forbid(unsafe_code)]
#![expect(
    clippy::items_after_statements,
    clippy::float_cmp,
    clippy::uninlined_format_args,
    clippy::single_char_pattern
)]

use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::textarea::TextArea;
use bubbles::textinput::TextInput;
use bubbles::timer::Timer;
use bubbles::viewport::Viewport;
use bubbletea::simulator::ProgramSimulator;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, batch};
use std::time::Duration;

// ============================================================================
// Scenario 1: Form with Focus Management
// Tests: TextInput + TextArea, Tab navigation, Key event routing
// ============================================================================

struct FormApp {
    name_input: TextInput,
    bio_input: TextArea,
    focus_index: usize,
}

impl FormApp {
    fn new() -> Self {
        let mut name = TextInput::new();
        name.set_placeholder("Name");
        name.focus(); // Initial focus

        let bio = TextArea::new();

        Self {
            name_input: name,
            bio_input: bio,
            focus_index: 0,
        }
    }
}

impl Model for FormApp {
    fn init(&self) -> Option<Cmd> {
        // Return batch of init commands from children
        batch(vec![self.name_input.init(), self.bio_input.init()])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle global navigation
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && key.key_type == KeyType::Tab
        {
            self.focus_index = (self.focus_index + 1) % 2;

            if self.focus_index == 0 {
                self.name_input.focus();
                self.bio_input.blur();
            } else {
                self.name_input.blur();
                self.bio_input.focus();
            }
            return None;
        }

        // Route messages to focused component (only update the focused one)
        if self.focus_index == 0 {
            self.name_input.update(msg)
        } else {
            self.bio_input.update(msg)
        }
    }

    fn view(&self) -> String {
        format!("{}\n{}", self.name_input.view(), self.bio_input.view())
    }
}

#[test]
fn test_form_focus_and_input_routing() {
    let mut sim = ProgramSimulator::new(FormApp::new());
    sim.init();

    // 1. Initial state: Name focused
    assert!(sim.model().name_input.focused());
    assert!(!sim.model().bio_input.focused());

    // 2. Type "Alice" into Name (step once per key to avoid cursor blink loop)
    for c in "Alice".chars() {
        sim.sim_key(c);
        sim.step(); // Process single key event
    }

    assert_eq!(sim.model().name_input.value(), "Alice");
    assert_eq!(sim.model().bio_input.value(), "");

    // 3. Tab to switch focus
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    assert!(!sim.model().name_input.focused());
    assert!(sim.model().bio_input.focused());

    // 4. Type "Dev" into Bio
    for c in "Dev".chars() {
        sim.sim_key(c);
        sim.step();
    }

    assert_eq!(sim.model().name_input.value(), "Alice"); // Should be unchanged
    assert_eq!(sim.model().bio_input.value(), "Dev");
}

// ============================================================================
// Scenario 2: Async Command Integration
// Tests: Spinner + Timer, Tick propagation, Cmd composition
// ============================================================================

struct AsyncApp {
    spinner: SpinnerModel,
    timer: Timer,
    finished: bool,
    last_msg_for_spinner: bool,
}

impl AsyncApp {
    fn new() -> Self {
        Self {
            spinner: SpinnerModel::with_spinner(spinners::dot()),
            timer: Timer::new(Duration::from_mins(1)), // Long timer to avoid early finish
            finished: false,
            last_msg_for_spinner: true, // Alternate between spinner and timer
        }
    }
}

impl Model for AsyncApp {
    fn init(&self) -> Option<Cmd> {
        // Start both
        batch(vec![self.spinner.init(), self.timer.init()])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Route alternately to prevent both consuming the same message
        // In a real app, messages are typically specific to each component (by ID)
        if self.last_msg_for_spinner {
            self.last_msg_for_spinner = false;
            self.spinner.update(msg)
        } else {
            self.last_msg_for_spinner = true;
            let cmd = self.timer.update(msg);
            // Check if timer finished
            if !self.timer.running() && !self.finished {
                self.finished = true;
            }
            cmd
        }
    }

    fn view(&self) -> String {
        if self.finished {
            "Done!".to_string()
        } else {
            format!("{} {}", self.spinner.view(), self.timer.view())
        }
    }
}

#[test]
fn test_async_component_integration() {
    let mut sim = ProgramSimulator::new(AsyncApp::new());

    // 1. Init should trigger commands for both components
    let init_cmd = sim.init();
    assert!(init_cmd.is_some(), "Init should return batch command");

    // Execute init batch (spinner tick + timer tick)
    if let Some(cmd) = init_cmd
        && let Some(batch_msg) = cmd.execute()
    {
        sim.send(batch_msg);
    }

    // 2. Step a few times to process commands (avoid run_until_empty due to timer tick loops)
    for _ in 0..5 {
        sim.step();
    }

    // 3. Verify view renders (spinner and timer both produce output)
    let view = sim.model().view();

    // If timer finished, view will be "Done!", otherwise it will contain spinner and timer
    // Either state is valid for this test - we're testing that the components can be composed
    assert!(!view.is_empty(), "View should not be empty");

    // The view should either be "Done!" or contain spinner characters
    let is_done = view.contains("Done!");
    let has_spinner = view.contains('⣾')
        || view.contains('⣽')
        || view.contains('⣻')
        || view.contains('⢿')
        || view.contains('⡿')
        || view.contains('⣟')
        || view.contains('⣯')
        || view.contains('⣷');

    assert!(
        is_done || has_spinner,
        "View should show either 'Done!' or spinner, got: {view}"
    );
}

// ============================================================================
// Scenario 3: Batch Commands & Viewport Scrolling
// Tests: Viewport + Key handling, Batch execution order
// ============================================================================

struct LogViewer {
    viewport: Viewport,
    content: String, // Store content separately since Viewport doesn't expose it
    auto_scroll: bool,
}

#[derive(Clone)]
struct AddLogMsg(String);

impl LogViewer {
    fn new() -> Self {
        let mut vp = Viewport::new(20, 5);
        let initial_content = "Log started...".to_string();
        vp.set_content(&initial_content);
        Self {
            viewport: vp,
            content: initial_content,
            auto_scroll: true,
        }
    }
}

impl Model for LogViewer {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(AddLogMsg(line)) = msg.downcast_ref::<AddLogMsg>() {
            // Append log line
            self.content.push('\n');
            self.content.push_str(line);
            self.viewport.set_content(&self.content);

            if self.auto_scroll {
                self.viewport.goto_bottom();
            }
            return None;
        }

        // Handle viewport navigation (returns () not Option<Cmd>)
        self.viewport.update(&msg);
        None
    }

    fn view(&self) -> String {
        self.viewport.view()
    }
}

#[test]
fn test_viewport_batch_updates() {
    let mut sim = ProgramSimulator::new(LogViewer::new());
    sim.init();

    // 1. Send a batch of log messages
    use bubbletea::message::BatchMsg;

    let batch = BatchMsg(vec![
        Cmd::new(|| Message::new(AddLogMsg("Line 1".into()))),
        Cmd::new(|| Message::new(AddLogMsg("Line 2".into()))),
        Cmd::new(|| Message::new(AddLogMsg("Line 3".into()))),
        Cmd::new(|| Message::new(AddLogMsg("Line 4".into()))),
        Cmd::new(|| Message::new(AddLogMsg("Line 5".into()))),
    ]);

    sim.send(Message::new(batch));
    sim.run_until_empty();

    // 2. Verify content added and scrolled
    let model = sim.model();
    assert!(model.content.contains("Line 5"));

    // Viewport height is 5. We added 5 lines + 1 initial = 6 lines.
    // With auto-scroll, we should be at the bottom.
    assert!(model.viewport.at_bottom());

    // 3. Test manual scrolling (simulating keys)
    sim.sim_key_type(KeyType::Up); // Scroll up
    sim.run_until_empty();

    assert!(!sim.model().viewport.at_bottom(), "Should scroll up");
}

// ============================================================================
// Scenario 4: Viewport Mouse Wheel Scrolling
// Tests: Mouse events routing to viewport
// ============================================================================

struct MouseScrollApp {
    viewport: Viewport,
}

impl MouseScrollApp {
    fn new() -> Self {
        let mut vp = Viewport::new(40, 5);
        // mouse_wheel_enabled is true by default
        // Add enough content to scroll
        vp.set_content(
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10",
        );
        Self { viewport: vp }
    }
}

impl Model for MouseScrollApp {
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

#[test]
fn test_viewport_mouse_wheel_scrolling() {
    use bubbletea::mouse::{MouseAction, MouseButton};

    let mut sim = ProgramSimulator::new(MouseScrollApp::new());
    sim.init();

    // 1. Initial state: at top
    assert!(sim.model().viewport.at_top());
    assert_eq!(sim.model().viewport.y_offset(), 0);

    // 2. Scroll down with mouse wheel
    sim.sim_mouse(5, 2, MouseButton::WheelDown, MouseAction::Press);
    sim.run_until_empty();

    assert!(
        !sim.model().viewport.at_top(),
        "Should scroll down on wheel"
    );

    // 3. Scroll back up with mouse wheel
    sim.sim_mouse(5, 2, MouseButton::WheelUp, MouseAction::Press);
    sim.run_until_empty();

    assert!(sim.model().viewport.at_top(), "Should scroll back to top");
}

// ============================================================================
// Scenario 5: Progress Updates from Async Commands
// Tests: Progress component with animated updates
// ============================================================================

use bubbles::progress::Progress;

struct ProgressApp {
    progress: Progress,
    percent: f64,
}

#[derive(Clone)]
struct SetProgressMsg(f64);

impl ProgressApp {
    fn new() -> Self {
        Self {
            progress: Progress::new().width(20),
            percent: 0.0,
        }
    }
}

impl Model for ProgressApp {
    fn init(&self) -> Option<Cmd> {
        self.progress.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle custom progress messages
        if let Some(SetProgressMsg(p)) = msg.downcast_ref::<SetProgressMsg>() {
            self.percent = *p;
            self.progress.set_percent(*p);
        }

        // Forward to progress for animation frames
        self.progress.update(msg)
    }

    fn view(&self) -> String {
        self.progress.view()
    }
}

#[test]
fn test_progress_async_updates() {
    let mut sim = ProgramSimulator::new(ProgressApp::new());

    // 1. Init triggers animation frame command
    let _init_cmd = sim.init();
    // Progress may or may not return an init command depending on implementation

    // 2. Set progress to 50%
    sim.send(Message::new(SetProgressMsg(0.5)));
    sim.run_until_empty();

    assert_eq!(sim.model().percent, 0.5);

    // 3. Set progress to 100%
    sim.send(Message::new(SetProgressMsg(1.0)));
    sim.run_until_empty();

    assert_eq!(sim.model().percent, 1.0);

    // 4. Verify view renders progress bar
    let view = sim.model().view();
    assert!(!view.is_empty(), "Progress view should render");
}

// ============================================================================
// Scenario 6: Multi-Component with Mouse Event Routing
// Tests: Mouse events route to correct component based on position
// ============================================================================

struct MultiPanelApp {
    left_viewport: Viewport,
    right_viewport: Viewport,
    // Layout: left panel is columns 0-19, right panel is columns 20-39
    left_clicks: usize,
    right_clicks: usize,
}

impl MultiPanelApp {
    fn new() -> Self {
        let mut left = Viewport::new(20, 10);
        left.set_content("Left panel\nClick here");
        // mouse_wheel_enabled is true by default

        let mut right = Viewport::new(20, 10);
        right.set_content("Right panel\nClick here");
        // mouse_wheel_enabled is true by default

        Self {
            left_viewport: left,
            right_viewport: right,
            left_clicks: 0,
            right_clicks: 0,
        }
    }
}

impl Model for MultiPanelApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        use bubbletea::mouse::MouseMsg;

        if let Some(mouse) = msg.downcast_ref::<MouseMsg>() {
            // Route based on X position
            if mouse.x < 20 {
                self.left_clicks += 1;
                self.left_viewport.update(&msg);
            } else {
                self.right_clicks += 1;
                self.right_viewport.update(&msg);
            }
        }

        None
    }

    fn view(&self) -> String {
        format!(
            "{} | {}",
            self.left_viewport.view(),
            self.right_viewport.view()
        )
    }
}

#[test]
fn test_mouse_event_routing_to_components() {
    use bubbletea::mouse::{MouseAction, MouseButton};

    let mut sim = ProgramSimulator::new(MultiPanelApp::new());
    sim.init();

    // 1. Click in left panel (x < 20)
    sim.sim_mouse(5, 2, MouseButton::Left, MouseAction::Press);
    sim.run_until_empty();

    assert_eq!(sim.model().left_clicks, 1);
    assert_eq!(sim.model().right_clicks, 0);

    // 2. Click in right panel (x >= 20)
    sim.sim_mouse(25, 2, MouseButton::Left, MouseAction::Press);
    sim.run_until_empty();

    assert_eq!(sim.model().left_clicks, 1);
    assert_eq!(sim.model().right_clicks, 1);

    // 3. Multiple clicks in each panel
    sim.sim_mouse(10, 5, MouseButton::Left, MouseAction::Press);
    sim.sim_mouse(30, 5, MouseButton::Left, MouseAction::Press);
    sim.run_until_empty();

    assert_eq!(sim.model().left_clicks, 2);
    assert_eq!(sim.model().right_clicks, 2);
}

// ============================================================================
// Scenario 7: Nested Component Rendering & Style Composition
// Tests: Component view() in parent view(), styles don't corrupt
// ============================================================================

use lipgloss::Style as LipStyle;

struct StyledPanelApp {
    input: TextInput,
    viewport: Viewport,
    panel_style: LipStyle,
    input_wrapper_style: LipStyle,
}

impl StyledPanelApp {
    fn new() -> Self {
        let mut input = TextInput::new();
        input.set_placeholder("Enter text...");
        input.set_prompt(">> ");

        let mut vp = Viewport::new(30, 5);
        vp.set_content("Content line 1\nContent line 2\nContent line 3");

        Self {
            input,
            viewport: vp,
            panel_style: LipStyle::new()
                .padding(1)
                .border(lipgloss::Border::rounded()),
            input_wrapper_style: LipStyle::new().foreground("205"),
        }
    }
}

impl Model for StyledPanelApp {
    fn init(&self) -> Option<Cmd> {
        self.input.init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Route to input
        self.input.update(msg)
    }

    fn view(&self) -> String {
        // Nested rendering: input wrapped in style, then viewport, all in panel
        let input_view = self.input_wrapper_style.render(&self.input.view());
        let viewport_view = self.viewport.view();

        let content = format!("{}\n{}", input_view, viewport_view);
        self.panel_style.render(&content)
    }
}

#[test]
fn test_nested_component_rendering() {
    let mut sim = ProgramSimulator::new(StyledPanelApp::new());
    sim.init();

    // 1. Get initial view
    let view = sim.model().view();

    // View should contain input prompt and viewport content
    assert!(
        view.contains(">>") || view.contains(">"),
        "Should contain input prompt"
    );
    assert!(
        view.contains("Content line"),
        "Should contain viewport content"
    );

    // 2. Type some text (step once per key to avoid cursor blink loop)
    sim.model_mut().input.focus();
    for c in "hello".chars() {
        sim.sim_key(c);
        sim.step();
    }

    // 3. Verify view still renders correctly with input value
    let view_after = sim.model().view();
    assert!(view_after.contains("hello"), "Should contain typed text");

    // 4. Verify styles are applied (view should have border characters from rounded border)
    // Rounded borders use: ╭ ╮ ╰ ╯ │ ─
    let has_border =
        view_after.contains('╭') || view_after.contains('│') || view_after.contains('─');
    assert!(has_border, "Should have border styling applied");
}

#[test]
fn test_style_composition_no_corruption() {
    let mut sim = ProgramSimulator::new(StyledPanelApp::new());
    sim.init();

    // 1. Get multiple renders
    let view1 = sim.model().view();
    let view2 = sim.model().view();
    let view3 = sim.model().view();

    // Views should be identical (no state corruption from rendering)
    assert_eq!(view1, view2, "Consecutive views should be identical");
    assert_eq!(view2, view3, "Consecutive views should be identical");

    // 2. Modify state and verify no corruption
    sim.model_mut().input.focus();
    sim.model_mut().input.set_value("test");

    let view_modified1 = sim.model().view();
    let view_modified2 = sim.model().view();

    assert_eq!(
        view_modified1, view_modified2,
        "Modified views should be identical"
    );
    assert_ne!(
        view1, view_modified1,
        "Modified view should differ from original"
    );
}

// ============================================================================
// Scenario 8: Focus Styling Changes
// Tests: Visual feedback for focus/blur state
// ============================================================================

struct FocusStyleApp {
    input1: TextInput,
    input2: TextInput,
    focused_idx: usize,
}

impl FocusStyleApp {
    fn new() -> Self {
        let mut i1 = TextInput::new();
        i1.set_prompt("[1] ");
        i1.focus();

        let mut i2 = TextInput::new();
        i2.set_prompt("[2] ");

        Self {
            input1: i1,
            input2: i2,
            focused_idx: 0,
        }
    }

    fn switch_focus(&mut self) {
        if self.focused_idx == 0 {
            self.input1.blur();
            self.input2.focus();
            self.focused_idx = 1;
        } else {
            self.input2.blur();
            self.input1.focus();
            self.focused_idx = 0;
        }
    }
}

impl Model for FocusStyleApp {
    fn init(&self) -> Option<Cmd> {
        batch(vec![self.input1.init(), self.input2.init()])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && key.key_type == KeyType::Tab
        {
            self.switch_focus();
            return None;
        }

        // Route to focused input
        if self.focused_idx == 0 {
            self.input1.update(msg)
        } else {
            self.input2.update(msg)
        }
    }

    fn view(&self) -> String {
        format!("{}\n{}", self.input1.view(), self.input2.view())
    }
}

#[test]
fn test_focus_styling_changes() {
    let mut sim = ProgramSimulator::new(FocusStyleApp::new());
    sim.init();

    // 1. Initial state: input1 focused
    assert!(sim.model().input1.focused());
    assert!(!sim.model().input2.focused());

    let view_initial = sim.model().view();

    // 2. Switch focus via Tab (step once to avoid cursor blink loop)
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    assert!(!sim.model().input1.focused());
    assert!(sim.model().input2.focused());

    let view_after_tab = sim.model().view();

    // Views might differ due to cursor blink state, but structure should be same
    // Both should contain the prompts
    assert!(view_initial.contains("[1]"));
    assert!(view_initial.contains("[2]"));
    assert!(view_after_tab.contains("[1]"));
    assert!(view_after_tab.contains("[2]"));

    // 3. Type in focused input (step once per key)
    for c in "focused".chars() {
        sim.sim_key(c);
        sim.step();
    }

    // Text should appear in input2 (now focused)
    assert_eq!(sim.model().input1.value(), "");
    assert_eq!(sim.model().input2.value(), "focused");

    // 4. Switch back and type
    sim.sim_key_type(KeyType::Tab);
    sim.step();

    for c in "also".chars() {
        sim.sim_key(c);
        sim.step();
    }

    assert_eq!(sim.model().input1.value(), "also");
    assert_eq!(sim.model().input2.value(), "focused");
}
