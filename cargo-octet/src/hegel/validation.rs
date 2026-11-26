const EXPECTED_EVIDENCE_TIER: &str = "transcript_assertion";
const EXPECTED_VERIFICATION_ROLE: &str = "recorded_only";
const RESULT_PASSED: &str = "passed";
const RESULT_COUNTEREXAMPLE: &str = "counterexample";

pub(super) fn evaluate(input: &super::request::Input, profile: &super::Profile) -> super::Evaluation {
    let mut diagnostics = super::profile_validation::validate(profile);
    diagnostics.extend(validate_input(input, profile));
    let summary: super::ProducerSummary = match serde_json::from_str(&input.summary_text) {
        Ok(value) => value,
        Err(error) => return super::request::failed(vec![format!("parse dedicated Hegel producer summary: {error}")]),
    };
    diagnostics.extend(validate_summary(&summary, input, profile));
    if !diagnostics.is_empty() {
        return super::request::failed(diagnostics);
    }
    let source_manifest_identity =
        match super::request::manifest_identity(&input.source_manifest, "Hegel source manifest") {
            Ok(value) => value,
            Err(error) => return super::request::failed(vec![error]),
        };
    let generator_manifest_identity =
        match super::request::manifest_identity(&input.generator_manifest, "Hegel generator manifest") {
            Ok(value) => value,
            Err(error) => return super::request::failed(vec![error]),
        };
    let mut details = super::ReceiptDetails {
        schema_version: super::RECEIPT_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: super::PROFILE_IDENTITY.to_owned(),
        law_catalog_identity: super::LAW_CATALOG_IDENTITY.to_owned(),
        property_id: summary.property_id.clone(),
        linked_ids: summary.linked_ids.clone(),
        source_scope: summary.source_scope.clone(),
        producer: profile.producer.clone(),
        settings: summary.settings.clone(),
        generator: summary.generator.clone(),
        command_argv: profile.command_argv.clone(),
        cargo_lock_identity: input.cargo_lock_identity.clone(),
        package_manifest_identity: input.package_manifest_identity.clone(),
        source_manifest_identity,
        generator_manifest_identity,
        summary_identity: super::prefixed_identity(input.summary_text.as_bytes()),
        bounded_log_identity: super::prefixed_identity(&input.bounded_log),
        counterexample_identity: input.counterexample.as_ref().map(|bytes| super::prefixed_identity(bytes)),
        result: summary.result,
        cases_completed: summary.cases_completed,
        issue_codes: super::normalize_issue_codes(&summary.issue_codes),
        projection_identity: String::new(),
        evidence_tier: profile.evidence_tier.clone(),
        verification_role: profile.verification_role.clone(),
        non_claims: profile.non_claims.clone(),
        expected_laws: summary.expected_laws,
        executed_laws: summary.executed_laws,
        unexecuted_laws: summary.unexecuted_laws,
    };
    details.projection_identity = match super::projection_bytes(&details) {
        Ok(bytes) => super::prefixed_identity(&bytes),
        Err(error) => return super::request::failed(vec![error]),
    };
    super::Evaluation {
        details: Some(details),
        diagnostics: Vec::new(),
    }
}

