pub trait Scaler {
    fn scale(&self, value: u32) -> u32;
}

pub struct Factor {
    pub offset: u32,
}

impl Scaler for Factor {
    fn scale(&self, value: u32) -> u32 {
        if value <= u32::MAX - self.offset {
            value + self.offset
        } else {
            u32::MAX
        }
    }
}

pub fn apply_scale<S: Scaler>(scaler: &S, value: u32) -> u32 {
    scaler.scale(value)
}
