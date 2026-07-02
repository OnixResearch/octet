use octet_standards::model::EnforcementMode;
use octet_standards::model::FamilySelection;
use octet_standards::model::PolicyEntry;
use octet_standards::model::PolicyRegistry;
use octet_standards::model::ProofRail;
use octet_standards::proof_artifact::proof_rail_is_relevant;
use octet_standards::source::SourceFile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectedRailIndex {
    pub(crate) policy_index: usize,
    pub(crate) rail_index: usize,
}

pub(crate) fn selected_rail_indices(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<SelectedRailIndex> {
    registry
        .policies
        .iter()
        .enumerate()
        .filter(|(_, policy)| selection.includes(&policy.pillar))
        .filter(|(_, policy)| policy_is_verified(policy))
        .flat_map(|(policy_index, policy)| matching_rail_indices(policy_index, policy, sources))
        .collect()
}

pub(crate) fn policy_is_verified(policy: &PolicyEntry) -> bool {
    policy.current_enforcement_mode == EnforcementMode::Verified
        || policy.target_enforcement_mode == EnforcementMode::Verified
}

fn matching_rail_indices(policy_index: usize, policy: &PolicyEntry, sources: &[SourceFile]) -> Vec<SelectedRailIndex> {
    policy
        .proof_rails
        .iter()
        .enumerate()
        .filter(|(_, rail)| rail_matches_sources(rail, sources))
        .map(|(rail_index, _)| SelectedRailIndex {
            policy_index,
            rail_index,
        })
        .collect()
}

fn rail_matches_sources(rail: &ProofRail, sources: &[SourceFile]) -> bool {
    proof_rail_is_relevant(rail, sources)
}

#[cfg(test)]
mod tests {
    use octet_standards::model::Classification;
    use octet_standards::model::EnforcementLevel;
    use octet_standards::model::EvidenceCommand;
    use octet_standards::model::FixtureCoverage;
    use octet_standards::model::FixtureStatus;
    use octet_standards::model::Severity;
    use octet_standards::model::SuppressionRequirements;

    use super::*;

    const POLICY_ID: &str = "octet.prove_core.fixture";
    const MATCHING_PILLAR: &str = "verification";
    const OTHER_PILLAR: &str = "operational";
    const SOURCE_PATH: &str = "src/verified/addition.rs";
    const MISSING_SOURCE_PATH: &str = "src/verified/missing.rs";
    const PROOF_PATH: &str = "proofs/addition.rs";

    #[test]
    fn selected_rail_core_accepts_verified_matching_source() {
        let registry =
            registry_with(policy_with(MATCHING_PILLAR, EnforcementMode::Lint, EnforcementMode::Verified, vec![
                rail_for(SOURCE_PATH),
            ]));
        let sources = vec![SourceFile::new(SOURCE_PATH, "pub fn checked() {}\n")];
        let selection = FamilySelection::from_families([MATCHING_PILLAR]);

        let selected = selected_rail_indices(&registry, &sources, &selection);

        assert_eq!(selected, vec![SelectedRailIndex {
            policy_index: 0,
            rail_index: 0,
        }]);
    }

    #[test]
    fn selected_rail_core_rejects_unverified_or_unmatched_inputs() {
        let unverified =
            registry_with(policy_with(MATCHING_PILLAR, EnforcementMode::Lint, EnforcementMode::Lint, vec![rail_for(
                SOURCE_PATH,
            )]));
        let sources = vec![SourceFile::new(SOURCE_PATH, "pub fn checked() {}\n")];
        let selection = FamilySelection::from_families([MATCHING_PILLAR]);
        assert!(selected_rail_indices(&unverified, &sources, &selection).is_empty());

        let wrong_family =
            registry_with(policy_with(OTHER_PILLAR, EnforcementMode::Verified, EnforcementMode::Verified, vec![
                rail_for(SOURCE_PATH),
            ]));
        assert!(selected_rail_indices(&wrong_family, &sources, &selection).is_empty());

        let missing_source =
            registry_with(policy_with(MATCHING_PILLAR, EnforcementMode::Verified, EnforcementMode::Verified, vec![
                rail_for(MISSING_SOURCE_PATH),
            ]));
        assert!(selected_rail_indices(&missing_source, &sources, &selection).is_empty());
    }

    fn registry_with(policy: PolicyEntry) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: vec![policy],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn policy_with(
        pillar: &str,
        current_enforcement_mode: EnforcementMode,
        target_enforcement_mode: EnforcementMode,
        proof_rails: Vec<ProofRail>,
    ) -> PolicyEntry {
        PolicyEntry {
            policy_id: POLICY_ID.to_owned(),
            pillar: pillar.to_owned(),
            source_path: "fixture".to_owned(),
            source_heading: "fixture".to_owned(),
            source_anchor: "fixture".to_owned(),
            summary: "fixture".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode,
            target_enforcement_mode,
            escalation_criteria: "fixture".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: EvidenceCommand::default(),
            proof_rails,
            checker_owner: "fixture".to_owned(),
            checker_id: "octet-standards".to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: SuppressionRequirements {
                requires_policy_id: true,
                requires_source_section: true,
                requires_owner: true,
                requires_rationale: true,
                requires_review_trigger: true,
                requires_safer_default: true,
            },
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Covered,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec!["fixture".to_owned()],
            classification_rationale: "fixture".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "fixture".to_owned(),
        }
    }

    fn rail_for(source_path: &str) -> ProofRail {
        ProofRail {
            source_paths: vec![source_path.to_owned()],
            proof_paths: vec![PROOF_PATH.to_owned()],
            receipt_paths: Vec::new(),
            ..ProofRail::default()
        }
    }
}
