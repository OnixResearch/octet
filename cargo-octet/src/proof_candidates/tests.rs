use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_json::json;

use super::*;

const PROPERTY_ID: &str = "octet.verus_proof_rails.addition-bound-production#rail-0";
const POLICY_ID: &str = "octet.verus_proof_rails.addition-bound-production";
const CHECKED_SCOPE: &str = "formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0";
const RECEIPT: &str = "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt";
const ANCHOR: &str =
    "octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0";
const NON_CLAIMS: [&str; 5] = [
    "not whole-program correctness",
    "not certification",
    "not business-domain validation",
    "not semantic equivalence",
    "not full safety",
];

fn candidate_args(
    source: PathBuf,
    proof_cards: Option<PathBuf>,
    proof_refs: Vec<PathBuf>,
    trust_boundary: Option<PathBuf>,
) -> ProveArgs {
    ProveArgs {
        registry: None,
        policy_artifacts_dir: None,
        source: vec![source],
        family: Vec::new(),
        rail: Vec::new(),
        artifact_dir: PathBuf::from("artifacts/octet-proofs"),
        output_format: OutputFormat::Json,
        command_line: "cargo octet prove --candidates".to_owned(),
        list: false,
        explain: false,
        candidates: true,
        proof_cards,
        proof_refs,
        trust_boundary,
        negative_check: None,
    }
}

fn card_json(status: &str) -> serde_json::Value {
    json!({
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "evidence_tier": "formal_proof",
        "evidence_origin": "octet_rerun",
        "status": status,
        "artifact_blake3_identities": [{"artifact": RECEIPT, "blake3": "a".repeat(64)}],
        "non_claims": NON_CLAIMS,
        "diagnostics": [],
    })
}

fn deck_json(status: &str) -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-cards/v1",
        "cards": [card_json(status)],
    })
}

fn write_deck(path: &Path, status: &str) {
    fs::write(path, serde_json::to_string_pretty(&deck_json(status)).expect("deck json")).expect("write deck");
}

fn reference_json(source_path: &str, source_text: &str, line: u32) -> serde_json::Value {
    json!({
        "schema_version": "octet-proof-evidence-ref/v1",
        "reference_id": "unchecked-narrowing-range-proof",
        "consumer_context": format!("allow tigerstyle::unchecked_narrowing at {source_path}:{line}"),
        "consumer_source_path": source_path,
        "consumer_source_blake3": blake3::hash(source_text.as_bytes()).to_hex().to_string(),
        "consumer_source_anchor": ANCHOR,
        "proof_or_property_id": PROPERTY_ID,
        "source_requirement_or_policy_id": POLICY_ID,
        "checked_rust_source_scope": CHECKED_SCOPE,
        "expected_evidence_tier": "formal_proof",
        "required_status": "passed",
        "required_artifacts": [RECEIPT],
        "non_claims": NON_CLAIMS,
    })
}

fn write_reference(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(value).expect("reference json")).expect("write reference");
}

fn trust_doc(prefixes: &[&str]) -> String {
    format!("# Trust boundary\n\n```text {TRUST_BOUNDARY_FENCE}\n{}\n```\n", prefixes.join("\n"))
}

#[test]
fn scanner_finds_tool_scoped_bare_multiline_and_expect_allowances() {
    let source = concat!(
        "// #[allow(tigerstyle::unchecked_narrowing)]\n",
        "/* #[allow(no_panic)] */\n",
        "const TEXT: &str = \"#[allow(tigerstyle::no_unwrap)]\";\n",
        "#[allow(tigerstyle::unchecked_narrowing)]\n",
        "fn first() {}\n",
        "#[allow(no_panic, reason = \"fixture\")]\n",
        "fn second() {}\n",
        "#[expect(\n",
        "    tigerstyle::unchecked_division,\n",
        "    reason = \"checked above\",\n",
        ")]\n",
        "fn third() {}\n",
    );

    let allowances = scan_allowances(source);

    assert_eq!(
        allowances
            .iter()
            .map(|allowance| (allowance.lint.as_str(), allowance.attribute.as_str(), allowance.line))
            .collect::<Vec<_>>(),
        vec![
            ("unchecked_narrowing", "allow", 4),
            ("no_panic", "allow", 6),
            ("unchecked_division", "expect", 8),
        ]
    );
}

