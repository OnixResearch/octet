pub const MAX_NODE_STATE_FILE_BYTES: u64 = 16 * 1_024 * 1_024;
pub const MAX_NODE_SECRET_BYTES: u64 = 1_024 * 1_024;
pub const ADD: u64 = 1 + (2 + 3);
pub const SUB: u64 = (20 - 3) - 2;
pub const PREFIX: &str = "blake3:";
pub const PREFIX_AND_HEX_LEN: usize = PREFIX.len() + 64;
pub static STATIC_PREFIX_AND_HEX_LEN: usize = PREFIX.len() + 64;
pub struct ArtifactRef;
impl ArtifactRef { pub const LEN: usize = PREFIX.len() + 64; }
pub trait Referenced { const LEN: usize = PREFIX.len() + 64; }
pub fn folded() -> u64 { 16 * 1_024 * 1_024 }
