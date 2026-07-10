use std::path::Path;
use std::path::PathBuf;

use crate::determinism;
use crate::fingerprint;
use crate::output;

pub const FUNCTION_ADDRESS_DOMAIN: &str = "octet.function-address.v1";
pub const FUNCTION_ADDRESS_CANONICALIZER: &str = "octet-source-token-function-address-v1";

const LINT_FUNCTION_ADDRESS_MISSING: &str = "function_address_missing";
const LINT_FUNCTION_ADDRESS_UNSUPPORTED: &str = "function_address_unsupported";
const FUNCTION_ADDRESS_GROUP: &str = "octet::function_address";
const FUNCTION_ADDRESS_PRINCIPLE: &str = "content-addressed function identity";
const FUNCTION_ADDRESS_EFFECT_FAMILY: &str = "source-identity";
const OBSERVE_SEVERITY: &str = "note";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";
const DEFAULT_LINE: u32 = 1;
const DEFAULT_COLUMN: u32 = 1;
const LINE_OFFSET: u32 = 1;
const COLUMN_OFFSET: u32 = 1;
const NEWLINE_BYTE: u8 = b'\n';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionAddressMode {
    Off,
    Observe,
    Warn,
    Deny,
}

impl FunctionAddressMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "off" => Ok(Self::Off),
            "observe" => Ok(Self::Observe),
            "warn" => Ok(Self::Warn),
            "deny" => Ok(Self::Deny),
            other => Err(format!(
                "unsupported function-address mode `{other}`; expected `off`, `observe`, `warn`, or `deny`"
            )),
        }
    }

    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

impl Default for FunctionAddressMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionAddressCheck {
    pub mode: FunctionAddressMode,
    pub paths: Vec<PathBuf>,
}

impl FunctionAddressCheck {
    pub fn off() -> Self {
        Self {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
        }
    }

    pub fn enabled_paths(&self, workspace_root: &Path) -> Vec<PathBuf> {
        if self.paths.is_empty() {
            vec![workspace_root.to_path_buf()]
        } else {
            self.paths.clone()
        }
    }
}

impl Default for FunctionAddressCheck {
    fn default() -> Self {
        Self::off()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionAddressRecord {
    pub path: String,
    pub name: String,
    pub line: u32,
    pub column: u32,
    pub function_address: String,
    pub normalized_body_hash: String,
    pub dependency_set_hash: String,
    pub dependency_identity_status: String,
    pub source_identity: String,
    pub caveats: Vec<String>,
}

pub fn collect_findings(
    check: &FunctionAddressCheck,
    workspace_root: &Path,
    config_hash: &str,
) -> Result<Vec<output::Finding>, String> {
    if !check.mode.is_enabled() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for path in check.enabled_paths(workspace_root) {
        let absolute = if path.is_absolute() {
            path
        } else {
            workspace_root.join(path)
        };
        fingerprint::collect_rust_files(&absolute, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut findings = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).map_err(|error| format!("read {}: {error}", file.display()))?;
        let source_identity = source_identity(&source);
        for span in fingerprint::find_functions(&source) {
            let function_source = &source[span.start..span.end];
            let record = function_address_record(&file, &source, &source_identity, &span, function_source);
            if span.marker.is_none() {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_MISSING,
                    "function has no reviewed content-address marker; run observe mode to baseline or add a reviewed receipt",
                    config_hash,
                ));
            }
            if record.dependency_identity_status == "unsupported-macro-context" {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_UNSUPPORTED,
                    "function contains macro context that blocks dependency-closure identity; add reviewed caveat or exclusion before deny mode",
                    config_hash,
                ));
            }
        }
    }

    Ok(findings)
}

fn function_address_record(
    path: &Path,
    source: &str,
    source_identity: &str,
    span: &fingerprint::FunctionSpan,
    function_source: &str,
) -> FunctionAddressRecord {
    let normalized = fingerprint::normalize_tokens(function_source);
    let dependency_set_hash = fingerprint::dependency_set_hash(function_source);
    let dependency_identity_status = fingerprint::dependency_identity_status(function_source).to_owned();
    let function_address = function_address(&normalized, &dependency_set_hash, &dependency_identity_status);
    let (line, column) = line_column_for_offset(source, span.start);
    let mut caveats = Vec::new();
    if span.macro_calls {
        caveats.push("macro-context-review-required".to_owned());
    }

    FunctionAddressRecord {
        path: determinism::normalize_path(path),
        name: span.name.clone(),
        line,
        column,
        function_address,
        normalized_body_hash: determinism::full_hash(&normalized),
        dependency_set_hash,
        dependency_identity_status,
        source_identity: source_identity.to_owned(),
        caveats,
    }
}

