//! Shared data model for Tiger Style policy checking.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

/// Policy classification recorded by the registry.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Classification {
    /// Machine-enforced rule.
    Hard,
    /// Non-blocking report.
    Advisory,
    /// Human-process guidance, not machine enforced.
    HumanProcessOnly,
    /// Known candidate held for later implementation.
    Deferred,
}

/// Policy enforcement level used by checker exits.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementLevel {
    /// Blocking rule.
    Hard,
    /// Non-blocking rule.
    Advisory,
    /// Human-only rule.
    HumanProcessOnly,
    /// Deferred rule.
    Deferred,
}

/// Policy enforcement mode declared by the Nickel registry.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementMode {
    /// Advisory lint observation.
    #[default]
    Lint,
    /// Blocking gate observation.
    Gate,
    /// Rust construction mechanisms make violations structurally invalid.
    Structural,
    /// Local proof or proof-equivalent rails back the policy.
    Verified,
    /// Human/oracle judgment is required.
    HumanOracle,
}

/// Mechanism kind exported from Nickel for structural policies.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MechanismKind {
    /// Crate graph or crate attribute boundary.
    #[default]
    CrateBoundary,
    /// Shell-owned capability API.
    CapabilityApi,
    /// Bounded primitive API.
    BoundedPrimitive,
    /// Type-state protocol API.
    TypeState,
}

/// Structural mechanism metadata exported from Nickel.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Mechanism {
    /// Mechanism kind.
    pub kind: MechanismKind,
    /// Human summary.
    pub description: String,
    /// Exported obligations consumed by checkers or reviewers.
    #[serde(default)]
    pub exported_obligations: Vec<String>,
    /// Crate-boundary obligations.
    #[serde(default)]
    pub crate_boundary: Vec<String>,
    /// Capability constructor owners.
    #[serde(default)]
    pub constructor_authority: Vec<String>,
    /// API obligations for bounded/type-state mechanisms.
    #[serde(default)]
    pub api_obligations: Vec<String>,
    /// Forbidden API patterns for bounded primitive mechanisms.
    #[serde(default)]
    pub forbidden_apis: Vec<String>,
    /// Approved bounded API patterns for bounded primitive mechanisms.
    #[serde(default)]
    pub approved_bounded_apis: Vec<String>,
    /// Approved wrapper scopes/functions for bounded primitive mechanisms.
    #[serde(default)]
    pub approved_wrapper_scopes: Vec<String>,
    /// Capacity witness strings accepted by bounded primitive mechanisms.
    #[serde(default)]
    pub capacity_witnesses: Vec<String>,
    /// Backpressure or overflow behavior witness strings accepted by bounded primitive mechanisms.
    #[serde(default)]
    pub backpressure_behaviors: Vec<String>,
    /// Forbidden ambient authority API patterns for capability-api mechanisms.
    #[serde(default)]
    pub forbidden_ambient_apis: Vec<String>,
    /// Core source path scopes where ambient authority constructors are forbidden.
    #[serde(default)]
    pub core_source_scopes: Vec<String>,
    /// Shell source path scopes where ambient authority constructors are allowed.
    #[serde(default)]
    pub shell_constructor_scopes: Vec<String>,
    /// Approved capability trait, token, or effect-data names for capability-api mechanisms.
    #[serde(default)]
    pub approved_capability_names: Vec<String>,
    /// Source path scopes where type-state protocol obligations apply.
    #[serde(default)]
    pub type_state_source_scopes: Vec<String>,
    /// Raw state alias patterns rejected by type-state mechanisms.
    #[serde(default)]
    pub raw_state_alias_patterns: Vec<String>,
    /// Raw transition or bypass patterns rejected by type-state mechanisms.
    #[serde(default)]
    pub raw_transition_patterns: Vec<String>,
    /// Required nominal state marker names accepted as type-state evidence.
    #[serde(default)]
    pub required_state_markers: Vec<String>,
    /// Required transition method names accepted as type-state evidence.
    #[serde(default)]
    pub required_transition_methods: Vec<String>,
}

/// Deterministic evidence command metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceCommand {
    /// Command to run.
    pub command: String,
    /// Expected pass signal.
    pub expected_signal: String,
}

impl Default for EvidenceCommand {
    fn default() -> Self {
        Self {
            command: "registry validation".to_owned(),
            expected_signal: "export succeeds".to_owned(),
        }
    }
}

