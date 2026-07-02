use std::path::Path;

const CANONICAL_CHECKED_PATH: &str =
    "Octet is the evidence-backed checked path for agent-written and human-written Rust systems code";
const TRUST_BOUNDARY_DOC: &str = "docs/trust-boundary-evidence.md";
const ADOPTION_STORY_DOC: &str = "docs/adoption-story.md";
const CONSUMER_GUIDE_DOC: &str = "docs/consumer-guide.md";
const DOWNSTREAM_ROLLOUT_DOC: &str = "docs/downstream-rollout.md";
const LINT_REFERENCE_DOC: &str = "docs/lint-reference.md";
const SAFETY_CRITICAL_DOC: &str = "docs/safety-critical.md";
const RELEASE_CANDIDATE_DOC: &str = "docs/release-candidate.md";
const README_DOC: &str = "README.md";
const VERUS_README_DOC: &str = "verus/README.md";
const CHECKED_CONTRACT_ANCHOR: &str = "docs/trust-boundary-evidence.md#checked-contract-ladder";
const VERUS_PROOF_PATH_ANCHOR: &str = "docs/trust-boundary-evidence.md#verus-proof-path-lane";

#[test]
fn checked_path_positioning_is_visible_in_primary_docs() {
    let readme = read_repo_text(README_DOC);
    let consumer = read_repo_text(CONSUMER_GUIDE_DOC);
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);

    assert_required_terms(&readme, &[
        CANONICAL_CHECKED_PATH,
        "Rust source flows through `cargo octet check` and",
        "reviewable evidence artifacts",
        "consumer CI or release gates",
    ]);
    assert_required_terms(&consumer, &[
        CANONICAL_CHECKED_PATH,
        "Rust source flows through Octet checks",
        "policy/config",
        "reviewable evidence artifacts",
        "consumer CI or release",
    ]);
    assert_required_terms(&trust_boundary, &[
        CANONICAL_CHECKED_PATH,
        "Rust source",
        "cargo octet check / octet-standards checks",
        "checked-in policy and config",
        "evidence artifacts",
        "consumer CI or release gate",
    ]);
}

#[test]
fn evidence_stack_index_covers_all_required_themes() {
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);

    assert_required_terms(&trust_boundary, &[
        "checked-path framing",
        "evidence stack",
        "evidence tiers",
        "provenance chain",
        "scope-boundary",
        "controlled-deployment",
    ]);
    assert_required_terms(&trust_boundary, &[
        "discipline",
        "static edge",
        "policy layer",
        "evidence chain",
        "Tiger Style guidance",
        "Dylint lint domains",
        "Nickel-authored policy registry",
        "status.json",
    ]);
}

#[test]
fn controlled_deployment_claims_and_checklist_are_visible() {
    let consumer = read_repo_text(CONSUMER_GUIDE_DOC);
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);
    let downstream = read_repo_text(DOWNSTREAM_ROLLOUT_DOC);

    assert_required_terms(&trust_boundary, &[
        "local or consumer-CI execution",
        "no required hosted service",
        "pin-capable runner path",
        "source-visible toolchain",
        "local artifact directory",
        "Consumers still own what their CI provider",
        "Pin the Octet revision",
        "Check in consumer-owned `dylint.toml`",
        "deterministic artifact directory",
        "pre-commit gate",
    ]);
    assert_required_terms(&consumer, &[
        "local or consumer-CI execution",
        "no required hosted service",
        "pin-capable runner path",
        "source-visible toolchain",
        "local artifact directory",
        "Consumers still own",
    ]);
    assert_required_terms(&downstream, &[
        "trust-boundary-evidence.md#controlled-deployment-posture",
        "agent-written and human-written Rust",
        "local evidence artifacts",
    ]);
}

#[test]
fn scope_boundaries_pair_unsupported_claims_with_supported_capabilities() {
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);
    let safety_critical = read_repo_text(SAFETY_CRITICAL_DOC);
    let release_candidate = read_repo_text(RELEASE_CANDIDATE_DOC);

    assert_required_terms(&trust_boundary, &[
        "does not prove whole-program correctness",
        "does not validate business or domain requirements",
        "does not make suppressions correct merely because they are present",
        "static rejection of known-dangerous source shapes",
        "policy validation",
        "reviewable suppressions",
        "SARIF, JSONL findings, transcripts, and fingerprints",
    ]);
    assert_required_terms(&safety_critical, &["trust-boundary-evidence.md#what-octet-does-not-claim"]);
    assert_required_terms(&release_candidate, &[
        "docs/trust-boundary-evidence.md",
        "controlled-deployment posture",
        "scope-boundary caveats",
    ]);
}

