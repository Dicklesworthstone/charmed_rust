//! Integration tests for bubbles components within the bubbletea event loop.
//!
//! These tests verify that components work correctly when composed in a parent App
//! and driven by the bubbletea runtime (simulated).

#![forbid(unsafe_code)]

use bubbles::spinner::{SpinnerModel, spinners};
use bubbles::textarea::TextArea;
use bubbles::textinput::TextInput;
use bubbles::timer::Timer;
use bubbles::viewport::Viewport;
use bubbletea::simulator::ProgramSimulator;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
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

        let mut bio = TextArea::new();
        bio.set_placeholder("Bio");

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
        Some(Cmd::batch(vec![
            self.name_input.init(),
            self.bio_input.init(),
        ]))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle global navigation
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Tab {
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
        }

        // Route messages to focused component
        let mut cmds = Vec::new();

        if self.focus_index == 0 {
            if let Some(cmd) = self.name_input.update(msg.clone()) {
                cmds.push(cmd);
            }
        } else {
            if let Some(cmd) = self.bio_input.update(msg.clone()) {
                cmds.push(cmd);
            }
        }

        if cmds.is_empty() {
            None
        } else {
            Some(Cmd::batch(cmds.into_iter().map(Some).collect()))
        }
    }

    fn view(&self) -> String {
        format!("{}
{}", self.name_input.view(), self.bio_input.view())
    }
}

#[test]
fn test_form_focus_and_input_routing() {
    let mut sim = ProgramSimulator::new(FormApp::new());
    sim.init();

    // 1. Initial state: Name focused
    assert!(sim.model().name_input.focused());
    assert!(!sim.model().bio_input.focused());

    // 2. Type "Alice" into Name
    for c in "Alice".chars() {
        sim.sim_key(c);
    }
    sim.run_until_empty(); // Process input events

    assert_eq!(sim.model().name_input.value(), "Alice");
    assert_eq!(sim.model().bio_input.value(), "");

    // 3. Tab to switch focus
    sim.sim_key_type(KeyType::Tab);
    sim.run_until_empty();

    assert!(!sim.model().name_input.focused());
    assert!(sim.model().bio_input.focused());

    // 4. Type "Dev" into Bio
    for c in "Dev".chars() {
        sim.sim_key(c);
    }
    sim.run_until_empty();

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
}

impl AsyncApp {
    fn new() -> Self {
        Self {
            spinner: SpinnerModel::with_spinner(spinners::dot()),
            timer: Timer::new(Duration::from_millis(50)), // Short timer for test
            finished: false,
        }
    }
}

