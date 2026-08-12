//! Pure project architecture gate evaluation over supplied Cargo and compiler facts.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use octet_architecture_ir::CapabilityFamily;
use octet_architecture_ir::CollectionStatus;
use octet_architecture_ir::CompilationUnit;
use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CompilerArchitectureReceipt;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CoverageStatus;
use octet_architecture_ir::SourceOrigin;
use octet_architecture_ir::SourceSpan;
use serde::Deserialize;
use serde::Serialize;

use crate::project_architecture_policy::ArchitectureCapabilityFamily;
use crate::project_architecture_policy::ArchitectureFindingIdentity;
use crate::project_architecture_policy::ArchitectureMode;
use crate::project_architecture_policy::ArchitectureRole;
use crate::project_architecture_policy::ArchitectureScopeKind;
use crate::project_architecture_policy::ProjectArchitecturePolicy;
use crate::project_architecture_policy::validate_architecture_waiver_binding;

/// Project architecture receipt schema emitted by this Octet release.
pub const PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION: &str = "octet-project-architecture-receipt/v1";
/// Repository path for the generated project architecture receipt contract.
pub const PROJECT_ARCHITECTURE_RECEIPT_CONTRACT_PATH: &str =
    "standards/project-architecture/contracts/project-architecture-receipt.generated.ncl";
/// Cargo graph schema consumed by the project architecture gate.
pub const PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION: &str = "octet-project-architecture-cargo-graph/v1";

const RECEIPT_ID_DOMAIN: &str = "octet.project-architecture-gate.receipt/v1";
const FINDING_ID_DOMAIN: &str = "octet.project-architecture-gate.finding/v1";
const CARGO_GRAPH_ID_DOMAIN: &str = "octet.project-architecture-gate.cargo-graph/v1";
const EVIDENCE_TIER: &str = "policy-check";
const CLAIM_ROLE_GRAPH: &str = "role-aware-dependency-graph";
const CLAIM_CORE_PURITY: &str = "domain-core-purity";
const CLAIM_PORT_OWNERSHIP: &str = "application-owned-port";
const CLAIM_ADAPTER_CONSTRUCTION: &str = "composition-root-construction";
const CLAIM_CONTRACT_BOUNDARY: &str = "inward-contract-boundary";
const CLAIM_CAPABILITY: &str = "capability-classification";
const CLAIM_NO_STD: &str = "no-std-obligation";
const CLAIM_COMPILER_COMPLETENESS: &str = "required-compiler-fact";
const CLAIM_INPUT_INTEGRITY: &str = "architecture-input-integrity";
const NON_CLAIMS: &[&str] = &[
    "complete-fcis-proof",
    "complete-hexagonal-proof",
    "complete-dependency-implementation-behavior",
    "runtime-determinism",
    "whole-program-correctness",
    "release-eligibility",
    "signer-authenticity",
];

/// One resolved Cargo dependency edge selected for architecture evaluation.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCargoEdge {
    /// Calling package name.
    pub caller_package: String,
    /// Resolved dependency package name.
    pub callee_package: String,
    /// Cargo edge kind, such as `normal` or `build`.
    pub edge_kind: String,
    /// Target predicate reported by Cargo, when present.
    pub target_predicate: Option<String>,
}

/// Exact resolved Cargo graph supplied by the runner shell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCargoGraph {
    /// Graph schema.
    pub schema_version: String,
    /// BLAKE3 graph identity.
    pub cargo_graph_id: String,
    /// Exact Cargo.lock byte identity.
    pub cargo_lock_blake3: String,
    /// Canonical locked Cargo metadata identity.
    pub cargo_metadata_blake3: String,
    /// Sorted selected production packages.
    pub selected_packages: Vec<String>,
    /// Exact selected target triple.
    pub target_triple: String,
    /// Sorted selected feature names.
    pub features: Vec<String>,
    /// Whether Cargo default features were selected.
    pub default_features: bool,
    /// Sorted resolved dependency edges.
    pub edges: Vec<ArchitectureCargoEdge>,
}

/// Stable architecture finding status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureFindingStatus {
    /// Current finding has no exact waiver.
    Open,
    /// One exact current waiver applies.
    Waived,
}

/// Stable architecture finding severity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureFindingSeverity {
    /// Input failure blocks every active mode.
    Integration,
    /// Policy violation is retained and mode promotion decides blocking.
    Policy,
}

/// One deterministic architecture policy finding.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitectureFinding {
    /// Stable BLAKE3 finding identity.
    pub finding_id: String,
    /// Stable diagnostic class.
    pub code: String,
    /// Finding severity class.
    pub severity: ArchitectureFindingSeverity,
    /// Waiver result.
    pub status: ArchitectureFindingStatus,
    /// Exact affected claim.
    pub affected_claim: String,
    /// Package involved in the finding, when known.
    pub package_name: Option<String>,
    /// Compilation unit involved in the finding, when known.
    pub unit_id: Option<String>,
    /// Compiler fact involved in the finding, when known.
    pub fact_id: Option<String>,
    /// Workspace source path, when available.
    pub source_path: Option<String>,
    /// Source envelope identity used for exact waiver binding.
    pub source_blake3: String,
    /// Reviewable finding message.
    pub message: String,
}

/// Architecture phase status. This status does not alter lint or closure authority.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitecturePhaseStatus {
    /// Complete evaluation found no open policy findings.
    Clean,
    /// Complete non-blocking evaluation retained open findings.
    Findings,
    /// Strict policy findings or required unknowns blocked the phase.
    Blocked,
    /// Required evaluation input was incomplete or mismatched.
    IntegrationFailure,
    /// Architecture gate passed but admission still requires separate closure evidence.
    ClosureRequired,
    /// Architecture gate and linked effect closure both passed.
    Admitted,
}

impl ArchitecturePhaseStatus {
    /// Stable machine-readable status name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Blocked => "blocked",
            Self::IntegrationFailure => "integration-failure",
            Self::ClosureRequired => "closure-required",
            Self::Admitted => "admitted",
        }
    }
}

/// Separate effect-closure parent status supplied to admission evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectClosureParent {
    /// Effect-closure receipt identity.
    pub receipt_id: String,
    /// True only when the separate closure checker admitted its own claim.
    pub admitted: bool,
    /// Evidence tier retained from the closure checker.
    pub evidence_tier: String,
    /// Closure non-claims retained without promotion.
    pub non_claims: Vec<String>,
}

/// Complete pure architecture-gate input.
pub struct ProjectArchitectureGateInput<'a> {
    /// Activated project policy.
    pub policy: &'a ProjectArchitecturePolicy,
    /// BLAKE3 identity of the Nickel policy source bytes.
    pub policy_source_blake3: &'a str,
    /// BLAKE3 identity of the exported policy bytes.
    pub policy_blake3: &'a str,
    /// Separate lint phase status supplied by the runner.
    pub lint_phase_status: &'a str,
    /// BLAKE3 identity of the normalized command.
    pub command_blake3: &'a str,
    /// Exact resolved Cargo graph.
    pub cargo_graph: &'a ArchitectureCargoGraph,
    /// Complete normalized compiler Architecture IR.
    pub architecture_ir: &'a CompilerArchitectureIr,
    /// Compiler coverage sidecar.
    pub coverage: &'a CompilerArchitectureCoverage,
    /// Compiler collection receipt.
    pub compiler_receipt: &'a CompilerArchitectureReceipt,
    /// Optional separate effect-closure result for admission mode.
    pub effect_closure: Option<&'a EffectClosureParent>,
}

/// Rust-owned architecture evaluation receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitectureReceipt {
    /// Receipt schema.
    pub schema_version: String,
    /// BLAKE3 receipt identity.
    pub receipt_id: String,
    /// Project identity.
    pub project_id: String,
    /// Selected architecture mode.
    pub mode: ArchitectureMode,
    /// Architecture phase status only.
    pub status: ArchitecturePhaseStatus,
    /// Evidence tier for the architecture evaluator.
    pub evidence_tier: String,
    /// Architecture checker identity.
    pub checker_id: String,
    /// Nickel policy source identity.
    pub policy_source_blake3: String,
    /// Exported policy identity.
    pub policy_blake3: String,
    /// Separate lint phase status.
    pub lint_phase_status: String,
    /// Normalized command identity.
    pub command_blake3: String,
    /// Resolved Cargo graph identity.
    pub cargo_graph_id: String,
    /// Compiler Architecture IR identity.
    pub architecture_ir_id: String,
    /// Compiler coverage identity.
    pub coverage_id: String,
    /// Compiler collection receipt identity.
    pub compiler_receipt_id: String,
    /// Exact compiler toolchain identities.
    pub toolchain_blake3s: Vec<String>,
    /// Exact compiler collector identities.
    pub collector_blake3s: Vec<String>,
    /// Exact compiler invocation identities.
    pub invocation_blake3s: Vec<String>,
    /// Exact target triples covered by compiler units.
    pub targets: Vec<String>,
    /// Exact selected feature sets covered by compiler units.
    pub feature_sets: Vec<Vec<String>>,
    /// Evaluated role assignments by package.
    pub package_roles: Vec<(String, ArchitectureRole)>,
    /// Evaluated application-owned port trait names.
    pub port_traits: Vec<String>,
    /// Evaluated capability families.
    pub capability_families: Vec<ArchitectureCapabilityFamily>,
    /// Deterministically ordered findings, including waived findings.
    pub findings: Vec<ProjectArchitectureFinding>,
    /// Compiler unknown identities retained by the gate.
    pub unknown_ids: Vec<String>,
    /// Applied waiver finding identities.
    pub applied_waiver_ids: Vec<String>,
    /// Separate effect-closure parent, when supplied.
    pub effect_closure: Option<EffectClosureParent>,
    /// Explicit architecture non-claims.
    pub non_claims: Vec<String>,
}

