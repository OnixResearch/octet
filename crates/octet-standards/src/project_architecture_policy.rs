//! Pure project architecture policy model and validation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

/// Project architecture policy schema accepted by this Octet release.
pub const PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION: &str = "octet-project-architecture-policy/v1";

const BLAKE3_HEX_LENGTH: usize = 64;
const PATH_SEPARATOR: char = '/';

/// Project-selected architecture enforcement mode.
// r[impl octet.project_architecture_policy.modes]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureMode {
    /// Record declared facts without promotion blocking.
    Inventory,
    /// Report architecture findings without promotion blocking.
    Advisory,
    /// Block declared architecture violations and required unknown facts.
    Gate,
    /// Add transitive effect-closure admission after the architecture gate.
    Admission,
}

/// Explicit architecture role for a package or source scope.
// r[impl octet.project_architecture_policy.roles]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureRole {
    /// Deterministic domain meaning and policy.
    DomainCore,
    /// Application orchestration and application-owned ports.
    ApplicationShell,
    /// Adapter that invokes the application.
    InboundAdapter,
    /// Adapter that implements an application-owned outbound port.
    OutboundAdapter,
    /// Wiring scope that selects concrete adapters.
    CompositionRoot,
    /// Test-only scope excluded from production claims.
    Test,
    /// Generated scope excluded from authored-source claims.
    Generated,
}

impl ArchitectureRole {
    /// Return true for roles that are part of the production architecture graph.
    #[must_use]
    pub fn is_production(self) -> bool {
        !matches!(self, Self::Test | Self::Generated)
    }
}

/// Kind of project scope bound to an architecture role.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureScopeKind {
    /// Cargo package name.
    Package,
    /// Workspace-relative source scope.
    Source,
}

/// Target and feature selection covered by architecture evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureTarget {
    /// Rust target triple.
    pub triple: String,
    /// Exact enabled Cargo features for this target selection.
    #[serde(default)]
    pub features: Vec<String>,
    /// True when Cargo default features are enabled.
    pub default_features: bool,
}

/// Declared production inventory for architecture coverage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureProductionScope {
    /// Cargo packages included in production claims.
    pub packages: Vec<String>,
    /// Workspace-relative source scopes included in production claims.
    pub source_scopes: Vec<String>,
    /// Target and feature selections included in production claims.
    pub targets: Vec<ArchitectureTarget>,
}

/// Explicit role assignment for one package or source scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureRoleAssignment {
    /// Scope kind.
    pub scope_kind: ArchitectureScopeKind,
    /// Package name or workspace-relative source scope.
    pub scope: String,
    /// Explicit role.
    pub role: ArchitectureRole,
}

/// Permitted dependency edges for one caller role.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDependencyRule {
    /// Role that owns the outgoing dependency edge.
    pub caller_role: ArchitectureRole,
    /// Roles that the caller may depend on.
    pub allowed_callee_roles: Vec<ArchitectureRole>,
    /// Cargo dependency edge kinds covered by this rule.
    pub edge_kinds: Vec<String>,
}

/// Application-owned Hexagonal port declaration.
// r[impl octet.project_architecture_policy.ports]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitecturePort {
    /// Compiler-qualified port trait name.
    pub port_trait: String,
    /// Application-shell source scope that owns the port contract.
    pub application_owner_scope: String,
    /// Adapter source scopes permitted to implement the port.
    pub permitted_adapter_scopes: Vec<String>,
    /// Concrete infrastructure type origins forbidden in inward contracts.
    #[serde(default)]
    pub forbidden_concrete_types: Vec<String>,
    /// True when the port is reserved and does not require an implementation.
    pub reserved: bool,
}

/// Effect-family classification declared by project policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureCapabilityFamily {
    /// Time observation.
    Time,
    /// Random value generation.
    Randomness,
    /// Filesystem access.
    Filesystem,
    /// Network access.
    Network,
    /// Child-process control.
    Process,
    /// Environment access.
    Environment,
    /// Secret access.
    Secrets,
    /// Unsafe Rust or foreign memory operations.
    UnsafeCode,
    /// Ambient mutable global state.
    GlobalState,
    /// Task, thread, lock, or synchronization effects.
    Concurrency,
    /// Logging, tracing, or metrics effects.
    Observability,
    /// Foreign-function calls.
    Ffi,
    /// Resource allocation governed by project policy.
    Allocation,
}

