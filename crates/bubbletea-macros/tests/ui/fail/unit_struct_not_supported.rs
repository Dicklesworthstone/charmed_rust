//! Test that derive(Model) fails on unit structs.

use bubbletea::Model;

#[derive(Model)]
struct UnitStruct;

fn main() {}
