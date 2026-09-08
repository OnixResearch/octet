architecture_target_macro::identity! {
    pub fn pure(value: bool) -> bool {
        value
    }
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
