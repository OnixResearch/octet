use std::format as render;
pub fn entry(value: u64) -> Result<(), String> { check(value) }
fn check(value: u64) -> Result<(), String> {
    if value > 0 { return Err(render!("bound {value}")); }
    Ok(())
}
