pub fn entry(value: u64) -> Option<u64> { candidate(value) }
fn candidate(value: u64) -> Option<u64> { if value > 0 { Some(value) } else { None } }