#[test]
fn evidence_tier_and_provenance_docs_name_machine_readable_fields() {
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);
    let consumer = read_repo_text(CONSUMER_GUIDE_DOC);

    assert_required_terms(&trust_boundary, &[
        "`static_lint`",
        "`policy_check`",
        "`ui_fixture`",
        "`autofix_verified`",
        "`transcript_assertion`",
        "`formal_proof`",
        "`manual_review_required`",
        "source ID",
        "checker ID",
        "BLAKE3 content hash",
        "checked scope",
    ]);
    assert_required_terms(&consumer, &["`evidence_tier`"]);
}

#[test]
fn verus_proof_path_explainer_names_required_stages_and_commands() {
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);
    let readme = read_repo_text(README_DOC);
    let verus_readme = read_repo_text(VERUS_README_DOC);

    assert_required_terms(&trust_boundary, &[
        "Verus proof-path lane",
        "requirement or policy ID",
        "Verus proof rail",
        "checked Rust source scope",
        "proof target",
        "verifier/solver/toolchain identity",
        "structured proof receipt",
        "`formal_proof` provenance",
        "artifact verifier",
        "cargo octet prove --list",
        "cargo octet prove --explain",
        "cargo octet prove",
        "cargo octet artifact verify",
    ]);
    assert_required_terms(&trust_boundary, &[
        "Trellis repository revision",
        "BLAKE3 source artifact identity",
        "BLAKE3 proof artifact identity",
        "bounded log identity when present",
        "receipt identity when present",
        "imported proof bundle",
        "proof card / manual review",
        "optional Octet rerun with pinned verifier identity",
        "standards/proof-imports/trellis.ncl",
        "standards/proof-imports/generated/trellis-proof-imports.json",
        "BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness",
    ]);
    assert_required_terms(&trust_boundary, &[
        "lint findings remain `static_lint`",
        "policy checks remain `policy_check`",
        "transcript assertions remain `transcript_assertion`",
        "imported Trellis bundles remain external or `manual_review_required` until rerun",
        "unresolved human obligations remain `manual_review_required`",
        "claim remains scoped to the declared property",
        "does not claim whole-program correctness",
        "certification",
        "business-domain validation",
        "semantic equivalence",
        "full safety",
        "verified runtime",
    ]);
    assert_required_terms(&readme, &[VERUS_PROOF_PATH_ANCHOR, "requirement or policy ID to Verus proof rail"]);
    assert_required_terms(&verus_readme, &[VERUS_PROOF_PATH_ANCHOR, "requirement or policy ID to Verus proof rail"]);
}

#[test]
fn checked_contract_ladder_docs_inventory_records_and_boundaries() {
    let trust_boundary = read_repo_text(TRUST_BOUNDARY_DOC);
    let readme = read_repo_text(README_DOC);
    let lint_reference = read_repo_text(LINT_REFERENCE_DOC);
    let verus_readme = read_repo_text(VERUS_README_DOC);

    assert_required_terms(&trust_boundary, &[
        "Checked-contract ladder",
        "advisory lint findings",
        "warning gates",
        "deny gates",
        "policy requirements",
        "receipt-backed validation",
        "scoped formal-proof receipts",
        "verified release artifacts",
        "Evidence tier or boundary",
        "operational gating rather than formal proof",
        "`static_lint`",
        "`policy_check`",
        "`formal_proof`",
        "cargo octet check",
        "octet-standards check",
        "cargo octet prove",
        "cargo octet artifact verify",
    ]);
    assert_required_terms(&trust_boundary, &[
        "requirement or policy ID",
        "source scope",
        "invariant or failure shape",
        "evidence tier",
        "command or gate",
        "artifact or receipt path",
        "reviewer action",
        "explicit non-claim",
        "lint diagnostic",
        "standards policy",
        "proof card",
        "provenance record",
        "artifact verifier",
    ]);
    assert_required_terms(&trust_boundary, &[
        "deterministic failure shape",
        "invalid state",
        "branch",
        "missing guard",
        "resource boundary",
        "remediation path",
        "not solver models",
        "not formal counterexamples",
    ]);
    assert_required_terms(&trust_boundary, &[
        "does not introduce Rust `requires` or `ensures` syntax",
        "does not provide complete solver-backed verification for arbitrary Rust",
        "does not add runtime fallback checks for unproved contracts",
        "does not make every ladder stage formal proof",
    ]);
    assert_required_terms(&readme, &[CHECKED_CONTRACT_ANCHOR]);
    assert_required_terms(&lint_reference, &["trust-boundary-evidence.md#checked-contract-ladder"]);
    assert_required_terms(&verus_readme, &["docs/trust-boundary-evidence.md#checked-contract-ladder"]);
}

