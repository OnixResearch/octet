pub struct Value;
impl Value {
    fn predicate(&self, value: u64) -> bool { if value > 0 { true } else { false } }
}
pub fn entry(value: u64) -> bool { Value.predicate(value) }