#[test]
fn scanner_expands_group_allowances_to_member_lints() {
    let allowances = scan_allowances("#[allow(tigerstyle::safety)]\nfn guarded() {}\n");

    let lints = allowances.iter().map(|allowance| allowance.lint.as_str()).collect::<Vec<_>>();
    for member in lint_names_for_group("tigerstyle::safety") {
        assert!(lints.contains(&member), "missing safety member `{member}`");
    }
    assert!(lints.contains(&"unchecked_narrowing"));
}

#[test]
fn rank_score_weights_severity_then_trust_boundary() {
    assert!(rank_score("deny", true) > rank_score("deny", false));
    assert!(rank_score("deny", false) > rank_score("warn", true));
    assert!(rank_score("warn", true) > rank_score("warn", false));
    assert!(rank_score("warn", false) > rank_score("allow", true));
    assert_eq!(rank_score("warn", true), 5);
    assert_eq!(rank_score("allow", false), 2);
}

#[test]
fn candidate_is_listed_then_drops_after_validated_proof_ref() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    let source_path = source_dir.join("codec.rs");
    let source_text = format!("#[allow(tigerstyle::unchecked_narrowing)]\n// {ANCHOR}\npub fn narrow() {{}}\n");
    fs::write(&source_path, &source_text).expect("source");
    let deck_path = temp.path().join("proof-cards.json");
    write_deck(&deck_path, "passed");
    let reference_path = temp.path().join("proof-ref.json");
    write_reference(&reference_path, &reference_json(source_path.to_string_lossy().as_ref(), &source_text, 1));
    let trust_path = temp.path().join("trust.md");
    let trust_prefix = format!("{}/", source_dir.display());
    fs::write(&trust_path, trust_doc(&[trust_prefix.as_str()])).expect("trust doc");

    let before = build_report(&candidate_args(source_dir.clone(), None, Vec::new(), Some(trust_path.clone())))
        .expect("before report");
    assert_eq!(before.candidates.len(), 1);
    assert_eq!(before.candidates[0].lint, "unchecked_narrowing");
    assert_eq!(before.candidates[0].proof_ref.status, STATUS_MISSING);
    assert!(before.candidates[0].trust_boundary);
    assert!(before.candidates[0].rail.is_some());
    assert_eq!(before.discharged, 0);

    let after = build_report(&candidate_args(source_dir, Some(deck_path), vec![reference_path], Some(trust_path)))
        .expect("after report");
    assert!(after.candidates.is_empty());
    assert_eq!(after.discharged, 1);
    assert!(!after.is_blocking());
}

#[test]
fn invalid_or_unbound_proof_ref_keeps_candidate_listed_and_blocks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    let source_path = source_dir.join("codec.rs");
    let source_text = format!("#[allow(tigerstyle::unchecked_narrowing)]\n// {ANCHOR}\npub fn narrow() {{}}\n");
    fs::write(&source_path, &source_text).expect("source");
    let deck_path = temp.path().join("proof-cards.json");
    write_deck(&deck_path, "passed");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&[])).expect("trust doc");

    let mut stale = reference_json(source_path.to_string_lossy().as_ref(), &source_text, 1);
    stale["consumer_source_blake3"] = json!("stale-digest");
    let stale_path = temp.path().join("stale-proof-ref.json");
    write_reference(&stale_path, &stale);
    let stale_report = build_report(&candidate_args(
        source_dir.clone(),
        Some(deck_path.clone()),
        vec![stale_path],
        Some(trust_path.clone()),
    ))
    .expect("stale report");
    assert_eq!(stale_report.candidates.len(), 1);
    assert_eq!(stale_report.candidates[0].proof_ref.status, STATUS_INVALID);
    assert!(stale_report.is_blocking());

    let mut unbound = reference_json(source_path.to_string_lossy().as_ref(), &source_text, 2);
    unbound["consumer_context"] = json!("allow tigerstyle::unchecked_narrowing at elsewhere.rs:7");
    let unbound_path = temp.path().join("unbound-proof-ref.json");
    write_reference(&unbound_path, &unbound);
    let unbound_report =
        build_report(&candidate_args(source_dir, Some(deck_path), vec![unbound_path], Some(trust_path)))
            .expect("unbound report");
    assert_eq!(unbound_report.candidates.len(), 1);
    assert_eq!(unbound_report.candidates[0].proof_ref.status, STATUS_UNBOUND);
    assert!(unbound_report.is_blocking());
}

