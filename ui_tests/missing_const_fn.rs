#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::missing_const_fn)]

const SCALE_FACTOR: u64 = 2;

fn scale_value(value: u64) -> u64 {
    value * SCALE_FACTOR
}

const fn const_offset(value: u64) -> u64 {
    value
}

fn offset_scaled_value(value: u64) -> u64 {
    const_offset(value) * SCALE_FACTOR
}

struct Doubler;

impl Doubler {
    fn twice(value: u64) -> u64 {
        value * SCALE_FACTOR
    }
}

// Trait methods cannot be `const` (E0379), and helpers that call non-const
// functions are not const-compatible.
trait Scale {
    fn scale(&self, value: u64) -> u64;

    fn halve(&self, value: u64) -> u64 {
        value / SCALE_FACTOR
    }
}

impl Scale for Doubler {
    fn scale(&self, value: u64) -> u64 {
        value * SCALE_FACTOR
    }
}

fn runtime_offset(value: u64) -> u64 {
    std::hint::black_box(value)
}

fn offset_then_scale(value: u64) -> u64 {
    runtime_offset(value) * SCALE_FACTOR
}

fn sum_then_scale(values: &[u64]) -> u64 {
    values.iter().sum::<u64>() * SCALE_FACTOR
}

fn main() {
    let _scaled = scale_value(SCALE_FACTOR);
    let _offset = offset_scaled_value(SCALE_FACTOR);
    let _twice = Doubler::twice(SCALE_FACTOR);
    let _trait_scaled = Doubler.scale(SCALE_FACTOR);
    let _halved = Doubler.halve(SCALE_FACTOR);
    let _runtime = offset_then_scale(SCALE_FACTOR);
    let _sum = sum_then_scale(&[SCALE_FACTOR]);
}