/// Render the Nickel contract derived from the Rust receipt DTO.
#[must_use]
pub fn render_project_architecture_receipt_contract() -> String {
    format!(
        r#"# @generated by crates/octet-standards/src/project_architecture_gate.rs; do not edit by hand.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Mode = OneOf ["inventory", "advisory", "gate", "admission"] in
let PhaseStatus = OneOf [
  "clean", "findings", "blocked", "integration-failure", "closure-required", "admitted",
] in
let FindingStatus = OneOf ["open", "waived"] in
let FindingSeverity = OneOf ["integration", "policy"] in
let Finding = {{
  finding_id | Blake3,
  code | NonEmptyString,
  severity | FindingSeverity,
  status | FindingStatus,
  affected_claim | NonEmptyString,
  package_name | Dyn,
  unit_id | Dyn,
  fact_id | Dyn,
  source_path | Dyn,
  source_blake3 | Blake3,
  message | NonEmptyString,
}} in
let EffectClosureParent = {{
  receipt_id | NonEmptyString,
  admitted | Bool,
  evidence_tier | NonEmptyString,
  non_claims | Array NonEmptyString,
}} in
{{
  ProjectArchitectureReceipt = {{
    schema_version | OneOf ["{PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION}"],
    receipt_id | Blake3,
    project_id | NonEmptyString,
    mode | Mode,
    status | PhaseStatus,
    evidence_tier | OneOf ["policy-check"],
    checker_id | NonEmptyString,
    policy_source_blake3 | Blake3,
    policy_blake3 | Blake3,
    lint_phase_status | OneOf ["clean", "warning-only", "lint-failure", "integration-failure"],
    command_blake3 | Blake3,
    cargo_graph_id | Blake3,
    architecture_ir_id | Blake3,
    coverage_id | Blake3,
    compiler_receipt_id | Blake3,
    toolchain_blake3s | Array Blake3,
    collector_blake3s | Array Blake3,
    invocation_blake3s | Array Blake3,
    targets | Array NonEmptyString,
    feature_sets | Array (Array NonEmptyString),
    package_roles | Array Dyn,
    port_traits | Array NonEmptyString,
    capability_families | Array NonEmptyString,
    findings | Array Finding,
    unknown_ids | Array Blake3,
    applied_waiver_ids | Array Blake3,
    effect_closure | Dyn,
    non_claims | Array NonEmptyString,
  }},
  EffectClosureParent = EffectClosureParent,
}}
"#
    )
}

/// Compute the canonical identity for one Cargo graph.
///
/// # Panics
///
/// Panics only if the fixed, serializable Cargo graph projection cannot be serialized.
#[must_use]
pub fn architecture_cargo_graph_identity(graph: &ArchitectureCargoGraph) -> String {
    octet_architecture_ir::canonical_identity(
        CARGO_GRAPH_ID_DOMAIN,
        &(
            &graph.schema_version,
            &graph.cargo_lock_blake3,
            &graph.cargo_metadata_blake3,
            &graph.selected_packages,
            &graph.target_triple,
            &graph.features,
            graph.default_features,
            &graph.edges,
        ),
    )
    .expect("architecture Cargo graph identity fields must serialize")
}

/// Compute and apply project architecture policy over supplied facts.
// r[impl octet.project_architecture_gate.role_graph]
// r[impl octet.project_architecture_gate.fcis]
// r[impl octet.project_architecture_gate.modes]
// r[impl octet.project_architecture_gate.waivers]
// r[impl octet.project_architecture_gate.admission]
// r[impl octet.project_architecture_gate.evidence]
#[must_use]
pub fn evaluate_project_architecture(input: &ProjectArchitectureGateInput<'_>) -> ProjectArchitectureReceipt {
    let mut findings = evaluate_findings(input);
    findings.sort();
    findings.dedup();
    apply_waivers(input, &mut findings);
    findings.sort();

    let integration_failure = findings.iter().any(|finding| {
        finding.severity == ArchitectureFindingSeverity::Integration
            && finding.status == ArchitectureFindingStatus::Open
    });
    let open_policy_findings = findings.iter().any(|finding| {
        finding.severity == ArchitectureFindingSeverity::Policy && finding.status == ArchitectureFindingStatus::Open
    });
    let status = classify_status(input.policy.mode, integration_failure, open_policy_findings, input.effect_closure);
    let targets = sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.target_triple.clone()).collect());
    let feature_sets =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| sorted_dedup(unit.features.clone())).collect());
    let package_roles = policy_package_roles(input.policy).into_iter().collect::<Vec<_>>();
    let port_traits = sorted_dedup(input.policy.ports.iter().map(|port| port.port_trait.clone()).collect());
    let capability_families = sorted_dedup(
        input.policy.capability_classifications.iter().map(|classification| classification.family).collect(),
    );
    let toolchain_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.toolchain_blake3.clone()).collect());
    let collector_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.collector_blake3.clone()).collect());
    let invocation_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.invocation_blake3.clone()).collect());
    let unknown_ids =
        sorted_dedup(input.architecture_ir.unknowns.iter().map(|unknown| unknown.unknown_id.clone()).collect());
    let applied_waiver_ids = findings
        .iter()
        .filter(|finding| finding.status == ArchitectureFindingStatus::Waived)
        .map(|finding| finding.finding_id.clone())
        .collect::<Vec<_>>();
    let mut non_claims = input.policy.non_claims.clone();
    non_claims.extend(NON_CLAIMS.iter().map(|claim| (*claim).to_owned()));
    let non_claims = sorted_dedup(non_claims);

    let mut receipt = ProjectArchitectureReceipt {
        schema_version: PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
        receipt_id: String::new(),
        project_id: input.policy.project_id.clone(),
        mode: input.policy.mode,
        status,
        evidence_tier: EVIDENCE_TIER.to_owned(),
        checker_id: "octet-standards.project-architecture-gate".to_owned(),
        policy_source_blake3: input.policy_source_blake3.to_owned(),
        policy_blake3: input.policy_blake3.to_owned(),
        lint_phase_status: input.lint_phase_status.to_owned(),
        command_blake3: input.command_blake3.to_owned(),
        cargo_graph_id: input.cargo_graph.cargo_graph_id.clone(),
        architecture_ir_id: input.architecture_ir.architecture_ir_id.clone(),
        coverage_id: input.coverage.coverage_id.clone(),
        compiler_receipt_id: input.compiler_receipt.receipt_id.clone(),
        toolchain_blake3s,
        collector_blake3s,
        invocation_blake3s,
        targets,
        feature_sets,
        package_roles,
        port_traits,
        capability_families,
        findings,
        unknown_ids,
        applied_waiver_ids,
        effect_closure: input.effect_closure.cloned(),
        non_claims,
    };
    receipt.receipt_id = receipt_identity(&receipt);
    receipt
}

