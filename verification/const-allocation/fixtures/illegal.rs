pub fn entry(value: u64) -> Result<(), String> { check(value) }
const fn check(value: u64) -> Result<(), String> {
    if value > 0 { return Err(format!("bound {value}")); }
    Ok(())
}