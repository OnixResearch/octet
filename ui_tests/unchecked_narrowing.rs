const SAMPLE_U64: u64 = 100;
const SAMPLE_U32: u32 = 100;

fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn guarded_narrow(x: u64) -> u32 {
    // Should NOT warn: bounded by if guard with MAX
    if x <= u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn unrelated_guard_still_warns(x: u64, y: u64) -> u32 {
    // Should warn: guard proves a different value
    if y <= u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn wrong_bound_still_warns(x: u64) -> u32 {
    // Should warn: guard proves the wrong target bound
    if x <= u16::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn unsafe_then_branch_still_warns(x: u64) -> u32 {
    // Should warn: the cast is in the branch where the bound is false
    if x > u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn safe_else_branch_is_allowed(x: u64) -> u32 {
    // Should NOT warn: the else branch implies x <= u32::MAX
    if x > u32::MAX as u64 {
        u32::MIN
    } else {
        x as u32
    }
}

fn try_from_guard_is_allowed(x: u64) -> u32 {
    // Should NOT warn: exact TryFrom guard proves target range
    if u32::try_from(x).is_ok() {
        x as u32
    } else {
        u32::MIN
    }
}

fn unrelated_try_from_guard_still_warns(x: u64, y: u64) -> u32 {
    // Should warn: TryFrom guard proves a different value
    if u32::try_from(y).is_ok() {
        x as u32
    } else {
        u32::MIN
    }
}

fn widening(x: u32) -> u64 {
    // Should NOT warn: widening cast
    x as u64
}

fn signed_upper_bound_only_still_warns(x: i64) -> u32 {
    // Should warn: negative values pass the upper-bound guard and wrap
    if x <= u32::MAX as i64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn signed_else_upper_bound_only_still_warns(x: i64) -> i32 {
    // Should warn: the else branch only proves `x <= i32::MAX`
    if x > i32::MAX as i64 {
        i32::MIN
    } else {
        x as i32
    }
}

fn signed_both_bounds_allowed(x: i64) -> u32 {
    // Should NOT warn: lower and upper bounds are both proven
    if x >= 0 && x <= u32::MAX as i64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn signed_nested_bounds_allowed(x: i64) -> i32 {
    // Should NOT warn: nested guards prove both target bounds
    if x >= i32::MIN as i64 {
        if x <= i32::MAX as i64 { x as i32 } else { i32::MAX }
    } else {
        i32::MIN
    }
}

fn main() {
    bare_narrow(SAMPLE_U64);
    guarded_narrow(SAMPLE_U64);
    unrelated_guard_still_warns(SAMPLE_U64, SAMPLE_U64);
    wrong_bound_still_warns(SAMPLE_U64);
    unsafe_then_branch_still_warns(SAMPLE_U64);
    safe_else_branch_is_allowed(SAMPLE_U64);
    try_from_guard_is_allowed(SAMPLE_U64);
    unrelated_try_from_guard_still_warns(SAMPLE_U64, SAMPLE_U64);
    widening(SAMPLE_U32);
    signed_upper_bound_only_still_warns(-1);
    signed_else_upper_bound_only_still_warns(-1);
    signed_both_bounds_allowed(-1);
    signed_nested_bounds_allowed(-1);
}
