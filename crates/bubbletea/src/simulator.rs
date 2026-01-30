//! Program simulator for testing lifecycle without a real terminal.
//!
//! This module provides a way to test Model implementations without
//! requiring a real terminal, enabling unit tests for the Elm Architecture.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::Model;
use crate::command::Cmd;
use crate::message::{BatchMsg, Message, QuitMsg, SequenceMsg};

/// Statistics tracked during simulation.
#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    /// Number of times init() was called.
    pub init_calls: usize,
    /// Number of times update() was called.
    pub update_calls: usize,
    /// Number of times view() was called.
    pub view_calls: usize,
    /// Commands that were returned from init/update.
    pub commands_returned: usize,
    /// Whether quit was requested.
    pub quit_requested: bool,
}

/// A simulator for testing Model implementations without a terminal.
///
/// # Example
///
/// ```rust
/// use bubbletea::{Model, Message, Cmd, simulator::ProgramSimulator};
///
/// struct Counter { count: i32 }
///
/// impl Model for Counter {
///     fn init(&self) -> Option<Cmd> { None }
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if let Some(n) = msg.downcast::<i32>() {
///             self.count += n;
///         }
///         None
///     }
///     fn view(&self) -> String {
///         format!("Count: {}", self.count)
///     }
/// }
///
/// let mut sim = ProgramSimulator::new(Counter { count: 0 });
/// sim.send(Message::new(5));
/// sim.send(Message::new(3));
/// sim.step();
/// sim.step();
///
/// assert_eq!(sim.model().count, 8);
/// ```
pub struct ProgramSimulator<M: Model> {
    model: M,
    input_queue: VecDeque<Message>,
    output_views: Vec<String>,
    stats: SimulationStats,
    initialized: bool,
}

impl<M: Model> ProgramSimulator<M> {
    /// Create a new simulator with the given model.
    pub fn new(model: M) -> Self {
        Self {
            model,
            input_queue: VecDeque::new(),
            output_views: Vec::new(),
            stats: SimulationStats::default(),
            initialized: false,
        }
    }

    /// Initialize the model, calling init() and capturing any returned command.
    pub fn init(&mut self) -> Option<Cmd> {
        if self.initialized {
            return None;
        }
        self.initialized = true;
        self.stats.init_calls += 1;

        // Call init
        let cmd = self.model.init();
        if cmd.is_some() {
            self.stats.commands_returned += 1;
        }

        // Call initial view
        self.stats.view_calls += 1;
        self.output_views.push(self.model.view());

        cmd
    }

    /// Queue a message for processing.
    pub fn send(&mut self, msg: Message) {
        self.input_queue.push_back(msg);
    }

    /// Process one message from the queue, calling update and view.
    ///
    /// Returns the command returned by update, if any.
    pub fn step(&mut self) -> Option<Cmd> {
        // Ensure initialized
        if !self.initialized {
            self.init();
        }

        if let Some(msg) = self.input_queue.pop_front() {
            // Check for quit
            if msg.is::<QuitMsg>() {
                self.stats.quit_requested = true;
                return Some(crate::quit());
            }

            // Handle batch messages specially - extract and execute the commands
            if msg.is::<BatchMsg>() {
                if let Some(batch) = msg.downcast::<BatchMsg>() {
                    for cmd in batch.0 {
                        if let Some(result_msg) = cmd.execute() {
                            self.input_queue.push_back(result_msg);
                        }
                    }
                }
                // View after batch
                self.stats.view_calls += 1;
                self.output_views.push(self.model.view());
                return None;
            }

            // Handle sequence messages specially - execute commands in order
            if msg.is::<SequenceMsg>() {
                if let Some(seq) = msg.downcast::<SequenceMsg>() {
                    for cmd in seq.0 {
                        if let Some(result_msg) = cmd.execute() {
                            self.input_queue.push_back(result_msg);
                        }
                    }
                }
                // View after sequence
                self.stats.view_calls += 1;
                self.output_views.push(self.model.view());
                return None;
            }

            // Update
            self.stats.update_calls += 1;
            let cmd = self.model.update(msg);
            if cmd.is_some() {
                self.stats.commands_returned += 1;
            }

            // View
            self.stats.view_calls += 1;
            self.output_views.push(self.model.view());

            return cmd;
        }

        None
    }

    /// Process all pending messages until the queue is empty or quit is requested.
    ///
    /// Returns the number of messages processed.
    /// Has a built-in safety limit of 1000 iterations to prevent infinite loops.
    pub fn run_until_empty(&mut self) -> usize {
        const MAX_ITERATIONS: usize = 1000;
        let mut processed = 0;
        while !self.input_queue.is_empty()
            && !self.stats.quit_requested
            && processed < MAX_ITERATIONS
        {
            if let Some(cmd) = self.step() {
                // Execute command and queue resulting message
                if let Some(msg) = cmd.execute() {
                    self.input_queue.push_back(msg);
                }
            }
            processed += 1;
        }
        processed
    }

    /// Run until quit is received or max_steps is reached.
    ///
    /// Returns the number of steps processed.
    pub fn run_until_quit(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        while steps < max_steps && !self.stats.quit_requested {
            if self.input_queue.is_empty() {
                break;
            }
            if let Some(cmd) = self.step() {
                // Execute command and queue resulting message
                if let Some(msg) = cmd.execute() {
                    self.input_queue.push_back(msg);
                }
            }
            steps += 1;
        }
        steps
    }

    /// Get a reference to the current model state.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Get a mutable reference to the current model state.
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Consume the simulator and return the final model.
    pub fn into_model(self) -> M {
        self.model
    }

    /// Get the simulation statistics.
    pub fn stats(&self) -> &SimulationStats {
        &self.stats
    }

    /// Get all captured view outputs.
    pub fn views(&self) -> &[String] {
        &self.output_views
    }

    /// Get the most recent view output.
    pub fn last_view(&self) -> Option<&str> {
        self.output_views.last().map(String::as_str)
    }

    /// Check if quit has been requested.
    pub fn is_quit(&self) -> bool {
        self.stats.quit_requested
    }

    /// Check if the model has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the number of pending messages.
    pub fn pending_count(&self) -> usize {
        self.input_queue.len()
    }

    // ========================================================================
    // Event Simulation Helpers
    // ========================================================================

    /// Simulate a key press (single character).
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbletea::simulator::ProgramSimulator;
    /// # use bubbletea::{Model, Message, Cmd};
    /// # struct MyModel;
    /// # impl Model for MyModel {
    /// #     fn init(&self) -> Option<Cmd> { None }
    /// #     fn update(&mut self, _: Message) -> Option<Cmd> { None }
    /// #     fn view(&self) -> String { String::new() }
    /// # }
    ///
    /// let mut sim = ProgramSimulator::new(MyModel);
    /// sim.init();
    /// sim.sim_key('a');  // Queue 'a' key press
    /// sim.step();
    /// ```
    pub fn sim_key(&mut self, c: char) {
        use crate::key::KeyMsg;
        self.send(Message::new(KeyMsg::from_char(c)));
    }

    /// Simulate a special key press (Enter, Escape, Arrow keys, etc.).
    pub fn sim_key_type(&mut self, key_type: crate::key::KeyType) {
        use crate::key::KeyMsg;
        self.send(Message::new(KeyMsg::from_type(key_type)));
    }

    /// Simulate a mouse event.
    ///
    /// # Arguments
    ///
    /// * `x` - Column position (0-indexed)
    /// * `y` - Row position (0-indexed)
    /// * `button` - Which button (Left, Right, Middle, etc.)
    /// * `action` - What happened (Press, Release, Motion)
    pub fn sim_mouse(
        &mut self,
        x: u16,
        y: u16,
        button: crate::mouse::MouseButton,
        action: crate::mouse::MouseAction,
    ) {
        use crate::mouse::MouseMsg;
        self.send(Message::new(MouseMsg {
            x,
            y,
            button,
            action,
            shift: false,
            alt: false,
            ctrl: false,
        }));
    }

    /// Simulate a window resize event.
    pub fn sim_resize(&mut self, width: u16, height: u16) {
        use crate::message::WindowSizeMsg;
        self.send(Message::new(WindowSizeMsg { width, height }));
    }

    /// Simulate a paste operation (bracketed paste).
    pub fn sim_paste(&mut self, text: &str) {
        use crate::key::KeyMsg;
        let runes: Vec<char> = text.chars().collect();
        self.send(Message::new(KeyMsg::from_runes(runes).with_paste()));
    }
}

/// A test model that tracks lifecycle calls with atomic counters.
///
/// Useful for verifying that init/update/view are called the expected
/// number of times.
pub struct TrackingModel {
    /// Counter for init calls.
    pub init_count: Arc<AtomicUsize>,
    /// Counter for update calls.
    pub update_count: Arc<AtomicUsize>,
    /// Counter for view calls.
    pub view_count: Arc<AtomicUsize>,
    /// Internal state for testing.
    pub value: i32,
}

impl TrackingModel {
    /// Create a new tracking model with fresh counters.
    pub fn new() -> Self {
        Self {
            init_count: Arc::new(AtomicUsize::new(0)),
            update_count: Arc::new(AtomicUsize::new(0)),
            view_count: Arc::new(AtomicUsize::new(0)),
            value: 0,
        }
    }

    /// Create a new tracking model with shared counters.
    pub fn with_counters(
        init_count: Arc<AtomicUsize>,
        update_count: Arc<AtomicUsize>,
        view_count: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            init_count,
            update_count,
            view_count,
            value: 0,
        }
    }

    /// Get the current init count.
    pub fn init_calls(&self) -> usize {
        self.init_count.load(Ordering::SeqCst)
    }

    /// Get the current update count.
    pub fn update_calls(&self) -> usize {
        self.update_count.load(Ordering::SeqCst)
    }

    /// Get the current view count.
    pub fn view_calls(&self) -> usize {
        self.view_count.load(Ordering::SeqCst)
    }
}

