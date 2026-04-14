#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod concurrency;
pub mod config;
mod naming;
mod purity;
mod safety;
mod structure;

dylint_linting::dylint_library!();

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let cfg = config::load();

    // -- Early passes (post-expansion, syntax-only) --
    // compound_assertion detects the expanded `if !(a && b) { panic(..) }` pattern
    lint_store.register_lints(&[safety::compound_assertion::COMPOUND_ASSERTION]);
    lint_store.register_early_pass(|| Box::new(safety::compound_assertion::CompoundAssertion));

    // -- Early passes (post-expansion, syntax-only) --
    lint_store.register_lints(&[structure::function_length::FUNCTION_LENGTH]);
    let fl_max = cfg.function_length_max;
    lint_store.register_early_pass(move || Box::new(structure::function_length::FunctionLength::new(fl_max)));

    lint_store.register_lints(&[naming::acronym_style::ACRONYM_STYLE]);
    let allowlist = cfg.acronym_allowlist.clone();
    lint_store.register_early_pass(move || Box::new(naming::acronym_style::AcronymStyle::new(allowlist.clone())));

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
    let nc_max = cfg.nesting_max_depth;
    lint_store.register_early_pass(move || Box::new(structure::nested_conditionals::NestedConditionals::new(nc_max)));

    lint_store.register_lints(&[safety::assertion_density::ASSERTION_DENSITY]);
    let ad_lines = cfg.assertion_density_min_lines;
    let ad_asserts = cfg.assertion_density_min_assertions;
    lint_store
        .register_early_pass(move || Box::new(safety::assertion_density::AssertionDensity::new(ad_lines, ad_asserts)));

    lint_store.register_lints(&[concurrency::multi_lock_ordering::MULTI_LOCK_ORDERING]);
    lint_store.register_early_pass(|| Box::new(concurrency::multi_lock_ordering::MultiLockOrdering));

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

    lint_store.register_lints(&[purity::ambient_clock::AMBIENT_CLOCK]);
    lint_store.register_late_pass(|_| Box::new(purity::ambient_clock::AmbientClock));

    lint_store.register_lints(&[safety::catch_all_on_enum::CATCH_ALL_ON_ENUM]);
    lint_store.register_late_pass(|_| Box::new(safety::catch_all_on_enum::CatchAllOnEnum));

    lint_store.register_lints(&[structure::platform_dependent_cast::PLATFORM_DEPENDENT_CAST]);
    lint_store.register_late_pass(|_| Box::new(structure::platform_dependent_cast::PlatformDependentCast));

    lint_store.register_lints(&[naming::float_for_currency::FLOAT_FOR_CURRENCY]);
    lint_store.register_late_pass(|_| Box::new(naming::float_for_currency::FloatForCurrency));

    lint_store.register_lints(&[structure::ambiguous_params::AMBIGUOUS_PARAMS]);
    lint_store.register_late_pass(|_| Box::new(structure::ambiguous_params::AmbiguousParams));

    lint_store.register_lints(&[safety::raw_arithmetic_overflow::RAW_ARITHMETIC_OVERFLOW]);
    lint_store.register_late_pass(|_| Box::new(safety::raw_arithmetic_overflow::RawArithmeticOverflow));

    lint_store.register_lints(&[safety::no_unwrap::NO_UNWRAP]);
    lint_store.register_late_pass(|_| Box::new(safety::no_unwrap::NoUnwrap));

    lint_store.register_lints(&[safety::no_panic::NO_PANIC]);
    lint_store.register_early_pass(|| Box::new(safety::no_panic::NoPanic));

    lint_store.register_lints(&[safety::no_recursion::NO_RECURSION]);
    lint_store.register_late_pass(|_| Box::new(safety::no_recursion::NoRecursion));
}