#[test]
fn unsupported_lints_are_reported_without_claiming_a_rail() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(&source_dir).expect("source dir");
    fs::write(source_dir.join("panic.rs"), "#[allow(no_panic)]\nfn guarded() {}\n").expect("source");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&[])).expect("trust doc");

    let report = build_report(&candidate_args(source_dir, None, Vec::new(), Some(trust_path))).expect("report");

    assert_eq!(report.candidates.len(), 1);
    assert!(report.candidates[0].unsupported);
    assert!(report.candidates[0].rail.is_none());
    assert_eq!(report.unsupported_lints.len(), 1);
    assert_eq!(report.unsupported_lints[0].lint, "no_panic");
    assert_eq!(report.unsupported_lints[0].candidates, 1);
}

#[test]
fn candidate_json_is_stable_and_rank_sorted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = temp.path().join("src");
    fs::create_dir_all(source_dir.join("nested")).expect("source dir");
    fs::write(
        source_dir.join("a.rs"),
        "#[allow(no_panic)]\nfn first() {}\n#[allow(tigerstyle::unchecked_narrowing)]\nfn second() {}\n",
    )
    .expect("a");
    fs::write(source_dir.join("nested/b.rs"), "#[allow(tigerstyle::unchecked_division)]\nfn third() {}\n").expect("b");
    let trust_path = temp.path().join("trust.md");
    fs::write(&trust_path, trust_doc(&["src/nested/"])).expect("trust doc");
    let args = candidate_args(source_dir, None, Vec::new(), Some(trust_path));

    let first = build_report(&args).expect("first report");
    let second = build_report(&args).expect("second report");
    let first_json = serde_json::to_string(&first).expect("first json");
    let second_json = serde_json::to_string(&second).expect("second json");

    assert_eq!(first_json, second_json);
    assert_eq!(
        first.candidates.iter().map(|candidate| candidate.rank).collect::<Vec<_>>(),
        (1..=first.candidates.len()).collect::<Vec<_>>()
    );
    assert!(first.candidates.windows(2).all(|pair| {
        pair[0].rank_score > pair[1].rank_score
            || (pair[0].rank_score == pair[1].rank_score
                && (pair[0].lint.as_str(), pair[0].path.as_str(), pair[0].line)
                    <= (pair[1].lint.as_str(), pair[1].path.as_str(), pair[1].line))
    }));
    assert_eq!(first.schema_version, SCHEMA_VERSION);
}

#[test]
fn trust_boundary_parser_reads_only_marked_fence() {
    let text = format!(
        "```text\nsrc/ignored.rs\n```\n\n```text {TRUST_BOUNDARY_FENCE}\nsrc/verified/\ncargo-octet/src/prove.rs\n```\n"
    );
    let prefixes = parse_trust_boundary_prefixes(&text);

    assert_eq!(prefixes, vec!["cargo-octet/src/prove.rs".to_owned(), "src/verified/".to_owned()]);
    assert!(on_trust_boundary("src/verified/addition.rs", &prefixes));
    assert!(!on_trust_boundary("src/verifiedness/addition.rs", &prefixes));
    assert!(!on_trust_boundary("src/ignored.rs", &prefixes));
}

fn negative_args(temp: &Path, baseline_status: &str, mutant_status: &str) -> (NegativeCheckArgs, PathBuf) {
    let baseline = temp.join("baseline/proof-cards.json");
    let mutant = temp.join("mutant/proof-cards.json");
    fs::create_dir_all(baseline.parent().expect("baseline parent")).expect("baseline dir");
    fs::create_dir_all(mutant.parent().expect("mutant parent")).expect("mutant dir");
    write_deck(&baseline, baseline_status);
    write_deck(&mutant, mutant_status);
    (
        NegativeCheckArgs {
            proof_cards: baseline.clone(),
            mutant_cards: mutant,
            proof_or_property_id: PROPERTY_ID.to_owned(),
            mutant_source: "src/codec_mutant.rs".to_owned(),
            output: None,
        },
        baseline,
    )
}

