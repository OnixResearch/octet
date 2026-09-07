pub fn push_bounded_entry(entries: &mut Vec<u64>, entry: u64) -> Result<(), &'static str> {
    ensure_entry_capacity(entries.len())?;
    entries.push(entry);
    Ok(())
}
pub fn push_bounded_name(names: &mut Vec<String>, name: String) -> Result<(), &'static str> {
    ensure_entry_capacity(names.len())?;
    names.push(name);
    Ok(())
}
fn ensure_entry_capacity(current: usize) -> Result<(), &'static str> {
    if current >= 100_000 { Err("capacity exceeded") } else { Ok(()) }
}
pub fn qualified(entries: &mut Vec<u64>) { Vec::push(entries, 1); }
pub struct Store;
impl Store {
    pub fn append(&self, entries: &mut Vec<u64>) { entries.push(1); }
}
