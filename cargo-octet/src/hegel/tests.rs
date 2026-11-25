use std::fmt::Write as _;

use super::*;

#[path = "suites/mod.rs"]
mod suites;

const CASE_BUDGET: u64 = 64;
const COUNTEREXAMPLE_CASES: u64 = 7;
const PROPERTY_ID: &str = "octet.hegel.issue-code-normalization-idempotent";
const SOURCE_SCOPE: &str =
    "cargo-octet/src/hegel/mod.rs,cargo-octet/src/hegel/tests.rs,cargo-octet/src/hegel/validation.rs";
const PASSED_RESULT: &str = "passed";
const EXPECTED_ARTIFACT_COUNT: usize = 5;
const OUTPUT_STAGE_EXTENSION: &str = "octet-stage";
const TEST_FILE_BYTE_LIMIT: u64 = 64;
const MAX_GENERATED_TEXT_BYTES: usize = 32;
const MAX_GENERATED_VECTOR_ITEMS: usize = 16;
const COUNTEREXAMPLE_RESULT: &str = "counterexample";
const COUNTEREXAMPLE_SCHEMA: &str = "octet-hegel-counterexample/v2";
const MAX_COUNTEREXAMPLE_TEXT_BYTES: usize = 512;
const MAX_COUNTEREXAMPLE_DRAWS: usize = 16;
const INJECTED_FAILURE_CHARS: usize = 3;
const INJECTED_PROPERTY: &str = "injected_short_text_property";

type InputMutation = Box<dyn Fn(&mut request::Input, &mut Profile)>;
pub(super) type ProfileLawSpec = (&'static str, &'static str, fn(Option<String>));

thread_local! {
    static PASSED_CASES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static CASE_FAILED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Counts generated cases that pass before the running property's first failure.
pub(super) struct CaseTally;

impl Drop for CaseTally {
    fn drop(&mut self) {
        if std::thread::panicking() {
            CASE_FAILED.set(true);
        } else if !CASE_FAILED.get() {
            PASSED_CASES.set(PASSED_CASES.get().saturating_add(1));
        }
    }
}

/// Bounded counterexample retained at the profile's `counterexample_path`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AdapterCounterexample {
    schema_version: String,
    law_id: String,
    law_family: String,
    property: String,
    issue_codes: Vec<String>,
    failure: String,
    draws: Vec<String>,
    truncated: bool,
    generator_identity: String,
    source_manifest_identity: String,
    shrink_termination: String,
    achieved_size: u64,
    replay_database_key: String,
    replay_bytes_hex: String,
}