#[test]
fn passed_mutant_reports_property_not_demonstrated_in_proof_cards() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (args, baseline) = negative_args(temp.path(), "passed", "passed");

    let code = run_record_negative_check(&args, OutputFormat::Json).expect("record negative check");
    let deck = proof_ref::read_deck_file(&baseline).expect("read deck");
    let negative = deck.cards[0].negative_check.as_ref().expect("negative check");

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_NOT_DEMONSTRATED);
    assert_eq!(negative.mutant_card_status, "passed");
    assert_eq!(negative.schema_version, NEGATIVE_CHECK_SCHEMA_VERSION);
    assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("property not demonstrated")));
}

#[test]
fn failed_mutant_reports_property_demonstrated_for_this_mutation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (args, baseline) = negative_args(temp.path(), "passed", "failed");

    let code = run_record_negative_check(&args, OutputFormat::Human).expect("record negative check");
    let deck = proof_ref::read_deck_file(&baseline).expect("read deck");
    let negative = deck.cards[0].negative_check.as_ref().expect("negative check");

    assert_eq!(code, 0);
    assert_eq!(negative.status, NEGATIVE_STATUS_DEMONSTRATED);
    assert_eq!(negative.mutant_card_status, "failed");
    assert!(negative.non_claims.iter().any(|non_claim| non_claim.contains("this mutation only")));
}

#[test]
fn missing_mutant_card_is_inconclusive_and_does_not_overwrite_mutant_deck() {
    let temp = tempfile::tempdir().expect("tempdir");
    let baseline = temp.path().join("baseline/proof-cards.json");
    let mutant = temp.path().join("mutant/proof-cards.json");
    fs::create_dir_all(baseline.parent().expect("baseline parent")).expect("baseline dir");
    fs::create_dir_all(mutant.parent().expect("mutant parent")).expect("mutant dir");
    write_deck(&baseline, "passed");
    fs::write(&mutant, "{\n  \"schema_version\": \"octet-proof-cards/v1\",\n  \"cards\": []\n}\n")
        .expect("empty mutant deck");
    let args = NegativeCheckArgs {
        proof_cards: baseline,
        mutant_cards: mutant.clone(),
        proof_or_property_id: PROPERTY_ID.to_owned(),
        mutant_source: "src/codec_mutant.rs".to_owned(),
        output: None,
    };

    let code = run_record_negative_check(&args, OutputFormat::Human).expect("record negative check");
    let deck = proof_ref::read_deck_file(Path::new(&args.proof_cards)).expect("read deck");
    let negative = deck.cards[0].negative_check.as_ref().expect("negative check");

    assert_eq!(code, 1);
    assert_eq!(negative.status, NEGATIVE_STATUS_INCONCLUSIVE);
    assert_eq!(negative.mutant_card_status, "absent");
    assert!(negative.diagnostics.iter().any(|diagnostic| diagnostic.contains("inconclusive")));

    let mut aliasing = args;
    aliasing.output = Some(mutant);
    let error = run_record_negative_check(&aliasing, OutputFormat::Human).expect_err("aliasing output");
    assert!(error.contains("must not overwrite"));
}

#[test]
fn fixture_crate_allowance_is_line_42_and_maps_the_existing_rail() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repository root");
    let fixture = repository.join("fixtures/proof-candidates");
    let report =
        build_report(&candidate_args(fixture.join("src"), None, Vec::new(), Some(fixture.join("trust-boundary.md"))))
            .expect("fixture report");

    assert_eq!(report.candidates.len(), 1);
    let candidate = &report.candidates[0];
    assert_eq!(candidate.lint, "unchecked_narrowing");
    assert_eq!(candidate.line, 42);
    assert_eq!(candidate.rail.as_ref().expect("mapped rail").proof_or_property_id, PROPERTY_ID);
    assert_eq!(candidate.proof_ref.status, STATUS_MISSING);
}