impl Model for AsyncApp {
    fn init(&self) -> Option<Cmd> {
        // Start both
        Some(Cmd::batch(vec![
            self.spinner.init(),
            self.timer.init(),
        ]))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        let mut cmds = Vec::new();

        // Update spinner (consumes tick messages)
        if let Some(cmd) = self.spinner.update(msg.clone()) {
            cmds.push(cmd);
        }

        // Update timer (consumes tick messages)
        // Note: In a real app, we might need to route specific ticks, 
        // but bubbles components usually filter by their own ID if strictly implemented.
        // Here we assume they handle generic ticks or self-scheduled ticks.
        if let Some(cmd) = self.timer.update(msg.clone()) {
            cmds.push(cmd);
        }

        // Check if timer finished
        if !self.timer.running() && !self.finished {
            self.finished = true;
        }

        if cmds.is_empty() {
            None
        } else {
            Some(Cmd::batch(cmds.into_iter().map(Some).collect()))
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
    
    // 1. Init should trigger ticks for both
    let init_cmd = sim.init();
    assert!(init_cmd.is_some());
    
    // Execute init batch (spinner tick + timer tick)
    if let Some(cmd) = init_cmd {
        if let Some(batch_msg) = cmd.execute() {
            sim.send(batch_msg);
        }
    }
    
    // 2. Process initial ticks
    // This should advance spinner frame and update timer
    let processed = sim.run_until_empty();
    assert!(processed >= 2, "Should process at least spinner and timer ticks");
    
    // Spinner frame should have advanced (frame 0 -> 1)
    // Note: Depends on internal implementation of SpinnerModel, assuming frame starts at 0
    // and updates on tick.
    
    // 3. Simulate passage of time/ticks until timer finishes
    // We simulate ticks by extracting pending commands from update() and executing them
    // The Simulator run_until_empty does this automatically for us!
    
    // However, since we want to verify intermediate state, we can step carefully.
    // Ideally, we'd inject time, but bubbles Timer uses Instant::now().
    // For this test, we verify that the loop runs and updates state.
    
    assert!(!sim.model().finished);
    assert!(sim.model().timer.running());
    
    // Verify view contains spinner
    let view = sim.model().view();
    assert!(view.contains('⣾') || view.contains('⣽') || view.contains('⣻') || view.contains('⢿') || view.contains('⡿') || view.contains('⣟') || view.contains('⣯') || view.contains('⣷'), "View should contain spinner dots");
}

// ============================================================================ 
// Scenario 3: Batch Commands & Viewport Scrolling
// Tests: Viewport + Key handling, Batch execution order
// ============================================================================ 

struct LogViewer {
    viewport: Viewport,
    auto_scroll: bool,
}

#[derive(Clone)]
struct AddLogMsg(String);

impl LogViewer {
    fn new() -> Self {
        let mut vp = Viewport::new(20, 5);
        vp.set_content("Log started...");
        Self {
            viewport: vp,
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
            let mut content = self.viewport.content().to_string();
            content.push_str("\n");
            content.push_str(line);
            self.viewport.set_content(&content);
            
            if self.auto_scroll {
                self.viewport.goto_bottom();
            }
            return None;
        }

        // Handle viewport navigation
        self.viewport.update(msg)
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
    assert!(model.viewport.content().contains("Line 5"));
    
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
        vp.set_mouse_wheel_enabled(true);
        // Add enough content to scroll
        vp.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10");
        Self { viewport: vp }
    }
}

impl Model for MouseScrollApp {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.viewport.update(msg)
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

    assert!(!sim.model().viewport.at_top(), "Should scroll down on wheel");

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
    let init_cmd = sim.init();
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
        left.set_mouse_wheel_enabled(true);

        let mut right = Viewport::new(20, 10);
        right.set_content("Right panel\nClick here");
        right.set_mouse_wheel_enabled(true);

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
                return self.left_viewport.update(msg);
            } else {
                self.right_clicks += 1;
                return self.right_viewport.update(msg);
            }
        }

        None
    }

    fn view(&self) -> String {
        format!("{} | {}", self.left_viewport.view(), self.right_viewport.view())
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
            panel_style: LipStyle::new().padding(1).border(lipgloss::Border::rounded()),
            input_wrapper_style: LipStyle::new().foreground(lipgloss::Color::from("205")),
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
    assert!(view.contains(">>") || view.contains(">"), "Should contain input prompt");
    assert!(view.contains("Content line"), "Should contain viewport content");

    // 2. Type some text
    sim.model_mut().input.focus();
    for c in "hello".chars() {
        sim.sim_key(c);
    }
    sim.run_until_empty();

    // 3. Verify view still renders correctly with input value
    let view_after = sim.model().view();
    assert!(view_after.contains("hello"), "Should contain typed text");

    // 4. Verify styles are applied (view should have border characters from rounded border)
    // Rounded borders use: ╭ ╮ ╰ ╯ │ ─
    let has_border = view_after.contains('╭') || view_after.contains('│') || view_after.contains('─');
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

    assert_eq!(view_modified1, view_modified2, "Modified views should be identical");
    assert_ne!(view1, view_modified1, "Modified view should differ from original");
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
        Some(Cmd::batch(vec![
            self.input1.init(),
            self.input2.init(),
        ]))
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Tab {
                self.switch_focus();
                return None;
            }
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

    // 2. Switch focus via Tab
    sim.sim_key_type(KeyType::Tab);
    sim.run_until_empty();

    assert!(!sim.model().input1.focused());
    assert!(sim.model().input2.focused());

    let view_after_tab = sim.model().view();

    // Views might differ due to cursor blink state, but structure should be same
    // Both should contain the prompts
    assert!(view_initial.contains("[1]"));
    assert!(view_initial.contains("[2]"));
    assert!(view_after_tab.contains("[1]"));
    assert!(view_after_tab.contains("[2]"));

    // 3. Type in focused input
    for c in "focused".chars() {
        sim.sim_key(c);
    }
    sim.run_until_empty();

    // Text should appear in input2 (now focused)
    assert_eq!(sim.model().input1.value(), "");
    assert_eq!(sim.model().input2.value(), "focused");

    // 4. Switch back and type
    sim.sim_key_type(KeyType::Tab);
    sim.run_until_empty();

    for c in "also".chars() {
        sim.sim_key(c);
    }
    sim.run_until_empty();

    assert_eq!(sim.model().input1.value(), "also");
    assert_eq!(sim.model().input2.value(), "focused");
}
