use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::determinism;
use crate::fingerprint;
use crate::output;

pub const FUNCTION_ADDRESS_ARTIFACT_NAME: &str = "function-addresses.json";
pub const FUNCTION_ADDRESS_DOMAIN: &str = "octet.function-address.v1";
pub const FUNCTION_ADDRESS_CANONICALIZER: &str = "octet-source-token-function-address-v1";

const FUNCTION_ADDRESS_SCHEMA_VERSION: u32 = 1;
const LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR: &str = "function_address_duplicate_locator";
const LINT_FUNCTION_ADDRESS_MALFORMED: &str = "function_address_malformed";
const LINT_FUNCTION_ADDRESS_MISSING: &str = "function_address_missing";
const LINT_FUNCTION_ADDRESS_OVERCLAIM: &str = "function_address_overclaim";
const LINT_FUNCTION_ADDRESS_STALE: &str = "function_address_stale";
const LINT_FUNCTION_ADDRESS_UNSUPPORTED: &str = "function_address_unsupported";
const FUNCTION_ADDRESS_GROUP: &str = "octet::function_address";
const FUNCTION_ADDRESS_PRINCIPLE: &str = "content-addressed function identity";
const FUNCTION_ADDRESS_EFFECT_FAMILY: &str = "source-identity";
const FUNCTION_ADDRESS_RECEIPT_CLASS: &str = "function-address-receipt";
const FUNCTION_ADDRESS_PRODUCER: &str = "octet";
const FUNCTION_ADDRESS_PRODUCER_ROLE: &str = "rust-source-shape-producer";
const FUNCTION_ADDRESS_SOURCE_LANGUAGE: &str = "rust";
const KAMACITE_CANONICAL_IDENTITY_OWNER: &str = "kamacite-preserves-bytes";
const KAMACITE_COMPATIBILITY_PROJECTION: &str = "kamacite.function-address-receipt.v1.compat-json";
const IDENTITY_ALGORITHM_BLAKE3: &str = "blake3";
const OBSERVE_SEVERITY: &str = "note";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";
const DEFAULT_LINE: u32 = 1;
const DEFAULT_COLUMN: u32 = 1;
const LINE_OFFSET: u32 = 1;
const COLUMN_OFFSET: u32 = 1;
const NEWLINE_BYTE: u8 = b'\n';
const BLAKE3_PREFIX: &str = "b3:";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const LOCATOR_SEPARATOR: &str = "\u{1f}";
const BASELINE_RECEIPT_PATH: &str = "function-address-baseline";
const BASELINE_RECEIPT_LOCATOR: &str = "receipt";
const NON_CLAIM_SEMANTIC_CORRECTNESS: &str = "not Rust semantic correctness";
const NON_CLAIM_COMPILER_CORRECTNESS: &str = "not compiler correctness";
const NON_CLAIM_WHOLE_PROGRAM_SAFETY: &str = "not whole-program safety";
const NON_CLAIM_RELEASE_ELIGIBILITY: &str = "not release eligibility";
const REQUIRED_NON_CLAIMS: &[&str] = &[
    NON_CLAIM_SEMANTIC_CORRECTNESS,
    NON_CLAIM_COMPILER_CORRECTNESS,
    NON_CLAIM_WHOLE_PROGRAM_SAFETY,
    NON_CLAIM_RELEASE_ELIGIBILITY,
];
const FORBIDDEN_CLAIM_FRAGMENTS: &[&str] = &[
    "semantic correctness",
    "compiler correctness",
    "runtime behavior",
    "whole-program safety",
    "verifier soundness",
    "release eligibility",
];

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
    pub baseline: Option<PathBuf>,
}

