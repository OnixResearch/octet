#![allow(dead_code)]

use std::mem::drop;

#[must_use]
struct Token;

struct Plain;

#[must_use]
fn build_token() -> Token {
    Token
}

fn fallible() -> Result<String, String> {
    Ok("ready".to_owned())
}

fn dropped_result() {
    let result = fallible();
    drop(result);
}

fn dropped_must_use_return_value() {
    drop(build_token());
}

fn dropped_non_must_use_value() {
    let plain = Plain;
    drop(plain);
}

fn dropped_option_value() {
    // `Option` is not `#[must_use]`; dropping it discards nothing required.
    let maybe = Some(Plain);
    drop(maybe);
}

#[allow(dropping_references)]
fn dropped_result_reference() -> bool {
    // Dropping a reference leaves the `Result` itself unconsumed.
    let result = fallible();
    drop(&result);
    result.is_ok()
}

#[cfg(test)]
fn test_context_dropped_result() {
    let result = fallible();
    drop(result);
}

fn main() {}
