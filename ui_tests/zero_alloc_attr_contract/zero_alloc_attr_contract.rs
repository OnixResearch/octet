// zero_alloc_attr_contract: functions marked with #[zero_alloc] that allocate

#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(unused)]

#[tigerstyle::zero_alloc]
fn makes_vec() {
    let _v: Vec<u8> = Vec::new();
}

#[tigerstyle::zero_alloc]
fn makes_box() {
    let _b: Box<u64> = Box::new(42u64);
}

#[tigerstyle::zero_alloc]
fn makes_string() {
    let _s = String::new();
}

#[tigerstyle::zero_alloc]
fn makes_string_from() {
    let _s = String::from("hello");
}

#[tigerstyle::zero_alloc]
fn makes_to_string() {
    let _s = "hello".to_string();
}

// format!() expands to a macro that ultimately produces a String
#[tigerstyle::zero_alloc]
fn makes_format() {
    let _s = format!("value: {}", 42);
}

// Array lengths, repeat counts, and statement-level items are nested bodies;
// walking them must not ICE, and later allocations are still reported.
#[tigerstyle::zero_alloc]
fn makes_vec_after_nested_bodies() {
    const K: u8 = 1;
    let _buf: [u8; 4] = [K; 4];
    let _v: Vec<u8> = Vec::new();
}

fn main() {}