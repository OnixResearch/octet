#![no_std]
#![crate_type = "lib"]
#![allow(dead_code, sentinel_fallback)]

// Without std, `TryFromIntError` is only reachable as `core::num::TryFromIntError`.

fn zero_fallback(count: u64) -> u32 {
    u32::try_from(count).unwrap_or(0)
}

fn max_fallback(count: u64) -> u8 {
    u8::try_from(count).unwrap_or(u8::MAX)
}

fn default_fallback(count: u64) -> u16 {
    u16::try_from(count).unwrap_or_default()
}

fn ordinary_result_fallback(input: Result<u32, &'static str>) -> u32 {
    input.unwrap_or(0)
}
