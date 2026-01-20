//! End-to-end tests for async Program lifecycle.
//!
//! These tests verify the complete async program lifecycle including:
//! - Async command execution within model context
//! - Graceful shutdown coordination
//! - Concurrent command handling
//! - Message ordering guarantees

#![cfg(feature = "async")]
// Test simulator doesn't need Send futures - it's single-threaded test infrastructure
#![allow(clippy::future_not_send)]
// Test helper methods don't need const
#![allow(clippy::missing_const_for_fn)]

use bubbletea::{AsyncCmd, Cmd, Message, Model, quit};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

// =============================================================================
// Test Models
// =============================================================================

/// A simple counter model that tracks async operations.
struct AsyncCounter {
    count: i32,
    async_ops_started: Arc<AtomicUsize>,
    async_ops_completed: Arc<AtomicUsize>,
}

impl AsyncCounter {
    fn new() -> Self {
        Self {
            count: 0,
            async_ops_started: Arc::new(AtomicUsize::new(0)),
            async_ops_completed: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[derive(Clone, Debug)]
enum CounterMsg {
    Increment,
    Decrement,
    AsyncIncrement,
    AsyncDone(i32),
    Quit,
}

impl Model for AsyncCounter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(counter_msg) = msg.downcast_ref::<CounterMsg>() {
            match counter_msg {
                CounterMsg::Increment => {
                    self.count += 1;
                    None
                }
                CounterMsg::Decrement => {
                    self.count -= 1;
                    None
                }
                CounterMsg::AsyncIncrement => {
                    self.async_ops_started.fetch_add(1, Ordering::SeqCst);
                    let completed = self.async_ops_completed.clone();
                    Some(Cmd::new(move || {
                        completed.fetch_add(1, Ordering::SeqCst);
                        Message::new(CounterMsg::AsyncDone(1))
                    }))
                }
                CounterMsg::AsyncDone(delta) => {
                    self.count += delta;
                    None
                }
                CounterMsg::Quit => Some(quit()),
            }
        } else {
            None
        }
    }

    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}

/// Model that tracks message ordering.
struct OrderedMessageModel {
    received_order: Vec<usize>,
}

#[derive(Clone, Debug)]
struct OrderedMsg(usize);

impl Model for OrderedMessageModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(ordered) = msg.downcast_ref::<OrderedMsg>() {
            self.received_order.push(ordered.0);
        }
        None
    }

    fn view(&self) -> String {
        format!("Received: {:?}", self.received_order)
    }
}

/// Model for testing graceful shutdown.
struct ShutdownModel {
    shutdown_started: Arc<AtomicBool>,
    cleanup_completed: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // Realistic enum - not all variants used in tests
enum ShutdownMsg {
    StartShutdown,
    CleanupDone,
}

impl Model for ShutdownModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(shutdown_msg) = msg.downcast_ref::<ShutdownMsg>() {
            match shutdown_msg {
                ShutdownMsg::StartShutdown => {
                    self.shutdown_started.store(true, Ordering::SeqCst);
                    Some(quit())
                }
                ShutdownMsg::CleanupDone => {
                    self.cleanup_completed.store(true, Ordering::SeqCst);
                    None
                }
            }
        } else {
            None
        }
    }

    fn view(&self) -> String {
        "Shutdown Model".to_string()
    }
}

// =============================================================================
// Async Simulator for E2E Testing
// =============================================================================

/// An async program simulator for testing lifecycle without a terminal.
struct AsyncProgramSimulator<M: Model> {
    model: M,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    initialized: bool,
    quit_requested: bool,
}

impl<M: Model> AsyncProgramSimulator<M> {
    fn new(model: M) -> Self {
        let (tx, rx) = mpsc::channel(256);
        Self {
            model,
            tx,
            rx,
            initialized: false,
            quit_requested: false,
        }
    }

    /// Initialize the model.
    async fn init(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        if let Some(cmd) = self.model.init() {
            self.execute_command(cmd).await;
        }
    }

    /// Send a message to be processed.
    async fn send(&self, msg: Message) {
        let _ = self.tx.send(msg).await;
    }

    /// Process one pending message.
    async fn step(&mut self) -> bool {
        if !self.initialized {
            self.init().await;
        }

        if let Ok(msg) = self.rx.try_recv() {
            if msg.is::<bubbletea::QuitMsg>() {
                self.quit_requested = true;
                return false;
            }

            if let Some(cmd) = self.model.update(msg) {
                self.execute_command(cmd).await;
            }
            true
        } else {
            false
        }
    }

