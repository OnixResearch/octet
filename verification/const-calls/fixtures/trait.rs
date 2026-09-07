pub trait Test { fn predicate(&self, value: u64) -> bool { if value > 0 { true } else { false } } }
pub struct Value;
impl Test for Value { fn predicate(&self, value: u64) -> bool { if value > 1 { true } else { false } } }
