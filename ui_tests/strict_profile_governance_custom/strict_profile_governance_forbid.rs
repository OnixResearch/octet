// A selected strict profile forbids the suppression-governance lints, so no
// source attribute can silence its own audit, even with a justification.
#![feature(register_tool)]
#![register_tool(tigerstyle)]

// Should error: `tigerstyle::structure` contains `unjustified_allow`.
#[allow(tigerstyle::structure)]
mod grouped {
    #[allow(clippy::needless_return)]
    pub fn early() -> u32 {
        return 1;
    }
}

// Should error: the bare alias names the audit lint itself.
#[allow(unjustified_allow)]
mod bare {
    #[allow(clippy::needless_return)]
    pub fn early() -> u32 {
        return 2;
    }
}

// Should error: the canonical name of the audit lint.
#[allow(tigerstyle::unjustified_allow)]
mod canonical {
    #[allow(clippy::needless_return)]
    pub fn early() -> u32 {
        return 3;
    }
}

// Should error: `tigerstyle::safety` contains `unjustified_no_todo_allow`.
#[expect(tigerstyle::safety)]
fn quiet() {}

// Should error: allowing rustc's `forbidden_lint_groups` would turn the
// errors above back into warnings.
#[allow(forbidden_lint_groups)]
fn unforbidden() {}

fn main() {
    let _ = (grouped::early(), bare::early(), canonical::early());
    quiet();
    unforbidden();
}