/// Domain-core treatment for one capability family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreCapabilityHandling {
    /// The core receives a prior observation as explicit data.
    ExplicitFact,
    /// The core returns a plan that the shell can interpret.
    EffectPlan,
    /// The capability family is forbidden in the core interface.
    Forbidden,
}

/// Project classification for one capability family.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCapabilityClassification {
    /// Capability family.
    pub family: ArchitectureCapabilityFamily,
    /// Permitted domain-core treatment.
    pub core_handling: CoreCapabilityHandling,
    /// Production role that owns execution.
    pub executor_role: ArchitectureRole,
    /// Reviewable reason for this classification.
    pub rationale: String,
}

/// Optional host-independence obligation for one domain-core scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureNoStdObligation {
    /// Domain-core package or source scope that must remain `no_std`.
    pub scope: String,
}

/// Exact, reviewable architecture finding waiver.
// r[impl octet.project_architecture_policy.waivers]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureWaiver {
    /// Stable architecture finding identity.
    pub finding_id: String,
    /// BLAKE3 identity of the policy used to create the finding.
    pub policy_blake3: String,
    /// BLAKE3 identity of the affected source material.
    pub source_blake3: String,
    /// Named waiver owner.
    pub owner: String,
    /// Reviewable waiver reason.
    pub reason: String,
    /// Condition that requires review or removal.
    pub review_condition: String,
    /// Exact claim affected by this waiver.
    pub affected_claim: String,
}

/// Typed project architecture policy consumed by Octet runtime code.
// r[impl octet.project_architecture_policy.schema]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitecturePolicy {
    /// Policy schema version.
    pub schema_version: String,
    /// Stable project identity.
    pub project_id: String,
    /// Project-selected enforcement mode.
    pub mode: ArchitectureMode,
    /// Production inventory and target coverage.
    pub production: ArchitectureProductionScope,
    /// Complete package, source, test, and generated role assignments.
    pub roles: Vec<ArchitectureRoleAssignment>,
    /// Role dependency rules.
    pub dependency_rules: Vec<ArchitectureDependencyRule>,
    /// Application-owned ports.
    #[serde(default)]
    pub ports: Vec<ArchitecturePort>,
    /// Capability-family classifications.
    pub capability_classifications: Vec<ArchitectureCapabilityClassification>,
    /// Optional `no_std` obligations, separate from FCIS role meaning.
    #[serde(default)]
    pub core_no_std: Vec<ArchitectureNoStdObligation>,
    /// Exact architecture finding waivers.
    #[serde(default)]
    pub waivers: Vec<ArchitectureWaiver>,
    /// Explicit project architecture non-claims.
    pub non_claims: Vec<String>,
}

/// Stable policy validation diagnostic.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProjectArchitecturePolicyDiagnostic {
    /// Stable diagnostic class.
    pub code: String,
    /// Policy field or relation that failed.
    pub field: String,
    /// Reviewable diagnostic message.
    pub message: String,
}

/// Facts required to decide whether one exact waiver is current.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureFindingIdentity<'a> {
    /// Stable architecture finding identity.
    pub finding_id: &'a str,
    /// BLAKE3 identity of the evaluated policy.
    pub policy_blake3: &'a str,
    /// BLAKE3 identity of the affected source.
    pub source_blake3: &'a str,
    /// Exact claim affected by the finding.
    pub affected_claim: &'a str,
}