impl FunctionAddressCheck {
    pub fn off() -> Self {
        Self {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
            baseline: None,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
    pub review: Option<String>,
    pub evidence: Option<String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceEnvelopeRecord {
    pub path: String,
    pub language: &'static str,
    pub source_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionAddressProducer {
    pub name: &'static str,
    pub role: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct KamaciteProjection {
    pub artifact_class: &'static str,
    pub compatibility_projection: &'static str,
    pub canonical_identity_owner: &'static str,
    pub identity_algorithm: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionAddressReport {
    pub schema: &'static str,
    pub schema_version: u32,
    pub receipt_class: &'static str,
    pub mode: &'static str,
    pub domain: &'static str,
    pub canonicalizer: &'static str,
    pub producer: FunctionAddressProducer,
    pub tool: &'static str,
    pub source_envelopes: Vec<SourceEnvelopeRecord>,
    pub records: Vec<FunctionAddressRecord>,
    pub diagnostic_count: usize,
    pub kamacite_projection: KamaciteProjection,
    pub non_claims: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct FunctionAddressRun {
    pub findings: Vec<output::Finding>,
    pub artifact_json: String,
}

#[derive(Debug, Deserialize)]
struct BaselineReport {
    records: Vec<BaselineRecord>,
    non_claims: Option<Vec<String>>,
    claims: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct BaselineRecord {
    path: String,
    name: String,
    function_address: String,
    source_identity: Option<String>,
}

#[derive(Debug, Default)]
struct BaselineIndex {
    addresses: BTreeMap<String, String>,
    source_identities: BTreeMap<String, String>,
    issues: Vec<BaselineIssue>,
}

#[derive(Debug)]
struct BaselineIssue {
    lint: &'static str,
    path: String,
    name: String,
    function_address: String,
    message: &'static str,
}

pub fn collect_run(
    check: &FunctionAddressCheck,
    workspace_root: &Path,
    config_hash: &str,
) -> Result<Option<FunctionAddressRun>, String> {
    if !check.mode.is_enabled() {
        return Ok(None);
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

    let baseline = load_baseline(check.baseline.as_deref())?;
    let mut records = Vec::new();
    let mut findings = baseline
        .issues
        .iter()
        .map(|issue| finding_for_baseline_issue(issue, check.mode, config_hash))
        .collect::<Vec<_>>();
    let mut source_envelopes = BTreeMap::new();
    let mut seen_locators = BTreeMap::new();
    for file in files {
        let source = std::fs::read_to_string(&file).map_err(|error| format!("read {}: {error}", file.display()))?;
        let source_identity = source_identity(&source);
        let normalized_path = determinism::normalize_path(&file);
        source_envelopes.insert(normalized_path.clone(), SourceEnvelopeRecord {
            path: normalized_path,
            language: FUNCTION_ADDRESS_SOURCE_LANGUAGE,
            source_identity: source_identity.clone(),
        });
        for span in fingerprint::find_functions(&source) {
            let function_source = &source[span.start..span.end];
            let record = function_address_record(&file, &source, &source_identity, &span, function_source);
            let locator = function_locator(&record.path, &record.name);
            if seen_locators.insert(locator.clone(), record.function_address.clone()).is_some() {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR,
                    "function locator appears more than once in the checked scope",
                    config_hash,
                ));
            }
            if let Some(marker_finding) =
                marker_finding_for_record(&record, span.marker.as_ref(), check.mode, config_hash)
            {
                findings.push(marker_finding);
            }
            if let Some(expected_address) = baseline.addresses.get(&locator) {
                if expected_address != &record.function_address {
                    findings.push(finding_for_record(
                        &record,
                        check.mode,
                        LINT_FUNCTION_ADDRESS_STALE,
                        "function address changed from reviewed baseline; update the reviewed receipt or inspect the change",
                        config_hash,
                    ));
                }
                if let Some(expected_source_identity) = baseline.source_identities.get(&locator)
                    && expected_source_identity != &record.source_identity
                {
                    findings.push(finding_for_record(
                        &record,
                        check.mode,
                        LINT_FUNCTION_ADDRESS_STALE,
                        "source envelope identity changed from reviewed baseline; update the reviewed receipt or inspect the change",
                        config_hash,
                    ));
                }
            } else if span.marker.is_none() {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_MISSING,
                    "function has no reviewed content-address marker or baseline entry; run observe mode to baseline or add a reviewed receipt",
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
            records.push(record);
        }
    }

    let report = FunctionAddressReport {
        schema: FUNCTION_ADDRESS_DOMAIN,
        schema_version: FUNCTION_ADDRESS_SCHEMA_VERSION,
        receipt_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
        mode: mode_name(check.mode),
        domain: FUNCTION_ADDRESS_DOMAIN,
        canonicalizer: FUNCTION_ADDRESS_CANONICALIZER,
        producer: FunctionAddressProducer {
            name: FUNCTION_ADDRESS_PRODUCER,
            role: FUNCTION_ADDRESS_PRODUCER_ROLE,
        },
        tool: env!("CARGO_PKG_VERSION"),
        source_envelopes: source_envelopes.into_values().collect(),
        records,
        diagnostic_count: findings.len(),
        kamacite_projection: KamaciteProjection {
            artifact_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
            compatibility_projection: KAMACITE_COMPATIBILITY_PROJECTION,
            canonical_identity_owner: KAMACITE_CANONICAL_IDENTITY_OWNER,
            identity_algorithm: IDENTITY_ALGORITHM_BLAKE3,
        },
        non_claims: REQUIRED_NON_CLAIMS.to_vec(),
    };
    let artifact_json = render_report_json(&report)?;

    Ok(Some(FunctionAddressRun {
        findings,
        artifact_json,
    }))
}

pub fn render_report_json(report: &FunctionAddressReport) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(report).map_err(|error| format!("serialize function-address report: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn load_baseline(path: Option<&Path>) -> Result<BaselineIndex, String> {
    let Some(path) = path else {
        return Ok(BaselineIndex::default());
    };
    let contents = std::fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let report: BaselineReport =
        serde_json::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))?;
    let mut addresses = BTreeMap::new();
    let mut source_identities = BTreeMap::new();
    let mut issues = Vec::new();
    for record in report.records {
        let locator = function_locator(&record.path, &record.name);
        if !is_blake3_identity(&record.function_address) {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_MALFORMED,
                path: record.path.clone(),
                name: record.name.clone(),
                function_address: determinism::full_hash(&record.function_address),
                message: "baseline receipt contains a malformed BLAKE3 function address",
            });
        }
        if let Some(source_identity) = record.source_identity.as_ref() {
            source_identities.insert(locator.clone(), source_identity.clone());
        }
        if addresses.insert(locator, record.function_address.clone()).is_some() {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR,
                path: record.path,
                name: record.name,
                function_address: record.function_address,
                message: "baseline receipt contains a duplicate function locator",
            });
        }
    }
    if !baseline_non_claims_cover_required(report.non_claims.as_deref()) {
        issues.push(BaselineIssue {
            lint: LINT_FUNCTION_ADDRESS_OVERCLAIM,
            path: BASELINE_RECEIPT_PATH.to_owned(),
            name: BASELINE_RECEIPT_LOCATOR.to_owned(),
            function_address: determinism::full_hash(BASELINE_RECEIPT_LOCATOR),
            message: "baseline receipt is missing required non-claim boundaries",
        });
    }
    for claim in report.claims.unwrap_or_default() {
        if is_forbidden_claim(&claim) {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_OVERCLAIM,
                path: BASELINE_RECEIPT_PATH.to_owned(),
                name: BASELINE_RECEIPT_LOCATOR.to_owned(),
                function_address: determinism::full_hash(&claim),
                message: "baseline receipt contains forbidden overclaim text",
            });
        }
    }
    Ok(BaselineIndex {
        addresses,
        source_identities,
        issues,
    })
}

fn function_locator(path: &str, name: &str) -> String {
    format!("{path}{LOCATOR_SEPARATOR}{name}")
}

fn mode_name(mode: FunctionAddressMode) -> &'static str {
    match mode {
        FunctionAddressMode::Off => "off",
        FunctionAddressMode::Observe => "observe",
        FunctionAddressMode::Warn => "warn",
        FunctionAddressMode::Deny => "deny",
    }
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
        review: span.marker.as_ref().and_then(|marker| marker.review().map(str::to_owned)),
        evidence: span.marker.as_ref().and_then(|marker| marker.evidence().map(str::to_owned)),
        caveats,
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn is_blake3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_blake3_hex)
}

