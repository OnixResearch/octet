use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::contract::validation::*;
use super::contract::*;
use super::evaluation::clause::*;
use super::evaluation::decision::*;
use super::evaluation::verification::*;
use super::evidence::*;
use super::*;

const SPEC_PATH: &str = "spec.md";
const SOURCE_PATH: &str = "src/lib.rs";
const PROOF_DECK: &str = "proof-cards.json";
const PROOF_CARD_ID: &str = "clause.add";

fn fixture_root() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write_file(root: &Path, locator: &str, bytes: &[u8]) -> SourceBinding {
    let path = root.join(locator);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&path, bytes).expect("write fixture file");
    SourceBinding {
        locator: locator.to_owned(),
        blake3: hash_bytes(bytes),
    }
}

fn contract(source_binding: SourceBinding, spec_hash: String, disposition: &str) -> Definition {
    Definition {
        schema_version: CONTRACT_SCHEMA_VERSION.to_owned(),
        contract_id: "octet.requirements.test".to_owned(),
        reviewed_specification: ReviewedSpecification {
            path: SPEC_PATH.to_owned(),
            blake3: spec_hash,
        },
        source_root: None,
        clauses: vec![RequirementClause {
            clause_id: "clause.add".to_owned(),
            statement: "addition is checked".to_owned(),
            required: true,
            disposition: disposition.to_owned(),
            checked_scope: vec!["formal-proof:fixture".to_owned()],
            expected_evidence_tier: DISPOSITION_FORMAL_PROOF.to_owned(),
            allowed_evidence_ids: vec![PROOF_CARD_ID.to_owned()],
            allowed_source_requirement_ids: vec!["clause.add".to_owned()],
            required_evidence_classes: vec!["proof_card".to_owned()],
            required_negative_controls: Vec::new(),
            required_bindings: RequirementBindings::default(),
            source_bindings: vec![source_binding],
            dependency_bindings: Vec::new(),
            assumption_id: None,
            exclusion_reason: None,
            required_provider_id: None,
        }],
        assumptions: Vec::new(),
        exclusions: Vec::new(),
        non_claims: Vec::new(),
    }
}

fn proof_deck(root: &Path, status: &str) -> ProofDeck {
    let artifact = "proof.receipt";
    let artifact_bytes = b"{\"status\":\"passed\"}";
    let artifact_hash = hash_bytes(artifact_bytes);
    fs::write(root.join(artifact), artifact_bytes).expect("write receipt");
    let deck = serde_json::json!({
        "schema_version": PROOF_CARDS_SCHEMA_VERSION,
        "cards": [{
            "proof_or_property_id": PROOF_CARD_ID,
            "source_requirement_or_policy_id": "clause.add",
            "checked_rust_source_scope": "formal-proof:fixture",
            "evidence_tier": "formal_proof",
            "status": status,
            "non_claims": ["not whole-program correctness"],
            "diagnostics": [],
            "artifact_blake3_identities": [{"artifact": artifact, "blake3": artifact_hash}],
        }],
    });
    fs::write(root.join(PROOF_DECK), serde_json::to_vec_pretty(&deck).expect("serialize deck")).expect("write deck");
    ProofDeck {
        evidence_id: PROOF_CARD_ID.to_owned(),
        deck: PROOF_DECK.to_owned(),
        artifact_root: ".".to_owned(),
    }
}

fn evidence(contract_blake3: String, source_binding: SourceBinding) -> Bundle {
    Bundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
        contract_id: "octet.requirements.test".to_owned(),
        contract_blake3: Some(contract_blake3),
        source_snapshot: vec![source_binding],
        proof_decks: Vec::new(),
        empirical_checks: Vec::new(),
        assumption_records: Vec::new(),
        non_claims: Vec::new(),
    }
}

mod evaluation;
mod freshness;

pub(crate) mod hooks {
    pub(crate) use super::super::contract::*;
    pub(crate) use super::super::evaluation::decision::Outcome;
    pub(crate) use super::super::evidence::*;

    pub(crate) const CHECK_STATUS_PASSED: &str = super::super::CHECK_STATUS_PASSED;
    pub(crate) const CONTRACT_SCHEMA_VERSION: &str = super::super::CONTRACT_SCHEMA_VERSION;
    pub(crate) const DISPOSITION_ACCEPTED_ASSUMPTION: &str = super::super::DISPOSITION_ACCEPTED_ASSUMPTION;
    pub(crate) const DISPOSITION_EMPIRICAL_CHECK: &str = super::super::DISPOSITION_EMPIRICAL_CHECK;
    pub(crate) const DISPOSITION_EXCLUDED: &str = super::super::DISPOSITION_EXCLUDED;
    pub(crate) const DISPOSITION_FORMAL_PROOF: &str = super::super::DISPOSITION_FORMAL_PROOF;
    pub(crate) const EVIDENCE_SCHEMA_VERSION: &str = super::super::EVIDENCE_SCHEMA_VERSION;
    pub(crate) const NEGATIVE_CHECK_SCHEMA_VERSION_V2: &str = super::super::NEGATIVE_CHECK_SCHEMA_VERSION_V2;
    pub(crate) const NEGATIVE_STATUS_DEMONSTRATED: &str = super::super::NEGATIVE_STATUS_DEMONSTRATED;
    pub(crate) const PROOF_CARDS_SCHEMA_VERSION: &str = super::super::PROOF_CARDS_SCHEMA_VERSION;
    pub(crate) const STATUS_CURRENT: &str = super::super::STATUS_CURRENT;
    pub(crate) const STATUS_FAILED: &str = super::super::STATUS_FAILED;
    pub(crate) const STATUS_MISSING: &str = super::super::STATUS_MISSING;
    pub(crate) const STATUS_PASSED: &str = super::super::STATUS_PASSED;
    pub(crate) const STATUS_STALE_BINDING: &str = super::super::STATUS_STALE_BINDING;

    pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
        super::super::hash_bytes(bytes)
    }

    pub(crate) fn digest(decision: &Outcome) -> Result<String, String> {
        super::super::evaluation::decision::digest(decision)
    }

    pub(crate) fn evaluate_contract(
        contract: &Definition,
        evidence: &Bundle,
        contract_bytes: &[u8],
        evidence_bytes: &[u8],
        source_root: &std::path::Path,
    ) -> Result<Outcome, String> {
        super::super::evaluation::decision::evaluate_contract(
            contract,
            evidence,
            contract_bytes,
            evidence_bytes,
            source_root,
        )
    }

    pub(crate) fn compare_decisions(recorded: &Outcome, recomputed: &Outcome, diagnostics: &mut Vec<String>) {
        super::super::evaluation::verification::compare_decisions(recorded, recomputed, diagnostics);
    }

    pub(crate) fn evaluate_freshness(
        contract: &Definition,
        index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
        contract_bytes: &[u8],
        source_root: Option<&std::path::Path>,
    ) -> super::super::freshness::InventoryReport {
        super::super::freshness::evaluate(contract, index, contract_bytes, source_root)
    }

    pub(crate) fn record_satisfies_bindings(
        record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
        clause: &RequirementClause,
    ) -> bool {
        super::super::related::record_satisfies_bindings(record, clause)
    }

    pub(crate) fn record_has_failure(record: &valence_core::evidence_codebase_index::EvidenceIndexRecord) -> bool {
        super::super::related::record_has_failure(record)
    }
}