/// Receipt verifier/toolchain metadata required from verified proof receipts.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptVerifier {
    /// Verifier/tool identity expected in proof receipts.
    #[serde(default)]
    pub verifier_id: String,
    /// Deterministic verifier/tool digest expected in proof receipts.
    #[serde(default)]
    pub verifier_digest: String,
    /// Verifier binary/build artifact path or label expected in proof receipts.
    #[serde(default)]
    pub verifier_binary_path: String,
    /// BLAKE3 digest of the verifier binary/build artifact expected in proof receipts.
    #[serde(default)]
    pub verifier_binary_digest: String,
    /// Verifier build closure identity expected in proof receipts.
    #[serde(default)]
    pub verifier_build_closure: String,
    /// BLAKE3 digest of the verifier build closure manifest expected in proof receipts.
    #[serde(default)]
    pub verifier_build_closure_digest: String,
    /// Source-root-relative build closure manifest paths covered by generated receipts.
    #[serde(default)]
    pub verifier_build_closure_paths: Vec<String>,
    /// Toolchain identity expected in proof receipts.
    #[serde(default)]
    pub toolchain_id: String,
    /// Deterministic toolchain digest expected in proof receipts.
    #[serde(default)]
    pub toolchain_digest: String,
}

/// Receipt provenance metadata required from verified proof receipts.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptProvenance {
    /// Receipt schema version line expected in proof receipts.
    #[serde(default)]
    pub schema_version: String,
    /// Runner identity expected in proof receipts.
    #[serde(default)]
    pub runner: String,
    /// Execution environment account expected in proof receipts.
    #[serde(default)]
    pub environment: String,
    /// True when receipts must name the configured backend.
    #[serde(default)]
    pub requires_backend: bool,
    /// True when receipts must include at least one configured trust-boundary line.
    #[serde(default)]
    pub requires_trust_boundary: bool,
}

/// Verified-mode proof rail metadata.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProofRail {
    /// Proof backend identifier.
    pub backend: String,
    /// Proof command.
    pub command: String,
    /// Source files covered by this rail.
    #[serde(default)]
    pub source_paths: Vec<String>,
    /// Proof or fixture paths.
    #[serde(default)]
    pub proof_paths: Vec<String>,
    /// Expected success signal.
    pub pass_signal: String,
    /// Trust-boundary notes.
    #[serde(default)]
    pub trust_boundaries: Vec<String>,
    /// Receipt binding mode, such as `none` or `blake3-command-and-source`.
    #[serde(default)]
    pub receipt_binding: String,
    /// Receipt provenance obligations required from proof receipts.
    #[serde(default)]
    pub receipt_provenance: ReceiptProvenance,
    /// Receipt verifier/toolchain obligations required from proof receipts.
    #[serde(default)]
    pub receipt_verifier: ReceiptVerifier,
}

/// Seed role exported from Nickel.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeedRole {
    /// Architecture reference only.
    ArchitectureReference,
    /// Trajectory fixture.
    TrajectoryFixture,
    /// Proof-relevant seed.
    ProofSeed,
    /// Core/shell shape fixture.
    CoreShellFixture,
}

/// Reference seed manifest metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SeedManifest {
    /// Stable seed name.
    pub name: String,
    /// Source path or repository.
    pub source_path: String,
    /// Source commit or local reference marker.
    pub source_commit: String,
    /// Seed role.
    pub role: SeedRole,
    /// Included paths.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Provenance note.
    pub provenance: String,
    /// License note.
    pub license: String,
    /// Content hash or manifest gate marker.
    pub content_hash: String,
    /// True when proof-relevant.
    pub proof_relevant: bool,
    /// Local acceptance commands/reviews.
    #[serde(default)]
    pub local_acceptance: Vec<String>,
}

/// Diagnostic severity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Blocking diagnostic.
    Error,
    /// Non-blocking warning.
    Warning,
    /// Informational finding.
    Info,
    /// Low-priority note.
    Note,
}

/// Suppression metadata requirements for one policy.
#[expect(
    clippy::struct_excessive_bools,
    reason = "wire schema mirrors Nickel boolean requirements for each metadata field"
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SuppressionRequirements {
    /// Suppression must cite the policy ID.
    pub requires_policy_id: bool,
    /// Suppression must cite the Tiger Style source section.
    pub requires_source_section: bool,
    /// Suppression must identify an owner.
    pub requires_owner: bool,
    /// Suppression must explain the rationale.
    pub requires_rationale: bool,
    /// Suppression must describe when it is reviewed.
    pub requires_review_trigger: bool,
    /// Suppression must describe the safer default being bypassed.
    pub requires_safer_default: bool,
}

