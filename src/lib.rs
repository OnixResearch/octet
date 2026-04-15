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
mod test_context;

use config::Config;

dylint_linting::dylint_library!();

pub(crate) struct LintInventoryEntry {
    pub name: &'static str,
    #[cfg_attr(not(test), allow(dead_code))]
    pub lint: &'static rustc_lint::Lint,
}

macro_rules! lint_inventory {
    ($( $name:literal, $lint:expr => |$lint_store:ident, $cfg:ident| $body:block ),+ $(,)?) => {
        pub(crate) const LINT_INVENTORY: &[LintInventoryEntry] = &[
            $(
                LintInventoryEntry {
                    name: $name,
                    lint: $lint,
                },
            )+
        ];

        fn register_inventory(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
            $(
                if !cfg.lint_is_disabled($name) {
                    lint_store.register_lints(&[$lint]);
                    let $lint_store = &mut *lint_store;
                    let $cfg = cfg;
                    $body
                }
            )+
        }
    };
}

#[cfg(test)]
fn default_level_name(level: rustc_lint::Level) -> &'static str {
    match level {
        rustc_lint::Level::Allow => "allow",
        rustc_lint::Level::Warn => "warn",
        rustc_lint::Level::ForceWarn => "force_warn",
        rustc_lint::Level::Deny => "deny",
        rustc_lint::Level::Forbid => "forbid",
        rustc_lint::Level::Expect => "expect",
    }
}

#[allow(invalid_reference_casting)]
fn apply_lint_level_overrides(sess: &rustc_session::Session, cfg: &Config) {
    let opts = std::ptr::addr_of!(sess.opts).cast_mut();

    // SAFETY: rustc calls `register_lints` during lint-store initialization,
    // before lint levels are computed from `sess.opts.lint_opts`. The API only
    // provides `&Session`, so the only way to feed config-owned per-lint levels
    // into rustc's normal lint-level machinery is to append to `lint_opts`
    // through a raw pointer at this setup point.
    unsafe {
        for lint in LINT_INVENTORY {
            if let Some(level) = cfg.lint_level(lint.name) {
                (*opts).lint_opts.push((lint.name.to_owned(), level.rustc_level()));
            }
        }
    }
}