/// Verify the stored receipt identity and required claim boundaries.
#[must_use]
pub fn verify_project_architecture_receipt(receipt: &ProjectArchitectureReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION {
        diagnostics.push("unsupported project architecture receipt schema".to_owned());
    }
    if receipt.receipt_id != receipt_identity(receipt) {
        diagnostics.push("project architecture receipt identity is stale or malformed".to_owned());
    }
    for required in NON_CLAIMS {
        if !receipt.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("project architecture receipt is missing non-claim `{required}`"));
        }
    }
    if receipt.evidence_tier != EVIDENCE_TIER {
        diagnostics.push("project architecture receipt evidence tier is not policy-check".to_owned());
    }
    if !matches!(
        receipt.lint_phase_status.as_str(),
        "clean" | "warning-only" | "lint-failure" | "integration-failure"
    ) {
        diagnostics.push("project architecture receipt has an unknown lint phase status".to_owned());
    }
    if let Some(closure) = &receipt.effect_closure
        && (closure.receipt_id.trim().is_empty()
            || closure.evidence_tier.trim().is_empty()
            || closure.non_claims.is_empty())
    {
        diagnostics.push("project architecture receipt has an incomplete effect-closure parent".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn evaluate_findings(input: &ProjectArchitectureGateInput<'_>) -> Vec<ProjectArchitectureFinding> {
    let mut findings = input_integrity_findings(input);
    if findings.is_empty() {
        let indexes = GateIndexes::new(input.architecture_ir);
        evaluate_role_graph(input, &indexes, &mut findings);
        evaluate_core_effects(input, &indexes, &mut findings);
        evaluate_ports(input, &indexes, &mut findings);
        evaluate_construction(input, &indexes, &mut findings);
        evaluate_contract_types(input, &indexes, &mut findings);
        evaluate_no_std(input, &indexes, &mut findings);
        evaluate_unknowns(input, &indexes, &mut findings);
    }
    findings
}

fn sorted_dedup<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}

fn input_integrity_findings(input: &ProjectArchitectureGateInput<'_>) -> Vec<ProjectArchitectureFinding> {
    let mut findings = Vec::new();
    let source_identity =
        input.architecture_ir.units.first().map_or(input.policy_blake3, |unit| unit.source_blake3.as_str());
    validate_cargo_inputs(input, source_identity, &mut findings);
    validate_compiler_inputs(input, source_identity, &mut findings);
    validate_unit_inputs(input, source_identity, &mut findings);
    findings
}

fn validate_cargo_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    require_input(
        findings,
        source_identity,
        input.cargo_graph.schema_version == PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION,
        "cargo-graph-schema",
        "resolved Cargo graph schema is unsupported",
    );
    require_input(
        findings,
        source_identity,
        input.cargo_graph.cargo_graph_id == architecture_cargo_graph_identity(input.cargo_graph),
        "cargo-graph-identity",
        "resolved Cargo graph identity is stale or malformed",
    );
    require_input(
        findings,
        source_identity,
        !input.cargo_graph.cargo_lock_blake3.trim().is_empty()
            && !input.cargo_graph.cargo_metadata_blake3.trim().is_empty(),
        "cargo-input-identities",
        "resolved Cargo graph lacks lockfile or metadata identity",
    );
    let expected_packages = input.policy.production.packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let selected_packages = input.cargo_graph.selected_packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    require_input(
        findings,
        source_identity,
        selected_packages == expected_packages,
        "cargo-package-selection",
        "resolved Cargo graph package selection does not match production policy",
    );
    let selected_targets = input
        .policy
        .production
        .targets
        .iter()
        .filter(|target| {
            target.triple == input.cargo_graph.target_triple
                && target.features == input.cargo_graph.features
                && target.default_features == input.cargo_graph.default_features
        })
        .count();
    require_input(
        findings,
        source_identity,
        selected_targets == 1,
        "cargo-target-selection",
        "resolved Cargo graph target or features do not match exactly one policy target",
    );
}

fn validate_compiler_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let checks = [
        (
            !input.policy_source_blake3.trim().is_empty(),
            "policy-source-identity",
            "Nickel policy source identity is empty",
        ),
        (
            !input.lint_phase_status.trim().is_empty(),
            "lint-phase-status",
            "separate lint phase status is empty",
        ),
        (
            input.architecture_ir.policy_blake3 == input.policy_blake3,
            "policy-ir-mismatch",
            "Architecture IR policy identity does not match the activated policy",
        ),
        (
            input.coverage.status == CoverageStatus::Complete,
            "compiler-coverage-incomplete",
            "compiler Architecture IR coverage is incomplete",
        ),
        (
            input.compiler_receipt.status == CollectionStatus::Complete,
            "compiler-receipt-incomplete",
            "compiler Architecture IR receipt is incomplete",
        ),
        (
            input.compiler_receipt.policy_blake3 == input.policy_blake3,
            "compiler-policy-mismatch",
            "compiler receipt policy identity does not match the activated policy",
        ),
        (
            input.compiler_receipt.coverage_id == input.coverage.coverage_id,
            "compiler-coverage-mismatch",
            "compiler receipt coverage identity does not match the coverage artifact",
        ),
        (
            input.compiler_receipt.architecture_ir_id.as_deref()
                == Some(input.architecture_ir.architecture_ir_id.as_str()),
            "compiler-ir-mismatch",
            "compiler receipt Architecture IR identity does not match the IR artifact",
        ),
        (!input.command_blake3.trim().is_empty(), "command-identity", "normalized command identity is empty"),
    ];
    for (condition, code, message) in checks {
        require_input(findings, source_identity, condition, code, message);
    }
}

fn validate_unit_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let expected_packages = input.policy.production.packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unit_packages =
        input.architecture_ir.units.iter().map(|unit| unit.package_name.as_str()).collect::<BTreeSet<_>>();
    require_input(
        findings,
        source_identity,
        unit_packages == expected_packages,
        "production-unit-coverage",
        "compiler units do not exactly cover declared production packages",
    );
    let policy_roles = policy_package_roles(input.policy);
    for unit in &input.architecture_ir.units {
        let role_matches = policy_roles.get(&unit.package_name).is_some_and(|role| compiler_role(*role) == unit.role);
        require_input(
            findings,
            source_identity,
            role_matches,
            "unit-role-mismatch",
            &format!("compiler unit `{}` role does not match explicit package policy", unit.package_name),
        );
    }
}

fn require_input(
    findings: &mut Vec<ProjectArchitectureFinding>,
    source_identity: &str,
    condition: bool,
    code: &str,
    message: &str,
) {
    if !condition {
        findings.push(new_finding(
            code,
            ArchitectureFindingSeverity::Integration,
            CLAIM_INPUT_INTEGRITY,
            None,
            None,
            None,
            None,
            source_identity,
            message,
        ));
    }
}

fn evaluate_role_graph(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let roles = policy_package_roles(input.policy);
    if !roles.values().any(|role| *role == ArchitectureRole::CompositionRoot) {
        findings.push(new_finding(
            "missing-composition-root",
            ArchitectureFindingSeverity::Policy,
            CLAIM_ROLE_GRAPH,
            None,
            None,
            None,
            None,
            input.policy_blake3,
            "production architecture has no explicit composition-root package",
        ));
    }
    for edge in &input.cargo_graph.edges {
        let Some(caller_role) = roles.get(&edge.caller_package) else {
            findings.push(new_finding(
                "unknown-cargo-caller",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                Some(&edge.caller_package),
                None,
                None,
                None,
                input.policy_blake3,
                &format!("Cargo graph caller `{}` has no production role", edge.caller_package),
            ));
            continue;
        };
        let allowed = if let Some(callee_role) = roles.get(&edge.callee_package) {
            input.policy.dependency_rules.iter().any(|rule| {
                rule.caller_role == *caller_role
                    && rule.edge_kinds.iter().any(|kind| kind == &edge.edge_kind)
                    && rule.allowed_callee_roles.iter().any(|role| role == callee_role)
            })
        } else {
            input.policy.provider_classifications.iter().any(|provider| {
                provider.package == edge.callee_package
                    && provider.permitted_caller_roles.iter().any(|role| role == caller_role)
                    && provider.edge_kinds.iter().any(|kind| kind == &edge.edge_kind)
            })
        };
        if !allowed {
            let unit = indexes.unit_for_package(&edge.caller_package);
            let callee = roles.get(&edge.callee_package).map_or_else(
                || format!("unclassified external provider `{}`", edge.callee_package),
                |callee_role| format!("package `{}` with role `{:?}`", edge.callee_package, callee_role),
            );
            findings.push(finding_for_unit(
                "forbidden-role-edge",
                CLAIM_ROLE_GRAPH,
                unit,
                None,
                None,
                &format!(
                    "package `{}` with role `{:?}` has forbidden {} dependency on {callee}",
                    edge.caller_package, caller_role, edge.edge_kind
                ),
            ));
        }
    }
}

