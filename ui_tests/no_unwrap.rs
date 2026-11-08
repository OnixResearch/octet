fn uses_unwrap() {
    // Should warn: unwrap on Result
    let val: Result<u32, &str> = Ok(1);
    let _ = val.unwrap();
}

fn uses_expect() {
    // Should warn: expect on Option
    let val: Option<u32> = Some(1);
    let _ = val.expect("missing");
}

fn unwrap_through_option_reference(value: &Option<u32>) -> u32 {
    // Should warn: unwrap on &Option auto-derefs
    value.unwrap()
}

fn expect_through_result_reference(value: &Result<u32, &'static str>) -> u32 {
    // Should warn: expect on &Result auto-derefs
    value.expect("missing")
}

fn unwrap_in_closure_over_option_slice(values: &[Option<u32>]) -> Vec<u32> {
    // Should warn: closure receives &Option<u32>
    values.iter().map(|value| value.unwrap()).collect()
}

fn uses_question_mark() -> Result<u32, &'static str> {
    // Should NOT warn: proper error propagation
    let val: Result<u32, &str> = Ok(1);
    let v = val?;
    Ok(v)
}

fn main() {
    uses_unwrap();
    uses_expect();
}

#[test]
fn unwrap_in_test_is_allowed() {
    let val: Option<u32> = Some(1);
    let _ = val.unwrap();
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn unwrap_in_cfg_test_module_is_allowed() {
        let val: Option<u32> = Some(1);
        let _ = val.unwrap();
    }
}