/// Parse and validate one deterministic project architecture JSON export.
///
/// # Errors
///
/// Returns sorted deterministic diagnostics for malformed JSON or invalid
/// cross-field policy state.
pub fn parse_project_architecture_policy(
    bytes: &[u8],
) -> Result<ProjectArchitecturePolicy, Vec<ProjectArchitecturePolicyDiagnostic>> {
    let policy = serde_json::from_slice::<ProjectArchitecturePolicy>(bytes).map_err(|error| {
        vec![diagnostic(
            "policy-json",
            "export",
            format!("project architecture export is not valid policy JSON: {error}"),
        )]
    })?;
    let diagnostics = validate_project_architecture_policy(&policy);
    if diagnostics.is_empty() {
        Ok(policy)
    } else {
        Err(diagnostics)
    }
}

/// Validate one parsed project architecture policy without I/O.
#[must_use]
pub fn validate_project_architecture_policy(
    policy: &ProjectArchitecturePolicy,
) -> Vec<ProjectArchitecturePolicyDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_scalar_fields(policy, &mut diagnostics);
    let assignments = validate_scopes_and_roles(policy, &mut diagnostics);
    validate_targets(policy, &mut diagnostics);
    validate_dependency_rules(policy, &mut diagnostics);
    validate_ports(policy, &assignments, &mut diagnostics);
    validate_capabilities(policy, &mut diagnostics);
    validate_no_std(policy, &assignments, &mut diagnostics);
    validate_waivers(policy, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Validate one waiver against one current finding identity.
#[must_use]
pub fn validate_architecture_waiver_binding(
    waiver: &ArchitectureWaiver,
    finding: &ArchitectureFindingIdentity<'_>,
) -> Vec<ProjectArchitecturePolicyDiagnostic> {
    let mut diagnostics = Vec::new();
    push_binding_mismatch(&mut diagnostics, "finding_id", &waiver.finding_id, finding.finding_id);
    push_binding_mismatch(&mut diagnostics, "policy_blake3", &waiver.policy_blake3, finding.policy_blake3);
    push_binding_mismatch(&mut diagnostics, "source_blake3", &waiver.source_blake3, finding.source_blake3);
    push_binding_mismatch(&mut diagnostics, "affected_claim", &waiver.affected_claim, finding.affected_claim);
    diagnostics.sort();
    diagnostics
}

fn validate_scalar_fields(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if policy.schema_version != PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            "schema-version",
            "schema_version",
            format!(
                "unsupported project architecture schema `{}`; expected `{PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION}`",
                policy.schema_version
            ),
        ));
    }
    push_blank(diagnostics, "project_id", &policy.project_id);
    if policy.production.packages.is_empty() {
        diagnostics.push(diagnostic(
            "production-coverage",
            "production.packages",
            "project architecture policy must declare at least one production package",
        ));
    }
    if policy.production.source_scopes.is_empty() {
        diagnostics.push(diagnostic(
            "production-coverage",
            "production.source_scopes",
            "project architecture policy must declare at least one production source scope",
        ));
    }
    if policy.non_claims.is_empty() {
        diagnostics.push(diagnostic(
            "missing-non-claims",
            "non_claims",
            "project architecture policy must declare at least one non-claim",
        ));
    }
    validate_non_empty_unique_strings("non_claims", &policy.non_claims, diagnostics);
}

