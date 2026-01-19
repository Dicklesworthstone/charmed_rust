//! Test that derive(Model) fails on unions.

use bubbletea::Model;

#[derive(Model)]
union NotAStruct {
    int_val: i32,
    float_val: f32,
}

fn main() {}