struct PropertyFailure {
    counterexample: AdapterCounterexample,
    cases_completed: u64,
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
fn issue_code_normalization_is_idempotent(tc: hegel::TestCase, db_path: Option<String>) {
    let values = tc.draw(
        hegel::generators::vecs(hegel::generators::text().max_size(MAX_GENERATED_TEXT_BYTES))
            .max_size(MAX_GENERATED_VECTOR_ITEMS),
    );
    let _case = CaseTally;
    let once = normalize_issue_codes(&values);
    let twice = normalize_issue_codes(&once);
    assert_eq!(once, twice);
    assert!(once.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(once.iter().all(|value| !value.is_empty() && value.trim() == value));
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
fn raw_projection_equivalence_is_rejected(tc: hegel::TestCase) {
    let value = tc.draw(hegel::generators::text().max_size(MAX_GENERATED_TEXT_BYTES));
    tc.assume(!value.trim().is_empty());
    let _case = CaseTally;
    let raw = vec![value.clone(), format!(" {value} ")];
    assert_ne!(raw, normalize_issue_codes(&raw));
}

fn run_named_profile_adapter(profile_id: &str) {
    let specs = suites::specs_for_profile(profile_id);
    let Some(output_path) = std::env::var_os("OCTET_HEGEL_OUTPUT") else {
        for (_, _, property) in specs {
            property(None);
        }
        if profile_id == PROFILE_ID {
            raw_projection_equivalence_is_rejected();
        }
        return;
    };
    let counterexample_path = std::path::PathBuf::from(required_environment("OCTET_HEGEL_COUNTEREXAMPLE"));
    record_profile_run(
        &specs,
        producer_summary_from_environment(profile_id),
        std::path::Path::new(&output_path),
        &counterexample_path,
    );
}

// r[verify octet.law_tested_deterministic_cores.hegel_catalog]
// r[verify octet.dynamic_evidence.hegel_profiled_run]
#[test]
fn hegel_profile_adapter() {
    run_named_profile_adapter(PROFILE_ID);
}

#[test]
fn identity_profile_adapter() {
    run_named_profile_adapter("octet-evidence-identity-laws-v1");
}

#[test]
fn lineage_profile_adapter() {
    run_named_profile_adapter("octet-evidence-lineage-laws-v1");
}

#[test]
fn catalog_profile_adapter() {
    run_named_profile_adapter("octet-catalog-composition-laws-v1");
}

#[test]
fn reduction_profile_adapter() {
    run_named_profile_adapter("octet-reduction-acceptance-laws-v1");
}

#[test]
fn admission_profile_adapter() {
    run_named_profile_adapter("octet-requirement-admission-laws-v1");
}

#[test]
fn freshness_profile_adapter() {
    run_named_profile_adapter("octet-freshness-mutation-laws-v1");
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
fn injected_short_text_property(tc: hegel::TestCase, db_path: Option<String>) {
    let value = tc.draw(hegel::generators::text().max_size(MAX_GENERATED_TEXT_BYTES));
    let _case = CaseTally;
    assert!(value.chars().count() < INJECTED_FAILURE_CHARS, "generated text is too long");
}

// r[verify octet.dynamic_evidence.hegel_property_receipt.counterexample]
#[test]
fn profile_adapter_records_failing_property_as_counterexample() {
    let (mut input, profile) = valid_input_for(PROFILE_ID);
    let directory = tempfile::tempdir().expect("adapter output directory");
    let summary_path = directory.path().join("summary.json");
    let counterexample_path = directory.path().join("counterexample.json");
    let mut summary: ProducerSummary = serde_json::from_str(&input.summary_text).expect("passed summary");
    summary.expected_laws = vec![INJECTED_PROPERTY.to_owned()];
    let injected: [ProfileLawSpec; 1] = [(INJECTED_PROPERTY, "pure_core", injected_short_text_property)];

    record_profile_run(&injected, summary, &summary_path, &counterexample_path);

    let counterexample = std::fs::read(&counterexample_path).expect("counterexample published");
    let retained: AdapterCounterexample = serde_json::from_slice(&counterexample).expect("counterexample JSON");
    assert_eq!(retained.schema_version, COUNTEREXAMPLE_SCHEMA);
    assert_eq!(retained.law_id, INJECTED_PROPERTY);
    assert_eq!(retained.property, INJECTED_PROPERTY);
    assert_eq!(retained.issue_codes, [INJECTED_PROPERTY]);
    assert_eq!(retained.shrink_termination, "shrunk");
    assert!(retained.achieved_size > 0);
    assert!(retained.failure.contains("generated text is too long"), "{retained:?}");
    assert!(retained.draws.iter().any(|line| line.starts_with("let value = ")), "{retained:?}");
    assert!(!retained.truncated);
    assert!(!retained.replay_database_key.is_empty(), "{retained:?}");
    assert!(!retained.replay_bytes_hex.is_empty(), "{retained:?}");
    assert!(decode_hex(&retained.replay_bytes_hex).is_ok(), "{retained:?}");
    assert!(u64::try_from(counterexample.len()).expect("length") <= profile.limits.max_counterexample_bytes);

    let mut summary_after: ProducerSummary =
        serde_json::from_slice(&std::fs::read(&summary_path).expect("read summary")).expect("summary");
    summary_after.property_id = PROPERTY_ID.to_owned();
    summary_after.expected_laws = vec![PROPERTY_ID.to_owned()];
    summary_after.executed_laws[0].law_id = PROPERTY_ID.to_owned();
    input.summary_text = String::from_utf8(pretty_bytes(&summary_after, "summary").expect("pretty")).expect("utf8");
    input.counterexample = Some(counterexample);
    let evaluation = validation::evaluate(&input, &profile);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("counterexample details");
    assert_eq!(details.result, COUNTEREXAMPLE_RESULT);
    assert!(details.cases_completed < profile.settings.case_budget);
    assert!(details.counterexample_identity.is_some());

    let text = request::render(&input).expect("render counterexample input");
    let command = profile.command_argv.join(" ");
    let receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
        rail: crate::evidence_rails::EvidenceRailKind::HegelProperty,
        mode: "run",
        import_text: &text,
        workspace_scope: ".",
        config_path: Some("standards/hegel/profiles.json"),
        baseline_identity: None,
        lockfile_identity: Some(input.cargo_lock_identity.clone()),
        locked: true,
        advisory_updated_by_prior_step: false,
        profile: Some(PROFILE_ID),
        requested_command: Some(&command),
        parent_artifacts: Vec::new(),
        tool_exit_code: Some(0),
    });
    assert_eq!(receipt.status, "failed");
    assert_eq!(receipt.finding_counts.get("hegel-counterexample"), Some(&1));
}

#[test]
fn all_registered_profiles_validate_and_execute_full_law_inventories() {
    let profiles = all_profiles().expect("profiles");
    assert_eq!(profiles.len(), profile_validation::REGISTERED_PROFILE_IDS.len());
    for profile in profiles {
        assert!(
            profile_validation::validate(&profile).is_empty(),
            "profile `{}` failed validation",
            profile.profile_id
        );
        let (mut input, _) = valid_input_for(&profile.profile_id);
        let specs = suites::specs_for_profile(&profile.profile_id);
        let expected_ids = laws_for_profile(&profile.profile_id)
            .expect("laws")
            .into_iter()
            .map(|law| law.law_id)
            .collect::<std::collections::BTreeSet<_>>();
        let spec_ids =
            specs.iter().map(|(law_id, _, _)| (*law_id).to_owned()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(expected_ids, spec_ids, "law mismatch for `{}`", profile.profile_id);

        let dir = tempfile::tempdir().expect("tempdir");
        let summary_path = dir.path().join("summary.json");
        let counterexample_path = dir.path().join("counterexample.json");
        let summary: ProducerSummary = serde_json::from_str(&input.summary_text).expect("summary");
        record_profile_run(&specs, summary, &summary_path, &counterexample_path);
        assert!(!counterexample_path.exists(), "unexpected counterexample for `{}`", profile.profile_id);

        input.summary_text = std::fs::read_to_string(&summary_path).expect("read summary");
        let evaluation = validation::evaluate(&input, &profile);
        assert!(
            evaluation.diagnostics.is_empty(),
            "profile `{}` failed evaluation: {:?}",
            profile.profile_id,
            evaluation.diagnostics
        );
        let details = evaluation.details.expect("details");
        assert_eq!(details.result, PASSED_RESULT);
        assert!(details.unexecuted_laws.is_empty());
        assert_eq!(details.executed_laws.len(), specs.len());
        assert!(details.executed_laws.iter().all(|law| law.cases_completed == profile.settings.case_budget));

        let text = request::render(&input).expect("render input");
        let command = profile.command_argv.join(" ");
        let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::HegelProperty,
            mode: "run",
            import_text: &text,
            workspace_scope: ".",
            config_path: Some("standards/hegel/profiles.json"),
            baseline_identity: None,
            lockfile_identity: Some(input.cargo_lock_identity.clone()),
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: Some(&profile.profile_id),
            requested_command: Some(&command),
            parent_artifacts: Vec::new(),
            tool_exit_code: Some(0),
        });
        receipt.config_identity = Some(PROFILE_IDENTITY.to_owned());
        assert!(
            crate::evidence_rails::validate_receipt(&receipt).is_empty(),
            "receipt validation failed for `{}`",
            profile.profile_id
        );
    }
}

#[test]
fn failing_counterexample_retains_law_identity_and_replays_from_saved_bytes() {
    let broken_law_id = "octet.hegel.freshness.buoy-single-role-mutation-and-role-confusion";
    let db_dir = tempfile::tempdir().expect("db dir");
    let db_root = db_dir.path().to_string_lossy().into_owned();
    let broken_control = |tc: hegel::TestCase| {
        let len = tc.draw(hegel::generators::integers::<usize>().min_value(0).max_value(16));
        let mut items = Vec::with_capacity(len);
        for idx in 0..len {
            items.push(format!("item-{idx}"));
        }
        // Deliberately broken invariant: claims bounded vector has < 3 elements.
        assert!(items.len() < INJECTED_FAILURE_CHARS, "control violated bounded length");
    };

    let failure = hegel::with_output_override(std::sync::Arc::new(|_| {}), || {
        std::panic::catch_unwind(move || {
            hegel::Hegel::new(broken_control)
                .settings(
                    hegel::Settings::new()
                        .test_cases(CASE_BUDGET)
                        .derandomize(true)
                        .database(Some(db_root))
                        .phases([hegel::Phase::Generate, hegel::Phase::Shrink]),
                )
                .__database_key(broken_law_id.to_owned())
                .run();
        })
    });
    assert!(failure.is_err(), "broken control must fail");

    let primary_dir = hegel_db_key_dir(broken_law_id.as_bytes());
    let saved_files = collect_regular_files(&db_dir.path().join(&primary_dir));
    assert!(!saved_files.is_empty(), "hegeltest must persist shrunk choice bytes in primary key dir");
    let (file_name, saved_bytes) = saved_files[0].clone();
    let rel_path = std::path::PathBuf::from(&primary_dir).join(file_name);
    let hex_bytes = encode_hex(&saved_bytes);

    let (mut input, profile) = valid_input_for("octet-freshness-mutation-laws-v1");
    let mut summary: ProducerSummary = serde_json::from_str(&input.summary_text).expect("summary");
    let counterexample = AdapterCounterexample {
        schema_version: COUNTEREXAMPLE_SCHEMA.to_owned(),
        law_id: broken_law_id.to_owned(),
        law_family: "freshness".to_owned(),
        property: broken_law_id.to_owned(),
        issue_codes: vec![broken_law_id.to_owned()],
        failure: "control violated bounded length".to_owned(),
        draws: vec![format!("len = {INJECTED_FAILURE_CHARS}")],
        truncated: false,
        generator_identity: summary.generator_manifest_identity.clone(),
        source_manifest_identity: summary.source_manifest_identity.clone(),
        shrink_termination: "shrunk".to_owned(),
        achieved_size: u64::try_from(saved_bytes.len()).expect("size"),
        replay_database_key: broken_law_id.to_owned(),
        replay_bytes_hex: hex_bytes.clone(),
    };
    let ce_bytes = pretty_bytes(&counterexample, "counterexample").expect("ce bytes");
    summary.result = COUNTEREXAMPLE_RESULT.to_owned();
    summary.cases_completed = 1;
    summary.issue_codes = vec![broken_law_id.to_owned()];
    summary.counterexample_identity = Some(prefixed_identity(&ce_bytes));
    summary.executed_laws[0].status = COUNTEREXAMPLE_RESULT.to_owned();
    summary.executed_laws[0].cases_completed = 1;
    summary.executed_laws[0].shrink_termination = "shrunk".to_owned();
    summary.executed_laws[0].achieved_size = Some(counterexample.achieved_size);
    input.summary_text = String::from_utf8(pretty_bytes(&summary, "summary").expect("s")).expect("utf8");
    input.counterexample = Some(ce_bytes);
    let valid_eval = validation::evaluate(&input, &profile);
    assert!(valid_eval.diagnostics.is_empty(), "{:?}", valid_eval.diagnostics);

    // Deterministic replay from retained bytes ONLY (Phase::Reuse, no Generate phase!).
    let replay_dir = tempfile::tempdir().expect("replay dir");
    let target_file = replay_dir.path().join(&rel_path);
    std::fs::create_dir_all(target_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&target_file, decode_hex(&hex_bytes).expect("decode hex")).expect("write replay bytes");
    let replay_root = replay_dir.path().to_string_lossy().into_owned();
    let observed_len = std::sync::Arc::new(std::sync::Mutex::new(None::<usize>));
    let obs_clone = std::sync::Arc::clone(&observed_len);
    let replay_result = hegel::with_output_override(std::sync::Arc::new(|_| {}), || {
        std::panic::catch_unwind(move || {
            hegel::Hegel::new(move |tc: hegel::TestCase| {
                let len = tc.draw(hegel::generators::integers::<usize>().min_value(0).max_value(16));
                *obs_clone.lock().expect("obs") = Some(len);
                assert!(len < INJECTED_FAILURE_CHARS, "control violated bounded length");
            })
            .settings(
                hegel::Settings::new()
                    .test_cases(1)
                    .derandomize(true)
                    .database(Some(replay_root))
                    .phases([hegel::Phase::Reuse, hegel::Phase::Generate]),
            )
            .__database_key(broken_law_id.to_owned())
            .run();
        })
    });
    assert!(replay_result.is_err(), "Phase::Reuse must replay failing bytes");
    assert_eq!(
        *observed_len.lock().expect("obs"),
        Some(INJECTED_FAILURE_CHARS),
        "replayed shrunk input must preserve minimal failing size"
    );

    // Stale generator or source identities cannot reuse the old receipt.
    let mut stale_gen = input.clone();
    stale_gen.generator_manifest[0].identity = prefixed_identity(b"stale-generator");
    assert!(validation::evaluate(&stale_gen, &profile).details.is_none());

    let mut stale_src = input;
    stale_src.source_manifest[0].identity = prefixed_identity(b"stale-source");
    assert!(validation::evaluate(&stale_src, &profile).details.is_none());
}

#[test]
fn zero_selected_filtered_ignored_or_zero_case_runs_fail_closed() {
    let (base_input, base_profile) = valid_input_for("octet-freshness-mutation-laws-v1");
    let base_summary: ProducerSummary = serde_json::from_str(&base_input.summary_text).expect("summary");

    // 1. Zero selected laws.
    let mut zero_laws = base_summary.clone();
    zero_laws.expected_laws.clear();
    zero_laws.executed_laws.clear();
    assert!(eval_summary(&base_input, &base_profile, &zero_laws).details.is_none());

    // 2. Filtered / unexecuted law while claiming passed.
    let mut filtered = base_summary.clone();
    let dropped = filtered.executed_laws.pop().expect("executed law");
    filtered.unexecuted_laws.push(dropped.law_id);
    assert!(eval_summary(&base_input, &base_profile, &filtered).details.is_none());

    // 3. Zero generated cases while claiming passed.
    let mut zero_cases = base_summary.clone();
    zero_cases.cases_completed = 0;
    zero_cases.executed_laws[0].cases_completed = 0;
    assert!(eval_summary(&base_input, &base_profile, &zero_cases).details.is_none());

    // 4. Ignored / incomplete law status while claiming passed.
    let mut ignored = base_summary.clone();
    ignored.executed_laws[0].status = "ignored".to_owned();
    assert!(eval_summary(&base_input, &base_profile, &ignored).details.is_none());

    // 5. Output truncation / resource exhaustion result status.
    for bad_result in ["incomplete", "truncated", "exhausted", "ignored"] {
        let mut exhausted = base_summary.clone();
        exhausted.result = bad_result.to_owned();
        assert!(eval_summary(&base_input, &base_profile, &exhausted).details.is_none());
    }
}

#[test]
fn buoy_mutation_matrix_is_complete_and_every_dimension_discriminates() {
    suites::buoy_matrix::verify_all_16_dimensions();
}

fn eval_summary(input: &request::Input, profile: &Profile, summary: &ProducerSummary) -> Evaluation {
    let mut candidate = input.clone();
    candidate.summary_text =
        String::from_utf8(pretty_bytes(summary, "mutated summary").expect("summary")).expect("utf8");
    validation::evaluate(&candidate, profile)
}

fn record_profile_run(
    specs: &[ProfileLawSpec],
    mut summary: ProducerSummary,
    summary_path: &std::path::Path,
    counterexample_path: &std::path::Path,
) {
    let mut expected_laws = specs.iter().map(|(law_id, _, _)| (*law_id).to_owned()).collect::<Vec<_>>();
    expected_laws.sort();
    summary.expected_laws = expected_laws;
    summary.executed_laws.clear();
    summary.unexecuted_laws.clear();

    let mut first_failure: Option<PropertyFailure> = None;
    for &(law_id, family, property) in specs {
        if first_failure.is_some() {
            summary.unexecuted_laws.push(law_id.to_owned());
            continue;
        }
        if let Some(failure) = run_profile_property(
            law_id,
            family,
            property,
            &summary.generator_manifest_identity,
            &summary.source_manifest_identity,
        ) {
            summary.executed_laws.push(LawExecutionRecord {
                law_id: law_id.to_owned(),
                family: family.to_owned(),
                status: COUNTEREXAMPLE_RESULT.to_owned(),
                cases_completed: failure.cases_completed,
                shrink_termination: failure.counterexample.shrink_termination.clone(),
                achieved_size: Some(failure.counterexample.achieved_size),
            });
            first_failure = Some(failure);
        } else {
            summary.executed_laws.push(LawExecutionRecord {
                law_id: law_id.to_owned(),
                family: family.to_owned(),
                status: PASSED_RESULT.to_owned(),
                cases_completed: summary.settings.case_budget,
                shrink_termination: "not_applicable".to_owned(),
                achieved_size: None,
            });
        }
    }
    summary.unexecuted_laws.sort();
    if let Some(failure) = first_failure {
        let bytes = pretty_bytes(&failure.counterexample, "Hegel producer adapter counterexample")
            .expect("serialize adapter counterexample");
        publish_adapter_output(counterexample_path, &bytes);
        summary.property_id = failure.counterexample.law_id.clone();
        summary.result = COUNTEREXAMPLE_RESULT.to_owned();
        summary.cases_completed = failure.cases_completed;
        summary.issue_codes = failure.counterexample.issue_codes;
        summary.counterexample_identity = Some(prefixed_identity(&bytes));
    }
    let bytes = pretty_bytes(&summary, "Hegel producer adapter summary").expect("serialize adapter summary");
    publish_adapter_output(summary_path, &bytes);
}

fn run_profile_property(
    law_id: &str,
    family: &str,
    property: fn(Option<String>),
    generator_identity: &str,
    source_manifest_identity: &str,
) -> Option<PropertyFailure> {
    let draws = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let sink_draws = std::sync::Arc::clone(&draws);
    let sink: std::sync::Arc<dyn Fn(&str) + Send + Sync> = std::sync::Arc::new(move |line: &str| {
        sink_draws.lock().expect("Hegel draw sink").push(line.to_owned());
    });
    let db_dir = tempfile::tempdir().expect("counterexample db dir");
    let db_root = db_dir.path().to_string_lossy().into_owned();
    PASSED_CASES.set(0);
    CASE_FAILED.set(false);
    let payload = hegel::with_output_override(sink, || std::panic::catch_unwind(|| property(Some(db_root)))).err()?;
    let message = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-text panic payload".to_owned());
    let draws = std::mem::take(&mut *draws.lock().expect("Hegel draw sink"));
    let mut truncated = draws.len() > MAX_COUNTEREXAMPLE_DRAWS;
    let bounded_draws = draws
        .iter()
        .take(MAX_COUNTEREXAMPLE_DRAWS)
        .map(|line| bounded_counterexample_text(line, &mut truncated))
        .collect::<Vec<_>>();
    let achieved_size = u64::try_from(bounded_draws.iter().map(String::len).sum::<usize>()).unwrap_or(u64::MAX);
    let (replay_database_key, replay_bytes_hex) = extract_replay_from_db(db_dir.path(), law_id);
    Some(PropertyFailure {
        counterexample: AdapterCounterexample {
            schema_version: COUNTEREXAMPLE_SCHEMA.to_owned(),
            law_id: law_id.to_owned(),
            law_family: family.to_owned(),
            property: law_id.to_owned(),
            issue_codes: vec![law_id.to_owned()],
            failure: bounded_counterexample_text(&message, &mut truncated),
            draws: bounded_draws,
            truncated,
            generator_identity: generator_identity.to_owned(),
            source_manifest_identity: source_manifest_identity.to_owned(),
            shrink_termination: "shrunk".to_owned(),
            achieved_size,
            replay_database_key,
            replay_bytes_hex,
        },
        cases_completed: PASSED_CASES.get(),
    })
}

fn extract_replay_from_db(db_root: &std::path::Path, fallback_key: &str) -> (String, String) {
    let metakeys = collect_regular_files(&db_root.join(hegel_db_key_dir(b".hegel-keys")));
    let Some((_, key_bytes)) = metakeys.into_iter().next() else {
        return (fallback_key.to_owned(), String::new());
    };
    let Ok(key_str) = String::from_utf8(key_bytes) else {
        return (fallback_key.to_owned(), String::new());
    };
    let values = collect_regular_files(&db_root.join(hegel_db_key_dir(key_str.as_bytes())));
    let Some((_, bytes)) = values.into_iter().next() else {
        return (key_str, String::new());
    };
    (key_str, encode_hex(&bytes))
}

fn bounded_counterexample_text(value: &str, truncated: &mut bool) -> String {
    if value.len() <= MAX_COUNTEREXAMPLE_TEXT_BYTES {
        return value.to_owned();
    }
    *truncated = true;
    value[..value.floor_char_boundary(MAX_COUNTEREXAMPLE_TEXT_BYTES)].to_owned()
}

fn publish_adapter_output(path: &std::path::Path, bytes: &[u8]) {
    let stage_path = path.with_extension(OUTPUT_STAGE_EXTENSION);
    std::fs::write(&stage_path, bytes).expect("stage adapter output");
    std::fs::rename(&stage_path, path).expect("publish adapter output");
}

// r[verify octet.dynamic_evidence.hegel_measured_identity]
// r[verify octet.dynamic_evidence.hegel_hardening_verification]
#[cfg(unix)]
#[test]
fn capability_read_rejects_leaf_and_parent_symlink_substitution() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().expect("tempdir");
    let outside = tempdir.path().join("outside");
    std::fs::create_dir_all(tempdir.path().join("safe")).expect("safe directory");
    std::fs::create_dir_all(&outside).expect("outside directory");
    std::fs::write(tempdir.path().join("safe/input.json"), b"safe").expect("safe input");
    std::fs::write(outside.join("input.json"), b"outside").expect("outside input");
    symlink(outside.join("input.json"), tempdir.path().join("leaf.json")).expect("leaf symlink");
    symlink(&outside, tempdir.path().join("linked-dir")).expect("directory symlink");

