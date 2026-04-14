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

const KNOWN_LINTS: &[&str] = &[
    "compound_assertion",
    "function_length",
    "acronym_style",
    "numeric_units",
    "unbounded_loop",
    "compound_condition",
    "verified_purity",
    "unjustified_allow",
    "nested_conditionals",
    "assertion_density",
    "multi_lock_ordering",
    "bool_naming",
    "ignored_result",
    "unchecked_narrowing",
    "unchecked_division",
    "contradictory_time",
    "ambient_clock",
    "catch_all_on_enum",
    "platform_dependent_cast",
    "float_for_currency",
    "ambiguous_params",
    "raw_arithmetic_overflow",
    "no_unwrap",
    "no_panic",
    "no_recursion",
];

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let cfg = config::load(sess, KNOWN_LINTS);

    // -- Early passes (post-expansion, syntax-only) --
    // compound_assertion detects the expanded `if !(a && b) { panic(..) }` pattern
    if !cfg.lint_is_disabled("compound_assertion") {
        lint_store.register_lints(&[safety::compound_assertion::COMPOUND_ASSERTION]);
        lint_store.register_early_pass(|| Box::new(safety::compound_assertion::CompoundAssertion));
    }

    // -- Early passes (post-expansion, syntax-only) --
    if !cfg.lint_is_disabled("function_length") {
        lint_store.register_lints(&[structure::function_length::FUNCTION_LENGTH]);
        let fl_max = cfg.function_length_max;
        lint_store.register_early_pass(move || Box::new(structure::function_length::FunctionLength::new(fl_max)));
    }

    if !cfg.lint_is_disabled("acronym_style") {
        lint_store.register_lints(&[naming::acronym_style::ACRONYM_STYLE]);
        let allowlist = cfg.acronym_allowlist.clone();
        lint_store.register_early_pass(move || Box::new(naming::acronym_style::AcronymStyle::new(allowlist.clone())));
    }

    if !cfg.lint_is_disabled("numeric_units") {
        lint_store.register_lints(&[naming::numeric_units::NUMERIC_UNITS]);
        lint_store.register_early_pass(|| Box::new(naming::numeric_units::NumericUnits));
    }

    if !cfg.lint_is_disabled("unbounded_loop") {
        lint_store.register_lints(&[safety::unbounded_loop::UNBOUNDED_LOOP]);
        lint_store.register_early_pass(|| Box::new(safety::unbounded_loop::UnboundedLoop));
    }

    if !cfg.lint_is_disabled("compound_condition") {
        lint_store.register_lints(&[structure::compound_condition::COMPOUND_CONDITION]);
        lint_store.register_early_pass(|| Box::new(structure::compound_condition::CompoundCondition));
    }

    if !cfg.lint_is_disabled("verified_purity") {
        lint_store.register_lints(&[purity::verified_purity::VERIFIED_PURITY]);
        lint_store.register_early_pass(|| Box::new(purity::verified_purity::VerifiedPurity));
    }

    if !cfg.lint_is_disabled("unjustified_allow") {
        lint_store.register_lints(&[structure::unjustified_allow::UNJUSTIFIED_ALLOW]);
        lint_store.register_early_pass(|| Box::new(structure::unjustified_allow::UnjustifiedAllow));
    }

    if !cfg.lint_is_disabled("nested_conditionals") {
        lint_store.register_lints(&[structure::nested_conditionals::NESTED_CONDITIONALS]);
        let nc_max = cfg.nesting_max_depth;
        lint_store
            .register_early_pass(move || Box::new(structure::nested_conditionals::NestedConditionals::new(nc_max)));
    }

    if !cfg.lint_is_disabled("assertion_density") {
        lint_store.register_lints(&[safety::assertion_density::ASSERTION_DENSITY]);
        let ad_lines = cfg.assertion_density_min_lines;
        let ad_asserts = cfg.assertion_density_min_assertions;
        lint_store.register_early_pass(move || {
            Box::new(safety::assertion_density::AssertionDensity::new(ad_lines, ad_asserts))
        });
    }

    if !cfg.lint_is_disabled("multi_lock_ordering") {
        lint_store.register_lints(&[concurrency::multi_lock_ordering::MULTI_LOCK_ORDERING]);
        lint_store.register_early_pass(|| Box::new(concurrency::multi_lock_ordering::MultiLockOrdering));
    }

    // -- Late passes (post-type-resolution, need type info) --
    if !cfg.lint_is_disabled("bool_naming") {
        lint_store.register_lints(&[naming::bool_naming::BOOL_NAMING]);
        lint_store.register_late_pass(|_| Box::new(naming::bool_naming::BoolNaming));
    }

    if !cfg.lint_is_disabled("ignored_result") {
        lint_store.register_lints(&[safety::ignored_result::IGNORED_RESULT]);
        lint_store.register_late_pass(|_| Box::new(safety::ignored_result::IgnoredResult));
    }

    if !cfg.lint_is_disabled("unchecked_narrowing") {
        lint_store.register_lints(&[safety::unchecked_narrowing::UNCHECKED_NARROWING]);
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_narrowing::UncheckedNarrowing));
    }

    if !cfg.lint_is_disabled("unchecked_division") {
        lint_store.register_lints(&[safety::unchecked_division::UNCHECKED_DIVISION]);
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_division::UncheckedDivision));
    }

    if !cfg.lint_is_disabled("contradictory_time") {
        lint_store.register_lints(&[purity::contradictory_time::CONTRADICTORY_TIME]);
        lint_store.register_late_pass(|_| Box::new(purity::contradictory_time::ContradictoryTime));
    }

    if !cfg.lint_is_disabled("ambient_clock") {
        lint_store.register_lints(&[purity::ambient_clock::AMBIENT_CLOCK]);
        lint_store.register_late_pass(|_| Box::new(purity::ambient_clock::AmbientClock));
    }

    if !cfg.lint_is_disabled("catch_all_on_enum") {
        lint_store.register_lints(&[safety::catch_all_on_enum::CATCH_ALL_ON_ENUM]);
        lint_store.register_late_pass(|_| Box::new(safety::catch_all_on_enum::CatchAllOnEnum));
    }

    if !cfg.lint_is_disabled("platform_dependent_cast") {
        lint_store.register_lints(&[structure::platform_dependent_cast::PLATFORM_DEPENDENT_CAST]);
        lint_store.register_late_pass(|_| Box::new(structure::platform_dependent_cast::PlatformDependentCast));
    }

    if !cfg.lint_is_disabled("float_for_currency") {
        lint_store.register_lints(&[naming::float_for_currency::FLOAT_FOR_CURRENCY]);
        lint_store.register_late_pass(|_| Box::new(naming::float_for_currency::FloatForCurrency));
    }

    if !cfg.lint_is_disabled("ambiguous_params") {
        lint_store.register_lints(&[structure::ambiguous_params::AMBIGUOUS_PARAMS]);
        lint_store.register_late_pass(|_| Box::new(structure::ambiguous_params::AmbiguousParams));
    }

    if !cfg.lint_is_disabled("raw_arithmetic_overflow") {
        lint_store.register_lints(&[safety::raw_arithmetic_overflow::RAW_ARITHMETIC_OVERFLOW]);
        lint_store.register_late_pass(|_| Box::new(safety::raw_arithmetic_overflow::RawArithmeticOverflow));
    }

    if !cfg.lint_is_disabled("no_unwrap") {
        lint_store.register_lints(&[safety::no_unwrap::NO_UNWRAP]);
        lint_store.register_late_pass(|_| Box::new(safety::no_unwrap::NoUnwrap));
    }

    if !cfg.lint_is_disabled("no_panic") {
        lint_store.register_lints(&[safety::no_panic::NO_PANIC]);
        lint_store.register_early_pass(|| Box::new(safety::no_panic::NoPanic));
    }

    if !cfg.lint_is_disabled("no_recursion") {
        lint_store.register_lints(&[safety::no_recursion::NO_RECURSION]);
        lint_store.register_late_pass(|_| Box::new(safety::no_recursion::NoRecursion));
    }
}
