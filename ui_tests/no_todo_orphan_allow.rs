#![feature(register_tool)]
#![register_tool(tigerstyle)]

fn do_work() {}

// stale tool-scoped item allow after cleanup
#[allow(tigerstyle::no_todo)]
fn orphan_tool_scoped_item() {
    do_work();
}

// stale bare item allow after cleanup
#[allow(no_todo)]
fn orphan_bare_item() {
    do_work();
}

fn orphan_tool_scoped_block() {
    // stale tool-scoped block allow after cleanup
    #[allow(tigerstyle::no_todo)]
    {
        do_work();
    }
}

fn orphan_bare_block() {
    // stale bare block allow after cleanup
    #[allow(no_todo)]
    {
        do_work();
    }
}

// cfg_attr-expanded stale tool-scoped allow after cleanup
#[cfg_attr(all(), allow(tigerstyle::no_todo))]
fn orphan_cfg_attr_tool_scoped() {
    do_work();
}

// cfg_attr-expanded stale bare allow after cleanup
#[cfg_attr(all(), allow(no_todo))]
fn orphan_cfg_attr_bare() {
    do_work();
}

// justified comment still only permits todo!
#[allow(tigerstyle::no_todo)]
fn tool_scoped_panic_still_warns() {
    panic!("panic should still warn");
}

// justified comment still only permits todo!
#[allow(no_todo)]
fn bare_unimplemented_still_warns() {
    unimplemented!("unimplemented should still warn");
}

// justified comment still only permits todo!
#[allow(tigerstyle::no_todo)]
fn tool_scoped_unreachable_still_warns() {
    unreachable!("unreachable should still warn");
}

// justified comment still only permits todo!
#[allow(no_todo)]
fn bare_zero_arg_panic_still_warns() {
    panic!();
}

// justified zero-argument stub while API lands
#[allow(no_todo)]
fn justified_zero_arg_todo_allow_stays_live() {
    todo!();
}

// justified path-qualified stub while API lands
#[allow(tigerstyle::no_todo)]
fn justified_path_qualified_todo_allow_stays_live() {
    core::todo!("qualified stub");
}

fn outer_allow_stays_live_when_inner_todo_remains() {
    // outer escape still live because inner todo remains
    #[allow(tigerstyle::no_todo)]
    {
        // inner stub while API lands
        #[allow(no_todo)]
        {
            todo!("nested stub");
        }
    }
}

fn main() {
    orphan_tool_scoped_item();
    orphan_bare_item();
    orphan_tool_scoped_block();
    orphan_bare_block();
    orphan_cfg_attr_tool_scoped();
    orphan_cfg_attr_bare();
    outer_allow_stays_live_when_inner_todo_remains();
    justified_zero_arg_todo_allow_stays_live();
    justified_path_qualified_todo_allow_stays_live();
}
