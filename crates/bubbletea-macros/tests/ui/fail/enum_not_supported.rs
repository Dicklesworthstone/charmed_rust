//! Test that derive(Model) fails on enums.

use bubbletea::Model;

#[derive(Model)]
enum NotAStruct {
    Variant1,
    Variant2,
}

fn main() {}
