//! Runtime tests for state change detection behavior.
//!
//! These tests verify that the generated state change detection code
//! correctly identifies when model state has changed.

use bubbletea::{Cmd, Message, Model};

/// Helper to create approximate float equality
fn float_eq(a: &f64, b: &f64) -> bool {
    (a - b).abs() < 0.01
}

#[derive(Model)]
struct TestModel {
    #[state]
    count: i32,

    #[state]
    name: String,

    // Not tracked
    internal: u64,
}

impl TestModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("{}: {}", self.name, self.count)
    }
}

#[test]
fn test_no_change_detected_when_unchanged() {
    let model = TestModel {
        count: 0,
        name: "test".to_string(),
        internal: 0,
    };

    let snapshot = model.__snapshot_state();
    assert!(!model.__state_changed(&snapshot));
}

#[test]
fn test_change_detected_for_int() {
    let mut model = TestModel {
        count: 0,
        name: "test".to_string(),
        internal: 0,
    };

    let snapshot = model.__snapshot_state();
    model.count = 1;
    assert!(model.__state_changed(&snapshot));
}

#[test]
fn test_change_detected_for_string() {
    let mut model = TestModel {
        count: 0,
        name: "test".to_string(),
        internal: 0,
    };

    let snapshot = model.__snapshot_state();
    model.name = "changed".to_string();
    assert!(model.__state_changed(&snapshot));
}

#[test]
fn test_untracked_field_does_not_trigger_change() {
    let mut model = TestModel {
        count: 0,
        name: "test".to_string(),
        internal: 0,
    };

    let snapshot = model.__snapshot_state();
    model.internal = 999;
    assert!(!model.__state_changed(&snapshot));
}

#[test]
fn test_multiple_snapshots_work_correctly() {
    let mut model = TestModel {
        count: 0,
        name: "test".to_string(),
        internal: 0,
    };

    let snapshot1 = model.__snapshot_state();
    model.count = 1;
    let snapshot2 = model.__snapshot_state();
    model.count = 2;

    assert!(model.__state_changed(&snapshot1));
    assert!(model.__state_changed(&snapshot2));
    assert!(!model.__state_changed(&model.__snapshot_state()));
}

// Test with custom equality function
#[derive(Model)]
struct CustomEqModel {
    #[state(eq = "float_eq")]
    progress: f64,
}

impl CustomEqModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("{:.2}", self.progress)
    }
}

#[test]
fn test_custom_equality_within_tolerance() {
    let mut model = CustomEqModel { progress: 0.5 };

    let snapshot = model.__snapshot_state();
    model.progress = 0.505; // Within 0.01 tolerance
    assert!(
        !model.__state_changed(&snapshot),
        "Small change should be ignored"
    );
}

#[test]
fn test_custom_equality_beyond_tolerance() {
    let mut model = CustomEqModel { progress: 0.5 };

    let snapshot = model.__snapshot_state();
    model.progress = 0.6; // Beyond 0.01 tolerance
    assert!(
        model.__state_changed(&snapshot),
        "Large change should be detected"
    );
}

// Test with skip attribute
#[derive(Model)]
struct SkipModel {
    #[state]
    tracked: i32,

    #[state(skip)]
    skipped: i32,
}

impl SkipModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        format!("{}", self.tracked)
    }
}

#[test]
fn test_skipped_field_ignored() {
    let mut model = SkipModel {
        tracked: 0,
        skipped: 0,
    };

    let snapshot = model.__snapshot_state();
    model.skipped = 100;
    assert!(
        !model.__state_changed(&snapshot),
        "Skipped field should not trigger change"
    );

    model.tracked = 100;
    assert!(
        model.__state_changed(&snapshot),
        "Tracked field should trigger change"
    );
}

// Test with no state fields
#[derive(Model)]
struct NoStateModel {
    data: String,
}

impl NoStateModel {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        self.data.clone()
    }
}

#[test]
fn test_no_state_fields_always_returns_false() {
    let mut model = NoStateModel {
        data: "test".to_string(),
    };

    let snapshot = model.__snapshot_state();
    model.data = "changed".to_string();
    assert!(
        !model.__state_changed(&snapshot),
        "Model with no #[state] fields should never report changes"
    );
}

// Test with generic type
#[derive(Model)]
struct GenericModel<T: Clone + PartialEq + Send + 'static> {
    #[state]
    value: T,
}

impl<T: Clone + PartialEq + Send + 'static> GenericModel<T> {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, _msg: Message) -> Option<Cmd> {
        None
    }

    fn view(&self) -> String {
        "generic".to_string()
    }
}

#[test]
fn test_generic_model_state_detection() {
    let mut model: GenericModel<i32> = GenericModel { value: 42 };

    let snapshot = model.__snapshot_state();
    assert!(!model.__state_changed(&snapshot));

    model.value = 100;
    assert!(model.__state_changed(&snapshot));
}

#[test]
fn test_generic_model_with_string() {
    let mut model: GenericModel<String> = GenericModel {
        value: "hello".to_string(),
    };

    let snapshot = model.__snapshot_state();
    model.value = "world".to_string();
    assert!(model.__state_changed(&snapshot));
}
