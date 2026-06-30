use std::path::Path;

const CANONICAL_CHECKED_PATH: &str =
    "Octet is the evidence-backed checked path for agent-written and human-written Rust systems code";
const TRUST_BOUNDARY_DOC: &str = "docs/trust-boundary-evidence.md";
const CONSUMER_GUIDE_DOC: &str = "docs/consumer-guide.md";
const DOWNSTREAM_ROLLOUT_DOC: &str = "docs/downstream-rollout.md";
const SAFETY_CRITICAL_DOC: &str = "docs/safety-critical.md";
const RELEASE_CANDIDATE_DOC: &str = "docs/release-candidate.md";
const README_DOC: &str = "README.md";

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
fn docs_do_not_contain_unscoped_overclaim_phrases() {
    for path in [
        README_DOC,
        CONSUMER_GUIDE_DOC,
        TRUST_BOUNDARY_DOC,
        DOWNSTREAM_ROLLOUT_DOC,
    ] {
        let text = read_repo_text(path);
        let diagnostics = overclaim_diagnostics(&text);
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

fn read_repo_text(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let full_path = root.join(path);
    std::fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("read {}: {error}", full_path.display()))
}
