#![warn(type_accumulation)]

pub mod crowded {
    pub struct Thing;

    impl Thing { pub fn a(&self) {} }
    impl Thing { pub fn b(&self) {} }
    impl Thing { pub fn c(&self) {} }
    impl Thing { pub fn d(&self) {} }
    impl Thing { pub fn e(&self) {} }
}

// File-backed module with 13 public items (limit: 12).
#[path = "type_accumulation/src/filebacked.rs"]
pub mod filebacked;

// Public fields, impl members, nested-module contents, re-export stems, and comments are not
// the module's own items: this module exposes 4 public items (limit: 12).
pub mod members_are_not_module_items {
    // pub pub pub pub pub pub pub pub pub pub pub pub pub
    pub struct Settings {
        pub first: u8,
        pub second: u8,
        pub third: u8,
        pub fourth: u8,
    }

    impl Settings {
        pub fn first(&self) -> u8 { self.first }
        pub fn second(&self) -> u8 { self.second }
        pub fn third(&self) -> u8 { self.third }
        pub fn fourth(&self) -> u8 { self.fourth }
    }

    pub mod nested {
        pub fn one() {}
        pub fn two() {}
    }

    pub use nested::{one, two};
}

fn main() {}
