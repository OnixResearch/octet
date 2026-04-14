fn will_panic() {
    // Should warn: panic! in production code
    panic!("something went wrong");
}

fn will_todo() {
    // Should warn: todo! in production code
    todo!("implement later");
}

fn will_unimplemented() {
    // Should warn: unimplemented!
    unimplemented!("not done");
}

fn main() {
    // Don't actually call the panicking functions
}
