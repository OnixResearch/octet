pub mod helper;

pub fn combine_steps(a: u32, b: u32, limit: u32) -> u32 {
    let first = helper::bounded_step(a, b, limit);
    helper::bounded_step(first, 1, limit)
}