lint_inventory! {
    // Early passes (post-expansion, syntax-only).
    // compound_assertion detects the expanded `if !(a && b) { panic(..) }` pattern.
    "compound_assertion", safety::compound_assertion::COMPOUND_ASSERTION => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(safety::compound_assertion::CompoundAssertion));
    },
    "function_length", structure::function_length::FUNCTION_LENGTH => |lint_store, cfg| {
        let fl_max = cfg.function_length_max;
        lint_store.register_early_pass(move || Box::new(structure::function_length::FunctionLength::new(fl_max)));
    },
    "acronym_style", naming::acronym_style::ACRONYM_STYLE => |lint_store, cfg| {
        let allowlist = cfg.acronym_allowlist.clone();
        lint_store.register_early_pass(move || Box::new(naming::acronym_style::AcronymStyle::new(allowlist.clone())));
    },
    "numeric_units", naming::numeric_units::NUMERIC_UNITS => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(naming::numeric_units::NumericUnits));
    },
    "unbounded_loop", safety::unbounded_loop::UNBOUNDED_LOOP => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(safety::unbounded_loop::UnboundedLoop));
    },
    "compound_condition", structure::compound_condition::COMPOUND_CONDITION => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(structure::compound_condition::CompoundCondition));
    },
    "verified_purity", purity::verified_purity::VERIFIED_PURITY => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(purity::verified_purity::VerifiedPurity));
    },
    "unjustified_allow", structure::unjustified_allow::UNJUSTIFIED_ALLOW => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(structure::unjustified_allow::UnjustifiedAllow));
    },
    "nested_conditionals", structure::nested_conditionals::NESTED_CONDITIONALS => |lint_store, cfg| {
        let nc_max = cfg.nesting_max_depth;
        lint_store.register_early_pass(move || Box::new(structure::nested_conditionals::NestedConditionals::new(nc_max)));
    },
    "assertion_density", safety::assertion_density::ASSERTION_DENSITY => |lint_store, cfg| {
        let ad_lines = cfg.assertion_density_min_lines;
        let ad_asserts = cfg.assertion_density_min_assertions;
        lint_store.register_early_pass(move || Box::new(safety::assertion_density::AssertionDensity::new(ad_lines, ad_asserts)));
    },
    "multi_lock_ordering", concurrency::multi_lock_ordering::MULTI_LOCK_ORDERING => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(concurrency::multi_lock_ordering::MultiLockOrdering));
    },

    // Late passes (post-type-resolution, need type info).
    "bool_naming", naming::bool_naming::BOOL_NAMING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::bool_naming::BoolNaming));
    },
    "ignored_result", safety::ignored_result::IGNORED_RESULT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::ignored_result::IgnoredResult));
    },
    "unchecked_narrowing", safety::unchecked_narrowing::UNCHECKED_NARROWING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_narrowing::UncheckedNarrowing));
    },
    "unchecked_division", safety::unchecked_division::UNCHECKED_DIVISION => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_division::UncheckedDivision));
    },
    "contradictory_time", purity::contradictory_time::CONTRADICTORY_TIME => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::contradictory_time::ContradictoryTime));
    },
    "ambient_clock", purity::ambient_clock::AMBIENT_CLOCK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::ambient_clock::AmbientClock));
    },
    "catch_all_on_enum", safety::catch_all_on_enum::CATCH_ALL_ON_ENUM => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::catch_all_on_enum::CatchAllOnEnum));
    },
    "platform_dependent_cast", structure::platform_dependent_cast::PLATFORM_DEPENDENT_CAST => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::platform_dependent_cast::PlatformDependentCast));
    },
    "float_for_currency", naming::float_for_currency::FLOAT_FOR_CURRENCY => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::float_for_currency::FloatForCurrency));
    },
    "ambiguous_params", structure::ambiguous_params::AMBIGUOUS_PARAMS => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::ambiguous_params::AmbiguousParams));
    },
    "raw_arithmetic_overflow", safety::raw_arithmetic_overflow::RAW_ARITHMETIC_OVERFLOW => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::raw_arithmetic_overflow::RawArithmeticOverflow));
    },
    "no_unwrap", safety::no_unwrap::NO_UNWRAP => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_unwrap::NoUnwrap));
    },
    "no_panic", safety::no_panic::NO_PANIC => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_panic::NoPanic::default()));
    },
    "no_recursion", safety::no_recursion::NO_RECURSION => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_recursion::NoRecursion));
    },
}

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let cfg = config::load(sess);
    apply_lint_level_overrides(sess, &cfg);
    register_inventory(lint_store, &cfg);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::LINT_INVENTORY;
    use super::default_level_name;
    use crate::config::canonical_lint_names;

    fn inventory_names() -> Vec<&'static str> {
        LINT_INVENTORY.iter().map(|lint| lint.name).collect()
    }

    fn inventory_levels() -> Vec<String> {
        LINT_INVENTORY
            .iter()
            .map(|lint| format!("{}={}", lint.name, default_level_name(lint.lint.default_level)))
            .collect()
    }

    #[test]
    fn lint_inventory_names_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for lint in LINT_INVENTORY {
            assert!(seen.insert(lint.name), "duplicate lint inventory entry: {}", lint.name);
        }
    }

    #[test]
    fn config_validation_derives_names_from_lint_inventory() {
        assert_eq!(canonical_lint_names(), inventory_names());
    }

    #[test]
    fn lint_inventory_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_inventory_names.txt");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read lint inventory snapshot {}: {error}", snapshot_path.display()));
        let actual = format!("{}\n", inventory_names().join("\n"));
        assert_eq!(expected, actual, "lint inventory snapshot mismatch");
    }

    #[test]
    fn lint_inventory_default_levels_snapshot_matches() {
        let snapshot_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_inventory_default_levels.txt");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read lint default-level snapshot {}: {error}", snapshot_path.display()));
        let actual = format!("{}\n", inventory_levels().join("\n"));
        assert_eq!(expected, actual, "lint default-level snapshot mismatch");
    }
}