    let valid =
        safe_read::read_checked_input(tempdir.path(), std::path::Path::new("safe/input.json"), TEST_FILE_BYTE_LIMIT)
            .expect("safe input");
    assert_eq!(valid, b"safe");
    for locator in ["leaf.json", "linked-dir/input.json"] {
        let error = safe_read::read_checked_input(tempdir.path(), std::path::Path::new(locator), TEST_FILE_BYTE_LIMIT)
            .expect_err("symlink substitution rejected");
        assert!(error.contains("symlink"));
    }
}

// r[verify octet.dynamic_evidence.hegel_profiled_run]
// r[verify octet.dynamic_evidence.hegel_output_contract]
// r[verify octet.dynamic_evidence.hegel_measured_identity]
#[test]
fn profiled_summary_binds_measured_inputs_and_replays_artifacts() {
    let (input, profile) = valid_input_for(PROFILE_ID);
    let text = request::render(&input).expect("render profiled input");
    let evaluation = request::evaluate(&text, PROFILE_ID);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("profiled details");
    assert!(validate(&details).is_empty());
    let artifacts = request::artifact_payloads(&details, &text).expect("artifact replay");
    assert_eq!(artifacts.len(), EXPECTED_ARTIFACT_COUNT);
    assert_eq!(details.command_argv, profile.command_argv);

    let command = profile.command_argv.join(" ");
    let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
        rail: crate::evidence_rails::EvidenceRailKind::HegelProperty,
        mode: "run",
        import_text: &text,
        workspace_scope: ".",
        config_path: Some("standards/hegel/profiles.json"),
        baseline_identity: None,
        lockfile_identity: Some(input.cargo_lock_identity.clone()),
        locked: true,
        advisory_updated_by_prior_step: false,
        profile: Some(PROFILE_ID),
        requested_command: Some(&command),
        parent_artifacts: Vec::new(),
        tool_exit_code: Some(0),
    });
    receipt.config_identity = Some(PROFILE_IDENTITY.to_owned());
    assert!(crate::evidence_rails::validate_receipt(&receipt).is_empty());
    assert!(receipt.hegel_property.and_then(|value| value.hardened).is_some());
}

