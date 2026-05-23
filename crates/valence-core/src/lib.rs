//! Pure Valence evidence helpers shared by Octet and future standalone Valence surfaces.
//!
//! This crate intentionally contains deterministic core logic only. CLI I/O,
//! filesystem reads, source scanning, object marker mutation, corpus orchestration,
//! supplement orchestration, stdout/stderr, and process exits remain in
//! `cargo-octet` until a future reviewed cutover changes that boundary.
//!
//! # Evidence boundary
//!
//! `valence-core` currently supports bounded local evidence replay for selected
//! Octet/Valence surfaces. It does not claim behavioral correctness, semantic
//! equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine
//! determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

pub mod evidence_index;
pub mod function_object;
