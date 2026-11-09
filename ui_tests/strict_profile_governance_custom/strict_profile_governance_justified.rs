// A selected strict profile still accepts a justified allow of an ordinary
// floor lint; only the suppression-governance lints are forbidden.

// Bounded fixture input: callers always pass a non-empty slice.
#[allow(no_unwrap)]
fn first(values: &[u32]) -> u32 {
    *values.first().unwrap()
}

fn main() {
    let _ = first(&[1]);
}
