#![allow(ambient_clock)]

use std::time::Duration;
use std::time::Instant;

fn contradictory(now_ms: u64) -> u64 {
    // Should warn: takes time param but reads system time
    let _t = Instant::now();
    now_ms + 1000
}

fn pure_time(now_ms: u64, deadline_ms: u64) -> bool {
    // Should NOT warn: no system time read
    now_ms > deadline_ms
}

fn shell_poll() -> Duration {
    // Should NOT warn: no time parameter
    Instant::now().elapsed()
}

fn system_time_read(timestamp: u64) -> u64 {
    // Should warn: `SystemTime::now` reads the system clock
    let _t = std::time::SystemTime::now();
    timestamp
}

type Monotonic = Instant;

fn aliased_instant_read(elapsed: Duration) -> Duration {
    // Should warn: the alias still resolves to `std::time::Instant::now`
    let _t = Monotonic::now();
    elapsed
}

trait Clock {
    fn now(&self) -> u64;
}

struct FixedClock(u64);

impl Clock for FixedClock {
    fn now(&self) -> u64 {
        self.0
    }
}

fn injected_clock(clock: &dyn Clock, deadline_ms: u64) -> bool {
    // Should NOT warn: the injected clock is not a system-time read
    clock.now() > deadline_ms
}

struct LogicalTicks;

impl LogicalTicks {
    fn now() -> u64 {
        7
    }
}

fn logical_time(timeout_ms: u64) -> u64 {
    // Should NOT warn: a local `now` function is not the system clock
    LogicalTicks::now() + timeout_ms
}

fn main() {
    contradictory(1000);
    pure_time(1000, 2000);
    let _ = shell_poll();
    system_time_read(1000);
    aliased_instant_read(Duration::from_secs(1));
    injected_clock(&FixedClock(5), 10);
    logical_time(1000);
}
