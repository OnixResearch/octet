pub fn bounded_step(base: u32, delta: u32, limit: u32) -> u32 {
    if delta <= limit && base <= limit - delta {
        base + delta
    } else {
        limit
    }
}