fn evaluate_core_effects(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let classifications = input
        .policy
        .capability_classifications
        .iter()
        .map(|classification| (classification.family, classification))
        .collect::<BTreeMap<_, _>>();
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Effect {
            fact_id,
            capability_family,
            primitive_definition,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        let Some(unit) = indexes.unit_for_fact(fact_id) else {
            continue;
        };
        let source = indexes.source_path(span_id);
        let policy_family = policy_capability(*capability_family);
        if unit.role == CompilerArchitectureRole::DomainCore {
            findings.push(finding_for_unit(
                "domain-core-effect",
                CLAIM_CORE_PURITY,
                Some(unit),
                Some(fact_id),
                source,
                &format!("domain-core compiler fact performs `{capability_family:?}` through `{primitive_definition}`"),
            ));
        } else if let Some(policy_family) = policy_family {
            let executor_matches = classifications
                .get(&policy_family)
                .is_some_and(|classification| compiler_role(classification.executor_role) == unit.role);
            if !executor_matches {
                findings.push(finding_for_unit(
                    "capability-executor-role",
                    CLAIM_CAPABILITY,
                    Some(unit),
                    Some(fact_id),
                    source,
                    &format!(
                        "capability `{capability_family:?}` executes in package `{}` outside its declared executor role",
                        unit.package_name
                    ),
                ));
            }
        } else {
            findings.push(finding_for_unit(
                "unclassified-capability",
                CLAIM_CAPABILITY,
                Some(unit),
                Some(fact_id),
                source,
                &format!("compiler effect capability `{capability_family:?}` has no project classification"),
            ));
        }
    }
}

fn evaluate_ports(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    for port in &input.policy.ports {
        let traits = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Trait {
                    fact_id,
                    qualified_path,
                    span_id,
                    ..
                } if qualified_path == &port.port_trait => Some((fact_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if traits.len() != 1 {
            findings.push(new_finding(
                "port-trait-resolution",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PORT_OWNERSHIP,
                None,
                None,
                None,
                None,
                input.policy_blake3,
                &format!("port trait `{}` must resolve to exactly one compiler trait fact", port.port_trait),
            ));
            continue;
        }
        let (trait_fact_id, trait_span_id) = traits[0];
        let trait_unit = indexes.unit_for_fact(trait_fact_id);
        let trait_source = indexes.source_path(trait_span_id);
        let owner_valid = trait_unit.is_some_and(|unit| unit.role == CompilerArchitectureRole::ApplicationShell)
            && trait_source.is_some_and(|path| path_in_scope(path, &port.application_owner_scope));
        if !owner_valid {
            findings.push(finding_for_unit(
                "port-owner-role",
                CLAIM_PORT_OWNERSHIP,
                trait_unit,
                Some(trait_fact_id),
                trait_source,
                &format!(
                    "port trait `{}` is not owned by its declared application-shell scope `{}`",
                    port.port_trait, port.application_owner_scope
                ),
            ));
        }

        let implementations = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Implementation {
                    fact_id,
                    trait_definition: Some(definition),
                    span_id,
                    ..
                } if definition == &port.port_trait => Some((fact_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if implementations.is_empty() && !port.reserved {
            findings.push(new_finding(
                "port-implementation-missing",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PORT_OWNERSHIP,
                None,
                None,
                None,
                None,
                input.policy_blake3,
                &format!("non-reserved port trait `{}` has no compiler-resolved implementation", port.port_trait),
            ));
        }
        for (fact_id, span_id) in implementations {
            let unit = indexes.unit_for_fact(fact_id);
            let source = indexes.source_path(span_id);
            let role_valid = unit.is_some_and(|current| {
                matches!(
                    current.role,
                    CompilerArchitectureRole::InboundAdapter | CompilerArchitectureRole::OutboundAdapter
                )
            });
            let scope_valid =
                source.is_some_and(|path| port.permitted_adapter_scopes.iter().any(|scope| path_in_scope(path, scope)));
            if !role_valid || !scope_valid {
                findings.push(finding_for_unit(
                    "port-implementation-scope",
                    CLAIM_PORT_OWNERSHIP,
                    unit,
                    Some(fact_id),
                    source,
                    &format!(
                        "port `{}` implementation is outside its permitted adapter roles or scopes",
                        port.port_trait
                    ),
                ));
            }
        }
    }
}

fn evaluate_construction(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let type_origins = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id, origin_crate, ..
            } => Some((type_id.as_str(), origin_crate.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let roles = policy_package_roles(input.policy);
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Constructor {
            fact_id,
            constructed_type_id,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        let Some(origin) = type_origins.get(constructed_type_id.as_str()) else {
            continue;
        };
        let is_adapter = roles
            .get(*origin)
            .is_some_and(|role| matches!(role, ArchitectureRole::InboundAdapter | ArchitectureRole::OutboundAdapter));
        if !is_adapter {
            continue;
        }
        let unit = indexes.unit_for_fact(fact_id);
        if unit.is_none_or(|current| current.role != CompilerArchitectureRole::CompositionRoot) {
            findings.push(finding_for_unit(
                "adapter-construction-outside-root",
                CLAIM_ADAPTER_CONSTRUCTION,
                unit,
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("concrete adapter type from package `{origin}` is constructed outside a composition root"),
            ));
        }
    }
}

fn evaluate_contract_types(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let forbidden = input
        .policy
        .ports
        .iter()
        .flat_map(|port| port.forbidden_concrete_types.iter())
        .collect::<BTreeSet<_>>();
    if forbidden.is_empty() {
        return;
    }
    let types = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                display,
                definition_path,
                origin_crate,
                ..
            } => Some((type_id.as_str(), (display, definition_path.as_deref(), origin_crate))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let items = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                fact_id,
                item_id,
                signature_id,
                span_id,
                ..
            } => signature_id.as_ref().map(|signature| (item_id.as_str(), (fact_id, signature, span_id))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Signature {
            owner_item_id,
            input_type_ids,
            output_type_id,
            is_public,
            ..
        } = fact
        else {
            continue;
        };
        if !is_public {
            continue;
        }
        let Some((item_fact_id, _, span_id)) = items.get(owner_item_id.as_str()) else {
            continue;
        };
        let unit = indexes.unit_for_fact(item_fact_id);
        if !unit.is_some_and(|current| {
            matches!(current.role, CompilerArchitectureRole::DomainCore | CompilerArchitectureRole::ApplicationShell)
        }) {
            continue;
        }
        for type_id in input_type_ids.iter().chain(std::iter::once(output_type_id)) {
            let Some((display, definition, origin)) = types.get(type_id.as_str()) else {
                continue;
            };
            let leaked = forbidden.iter().any(|pattern| {
                display.contains(pattern.as_str())
                    || definition.is_some_and(|value| value.contains(pattern.as_str()))
                    || origin.contains(pattern.as_str())
            });
            if leaked {
                findings.push(finding_for_unit(
                    "infrastructure-type-leakage",
                    CLAIM_CONTRACT_BOUNDARY,
                    unit,
                    Some(item_fact_id),
                    indexes.source_path(span_id),
                    &format!("public inward contract exposes forbidden compiler type `{display}`"),
                ));
            }
        }
    }
}

fn evaluate_no_std(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let roles = policy_package_roles(input.policy);
    for obligation in &input.policy.core_no_std {
        let package_match = roles.get(&obligation.scope).copied() == Some(ArchitectureRole::DomainCore);
        let units = if package_match {
            input
                .architecture_ir
                .units
                .iter()
                .filter(|unit| unit.package_name == obligation.scope)
                .collect::<Vec<_>>()
        } else {
            indexes.units_for_source_scope(&obligation.scope)
        };
        let has_no_std = units.iter().all(|unit| indexes.package_has_configuration(&unit.package_name, "crate:no-std"));
        if units.is_empty() || !has_no_std {
            findings.push(new_finding(
                "no-std-obligation",
                ArchitectureFindingSeverity::Policy,
                CLAIM_NO_STD,
                None,
                None,
                None,
                Some(obligation.scope.clone()),
                input.policy_blake3,
                &format!("domain-core scope `{}` lacks compiler-confirmed crate:no-std", obligation.scope),
            ));
        }
    }
}

fn evaluate_unknowns(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    for unknown in &input.architecture_ir.unknowns {
        let unit = indexes.unit_for_unknown(&unknown.unknown_id);
        let source = unknown.span_id.as_deref().and_then(|span_id| indexes.source_path(span_id));
        findings.push(finding_for_unit(
            "required-compiler-unknown",
            CLAIM_COMPILER_COMPLETENESS,
            unit,
            Some(&unknown.unknown_id),
            source,
            &format!("required compiler fact remains unknown: {:?}: {}", unknown.kind, unknown.detail),
        ));
    }
}

fn apply_waivers(input: &ProjectArchitectureGateInput<'_>, findings: &mut Vec<ProjectArchitectureFinding>) {
    let finding_ids = findings.iter().map(|finding| finding.finding_id.clone()).collect::<BTreeSet<_>>();
    let mut stale = Vec::new();
    for waiver in &input.policy.waivers {
        let Some(finding) = findings.iter_mut().find(|finding| finding.finding_id == waiver.finding_id) else {
            stale.push(new_finding(
                "stale-waiver",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                None,
                None,
                None,
                None,
                &waiver.source_blake3,
                &format!("architecture waiver `{}` has no current finding", waiver.finding_id),
            ));
            continue;
        };
        let identity = ArchitectureFindingIdentity {
            finding_id: &finding.finding_id,
            policy_blake3: input.policy_blake3,
            source_blake3: &finding.source_blake3,
            affected_claim: &finding.affected_claim,
        };
        if validate_architecture_waiver_binding(waiver, &identity).is_empty() {
            finding.status = ArchitectureFindingStatus::Waived;
        } else {
            stale.push(new_finding(
                "stale-waiver",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                finding.package_name.as_deref(),
                finding.unit_id.as_deref(),
                finding.fact_id.as_deref(),
                finding.source_path.clone(),
                &finding.source_blake3,
                &format!("architecture waiver `{}` does not match current finding bindings", waiver.finding_id),
            ));
        }
    }
    debug_assert!(findings.iter().all(|finding| finding_ids.contains(&finding.finding_id)));
    findings.extend(stale);
}

fn classify_status(
    mode: ArchitectureMode,
    integration_failure: bool,
    open_policy_findings: bool,
    closure: Option<&EffectClosureParent>,
) -> ArchitecturePhaseStatus {
    if integration_failure {
        return ArchitecturePhaseStatus::IntegrationFailure;
    }
    match mode {
        ArchitectureMode::Inventory | ArchitectureMode::Advisory => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Findings
            } else {
                ArchitecturePhaseStatus::Clean
            }
        }
        ArchitectureMode::Gate => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Blocked
            } else {
                ArchitecturePhaseStatus::Clean
            }
        }
        ArchitectureMode::Admission => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Blocked
            } else {
                match closure {
                    Some(parent) if parent.admitted => ArchitecturePhaseStatus::Admitted,
                    Some(_) => ArchitecturePhaseStatus::Blocked,
                    None => ArchitecturePhaseStatus::ClosureRequired,
                }
            }
        }
    }
}