#[test]
fn adoption_story_docs_define_case_study_methodology_and_boundaries() {
    let adoption = read_repo_text(ADOPTION_STORY_DOC);
    let readme = read_repo_text(README_DOC);
    let consumer = read_repo_text(CONSUMER_GUIDE_DOC);

    assert_required_terms(&adoption, &[
        "Rust-compatible, evidence-backed checked path",
        "cargo octet check",
        "checked-in config or policy",
        "evidence artifacts",
        "CI or release gate",
        "artifact verification before promotion",
    ]);
    assert_required_terms(&adoption, &[
        "Built with Octet case-study format",
        "consumer or archive source",
        "checked scope",
        "command or gate",
        "configuration source",
        "evidence artifact or archive path",
        "outcome summary",
        "scope boundary",
    ]);
    assert_required_terms(&adoption, &[
        "openspec/changes/archive/2026-04-13-aspen-consumer-rollout/",
        "aspen-time",
        "aspen-hlc",
        "aspen-core",
        "evidence/aspen-default-run.txt",
        "evidence/validation-counts.txt",
        "six findings",
        "historical rollout evidence for the archived starter scope",
    ]);
    assert_required_terms(&adoption, &[
        "Methodology before public metrics",
        "command or gate that produced the measurement",
        "input scope and configuration",
        "environment or platform",
        "raw artifact, transcript, or reproducible evidence path",
        "interpretation boundary for the metric",
        "held back until the raw command and artifact path exist",
    ]);
    assert_required_terms(&adoption, &[
        "not a new programming language",
        "not a Rust replacement",
        "does not claim mathematical certainty",
        "whole-program correctness",
        "certification",
        "a verified runtime",
        "language-level contract syntax",
    ]);
    assert_required_terms(&readme, &[ADOPTION_STORY_DOC]);
    assert_required_terms(&consumer, &["adoption-story.md"]);
}

#[test]
fn verus_proof_path_checker_rejects_unscoped_overclaim_text() {
    let bad_text = "The Verus proof path proves whole-program correctness. It provides certification and provides business-domain validation. It proves semantic equivalence and guarantees full safety for Octet. Octet is a verified runtime.";

    let diagnostics = proof_path_overclaim_diagnostics(bad_text);

    assert_eq!(diagnostics, vec![
        "unscoped whole-program-correctness claim".to_owned(),
        "unscoped certification claim".to_owned(),
        "unscoped business-domain-validation claim".to_owned(),
        "unscoped semantic-equivalence claim".to_owned(),
        "unscoped full-safety claim".to_owned(),
        "unscoped verified-runtime claim".to_owned(),
    ]);
}

#[test]
fn docs_do_not_contain_unscoped_overclaim_phrases() {
    for path in [
        README_DOC,
        CONSUMER_GUIDE_DOC,
        TRUST_BOUNDARY_DOC,
        ADOPTION_STORY_DOC,
        DOWNSTREAM_ROLLOUT_DOC,
        LINT_REFERENCE_DOC,
        VERUS_README_DOC,
    ] {
        let text = read_repo_text(path);
        let mut diagnostics = overclaim_diagnostics(&text);
        diagnostics.extend(proof_path_overclaim_diagnostics(&text));
        diagnostics.extend(checked_contract_overclaim_diagnostics(&text));
        diagnostics.extend(adoption_story_overclaim_diagnostics(&text));
        assert!(diagnostics.is_empty(), "{path} overclaim diagnostics: {diagnostics:?}");
    }
}

#[test]
fn overclaim_checker_rejects_bad_positioning_text() {
    let bad_text = "Octet is a verified runtime. Octet proves program correctness. Source never leaves.";

    let diagnostics = overclaim_diagnostics(bad_text);

    assert_eq!(diagnostics, vec![
        "unscoped verified-runtime claim".to_owned(),
        "unscoped correctness-proof claim".to_owned(),
        "absolute source-locality claim".to_owned(),
    ]);
}

#[test]
fn checked_contract_overclaim_checker_rejects_bad_ladder_text() {
    let bad_text = "Octet has built-in `requires` syntax. Octet provides complete solver-backed verification for arbitrary Rust. Octet emits runtime fallback checks for unproved contracts. Every ladder stage is formal proof.";

    let diagnostics = checked_contract_overclaim_diagnostics(bad_text);

    assert_eq!(diagnostics, vec![
        "unsupported language-level contract syntax claim".to_owned(),
        "unsupported solver-completeness claim".to_owned(),
        "unsupported runtime-fallback claim".to_owned(),
        "unsupported all-stages-formal-proof claim".to_owned(),
    ]);
}