    /// Process all pending messages.
    async fn run_until_empty(&mut self) {
        if !self.initialized {
            self.init().await;
        }

        while !self.quit_requested {
            if !self.step().await {
                // No message processed and not quit - try to receive
                tokio::select! {
                    msg = self.rx.recv() => {
                        if let Some(msg) = msg {
                            if msg.is::<bubbletea::QuitMsg>() {
                                self.quit_requested = true;
                                break;
                            }
                            if let Some(cmd) = self.model.update(msg) {
                                self.execute_command(cmd).await;
                            }
                        } else {
                            break;
                        }
                    }
                    () = tokio::time::sleep(Duration::from_millis(10)) => {
                        break;
                    }
                }
            }
        }
    }

    /// Execute a command and queue resulting message.
    #[allow(clippy::unused_async)] // async kept for API consistency
    async fn execute_command(&self, cmd: Cmd) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Some(msg) = cmd.execute() {
                let _ = tx.send(msg).await;
            }
        });
    }

    fn model(&self) -> &M {
        &self.model
    }

    fn is_quit(&self) -> bool {
        self.quit_requested
    }
}

// =============================================================================
// Lifecycle Tests
// =============================================================================

mod lifecycle_tests {
    use super::*;

    #[tokio::test]
    async fn test_program_initializes_correctly() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        assert!(!sim.initialized);
        sim.init().await;
        assert!(sim.initialized);
        assert_eq!(sim.model().count, 0);
    }

    #[tokio::test]
    async fn test_messages_processed_correctly() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;
        sim.send(Message::new(CounterMsg::Increment)).await;
        sim.send(Message::new(CounterMsg::Increment)).await;
        sim.send(Message::new(CounterMsg::Decrement)).await;

        sim.run_until_empty().await;

        assert_eq!(sim.model().count, 1); // 0 + 1 + 1 - 1 = 1
    }

    #[tokio::test]
    async fn test_quit_triggers_shutdown() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;
        sim.send(Message::new(CounterMsg::Increment)).await;
        sim.send(Message::new(CounterMsg::Quit)).await;

        sim.run_until_empty().await;

        assert!(sim.is_quit());
        // Increment processed before quit
        assert_eq!(sim.model().count, 1);
    }

    #[tokio::test]
    async fn test_view_renders_correctly() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;
        assert_eq!(sim.model().view(), "Count: 0");

        sim.send(Message::new(CounterMsg::Increment)).await;
        sim.run_until_empty().await;
        assert_eq!(sim.model().view(), "Count: 1");
    }
}

// =============================================================================
// Async Command Integration Tests
// =============================================================================

mod async_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_async_command_executes() {
        let model = AsyncCounter::new();
        let ops_started = model.async_ops_started.clone();
        let ops_completed = model.async_ops_completed.clone();

        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;
        sim.send(Message::new(CounterMsg::AsyncIncrement)).await;

        // Process the message that triggers async command
        sim.step().await;

        // Wait for async command to complete
        sleep(Duration::from_millis(50)).await;

        // Process the AsyncDone message
        sim.run_until_empty().await;

        assert_eq!(ops_started.load(Ordering::SeqCst), 1);
        assert_eq!(ops_completed.load(Ordering::SeqCst), 1);
        assert_eq!(sim.model().count, 1);
    }

    #[tokio::test]
    async fn test_multiple_async_commands() {
        let model = AsyncCounter::new();
        let ops_started = model.async_ops_started.clone();
        let ops_completed = model.async_ops_completed.clone();

        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Send multiple async increment commands
        for _ in 0..5 {
            sim.send(Message::new(CounterMsg::AsyncIncrement)).await;
        }

        // Process all messages
        for _ in 0..5 {
            sim.step().await;
        }

        // Wait for all async commands to complete
        sleep(Duration::from_millis(100)).await;

        // Process all AsyncDone messages
        sim.run_until_empty().await;

        assert_eq!(ops_started.load(Ordering::SeqCst), 5);
        assert_eq!(ops_completed.load(Ordering::SeqCst), 5);
        assert_eq!(sim.model().count, 5);
    }

    #[tokio::test]
    async fn test_mixed_sync_async_commands() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Mix sync and async
        sim.send(Message::new(CounterMsg::Increment)).await;
        sim.send(Message::new(CounterMsg::AsyncIncrement)).await;
        sim.send(Message::new(CounterMsg::Increment)).await;

        // Process all
        for _ in 0..3 {
            sim.step().await;
        }

        // Wait for async
        sleep(Duration::from_millis(50)).await;

        sim.run_until_empty().await;

        // 2 sync + 1 async = 3
        assert_eq!(sim.model().count, 3);
    }
}

// =============================================================================
// Message Ordering Tests
// =============================================================================

mod ordering_tests {
    use super::*;

