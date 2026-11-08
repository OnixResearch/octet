use std::time::Duration;
// Should error: importing a wall-clock type is itself the violation.
use std::time::Instant;
// Should error: importing an I/O type is itself the violation.
use std::fs::File;
// Should error: nested import groups are checked per imported path.
use std::{fs, time::SystemTime};

pub async fn impure_async() {
    // Should error: async fn in verified module
}

fn pure_duration() -> Duration {
    // Should NOT error: Duration is a pure value type
    Duration::from_millis(500)
}

fn wall_clock() -> Instant {
    Instant::now()
}

fn open_config() -> Option<File> {
    File::open("config.toml").ok()
}

fn main() {
    let _ = pure_duration();
    let _ = wall_clock();
    let _ = open_config();
}