#[test]
fn adoption_story_overclaim_checker_rejects_bad_positioning_text() {
    let bad_text = "Octet is a new programming language. Octet replaces Rust. Octet guarantees mathematical certainty. Octet proves whole-program correctness. Octet certifies systems. Octet has a verified runtime.";

    let diagnostics = adoption_story_overclaim_diagnostics(bad_text);

    assert_eq!(diagnostics, vec![
        "unsupported new-language claim".to_owned(),
        "unsupported rust-replacement claim".to_owned(),
        "unsupported mathematical-certainty claim".to_owned(),
        "unsupported whole-program-correctness claim".to_owned(),
        "unsupported certification claim".to_owned(),
        "unsupported verified-runtime claim".to_owned(),
    ]);
}

fn assert_required_terms(text: &str, terms: &[&str]) {
    let normalized_text = normalize_markdown_text(text);
    let missing = terms
        .iter()
        .copied()
        .filter(|term| {
            let normalized_term = normalize_markdown_text(term);
            !text.contains(term) && !normalized_text.contains(&normalized_term)
        })
        .collect::<Vec<_>>();

    assert!(missing.is_empty(), "missing required terms: {missing:?}");
}

fn normalize_markdown_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn overclaim_diagnostics(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut diagnostics = Vec::new();

    if lower.contains("octet is a verified runtime") || lower.contains("octet is the verified runtime") {
        diagnostics.push("unscoped verified-runtime claim".to_owned());
    }
    if lower.contains("octet proves program correctness") || lower.contains("proves program correctness") {
        diagnostics.push("unscoped correctness-proof claim".to_owned());
    }
    if lower.contains("octet guarantees safety") || lower.contains("guarantees safety") {
        diagnostics.push("unscoped safety-guarantee claim".to_owned());
    }
    if lower.contains("octet validates business requirements") {
        diagnostics.push("unscoped business-validation claim".to_owned());
    }
    if lower.contains("proof stack") {
        diagnostics.push("unscoped proof-stack claim".to_owned());
    }
    if lower.contains("source never leaves") {
        diagnostics.push("absolute source-locality claim".to_owned());
    }

    diagnostics
}

fn proof_path_overclaim_diagnostics(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut diagnostics = Vec::new();

    push_if_contains(
        &mut diagnostics,
        &lower,
        &["proves whole-program correctness", "proof of whole-program correctness"],
        "unscoped whole-program-correctness claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["provides certification", "claims certification"],
        "unscoped certification claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["provides business-domain validation", "validates business-domain"],
        "unscoped business-domain-validation claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["proves semantic equivalence", "guarantees semantic equivalence"],
        "unscoped semantic-equivalence claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["proves full safety", "guarantees full safety"],
        "unscoped full-safety claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["octet is a verified runtime", "octet is the verified runtime"],
        "unscoped verified-runtime claim",
    );

    diagnostics
}

fn checked_contract_overclaim_diagnostics(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut diagnostics = Vec::new();

    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "built-in `requires` syntax",
            "built-in requires syntax",
            "built-in `ensures` syntax",
        ],
        "unsupported language-level contract syntax claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "octet provides complete solver-backed verification",
            "octet provides solver-complete verification",
        ],
        "unsupported solver-completeness claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "octet emits runtime fallback checks",
            "octet adds runtime fallback checks",
        ],
        "unsupported runtime-fallback claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "every ladder stage is formal proof",
            "all ladder stages are formal proof",
        ],
        "unsupported all-stages-formal-proof claim",
    );

    diagnostics
}

fn adoption_story_overclaim_diagnostics(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut diagnostics = Vec::new();

    push_if_contains(
        &mut diagnostics,
        &lower,
        &["octet is a new programming language", "octet is a new language"],
        "unsupported new-language claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["octet replaces rust", "octet is a rust replacement"],
        "unsupported rust-replacement claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "octet guarantees mathematical certainty",
            "octet provides mathematical certainty",
        ],
        "unsupported mathematical-certainty claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &[
            "octet proves whole-program correctness",
            "octet proves all code correct",
        ],
        "unsupported whole-program-correctness claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["octet certifies systems", "octet provides certification"],
        "unsupported certification claim",
    );
    push_if_contains(
        &mut diagnostics,
        &lower,
        &["octet has a verified runtime", "octet ships a verified runtime"],
        "unsupported verified-runtime claim",
    );

    diagnostics
}

fn push_if_contains(diagnostics: &mut Vec<String>, text: &str, needles: &[&str], diagnostic: &str) {
    if needles.iter().any(|needle| text.contains(needle)) {
        diagnostics.push(diagnostic.to_owned());
    }
}

fn read_repo_text(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let full_path = root.join(path);
    std::fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("read {}: {error}", full_path.display()))
}
