use std::time::{Duration, Instant};

fn deadline_from(timeout: Duration) -> Instant {
    Instant::now() + timeout
}

mod renamed {
    use std::time::Instant as Clock;

    pub fn started() -> Clock {
        // Should warn: the renamed import still resolves to `Instant::now`
        Clock::now()
    }
}

type WallClock = std::time::SystemTime;

fn wall_clock() -> WallClock {
    // Should warn: the type alias still resolves to `SystemTime::now`
    WallClock::now()
}

struct LogicalClock;

impl LogicalClock {
    fn now() -> u64 {
        7
    }
}

fn logical_now() -> u64 {
    // Should NOT warn: a local `now` function is not a wall-clock read
    LogicalClock::now()
}

fn main() {
    let _deadline = deadline_from(Duration::from_secs(1));
    let _ = renamed::started();
    let _ = wall_clock();
    let _ = logical_now();
}
