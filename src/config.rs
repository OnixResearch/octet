//! Per-project configuration loaded from `dylint.toml` in the workspace root.
//!
//! Projects place a `dylint.toml` next to their `Cargo.toml`:
//!
//! ```toml
//! [octet]
//! excessive_file_length_max = 300
//! function_length_max = 100
//! module_file_count_max = 7
//! nesting_max_depth = 4
//! assertion_density_min_lines = 30
//! assertion_density_min_assertions = 3
//! max_parameters = 7
//! acronym_allowlist = ["IO", "ID"]
//! disabled_lints = ["no_unwrap", "no_panic"]
//!
//! [octet.lint_levels]
//! no_unwrap = "deny"
//! ```

use std::collections::BTreeSet;
use std::collections::HashMap;

use serde::Deserialize;

/// All supported per-lint severity overrides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
}

impl LintLevel {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "allow" => Some(Self::Allow),
            "warn" => Some(Self::Warn),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }

    pub fn rustc_level(self) -> rustc_lint::Level {
        match self {
            Self::Allow => rustc_lint::Level::Allow,
            Self::Warn => rustc_lint::Level::Warn,
            Self::Deny => rustc_lint::Level::Deny,
        }
    }
}

/// All configurable thresholds and allowlists. Missing fields use defaults.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Maximum non-blank, non-comment lines in a function body.
    pub function_length_max: usize,

    /// Maximum physical lines in a production source file.
    pub excessive_file_length_max: usize,

    /// Maximum Rust source files in one module directory.
    pub module_file_count_max: usize,

    /// Maximum if/match nesting depth before warning.
    pub nesting_max_depth: usize,

    /// Minimum code lines before assertion density is checked.
    pub assertion_density_min_lines: usize,

    /// Minimum assertion count required in functions exceeding the line threshold.
    pub assertion_density_min_assertions: usize,

    /// Maximum counted parameters before warning.
    pub max_parameters: usize,

    /// Acronyms that should not trigger the acronym_style lint.
    /// Entries are case-sensitive and matched against the uppercase run
    /// (e.g., `"IO"` allows `IOError`).
    pub acronym_allowlist: Vec<String>,

    /// Lint names that should not register for this workspace.
    pub disabled_lints: Vec<String>,

    /// Raw per-lint severity overrides loaded from `dylint.toml`.
    pub lint_levels: HashMap<String, String>,

    /// Validated per-lint severity overrides used during registration.
    #[serde(skip)]
    validated_lint_levels: HashMap<String, LintLevel>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            function_length_max: 70,
            excessive_file_length_max: 300,
            module_file_count_max: 7,
            nesting_max_depth: 3,
            assertion_density_min_lines: 20,
            assertion_density_min_assertions: 2,
            max_parameters: 5,
            acronym_allowlist: Vec::new(),
            disabled_lints: Vec::new(),
            lint_levels: HashMap::new(),
            validated_lint_levels: HashMap::new(),
        }
    }
}

pub(crate) fn canonical_lint_names() -> Vec<&'static str> {
    crate::LINT_INVENTORY.iter().map(|lint| lint.name).collect()
}

impl Config {
    pub fn lint_is_disabled(&self, lint_name: &str) -> bool {
        self.disabled_lints.iter().any(|disabled| disabled == lint_name)
    }

    pub fn lint_level(&self, lint_name: &str) -> Option<LintLevel> {
        self.validated_lint_levels.get(lint_name).copied()
    }

    fn validate(&mut self) -> Vec<String> {
        let known_lints = canonical_lint_names();
        let disabled_lints = self.disabled_lints.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let mut warnings = Vec::new();
        let mut unknown_disabled_lints = BTreeSet::new();

        for lint_name in &self.disabled_lints {
            if !known_lints.iter().any(|known| known == &lint_name.as_str())
                && unknown_disabled_lints.insert(lint_name.as_str())
            {
                warnings.push(format!("unknown octet lint in `dylint.toml` disabled_lints: `{lint_name}`"));
            }
        }

        self.validated_lint_levels.clear();
        let mut lint_levels = self.lint_levels.iter().collect::<Vec<_>>();
        lint_levels.sort_by_key(|(lint_name, _)| *lint_name);

        for (lint_name, raw_level) in lint_levels {
            if !known_lints.iter().any(|known| known == &lint_name.as_str()) {
                warnings.push(format!("unknown octet lint in `dylint.toml` lint_levels: `{lint_name}`"));
                continue;
            }

            if disabled_lints.contains(lint_name.as_str()) {
                warnings.push(format!(
                    "conflicting octet config in `dylint.toml`: `{lint_name}` appears in both `disabled_lints` and `lint_levels`; ignoring level override"
                ));
                continue;
            }

            let Some(level) = LintLevel::parse(raw_level) else {
                warnings.push(format!(
                    "invalid octet lint level in `dylint.toml` lint_levels for `{lint_name}`: `{raw_level}` (expected `allow`, `warn`, or `deny`)"
                ));
                continue;
            };

            self.validated_lint_levels.insert(lint_name.clone(), level);
        }

        warnings
    }
}

