#![warn(unjustified_allow)]

#[allow(clippy::needless_return)]
fn bare_allow() -> u32 {
    return 1;
}

// Should NOT warn: has justification comment
// Verified casts: Verus checks every narrowing in this module.
#[allow(clippy::cast_possible_truncation)]
fn justified_allow(x: u64) -> u32 {
    x as u32
}

fn main() {
    bare_allow();
    justified_allow(1);
}

struct Counter;

impl Counter {
    #[allow(clippy::needless_return)]
    fn bare_method_allow(&self) -> u32 {
        return 1;
    }

    // Should NOT warn: has justification comment
    // Early return mirrors the trait contract documented on `Allowed`.
    #[allow(clippy::needless_return)]
    fn justified_method_allow(&self) -> u32 {
        return 2;
    }
}

trait Allowed {
    #[allow(clippy::needless_return)]
    fn bare_default_allow(&self) -> u32 {
        return 1;
    }
}

// Should NOT warn: `reason = "..."` carries its own justification. The blank
// line keeps this fixture note from counting as a nearby `//` justification.

#[allow(clippy::needless_return, reason = "early return mirrors the protocol state table")]
fn reasoned_allow() -> u32 {
    return 3;
}

// Should warn: an empty reason is not a justification.

#[allow(clippy::needless_return, reason = "  ")]
fn empty_reason_allow() -> u32 {
    return 4;
}

impl Counter {
    // Should NOT warn: `reason = "..."` on a method allow.

    #[allow(
        clippy::needless_return,
        reason = "early return keeps the branch shape aligned with `bare_method_allow`"
    )]
    fn reasoned_method_allow(&self) -> u32 {
        return 5;
    }
}
