mod octet {
    pub struct Config;
}

/// Compatibility alias retained for downstream Slotcar callers.
pub use crate::octet::Config as OctetConfig;

// Should NOT warn: the note sits above the attribute, not directly above the `use`.
// Compatibility alias kept for the 0.x loader.
#[doc(hidden)]
pub use crate::octet::Config as LegacyConfig;

// Should NOT warn: the note is on the first line of a multi-line doc comment.
/// Compatibility alias retained for downstream Slotcar callers.
/// New code should name `octet::Config` directly.
pub use crate::octet::Config as SlotcarConfig;

// Should warn: an `Incompatible` note is not an alias note.
/// Incompatible with the old loader.
pub use crate::octet::Config as LoaderConfig;

fn main() {}
