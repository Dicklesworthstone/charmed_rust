//! The program installs a terminal-restoring panic hook while it runs; the
//! user's own hook must be back in place afterwards. Kept in its own test
//! binary because panic hooks are process-global.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use bubbletea::{Cmd, Message, Model, Program, quit};

/// Serialize tests in this binary because `std::panic::set_hook` is process-global.
static HOOK_TEST_LOCK: Mutex<()> = Mutex::new(());

struct QuitImmediately;

impl Model for QuitImmediately {
    fn init(&self) -> Option<Cmd> {
        Some(quit())
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        String::new()
    }
}

#[test]
fn user_panic_hook_survives_program_run() {
    let _guard = HOOK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    std::panic::set_hook(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    Program::new(QuitImmediately)
        .with_input(std::io::Cursor::new(Vec::new()))
        .with_output(Vec::new())
        .run()
        .expect("program runs");

    let result = std::panic::catch_unwind(|| panic!("after the program"));
    assert!(result.is_err());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "user hook should still be installed"
    );
    let _ = std::panic::take_hook();
}

#[test]
fn repeated_runs_do_not_nest_panic_hooks() {
    let _guard = HOOK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    std::panic::set_hook(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    // With closure-wrapping restore, repeated runs built an unbounded recursive
    // panic hook chain and leaked a closure per run. Restoring the original hook
    // verbatim keeps stack depth and allocations O(1) (#99).
    for _ in 0..200 {
        Program::new(QuitImmediately)
            .with_custom_io()
            .with_fps(120)
            .with_output(Vec::new())
            .run()
            .expect("program runs");
    }

    let result = std::panic::catch_unwind(|| panic!("after 200 runs"));
    assert!(result.is_err());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "user hook should be invoked once without nested wrappers"
    );
    let _ = std::panic::take_hook();
}
