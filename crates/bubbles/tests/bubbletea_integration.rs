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
