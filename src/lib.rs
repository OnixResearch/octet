#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_errors;
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

#[unsafe(no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    // -- Pre-expansion pass (sees unexpanded macros) --
    lint_store.register_lints(&[safety::compound_assertion::COMPOUND_ASSERTION]);
    lint_store.register_pre_expansion_pass(|| {
        Box::new(safety::compound_assertion::CompoundAssertion)
    });

    // -- Early passes (post-expansion, syntax-only) --
    lint_store.register_lints(&[structure::function_length::FUNCTION_LENGTH]);
    lint_store.register_early_pass(|| Box::new(structure::function_length::FunctionLength));

    lint_store.register_lints(&[naming::acronym_style::ACRONYM_STYLE]);
    lint_store.register_early_pass(|| Box::new(naming::acronym_style::AcronymStyle));

    lint_store.register_lints(&[naming::numeric_units::NUMERIC_UNITS]);
    lint_store.register_early_pass(|| Box::new(naming::numeric_units::NumericUnits));
}
