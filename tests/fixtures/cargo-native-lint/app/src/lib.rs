pub fn greeting() -> &'static str {
    used_dep::label()
}

#[cfg(feature = "extra")]
pub fn extra_greeting() -> &'static str {
    feature_dep::extra_label()
}
