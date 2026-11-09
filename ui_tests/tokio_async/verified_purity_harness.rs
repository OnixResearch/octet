#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::ambient_clock)]

// The crate root sits outside `verified/`, as `src/lib.rs` does in a normal
// layout; only the file-backed module below is a verified module. The fixture
// compiles with `--test`, so harness test functions survive expansion.
#[path = "src/verified/mod.rs"]
mod verified;