fn validate_scopes_and_roles(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) -> BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole> {
    validate_non_empty_unique_strings("production.packages", &policy.production.packages, diagnostics);
    validate_non_empty_unique_strings("production.source_scopes", &policy.production.source_scopes, diagnostics);
    validate_workspace_relative_paths("production.source_scopes", &policy.production.source_scopes, diagnostics);

    let expected_packages = policy.production.packages.iter().cloned().collect::<BTreeSet<_>>();
    let expected_sources = policy.production.source_scopes.iter().cloned().collect::<BTreeSet<_>>();
    let mut assignments = BTreeMap::new();
    for assignment in &policy.roles {
        push_blank(diagnostics, "roles.scope", &assignment.scope);
        if assignment.scope_kind == ArchitectureScopeKind::Source {
            validate_workspace_relative_path("roles.scope", &assignment.scope, diagnostics);
        }
        let key = (assignment.scope_kind, assignment.scope.clone());
        if assignments.insert(key, assignment.role).is_some() {
            diagnostics.push(diagnostic(
                "ambiguous-role",
                "roles",
                format!("scope `{}` has more than one architecture role", assignment.scope),
            ));
        }
        let is_production_scope = match assignment.scope_kind {
            ArchitectureScopeKind::Package => expected_packages.contains(&assignment.scope),
            ArchitectureScopeKind::Source => expected_sources.contains(&assignment.scope),
        };
        if is_production_scope != assignment.role.is_production() {
            diagnostics.push(diagnostic(
                "role-coverage",
                "roles",
                format!(
                    "scope `{}` has role `{:?}` that does not match its production inclusion",
                    assignment.scope, assignment.role
                ),
            ));
        }
    }

    for package in &expected_packages {
        require_assignment(&assignments, ArchitectureScopeKind::Package, package, diagnostics);
    }
    for source in &expected_sources {
        require_assignment(&assignments, ArchitectureScopeKind::Source, source, diagnostics);
    }

    let source_assignments = policy
        .roles
        .iter()
        .filter(|assignment| assignment.scope_kind == ArchitectureScopeKind::Source)
        .collect::<Vec<_>>();
    for (index, left) in source_assignments.iter().enumerate() {
        for right in source_assignments.iter().skip(index.saturating_add(1)) {
            if paths_overlap(&left.scope, &right.scope) {
                diagnostics.push(diagnostic(
                    "overlapping-scope",
                    "roles",
                    format!("source scopes `{}` and `{}` overlap", left.scope, right.scope),
                ));
            }
        }
    }
    assignments
}

fn validate_targets(policy: &ProjectArchitecturePolicy, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    if policy.production.targets.is_empty() {
        diagnostics.push(diagnostic(
            "target-coverage",
            "production.targets",
            "project architecture policy must declare at least one target selection",
        ));
        return;
    }
    let mut target_keys = BTreeSet::new();
    for target in &policy.production.targets {
        push_blank(diagnostics, "production.targets.triple", &target.triple);
        validate_non_empty_unique_strings("production.targets.features", &target.features, diagnostics);
        let mut features = target.features.clone();
        features.sort();
        let key = (target.triple.clone(), features, target.default_features);
        if !target_keys.insert(key) {
            diagnostics.push(diagnostic(
                "duplicate-target",
                "production.targets",
                format!("target selection `{}` is declared more than once", target.triple),
            ));
        }
    }
}

fn validate_dependency_rules(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let production_roles = policy
        .roles
        .iter()
        .filter(|assignment| assignment.role.is_production())
        .map(|assignment| assignment.role)
        .collect::<BTreeSet<_>>();
    let mut callers = BTreeSet::new();
    for rule in &policy.dependency_rules {
        if !rule.caller_role.is_production() {
            diagnostics.push(diagnostic(
                "invalid-role-rule",
                "dependency_rules.caller_role",
                "test and generated roles cannot own production dependency rules",
            ));
        }
        if !callers.insert(rule.caller_role) {
            diagnostics.push(diagnostic(
                "duplicate-role-rule",
                "dependency_rules",
                format!("role `{:?}` has more than one dependency rule", rule.caller_role),
            ));
        }
        if rule.allowed_callee_roles.iter().any(|role| !role.is_production()) {
            diagnostics.push(diagnostic(
                "invalid-role-rule",
                "dependency_rules.allowed_callee_roles",
                "production dependency rules cannot permit test or generated roles",
            ));
        }
        if rule.edge_kinds.is_empty() {
            diagnostics.push(diagnostic(
                "missing-edge-kinds",
                "dependency_rules.edge_kinds",
                "each dependency rule must select at least one Cargo edge kind",
            ));
        }
        validate_non_empty_unique_strings("dependency_rules.edge_kinds", &rule.edge_kinds, diagnostics);
    }
    for role in production_roles {
        if !callers.contains(&role) {
            diagnostics.push(diagnostic(
                "missing-role-rule",
                "dependency_rules",
                format!("production role `{role:?}` has no dependency rule"),
            ));
        }
    }
}

