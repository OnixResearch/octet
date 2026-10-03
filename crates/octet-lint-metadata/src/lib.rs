#[path = "identity.rs"]
pub mod identity;
pub mod nominal_domain {
    pub use crate::identity::*;
}
mod catalog;
mod rules;
mod schema;
pub mod strict_profile;

pub const CANONICAL_NAMESPACE: &str = "tigerstyle";
pub const GROUP_SAFETY: &str = "tigerstyle::safety";
pub const GROUP_NAMING: &str = "tigerstyle::naming";
pub const GROUP_PERFORMANCE: &str = "tigerstyle::performance";
pub const GROUP_STRUCTURE: &str = "tigerstyle::structure";
pub const GROUP_PURITY: &str = "tigerstyle::purity";
pub const GROUP_CONCURRENCY: &str = "tigerstyle::concurrency";
pub const GROUP_NAMES: &[&str] = &[
    GROUP_SAFETY,
    GROUP_NAMING,
    GROUP_PERFORMANCE,
    GROUP_STRUCTURE,
    GROUP_PURITY,
    GROUP_CONCURRENCY,
];

pub const DEFAULT_ALLOW: &str = "allow";
pub const DEFAULT_DENY: &str = "deny";
pub const DEFAULT_WARN: &str = "warn";

pub const EFFECT_CONCURRENCY: &str = "concurrency";
pub const EFFECT_FAILURE: &str = "failure";
pub const EFFECT_RESOURCE: &str = "resource";
pub const EFFECT_TIME: &str = "time";
pub const EFFECT_UNKNOWN: &str = "unknown";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExampleMode {
    Required,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct LintMetadata {
    pub lint_name: &'static str,
    pub default_level: &'static str,
    pub principle: &'static str,
    pub guide_section: &'static str,
    pub domain_group: &'static str,
    pub guidance: &'static str,
    pub docs_summary: &'static str,
    pub effect_family: &'static str,
    pub example_mode: ExampleMode,
    pub example_text: Option<&'static str>,
}

pub const LINT_METADATA: &[LintMetadata] = &catalog::COLLECTED;

#[must_use]
pub fn all_metadata() -> &'static [LintMetadata] {
    LINT_METADATA
}

#[must_use]
pub fn canonical_lint_name(lint_name: &str) -> String {
    format!("{CANONICAL_NAMESPACE}::{lint_name}")
}

#[must_use]
pub fn metadata_for_lint(lint_name: &str) -> Option<&'static LintMetadata> {
    LINT_METADATA.iter().find(|meta| meta.lint_name == lint_name)
}

#[must_use]
pub fn lint_names_for_group(group_name: &str) -> Vec<&'static str> {
    LINT_METADATA
        .iter()
        .filter(|meta| meta.domain_group == group_name)
        .map(|meta| meta.lint_name)
        .collect()
}
