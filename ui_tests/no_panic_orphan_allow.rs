#![feature(register_tool)]
#![register_tool(tigerstyle)]

fn do_work() {}

#[allow(tigerstyle::no_panic)]
fn orphan_item_no_comment() {
    do_work();
}

// stale item allow after cleanup
#[allow(tigerstyle::no_panic)]
fn orphan_item_with_comment() {
    do_work();
}

fn orphan_block_no_comment() {
    #[allow(tigerstyle::no_panic)]
    {
        do_work();
    }
}

fn orphan_block_with_comment() {
    // stale block allow after cleanup
    #[allow(tigerstyle::no_panic)]
    {
        do_work();
    }
}

// cfg_attr-expanded stale allow after cleanup
#[cfg_attr(all(), allow(tigerstyle::no_panic))]
fn orphan_cfg_attr() {
    do_work();
}

fn outer_allow_stays_live_when_inner_todo_remains() {
    // outer escape still live because inner todo remains
    #[allow(tigerstyle::no_panic)]
    {
        // inner stub while API lands
        #[allow(tigerstyle::no_panic)]
        {
            todo!("nested stub");
        }
    }
}

fn main() {
    orphan_item_no_comment();
    orphan_item_with_comment();
    orphan_block_no_comment();
    orphan_block_with_comment();
    orphan_cfg_attr();
    outer_allow_stays_live_when_inner_todo_remains();
}