impl Default for TrackingModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for TrackingModel {
    fn init(&self) -> Option<Cmd> {
        self.init_count.fetch_add(1, Ordering::SeqCst);
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.update_count.fetch_add(1, Ordering::SeqCst);

        // Handle increment/decrement messages
        if let Some(n) = msg.downcast::<i32>() {
            self.value += n;
        }

        None
    }

    fn view(&self) -> String {
        self.view_count.fetch_add(1, Ordering::SeqCst);
        format!("Value: {}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_init_called_once() {
        let model = TrackingModel::new();
        let init_count = model.init_count.clone();

        let mut sim = ProgramSimulator::new(model);

        // Before init
        assert_eq!(init_count.load(Ordering::SeqCst), 0);

        // Explicit init
        sim.init();
        assert_eq!(init_count.load(Ordering::SeqCst), 1);

        // Second init should not increment
        sim.init();
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_simulator_view_called_after_init() {
        let model = TrackingModel::new();
        let view_count = model.view_count.clone();

        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // View called once after init
        assert_eq!(view_count.load(Ordering::SeqCst), 1);
        assert_eq!(sim.views().len(), 1);
        assert_eq!(sim.last_view(), Some("Value: 0"));
    }

    #[test]
    fn test_simulator_update_increments_value() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(5));
        sim.send(Message::new(3));
        sim.step();
        sim.step();

        assert_eq!(sim.model().value, 8);
        assert_eq!(sim.stats().update_calls, 2);
    }

