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
//! ```

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            function_length_max: 70,
            nesting_max_depth: 3,
            assertion_density_min_lines: 20,
            assertion_density_min_assertions: 2,
            acronym_allowlist: Vec::new(),
        }
    }
}

/// Load the project config from dylint.toml, falling back to defaults.
pub fn load() -> Config {
    dylint_linting::config_or_default::<Config>(env!("CARGO_PKG_NAME"))
}