// r[verify octet.dynamic_evidence.hegel_hardening_verification]
#[test]
fn profiled_summary_rejects_authority_drift_stale_identities_limits_and_overclaims() {
    let (input, profile) = valid_input_for(PROFILE_ID);
    let mutations: Vec<InputMutation> = vec![
        Box::new(|_, profile| profile.command_argv.push("--ignored".to_owned())),
        Box::new(|input, _| input.summary_text = "not-json".to_owned()),
        Box::new(|input, _| input.cargo_lock_identity = prefixed_identity(b"stale-lock")),
        Box::new(|input, _| input.source_manifest[0].locator = "../escape.rs".to_owned()),
        Box::new(|_input, profile| profile.limits.max_summary_bytes = 1),
        Box::new(|input, profile| {
            profile.limits.max_log_bytes = 1;
            input.bounded_log = b"too-long".to_vec();
        }),
        Box::new(|_input, profile| profile.limits.max_counterexample_bytes = 0),
        Box::new(|_input, profile| profile.limits.max_source_files = 0),
        Box::new(|_input, profile| profile.limits.max_generator_files = 0),
        Box::new(|_input, profile| profile.limits.max_issue_codes = 0),
        Box::new(|_input, profile| profile.limits.timeout_ms = 0),
        Box::new(|_input, profile| profile.limits.poll_interval_ms = 0),
        Box::new(|_input, profile| profile.limits.teardown_timeout_ms = 0),
    ];
    for mutate in mutations {
        let mut candidate = input.clone();
        let mut selected = profile.clone();
        mutate(&mut candidate, &mut selected);
        assert!(validation::evaluate(&candidate, &selected).details.is_none());
    }

    let mut promoted =
        request::evaluate(&request::render(&input).expect("render"), PROFILE_ID).details.expect("details");
    promoted.evidence_tier = "formal_proof".to_owned();
    assert!(validate(&promoted).iter().any(|diagnostic| diagnostic.contains("boundary")));
}

