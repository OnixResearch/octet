// A selected strict profile still accepts a justified allow of an ordinary
// floor lint, a stronger level for a governance lint, a consumer's own group
// forbid next to derive output that allows a member of that group, and a
// `warn` that a later `deny` or `forbid` on the same node raises back.
#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![forbid(nonstandard_style)]
#![deny(unjustified_allow)]
#![warn(tigerstyle::safety)]
#![deny(tigerstyle::safety)]

// Bounded fixture input: callers always pass a non-empty slice.
#[allow(no_unwrap)]
fn first(values: &[u32]) -> u32 {
    *values.first().unwrap()
}

#[derive(serde::Serialize)]
struct Reading {
    celsius_milli: i64,
}

#[warn(tigerstyle::safety)]
#[forbid(tigerstyle::safety)]
mod restored {}

// An exported macro body follows the same rules where it is defined: a
// justified allow of an ordinary floor lint and a `warn` raised back later in
// the same run of attributes are accepted.
#[macro_export]
macro_rules! exported_restored {
    ($item:item) => {
        // Callers only wrap items whose unwraps a bounds check precedes.
        #[allow(tigerstyle::no_unwrap)]
        #[warn(tigerstyle::safety)]
        #[deny(tigerstyle::safety)]
        $item
    };
}

fn main() {
    let _ = first(&[1]);
    let _ = Reading { celsius_milli: 1 };
}
