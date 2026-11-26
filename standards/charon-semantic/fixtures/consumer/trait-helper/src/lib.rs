pub mod helper;

pub fn scale_twice(factor: &helper::Factor, value: u32) -> u32 {
    let first = helper::apply_scale(factor, value);
    helper::apply_scale(factor, first)
}