fn prefer_octet_or_legacy_config(octet: Option<Config>, slotcar: Option<Config>, tigerstyle: Option<Config>) -> Config {
    octet.or(slotcar).or(tigerstyle).unwrap_or_default()
}

/// Load the project config from `dylint.toml`, falling back to defaults.
pub fn load(sess: &rustc_session::Session) -> Config {
    let mut cfg = prefer_octet_or_legacy_config(
        dylint_linting::config::<Config>(env!("CARGO_PKG_NAME")).unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
        dylint_linting::config::<Config>("slotcar").unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
        dylint_linting::config::<Config>("tigerstyle").unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
    );
    for warning in cfg.validate() {
        sess.dcx().warn(warning);
    }
    cfg
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::Config;
    use super::LintLevel;
    use super::prefer_octet_or_legacy_config;

    #[derive(Deserialize)]
    struct Wrapper {
        octet: Option<Config>,
        slotcar: Option<Config>,
        tigerstyle: Option<Config>,
    }

    fn parse_config(doc: &str) -> Config {
        let wrapper = toml::from_str::<Wrapper>(doc).expect("parse test dylint.toml");
        prefer_octet_or_legacy_config(wrapper.octet, wrapper.slotcar, wrapper.tigerstyle)
    }

    fn parse_and_validate(doc: &str) -> (Config, Vec<String>) {
        let mut cfg = parse_config(doc);
        let warnings = cfg.validate();
        (cfg, warnings)
    }

    #[test]
    fn legacy_slotcar_config_is_used_when_octet_section_is_absent() {
        let legacy = Config {
            acronym_allowlist: vec!["IO".to_owned()],
            ..Default::default()
        };

        let selected = prefer_octet_or_legacy_config(None, Some(legacy), None);

        assert_eq!(selected.acronym_allowlist, vec!["IO".to_owned()]);
    }

    #[test]
    fn octet_config_wins_over_legacy_slotcar_and_tigerstyle_config() {
        let primary = Config {
            acronym_allowlist: vec!["RPC".to_owned()],
            ..Default::default()
        };
        let legacy = Config {
            acronym_allowlist: vec!["IO".to_owned()],
            ..Default::default()
        };

        let selected = prefer_octet_or_legacy_config(
            Some(primary),
            Some(legacy),
            Some(Config {
                max_parameters: 9,
                ..Default::default()
            }),
        );

        assert_eq!(selected.acronym_allowlist, vec!["RPC".to_owned()]);
    }

    #[test]
    fn omitted_disabled_lints_and_lint_levels_default_to_empty() {
        let cfg = parse_config("[octet]\n");
        assert_eq!(cfg.max_parameters, 5);
        assert!(cfg.disabled_lints.is_empty());
        assert!(cfg.lint_levels.is_empty());
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn explicit_disabled_lints_and_lint_levels_deserialize_successfully() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
max_parameters = 7
disabled_lints = ["no_panic"]

[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        assert!(warnings.is_empty());
        assert_eq!(cfg.max_parameters, 7);
        assert_eq!(cfg.disabled_lints, vec!["no_panic".to_owned()]);
        assert_eq!(cfg.lint_levels.get("no_unwrap"), Some(&"deny".to_owned()));
        assert_eq!(cfg.lint_level("no_unwrap"), Some(LintLevel::Deny));
    }

    #[test]
    fn unknown_disabled_lints_warn_without_failing_validation() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["not_a_real_lint"]
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` disabled_lints: `not_a_real_lint`"]);
        assert!(cfg.lint_levels.is_empty());
    }

    #[test]
    fn mixed_disabled_lints_keeps_valid_entry_while_warning_on_unknown_name() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["not_a_real_lint", "no_unwrap"]
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` disabled_lints: `not_a_real_lint`"]);
        assert!(cfg.lint_is_disabled("no_unwrap"));
    }

    #[test]
    fn unknown_lint_levels_warn_without_failing_validation() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
not_a_real_lint = "deny"
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` lint_levels: `not_a_real_lint`"]);
        assert_eq!(cfg.lint_level("not_a_real_lint"), None);
    }

    #[test]
    fn mixed_lint_levels_ignores_unknown_name_but_keeps_valid_override() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
not_a_real_lint = "deny"
no_unwrap = "warn"
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` lint_levels: `not_a_real_lint`"]);
        assert_eq!(cfg.lint_level("no_unwrap"), Some(LintLevel::Warn));
    }

    #[test]
    fn invalid_lint_level_warns_and_keeps_default_behavior() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
