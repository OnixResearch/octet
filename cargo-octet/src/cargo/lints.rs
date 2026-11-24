pub(crate) mod parse;
pub(crate) mod policy;

#[cfg(test)]
mod tests;

use std::io::BufRead as _;
use std::io::Write as _;

const CARGO_MIN_MAJOR: u64 = 1;
const CARGO_1_100_MINOR: u64 = 100;
const MAX_STDERR_CAPTURE_BYTES: usize = 256 * 1_024;
const CARGO_LINT_PREFIX: &str = "cargo::";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CargoVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
    pub(crate) raw: String,
}

impl CargoVersion {
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

    pub(crate) fn is_cargo_1_100_or_newer(&self) -> bool {
        (self.major, self.minor) >= (CARGO_MIN_MAJOR, CARGO_1_100_MINOR)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CargoLintCoverageStatus {
    Checked,
    Partial,
    UnsupportedCargoVersion,
    Unselected,
    Error,
}

impl CargoLintCoverageStatus {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CargoLintAssessment {
    pub(crate) status: CargoLintCoverageStatus,
    pub(crate) cargo_version_raw: String,
    pub(crate) boundary_identity: String,
    pub(crate) scope_summary: String,
    pub(crate) exclusions: Vec<String>,
    pub(crate) boundary_reason: String,
    pub(crate) raw_excerpt: String,
    pub(crate) fail_closed_message: Option<String>,
}

impl CargoLintAssessment {
    pub(crate) fn should_render_in_summary(&self) -> bool {
        self.status != CargoLintCoverageStatus::Unselected
    }
}

pub(crate) struct CapturedStderr {
    pub(crate) text: String,
    pub(crate) is_truncated: bool,
}

pub(crate) fn stream_bounded_stderr(
    reader: impl std::io::Read,
    echo_to_stderr: bool,
) -> Result<CapturedStderr, String> {
    let mut buf_reader = std::io::BufReader::new(reader);
    let mut captured = String::new();
    let mut is_truncated = false;
    let mut line_buf = String::new();
    loop {
        line_buf.clear();
        let bytes_read = buf_reader.read_line(&mut line_buf).map_err(|error| format!("read cargo stderr: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        if echo_to_stderr {
            let mut stderr_lock = std::io::stderr().lock();
            let _ = stderr_lock.write_all(line_buf.as_bytes());
        }
        if captured.len().saturating_add(line_buf.len()) <= MAX_STDERR_CAPTURE_BYTES {
            captured.push_str(&line_buf);
        } else {
            is_truncated = true;
        }
    }
    Ok(CapturedStderr {
        text: captured,
        is_truncated,
    })
}

pub(crate) fn manifest_cargo_lints_json(document: &toml::Table) -> Option<serde_json::Value> {
    let package_lints = policy::cargo_lints_table(document.get("lints"));
    let workspace_lints = document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| policy::cargo_lints_table(workspace.get("lints")));
    if package_lints.is_none() && workspace_lints.is_none() {
        return None;
    }
    let mut map = serde_json::Map::new();
    if let Some(ws) = workspace_lints {
        map.insert("workspace".to_owned(), policy::canonical_toml_json(&toml::Value::Table(ws.clone())));
    }
    if let Some(pkg) = package_lints {
        map.insert("package".to_owned(), policy::canonical_toml_json(&toml::Value::Table(pkg.clone())));
    }
    Some(serde_json::Value::Object(map))
}

pub(crate) fn is_cargo_lint_name(lint_name: &str) -> bool {
    lint_name.starts_with(CARGO_LINT_PREFIX)
}

pub(crate) type ManifestPathNormalizer<'a> = &'a dyn Fn(&str) -> Result<(String, String), String>;

pub(crate) fn findings_from_diagnostics(
    diagnostics: &[parse::ParsedCargoDiagnostic],
    existing_findings: &[crate::output::Finding],
    config_hash: &str,
    cache_enabled: bool,
    normalize_path: ManifestPathNormalizer<'_>,
) -> Result<Vec<crate::output::Finding>, String> {
    let mut added = Vec::new();
    for diagnostic in diagnostics {
        let (file, crate_name) = normalize_path(&diagnostic.raw_path)?;
        let already_present = existing_findings.iter().any(|existing| {
            existing.lint == diagnostic.lint_name
                && existing.file == file
                && existing.line == diagnostic.line
                && existing.column == diagnostic.column
                && existing.message == diagnostic.message
        });
        if already_present {
            continue;
        }
        added.push(build_stderr_finding(
            existing_findings.len() + added.len() + 1,
            diagnostic,
            file,
            crate_name,
            config_hash,
            cache_enabled,
        ));
    }
    Ok(added)
}

fn build_stderr_finding(
    index: usize,
    diagnostic: &parse::ParsedCargoDiagnostic,
    file: String,
    crate_name: String,
    config_hash: &str,
    cache_enabled: bool,
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
        &crate_name,
        &diagnostic.message,
        None,
        &normalized_snippet_hash,
        Some(config_hash),
    );
    let meta = &crate::output::CARGO_LINT_META;
    crate::output::Finding {
        version: crate::output::finding_version(),
        index: format!("F{index}"),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: identity_status.to_owned(),
        normalized_snippet_hash,
        config_hash: config_hash.to_owned(),
        cache_status: if cache_enabled { "pending" } else { "disabled" }.to_owned(),
        lint: diagnostic.lint_name.clone(),
        group: meta.group.to_owned(),
        severity: diagnostic.severity.to_owned(),
        file,
        line: diagnostic.line,
        column: diagnostic.column,
        message: diagnostic.message.clone(),
        suggestion: None,
        applicability: None,
        principle: meta.principle.to_owned(),
        effect_family: meta.effect_family.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name,
    }
}
