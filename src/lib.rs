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

    lint_store.register_lints(&[safety::unbounded_loop::UNBOUNDED_LOOP]);
    lint_store.register_early_pass(|| Box::new(safety::unbounded_loop::UnboundedLoop));

    lint_store.register_lints(&[structure::compound_condition::COMPOUND_CONDITION]);
    lint_store.register_early_pass(|| Box::new(structure::compound_condition::CompoundCondition));

    lint_store.register_lints(&[purity::verified_purity::VERIFIED_PURITY]);
    lint_store.register_early_pass(|| Box::new(purity::verified_purity::VerifiedPurity));

    lint_store.register_lints(&[structure::unjustified_allow::UNJUSTIFIED_ALLOW]);
    lint_store.register_early_pass(|| Box::new(structure::unjustified_allow::UnjustifiedAllow));

    lint_store.register_lints(&[structure::nested_conditionals::NESTED_CONDITIONALS]);
    lint_store.register_early_pass(|| Box::new(structure::nested_conditionals::NestedConditionals));

    // -- Late passes (post-type-resolution, need type info) --
    lint_store.register_lints(&[naming::bool_naming::BOOL_NAMING]);
    lint_store.register_late_pass(|_| Box::new(naming::bool_naming::BoolNaming));

    lint_store.register_lints(&[safety::ignored_result::IGNORED_RESULT]);
    lint_store.register_late_pass(|_| Box::new(safety::ignored_result::IgnoredResult));

    lint_store.register_lints(&[safety::unchecked_narrowing::UNCHECKED_NARROWING]);
    lint_store.register_late_pass(|_| Box::new(safety::unchecked_narrowing::UncheckedNarrowing));

    lint_store.register_lints(&[safety::unchecked_division::UNCHECKED_DIVISION]);
    lint_store.register_late_pass(|_| Box::new(safety::unchecked_division::UncheckedDivision));

    lint_store.register_lints(&[purity::contradictory_time::CONTRADICTORY_TIME]);
    lint_store.register_late_pass(|_| Box::new(purity::contradictory_time::ContradictoryTime));

    // -- Phase 6: clankers pattern lints --
    lint_store.register_lints(&[safety::catch_all_on_enum::CATCH_ALL_ON_ENUM]);
    lint_store.register_late_pass(|_| Box::new(safety::catch_all_on_enum::CatchAllOnEnum));

    lint_store.register_lints(&[structure::platform_dependent_cast::PLATFORM_DEPENDENT_CAST]);
    lint_store.register_late_pass(|_| Box::new(structure::platform_dependent_cast::PlatformDependentCast));

    lint_store.register_lints(&[naming::float_for_currency::FLOAT_FOR_CURRENCY]);
    lint_store.register_late_pass(|_| Box::new(naming::float_for_currency::FloatForCurrency));

    // -- Phase 7: tier 3 lints --
    lint_store.register_lints(&[safety::assertion_density::ASSERTION_DENSITY]);
    lint_store.register_early_pass(|| Box::new(safety::assertion_density::AssertionDensity));

    lint_store.register_lints(&[structure::ambiguous_params::AMBIGUOUS_PARAMS]);
    lint_store.register_late_pass(|_| Box::new(structure::ambiguous_params::AmbiguousParams));

    lint_store.register_lints(&[safety::raw_arithmetic_overflow::RAW_ARITHMETIC_OVERFLOW]);
    lint_store.register_late_pass(|_| Box::new(safety::raw_arithmetic_overflow::RawArithmeticOverflow));

    lint_store.register_lints(&[concurrency::multi_lock_ordering::MULTI_LOCK_ORDERING]);
    lint_store.register_early_pass(|| Box::new(concurrency::multi_lock_ordering::MultiLockOrdering));

    // -- Tiger Style safety: no-panic family --
    lint_store.register_lints(&[safety::no_unwrap::NO_UNWRAP]);
    lint_store.register_late_pass(|_| Box::new(safety::no_unwrap::NoUnwrap));

    lint_store.register_lints(&[safety::no_panic::NO_PANIC]);
    lint_store.register_pre_expansion_pass(|| Box::new(safety::no_panic::NoPanic));

    lint_store.register_lints(&[safety::no_recursion::NO_RECURSION]);
    lint_store.register_late_pass(|_| Box::new(safety::no_recursion::NoRecursion));
}