struct GateIndexes<'a> {
    ir: &'a CompilerArchitectureIr,
    units_by_package: BTreeMap<&'a str, &'a CompilationUnit>,
    fact_units: BTreeMap<&'a str, &'a CompilationUnit>,
    unknown_units: BTreeMap<&'a str, &'a CompilationUnit>,
    spans: BTreeMap<&'a str, &'a SourceSpan>,
}

impl<'a> GateIndexes<'a> {
    fn new(ir: &'a CompilerArchitectureIr) -> Self {
        let units_by_package =
            ir.units.iter().map(|unit| (unit.package_name.as_str(), unit)).collect::<BTreeMap<_, _>>();
        let item_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id, origin_crate, ..
                } => Some((item_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let type_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id, origin_crate, ..
                } => Some((type_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let mut fact_units = BTreeMap::new();
        for fact in &ir.facts {
            let package = fact_package(fact, &item_packages, &type_packages);
            if let Some(unit) = package.and_then(|package| units_by_package.get(package).copied()) {
                fact_units.insert(fact.fact_id(), unit);
            }
        }
        let unknown_units = ir
            .unknowns
            .iter()
            .filter_map(|unknown| {
                let package =
                    unknown.owner_item_id.as_deref().and_then(|item_id| item_packages.get(item_id).copied())?;
                Some((unknown.unknown_id.as_str(), *units_by_package.get(package)?))
            })
            .collect::<BTreeMap<_, _>>();
        let spans = ir.spans.iter().map(|span| (span.span_id.as_str(), span)).collect::<BTreeMap<_, _>>();
        Self {
            ir,
            units_by_package,
            fact_units,
            unknown_units,
            spans,
        }
    }

    fn unit_for_package(&self, package: &str) -> Option<&'a CompilationUnit> {
        self.units_by_package.get(package).copied()
    }

    fn unit_for_fact(&self, fact_id: &str) -> Option<&'a CompilationUnit> {
        self.fact_units.get(fact_id).copied()
    }

    fn unit_for_unknown(&self, unknown_id: &str) -> Option<&'a CompilationUnit> {
        self.unknown_units.get(unknown_id).copied()
    }

    fn source_path(&self, span_id: &str) -> Option<&'a str> {
        self.spans.get(span_id).and_then(|span| match &span.origin {
            SourceOrigin::Workspace { source_path } => Some(source_path.as_str()),
            SourceOrigin::ExternalGenerated { .. } => None,
        })
    }

    fn units_for_source_scope(&self, scope: &str) -> Vec<&'a CompilationUnit> {
        let packages = self
            .ir
            .facts
            .iter()
            .filter_map(|fact| {
                let span_id = fact_span_id(fact)?;
                let source_path = self.source_path(span_id)?;
                let unit = path_in_scope(source_path, scope).then(|| self.unit_for_fact(fact.fact_id())).flatten()?;
                Some((unit.package_name.as_str(), unit))
            })
            .collect::<BTreeMap<_, _>>();
        packages.into_values().collect()
    }

    fn package_has_configuration(&self, package: &str, predicate: &str) -> bool {
        let expected = format!("{predicate}={package}");
        self.ir.facts.iter().any(|fact| {
            matches!(fact, CompilerArchitectureFact::Configuration { predicate: current, active: true, .. } if current == &expected)
        })
    }
}

fn fact_package<'a>(
    fact: &'a CompilerArchitectureFact,
    item_packages: &BTreeMap<&str, &'a str>,
    type_packages: &BTreeMap<&str, &'a str>,
) -> Option<&'a str> {
    match fact {
        CompilerArchitectureFact::Item { origin_crate, .. } | CompilerArchitectureFact::Trait { origin_crate, .. } => {
            Some(origin_crate)
        }
        CompilerArchitectureFact::Type { origin_crate, .. } => Some(origin_crate),
        CompilerArchitectureFact::Implementation { self_type_id, .. } => {
            type_packages.get(self_type_id.as_str()).copied()
        }
        CompilerArchitectureFact::Signature { owner_item_id, .. } => item_packages.get(owner_item_id.as_str()).copied(),
        CompilerArchitectureFact::Call { caller_item_id, .. }
        | CompilerArchitectureFact::Constructor { caller_item_id, .. }
        | CompilerArchitectureFact::Effect { caller_item_id, .. } => {
            item_packages.get(caller_item_id.as_str()).copied()
        }
        CompilerArchitectureFact::Module { .. }
        | CompilerArchitectureFact::Expansion { .. }
        | CompilerArchitectureFact::Configuration { .. } => None,
    }
}

fn fact_span_id(fact: &CompilerArchitectureFact) -> Option<&str> {
    match fact {
        CompilerArchitectureFact::Module { span_id, .. }
        | CompilerArchitectureFact::Item { span_id, .. }
        | CompilerArchitectureFact::Trait { span_id, .. }
        | CompilerArchitectureFact::Implementation { span_id, .. }
        | CompilerArchitectureFact::Call { span_id, .. }
        | CompilerArchitectureFact::Constructor { span_id, .. }
        | CompilerArchitectureFact::Effect { span_id, .. }
        | CompilerArchitectureFact::Configuration {
            span_id: Some(span_id), ..
        } => Some(span_id),
        CompilerArchitectureFact::Type { .. }
        | CompilerArchitectureFact::Signature { .. }
        | CompilerArchitectureFact::Expansion { .. }
        | CompilerArchitectureFact::Configuration { span_id: None, .. } => None,
    }
}

fn policy_package_roles(policy: &ProjectArchitecturePolicy) -> BTreeMap<String, ArchitectureRole> {
    policy
        .roles
        .iter()
        .filter(|assignment| assignment.scope_kind == ArchitectureScopeKind::Package && assignment.role.is_production())
        .map(|assignment| (assignment.scope.clone(), assignment.role))
        .collect()
}

fn compiler_role(role: ArchitectureRole) -> CompilerArchitectureRole {
    match role {
        ArchitectureRole::DomainCore => CompilerArchitectureRole::DomainCore,
        ArchitectureRole::ApplicationShell => CompilerArchitectureRole::ApplicationShell,
        ArchitectureRole::InboundAdapter => CompilerArchitectureRole::InboundAdapter,
        ArchitectureRole::OutboundAdapter => CompilerArchitectureRole::OutboundAdapter,
        ArchitectureRole::CompositionRoot => CompilerArchitectureRole::CompositionRoot,
        ArchitectureRole::Test => CompilerArchitectureRole::Test,
        ArchitectureRole::Generated => CompilerArchitectureRole::Generated,
    }
}

