//! The program installs a terminal-restoring panic hook while it runs; the
//! user's own hook must be back in place afterwards. Kept in its own test
//! binary because panic hooks are process-global.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bubbletea::{Cmd, Message, Model, Program, quit};

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