    #[tokio::test]
    async fn test_messages_maintain_order() {
        let model = OrderedMessageModel {
            received_order: vec![],
        };
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Send messages in order
        for i in 1..=5 {
            sim.send(Message::new(OrderedMsg(i))).await;
        }

        sim.run_until_empty().await;

        // Verify order preserved
        assert_eq!(sim.model().received_order, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_many_messages_maintain_order() {
        let model = OrderedMessageModel {
            received_order: vec![],
        };
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Send many messages
        for i in 1..=100 {
            sim.send(Message::new(OrderedMsg(i))).await;
        }

        sim.run_until_empty().await;

        // Verify all received in order
        let expected: Vec<usize> = (1..=100).collect();
        assert_eq!(sim.model().received_order, expected);
    }
}

// =============================================================================
// Shutdown Tests
// =============================================================================

mod shutdown_tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_shutdown() {
        let shutdown_started = Arc::new(AtomicBool::new(false));
        let cleanup_completed = Arc::new(AtomicBool::new(false));

        let model = ShutdownModel {
            shutdown_started: shutdown_started.clone(),
            cleanup_completed: cleanup_completed.clone(),
        };
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;
        sim.send(Message::new(ShutdownMsg::StartShutdown)).await;

        sim.run_until_empty().await;

        assert!(shutdown_started.load(Ordering::SeqCst));
        assert!(sim.is_quit());
    }

    #[tokio::test]
    async fn test_shutdown_with_pending_messages() {
        let model = AsyncCounter::new();
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Process some messages first
        for _ in 0..5 {
            sim.send(Message::new(CounterMsg::Increment)).await;
        }
        sim.run_until_empty().await;
        assert_eq!(sim.model().count, 5);

        // Now send quit - verify it triggers shutdown
        sim.send(Message::new(CounterMsg::Quit)).await;
        sim.run_until_empty().await;

        assert!(sim.is_quit());
        // Count should still be 5 (quit processed, no more increments)
        assert_eq!(sim.model().count, 5);
    }
}

// =============================================================================
// Concurrent Command Tests
// =============================================================================

mod concurrent_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_async_commands_complete() {
        struct ConcurrentModel {
            completed: Arc<AtomicUsize>,
        }

        #[derive(Clone)]
        struct StartAsync;
        #[derive(Clone)]
        struct AsyncCompleted;

        impl Model for ConcurrentModel {
            fn init(&self) -> Option<Cmd> {
                None
            }

            fn update(&mut self, msg: Message) -> Option<Cmd> {
                if msg.is::<StartAsync>() {
                    let completed = self.completed.clone();
                    Some(Cmd::new(move || {
                        std::thread::sleep(Duration::from_millis(10));
                        completed.fetch_add(1, Ordering::SeqCst);
                        Message::new(AsyncCompleted)
                    }))
                } else {
                    None
                }
            }

            fn view(&self) -> String {
                String::new()
            }
        }

        let completed = Arc::new(AtomicUsize::new(0));
        let model = ConcurrentModel {
            completed: completed.clone(),
        };
        let mut sim = AsyncProgramSimulator::new(model);

        sim.init().await;

        // Start multiple concurrent operations
        for _ in 0..5 {
            sim.send(Message::new(StartAsync)).await;
        }

        // Process the StartAsync messages
        for _ in 0..5 {
            sim.step().await;
        }

        // Wait for all concurrent operations
        sleep(Duration::from_millis(100)).await;

        // All should complete
        assert_eq!(completed.load(Ordering::SeqCst), 5);
    }
}

// =============================================================================
// AsyncCmd Specific Tests
// =============================================================================

#[cfg(feature = "async")]
mod async_cmd_tests {
    use super::*;

    #[tokio::test]
    async fn test_async_cmd_executes() {
        struct Result(i32);

        let cmd = AsyncCmd::new(|| async { Message::new(Result(42)) });
        let msg = cmd.execute().await.unwrap();
        let result = msg.downcast::<Result>().unwrap();
        assert_eq!(result.0, 42);
    }

    #[tokio::test]
    async fn test_async_cmd_with_delay() {
        struct DelayedResult;

        let start = std::time::Instant::now();
        let cmd = AsyncCmd::new(|| async {
            sleep(Duration::from_millis(50)).await;
            Message::new(DelayedResult)
        });

        let msg = cmd.execute().await.unwrap();
        let elapsed = start.elapsed();

        assert!(msg.is::<DelayedResult>());
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_multiple_async_cmds_run_concurrently() {
        #[allow(dead_code)] // Used as message marker
        struct TaskResult(usize);

        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // Spawn 5 concurrent async commands
        for i in 0..5 {
            let counter = counter.clone();
            let cmd = AsyncCmd::new(move || {
                let counter = counter.clone();
                async move {
                    sleep(Duration::from_millis(10)).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                    Message::new(TaskResult(i))
                }
            });
            handles.push(tokio::spawn(async move { cmd.execute().await }));
        }

        // Wait for all
        for handle in handles {
            let _ = handle.await;
        }

        // All should have completed
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }
}
