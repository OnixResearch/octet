# Purity Lints — Delta Spec

## ADDED Requirements

### Requirement: Verified Module Purity

The lint MUST detect the following in files under any `verified/` directory:

1. `async fn` or `async move` blocks
2. `tokio::` or `async_std::` path references
3. `std::io::` usage (Read, Write, BufReader, etc.)
4. `std::time::SystemTime` or `std::time::Instant` calls
5. `std::fs::` filesystem operations
6. `std::net::` networking
7. `std::process::` subprocess spawning

The lint MUST emit a clear message: "I/O operation in verified module —
verified/ functions must be pure (no async, no IO, no system time)."

The lint MUST NOT fire on `std::time::Duration` (it's a value type, not
a clock read).

The lint MUST NOT fire on `#[cfg(test)]` blocks inside verified modules
(tests may use IO for setup).

The lint SHOULD detect `rand::` usage and suggest passing randomness as
an explicit parameter.

#### Scenario: Async function in verified module

- GIVEN `src/verified/lock.rs` contains `pub async fn check_expiry()`
- WHEN the lint runs
- THEN it warns: "async fn in verified module — verified functions must be synchronous and pure"

#### Scenario: System time in verified module

- GIVEN `src/verified/deadline.rs` calls `SystemTime::now()`
- WHEN the lint runs
- THEN it warns and suggests passing time as a `u64` parameter

#### Scenario: Duration type usage

- GIVEN `src/verified/timeout.rs` uses `Duration::from_millis(500)`
- WHEN the lint runs
- THEN it does NOT fire (Duration is a pure value)

#### Scenario: Test helper in verified module

- GIVEN `src/verified/lock.rs` has `#[cfg(test)] mod tests { use std::fs; }`
- WHEN the lint runs
- THEN it does NOT fire on the test module
