fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn guarded_narrow(x: u64) -> u32 {
    // Should NOT warn: bounded by if guard with MAX
    if x <= u32::MAX as u64 {
        x as u32
    } else {
        0
    }
}

fn widening(x: u32) -> u64 {
    // Should NOT warn: widening cast
    x as u64
}

fn main() {
    bare_narrow(100);
    guarded_narrow(100);
    widening(100);
}
