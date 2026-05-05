use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tigerstyle_standards::model::Classification;
use tigerstyle_standards::model::EnforcementLevel;
use tigerstyle_standards::model::EnforcementMode;
use tigerstyle_standards::model::EvidenceCommand;
use tigerstyle_standards::model::FixtureCoverage;
use tigerstyle_standards::model::FixtureStatus;
use tigerstyle_standards::model::PolicyEntry;
use tigerstyle_standards::model::PolicyRegistry;
use tigerstyle_standards::model::Severity;
use tigerstyle_standards::model::SuppressionRequirements;
use tigerstyle_standards::report::CheckReport;

const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
const ARCHITECTURE_FAMILY: &str = "architecture";
const CHECKER_ID: &str = "tigerstyle-standards";
const REGISTRY_FILE_NAME: &str = "registry.json";
const ADVISORY_FIXTURE: &str = "standards/check-fixtures/architecture/advisory/public-api-budget";
const HARD_FAILURE_FIXTURE: &str = "standards/check-fixtures/architecture/noncompliant/missing-no-std";

#[test]
fn advisory_only_cli_report_exits_successfully() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);

    let output = run_architecture_check(&registry, &repo_path(ADVISORY_FIXTURE));
    assert_success(&output);
    let report = parse_report(&output);

    assert_eq!(report.diagnostics.len(), 1);
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.policy_id, PUBLIC_API_BUDGET_POLICY_ID);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(diagnostic.source_section.contains("when-boundaries-are-too-tight"));
    assert!(report.is_advisory_only());
    assert!(!report.has_blocking_diagnostics());
}

#[test]
fn hard_policy_cli_report_exits_with_failure() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);

    let output = run_architecture_check(&registry, &repo_path(HARD_FAILURE_FIXTURE));
    assert_failure(&output);
    let report = parse_report(&output);

    assert!(report.has_blocking_diagnostics());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.policy_id == FCIS_POLICY_ID && diagnostic.severity == Severity::Error)
    );
}

fn run_architecture_check(registry: &Path, source: &Path) -> Output {
    Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(registry)
        .arg("--source")
        .arg(source)
        .arg("--family")
        .arg(ARCHITECTURE_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run")
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "expected success, stderr was: {}", String::from_utf8_lossy(&output.stderr));
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure, stdout was: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn parse_report(output: &Output) -> CheckReport {
    serde_json::from_slice(&output.stdout).expect("stdout should be a standards report")
}

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_tigerstyle-standards")
}

fn repo_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative_path)
        .canonicalize()
        .expect("repo fixture path should exist")
}

fn write_registry(temp: &TempDir) -> PathBuf {
    let registry_path = temp.path().join(REGISTRY_FILE_NAME);
    let registry_json = serde_json::to_string(&test_registry()).expect("registry should serialize");
    std::fs::write(&registry_path, registry_json).expect("registry file should be written");
    registry_path
}

fn test_registry() -> PolicyRegistry {
    PolicyRegistry {
        schema_version: "test".to_owned(),
        checker_ids: vec![CHECKER_ID.to_owned()],
        policies: vec![
            hard_policy(FCIS_POLICY_ID, "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see"),
            hard_policy(
                ROOT_THINNESS_POLICY_ID,
                "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
            ),
            advisory_policy(
                PUBLIC_API_BUDGET_POLICY_ID,
                "docs/standards/source/architecture.adoc#when-boundaries-are-too-tight",
            ),
        ],
        seed_manifests: Vec::new(),
    }
}

fn hard_policy(policy_id: &str, source_anchor: &str) -> PolicyEntry {
    PolicyEntry {
        policy_id: policy_id.to_owned(),
        pillar: ARCHITECTURE_FAMILY.to_owned(),
        source_path: "docs/standards/source/architecture.adoc".to_owned(),
        source_heading: "Boundaries the Compiler Can See".to_owned(),
        source_anchor: source_anchor.to_owned(),
        summary: "test hard policy".to_owned(),
        classification: Classification::Hard,
        enforcement_level: EnforcementLevel::Hard,
        current_enforcement_mode: EnforcementMode::Gate,
        target_enforcement_mode: EnforcementMode::Gate,
        escalation_criteria: "already gated".to_owned(),
        mechanisms: Vec::new(),
        deterministic_checker: EvidenceCommand::default(),
        proof_rails: Vec::new(),
        checker_owner: ARCHITECTURE_FAMILY.to_owned(),
        checker_id: CHECKER_ID.to_owned(),
        default_severity: Severity::Error,
        suppression_requirements: hard_suppression_requirements(),
        fixture_coverage: FixtureCoverage {
            status: FixtureStatus::Planned,
            compliant: Vec::new(),
            noncompliant: Vec::new(),
        },
        verification_evidence_requirements: vec!["compliant fixture".to_owned(), "noncompliant fixture".to_owned()],
        classification_rationale: "deterministic hard policy".to_owned(),
        false_positive_risk: "low".to_owned(),
        existing_enforcement: Vec::new(),
        custom_checker_justification: "test".to_owned(),
    }
}

fn advisory_policy(policy_id: &str, source_anchor: &str) -> PolicyEntry {
    PolicyEntry {
        classification: Classification::Advisory,
        enforcement_level: EnforcementLevel::Advisory,
        default_severity: Severity::Warning,
        suppression_requirements: advisory_suppression_requirements(),
        fixture_coverage: FixtureCoverage {
            status: FixtureStatus::NotApplicable,
            compliant: Vec::new(),
            noncompliant: Vec::new(),
        },
        verification_evidence_requirements: vec![
            "deterministic sample input".to_owned(),
            "stable advisory report".to_owned(),
        ],
        classification_rationale: "surface growth is reviewable but contextual".to_owned(),
        false_positive_risk: "medium".to_owned(),
        ..hard_policy(policy_id, source_anchor)
    }
}

fn hard_suppression_requirements() -> SuppressionRequirements {
    SuppressionRequirements {
        requires_policy_id: true,
        requires_source_section: true,
        requires_owner: true,
        requires_rationale: true,
        requires_review_trigger: true,
        requires_safer_default: true,
    }
}

fn advisory_suppression_requirements() -> SuppressionRequirements {
    SuppressionRequirements {
        requires_policy_id: true,
        requires_source_section: true,
        requires_owner: false,
        requires_rationale: true,
        requires_review_trigger: false,
        requires_safer_default: false,
    }
}
