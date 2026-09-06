fn will_todo() {
    // Should warn: todo! in production code
    todo!("implement later");
}

fn main() {
    // Don't actually call the unfinished function
}

#[test]
fn test_only_todo_is_allowed() {
    todo!("test-only todo");
}

#[test]
fn test_only_bare_todo_is_allowed() {
    todo!();
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn todo_in_cfg_test_module_is_allowed() {
        todo!("cfg-test-only todo");
    }

    pub(super) fn bare_todo_in_cfg_test_module_is_allowed() {
        todo!();
    }
}

fn bare_tail() -> bool {
    todo!()
}

fn bare_statement() {
    todo!();
}

fn braced_macro() {
    todo! {}
}

fn formatted_message() {
    todo!("missing {}", 1);
}

fn distinct_callsites(left: bool) {
    if left {
        todo!()
    } else {
        todo!()
    }
}

fn ordinary_text() -> &'static str {
    "todo!()"
}
