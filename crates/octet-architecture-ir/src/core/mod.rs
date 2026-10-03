mod checks;
mod diagnostics;
mod digest;
mod normalization;

pub use checks::verify_ir_unit_bindings;
pub use checks::verify_observation_set;
pub use digest::compilation_unit_identity;
pub use digest::coverage_identity;
pub use digest::fact_identity;
pub use digest::ir_identity;
pub use digest::receipt_identity;
pub use digest::shard_identity;
pub use digest::source_span_identity;
pub use digest::unknown_fact_identity;
pub use normalization::normalize_architecture_shards;

const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const HEX_CHARS_PER_BYTE: usize = 2;
fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn valid_workspace_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && path.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}