    #[test]
    fn test_simulator_view_called_after_each_update() {
        let model = TrackingModel::new();
        let view_count = model.view_count.clone();

        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // 1 view from init
        assert_eq!(view_count.load(Ordering::SeqCst), 1);

        sim.send(Message::new(1));
        sim.step();
        // 1 from init + 1 from update
        assert_eq!(view_count.load(Ordering::SeqCst), 2);

        sim.send(Message::new(2));
        sim.step();
        // 1 from init + 2 from updates
        assert_eq!(view_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_simulator_quit_stops_processing() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(1));
        sim.send(Message::new(QuitMsg));
        sim.send(Message::new(2)); // Should not be processed

        sim.run_until_quit(10);

        assert!(sim.is_quit());
        assert_eq!(sim.model().value, 1); // Only first increment processed
    }

    #[test]
    fn test_simulator_run_until_empty() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(1));
        sim.send(Message::new(2));
        sim.send(Message::new(3));

        let processed = sim.run_until_empty();

        assert_eq!(processed, 3);
        assert_eq!(sim.model().value, 6);
    }

    #[test]
    fn test_simulator_stats() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(1));
        sim.send(Message::new(2));
        sim.step();
        sim.step();

        let stats = sim.stats();
        assert_eq!(stats.init_calls, 1);
        assert_eq!(stats.update_calls, 2);
        assert_eq!(stats.view_calls, 3); // 1 init + 2 updates
        assert!(!stats.quit_requested);
    }

    #[test]
    fn test_simulator_into_model() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);

        sim.init();
        sim.send(Message::new(42));
        sim.step();

        let final_model = sim.into_model();
        assert_eq!(final_model.value, 42);
    }

    #[test]
    fn test_simulator_implicit_init() {
        let model = TrackingModel::new();
        let init_count = model.init_count.clone();

        let mut sim = ProgramSimulator::new(model);

        // step() should implicitly init
        sim.send(Message::new(1));
        sim.step();

        assert_eq!(init_count.load(Ordering::SeqCst), 1);
        assert!(sim.is_initialized());
    }

    #[test]
    fn test_simulator_batch_command() {
        use crate::batch;

        // Model that triggers a batch command on a specific message
        struct BatchTrigger;
        #[derive(Clone, Copy)]
        struct SetValue(i32);
        #[derive(Clone, Copy)]
        struct AddValue(i32);

        struct BatchModel {
            value: i32,
        }

        impl Model for BatchModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }

            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if msg.is::<BatchTrigger>() {
                    // Return a batch of two commands
                    return batch(vec![
                        Some(crate::Cmd::new(|| Message::new(SetValue(10)))),
                        Some(crate::Cmd::new(|| Message::new(AddValue(5)))),
                    ]);
                }
                if let Some(SetValue(v)) = msg.downcast_ref::<SetValue>() {
                    self.value = *v;
                } else if let Some(AddValue(v)) = msg.downcast_ref::<AddValue>() {
                    self.value += *v;
                }
                None
            }

            fn view(&self) -> String {
                format!("Value: {}", self.value)
            }
        }

        let mut sim = ProgramSimulator::new(BatchModel { value: 0 });
        sim.init();

        // Send the batch trigger message
        sim.send(Message::new(BatchTrigger));

        // Step once to get the batch command
        let cmd = sim.step();
        assert!(cmd.is_some(), "Should return batch command");

        // Execute the batch command, which returns BatchMsg
        let batch_msg = cmd.unwrap().execute();
        assert!(batch_msg.is_some(), "Batch command should return BatchMsg");

        // Send the BatchMsg to the simulator
        sim.send(batch_msg.unwrap());

        // Process all messages
        sim.run_until_empty();

        // Value should be 10 + 5 = 15
        assert_eq!(
            sim.model().value,
            15,
            "Batch commands should set 10 then add 5"
        );
    }

    // ========================================================================
    // Event Simulation Tests (bd-ikfq)
    // ========================================================================

    #[test]
    fn test_sim_key_sends_char() {
        use crate::key::{KeyMsg, KeyType};

        struct KeyModel {
            keys: Vec<char>,
        }

        impl Model for KeyModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(key) = msg.downcast_ref::<KeyMsg>()
                    && key.key_type == KeyType::Runes
                {
                    self.keys.extend(&key.runes);
                }
                None
            }
            fn view(&self) -> String {
                format!("Keys: {:?}", self.keys)
            }
        }

        let mut sim = ProgramSimulator::new(KeyModel { keys: Vec::new() });
        sim.init();
        sim.sim_key('a');
        sim.sim_key('b');
        sim.sim_key('c');
        sim.run_until_empty();

        assert_eq!(sim.model().keys, vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_sim_key_type_sends_special_keys() {
        use crate::key::{KeyMsg, KeyType};

        struct KeyModel {
            special_keys: Vec<KeyType>,
        }

        impl Model for KeyModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(key) = msg.downcast_ref::<KeyMsg>()
                    && key.key_type != KeyType::Runes
                {
                    self.special_keys.push(key.key_type);
                }
                None
            }
            fn view(&self) -> String {
                format!("Keys: {:?}", self.special_keys)
            }
        }

        let mut sim = ProgramSimulator::new(KeyModel {
            special_keys: Vec::new(),
        });
        sim.init();
        sim.sim_key_type(KeyType::Enter);
        sim.sim_key_type(KeyType::Esc);
        sim.sim_key_type(KeyType::Tab);
        sim.sim_key_type(KeyType::Up);
        sim.sim_key_type(KeyType::Down);
        sim.run_until_empty();

        assert_eq!(
            sim.model().special_keys,
            vec![
                KeyType::Enter,
                KeyType::Esc,
                KeyType::Tab,
                KeyType::Up,
                KeyType::Down,
            ]
        );
    }

    #[test]
    fn test_sim_mouse_sends_clicks() {
        use crate::mouse::{MouseAction, MouseButton, MouseMsg};

        struct MouseModel {
            clicks: Vec<(u16, u16)>,
        }

        impl Model for MouseModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(mouse) = msg.downcast_ref::<MouseMsg>()
                    && mouse.button == MouseButton::Left
                    && mouse.action == MouseAction::Press
                {
                    self.clicks.push((mouse.x, mouse.y));
                }
                None
            }
            fn view(&self) -> String {
                format!("Clicks: {:?}", self.clicks)
            }
        }

        let mut sim = ProgramSimulator::new(MouseModel { clicks: Vec::new() });
        sim.init();
        sim.sim_mouse(10, 5, MouseButton::Left, MouseAction::Press);
        sim.sim_mouse(20, 15, MouseButton::Left, MouseAction::Press);
        sim.run_until_empty();

        assert_eq!(sim.model().clicks, vec![(10, 5), (20, 15)]);
    }

    #[test]
    fn test_sim_resize_sends_dimensions() {
        use crate::message::WindowSizeMsg;

        struct SizeModel {
            width: u16,
            height: u16,
        }

        impl Model for SizeModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
                    self.width = size.width;
                    self.height = size.height;
                }
                None
            }
            fn view(&self) -> String {
                format!("{}x{}", self.width, self.height)
            }
        }

        let mut sim = ProgramSimulator::new(SizeModel {
            width: 0,
            height: 0,
        });
        sim.init();
        sim.sim_resize(120, 40);
        sim.run_until_empty();

        assert_eq!(sim.model().width, 120);
        assert_eq!(sim.model().height, 40);
    }

    #[test]
    fn test_sim_paste_sends_text() {
        use crate::key::KeyMsg;

        struct PasteModel {
            pasted: String,
        }

        impl Model for PasteModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(key) = msg.downcast_ref::<KeyMsg>()
                    && key.paste
                {
                    self.pasted = key.runes.iter().collect();
                }
                None
            }
            fn view(&self) -> String {
                format!("Pasted: {}", self.pasted)
            }
        }

        let mut sim = ProgramSimulator::new(PasteModel {
            pasted: String::new(),
        });
        sim.init();
        sim.sim_paste("Hello, World!");
        sim.run_until_empty();

        assert_eq!(sim.model().pasted, "Hello, World!");
    }

    // ========================================================================
    // Sequence Command Tests (bd-ikfq)
    // ========================================================================

    #[test]
    fn test_simulator_sequence_command() {
        use crate::sequence;

        struct SequenceTrigger;
        #[derive(Clone, Copy)]
        struct Append(char);

        struct SequenceModel {
            chars: String,
        }

        impl Model for SequenceModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if msg.is::<SequenceTrigger>() {
                    return sequence(vec![
                        Some(crate::Cmd::new(|| Message::new(Append('A')))),
                        Some(crate::Cmd::new(|| Message::new(Append('B')))),
                        Some(crate::Cmd::new(|| Message::new(Append('C')))),
                    ]);
                }
                if let Some(Append(c)) = msg.downcast_ref::<Append>() {
                    self.chars.push(*c);
                }
                None
            }
            fn view(&self) -> String {
                self.chars.clone()
            }
        }

        let mut sim = ProgramSimulator::new(SequenceModel {
            chars: String::new(),
        });
        sim.init();
        sim.send(Message::new(SequenceTrigger));

        // Step to get the sequence command
        let cmd = sim.step();
        assert!(cmd.is_some());

        // Execute and send the result
        if let Some(msg) = cmd.unwrap().execute() {
            sim.send(msg);
        }

        sim.run_until_empty();

        // Sequence should execute in order: A, B, C
        assert_eq!(sim.model().chars, "ABC");
    }

    // ========================================================================
    // Edge Case Tests (bd-ikfq)
    // ========================================================================

    #[test]
    fn test_empty_batch_does_not_panic() {
        use crate::batch;

        struct EmptyBatchModel;

        impl Model for EmptyBatchModel {
            fn init(&self) -> Option<crate::Cmd> {
                // Return an empty batch
                batch(vec![])
            }
            fn update(&mut self, _: Message) -> Option<crate::Cmd> {
                None
            }
            fn view(&self) -> String {
                "ok".to_string()
            }
        }

        let mut sim = ProgramSimulator::new(EmptyBatchModel);
        let cmd = sim.init();

        // Should handle empty batch gracefully
        if let Some(c) = cmd {
            let msg = c.execute();
            if let Some(m) = msg {
                sim.send(m);
                sim.run_until_empty();
            }
        }

        assert_eq!(sim.last_view(), Some("ok"));
    }

    #[test]
    fn test_recursive_updates_bounded() {
        // Model that spawns new messages from update
        struct RecursiveModel {
            count: usize,
        }

        impl Model for RecursiveModel {
            fn init(&self) -> Option<crate::Cmd> {
                None
            }
            fn update(&mut self, msg: Message) -> Option<crate::Cmd> {
                if let Some(&n) = msg.downcast_ref::<usize>() {
                    self.count += 1;
                    if n > 0 {
                        // Spawn more messages
                        return Some(crate::Cmd::new(move || Message::new(n - 1)));
                    }
                }
                None
            }
            fn view(&self) -> String {
                format!("Count: {}", self.count)
            }
        }

        let mut sim = ProgramSimulator::new(RecursiveModel { count: 0 });
        sim.init();
        sim.send(Message::new(100usize)); // Will spawn 100 recursive messages

        let processed = sim.run_until_empty();

        // Should process all but stay bounded by MAX_ITERATIONS
        assert!(processed <= 1000);
        assert_eq!(sim.model().count, 101); // Initial + 100 recursive
    }

    #[test]
    fn test_large_message_queue() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // Queue many messages
        for i in 0_i32..500 {
            sim.send(Message::new(i));
        }

        let processed = sim.run_until_empty();

        assert_eq!(processed, 500);
        // Sum of 0..500 = 499*500/2 = 124750
        assert_eq!(sim.model().value, 124750);
    }

    #[test]
    fn test_model_mut_allows_direct_modification() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // Directly modify the model
        sim.model_mut().value = 999;

        assert_eq!(sim.model().value, 999);
    }

    #[test]
    fn test_step_without_messages_returns_none() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        // No messages in queue
        let cmd = sim.step();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_views_accumulate() {
        let model = TrackingModel::new();
        let mut sim = ProgramSimulator::new(model);
        sim.init();

        assert_eq!(sim.views().len(), 1);

        sim.send(Message::new(1));
        sim.step();
        assert_eq!(sim.views().len(), 2);

        sim.send(Message::new(2));
        sim.step();
        assert_eq!(sim.views().len(), 3);

        // Views should show progression
        assert_eq!(sim.views()[0], "Value: 0");
        assert_eq!(sim.views()[1], "Value: 1");
        assert_eq!(sim.views()[2], "Value: 3");
    }
}