#[test]
fn receipt_identities_reject_lowercase_non_hex_letters() {
    let (input, _) = valid_input_for(PROFILE_ID);
    let mut details =
        request::evaluate(&request::render(&input).expect("render"), PROFILE_ID).details.expect("details");
    details.cargo_lock_identity = format!("b3:{}", "z".repeat(blake3::OUT_LEN * 2));
    details.projection_identity = prefixed_identity(&projection_bytes(&details).expect("projection bytes"));
    assert_eq!(validate(&details), vec!["profiled Hegel receipt identity is not canonical BLAKE3".to_owned()]);
}

#[test]
fn counterexample_contract_requires_matching_retained_bytes() {
    let (mut input, profile) = valid_input_for(PROFILE_ID);
    let counterexample =
        include_bytes!("../../tests/fixtures/evidence-rails/hegel-hardened-counterexample.json").to_vec();
    let mut summary: ProducerSummary = serde_json::from_str(&input.summary_text).expect("summary");
    summary.result = "counterexample".to_owned();
    summary.cases_completed = COUNTEREXAMPLE_CASES;
    summary.issue_codes = vec!["duplicate".to_owned()];
    summary.counterexample_identity = Some(prefixed_identity(&counterexample));
    summary.executed_laws[0].status = "counterexample".to_owned();
    summary.executed_laws[0].cases_completed = COUNTEREXAMPLE_CASES;
    input.counterexample = Some(counterexample);
    input.summary_text = String::from_utf8(pretty_bytes(&summary, "counterexample summary").expect("summary bytes"))
        .expect("summary UTF-8");
    let evaluation = validation::evaluate(&input, &profile);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);

    let mut stale = input;
    stale.counterexample = Some(b"substituted".to_vec());
    assert!(validation::evaluate(&stale, &profile).details.is_none());
}

