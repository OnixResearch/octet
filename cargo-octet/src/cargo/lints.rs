pub(crate) mod ansi;
pub(crate) mod boundary;
pub(crate) mod parse;
pub(crate) mod policy;
pub(crate) mod stream;

#[cfg(test)]
mod tests;

const CARGO_MIN_MAJOR: u64 = 1;
const CARGO_1_100_MINOR: u64 = 100;
const CARGO_LINT_PREFIX: &str = "cargo::";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
    pub(crate) raw: String,
}

impl ToolchainVersion {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let after_prefix = trimmed.strip_prefix("cargo ")?;
        let version_token = after_prefix.split_whitespace().next()?;
        let core_version = version_token.split(['-', '+']).next()?;
        let mut parts = core_version.split('.');
        let major = parts.next()?.parse::<u64>().ok()?;
        let minor = parts.next()?.parse::<u64>().ok()?;
        let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
        Some(Self {
            major,
            minor,
            patch,
            raw: trimmed.to_owned(),
        })
    }

    pub(crate) fn probe(toolchain: &str) -> Option<Self> {
        let output = std::process::Command::new("cargo")
            .env("RUSTUP_TOOLCHAIN", toolchain)
            .arg("--version")
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        Self::parse(&text)
    }

    pub(crate) fn is_1_100_or_newer(&self) -> bool {
        (self.major, self.minor) >= (CARGO_MIN_MAJOR, CARGO_1_100_MINOR)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoverageStatus {
    Checked,
    Partial,
    UnsupportedCargoVersion,
    Unselected,
    Error,
}

impl CoverageStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Partial => "partial",
            Self::UnsupportedCargoVersion => "unsupported-cargo-version",
            Self::Unselected => "unselected",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LintStatus {
    pub(crate) status: &'static str,
    pub(crate) cargo_version: String,
    pub(crate) boundary_identity: String,
    pub(crate) scope_summary: String,
    pub(crate) exclusions: Vec<String>,
    pub(crate) boundary_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LintAssessment {
    pub(crate) status: CoverageStatus,
    pub(crate) cargo_version_raw: String,
    pub(crate) boundary_identity: String,
    pub(crate) scope_summary: String,
    pub(crate) exclusions: Vec<String>,
    pub(crate) boundary_reason: String,
    pub(crate) raw_excerpt: String,
    pub(crate) fail_closed_message: Option<String>,
}

impl LintAssessment {
    pub(crate) fn should_render_in_summary(&self) -> bool {
        self.status != CoverageStatus::Unselected
    }

    pub(crate) fn to_structured_status(&self) -> Option<LintStatus> {
        (self.status != CoverageStatus::Unselected).then(|| LintStatus {
            status: self.status.as_str(),
            cargo_version: self.cargo_version_raw.clone(),
            boundary_identity: self.boundary_identity.clone(),
            scope_summary: self.scope_summary.clone(),
            exclusions: self.exclusions.clone(),
            boundary_reason: self.boundary_reason.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StderrDiagnostic {
    pub(crate) lint_name: String,
    pub(crate) severity: &'static str,
    pub(crate) message: String,
    pub(crate) raw_path: String,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) snippet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct StderrErrorSignals {
    pub(crate) has_compile_error: bool,
    pub(crate) has_manifest_finalize_error: bool,
    pub(crate) has_unrecognized_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct CaptureIntegrity {
    pub(crate) is_truncated: bool,
    pub(crate) has_invalid_utf8: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StderrReport {
    pub(crate) diagnostics: Vec<StderrDiagnostic>,
    pub(crate) has_unused_manifest_key_lints_cargo: bool,
    pub(crate) has_unknown_cargo_lint: bool,
    pub(crate) errors: StderrErrorSignals,
    pub(crate) capture: CaptureIntegrity,
    pub(crate) raw_excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceInheritanceMode {
    Ignore,
    Track,
}

pub(crate) fn has_workspace_lint_inheritance(document: &toml::Table) -> bool {
    document
        .get("lints")
        .and_then(toml::Value::as_table)
        .and_then(|tbl| tbl.get("workspace"))
        .and_then(toml::Value::as_bool)
        == Some(true)
}

pub(crate) fn has_workspace_lint_table(document: &toml::Table) -> bool {
    document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| policy::manifest_lint_table(workspace.get("lints")))
        .is_some()
}

pub(crate) fn manifest_lint_table_json(
    document: &toml::Table,
    inheritance_mode: WorkspaceInheritanceMode,
) -> Option<serde_json::Value> {
    let package_lints = policy::manifest_lint_table(document.get("lints"));
    let workspace_lints = document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| policy::manifest_lint_table(workspace.get("lints")));
    let has_inherited_workspace_lints = (inheritance_mode == WorkspaceInheritanceMode::Track
        || workspace_lints.is_some())
        && has_workspace_lint_inheritance(document);
    if package_lints.is_none() && workspace_lints.is_none() && !has_inherited_workspace_lints {
        return None;
    }
    let mut map = serde_json::Map::new();
    if let Some(ws) = workspace_lints {
        map.insert("workspace".to_owned(), crate::determinism::canonical_toml_json(&toml::Value::Table(ws.clone())));
    }
    if let Some(pkg) = package_lints {
        map.insert("package".to_owned(), crate::determinism::canonical_toml_json(&toml::Value::Table(pkg.clone())));
    }
    if has_inherited_workspace_lints {
        map.insert("workspace_inherited".to_owned(), serde_json::Value::Bool(true));
    }
    Some(serde_json::Value::Object(map))
}

pub(crate) fn is_native_lint_name(lint_name: &str) -> bool {
    lint_name.starts_with(CARGO_LINT_PREFIX)
}

pub(crate) type ManifestPathNormalizer<'a> = &'a dyn Fn(&str) -> Result<(String, String), String>;

pub(crate) fn findings_from_diagnostics(
    diagnostics: &[StderrDiagnostic],
    existing_findings: &[crate::output::Finding],
    config_hash: &str,
    is_cache_enabled: bool,
    normalize_path: ManifestPathNormalizer<'_>,
) -> Result<Vec<crate::output::Finding>, String> {
    let mut added = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        let (file, crate_name) = normalize_path(&diagnostic.raw_path)?;
        let is_already_present = existing_findings.iter().any(|existing| {
            existing.lint == diagnostic.lint_name
                && existing.file == file
                && existing.line == diagnostic.line
                && existing.column == diagnostic.column
                && existing.message == diagnostic.message
        });
        if is_already_present {
            continue;
        }
        added.push(build_stderr_finding(existing_findings.len() + added.len() + 1, diagnostic, StderrFindingContext {
            file,
            crate_name,
            config_hash,
            is_cache_enabled,
        }));
    }
    Ok(added)
}

struct StderrFindingContext<'a> {
    file: String,
    crate_name: String,
    config_hash: &'a str,
    is_cache_enabled: bool,
}

fn build_stderr_finding(
    index: usize,
    diagnostic: &StderrDiagnostic,
    context: StderrFindingContext<'_>,
) -> crate::output::Finding {
    let normalized_snippet_hash = diagnostic
        .snippet
        .as_deref()
        .filter(|snippet| !snippet.trim().is_empty())
        .map_or_else(|| "unsupported".to_owned(), crate::determinism::normalized_snippet_hash);
    let identity_status = if normalized_snippet_hash == "unsupported" {
        "unsupported"
    } else {
        "computed"
    };
    let semantic_id = crate::determinism::semantic_finding_id(
        &diagnostic.lint_name,
        &context.crate_name,
        &diagnostic.message,
        None,
        &normalized_snippet_hash,
        Some(context.config_hash),
    );
    let meta = &crate::output::CARGO_LINT_META;
    crate::output::Finding {
        version: crate::output::finding_version(),
        index: format!("F{index}"),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: identity_status.to_owned(),
        normalized_snippet_hash,
        config_hash: context.config_hash.to_owned(),
        cache_status: if context.is_cache_enabled {
            "pending"
        } else {
            "disabled"
        }
        .to_owned(),
        lint: diagnostic.lint_name.clone(),
        group: meta.group.to_owned(),
        severity: diagnostic.severity.to_owned(),
        file: context.file,
        line: diagnostic.line,
        column: diagnostic.column,
        message: diagnostic.message.clone(),
        suggestion: None,
        applicability: None,
        principle: meta.principle.to_owned(),
        effect_family: meta.effect_family.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name: context.crate_name,
    }
}
