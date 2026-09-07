pub fn entry(value: u64) -> &'static str { describe(value) }
fn describe(value: u64) -> &'static str {
    if value > 0 { "Err(format!(value))" } else { "empty" }
}
