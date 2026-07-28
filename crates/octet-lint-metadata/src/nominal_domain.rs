use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const NOMINAL_DOMAIN_SCHEMA_VERSION: &str = "octet-nominal-domain-policy/v1";
pub const NOMINAL_DOMAIN_INVENTORY_SCHEMA_VERSION: &str = "octet-nominal-domain-inventory/v1";

const BLAKE3_HEX_LENGTH: usize = 64;
const NOMINAL_DOMAIN_CATALOG_JSON: &str = include_str!("../../../standards/nominal-domains/generated/catalog.json");

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementMode {
    Inventory,
    Warn,
    Deny,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RawRepresentation {
    String,
    U64,
    Usize,
    Path,
}

impl RawRepresentation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::Path => "path",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperKind {
    OpaqueNewtype,
    RefinedNewtype,
    MarkerGeneric,
    UnitNewtype,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConstructorPolicy {
    PrivateInfallible,
    PrivateFallible,
    MarkerOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NominalDomainPolicy {
    pub domain_id: String,
    pub symbols: Vec<String>,
    pub raw_representation: RawRepresentation,
    pub core_scopes: Vec<String>,
    pub compatibility_scopes: Vec<String>,
    pub compatibility_required: bool,
    pub wrapper_kind: WrapperKind,
    pub constructor_policy: ConstructorPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NominalDomainCatalog {
    pub schema_version: String,
    pub default_mode: EnforcementMode,
    pub heuristic_suffixes: Vec<String>,
    pub domains: Vec<NominalDomainPolicy>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorKind {
    TypeAlias,
    Field,
    Parameter,
    Result,
    EnumPayload,
    MapKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainDescriptor<'a> {
    pub symbol: &'a str,
    pub raw_representation: RawRepresentation,
    pub source_path: &'a str,
    pub kind: DescriptorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainDisposition {
    ExactCore { domain_id: String },
    Compatibility { domain_id: String },
    ExactOutsideScope { domain_id: String },
    HeuristicCandidate { suffix: String },
    Unclassified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WrapperFacts {
    pub inner_field_public: bool,
    pub unchecked_from_raw: bool,
    pub has_fallible_constructor: bool,
    pub boundary_erasing_deref: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventorySource {
    pub repository: String,
    pub revision: String,
    pub path: String,
    pub blake3: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InventoryClassification {
    ExactCore,
    HeuristicCandidate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventoryFinding {
    pub lint_name: String,
    pub classification: InventoryClassification,
    pub domain_id: Option<String>,
    pub symbol: String,
    pub source_path: String,
    pub line: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventorySummary {
    pub exact_findings: usize,
    pub heuristic_candidates: usize,
    pub compatibility_skips: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventoryReview {
    pub reviewed: bool,
    pub true_positives: usize,
    pub false_positives: usize,
    pub coverage_gaps: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventoryReceipt {
    pub schema_version: String,
    pub pilot_id: String,
    pub mode: EnforcementMode,
    pub source: InventorySource,
    pub policy_blake3: String,
    pub findings: Vec<InventoryFinding>,
    pub summary: InventorySummary,
    pub review: InventoryReview,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WrapperViolation {
    PublicInnerField,
    UncheckedPrimitiveConversion,
    MissingFallibleConstructor,
    BoundaryErasingDeref,
}

/// Loads the checked-in nominal-domain catalog.
///
/// # Errors
///
/// Returns an error when the JSON projection is malformed or violates the Rust contract.
pub fn load_nominal_domain_catalog() -> Result<NominalDomainCatalog, String> {
    parse_nominal_domain_catalog(NOMINAL_DOMAIN_CATALOG_JSON)
}

/// Parses and checks a nominal-domain catalog without file or compiler access.
///
/// # Errors
///
/// Returns deterministic diagnostics for malformed or invalid policy data.
pub fn parse_nominal_domain_catalog(text: &str) -> Result<NominalDomainCatalog, String> {
    let catalog = serde_json::from_str::<NominalDomainCatalog>(text)
        .map_err(|error| format!("parse nominal-domain catalog: {error}"))?;
    let diagnostics = nominal_domain_catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        return Ok(catalog);
    }

    Err(format!("invalid nominal-domain catalog: {}", diagnostics.join("; ")))
}

#[must_use]
pub fn nominal_domain_catalog_diagnostics(catalog: &NominalDomainCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_catalog_header(catalog, &mut diagnostics);
    validate_domain_rows(catalog, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_catalog_header(catalog: &NominalDomainCatalog, diagnostics: &mut Vec<String>) {
    if catalog.schema_version != NOMINAL_DOMAIN_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{NOMINAL_DOMAIN_SCHEMA_VERSION}`, got `{}`",
            catalog.schema_version
        ));
    }
    if catalog.domains.is_empty() {
        diagnostics.push("domains must not be empty".to_owned());
    }
    validate_non_empty_unique_values("heuristic_suffixes", &catalog.heuristic_suffixes, diagnostics);
    validate_non_empty_unique_values("non_claims", &catalog.non_claims, diagnostics);
}

fn validate_domain_rows(catalog: &NominalDomainCatalog, diagnostics: &mut Vec<String>) {
    let mut domain_ids = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for domain in &catalog.domains {
        validate_domain_identity(domain, &mut domain_ids, &mut symbols, diagnostics);
        validate_domain_scopes(domain, diagnostics);
        validate_constructor_policy(domain, diagnostics);
    }
}

fn validate_domain_identity(
    domain: &NominalDomainPolicy,
    domain_ids: &mut BTreeSet<String>,
    symbols: &mut BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    if domain.domain_id.trim().is_empty() {
        diagnostics.push("domain_id must not be empty".to_owned());
    } else if !domain_ids.insert(domain.domain_id.clone()) {
        diagnostics.push(format!("duplicate domain_id `{}`", domain.domain_id));
    }
    if domain.symbols.is_empty() {
        diagnostics.push(format!("domain `{}` symbols must not be empty", domain.domain_id));
    }
    for symbol in &domain.symbols {
        if symbol.trim().is_empty() {
            diagnostics.push(format!("domain `{}` contains an empty symbol", domain.domain_id));
        } else if !symbols.insert(symbol.clone()) {
            diagnostics.push(format!("duplicate nominal-domain symbol `{symbol}`"));
        }
    }
}

fn validate_domain_scopes(domain: &NominalDomainPolicy, diagnostics: &mut Vec<String>) {
    if domain.core_scopes.is_empty() {
        diagnostics.push(format!("domain `{}` core_scopes must not be empty", domain.domain_id));
    }
    validate_non_empty_unique_values(
        &format!("domain `{}` core_scopes", domain.domain_id),
        &domain.core_scopes,
        diagnostics,
    );
    validate_non_empty_unique_values(
        &format!("domain `{}` compatibility_scopes", domain.domain_id),
        &domain.compatibility_scopes,
        diagnostics,
    );
    if domain.compatibility_required && domain.compatibility_scopes.is_empty() {
        diagnostics.push(format!("domain `{}` requires at least one compatibility scope", domain.domain_id));
    }
    for core_scope in &domain.core_scopes {
        if domain
            .compatibility_scopes
            .iter()
            .any(|compatibility_scope| normalized_path(core_scope) == normalized_path(compatibility_scope))
        {
            diagnostics
                .push(format!("domain `{}` repeats scope `{core_scope}` as core and compatibility", domain.domain_id));
        }
    }
}

fn validate_constructor_policy(domain: &NominalDomainPolicy, diagnostics: &mut Vec<String>) {
    let valid = match domain.wrapper_kind {
        WrapperKind::RefinedNewtype => domain.constructor_policy == ConstructorPolicy::PrivateFallible,
        WrapperKind::MarkerGeneric => domain.constructor_policy == ConstructorPolicy::MarkerOnly,
        WrapperKind::OpaqueNewtype | WrapperKind::UnitNewtype => {
            domain.constructor_policy != ConstructorPolicy::MarkerOnly
        }
    };
    if !valid {
        diagnostics.push(format!("domain `{}` constructor policy does not match wrapper kind", domain.domain_id));
    }
}

fn validate_non_empty_unique_values(label: &str, values: &[String], diagnostics: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            diagnostics.push(format!("{label} must not contain empty text"));
        } else if !seen.insert(value.as_str()) {
            diagnostics.push(format!("{label} repeats `{value}`"));
        }
    }
}

#[must_use]
pub fn classify_descriptor(catalog: &NominalDomainCatalog, descriptor: DomainDescriptor<'_>) -> DomainDisposition {
    if let Some(domain) = exact_domain(catalog, descriptor) {
        return classify_exact_scope(domain, descriptor.source_path);
    }
    if let Some(suffix) = heuristic_suffix(catalog, descriptor.symbol) {
        return DomainDisposition::HeuristicCandidate {
            suffix: suffix.to_owned(),
        };
    }
    DomainDisposition::Unclassified
}

fn exact_domain<'a>(
    catalog: &'a NominalDomainCatalog,
    descriptor: DomainDescriptor<'_>,
) -> Option<&'a NominalDomainPolicy> {
    catalog.domains.iter().find(|domain| {
        domain.raw_representation == descriptor.raw_representation
            && domain.symbols.iter().any(|symbol| symbol == descriptor.symbol)
    })
}

fn classify_exact_scope(domain: &NominalDomainPolicy, source_path: &str) -> DomainDisposition {
    if source_matches_any_scope(source_path, &domain.compatibility_scopes) {
        return DomainDisposition::Compatibility {
            domain_id: domain.domain_id.clone(),
        };
    }
    if source_matches_any_scope(source_path, &domain.core_scopes) {
        return DomainDisposition::ExactCore {
            domain_id: domain.domain_id.clone(),
        };
    }
    DomainDisposition::ExactOutsideScope {
        domain_id: domain.domain_id.clone(),
    }
}

fn heuristic_suffix<'a>(catalog: &'a NominalDomainCatalog, symbol: &str) -> Option<&'a str> {
    let terminal = symbol.rsplit("::").next().unwrap_or(symbol);
    catalog
        .heuristic_suffixes
        .iter()
        .find(|suffix| terminal.ends_with(suffix.as_str()))
        .map(String::as_str)
}

fn source_matches_any_scope(source_path: &str, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| source_matches_scope(source_path, scope))
}

fn source_matches_scope(source_path: &str, scope: &str) -> bool {
    let source_path = normalized_path(source_path);
    let scope = normalized_path(scope);
    if source_path == scope {
        return true;
    }
    source_path.ends_with(&format!("/{scope}"))
}

fn normalized_path(path: &str) -> String {
    path.trim_matches('"').replace('\\', "/").trim_start_matches("./").to_owned()
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn finding_sort_key(finding: &InventoryFinding) -> (&str, usize, &str, &str) {
    (&finding.source_path, finding.line, &finding.lint_name, &finding.symbol)
}

#[must_use]
pub fn validate_inventory_receipt(receipt: &InventoryReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != NOMINAL_DOMAIN_INVENTORY_SCHEMA_VERSION {
        diagnostics.push(format!("unsupported inventory schema `{}`", receipt.schema_version));
    }
    if receipt.mode != EnforcementMode::Inventory {
        diagnostics.push("pilot receipt mode must be `inventory`".to_owned());
    }
    if receipt.pilot_id.trim().is_empty() {
        diagnostics.push("pilot_id must not be empty".to_owned());
    }
    if !is_blake3_hex(&receipt.source.blake3) {
        diagnostics.push("source.blake3 must be lowercase BLAKE3 hex".to_owned());
    }
    if !is_blake3_hex(&receipt.policy_blake3) {
        diagnostics.push("policy_blake3 must be lowercase BLAKE3 hex".to_owned());
    }
    if receipt.findings.windows(2).any(|pair| finding_sort_key(&pair[0]) > finding_sort_key(&pair[1])) {
        diagnostics.push("findings must use deterministic source, line, lint, and symbol order".to_owned());
    }
    for finding in &receipt.findings {
        if finding.line == 0 {
            diagnostics.push(format!("finding `{}` must have a one-based line", finding.symbol));
        }
        match finding.classification {
            InventoryClassification::ExactCore if finding.domain_id.is_none() => {
                diagnostics.push(format!("exact finding `{}` must name a domain_id", finding.symbol));
            }
            InventoryClassification::HeuristicCandidate if finding.domain_id.is_some() => {
                diagnostics.push(format!("heuristic finding `{}` must not claim a domain_id", finding.symbol));
            }
            InventoryClassification::ExactCore | InventoryClassification::HeuristicCandidate => {}
        }
    }
    let exact_findings = receipt
        .findings
        .iter()
        .filter(|finding| finding.classification == InventoryClassification::ExactCore)
        .count();
    let heuristic_candidates = receipt
        .findings
        .iter()
        .filter(|finding| finding.classification == InventoryClassification::HeuristicCandidate)
        .count();
    if receipt.summary.exact_findings != exact_findings {
        diagnostics.push("summary.exact_findings does not match findings".to_owned());
    }
    if receipt.summary.heuristic_candidates != heuristic_candidates {
        diagnostics.push("summary.heuristic_candidates does not match findings".to_owned());
    }
    if receipt.review.true_positives.saturating_add(receipt.review.false_positives) != receipt.findings.len() {
        diagnostics.push("review counts do not cover every finding".to_owned());
    }
    if receipt.non_claims.is_empty() || receipt.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push("non_claims must contain non-empty text".to_owned());
    }
    diagnostics
}

#[must_use]
pub fn wrapper_contract_violations(domain: &NominalDomainPolicy, facts: WrapperFacts) -> Vec<WrapperViolation> {
    let mut violations = Vec::new();
    let refined = domain.wrapper_kind == WrapperKind::RefinedNewtype;
    if refined && facts.inner_field_public {
        violations.push(WrapperViolation::PublicInnerField);
    }
    if refined && facts.unchecked_from_raw {
        violations.push(WrapperViolation::UncheckedPrimitiveConversion);
    }
    if domain.constructor_policy == ConstructorPolicy::PrivateFallible && !facts.has_fallible_constructor {
        violations.push(WrapperViolation::MissingFallibleConstructor);
    }
    if facts.boundary_erasing_deref {
        violations.push(WrapperViolation::BoundaryErasingDeref);
    }
    violations.sort();
    violations.dedup();
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANTLE_PLAN_PATH: &str = "crates/crunch-build/src/dynamic_plan.rs";
    const WIRE_PATH: &str = "crates/crunch-build/src/wire.rs";

    fn unit_domain() -> NominalDomainPolicy {
        NominalDomainPolicy {
            domain_id: "fixture.unit-id".to_owned(),
            symbols: vec!["UnitId".to_owned(), "DynamicUnit::id".to_owned()],
            raw_representation: RawRepresentation::String,
            core_scopes: vec![MANTLE_PLAN_PATH.to_owned()],
            compatibility_scopes: vec![WIRE_PATH.to_owned()],
            compatibility_required: true,
            wrapper_kind: WrapperKind::RefinedNewtype,
            constructor_policy: ConstructorPolicy::PrivateFallible,
        }
    }

    fn catalog() -> NominalDomainCatalog {
        NominalDomainCatalog {
            schema_version: NOMINAL_DOMAIN_SCHEMA_VERSION.to_owned(),
            default_mode: EnforcementMode::Inventory,
            heuristic_suffixes: vec!["Id".to_owned(), "Digest".to_owned()],
            domains: vec![unit_domain()],
            non_claims: vec!["Source-shape policy only.".to_owned()],
        }
    }

    fn descriptor(symbol: &'static str, source_path: &'static str) -> DomainDescriptor<'static> {
        DomainDescriptor {
            symbol,
            raw_representation: RawRepresentation::String,
            source_path,
            kind: DescriptorKind::TypeAlias,
        }
    }

    #[test]
    fn embedded_catalog_is_valid_and_inventory_only() {
        let catalog = load_nominal_domain_catalog().expect("embedded catalog must pass");

        assert_eq!(catalog.schema_version, NOMINAL_DOMAIN_SCHEMA_VERSION);
        assert_eq!(catalog.default_mode, EnforcementMode::Inventory);
        assert!(!catalog.domains.is_empty());
        assert!(!catalog.heuristic_suffixes.is_empty());
        assert!(nominal_domain_catalog_diagnostics(&catalog).is_empty());
    }

    #[test]
    fn exact_core_compatibility_and_outside_scopes_are_distinct() {
        let catalog = catalog();

        assert_eq!(
            classify_descriptor(&catalog, descriptor("UnitId", MANTLE_PLAN_PATH)),
            DomainDisposition::ExactCore {
                domain_id: "fixture.unit-id".to_owned()
            }
        );
        assert_eq!(classify_descriptor(&catalog, descriptor("UnitId", WIRE_PATH)), DomainDisposition::Compatibility {
            domain_id: "fixture.unit-id".to_owned()
        });
        assert_eq!(
            classify_descriptor(&catalog, descriptor("UnitId", "src/other.rs")),
            DomainDisposition::ExactOutsideScope {
                domain_id: "fixture.unit-id".to_owned()
            }
        );
    }

    #[test]
    fn heuristic_candidates_never_replace_exact_scope_classification() {
        let catalog = catalog();

        assert_eq!(
            classify_descriptor(&catalog, descriptor("SessionId", "src/session.rs")),
            DomainDisposition::HeuristicCandidate {
                suffix: "Id".to_owned()
            }
        );
        assert_eq!(
            classify_descriptor(&catalog, descriptor("message", "src/session.rs")),
            DomainDisposition::Unclassified
        );
        assert_eq!(
            classify_descriptor(&catalog, descriptor("UnitId", "src/session.rs")),
            DomainDisposition::ExactOutsideScope {
                domain_id: "fixture.unit-id".to_owned()
            }
        );
    }

    #[test]
    fn malformed_catalog_fails_closed_with_deterministic_diagnostics() {
        let mut catalog = catalog();
        catalog.schema_version = "future".to_owned();
        catalog.domains.push(unit_domain());
        catalog.domains[0].compatibility_scopes = vec![MANTLE_PLAN_PATH.to_owned()];
        catalog.domains[0].constructor_policy = ConstructorPolicy::PrivateInfallible;

        let diagnostics = nominal_domain_catalog_diagnostics(&catalog);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("schema_version")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate domain_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate nominal-domain symbol")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("core and compatibility")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("constructor policy")));
        assert!(diagnostics.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn refined_wrapper_contract_accepts_checked_private_construction() {
        let domain = unit_domain();
        let facts = WrapperFacts {
            inner_field_public: false,
            unchecked_from_raw: false,
            has_fallible_constructor: true,
            boundary_erasing_deref: false,
        };

        assert!(wrapper_contract_violations(&domain, facts).is_empty());
    }

    #[test]
    fn refined_wrapper_contract_reports_every_supported_bypass() {
        let domain = unit_domain();
        let facts = WrapperFacts {
            inner_field_public: true,
            unchecked_from_raw: true,
            has_fallible_constructor: false,
            boundary_erasing_deref: true,
        };

        assert_eq!(wrapper_contract_violations(&domain, facts), vec![
            WrapperViolation::PublicInnerField,
            WrapperViolation::UncheckedPrimitiveConversion,
            WrapperViolation::MissingFallibleConstructor,
            WrapperViolation::BoundaryErasingDeref,
        ]);
    }

    #[test]
    fn frozen_pilot_receipts_are_valid_and_reviewed() {
        let mantle: InventoryReceipt = serde_json::from_str(include_str!(
            "../../../cairn/changes/enforce-nominal-domain-types/evidence/pilots/mantle-dynamic-plan.json"
        ))
        .expect("parse Mantle pilot receipt");
        let valence: InventoryReceipt = serde_json::from_str(include_str!(
            "../../../cairn/changes/enforce-nominal-domain-types/evidence/pilots/valence-shared-identity.json"
        ))
        .expect("parse Valence pilot receipt");

        let policy_blake3 = blake3::hash(NOMINAL_DOMAIN_CATALOG_JSON.as_bytes()).to_hex().to_string();
        for receipt in [&mantle, &valence] {
            assert!(validate_inventory_receipt(receipt).is_empty());
            assert!(receipt.review.reviewed);
            assert_eq!(receipt.policy_blake3, policy_blake3);
        }
        assert!(!mantle.findings.is_empty());
        assert!(valence.findings.is_empty());
        assert!(!valence.review.coverage_gaps.is_empty());
    }

    #[test]
    fn inventory_receipt_validation_rejects_stale_counts_hashes_and_order() {
        let mut receipt: InventoryReceipt = serde_json::from_str(include_str!(
            "../../../cairn/changes/enforce-nominal-domain-types/evidence/pilots/mantle-dynamic-plan.json"
        ))
        .expect("parse Mantle pilot receipt");
        receipt.policy_blake3 = "stale".to_owned();
        receipt.summary.exact_findings = 0;
        receipt.findings.swap(0, 1);

        let diagnostics = validate_inventory_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("policy_blake3")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("exact_findings")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("deterministic")));
    }

    #[test]
    fn malformed_json_and_unknown_enum_values_are_rejected() {
        let malformed = parse_nominal_domain_catalog("{");
        let unknown_representation = NOMINAL_DOMAIN_CATALOG_JSON.replace("\"string\"", "\"uuid\"");

        assert!(malformed.is_err());
        assert!(parse_nominal_domain_catalog(&unknown_representation).is_err());
    }
}