fn validate_ports(
    policy: &ProjectArchitecturePolicy,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut traits = BTreeSet::new();
    for port in &policy.ports {
        push_blank(diagnostics, "ports.port_trait", &port.port_trait);
        if !traits.insert(port.port_trait.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-port",
                "ports.port_trait",
                format!("port trait `{}` is declared more than once", port.port_trait),
            ));
        }
        require_scope_role(
            assignments,
            &port.application_owner_scope,
            ArchitectureRole::ApplicationShell,
            "ports.application_owner_scope",
            diagnostics,
        );
        if !port.reserved && port.permitted_adapter_scopes.is_empty() {
            diagnostics.push(diagnostic(
                "missing-adapter-scope",
                "ports.permitted_adapter_scopes",
                format!("non-reserved port `{}` has no permitted adapter scope", port.port_trait),
            ));
        }
        validate_non_empty_unique_strings(
            "ports.permitted_adapter_scopes",
            &port.permitted_adapter_scopes,
            diagnostics,
        );
        validate_non_empty_unique_strings(
            "ports.forbidden_concrete_types",
            &port.forbidden_concrete_types,
            diagnostics,
        );
        for adapter_scope in &port.permitted_adapter_scopes {
            let role = assignments.get(&(ArchitectureScopeKind::Source, adapter_scope.clone()));
            if !matches!(role, Some(ArchitectureRole::InboundAdapter | ArchitectureRole::OutboundAdapter)) {
                diagnostics.push(diagnostic(
                    "invalid-adapter-role",
                    "ports.permitted_adapter_scopes",
                    format!("port adapter scope `{adapter_scope}` is not assigned an adapter role"),
                ));
            }
        }
    }
}

fn validate_capabilities(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if policy.capability_classifications.is_empty() {
        diagnostics.push(diagnostic(
            "missing-capability-classification",
            "capability_classifications",
            "project architecture policy must classify at least one capability family",
        ));
    }
    let mut families = BTreeSet::new();
    for classification in &policy.capability_classifications {
        if !families.insert(classification.family) {
            diagnostics.push(diagnostic(
                "duplicate-capability-family",
                "capability_classifications.family",
                format!("capability family `{:?}` is classified more than once", classification.family),
            ));
        }
        if !matches!(
            classification.executor_role,
            ArchitectureRole::ApplicationShell | ArchitectureRole::OutboundAdapter | ArchitectureRole::CompositionRoot
        ) {
            diagnostics.push(diagnostic(
                "invalid-capability-executor",
                "capability_classifications.executor_role",
                "capability execution must belong to the application shell, an outbound adapter, or the composition root",
            ));
        }
        push_blank(diagnostics, "capability_classifications.rationale", &classification.rationale);
    }
}

fn validate_no_std(
    policy: &ProjectArchitecturePolicy,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut scopes = BTreeSet::new();
    for obligation in &policy.core_no_std {
        push_blank(diagnostics, "core_no_std.scope", &obligation.scope);
        if !scopes.insert(obligation.scope.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-no-std-scope",
                "core_no_std.scope",
                format!("no_std scope `{}` is declared more than once", obligation.scope),
            ));
        }
        let role = assignments
            .get(&(ArchitectureScopeKind::Package, obligation.scope.clone()))
            .or_else(|| assignments.get(&(ArchitectureScopeKind::Source, obligation.scope.clone())));
        if role != Some(&ArchitectureRole::DomainCore) {
            diagnostics.push(diagnostic(
                "invalid-no-std-role",
                "core_no_std.scope",
                format!("no_std scope `{}` is not assigned domain-core", obligation.scope),
            ));
        }
    }
}