/// Fixture coverage status for one policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureStatus {
    /// Fixtures are planned but not complete.
    Planned,
    /// Fixtures are complete.
    Covered,
    /// Fixtures are not applicable for this policy.
    NotApplicable,
}

/// Registry fixture coverage fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FixtureCoverage {
    /// Coverage status.
    pub status: FixtureStatus,
    /// Passing fixture references.
    pub compliant: Vec<String>,
    /// Failing fixture references.
    pub noncompliant: Vec<String>,
}

/// One policy registry entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyEntry {
    /// Stable policy ID.
    pub policy_id: String,
    /// Tiger Style pillar.
    pub pillar: String,
    /// Source file path.
    pub source_path: String,
    /// Source heading.
    pub source_heading: String,
    /// Source anchor.
    pub source_anchor: String,
    /// Human summary.
    pub summary: String,
    /// Policy classification.
    pub classification: Classification,
    /// Enforcement level.
    pub enforcement_level: EnforcementLevel,
    /// Current enforcement mode.
    #[serde(default)]
    pub current_enforcement_mode: EnforcementMode,
    /// Target enforcement mode.
    #[serde(default)]
    pub target_enforcement_mode: EnforcementMode,
    /// Required criteria for target-mode escalation.
    #[serde(default)]
    pub escalation_criteria: String,
    /// Structural mechanisms declared for this policy.
    #[serde(default)]
    pub mechanisms: Vec<Mechanism>,
    /// Deterministic checker command metadata.
    #[serde(default)]
    pub deterministic_checker: EvidenceCommand,
    /// Verified proof rails.
    #[serde(default)]
    pub proof_rails: Vec<ProofRail>,
    /// Owner for checker behavior.
    pub checker_owner: String,
    /// Canonical checker ID.
    pub checker_id: String,
    /// Default diagnostic severity.
    pub default_severity: Severity,
    /// Suppression metadata requirements.
    pub suppression_requirements: SuppressionRequirements,
    /// Registry-declared fixture coverage.
    pub fixture_coverage: FixtureCoverage,
    /// Evidence required before this policy is considered verified.
    pub verification_evidence_requirements: Vec<String>,
    /// Why the classification was chosen.
    pub classification_rationale: String,
    /// False-positive risk statement.
    pub false_positive_risk: String,
    /// Existing toolchain rails that may enforce this policy.
    pub existing_enforcement: Vec<String>,
    /// Why a custom checker is justified, if one is used.
    pub custom_checker_justification: String,
}

/// Exported policy registry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyRegistry {
    /// Registry schema version.
    pub schema_version: String,
    /// Valid checker IDs.
    pub checker_ids: Vec<String>,
    /// Policy entries.
    pub policies: Vec<PolicyEntry>,
    /// Optional reference seed manifests.
    #[serde(default)]
    pub seed_manifests: Vec<SeedManifest>,
}

impl PolicyRegistry {
    /// Returns policy by stable policy ID.
    #[must_use]
    pub fn policy_by_id(&self, policy_id: &str) -> Option<&PolicyEntry> {
        self.policies.iter().find(|policy| policy.policy_id == policy_id)
    }

    /// Returns true when a checker ID is declared in the registry.
    #[must_use]
    pub fn has_checker_id(&self, checker_id: &str) -> bool {
        self.checker_ids.iter().any(|known_id| known_id == checker_id)
    }
}

/// Rule-family selection for checker runs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FamilySelection {
    selected: BTreeSet<String>,
}

impl FamilySelection {
    /// Selects every family.
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Builds a family selection from pillar names.
    #[must_use]
    pub fn from_families<I, S>(families: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let selected = families.into_iter().map(Into::into).collect();
        Self { selected }
    }

    /// Returns true when all families are selected.
    #[must_use]
    pub fn includes_all(&self) -> bool {
        self.selected.is_empty()
    }

    /// Returns true when this family should run.
    #[must_use]
    pub fn includes(&self, family: &str) -> bool {
        self.includes_all() || self.selected.contains(family)
    }
}
