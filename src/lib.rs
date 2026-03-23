#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod concurrency;
mod naming;
mod purity;
mod safety;
mod structure;

dylint_linting::dylint_library!();

/// Register all Tiger Style lints with the compiler's lint store.
///
/// Called by dylint when loading this library. Each domain module
/// contributes its lints via `register` functions added as lints
/// are implemented.
#[unsafe(no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    // Phase 2+: lints registered here as they are implemented.
    let _ = lint_store;
}
