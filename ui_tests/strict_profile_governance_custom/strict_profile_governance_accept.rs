// A selected strict profile still accepts a justified allow of an ordinary
// floor lint, a stronger level for a governance lint, and a consumer's own
// group forbid next to derive output that allows a member of that group.
#![forbid(nonstandard_style)]
#![deny(unjustified_allow)]

// Bounded fixture input: callers always pass a non-empty slice.
#[allow(no_unwrap)]
fn first(values: &[u32]) -> u32 {
    *values.first().unwrap()
}

#[derive(serde::Serialize)]
struct Reading {
    celsius_milli: i64,
}

fn main() {
    let _ = first(&[1]);
    let _ = Reading { celsius_milli: 1 };
}