no_unwrap = "fatal"
"#,
        );

        assert_eq!(warnings, vec![
            "invalid octet lint level in `dylint.toml` lint_levels for `no_unwrap`: `fatal` (expected `allow`, `warn`, or `deny`)"
        ]);
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn mixed_lint_levels_ignore_invalid_value_but_keep_valid_override() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
no_unwrap = "fatal"
no_panic = "deny"
"#,
        );

        assert_eq!(warnings, vec![
            "invalid octet lint level in `dylint.toml` lint_levels for `no_unwrap`: `fatal` (expected `allow`, `warn`, or `deny`)"
        ]);
        assert_eq!(cfg.lint_level("no_panic"), Some(LintLevel::Deny));
    }

    #[test]
    fn overlapping_disabled_lints_and_lint_levels_warn_and_keep_lint_disabled() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        assert_eq!(warnings, vec![
            "conflicting octet config in `dylint.toml`: `no_unwrap` appears in both `disabled_lints` and `lint_levels`; ignoring level override"
        ]);
        assert!(cfg.lint_is_disabled("no_unwrap"));
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn raw_lint_levels_preserve_original_strings() {
        let cfg = parse_config(
            r#"
[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        let mut expected = HashMap::new();
        expected.insert("no_unwrap".to_owned(), "deny".to_owned());
        assert_eq!(cfg.lint_levels, expected);
    }

    const INDEXABLE_POLICY_LINTS: &[&str] = &[
        "drop_must_use",
        "fallible_int_fallback",
        "magic_number",
        "excessive_file_length",
        "module_file_count",
        "underscore_in_module_filename",
        "non_trait_imports",
        "renamed_imports",
        "path_segment_repetition",
    ];

    #[test]
    fn indexable_policy_lints_are_accepted_by_config_validation() {
        let disabled = INDEXABLE_POLICY_LINTS
            .iter()
            .map(|lint_name| format!("\"{lint_name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let levels = INDEXABLE_POLICY_LINTS
            .iter()
            .map(|lint_name| format!("{lint_name} = \"deny\""))
            .collect::<Vec<_>>()
            .join("\n");

        let (disabled_cfg, disabled_warnings) =
            parse_and_validate(&format!("[octet]\ndisabled_lints = [{disabled}]\n"));
        assert!(disabled_warnings.is_empty(), "indexable disabled_lints should validate without warnings");
        for lint_name in INDEXABLE_POLICY_LINTS {
            assert!(disabled_cfg.lint_is_disabled(lint_name), "{lint_name} should be accepted in disabled_lints");
        }

        let (levels_cfg, level_warnings) = parse_and_validate(&format!("[octet.lint_levels]\n{levels}\n"));
        assert!(level_warnings.is_empty(), "indexable lint_levels should validate without warnings");
        for lint_name in INDEXABLE_POLICY_LINTS {
            assert_eq!(
                levels_cfg.lint_level(lint_name),
                Some(LintLevel::Deny),
                "{lint_name} should accept lint_levels override"
            );
        }
    }

    #[test]
    fn parsed_octet_section_wins_over_legacy_sections() {
        let cfg = parse_config(
            r#"
[octet]
max_parameters = 7

[slotcar]
max_parameters = 8

[tigerstyle]
max_parameters = 9
"#,
        );

        assert_eq!(cfg.max_parameters, 7);
    }

    #[test]
    fn parsed_slotcar_section_wins_over_tigerstyle_fallback() {
        let cfg = parse_config(
            r#"
[slotcar]
max_parameters = 8

[tigerstyle]
max_parameters = 9
"#,
        );

        assert_eq!(cfg.max_parameters, 8);
    }
}