pub(crate) fn valid_input_for(profile_id: &str) -> (request::Input, Profile) {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repo root");
    let profile = selected_profile(profile_id).expect("profile");
    let file_limit = shell::source_file_limit(&profile).expect("source file limit");
    let mut source_manifest = profile
        .source_scope
        .iter()
        .map(|locator| {
            let bytes = safe_read::read_checked_input(repo_root, std::path::Path::new(locator), file_limit)
                .expect("read source scope");
            file_identity(locator, &bytes)
        })
        .collect::<Vec<_>>();
    source_manifest.sort_by(|left, right| left.locator.cmp(&right.locator));
    let mut generator_manifest = profile
        .generator_inputs
        .iter()
        .map(|locator| {
            let bytes = safe_read::read_checked_input(repo_root, std::path::Path::new(locator), file_limit)
                .expect("read generator input");
            file_identity(locator, &bytes)
        })
        .collect::<Vec<_>>();
    generator_manifest.sort_by(|left, right| left.locator.cmp(&right.locator));

    let cargo_lock_identity = prefixed_identity(
        &safe_read::read_checked_input(repo_root, std::path::Path::new("Cargo.lock"), file_limit)
            .expect("read Cargo.lock"),
    );
    let package_manifest_identity = prefixed_identity(
        &safe_read::read_checked_input(
            repo_root,
            std::path::Path::new("cargo-octet/Cargo.toml"),
            profile.limits.max_summary_bytes,
        )
        .expect("read package manifest"),
    );
    let source_manifest_identity = request::manifest_identity(&source_manifest, "source manifest").expect("source");
    let generator_manifest_identity =
        request::manifest_identity(&generator_manifest, "generator manifest").expect("generator");

    let laws = laws_for_profile(profile_id).expect("laws");
    let mut expected_laws = laws.iter().map(|law| law.law_id.clone()).collect::<Vec<_>>();
    expected_laws.sort();
    let executed_laws = laws
        .iter()
        .map(|law| LawExecutionRecord {
            law_id: law.law_id.clone(),
            family: law.family.clone(),
            status: PASSED_RESULT.to_owned(),
            cases_completed: profile.settings.case_budget,
            shrink_termination: "not_applicable".to_owned(),
            achieved_size: None,
        })
        .collect::<Vec<_>>();
    let property_id = expected_laws.first().cloned().unwrap_or_else(|| PROPERTY_ID.to_owned());

    let summary = ProducerSummary {
        schema_version: SUMMARY_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        package: profile.producer.package.clone(),
        package_version: profile.producer.package_version.clone(),
        hegel_version: profile.producer.hegel_version.clone(),
        backend: profile.producer.backend.clone(),
        property_catalog: profile.property_catalog.clone(),
        property_id,
        linked_ids: vec!["octet.dynamic_evidence.hegel_output_contract".to_owned()],
        source_scope: profile.source_scope.join(","),
        settings: profile.settings.clone(),
        generator: GeneratorSummary {
            family: format!("bounded-{}", profile.family),
            content_hash: LAW_CATALOG_IDENTITY.to_owned(),
        },
        source_manifest_identity,
        generator_manifest_identity,
        cargo_lock_identity: cargo_lock_identity.clone(),
        package_manifest_identity: package_manifest_identity.clone(),
        result: PASSED_RESULT.to_owned(),
        cases_completed: profile.settings.case_budget,
        issue_codes: Vec::new(),
        counterexample_identity: None,
        expected_laws,
        executed_laws,
        unexecuted_laws: Vec::new(),
    };
    let summary_text = String::from_utf8(pretty_bytes(&summary, "producer summary").expect("summary")).expect("UTF-8");
    (
        request::Input {
            schema_version: REQUEST_SCHEMA.to_owned(),
            summary_text,
            bounded_log: b"bounded Hegel log".to_vec(),
            source_manifest,
            generator_manifest,
            cargo_lock_identity,
            package_manifest_identity,
            counterexample: None,
        },
        profile,
    )
}

