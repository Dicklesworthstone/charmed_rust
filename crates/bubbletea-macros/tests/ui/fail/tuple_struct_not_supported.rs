//! Test that derive(Model) fails on tuple structs.

use bubbletea::Model;

#[derive(Model)]
struct TupleStruct(i32, String);

fn main() {}
