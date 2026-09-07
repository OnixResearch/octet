struct Amount;
impl Amount { const fn get(value: u64) -> u64 { value } }
pub fn entry(value: u64) -> u64 { choose(value) }
fn choose(value: u64) -> u64 {
    if value > 0 { Amount::get(value) } else { 0 }
}