fn file_identity(locator: &str, bytes: &[u8]) -> FileIdentity {
    FileIdentity {
        locator: locator.to_owned(),
        identity: prefixed_identity(bytes),
        bytes: u64::try_from(bytes.len()).expect("fixture byte length"),
    }
}

fn producer_summary_from_environment(profile_id: &str) -> ProducerSummary {
    let profile = selected_profile(profile_id).expect("profile");
    let laws = laws_for_profile(profile_id).expect("laws");
    let mut expected_laws = laws.iter().map(|law| law.law_id.clone()).collect::<Vec<_>>();
    expected_laws.sort();
    let property_id = expected_laws.first().cloned().unwrap_or_else(|| PROPERTY_ID.to_owned());
    let source_scope = if profile_id == PROFILE_ID {
        SOURCE_SCOPE.to_owned()
    } else {
        profile.source_scope.join(",")
    };
    ProducerSummary {
        schema_version: SUMMARY_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: required_environment("OCTET_HEGEL_PROFILE_IDENTITY"),
        package: profile.producer.package.clone(),
        package_version: profile.producer.package_version.clone(),
        hegel_version: profile.producer.hegel_version.clone(),
        backend: profile.producer.backend.clone(),
        property_catalog: profile.property_catalog.clone(),
        property_id,
        linked_ids: vec!["octet.dynamic_evidence.hegel_output_contract".to_owned()],
        source_scope,
        settings: profile.settings,
        generator: GeneratorSummary {
            family: format!("bounded-{}", profile.family),
            content_hash: LAW_CATALOG_IDENTITY.to_owned(),
        },
        source_manifest_identity: required_environment("OCTET_HEGEL_SOURCE_IDENTITY"),
        generator_manifest_identity: required_environment("OCTET_HEGEL_GENERATOR_IDENTITY"),
        cargo_lock_identity: required_environment("OCTET_HEGEL_LOCK_IDENTITY"),
        package_manifest_identity: required_environment("OCTET_HEGEL_PACKAGE_IDENTITY"),
        result: PASSED_RESULT.to_owned(),
        cases_completed: CASE_BUDGET,
        issue_codes: Vec::new(),
        counterexample_identity: None,
        expected_laws,
        executed_laws: Vec::new(),
        unexecuted_laws: Vec::new(),
    }
}

