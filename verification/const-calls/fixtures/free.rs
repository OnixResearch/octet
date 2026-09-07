pub fn ordinary(value: u64) -> bool { value > 0 }
pub fn entry(value: u64) -> bool { candidate(value) }
fn candidate(value: u64) -> bool { if value > 0 { ordinary(value) } else { false } }
