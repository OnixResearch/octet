fn will_todo() {
    // Should warn: todo! in production code
    todo!("implement later");
}

fn will_zero_arg_todo() {
    // Should warn: zero-argument todo! expands to a plain call, not a block
    todo!();
}

fn will_path_qualified_todo() {
    // Should warn: path-qualified todo!
    std::todo!("implement later");
}

fn main() {
    // Don't actually call the unfinished function
}

#[test]
fn test_only_todo_is_allowed() {
    todo!("test-only todo");
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn todo_in_cfg_test_module_is_allowed() {
        todo!("cfg-test-only todo");
    }
}
