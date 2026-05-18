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
mod diagnostics;
mod naming;
mod performance;
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

fn lint_id_for_name(lint_name: &str) -> rustc_lint::LintId {
    let entry = LINT_INVENTORY
        .iter()
        .find(|lint| lint.name == lint_name)
        .unwrap_or_else(|| panic!("missing lint inventory entry for `{lint_name}`"));
    rustc_lint::LintId::of(entry.lint)
}

fn register_no_todo_bundle(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    let emit_no_todo = !cfg.lint_is_disabled("no_todo");
    let emit_unjustified_allow = !cfg.lint_is_disabled("unjustified_no_todo_allow");
    if !emit_no_todo && !emit_unjustified_allow {
        return;
    }

    lint_store
        .register_late_pass(move |_| Box::new(safety::no_todo::NoTodo::new(emit_no_todo, emit_unjustified_allow)));
}

fn register_domain_groups(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    for group in diagnostics::group_snapshots() {
        let lint_ids = group
            .lint_names
            .into_iter()
            .filter(|lint_name| !cfg.lint_is_disabled(lint_name))
            .map(lint_id_for_name)
            .collect::<Vec<_>>();
        if lint_ids.is_empty() {
            continue;
        }

        lint_store.register_group(true, group.group_name, None, lint_ids);
    }
}

