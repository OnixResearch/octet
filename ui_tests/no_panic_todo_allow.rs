#![feature(register_tool)]
#![register_tool(tigerstyle)]

// justified stub while API lands
#[allow(tigerstyle::no_panic)]
fn function_level_allowed() {
    todo!("allowed function-level stub");
}

#[allow(tigerstyle::no_panic)] // justified inline stub while API lands
fn same_line_allowed() {
    todo!("allowed same-line stub");
}

fn block_level_allowed() {
    // justified block stub while API lands
    #[allow(tigerstyle::no_panic)]
    {
        todo!("allowed block-level stub");
    }
}

/// doc comments do not justify panic escapes
#[allow(tigerstyle::no_panic)]
fn doc_comment_still_warns() {
    todo!("doc comment should still warn");
}

/* block comments do not justify panic escapes */
#[allow(tigerstyle::no_panic)]
fn block_comment_still_warns() {
    todo!("block comment should still warn");
}

// blank line breaks adjacency

#[allow(tigerstyle::no_panic)]
fn blank_line_still_warns() {
    todo!("blank line should still warn");
}

#[allow(tigerstyle::no_panic)]
fn missing_comment_still_warns() {
    todo!("missing comment should still warn");
}

// justified comment still only permits todo!
#[allow(tigerstyle::no_panic)]
fn justified_panic_still_warns() {
    panic!("panic should still warn");
}

// justified comment still only permits todo!
#[allow(tigerstyle::no_panic)]
fn justified_unimplemented_still_warns() {
    unimplemented!("unimplemented should still warn");
}

// justified comment still only permits todo!
#[allow(tigerstyle::no_panic)]
fn justified_unreachable_still_warns() {
    unreachable!("unreachable should still warn");
}

fn main() {
    function_level_allowed();
    same_line_allowed();
    block_level_allowed();
}