fn marker_function_address(marker: &fingerprint::Marker) -> String {
    format!("{BLAKE3_PREFIX}{}", marker.stored_hash())
}

fn marker_digest(function_address: &str) -> &str {
    function_address.strip_prefix(BLAKE3_PREFIX).unwrap_or(function_address)
}

fn baseline_non_claims_cover_required(non_claims: Option<&[String]>) -> bool {
    let Some(non_claims) = non_claims else {
        return false;
    };
    REQUIRED_NON_CLAIMS
        .iter()
        .all(|required| non_claims.iter().any(|non_claim| non_claim.as_str() == *required))
}

fn is_forbidden_claim(claim: &str) -> bool {
    let lowered = claim.to_ascii_lowercase();
    FORBIDDEN_CLAIM_FRAGMENTS.iter().any(|fragment| lowered.contains(fragment))
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

fn marker_finding_for_record(
    record: &FunctionAddressRecord,
    marker: Option<&fingerprint::Marker>,
    mode: FunctionAddressMode,
    config_hash: &str,
) -> Option<output::Finding> {
    let marker = marker?;
    if !is_blake3_hex(marker.stored_hash()) {
        return Some(finding_for_record(
            record,
            mode,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address marker contains a malformed BLAKE3 digest",
            config_hash,
        ));
    }
    if marker_function_address(marker) != record.function_address {
        return Some(finding_for_record(
            record,
            mode,
            LINT_FUNCTION_ADDRESS_STALE,
            "function-address marker does not match the computed function address",
            config_hash,
        ));
    }
    None
}

fn finding_for_baseline_issue(issue: &BaselineIssue, mode: FunctionAddressMode, config_hash: &str) -> output::Finding {
    let record = FunctionAddressRecord {
        path: issue.path.clone(),
        name: issue.name.clone(),
        line: DEFAULT_LINE,
        column: DEFAULT_COLUMN,
        function_address: issue.function_address.clone(),
        normalized_body_hash: issue.function_address.clone(),
        dependency_set_hash: issue.function_address.clone(),
        dependency_identity_status: "baseline-receipt".to_owned(),
        source_identity: issue.function_address.clone(),
        review: None,
        evidence: None,
        caveats: Vec::new(),
    };
    finding_for_record(&record, mode, issue.lint, issue.message, config_hash)
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

    fn collect_test_findings(
        check: &FunctionAddressCheck,
        workspace_root: &Path,
        config_hash: &str,
    ) -> Vec<output::Finding> {
        collect_test_run(check, workspace_root, config_hash).findings
    }

    fn collect_test_run(check: &FunctionAddressCheck, workspace_root: &Path, config_hash: &str) -> FunctionAddressRun {
        collect_run(check, workspace_root, config_hash)
            .expect("collect run")
            .expect("enabled mode returns run")
    }

    fn artifact_value(run: &FunctionAddressRun) -> serde_json::Value {
        serde_json::from_str(&run.artifact_json).expect("parse artifact JSON")
    }

    fn valid_baseline_json(records: serde_json::Value) -> String {
        serde_json::json!({
            "records": records,
            "non_claims": REQUIRED_NON_CLAIMS,
        })
        .to_string()
    }

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
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

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
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

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
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_MISSING));
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_UNSUPPORTED));
        assert!(findings.iter().all(|finding| finding.severity == WARNING_SEVERITY));
    }

    #[test]
    fn enabled_run_renders_function_address_artifact_with_non_claims() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_run(&check, temp.path(), "config").expect("collect run").expect("enabled mode returns run");

        assert_eq!(run.findings.len(), 1);
        let artifact = artifact_value(&run);
        assert_eq!(artifact["schema"], FUNCTION_ADDRESS_DOMAIN);
        assert_eq!(artifact["receipt_class"], FUNCTION_ADDRESS_RECEIPT_CLASS);
        assert_eq!(artifact["producer"]["role"], FUNCTION_ADDRESS_PRODUCER_ROLE);
        assert_eq!(artifact["kamacite_projection"]["compatibility_projection"], KAMACITE_COMPATIBILITY_PROJECTION);
        assert!(run.artifact_json.contains(NON_CLAIM_SEMANTIC_CORRECTNESS));
        assert!(run.artifact_json.contains("\"source_envelopes\""));
        assert!(run.artifact_json.contains("\"records\""));
    }

    #[test]
    fn baseline_suppresses_unchanged_missing_marker_debt() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_run(&observe, temp.path(), "config")
            .expect("collect baseline")
            .expect("enabled mode returns run");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert!(findings.is_empty());
    }

    #[test]
    fn baseline_reports_stale_function_address() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_run(&observe, temp.path(), "config")
            .expect("collect baseline")
            .expect("enabled mode returns run");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        std::fs::write(&source, "fn f() -> u32 { 2 }\n").expect("write changed source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|finding| finding.lint == LINT_FUNCTION_ADDRESS_STALE));
        assert!(findings.iter().all(|finding| finding.severity == ERROR_SEVERITY));
    }

    #[test]
    fn baseline_reports_stale_source_identity_without_function_change() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_test_run(&observe, temp.path(), "config");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        std::fs::write(&source, "// source envelope changed\nfn f() -> u32 { 1 }\n").expect("write changed source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
        assert!(findings[0].message.contains("source envelope identity changed"));
    }

    #[test]
    fn baseline_receipt_missing_non_claims_reports_overclaim_boundary() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&baseline, serde_json::json!({"records": []}).to_string()).expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_OVERCLAIM);
    }

    #[test]
    fn matching_marker_satisfies_deny_mode_and_exports_review_fields() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let observed = collect_test_run(&observe, temp.path(), "config");
        let address = artifact_value(&observed)["records"][0]["function_address"]
            .as_str()
            .expect("function address")
            .to_owned();
        std::fs::write(
            &source,
            format!(
                "// octet-fingerprint: b3:{}; review=R-1; evidence=E-1\nfn f() -> u32 {{ 1 }}\n",
                marker_digest(&address)
            ),
        )
        .expect("write marked source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_test_run(&deny, temp.path(), "config");
        let artifact = artifact_value(&run);

        assert!(run.findings.is_empty());
        assert_eq!(artifact["records"][0]["review"], "R-1");
        assert_eq!(artifact["records"][0]["evidence"], "E-1");
    }

    #[test]
    fn stale_marker_reports_stale_function_address() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let wrong_address = determinism::full_hash("wrong-address");
        std::fs::write(
            &source,
            format!(
                "// octet-fingerprint: b3:{}; review=R; evidence=E\nfn f() -> u32 {{ 1 }}\n",
                marker_digest(&wrong_address)
            ),
        )
        .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
    }

    #[test]
    fn malformed_marker_reports_malformed_digest() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "// octet-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n")
            .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MALFORMED);
    }

    #[test]
    fn duplicate_source_locator_reports_duplicate_locator() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\nfn f() -> u32 { 2 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR));
    }

    #[test]
    fn duplicate_baseline_receipt_reports_duplicate_locator() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(serde_json::json!([
                {"path": "src/lib.rs", "name": "f", "function_address": determinism::full_hash("one")},
                {"path": "src/lib.rs", "name": "f", "function_address": determinism::full_hash("two")}
            ])),
        )
        .expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR);
    }

    #[test]
    fn baseline_receipt_overclaim_reports_overclaim() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        let mut receipt = serde_json::from_str::<serde_json::Value>(&valid_baseline_json(serde_json::json!([])))
            .expect("baseline JSON");
        receipt["claims"] = serde_json::json!(["proves release eligibility"]);
        std::fs::write(&baseline, receipt.to_string()).expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_OVERCLAIM);
    }
}
