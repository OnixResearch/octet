pub fn bounded_step(input: u32) -> Result<u32, &'static str> {
    input.checked_add(1).ok_or("overflow")
}