fn function_address(normalized: &str, dependency_set_hash: &str, dependency_identity_status: &str) -> String {
    let payload = serde_json::json!({
        "domain": FUNCTION_ADDRESS_DOMAIN,
        "canonicalizer": FUNCTION_ADDRESS_CANONICALIZER,
        "normalized": normalized,
        "dependency_set_hash": dependency_set_hash,
        "dependency_identity_status": dependency_identity_status,
        "tool": env!("CARGO_PKG_VERSION"),
    });
    determinism::full_hash(&payload.to_string())
}

fn source_identity(source: &str) -> String {
    determinism::full_hash(source)
}

fn finding_for_record(
    record: &FunctionAddressRecord,
    mode: FunctionAddressMode,
    lint: &str,
    message: &str,
    config_hash: &str,
) -> output::Finding {
    let severity = severity_for_mode(mode);
    let semantic_id = determinism::semantic_finding_id(
        lint,
        "function-address",
        message,
        None,
        &record.function_address,
        Some(config_hash),
    );
    output::Finding {
        version: output::finding_version(),
        index: String::new(),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: "computed".to_owned(),
        normalized_snippet_hash: record.function_address.clone(),
        config_hash: config_hash.to_owned(),
        cache_status: "disabled".to_owned(),
        lint: lint.to_owned(),
        group: FUNCTION_ADDRESS_GROUP.to_owned(),
        severity: severity.to_owned(),
        file: record.path.clone(),
        line: record.line,
        column: record.column,
        message: format!(
            "{message}; function={} function_address={} source_identity={} canonicalizer={}",
            record.name, record.function_address, record.source_identity, FUNCTION_ADDRESS_CANONICALIZER
        ),
        suggestion: None,
        applicability: None,
        principle: FUNCTION_ADDRESS_PRINCIPLE.to_owned(),
        effect_family: FUNCTION_ADDRESS_EFFECT_FAMILY.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name: "function-address".to_owned(),
    }
}

fn severity_for_mode(mode: FunctionAddressMode) -> &'static str {
    match mode {
        FunctionAddressMode::Off => OBSERVE_SEVERITY,
        FunctionAddressMode::Observe => OBSERVE_SEVERITY,
        FunctionAddressMode::Warn => WARNING_SEVERITY,
        FunctionAddressMode::Deny => ERROR_SEVERITY,
    }
}

fn line_column_for_offset(source: &str, offset: usize) -> (u32, u32) {
    let mut line = DEFAULT_LINE;
    let mut column = DEFAULT_COLUMN;
    for byte in source.as_bytes().iter().take(offset) {
        if *byte == NEWLINE_BYTE {
            line = line.saturating_add(LINE_OFFSET);
            column = DEFAULT_COLUMN;
        } else {
            column = column.saturating_add(COLUMN_OFFSET);
        }
    }
    (line, column)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn adoption_modes_parse_and_classify() {
        assert_eq!(FunctionAddressMode::parse("off"), Ok(FunctionAddressMode::Off));
        assert_eq!(FunctionAddressMode::parse("observe"), Ok(FunctionAddressMode::Observe));
        assert_eq!(FunctionAddressMode::parse("warn"), Ok(FunctionAddressMode::Warn));
        assert_eq!(FunctionAddressMode::parse("deny"), Ok(FunctionAddressMode::Deny));
        assert!(FunctionAddressMode::parse("strict").is_err());
    }

    #[test]
    fn observe_mode_emits_non_failing_missing_marker_finding() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![source],
        };

        let findings = collect_findings(&check, temp.path(), "config").expect("collect findings");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MISSING);
        assert_eq!(findings[0].severity, OBSERVE_SEVERITY);
        assert!(findings[0].message.contains(FUNCTION_ADDRESS_CANONICALIZER));
    }

    #[test]
    fn deny_mode_promotes_missing_marker_to_error() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
        };

        let findings = collect_findings(&check, temp.path(), "config").expect("collect findings");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, ERROR_SEVERITY);
    }

    #[test]
    fn macro_context_gets_unsupported_finding() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() { println!(\"hi\"); }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: vec![PathBuf::from("lib.rs")],
        };

        let findings = collect_findings(&check, temp.path(), "config").expect("collect findings");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_MISSING));
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_UNSUPPORTED));
        assert!(findings.iter().all(|finding| finding.severity == WARNING_SEVERITY));
    }
}
