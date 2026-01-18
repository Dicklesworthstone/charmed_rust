//! Program simulator for testing lifecycle without a real terminal.
//!
//! This module provides a way to test Model implementations without
//! requiring a real terminal, enabling unit tests for the Elm Architecture.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::command::Cmd;
use crate::message::{Message, QuitMsg};
use crate::Model;

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
    pub fn run_until_empty(&mut self) -> usize {
        let mut processed = 0;
        while !self.input_queue.is_empty() && !self.stats.quit_requested {
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
}