fn policy_capability(family: CapabilityFamily) -> Option<ArchitectureCapabilityFamily> {
    match family {
        CapabilityFamily::Time => Some(ArchitectureCapabilityFamily::Time),
        CapabilityFamily::Randomness => Some(ArchitectureCapabilityFamily::Randomness),
        CapabilityFamily::Filesystem => Some(ArchitectureCapabilityFamily::Filesystem),
        CapabilityFamily::Network => Some(ArchitectureCapabilityFamily::Network),
        CapabilityFamily::Process => Some(ArchitectureCapabilityFamily::Process),
        CapabilityFamily::Environment => Some(ArchitectureCapabilityFamily::Environment),
        CapabilityFamily::Secrets => Some(ArchitectureCapabilityFamily::Secrets),
        CapabilityFamily::UnsafeCode => Some(ArchitectureCapabilityFamily::UnsafeCode),
        CapabilityFamily::GlobalState => Some(ArchitectureCapabilityFamily::GlobalState),
        CapabilityFamily::Concurrency => Some(ArchitectureCapabilityFamily::Concurrency),
        CapabilityFamily::Observability => Some(ArchitectureCapabilityFamily::Observability),
        CapabilityFamily::Ffi => Some(ArchitectureCapabilityFamily::Ffi),
        CapabilityFamily::Allocation => Some(ArchitectureCapabilityFamily::Allocation),
        CapabilityFamily::Unknown => None,
    }
}

fn finding_for_unit(
    code: &str,
    affected_claim: &str,
    unit: Option<&CompilationUnit>,
    fact_id: Option<&str>,
    source_path: Option<&str>,
    message: &str,
) -> ProjectArchitectureFinding {
    let source_blake3 = unit
        .map(|current| current.source_blake3.as_str())
        .or(fact_id)
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    new_finding(
        code,
        ArchitectureFindingSeverity::Policy,
        affected_claim,
        unit.map(|current| current.package_name.as_str()),
        unit.map(|current| current.unit_id.as_str()),
        fact_id,
        source_path.map(str::to_owned),
        source_blake3,
        message,
    )
}

#[allow(clippy::too_many_arguments)]
fn new_finding(
    code: &str,
    severity: ArchitectureFindingSeverity,
    affected_claim: &str,
    package_name: Option<&str>,
    unit_id: Option<&str>,
    fact_id: Option<&str>,
    source_path: Option<String>,
    source_blake3: &str,
    message: &str,
) -> ProjectArchitectureFinding {
    let finding_id = octet_architecture_ir::canonical_identity(
        FINDING_ID_DOMAIN,
        &(code, affected_claim, package_name, unit_id, fact_id, &source_path, source_blake3, message),
    )
    .expect("project architecture finding identity fields must serialize");
    ProjectArchitectureFinding {
        finding_id,
        code: code.to_owned(),
        severity,
        status: ArchitectureFindingStatus::Open,
        affected_claim: affected_claim.to_owned(),
        package_name: package_name.map(str::to_owned),
        unit_id: unit_id.map(str::to_owned),
        fact_id: fact_id.map(str::to_owned),
        source_path,
        source_blake3: source_blake3.to_owned(),
        message: message.to_owned(),
    }
}

fn receipt_identity(receipt: &ProjectArchitectureReceipt) -> String {
    let mut projection = receipt.clone();
    projection.receipt_id.clear();
    octet_architecture_ir::canonical_identity(RECEIPT_ID_DOMAIN, &projection)
        .expect("project architecture receipt identity fields must serialize")
}

