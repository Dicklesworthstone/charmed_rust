//! Spinner shown while a blocking action runs.
//!
//! Port of Go huh's `spinner` subpackage: display an animated spinner and a
//! title while some work happens, then clear the line and hand back the
//! work's result.
//!
//! # Example
//!
//! ```rust,no_run
//! use huh::spinner::Spinner;
//!
//! let answer = Spinner::new()
//!     .title("Computing the answer...")
//!     .run(|| {
//!         std::thread::sleep(std::time::Duration::from_secs(2));
//!         42
//!     })
//!     .expect("not cancelled");
//! assert_eq!(answer, 42);
//! ```
//!
//! In accessible mode, or when stdout is not a terminal, no animation is
//! drawn: the title is printed once and the action runs in the foreground.

use std::io::{self, IsTerminal, Write};
use std::sync::{Arc, Mutex, PoisonError};

use bubbles::spinner::{Spinner as SpinnerKind, SpinnerModel, spinners};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program};
use lipgloss::Style;

use crate::{FormError, Result};

/// Sent when the action finishes.
struct ActionDoneMsg;

/// A spinner that runs an action (Go: `spinner.New()`).
#[derive(Debug, Clone)]
pub struct Spinner {
    title: String,
    kind: SpinnerKind,
    style: Style,
    title_style: Style,
    accessible: bool,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Creates a spinner with the dots animation and a "Loading..." title.
    pub fn new() -> Self {
        Self {
            title: "Loading...".to_string(),
            kind: spinners::dot(),
            style: Style::new().foreground("#F780E2"),
            title_style: Style::new(),
            accessible: false,
        }
    }

    /// Sets the title shown next to the spinner.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the animation (Go: `Type`), e.g. `bubbles::spinner::spinners::line()`.
    #[must_use]
    pub fn spinner_type(mut self, kind: SpinnerKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the style of the spinner glyphs.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the style of the title.
    #[must_use]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Enables accessible mode: print the title once instead of animating.
    #[must_use]
    pub fn accessible(mut self, accessible: bool) -> Self {
        self.accessible = accessible;
        self
    }

    /// Runs `action` while showing the spinner and returns its result
    /// (Go: `Action(...).Run()`).
    ///
    /// # Errors
    ///
    /// Returns [`FormError::UserAborted`] if the user presses Ctrl+C or Esc
    /// before the action completes (the action keeps running in the
    /// background), or [`FormError::Io`] if the terminal fails.
    pub fn run<T, F>(self, action: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        if self.accessible || !io::stdout().is_terminal() {
            return self.run_accessible(&mut io::stdout(), action);
        }

        let slot = Arc::new(Mutex::new(None));
        let ui = SpinnerUi::new(&self, Arc::clone(&slot), action);
        let ui = Program::new(ui)
            .run()
            .map_err(|e| FormError::Io(e.to_string()))?;
        if !ui.done {
            return Err(FormError::UserAborted);
        }
        take(&slot).ok_or(FormError::UserAborted)
    }

    /// Prints the title to `w`, then runs `action` in the foreground.
    ///
    /// # Errors
    ///
    /// Returns [`FormError::Io`] if writing the title fails.
    pub fn run_accessible<T, F>(&self, w: &mut dyn Write, action: F) -> Result<T>
    where
        F: FnOnce() -> T,
    {
        writeln!(w, "{}", self.title).map_err(|e| FormError::Io(e.to_string()))?;
        w.flush().map_err(|e| FormError::Io(e.to_string()))?;
        Ok(action())
    }
}

fn take<T>(slot: &Mutex<Option<T>>) -> Option<T> {
    slot.lock().unwrap_or_else(PoisonError::into_inner).take()
}

type Action = Box<dyn FnOnce() + Send>;

/// The bubbletea model behind [`Spinner::run`].
struct SpinnerUi {
    spinner: SpinnerModel,
    title: String,
    title_style: Style,
    action: Mutex<Option<Action>>,
    done: bool,
}

impl SpinnerUi {
    fn new<T, F>(config: &Spinner, slot: Arc<Mutex<Option<T>>>, action: F) -> Self
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let action: Action = Box::new(move || {
            let out = action();
            *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(out);
        });
        Self {
            spinner: SpinnerModel::with_spinner(config.kind.clone()).style(config.style.clone()),
            title: config.title.clone(),
            title_style: config.title_style.clone(),
            action: Mutex::new(Some(action)),
            done: false,
        }
    }
}

impl Model for SpinnerUi {
    fn init(&self) -> Option<Cmd> {
        let tick = self.spinner.tick();
        let action = self
            .action
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let run = action.map(|action| {
            Cmd::new(move || {
                action();
                Message::new(ActionDoneMsg)
            })
        });
        bubbletea::batch(vec![Some(Cmd::new(move || tick)), run])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if msg.is::<ActionDoneMsg>() {
            self.done = true;
            return Some(bubbletea::quit());
        }
        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && matches!(key.key_type, KeyType::CtrlC | KeyType::Esc)
        {
            return Some(bubbletea::quit());
        }
        self.spinner.update(msg)
    }

    fn view(&self) -> String {
        if self.done {
            return String::new();
        }
        format!(
            "{} {}",
            self.spinner.view(),
            self.title_style.render(&self.title)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessible_prints_title_and_returns_result() {
        let mut out = Vec::new();
        let result = Spinner::new()
            .title("Working")
            .run_accessible(&mut out, || 7)
            .expect("runs");
        assert_eq!(result, 7);
        assert_eq!(String::from_utf8(out).unwrap(), "Working\n");
    }

    #[test]
    fn run_without_terminal_falls_back_to_foreground() {
        // Test harness stdout is not a terminal.
        let result = Spinner::new().title("x").run(|| "done").expect("runs");
        assert_eq!(result, "done");
    }

    #[test]
    fn model_runs_action_and_quits() {
        let slot = Arc::new(Mutex::new(None));
        let mut ui = SpinnerUi::new(&Spinner::new().title("T"), Arc::clone(&slot), || 5);
        assert!(ui.view().ends_with(" T"), "{}", ui.view());

        // init batches the first tick and the action.
        let batch = ui.init().expect("init cmd").execute().expect("batch msg");
        let cmds = batch
            .downcast::<bubbletea::message::BatchMsg>()
            .expect("batch")
            .0;
        assert_eq!(cmds.len(), 2);
        let mut msgs: Vec<Message> = cmds.into_iter().filter_map(Cmd::execute).collect();
        let done_idx = msgs
            .iter()
            .position(|m| m.is::<ActionDoneMsg>())
            .expect("action done message");
        assert_eq!(take(&slot), Some(5));

        let quit = ui.update(msgs.remove(done_idx)).expect("quit cmd");
        assert!(quit.execute().is_some_and(|m| m.is::<bubbletea::QuitMsg>()));
        assert!(ui.done);
        assert_eq!(ui.view(), "");
    }

    #[test]
    fn escape_quits_without_finishing() {
        let slot: Arc<Mutex<Option<()>>> = Arc::new(Mutex::new(None));
        let mut ui = SpinnerUi::new(&Spinner::new(), slot, || ());
        let cmd = ui
            .update(Message::new(KeyMsg::from_type(KeyType::Esc)))
            .expect("quit");
        assert!(cmd.execute().is_some_and(|m| m.is::<bubbletea::QuitMsg>()));
        assert!(!ui.done);
    }
}
