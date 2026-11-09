// A selected strict profile keeps its suppression-governance lints at
// `deny`, and no source `allow`, `expect`, or `warn` may reach them, even
// with a justification: such an attribute would silence its own audit.
#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::structure)]

// Should error: `tigerstyle::structure` contains `unjustified_allow`.
#[allow(tigerstyle::structure)]
mod grouped {
    #[allow(clippy::needless_return)]
    pub fn early() -> u32 {
        return 1;
    }
}

// Should error: a justification does not make the group allow legal.
// The structure lints are noisy in this generated module.
#[allow(tigerstyle::structure)]
mod justified_group {}

// Should error: the bare alias names the audit lint itself.
#[allow(unjustified_allow)]
mod bare {}

// Should error: the canonical name of the audit lint.
#[expect(tigerstyle::unjustified_allow)]
mod expected {}

// Should error: `warn` would lower the `no_todo` audit below the floor.
#[warn(unjustified_no_todo_allow)]
mod warned {}

// Should error: `tigerstyle::safety` contains `unjustified_no_todo_allow`.
#[allow(tigerstyle::safety)]
fn quiet() {}

fn main() {
    let _ = grouped::early();
    quiet();
}
