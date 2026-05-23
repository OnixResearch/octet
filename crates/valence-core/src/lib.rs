//! Pure Valence evidence helpers shared by Octet and future standalone Valence surfaces.
//!
//! This crate intentionally contains deterministic core logic only. CLI I/O,
//! filesystem reads, and process orchestration remain in `cargo-octet`.

pub mod evidence_index;
pub mod function_object;