fn validate_waivers(policy: &ProjectArchitecturePolicy, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    let mut finding_ids = BTreeSet::new();
    for waiver in &policy.waivers {
        push_blank(diagnostics, "waivers.finding_id", &waiver.finding_id);
        push_blank(diagnostics, "waivers.owner", &waiver.owner);
        push_blank(diagnostics, "waivers.reason", &waiver.reason);
        push_blank(diagnostics, "waivers.review_condition", &waiver.review_condition);
        push_blank(diagnostics, "waivers.affected_claim", &waiver.affected_claim);
        validate_blake3("waivers.policy_blake3", &waiver.policy_blake3, diagnostics);
        validate_blake3("waivers.source_blake3", &waiver.source_blake3, diagnostics);
        if !finding_ids.insert(waiver.finding_id.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-waiver",
                "waivers.finding_id",
                format!("finding `{}` has more than one architecture waiver", waiver.finding_id),
            ));
        }
    }
}

fn require_assignment(
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    kind: ArchitectureScopeKind,
    scope: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if !assignments.contains_key(&(kind, scope.to_owned())) {
        diagnostics.push(diagnostic(
            "missing-role",
            "roles",
            format!("production scope `{scope}` has no architecture role"),
        ));
    }
}

fn require_scope_role(
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    scope: &str,
    required: ArchitectureRole,
    field: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let actual = assignments.get(&(ArchitectureScopeKind::Source, scope.to_owned()));
    if actual != Some(&required) {
        diagnostics.push(diagnostic(
            "invalid-scope-role",
            field,
            format!("scope `{scope}` must have role `{required:?}`; found {actual:?}"),
        ));
    }
}

fn validate_non_empty_unique_strings(
    field: &str,
    values: &[String],
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        push_blank(diagnostics, field, value);
        if !seen.insert(value.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-value",
                field,
                format!("value `{value}` is declared more than once"),
            ));
        }
    }
}

fn validate_workspace_relative_paths(
    field: &str,
    paths: &[String],
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    for path in paths {
        validate_workspace_relative_path(field, path, diagnostics);
    }
}

fn validate_workspace_relative_path(
    field: &str,
    path: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let normalized = path.replace('\\', "/");
    let escapes = normalized.starts_with(PATH_SEPARATOR)
        || normalized.split(PATH_SEPARATOR).any(|component| component == "..")
        || normalized.is_empty()
        || normalized == ".";
    if escapes {
        diagnostics.push(diagnostic(
            "invalid-workspace-path",
            field,
            format!("path `{path}` must be workspace-relative and must not escape the workspace"),
        ));
    }
}

fn paths_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_end_matches(PATH_SEPARATOR);
    let right = right.trim_end_matches(PATH_SEPARATOR);
    left == right
        || left.strip_prefix(right).is_some_and(|suffix| suffix.starts_with(PATH_SEPARATOR))
        || right.strip_prefix(left).is_some_and(|suffix| suffix.starts_with(PATH_SEPARATOR))
}

fn validate_blake3(field: &str, value: &str, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    let valid = value.len() == BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !valid {
        diagnostics.push(diagnostic(
            "invalid-blake3",
            field,
            format!("BLAKE3 identity must contain exactly {BLAKE3_HEX_LENGTH} lowercase hexadecimal characters"),
        ));
    }
}

fn push_blank(diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(diagnostic(
            "empty-field",
            field,
            format!("project architecture field `{field}` must not be empty"),
        ));
    }
}

fn push_binding_mismatch(
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
    field: &str,
    actual: &str,
    expected: &str,
) {
    if actual != expected {
        diagnostics.push(diagnostic(
            "stale-waiver",
            format!("waiver.{field}"),
            format!("architecture waiver `{field}` is stale; expected `{expected}`, found `{actual}`"),
        ));
    }
}

fn diagnostic(
    code: impl Into<String>,
    field: impl Into<String>,
    message: impl Into<String>,
) -> ProjectArchitecturePolicyDiagnostic {
    ProjectArchitecturePolicyDiagnostic {
        code: code.into(),
        field: field.into(),
        message: message.into(),
    }
}

