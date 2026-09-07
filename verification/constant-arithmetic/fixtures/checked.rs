pub fn add(left: u64, right: u64) -> Option<u64> { left.checked_add(right) }
pub fn sub(left: u64, right: u64) -> Option<u64> { left.checked_sub(right) }
pub fn mul(left: u64, right: u64) -> Option<u64> { left.checked_mul(right) }
