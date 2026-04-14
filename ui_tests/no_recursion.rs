fn factorial(n: u64) -> u64 {
    // Should warn: direct recursion
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn iterative_factorial(n: u64) -> u64 {
    // Should NOT warn: uses iteration
    let mut result = 1;
    for i in 2..=n {
        result *= i;
    }
    result
}

fn main() {
    factorial(5);
    iterative_factorial(5);
}
