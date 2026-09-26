#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![deny(tigerstyle::no_unwrap)]

fn test_helper() {
    let value: Option<u32> = Some(1);
    assert_eq!(value.unwrap(), 1);
}

#[test]
fn integration_test_uses_helper() {
    test_helper();
}