fn required_environment(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing {name}"))
}

fn collect_regular_files(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    collect_regular_files_inner(root, root, &mut out);
    out.sort_by(|left, right| left.0.cmp(&right.0));
    out
}

fn collect_regular_files_inner(
    root: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(std::path::PathBuf, Vec<u8>)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_regular_files_inner(root, &path, out);
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(root)
            && let Ok(bytes) = std::fs::read(&path)
        {
            out.push((rel.to_path_buf(), bytes));
        }
    }
}

fn hegel_db_key_dir(key: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in b"native:".iter().chain(key.iter()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("hex write");
    }
    out
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("odd hex length".to_owned());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for idx in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[idx..idx + 2], 16).map_err(|error| error.to_string())?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[test]
fn all_profile_source_and_generator_inputs_fit_checked_source_file_limit() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repo root");
    for profile in all_profiles().expect("profiles") {
        let limit = shell::source_file_limit(&profile).expect("source file limit");
        for locator in profile.source_scope.iter().chain(profile.generator_inputs.iter()) {
            let bytes = safe_read::read_checked_input(repo_root, std::path::Path::new(locator), limit)
                .unwrap_or_else(|error| panic!("profile `{}` input `{locator}`: {error}", profile.profile_id));
            assert!(!bytes.is_empty(), "{locator} should be non-empty");
        }
    }
    let tempdir = tempfile::tempdir().expect("tempdir");
    std::fs::write(tempdir.path().join("oversized.rs"), vec![
        b'x';
        usize::try_from(TEST_FILE_BYTE_LIMIT + 1).expect("len")
    ])
    .expect("write oversized");
    let error =
        safe_read::read_checked_input(tempdir.path(), std::path::Path::new("oversized.rs"), TEST_FILE_BYTE_LIMIT)
            .expect_err("oversized input must fail");
    assert!(error.contains("exceeds"), "{error}");
}

#[test]
fn validate_details_rejects_zero_cases_truncated_outcomes_and_legacy_inventory_drift() {
    let recompute = |details: &mut ReceiptDetails| {
        details.projection_identity = prefixed_identity(&projection_bytes(details).expect("projection bytes"));
    };

    // Valid legacy `octet-pure-core-laws-v1` with empty law inventory still passes.
    let (pure_input, _) = valid_input_for(PROFILE_ID);
    let pure_base = request::evaluate(&request::render(&pure_input).expect("render"), PROFILE_ID)
        .details
        .expect("pure details");
    let mut legacy_empty = pure_base.clone();
    legacy_empty.expected_laws.clear();
    legacy_empty.executed_laws.clear();
    legacy_empty.unexecuted_laws.clear();
    recompute(&mut legacy_empty);
    assert!(validate(&legacy_empty).is_empty(), "{:?}", validate(&legacy_empty));

    // Reject cases_completed == 0 on passing runs (both legacy empty inventory and family profile).
    let (family_input, _) = valid_input_for("octet-evidence-lineage-laws-v1");
    let family_base =
        request::evaluate(&request::render(&family_input).expect("render"), "octet-evidence-lineage-laws-v1")
            .details
            .expect("family details");
    for base in [&legacy_empty, &family_base] {
        let mut zero_cases = base.clone();
        zero_cases.cases_completed = 0;
        recompute(&mut zero_cases);
        assert!(!validate(&zero_cases).is_empty(), "zero cases should fail for {}", base.profile_id);

        for bad_result in ["truncated", "exhausted", "incomplete", "ignored"] {
            let mut bad_outcome = base.clone();
            bad_outcome.result = bad_result.to_owned();
            recompute(&mut bad_outcome);
            assert!(!validate(&bad_outcome).is_empty(), "{bad_result} should fail for {}", base.profile_id);
        }
    }

    // Legacy `octet-pure-core-laws-v1` with empty expected_laws cannot smuggle unexecuted or executed
    // laws.
    let mut legacy_unexecuted = legacy_empty.clone();
    legacy_unexecuted.unexecuted_laws.push(PROPERTY_ID.to_owned());
    recompute(&mut legacy_unexecuted);
    assert!(!validate(&legacy_unexecuted).is_empty());

    let mut legacy_executed = legacy_empty;
    legacy_executed.executed_laws = pure_base.executed_laws;
    recompute(&mut legacy_executed);
    assert!(!validate(&legacy_executed).is_empty());
}
