mod app { pub const fn format(value: u64) -> u64 { value } }
pub fn entry(value: u64) -> u64 { choose(value) }
fn choose(value: u64) -> u64 {
    if value > 0 { app::format(value) } else { 0 }
}
