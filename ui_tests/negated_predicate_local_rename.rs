fn multiple_uses() {
    let is_not_ready = false;
    let _a = is_not_ready;
    let _b = is_not_ready;
    let _c = !is_not_ready;
}

fn closure_use() -> usize {
    let is_not_empty = true;
    let values = [1, 2, 3];
    values.iter().filter(|_| is_not_empty).count()
}

fn async_block_use() {
    let is_not_done = true;
    let _future = async move { is_not_done };
}

struct Flags {
    is_not_hidden: bool,
}

fn shorthand_field_use() -> bool {
    let is_not_hidden = true;
    let flags = Flags { is_not_hidden };
    flags.is_not_hidden
}

fn main() {
    multiple_uses();
    let _ = closure_use();
    async_block_use();
    let _ = shorthand_field_use();
}