// r[impl octet.project_architecture_policy.verification]
#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const STALE_BINDING_DIAGNOSTICS: usize = 2;

    #[test]
    fn valid_policy_accepts_every_mode_and_role() {
        let mut policy = valid_policy();
        let modes = [
            ArchitectureMode::Inventory,
            ArchitectureMode::Advisory,
            ArchitectureMode::Gate,
            ArchitectureMode::Admission,
        ];
        for mode in modes {
            policy.mode = mode;
            assert_eq!(validate_project_architecture_policy(&policy), Vec::new());
        }
        let roles = policy.roles.iter().map(|assignment| assignment.role).collect::<BTreeSet<_>>();
        assert_eq!(
            roles,
            BTreeSet::from([
                ArchitectureRole::DomainCore,
                ArchitectureRole::ApplicationShell,
                ArchitectureRole::InboundAdapter,
                ArchitectureRole::OutboundAdapter,
                ArchitectureRole::CompositionRoot,
                ArchitectureRole::Test,
                ArchitectureRole::Generated,
            ])
        );
    }

    #[test]
    fn invalid_policy_rejects_unknown_role_overlap_and_missing_coverage() {
        let mut value = serde_json::to_value(valid_policy()).expect("serialize valid policy");
        value["roles"][0]["role"] = serde_json::Value::String("core-ish".to_owned());
        let unknown = serde_json::to_vec(&value).expect("serialize malformed policy");
        let unknown_diagnostics = parse_project_architecture_policy(&unknown).expect_err("unknown role must fail");
        assert!(unknown_diagnostics.iter().any(|diagnostic| diagnostic.code == "policy-json"));

        let mut policy = valid_policy();
        policy.roles.retain(|assignment| assignment.scope != "crates/domain/src");
        policy.roles.push(ArchitectureRoleAssignment {
            scope_kind: ArchitectureScopeKind::Source,
            scope: "crates/application/src/nested".to_owned(),
            role: ArchitectureRole::Generated,
        });
        let diagnostics = validate_project_architecture_policy(&policy);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "missing-role"));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "overlapping-scope"));
    }

    #[test]
    fn waiver_binding_accepts_current_and_rejects_stale_facts() {
        let waiver = ArchitectureWaiver {
            finding_id: "architecture.port.owner".to_owned(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: HASH_B.to_owned(),
            owner: "architecture-team".to_owned(),
            reason: "migration is bounded".to_owned(),
            review_condition: "remove after adapter migration".to_owned(),
            affected_claim: "application-owned-port".to_owned(),
        };
        let current = ArchitectureFindingIdentity {
            finding_id: "architecture.port.owner",
            policy_blake3: HASH_A,
            source_blake3: HASH_B,
            affected_claim: "application-owned-port",
        };
        assert_eq!(validate_architecture_waiver_binding(&waiver, &current), Vec::new());

        let stale = ArchitectureFindingIdentity {
            finding_id: current.finding_id,
            policy_blake3: HASH_B,
            source_blake3: HASH_A,
            affected_claim: current.affected_claim,
        };
        let diagnostics = validate_architecture_waiver_binding(&waiver, &stale);
        assert_eq!(diagnostics.len(), STALE_BINDING_DIAGNOSTICS);
        assert!(diagnostics.iter().all(|diagnostic| diagnostic.code == "stale-waiver"));
    }

    fn valid_policy() -> ProjectArchitecturePolicy {
        let production_packages = ["domain", "application", "inbound", "outbound", "host"];
        let production_sources = [
            "crates/domain/src",
            "crates/application/src",
            "crates/inbound/src",
            "crates/outbound/src",
            "crates/host/src",
        ];
        ProjectArchitecturePolicy {
            schema_version: PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION.to_owned(),
            project_id: "example-project".to_owned(),
            mode: ArchitectureMode::Gate,
            production: ArchitectureProductionScope {
                packages: production_packages.iter().map(ToString::to_string).collect(),
                source_scopes: production_sources.iter().map(ToString::to_string).collect(),
                targets: vec![ArchitectureTarget {
                    triple: "x86_64-unknown-linux-gnu".to_owned(),
                    features: vec!["production".to_owned()],
                    default_features: false,
                }],
            },
            roles: vec![
                role(ArchitectureScopeKind::Package, "domain", ArchitectureRole::DomainCore),
                role(ArchitectureScopeKind::Package, "application", ArchitectureRole::ApplicationShell),
                role(ArchitectureScopeKind::Package, "inbound", ArchitectureRole::InboundAdapter),
                role(ArchitectureScopeKind::Package, "outbound", ArchitectureRole::OutboundAdapter),
                role(ArchitectureScopeKind::Package, "host", ArchitectureRole::CompositionRoot),
                role(ArchitectureScopeKind::Source, "crates/domain/src", ArchitectureRole::DomainCore),
                role(ArchitectureScopeKind::Source, "crates/application/src", ArchitectureRole::ApplicationShell),
                role(ArchitectureScopeKind::Source, "crates/inbound/src", ArchitectureRole::InboundAdapter),
                role(ArchitectureScopeKind::Source, "crates/outbound/src", ArchitectureRole::OutboundAdapter),
                role(ArchitectureScopeKind::Source, "crates/host/src", ArchitectureRole::CompositionRoot),
                role(ArchitectureScopeKind::Source, "tests", ArchitectureRole::Test),
                role(ArchitectureScopeKind::Source, "target/generated", ArchitectureRole::Generated),
            ],
            dependency_rules: vec![
                dependency_rule(ArchitectureRole::DomainCore, &[ArchitectureRole::DomainCore]),
                dependency_rule(ArchitectureRole::ApplicationShell, &[
                    ArchitectureRole::ApplicationShell,
                    ArchitectureRole::DomainCore,
                ]),
                dependency_rule(ArchitectureRole::InboundAdapter, &[
                    ArchitectureRole::InboundAdapter,
                    ArchitectureRole::ApplicationShell,
                ]),
                dependency_rule(ArchitectureRole::OutboundAdapter, &[
                    ArchitectureRole::OutboundAdapter,
                    ArchitectureRole::ApplicationShell,
                ]),
                dependency_rule(ArchitectureRole::CompositionRoot, &[
                    ArchitectureRole::CompositionRoot,
                    ArchitectureRole::ApplicationShell,
                    ArchitectureRole::InboundAdapter,
                    ArchitectureRole::OutboundAdapter,
                ]),
            ],
            ports: vec![ArchitecturePort {
                port_trait: "application::Clock".to_owned(),
                application_owner_scope: "crates/application/src".to_owned(),
                permitted_adapter_scopes: vec!["crates/outbound/src".to_owned()],
                forbidden_concrete_types: vec!["std::time::SystemTime".to_owned()],
                reserved: false,
            }],
            capability_classifications: vec![ArchitectureCapabilityClassification {
                family: ArchitectureCapabilityFamily::Time,
                core_handling: CoreCapabilityHandling::ExplicitFact,
                executor_role: ArchitectureRole::OutboundAdapter,
                rationale: "the shell observes time before domain evaluation".to_owned(),
            }],
            core_no_std: vec![ArchitectureNoStdObligation {
                scope: "domain".to_owned(),
            }],
            waivers: vec![ArchitectureWaiver {
                finding_id: "architecture.example".to_owned(),
                policy_blake3: HASH_A.to_owned(),
                source_blake3: HASH_B.to_owned(),
                owner: "architecture-team".to_owned(),
                reason: "bounded migration".to_owned(),
                review_condition: "remove after migration".to_owned(),
                affected_claim: "example-claim".to_owned(),
            }],
            non_claims: vec!["runtime-correctness".to_owned(), "release-eligibility".to_owned()],
        }
    }

    fn role(scope_kind: ArchitectureScopeKind, scope: &str, role: ArchitectureRole) -> ArchitectureRoleAssignment {
        ArchitectureRoleAssignment {
            scope_kind,
            scope: scope.to_owned(),
            role,
        }
    }

    fn dependency_rule(
        caller_role: ArchitectureRole,
        allowed_callee_roles: &[ArchitectureRole],
    ) -> ArchitectureDependencyRule {
        ArchitectureDependencyRule {
            caller_role,
            allowed_callee_roles: allowed_callee_roles.to_vec(),
            edge_kinds: vec!["normal".to_owned(), "build".to_owned()],
        }
    }
}