pub(super) fn validate_details(details: &super::ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let registered = super::selected_profile(&details.profile_id).ok();
    let is_command_matched = registered.as_ref().is_some_and(|profile| profile.command_argv == details.command_argv)
        && super::profile_validation::is_valid_fixed_command(&details.command_argv);
    if details.schema_version != super::RECEIPT_SCHEMA
        || registered.is_none()
        || details.profile_identity != super::PROFILE_IDENTITY
        || details.law_catalog_identity != super::LAW_CATALOG_IDENTITY
    {
        diagnostics.push("profiled Hegel receipt schema or checked identity is unsupported".to_owned());
    }
    if details.property_id.trim().is_empty()
        || details.linked_ids.is_empty()
        || details.source_scope.trim().is_empty()
        || details.producer.package != "cargo-octet"
        || details.producer.hegel_version != "0.14.27"
        || details.generator.content_hash != super::LAW_CATALOG_IDENTITY
        || details.settings.case_budget == 0
        || !is_command_matched
        || details.evidence_tier != EXPECTED_EVIDENCE_TIER
        || details.verification_role != EXPECTED_VERIFICATION_ROLE
        || !details.non_claims.iter().any(|claim| claim == super::REQUIRED_NON_CLAIM)
    {
        diagnostics.push("profiled Hegel receipt authority or evidence boundary is unsupported".to_owned());
    }
    validate_details_identities_and_inventory(details, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_details_identities_and_inventory(details: &super::ReceiptDetails, diagnostics: &mut Vec<String>) {
    for identity in [
        &details.cargo_lock_identity,
        &details.package_manifest_identity,
        &details.source_manifest_identity,
        &details.generator_manifest_identity,
        &details.summary_identity,
        &details.bounded_log_identity,
        &details.projection_identity,
    ] {
        validate_b3(identity, "profiled Hegel receipt", diagnostics);
    }
    if let Some(identity) = details.counterexample_identity.as_ref() {
        validate_b3(identity, "profiled Hegel counterexample", diagnostics);
    }
    if details.issue_codes != super::normalize_issue_codes(&details.issue_codes) {
        diagnostics.push("profiled Hegel receipt issue codes are not canonical".to_owned());
    }
    match (details.result.as_str(), &details.counterexample_identity) {
        (RESULT_PASSED, None) if details.cases_completed == details.settings.case_budget => {}
        (RESULT_COUNTEREXAMPLE, Some(_)) if details.cases_completed <= details.settings.case_budget => {}
        _ => diagnostics.push("profiled Hegel receipt outcome or counterexample is inconsistent".to_owned()),
    }
    let has_inventory =
        !details.expected_laws.is_empty() || !details.executed_laws.is_empty() || !details.unexecuted_laws.is_empty();
    if details.profile_id != super::PROFILE_ID || has_inventory {
        if details.expected_laws != expected_law_ids(&details.profile_id) {
            diagnostics.push("profiled Hegel receipt expected_laws do not match checked law catalog".to_owned());
        }
        validate_law_inventory(
            (&details.result, details.settings.case_budget),
            &details.expected_laws,
            &details.executed_laws,
            &details.unexecuted_laws,
            diagnostics,
        );
    }
    match super::projection_bytes(details) {
        Ok(bytes) if super::prefixed_identity(&bytes) == details.projection_identity => {}
        Ok(_) => diagnostics.push("profiled Hegel projection identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
}

fn validate_input(input: &super::request::Input, profile: &super::Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let is_summary_too_large =
        !u64::try_from(input.summary_text.len()).is_ok_and(|len| len <= profile.limits.max_summary_bytes);
    let is_ce_too_large = input.counterexample.as_ref().is_some_and(|bytes| {
        !u64::try_from(bytes.len()).is_ok_and(|len| len <= profile.limits.max_counterexample_bytes)
    });
    if is_summary_too_large
        || input.bounded_log.len() > profile.limits.max_log_bytes
        || input.source_manifest.len() != profile.source_scope.len()
        || input.generator_manifest.len() != profile.generator_inputs.len()
        || is_ce_too_large
    {
        diagnostics.push("profiled Hegel input exceeds a checked profile limit".to_owned());
    }
    validate_manifest(&input.source_manifest, &profile.source_scope, "source", &mut diagnostics);
    validate_manifest(&input.generator_manifest, &profile.generator_inputs, "generator", &mut diagnostics);
    validate_b3(&input.cargo_lock_identity, "Cargo lock", &mut diagnostics);
    validate_b3(&input.package_manifest_identity, "package manifest", &mut diagnostics);
    diagnostics
}

fn validate_manifest(
    manifest: &[super::FileIdentity],
    expected: &[String],
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let expected = expected.iter().collect::<std::collections::BTreeSet<_>>();
    let mut observed = std::collections::BTreeSet::new();
    for file in manifest {
        if super::profile_validation::unsafe_relative_path(&file.locator)
            || file.bytes == 0
            || !observed.insert(&file.locator)
        {
            diagnostics.push(format!("Hegel {label} manifest contains an unsafe, empty, or duplicate entry"));
        }
        validate_b3(&file.identity, &format!("Hegel {label} manifest"), diagnostics);
    }
    if observed != expected {
        diagnostics.push(format!("Hegel {label} manifest does not match the checked profile"));
    }
}

fn validate_summary(
    summary: &super::ProducerSummary,
    input: &super::request::Input,
    profile: &super::Profile,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let source_identity = super::request::manifest_identity(&input.source_manifest, "Hegel source manifest").ok();
    let generator_identity =
        super::request::manifest_identity(&input.generator_manifest, "Hegel generator manifest").ok();
    let expected_laws = expected_law_ids(&profile.profile_id);
    if summary.schema_version != super::SUMMARY_SCHEMA
        || summary.profile_id != profile.profile_id
        || summary.profile_identity != super::PROFILE_IDENTITY
        || summary.package != profile.producer.package
        || summary.package_version != profile.producer.package_version
        || summary.hegel_version != profile.producer.hegel_version
        || summary.backend != profile.producer.backend
        || summary.property_catalog != profile.property_catalog
        || expected_laws.is_empty()
        || !expected_laws.contains(&summary.property_id)
        || summary.expected_laws != expected_laws
        || summary.source_scope != profile.source_scope.join(",")
        || summary.settings != profile.settings
        || summary.generator.content_hash != super::LAW_CATALOG_IDENTITY
        || source_identity.as_ref() != Some(&summary.source_manifest_identity)
        || generator_identity.as_ref() != Some(&summary.generator_manifest_identity)
        || summary.cargo_lock_identity != input.cargo_lock_identity
        || summary.package_manifest_identity != input.package_manifest_identity
    {
        diagnostics.push("Hegel producer summary does not match measured profile or source facts".to_owned());
    }
    if summary.cases_completed > profile.settings.case_budget
        || summary.issue_codes.len() > profile.limits.max_issue_codes
        || summary.issue_codes != super::normalize_issue_codes(&summary.issue_codes)
    {
        diagnostics.push("Hegel producer summary result or issue-code limit is invalid".to_owned());
    }
    validate_law_inventory(
        (&summary.result, profile.settings.case_budget),
        &summary.expected_laws,
        &summary.executed_laws,
        &summary.unexecuted_laws,
        &mut diagnostics,
    );
    match (summary.result.as_str(), &summary.counterexample_identity, &input.counterexample) {
        (RESULT_PASSED, None, None) if summary.cases_completed == profile.settings.case_budget => {}
        (RESULT_COUNTEREXAMPLE, Some(declared), Some(bytes)) if super::prefixed_identity(bytes) == *declared => {}
        _ => diagnostics.push("Hegel producer summary result and retained counterexample are inconsistent".to_owned()),
    }
    diagnostics
}

fn expected_law_ids(profile_id: &str) -> Vec<String> {
    let mut ids = super::laws_for_profile(profile_id)
        .map(|laws| laws.into_iter().map(|law| law.law_id).collect::<Vec<_>>())
        .unwrap_or_default();
    ids.sort();
    ids
}

fn validate_law_inventory(
    outcome: (&str, u64),
    expected_laws: &[String],
    executed_laws: &[super::LawExecutionRecord],
    unexecuted_laws: &[String],
    diagnostics: &mut Vec<String>,
) {
    let (result, case_budget) = outcome;
    if expected_laws.is_empty() || executed_laws.is_empty() {
        diagnostics.push("Hegel profile execution has zero expected or executed laws".to_owned());
        return;
    }
    let expected_set = expected_laws.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    let mut covered = std::collections::BTreeSet::new();
    for record in executed_laws {
        if !expected_set.contains(record.law_id.as_str()) || !covered.insert(record.law_id.as_str()) {
            diagnostics.push("Hegel executed law record is unexpected or duplicated".to_owned());
        }
        if record.cases_completed == 0 && result == RESULT_PASSED {
            diagnostics.push("Hegel passing law executed zero generated cases".to_owned());
        }
        if result == RESULT_PASSED && (record.status != RESULT_PASSED || record.cases_completed != case_budget) {
            diagnostics.push("Hegel passing profile contains non-passing or incomplete law".to_owned());
        }
    }
    for skipped in unexecuted_laws {
        if !expected_set.contains(skipped.as_str()) || !covered.insert(skipped.as_str()) {
            diagnostics.push("Hegel unexecuted law record is unexpected or duplicated".to_owned());
        }
    }
    if covered != expected_set {
        diagnostics.push("Hegel law inventory does not account for every expected law".to_owned());
    }
    if result == RESULT_PASSED && !unexecuted_laws.is_empty() {
        diagnostics.push("Hegel passing profile cannot leave expected laws unexecuted".to_owned());
    }
    if result == RESULT_COUNTEREXAMPLE && !executed_laws.iter().any(|record| record.status == RESULT_COUNTEREXAMPLE) {
        diagnostics.push("Hegel counterexample profile lacks failing law record".to_owned());
    }
}

fn validate_b3(value: &str, label: &str, diagnostics: &mut Vec<String>) {
    let is_valid = value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == blake3::OUT_LEN * 2
            && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    if !is_valid {
        diagnostics.push(format!("{label} identity is not canonical BLAKE3"));
    }
}
