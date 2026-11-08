fn bare_division(total: u64, count: u64) -> u64 {
    // Should warn: division by variable without zero check
    total / count
}

fn guarded_division(total: u64, count: u64) -> u64 {
    // Should NOT warn: zero guard in enclosing if
    if count > 0 {
        total / count
    } else {
        0
    }
}

fn literal_division(total: u64) -> u64 {
    // Should NOT warn: division by literal
    total / 1000
}

fn zero_branch_division(total: u64, count: u64) -> u64 {
    // Should warn: the then-branch of `count == 0` divides by zero
    if count == 0 {
        return total / count;
    }
    total
}

fn nonzero_guard_else_branch(total: u64, count: u64) -> u64 {
    // Should warn: the else-branch of `count > 0` has `count == 0`
    if count > 0 {
        0
    } else {
        total / count
    }
}

fn guard_on_other_name(total: u64, len: u64, n: u64) -> u64 {
    // Should warn: the guard checks `len`, not `n`
    if len > 0 {
        total / n
    } else {
        0
    }
}

fn assert_on_other_divisor(a: u64, b: u64, c: u64) -> u64 {
    // Should warn: the assert checks `c`, not `b`
    assert!(c > 0);
    a / b
}

fn later_assert(total: u64, count: u64) -> u64 {
    // Should warn: a later assert cannot justify an earlier division
    let quotient = total / count;
    assert!(count > 0);
    quotient
}

fn zero_check_else_branch(total: u64, count: u64) -> u64 {
    // Should NOT warn: the else-branch of `count == 0` has `count != 0`
    if count == 0 {
        0
    } else {
        total / count
    }
}

fn early_return_zero_check(total: u64, count: u64) -> u64 {
    // Should NOT warn: earlier same-block early return proves `count != 0`
    if count == 0 {
        return 0;
    }
    total / count
}

fn mirrored_guard(total: u64, count: u64) -> u64 {
    // Should NOT warn: mirrored `0 != count` guard
    if 0 != count {
        total / count
    } else {
        0
    }
}

fn at_least_one_guard(total: u64, count: u64) -> u64 {
    // Should NOT warn: `count >= 1` proves nonzero
    if count >= 1 {
        total % count
    } else {
        0
    }
}

fn asserted_divisor(total: u64, count: u64) -> u64 {
    // Should NOT warn: earlier assert proves `count > 0`
    assert!(count > 0);
    total % count
}

fn asserted_divisor_in_loop(values: &[u64], count: u64) -> u64 {
    // Should NOT warn: earlier assert in an enclosing block proves `count != 0`
    assert!(count != 0);
    let mut sum = 0;
    for value in values {
        sum += value / count;
    }
    sum
}

fn main() {
    bare_division(100, 10);
    guarded_division(100, 10);
    literal_division(100);
    zero_branch_division(100, 10);
    nonzero_guard_else_branch(100, 10);
    guard_on_other_name(100, 10, 10);
    assert_on_other_divisor(100, 10, 10);
    later_assert(100, 10);
    zero_check_else_branch(100, 10);
    early_return_zero_check(100, 10);
    mirrored_guard(100, 10);
    at_least_one_guard(100, 10);
    asserted_divisor(100, 10);
    asserted_divisor_in_loop(&[100], 10);
}