fn path_in_scope(path: &str, scope: &str) -> bool {
    path == scope || path.strip_prefix(scope).is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use octet_architecture_ir::ArchitectureCollectionLimits;
    use octet_architecture_ir::ItemKind;
    use octet_architecture_ir::UnknownCompilerFact;
    use octet_architecture_ir::UnknownFactKind;

    use super::*;
    use crate::project_architecture_policy::ArchitectureCapabilityClassification;
    use crate::project_architecture_policy::ArchitectureDependencyRule;
    use crate::project_architecture_policy::ArchitecturePort;
    use crate::project_architecture_policy::ArchitectureProductionScope;
    use crate::project_architecture_policy::ArchitectureRoleAssignment;
    use crate::project_architecture_policy::ArchitectureTarget;
    use crate::project_architecture_policy::ArchitectureWaiver;
    use crate::project_architecture_policy::CoreCapabilityHandling;
    use crate::project_architecture_policy::PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const TARGET: &str = "x86_64-unknown-linux-gnu";

    #[test]
    fn generated_receipt_contract_is_fresh() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root.join(PROJECT_ARCHITECTURE_RECEIPT_CONTRACT_PATH);
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read generated receipt contract {}: {error}", path.display()));

        assert_eq!(expected, render_project_architecture_receipt_contract());
    }

    #[test]
    fn clean_role_graph_passes_and_receipt_repeats() {
        let fixture = Fixture::new(ArchitectureMode::Gate);
        let first = fixture.evaluate(None);
        let second = fixture.evaluate(None);

        assert_eq!(first.status, ArchitecturePhaseStatus::Clean);
        assert!(first.findings.is_empty());
        assert_eq!(first, second);
        assert!(verify_project_architecture_receipt(&first).is_empty());
    }

    #[test]
    fn forbidden_role_edge_blocks_gate_but_not_inventory() {
        let mut gate = Fixture::new(ArchitectureMode::Gate);
        gate.graph.edges.push(edge("domain", "outbound"));
        gate.refresh_graph_id();
        let gate_receipt = gate.evaluate(None);
        assert_eq!(gate_receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(gate_receipt.findings.iter().any(|finding| finding.code == "forbidden-role-edge"));

        gate.policy.mode = ArchitectureMode::Inventory;
        let inventory = gate.evaluate(None);
        assert_eq!(inventory.status, ArchitecturePhaseStatus::Findings);
        assert_eq!(inventory.findings, gate_receipt.findings);
    }

    #[test]
    fn unclassified_external_provider_blocks_strict_mode() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.graph.edges.push(edge("domain", "ambient-vendor"));
        fixture.refresh_graph_id();

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| {
            finding.code == "forbidden-role-edge" && finding.message.contains("unclassified external provider")
        }));
    }

    #[test]
    fn core_effect_and_required_unknown_block_gate() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_domain_effect();
        fixture.add_domain_unknown();

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| finding.code == "domain-core-effect"));
        assert!(receipt.findings.iter().any(|finding| finding.code == "required-compiler-unknown"));
    }

    #[test]
    fn complete_port_placement_passes_and_shell_implementation_blocks() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_port_facts("outbound");
        let clean = fixture.evaluate(None);
        assert_eq!(clean.status, ArchitecturePhaseStatus::Clean);

        let mut invalid = Fixture::new(ArchitectureMode::Gate);
        invalid.add_port_facts("application");
        let blocked = invalid.evaluate(None);
        assert_eq!(blocked.status, ArchitecturePhaseStatus::Blocked);
        assert!(blocked.findings.iter().any(|finding| finding.code == "port-implementation-scope"));
    }

    #[test]
    fn incomplete_compiler_coverage_is_an_integration_failure_in_advisory_mode() {
        let mut fixture = Fixture::new(ArchitectureMode::Advisory);
        fixture.coverage.status = CoverageStatus::Incomplete;

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::IntegrationFailure);
        assert!(receipt.findings.iter().any(|finding| finding.code == "compiler-coverage-incomplete"));
    }

    #[test]
    fn exact_waiver_is_retained_while_stale_waiver_fails_closed() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.graph.edges.push(edge("domain", "outbound"));
        fixture.refresh_graph_id();
        let finding = fixture.evaluate(None).findings.remove(0);
        fixture.policy.waivers.push(ArchitectureWaiver {
            finding_id: finding.finding_id.clone(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: finding.source_blake3.clone(),
            owner: "architecture-team".to_owned(),
            reason: "bounded migration".to_owned(),
            review_condition: "remove after provider extraction".to_owned(),
            affected_claim: finding.affected_claim.clone(),
        });
        let waived = fixture.evaluate(None);
        assert_eq!(waived.status, ArchitecturePhaseStatus::Clean);
        assert_eq!(waived.findings[0].status, ArchitectureFindingStatus::Waived);

        fixture.policy.waivers[0].source_blake3 = HASH_C.to_owned();
        let stale = fixture.evaluate(None);
        assert_eq!(stale.status, ArchitecturePhaseStatus::IntegrationFailure);
        assert!(stale.findings.iter().any(|current| current.code == "stale-waiver"));
    }

    #[test]
    fn admission_keeps_effect_closure_separate() {
        let fixture = Fixture::new(ArchitectureMode::Admission);
        assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::ClosureRequired);
        let closure = EffectClosureParent {
            receipt_id: HASH_C.to_owned(),
            admitted: true,
            evidence_tier: "selected-source-token-closure".to_owned(),
            non_claims: vec!["complete-mir-closure".to_owned()],
        };
        let admitted = fixture.evaluate(Some(&closure));
        assert_eq!(admitted.status, ArchitecturePhaseStatus::Admitted);
        assert_eq!(admitted.effect_closure, Some(closure));
        assert!(!admitted.non_claims.iter().any(|claim| claim == "complete-mir-closure"));

        let denied = EffectClosureParent {
            receipt_id: HASH_B.to_owned(),
            admitted: false,
            evidence_tier: "policy-check".to_owned(),
            non_claims: vec!["complete-mir-closure".to_owned()],
        };
        let blocked = fixture.evaluate(Some(&denied));
        assert_eq!(blocked.status, ArchitecturePhaseStatus::Blocked);
        assert_eq!(blocked.effect_closure, Some(denied));
    }

    #[test]
    fn receipt_verifier_rejects_tampering_and_missing_non_claims() {
        let fixture = Fixture::new(ArchitectureMode::Gate);
        let mut receipt = fixture.evaluate(None);
        receipt.project_id = "tampered".to_owned();
        receipt.non_claims.retain(|claim| claim != "whole-program-correctness");

        let diagnostics = verify_project_architecture_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program-correctness")));
    }

    struct Fixture {
        policy: ProjectArchitecturePolicy,
        graph: ArchitectureCargoGraph,
        ir: CompilerArchitectureIr,
        coverage: CompilerArchitectureCoverage,
        compiler_receipt: CompilerArchitectureReceipt,
    }

    impl Fixture {
        #[allow(clippy::too_many_lines)]
        fn new(mode: ArchitectureMode) -> Self {
            let package_roles = [
                ("domain", ArchitectureRole::DomainCore),
                ("application", ArchitectureRole::ApplicationShell),
                ("inbound", ArchitectureRole::InboundAdapter),
                ("outbound", ArchitectureRole::OutboundAdapter),
                ("host", ArchitectureRole::CompositionRoot),
            ];
            let source_roles = [
                ("crates/domain/src", ArchitectureRole::DomainCore),
                ("crates/application/src", ArchitectureRole::ApplicationShell),
                ("crates/inbound/src", ArchitectureRole::InboundAdapter),
                ("crates/outbound/src", ArchitectureRole::OutboundAdapter),
                ("crates/host/src", ArchitectureRole::CompositionRoot),
            ];
            let mut roles = package_roles
                .iter()
                .map(|(scope, role)| role_assignment(ArchitectureScopeKind::Package, scope, *role))
                .collect::<Vec<_>>();
            roles.extend(
                source_roles
                    .iter()
                    .map(|(scope, role)| role_assignment(ArchitectureScopeKind::Source, scope, *role)),
            );
            let units =
                package_roles.iter().map(|(package, role)| unit(package, compiler_role(*role))).collect::<Vec<_>>();
            let policy = ProjectArchitecturePolicy {
                schema_version: PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION.to_owned(),
                project_id: "fixture".to_owned(),
                mode,
                production: ArchitectureProductionScope {
                    packages: package_roles.iter().map(|(package, _)| (*package).to_owned()).collect(),
                    source_scopes: source_roles.iter().map(|(scope, _)| (*scope).to_owned()).collect(),
                    targets: vec![ArchitectureTarget {
                        triple: TARGET.to_owned(),
                        features: Vec::new(),
                        default_features: true,
                    }],
                },
                roles,
                dependency_rules: package_roles
                    .iter()
                    .map(|(_, role)| ArchitectureDependencyRule {
                        caller_role: *role,
                        allowed_callee_roles: allowed_roles(*role),
                        edge_kinds: vec!["normal".to_owned(), "build".to_owned()],
                    })
                    .collect(),
                provider_classifications: Vec::new(),
                ports: Vec::new(),
                capability_classifications: vec![ArchitectureCapabilityClassification {
                    family: ArchitectureCapabilityFamily::Filesystem,
                    core_handling: CoreCapabilityHandling::EffectPlan,
                    executor_role: ArchitectureRole::OutboundAdapter,
                    rationale: "outbound adapter executes filesystem plans".to_owned(),
                }],
                core_no_std: Vec::new(),
                admission: (mode == ArchitectureMode::Admission).then(|| {
                    crate::project_architecture_policy::ArchitectureAdmissionBinding {
                        strict_profile: "deterministic-core".to_owned(),
                        effect_closure_policy: "octet-effect-closure.json".to_owned(),
                    }
                }),
                waivers: Vec::new(),
                collection_limits: limits(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            let mut graph = ArchitectureCargoGraph {
                schema_version: PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION.to_owned(),
                cargo_graph_id: String::new(),
                cargo_lock_blake3: HASH_A.to_owned(),
                cargo_metadata_blake3: HASH_B.to_owned(),
                selected_packages: vec![
                    "application".to_owned(),
                    "domain".to_owned(),
                    "host".to_owned(),
                    "inbound".to_owned(),
                    "outbound".to_owned(),
                ],
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                features: Vec::new(),
                default_features: true,
                edges: vec![
                    edge("application", "domain"),
                    edge("inbound", "application"),
                    edge("outbound", "application"),
                    edge("host", "application"),
                    edge("host", "inbound"),
                    edge("host", "outbound"),
                ],
            };
            graph.edges.sort();
            graph.cargo_graph_id = architecture_cargo_graph_identity(&graph);
            let ir = CompilerArchitectureIr {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION.to_owned(),
                architecture_ir_id: HASH_B.to_owned(),
                policy_blake3: HASH_A.to_owned(),
                units,
                spans: Vec::new(),
                facts: Vec::new(),
                unknowns: Vec::new(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            let coverage = CompilerArchitectureCoverage {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION.to_owned(),
                coverage_id: HASH_C.to_owned(),
                status: CoverageStatus::Complete,
                expected_unit_ids: ir.units.iter().map(|unit| unit.unit_id.clone()).collect(),
                observed_unit_ids: ir.units.iter().map(|unit| unit.unit_id.clone()).collect(),
                missing_unit_ids: Vec::new(),
                unexpected_unit_ids: Vec::new(),
                duplicate_unit_ids: Vec::new(),
                invalid_shard_ids: Vec::new(),
            };
            let compiler_receipt = CompilerArchitectureReceipt {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
                receipt_id: HASH_B.to_owned(),
                status: CollectionStatus::Complete,
                policy_blake3: HASH_A.to_owned(),
                coverage_id: HASH_C.to_owned(),
                architecture_ir_id: Some(HASH_B.to_owned()),
                issue_ids: Vec::new(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            Self {
                policy,
                graph,
                ir,
                coverage,
                compiler_receipt,
            }
        }

        fn evaluate(&self, effect_closure: Option<&EffectClosureParent>) -> ProjectArchitectureReceipt {
            evaluate_project_architecture(&ProjectArchitectureGateInput {
                policy: &self.policy,
                policy_source_blake3: HASH_C,
                policy_blake3: HASH_A,
                lint_phase_status: "clean",
                command_blake3: HASH_B,
                cargo_graph: &self.graph,
                architecture_ir: &self.ir,
                coverage: &self.coverage,
                compiler_receipt: &self.compiler_receipt,
                effect_closure,
            })
        }

        fn refresh_graph_id(&mut self) {
            self.graph.edges.sort();
            self.graph.cargo_graph_id = architecture_cargo_graph_identity(&self.graph);
        }

        fn add_domain_effect(&mut self) {
            let span = span("crates/domain/src/lib.rs");
            let item = item(&span.span_id, "domain::decide");
            let mut effect = CompilerArchitectureFact::Effect {
                fact_id: String::new(),
                caller_item_id: item_id(&item).to_owned(),
                capability_family: CapabilityFamily::Filesystem,
                primitive_definition: "std::fs::read".to_owned(),
                span_id: span.span_id.clone(),
            };
            set_fact_id(&mut effect);
            self.add_unit_facts("domain", vec![span], vec![item, effect], Vec::new());
        }

        fn add_domain_unknown(&mut self) {
            let span = span("crates/domain/src/dispatch.rs");
            let item = item(&span.span_id, "domain::dispatch");
            let mut unknown = UnknownCompilerFact {
                unknown_id: String::new(),
                kind: UnknownFactKind::DynamicDispatch,
                owner_item_id: Some(item_id(&item).to_owned()),
                target_identity: Some("dyn domain::Port".to_owned()),
                span_id: Some(span.span_id.clone()),
                detail: "dynamic target is unresolved".to_owned(),
            };
            unknown.unknown_id = octet_architecture_ir::unknown_fact_identity(&unknown);
            self.add_unit_facts("domain", vec![span], vec![item], vec![unknown]);
        }

        fn add_port_facts(&mut self, implementation_package: &str) {
            self.policy.ports.push(ArchitecturePort {
                port_trait: "application::ports::Repository".to_owned(),
                application_owner_scope: "crates/application/src".to_owned(),
                permitted_adapter_scopes: vec!["crates/outbound/src".to_owned()],
                forbidden_concrete_types: Vec::new(),
                reserved: false,
            });
            let trait_span = span("crates/application/src/ports.rs");
            let mut trait_fact = CompilerArchitectureFact::Trait {
                fact_id: String::new(),
                trait_id: HASH_C.to_owned(),
                qualified_path: "application::ports::Repository".to_owned(),
                origin_crate: "application".to_owned(),
                span_id: trait_span.span_id.clone(),
            };
            set_fact_id(&mut trait_fact);
            self.add_unit_facts("application", vec![trait_span], vec![trait_fact], Vec::new());

            let implementation_scope = if implementation_package == "outbound" {
                "crates/outbound/src/repository.rs"
            } else {
                "crates/application/src/repository.rs"
            };
            let implementation_span = span(implementation_scope);
            let self_type_id = octet_architecture_ir::canonical_identity(
                "fixture-type",
                &(implementation_package, "RepositoryAdapter"),
            )
            .expect("fixture type identity");
            let mut self_type = CompilerArchitectureFact::Type {
                fact_id: String::new(),
                type_id: self_type_id.clone(),
                display: format!("{implementation_package}::RepositoryAdapter"),
                definition_path: Some(format!("{implementation_package}::RepositoryAdapter")),
                origin_crate: implementation_package.to_owned(),
                type_kind: octet_architecture_ir::TypeKind::Adt,
            };
            set_fact_id(&mut self_type);
            let mut implementation = CompilerArchitectureFact::Implementation {
                fact_id: String::new(),
                implementation_id: HASH_C.to_owned(),
                trait_id: None,
                trait_definition: Some("application::ports::Repository".to_owned()),
                self_type_id,
                span_id: implementation_span.span_id.clone(),
            };
            set_fact_id(&mut implementation);
            self.add_unit_facts(
                implementation_package,
                vec![implementation_span],
                vec![self_type, implementation],
                Vec::new(),
            );
        }

        fn add_unit_facts(
            &mut self,
            package: &str,
            spans: Vec<SourceSpan>,
            facts: Vec<CompilerArchitectureFact>,
            unknowns: Vec<UnknownCompilerFact>,
        ) {
            assert!(self.ir.units.iter().any(|unit| unit.package_name == package), "fixture unit");
            self.ir.spans.extend(spans);
            self.ir.facts.extend(facts);
            self.ir.unknowns.extend(unknowns);
            self.ir.spans.sort();
            self.ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
            self.ir.unknowns.sort();
        }
    }

    fn role_assignment(
        scope_kind: ArchitectureScopeKind,
        scope: &str,
        role: ArchitectureRole,
    ) -> ArchitectureRoleAssignment {
        ArchitectureRoleAssignment {
            scope_kind,
            scope: scope.to_owned(),
            role,
        }
    }

    fn allowed_roles(role: ArchitectureRole) -> Vec<ArchitectureRole> {
        match role {
            ArchitectureRole::DomainCore => vec![ArchitectureRole::DomainCore],
            ArchitectureRole::ApplicationShell => {
                vec![ArchitectureRole::ApplicationShell, ArchitectureRole::DomainCore]
            }
            ArchitectureRole::InboundAdapter => {
                vec![ArchitectureRole::InboundAdapter, ArchitectureRole::ApplicationShell]
            }
            ArchitectureRole::OutboundAdapter => {
                vec![ArchitectureRole::OutboundAdapter, ArchitectureRole::ApplicationShell]
            }
            ArchitectureRole::CompositionRoot => vec![
                ArchitectureRole::CompositionRoot,
                ArchitectureRole::ApplicationShell,
                ArchitectureRole::InboundAdapter,
                ArchitectureRole::OutboundAdapter,
                ArchitectureRole::DomainCore,
            ],
            ArchitectureRole::Test | ArchitectureRole::Generated => Vec::new(),
        }
    }

    fn edge(source_package: &str, target_package: &str) -> ArchitectureCargoEdge {
        ArchitectureCargoEdge {
            caller_package: source_package.to_owned(),
            callee_package: target_package.to_owned(),
            edge_kind: "normal".to_owned(),
            target_predicate: None,
        }
    }

    fn unit(package: &str, role: CompilerArchitectureRole) -> CompilationUnit {
        CompilationUnit {
            unit_id: octet_architecture_ir::canonical_identity("fixture-unit", &(package, role)).expect("unit id"),
            package_name: package.to_owned(),
            package_version: "0.1.0".to_owned(),
            target_name: package.to_owned(),
            target_kind: "lib".to_owned(),
            crate_types: vec!["rlib".to_owned()],
            target_triple: TARGET.to_owned(),
            features: Vec::new(),
            default_features: true,
            role,
            toolchain_blake3: HASH_B.to_owned(),
            collector_blake3: HASH_C.to_owned(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: octet_architecture_ir::canonical_identity("fixture-source", &package).expect("source id"),
            invocation_blake3: HASH_B.to_owned(),
        }
    }

    fn span(path: &str) -> SourceSpan {
        let mut span = SourceSpan {
            span_id: String::new(),
            origin: SourceOrigin::Workspace {
                source_path: path.to_owned(),
            },
            byte_start: 0,
            byte_end: 1,
            line_start: 1,
            column_start: 1,
            from_expansion: false,
        };
        span.span_id = octet_architecture_ir::source_span_identity(&span);
        span
    }

    fn item(span_id: &str, qualified_path: &str) -> CompilerArchitectureFact {
        let mut fact = CompilerArchitectureFact::Item {
            fact_id: String::new(),
            item_id: octet_architecture_ir::canonical_identity("fixture-item", &qualified_path).expect("item id"),
            qualified_path: qualified_path.to_owned(),
            item_kind: ItemKind::Function,
            origin_crate: qualified_path.split("::").next().unwrap_or(qualified_path).to_owned(),
            visibility: "public".to_owned(),
            signature_id: None,
            span_id: span_id.to_owned(),
        };
        set_fact_id(&mut fact);
        fact
    }

    fn item_id(fact: &CompilerArchitectureFact) -> &str {
        let CompilerArchitectureFact::Item { item_id, .. } = fact else {
            panic!("fixture item fact expected");
        };
        item_id
    }

    fn set_fact_id(fact: &mut CompilerArchitectureFact) {
        let identity = octet_architecture_ir::fact_identity(fact);
        match fact {
            CompilerArchitectureFact::Module { fact_id, .. }
            | CompilerArchitectureFact::Item { fact_id, .. }
            | CompilerArchitectureFact::Type { fact_id, .. }
            | CompilerArchitectureFact::Trait { fact_id, .. }
            | CompilerArchitectureFact::Implementation { fact_id, .. }
            | CompilerArchitectureFact::Signature { fact_id, .. }
            | CompilerArchitectureFact::Call { fact_id, .. }
            | CompilerArchitectureFact::Constructor { fact_id, .. }
            | CompilerArchitectureFact::Effect { fact_id, .. }
            | CompilerArchitectureFact::Expansion { fact_id, .. }
            | CompilerArchitectureFact::Configuration { fact_id, .. } => *fact_id = identity,
        }
    }

    fn limits() -> ArchitectureCollectionLimits {
        const FIXTURE_LIMIT: usize = 64;
        ArchitectureCollectionLimits {
            max_units: FIXTURE_LIMIT,
            max_shards: FIXTURE_LIMIT,
            max_modules: FIXTURE_LIMIT,
            max_items: FIXTURE_LIMIT,
            max_types: FIXTURE_LIMIT,
            max_traits: FIXTURE_LIMIT,
            max_implementations: FIXTURE_LIMIT,
            max_signatures: FIXTURE_LIMIT,
            max_calls: FIXTURE_LIMIT,
            max_constructors: FIXTURE_LIMIT,
            max_effects: FIXTURE_LIMIT,
            max_expansions: FIXTURE_LIMIT,
            max_configurations: FIXTURE_LIMIT,
            max_unknowns: FIXTURE_LIMIT,
            max_spans: FIXTURE_LIMIT,
            max_span_bytes: FIXTURE_LIMIT,
            max_shard_bytes: FIXTURE_LIMIT * FIXTURE_LIMIT,
            max_artifact_bytes: FIXTURE_LIMIT * FIXTURE_LIMIT,
        }
    }
}
