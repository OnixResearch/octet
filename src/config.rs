//! Per-project configuration loaded from `dylint.toml` in the workspace root.
//!
//! Projects place a `dylint.toml` next to their `Cargo.toml`:
//!
//! ```toml
//! [tigerstyle]
//! function_length_max = 100
//! nesting_max_depth = 4
//! assertion_density_min_lines = 30
//! assertion_density_min_assertions = 3
//! acronym_allowlist = ["IO", "ID"]
//! disabled_lints = ["no_unwrap", "no_panic"]
//! ```

use std::collections::BTreeSet;

use serde::Deserialize;

/// All configurable thresholds and allowlists. Missing fields use defaults.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Maximum non-blank, non-comment lines in a function body.
    pub function_length_max: usize,

    /// Maximum if/match nesting depth before warning.
    pub nesting_max_depth: usize,

    /// Minimum code lines before assertion density is checked.
    pub assertion_density_min_lines: usize,

    /// Minimum assertion count required in functions exceeding the line threshold.
    pub assertion_density_min_assertions: usize,

    /// Acronyms that should not trigger the acronym_style lint.
    /// Entries are case-sensitive and matched against the uppercase run
    /// (e.g., `"IO"` allows `IOError`).
    pub acronym_allowlist: Vec<String>,

    /// Lint names that should not register for this workspace.
    pub disabled_lints: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            function_length_max: 70,
            nesting_max_depth: 3,
            assertion_density_min_lines: 20,
            assertion_density_min_assertions: 2,
            acronym_allowlist: Vec::new(),
            disabled_lints: Vec::new(),
        }
    }
}

impl Config {
    pub fn lint_is_disabled(&self, lint_name: &str) -> bool {
        self.disabled_lints.iter().any(|disabled| disabled == lint_name)
    }

    fn warn_unknown_disabled_lints(&self, sess: &rustc_session::Session, known_lints: &[&str]) {
        let mut unknown = BTreeSet::new();
        for lint_name in &self.disabled_lints {
            if !known_lints.iter().any(|known| known == &lint_name.as_str()) {
                unknown.insert(lint_name.as_str());
            }
        }

        for lint_name in unknown {
            sess.dcx().warn(format!("unknown tigerstyle lint in `dylint.toml` disabled_lints: `{lint_name}`",));
        }
    }
}

/// Load the project config from dylint.toml, falling back to defaults.
pub fn load(sess: &rustc_session::Session, known_lints: &[&str]) -> Config {
    let cfg = dylint_linting::config_or_default::<Config>(env!("CARGO_PKG_NAME"));
    cfg.warn_unknown_disabled_lints(sess, known_lints);
    cfg
}
