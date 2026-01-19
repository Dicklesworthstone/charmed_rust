//! Test that a generic struct with #[state] derives correctly.

use bubbletea::{Cmd, Message, Model};

#[derive(Model)]
struct GenericWithState<T>
where
    T: Clone + PartialEq + Send + 'static,
{
    #[state]
    value: T,
    metadata: String,
}

impl<T> GenericWithState<T>
where
    T: Clone + PartialEq + Send + 'static,
{
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("Metadata: {}", self.metadata)
    }
}

fn main() {
    let app: GenericWithState<i32> = GenericWithState {
        value: 42,
        metadata: "test".to_string(),
    };

    // Verify Model trait is implemented
    let _ = <GenericWithState<i32> as Model>::view(&app);

    // Verify state tracking methods exist
    let snapshot = app.__snapshot_state();
    let _ = app.__state_changed(&snapshot);

    // Test with a different type
    let string_app: GenericWithState<String> = GenericWithState {
        value: "hello".to_string(),
        metadata: "string test".to_string(),
    };
    let string_snapshot = string_app.__snapshot_state();
    let _ = string_app.__state_changed(&string_snapshot);
}
