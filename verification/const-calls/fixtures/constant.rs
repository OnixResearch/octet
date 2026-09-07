pub struct Value;
impl Value { pub const fn predicate(&self, value: u64) -> bool { value > 0 } }
pub fn entry(value: u64) -> bool { candidate(&Value, value) }
fn candidate(receiver: &Value, value: u64) -> bool { if value > 0 { receiver.predicate(value) } else { false } }
