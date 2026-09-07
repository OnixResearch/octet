pub fn push_bounded_name(names: &mut [String], name: String) -> Result<(), &'static str> {
    if names.len() >= 100_000 { return Err("capacity exceeded"); }
    names.push(name);
    Ok(())
}
