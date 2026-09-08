architecture_target_macro::identity! {
    pub fn pure(value: bool) -> bool {
        value
    }
}

pub trait Associated {
    type Value;
    const PRESENT: bool;

    fn identity(value: Self::Value) -> Self::Value {
        value
    }
}

pub struct AssociatedValue;

impl Associated for AssociatedValue {
    type Value = bool;
    const PRESENT: bool = true;
}

const fn constant_width() -> usize {
    1
}

const fn constant_truth() -> bool {
    true
}

fn closure_truth() -> bool {
    true
}

// Both array positions create anonymous constant bodies with resolved calls.
architecture_target_macro::identity! {
    pub fn anonymous_body() -> [bool; constant_width()] {
        [true; constant_width()]
    }
}

pub fn inline_body() -> bool {
    const { constant_truth() }
}

pub fn closure_body() -> bool {
    let invoke = || closure_truth();
    invoke()
}

#[cfg(feature = "extra")]
pub fn extra_effect() {
    let _ = std::fs::read("never-executed");
}

#[cfg(test)]
#[test]
fn test_effect() {
    let _ = std::fs::read("never-executed");
}