fn register_compatibility_aliases(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    for lint in LINT_INVENTORY {
        if cfg.lint_is_disabled(lint.name) {
            continue;
        }

        lint_store.register_group(true, lint.name, None, vec![rustc_lint::LintId::of(lint.lint)]);
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
    "magic_number", structure::magic_number::MAGIC_NUMBER => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::magic_number::MagicNumber));
    },
    "excessive_file_length", structure::excessive_file_length::EXCESSIVE_FILE_LENGTH => |lint_store, cfg| {
        let max_lines = cfg.excessive_file_length_max;
        lint_store.register_early_pass(move || Box::new(structure::excessive_file_length::ExcessiveFileLength::new(max_lines)));
    },
    "module_file_count", structure::module_file_count::MODULE_FILE_COUNT => |lint_store, cfg| {
        let max_files = cfg.module_file_count_max;
        lint_store.register_early_pass(move || Box::new(structure::module_file_count::ModuleFileCount::new(max_files)));
    },
    "underscore_in_module_filename", naming::underscore_in_module_filename::UNDERSCORE_IN_MODULE_FILENAME => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(naming::underscore_in_module_filename::UnderscoreInModuleFilename));
    },
    "non_trait_imports", naming::non_trait_imports::NON_TRAIT_IMPORTS => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::non_trait_imports::NonTraitImports));
    },
    "unbounded_loop", safety::unbounded_loop::UNBOUNDED_LOOP => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_loop::UnboundedLoop));
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
    "explicit_defaults", safety::explicit_defaults::EXPLICIT_DEFAULTS => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::explicit_defaults::ExplicitDefaults));
    },
    "unbounded_channel", safety::unbounded_channel::UNBOUNDED_CHANNEL => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_channel::UnboundedChannel));
    },
    "unbounded_collection_growth", safety::unbounded_collection_growth::UNBOUNDED_COLLECTION_GROWTH => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_collection_growth::UnboundedCollectionGrowth));
    },
    "sentinel_fallback", safety::sentinel_fallback::SENTINEL_FALLBACK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::sentinel_fallback::SentinelFallback));
    },
    "usize_in_public_api", structure::usize_in_public_api::USIZE_IN_PUBLIC_API => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::usize_in_public_api::UsizeInPublicApi));
    },
    "borrowed_argument_types", structure::borrowed_argument_types::BORROWED_ARGUMENT_TYPES => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::borrowed_argument_types::BorrowedArgumentTypes));
    },
    "deref_polymorphism", structure::deref_polymorphism::DEREF_POLYMORPHISM => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::deref_polymorphism::DerefPolymorphism));
    },
    "clone_then_borrow", performance::clone_then_borrow::CLONE_THEN_BORROW => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(performance::clone_then_borrow::CloneThenBorrow));
    },
    "too_many_parameters", structure::too_many_parameters::TOO_MANY_PARAMETERS => |lint_store, cfg| {
        let max_parameters = cfg.max_parameters;
        lint_store.register_late_pass(move |_| Box::new(structure::too_many_parameters::TooManyParameters::new(max_parameters)));
    },
    "negated_predicate", naming::negated_predicate::NEGATED_PREDICATE => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::negated_predicate::NegatedPredicate));
    },
    "bool_naming", naming::bool_naming::BOOL_NAMING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::bool_naming::BoolNaming));
    },
    "ignored_result", safety::ignored_result::IGNORED_RESULT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::ignored_result::IgnoredResult));
    },
    "drop_must_use", safety::drop_must_use::DROP_MUST_USE => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::drop_must_use::DropMustUse));
    },
    "fallible_int_fallback", safety::fallible_int_fallback::FALLIBLE_INT_FALLBACK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::fallible_int_fallback::FallibleIntFallback));
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
        lint_store.register_late_pass(|_| Box::new(safety::no_panic::NoPanic));
    },
    "public_unsafe_api", safety::public_unsafe_api::PUBLIC_UNSAFE_API => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::public_unsafe_api::PublicUnsafeApi));
    },
    "no_todo", safety::no_todo::NO_TODO => |_lint_store, _cfg| {
    },
    "unjustified_no_todo_allow", safety::no_todo::UNJUSTIFIED_NO_TODO_ALLOW => |_lint_store, _cfg| {
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
    register_no_todo_bundle(lint_store, &cfg);
    register_domain_groups(lint_store, &cfg);
    register_compatibility_aliases(lint_store, &cfg);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use serde::Deserialize;

    use super::LINT_INVENTORY;
    use super::default_level_name;
    use super::lint_id_for_name;
    use super::register_compatibility_aliases;
    use super::register_domain_groups;
    use super::register_inventory;
    use crate::config::Config;
    use crate::config::canonical_lint_names;
    use crate::diagnostics;
    use crate::diagnostics::ExampleMode;

    fn inventory_names() -> Vec<&'static str> {
        LINT_INVENTORY.iter().map(|lint| lint.name).collect()
    }

    fn inventory_levels() -> Vec<String> {
        LINT_INVENTORY
            .iter()
            .map(|lint| format!("{}={}", lint.name, default_level_name(lint.lint.default_level)))
            .collect()
    }

    fn diagnostic_metadata_names() -> Vec<&'static str> {
        diagnostics::all_metadata().iter().map(|meta| meta.lint_name).collect()
    }

    fn diagnostic_metadata_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(diagnostics::all_metadata()).expect("serialize diagnostic metadata snapshot")
        )
    }

    fn diagnostic_group_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&diagnostics::group_snapshots()).expect("serialize diagnostic group snapshot")
        )
    }

    fn compatibility_alias_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&diagnostics::compatibility_alias_snapshots())
                .expect("serialize compatibility alias snapshot")
        )
    }

    #[derive(Debug, Deserialize)]
    struct UiFixtureSet {
        all_fixtures: Vec<String>,
        expected_failures: Vec<String>,
        canonical_tool_group_fixtures: Vec<String>,
        compatibility_fixtures: Vec<String>,
        config_fixtures: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct LintFixtureCoverage {
        lint_name: String,
        fixture: String,
    }

    fn read_fixture_json<T: for<'de> Deserialize<'de>>(name: &str) -> T {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read fixture artifact {}: {error}", path.display()));
        serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse fixture artifact {}: {error}", path.display()))
    }

    fn ui_fixture_set() -> UiFixtureSet {
        read_fixture_json("ui_fixture_set.json")
    }

    fn lint_fixture_coverage() -> Vec<LintFixtureCoverage> {
        read_fixture_json("lint_emitting_fixtures.json")
    }

    fn discovered_ui_fixtures() -> Vec<String> {
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let mut stack = vec![ui_dir.clone()];
        let mut fixtures = Vec::new();

        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir)
                .unwrap_or_else(|error| panic!("read UI fixture dir {}: {error}", dir.display()))
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|error| panic!("collect UI fixture dir {}: {error}", dir.display()));
            entries.sort_by_key(|entry| entry.path());

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let relative = path.strip_prefix(&ui_dir).expect("fixture under ui_tests");
                if relative.components().any(|component| component.as_os_str() == "src") {
                    continue;
                }
                fixtures.push(relative.with_extension("").to_string_lossy().replace('\\', "/"));
            }
        }

        fixtures.sort();
        fixtures
    }

    fn fixture_stderr_path(fixture: &str) -> PathBuf {
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let fixture_path = ui_dir.join(format!("{fixture}.rs"));
        let stem = fixture_path.file_stem().expect("fixture file stem");
        ui_dir.join(format!("{}.stderr", stem.to_string_lossy()))
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
    fn diagnostic_metadata_names_match_lint_inventory() {
        assert_eq!(diagnostic_metadata_names(), inventory_names());
    }

    #[test]
    fn diagnostic_metadata_required_examples_are_concrete() {
        for meta in diagnostics::all_metadata() {
            match meta.example_mode {
                ExampleMode::Required => {
                    assert!(meta.example_text.is_some(), "missing required example text for {}", meta.lint_name)
                }
                ExampleMode::None => {
                    assert!(meta.example_text.is_none(), "unexpected example text for {}", meta.lint_name)
                }
            }
        }
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

    #[test]
    fn diagnostic_metadata_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_diagnostic_metadata.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read diagnostic metadata snapshot {}: {error}", snapshot_path.display()));
        let actual = diagnostic_metadata_snapshot();
        assert_eq!(expected, actual, "diagnostic metadata snapshot mismatch");
    }

    fn lint_store_with_groups_and_aliases(cfg: &Config) -> rustc_lint::LintStore {
        let mut store = rustc_lint::LintStore::new();
        register_inventory(&mut store, cfg);
        register_domain_groups(&mut store, cfg);
        register_compatibility_aliases(&mut store, cfg);
        store
    }

    fn expected_group_ids(group_name: &str, cfg: &Config) -> Vec<rustc_lint::LintId> {
        diagnostics::lint_names_for_group(group_name)
            .into_iter()
            .filter(|lint_name| !cfg.lint_is_disabled(lint_name))
            .map(lint_id_for_name)
            .collect()
    }

    #[test]
    fn all_lints_register_canonical_and_bare_names() {
        let store = lint_store_with_groups_and_aliases(&Config::default());

        for lint in LINT_INVENTORY {
            let canonical_name = diagnostics::canonical_lint_name(lint.name);
            let canonical = store
                .find_lints(&canonical_name)
                .unwrap_or_else(|| panic!("missing canonical tool lint `{canonical_name}`"));
            let bare = store
                .find_lints(lint.name)
                .unwrap_or_else(|| panic!("missing bare compatibility alias `{}`", lint.name));
            assert_eq!(
                canonical, bare,
                "bare alias `{}` should resolve to canonical tool lint `{canonical_name}`",
                lint.name
            );
        }
    }

    #[test]
    fn domain_groups_register_expected_members() {
        let cfg = Config::default();
        let store = lint_store_with_groups_and_aliases(&cfg);

        for &group_name in diagnostics::GROUP_NAMES {
            let expected = expected_group_ids(group_name, &cfg);
            let actual = store
                .find_lints(group_name)
                .unwrap_or_else(|| panic!("missing canonical domain group `{group_name}`"));
            assert_eq!(actual, expected.as_slice(), "domain group `{group_name}` membership mismatch");
        }
    }

    #[test]
    fn disabled_lints_are_omitted_from_groups_and_aliases() {
        let mut cfg = Config::default();
        cfg.disabled_lints = vec!["no_unwrap".to_owned(), "no_panic".to_owned()];

        let store = lint_store_with_groups_and_aliases(&cfg);
        for lint_name in ["no_unwrap", "no_panic"] {
            let canonical_name = diagnostics::canonical_lint_name(lint_name);
            assert!(
                store.find_lints(&canonical_name).is_none(),
                "disabled canonical tool lint `{canonical_name}` should not register"
            );
            assert!(store.find_lints(lint_name).is_none(), "disabled bare alias `{lint_name}` should not register");
        }

        let safety = store
            .find_lints(diagnostics::GROUP_SAFETY)
            .expect("safety group should still register for remaining lints");
        let expected_safety = expected_group_ids(diagnostics::GROUP_SAFETY, &cfg);
        assert_eq!(safety, expected_safety.as_slice(), "disabled safety lints should be omitted from the safety group");
    }

    #[test]
    fn diagnostic_group_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_group_memberships.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read diagnostic group snapshot {}: {error}", snapshot_path.display()));
        let actual = diagnostic_group_snapshot();
        assert_eq!(expected, actual, "diagnostic group snapshot mismatch");
    }

    #[test]
    fn compatibility_alias_snapshot_matches() {
        let snapshot_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_compatibility_aliases.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read compatibility alias snapshot {}: {error}", snapshot_path.display()));
        let actual = compatibility_alias_snapshot();
        assert_eq!(expected, actual, "compatibility alias snapshot mismatch");
    }

    #[test]
    fn ui_fixture_set_artifact_covers_the_full_corpus() {
        let fixture_set = ui_fixture_set();
        let discovered = discovered_ui_fixtures();
        let listed = fixture_set.all_fixtures.clone();
        let mut sorted_listed = listed.clone();
        sorted_listed.sort();
        sorted_listed.dedup();

        assert_eq!(listed.len(), sorted_listed.len(), "ui_fixture_set.json should not list duplicate fixtures");
        assert_eq!(sorted_listed, discovered, "ui_fixture_set.json must match the full checked-in UI fixture corpus");

        for fixture in fixture_set
            .expected_failures
            .iter()
            .chain(fixture_set.canonical_tool_group_fixtures.iter())
            .chain(fixture_set.compatibility_fixtures.iter())
            .chain(fixture_set.config_fixtures.iter())
        {
            assert!(
                listed.contains(fixture),
                "ui_fixture_set.json category entry `{fixture}` must appear in all_fixtures"
            );
        }
    }

    #[test]
    fn lint_emitting_fixture_manifest_matches_inventory_and_emits() {
        let coverage = lint_fixture_coverage();
        let coverage_names = coverage.iter().map(|entry| entry.lint_name.as_str()).collect::<Vec<_>>();
        let fixture_set = ui_fixture_set();

        assert_eq!(
            coverage_names,
            diagnostic_metadata_names(),
            "lint_emitting_fixtures.json must cover every lint exactly once in inventory order"
        );

        for entry in coverage {
            assert!(
                fixture_set.all_fixtures.contains(&entry.fixture),
                "lint fixture `{}` for `{}` must be listed in ui_fixture_set.json",
                entry.fixture,
                entry.lint_name
            );

            let stderr_path = fixture_stderr_path(&entry.fixture);
            let stderr = fs::read_to_string(&stderr_path)
                .unwrap_or_else(|error| panic!("read fixture stderr {}: {error}", stderr_path.display()));
            let canonical_name = diagnostics::canonical_lint_name(&entry.lint_name);
            assert!(
                stderr.contains(&canonical_name),
                "fixture `{}` must emit `{canonical_name}` in {}",
                entry.fixture,
                stderr_path.display()
            );
        }
    }

    const AUTOFIX_ROLLOUT_LINTS: &[&str] = &[
        "compound_assertion",
        "bool_naming",
        "acronym_style",
        "numeric_units",
        "unchecked_division",
        "raw_arithmetic_overflow",
        "unchecked_narrowing",
        "platform_dependent_cast",
        "unjustified_allow",
        // Phase 1: single-span name/type replacement
        "negated_predicate",
        "float_for_currency",
        "no_todo",
        "ambient_clock",
        // Phase 2: method-call / expression replacement
        "no_unwrap",
        "sentinel_fallback",
        "ignored_result",
        "contradictory_time",
        // Phase 3: constructor/call replacement
        "unbounded_channel",
        "explicit_defaults",
        "catch_all_on_enum",
        // Phase 4: structural / insertion
        "usize_in_public_api",
        "multi_lock_ordering",
        "compound_condition",
        // Phase 5: complex structural
        "unbounded_collection_growth",
    ];

    const AUTOFIX_OUT_OF_SCOPE_LINTS: &[&str] = &[
        "assertion_density",
        "drop_must_use",
        "fallible_int_fallback",
        "magic_number",
        "underscore_in_module_filename",
        "non_trait_imports",
        "function_length",
        "ambiguous_params",
        "no_recursion",
        "nested_conditionals",
        "no_panic",
        "too_many_parameters",
        "unbounded_loop",
        "unjustified_no_todo_allow",
        "public_unsafe_api",
        "verified_purity",
    ];

    /// Lints that share a source file with a rollout lint and therefore
    /// cannot satisfy the "source must not contain emit_tiger_style_lint_with_suggestion"
    /// assertion for out-of-scope lints.
    const SHARED_SOURCE_EXEMPTIONS: &[&str] = &["unjustified_no_todo_allow"];

    fn expected_autofix_tier(lint_name: &str) -> &'static str {
        match lint_name {
            "compound_assertion" | "bool_naming" => "MachineApplicable",
            "acronym_style"
            | "unchecked_division"
            | "raw_arithmetic_overflow"
            | "unchecked_narrowing"
            | "platform_dependent_cast"
            | "no_unwrap"
            | "sentinel_fallback"
            | "ignored_result"
            | "contradictory_time"
            | "usize_in_public_api"
            | "borrowed_argument_types"
            | "clone_then_borrow" => "MaybeIncorrect",
            "numeric_units"
            | "unjustified_allow"
            | "negated_predicate"
            | "float_for_currency"
            | "no_todo"
            | "ambient_clock"
            | "unbounded_channel"
            | "explicit_defaults"
            | "catch_all_on_enum"
            | "multi_lock_ordering"
            | "compound_condition"
            | "unbounded_collection_growth" => "HasPlaceholders",
            other => panic!("missing autofix tier for `{other}`"),
        }
    }

    fn lint_source_path(lint_name: &str) -> &'static str {
        match lint_name {
            "compound_assertion" => "src/safety/compound_assertion.rs",
            "function_length" => "src/structure/function_length.rs",
            "acronym_style" => "src/naming/acronym_style.rs",
            "numeric_units" => "src/naming/numeric_units.rs",
            "underscore_in_module_filename" => "src/naming/underscore_in_module_filename.rs",
            "non_trait_imports" => "src/naming/non_trait_imports.rs",
            "unbounded_loop" => "src/safety/unbounded_loop.rs",
            "compound_condition" => "src/structure/compound_condition.rs",
            "verified_purity" => "src/purity/verified_purity.rs",
            "unjustified_allow" => "src/structure/unjustified_allow.rs",
            "nested_conditionals" => "src/structure/nested_conditionals.rs",
            "assertion_density" => "src/safety/assertion_density.rs",
            "multi_lock_ordering" => "src/concurrency/multi_lock_ordering.rs",
            "explicit_defaults" => "src/safety/explicit_defaults.rs",
            "unbounded_channel" => "src/safety/unbounded_channel.rs",
            "unbounded_collection_growth" => "src/safety/unbounded_collection_growth.rs",
            "sentinel_fallback" => "src/safety/sentinel_fallback.rs",
            "magic_number" => "src/structure/magic_number.rs",
            "drop_must_use" => "src/safety/drop_must_use.rs",
            "fallible_int_fallback" => "src/safety/fallible_int_fallback.rs",
            "usize_in_public_api" => "src/structure/usize_in_public_api.rs",
            "borrowed_argument_types" => "src/structure/borrowed_argument_types.rs",
            "clone_then_borrow" => "src/performance/clone_then_borrow.rs",
            "too_many_parameters" => "src/structure/too_many_parameters.rs",
            "negated_predicate" => "src/naming/negated_predicate.rs",
            "bool_naming" => "src/naming/bool_naming.rs",
            "ignored_result" => "src/safety/ignored_result.rs",
            "unchecked_narrowing" => "src/safety/unchecked_narrowing.rs",
            "unchecked_division" => "src/safety/unchecked_division.rs",
            "contradictory_time" => "src/purity/contradictory_time.rs",
            "ambient_clock" => "src/purity/ambient_clock.rs",
            "catch_all_on_enum" => "src/safety/catch_all_on_enum.rs",
            "platform_dependent_cast" => "src/structure/platform_dependent_cast.rs",
            "float_for_currency" => "src/naming/float_for_currency.rs",
            "ambiguous_params" => "src/structure/ambiguous_params.rs",
            "raw_arithmetic_overflow" => "src/safety/raw_arithmetic_overflow.rs",
            "no_unwrap" => "src/safety/no_unwrap.rs",
            "no_panic" => "src/safety/no_panic.rs",
            "public_unsafe_api" => "src/safety/public_unsafe_api.rs",
            "no_todo" | "unjustified_no_todo_allow" => "src/safety/no_todo.rs",
            "no_recursion" => "src/safety/no_recursion.rs",
            other => panic!("missing lint source mapping for `{other}`"),
        }
    }

    #[test]
    fn autofix_rollout_scope_matches_change_contract() {
        let coverage = lint_fixture_coverage();
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let fixed_fixtures = fs::read_dir(&ui_dir)
            .unwrap_or_else(|error| panic!("read autofix fixture dir {}: {error}", ui_dir.display()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("collect autofix fixture dir {}: {error}", ui_dir.display()))
            .into_iter()
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|ext| ext.to_str()) == Some("fixed"))
                    .then(|| path.file_stem().expect("fixed fixture file stem").to_string_lossy().into_owned())
            })
            .collect::<Vec<_>>();

        for &lint_name in AUTOFIX_ROLLOUT_LINTS {
            let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_source_path(lint_name));
            let source = fs::read_to_string(&source_path)
                .unwrap_or_else(|error| panic!("read rollout lint source {}: {error}", source_path.display()));
            assert!(
                source.contains("emit_tiger_style_lint_with_suggestion"),
                "rollout lint `{lint_name}` must route through structured suggestion emission in {}",
                source_path.display()
            );
        }

        for &lint_name in AUTOFIX_OUT_OF_SCOPE_LINTS {
            let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_source_path(lint_name));
            let source = fs::read_to_string(&source_path)
                .unwrap_or_else(|error| panic!("read out-of-scope lint source {}: {error}", source_path.display()));
            // Skip source-file check for lints that share a file with a rollout lint
            if !SHARED_SOURCE_EXEMPTIONS.contains(&lint_name) {
                assert!(
                    !source.contains("emit_tiger_style_lint_with_suggestion"),
                    "out-of-scope lint `{lint_name}` must stay diagnostic-only in {}",
                    source_path.display()
                );
            }
            assert!(
                fixed_fixtures.iter().all(|fixture| !fixture.contains(lint_name)),
                "out-of-scope lint `{lint_name}` must stay out of `.fixed` autofix artifacts"
            );
        }

        for entry in coverage {
            if !AUTOFIX_ROLLOUT_LINTS.contains(&entry.lint_name.as_str()) {
                continue;
            }
            let stderr_path = fixture_stderr_path(&entry.fixture);
            let stderr = fs::read_to_string(&stderr_path)
                .unwrap_or_else(|error| panic!("read fixture stderr {}: {error}", stderr_path.display()));
            let expected_tier = expected_autofix_tier(&entry.lint_name);
            assert!(
                stderr.contains(&format!("structured suggestion applicability: {expected_tier}")),
                "rollout lint `{}` must show tier `{expected_tier}` in {}",
                entry.lint_name,
                stderr_path.display()
            );
        }
    }

    #[test]
    fn lint_modules_route_help_and_note_through_diagnostics_module() {
        let lint_dirs = [
            "src/concurrency",
            "src/naming",
            "src/performance",
            "src/purity",
            "src/safety",
            "src/structure",
        ];
        let forbidden = [
            "Tiger Style (",
            "see Tiger Style guide:",
            ".help(",
            ".note(",
            ".span_suggestion(",
            ".span_suggestion_verbose(",
            ".multipart_suggestion(",
            "span_help(",
            "span_note(",
        ];

        for lint_dir in lint_dirs {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_dir);
            for entry in fs::read_dir(&dir).unwrap_or_else(|error| panic!("read lint dir {}: {error}", dir.display())) {
                let path = entry.expect("lint dir entry").path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("read lint source {}: {error}", path.display()));
                for needle in forbidden {
                    assert!(
                        !source.contains(needle),
                        "{} must route help/note text through src/diagnostics.rs; found `{needle}`",
                        path.display()
                    );
                }
            }
        }
    }
}
