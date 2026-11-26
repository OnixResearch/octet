#[derive(Clone, Debug, serde::Deserialize)]
struct NativeExport {
    schema_version: String,
    kani_version: String,
    harnesses: Vec<super::harness::NativeUnit>,
}

pub fn check_observation(text: &str) -> super::Evaluation {
    let observation: super::Observation = match serde_json::from_str(text) {
        Ok(obs) => obs,
        Err(error) => return failed_evaluation(format!("Kani producer observation JSON invalid: {error}")),
    };
    let profile = match super::selected_profile(&observation.profile_id) {
        Ok(profile) => profile,
        Err(error) => return failed_evaluation(error),
    };
    let mut diagnostics = Vec::with_capacity(4);
    let mut findings = Vec::with_capacity(4);
    validate_observation_header(&observation, &profile, &mut diagnostics);
    let export = parse_raw_export(&observation.raw_export_json, &mut diagnostics);
    let (mut execution_status, harnesses, totals) =
        reconcile_export(&observation, &profile, export.as_ref(), &mut diagnostics, &mut findings);
    let outcome = super::review::assess_assumptions(
        &observation,
        &profile,
        &mut execution_status,
        &mut diagnostics,
        &mut findings,
    );
    build_evaluation(EvalBundle {
        observation: &observation,
        profile,
        execution_status,
        harnesses,
        totals,
        outcome,
        diagnostics,
        findings,
    })
}

struct EvalBundle<'a> {
    observation: &'a super::Observation,
    profile: super::Profile,
    execution_status: String,
    harnesses: Vec<super::HarnessNormalizedResult>,
    totals: super::PropertyTotals,
    outcome: super::review::AssumptionOutcome,
    diagnostics: Vec<String>,
    findings: Vec<(String, String, String)>,
}

fn parse_raw_export(raw_json: &str, diagnostics: &mut Vec<String>) -> Option<NativeExport> {
    if raw_json.trim().is_empty() {
        diagnostics.push("Kani producer execution produced empty --export-json payload".to_owned());
        return None;
    }
    match serde_json::from_str::<NativeExport>(raw_json) {
        Ok(export) => Some(export),
        Err(error) => {
            diagnostics.push(format!("parse Kani --export-json payload: {error}"));
            None
        }
    }
}

fn build_evaluation(mut bundle: EvalBundle<'_>) -> super::Evaluation {
    let mut selected_harnesses = Vec::with_capacity(bundle.profile.harnesses.len());
    for spec in &bundle.profile.harnesses {
        selected_harnesses.push(spec.name.clone());
    }
    let mut executed_harnesses = Vec::with_capacity(bundle.harnesses.len());
    for harness in &bundle.harnesses {
        executed_harnesses.push(harness.name.clone());
    }
    let summary_json = super::review::render_normalized_summary(
        &super::review::SummaryContext {
            obs: bundle.observation,
            profile: &bundle.profile,
            exec_status: &bundle.execution_status,
            outcome: &bundle.outcome,
            harnesses: &bundle.harnesses,
        },
        &bundle.totals,
    );
    let raw_export_identity = super::prefixed_identity(bundle.observation.raw_export_json.as_bytes());
    let normalized_summary_identity = summary_json
        .as_ref()
        .map_or_else(|| super::prefixed_identity(b""), |s| super::prefixed_identity(s.as_bytes()));
    let details = super::ReceiptDetails {
        schema_version: super::RECEIPT_SCHEMA.to_owned(),
        profile_id: bundle.profile.id,
        profile_identity: bundle.observation.profile_identity.clone(),
        package: bundle.profile.package,
        target: bundle.profile.target,
        command_argv: bundle.observation.command_argv.clone(),
        source_manifest_identity: bundle.observation.source_manifest_identity.clone(),
        lockfile_identity: bundle.observation.lockfile_identity.clone(),
        toolchain: bundle.observation.toolchain.clone(),
        selected_harnesses,
        executed_harnesses,
        harnesses: bundle.harnesses,
        property_totals: bundle.totals,
        execution_status: bundle.execution_status.clone(),
        assumption_status: bundle.outcome.assumption_status,
        is_assumptions_reviewed: bundle.outcome.is_assumptions_reviewed,
        assumption_review_identity: bundle.outcome.record_identity,
        raw_export_identity,
        normalized_summary_identity,
        bounded_log_identity: bundle.observation.bounded_log_identity.clone(),
        evidence_tier: super::EVIDENCE_TIER.to_owned(),
        non_claims: vec![super::REQUIRED_NON_CLAIM.to_owned()],
    };
    bundle.diagnostics.sort();
    bundle.diagnostics.dedup();
    let status = compute_receipt_status(&bundle.execution_status, &bundle.diagnostics, &bundle.findings);
    super::Evaluation {
        details: Some(details),
        normalized_summary_json: summary_json,
        raw_export_json: Some(bundle.observation.raw_export_json.clone()),
        diagnostics: bundle.diagnostics,
        status,
        finding_messages: bundle.findings,
    }
}

fn validate_observation_header(obs: &super::Observation, profile: &super::Profile, diagnostics: &mut Vec<String>) {
    if obs.schema_version != super::OBSERVATION_SCHEMA {
        diagnostics.push(format!("unsupported Kani observation schema `{}`", obs.schema_version));
    }
    if obs.profile_identity != super::EXPECTED_PROFILE_IDENTITY {
        diagnostics.push("Kani producer profile_identity mismatch".to_owned());
    }
    if !super::is_b3_identity(&obs.source_manifest_identity) {
        diagnostics.push("Kani producer requires BLAKE3 source_manifest_identity".to_owned());
    }
    if let Some(expected) = obs.expected_source_manifest_identity.as_deref()
        && expected != obs.source_manifest_identity
    {
        diagnostics.push("Kani source_manifest_identity is stale relative to workspace".to_owned());
    }
    if let Some(expected_lock) = obs.expected_lockfile_identity.as_deref()
        && obs.lockfile_identity.as_deref() != Some(expected_lock)
    {
        diagnostics.push("Kani lockfile_identity is stale or missing".to_owned());
    }
    if obs.toolchain.rustc_release != profile.kani_pin.rustc_release {
        diagnostics.push(format!(
            "Kani rustc `{}` != pinned `{}`",
            obs.toolchain.rustc_release, profile.kani_pin.rustc_release
        ));
    }
    if !obs.toolchain.kani_version.contains(&profile.kani_pin.kani_version) {
        diagnostics.push(format!(
            "Kani version `{}` != pinned `{}`",
            obs.toolchain.kani_version, profile.kani_pin.kani_version
        ));
    }
    if obs.toolchain.solver_mode != profile.kani_pin.solver_mode {
        diagnostics.push(format!(
            "Kani solver_mode `{}` != pinned `{}`",
            obs.toolchain.solver_mode, profile.kani_pin.solver_mode
        ));
    }
}

fn reconcile_export(
    obs: &super::Observation,
    profile: &super::Profile,
    export: Option<&NativeExport>,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) -> (String, Vec<super::HarnessNormalizedResult>, super::PropertyTotals) {
    let mut totals = super::harness::empty_totals();
    if obs.timed_out.unwrap_or(false) {
        diagnostics.push("Kani execution timed out before completion".to_owned());
        return ("timeout".to_owned(), Vec::with_capacity(0), totals);
    }
    if obs.truncated.unwrap_or(false) {
        diagnostics.push("Kani bounded log output was truncated".to_owned());
        return ("incomplete_analysis".to_owned(), Vec::with_capacity(0), totals);
    }
    let Some(export) = export else {
        return ("tool_error".to_owned(), Vec::with_capacity(0), totals);
    };
    if export.schema_version != super::EXPORT_SCHEMA {
        diagnostics.push(format!("unsupported Kani export schema `{}`", export.schema_version));
        return ("unsupported".to_owned(), Vec::with_capacity(0), totals);
    }
    if !export.kani_version.contains(&profile.kani_pin.kani_version) {
        diagnostics
            .push(format!("export Kani `{}` != pinned `{}`", export.kani_version, profile.kani_pin.kani_version));
    }
    let mut by_id: std::collections::BTreeMap<(String, String, String), &super::harness::NativeUnit> =
        std::collections::BTreeMap::new();
    for harness in &export.harnesses {
        let key = (harness.package.clone(), harness.target.clone(), harness.name.clone());
        if by_id.insert(key, harness).is_some() {
            diagnostics.push(format!("duplicate Kani harness export for `{}`", harness.name));
        }
    }
    reconcile_selected_harnesses(profile, &by_id, &mut totals, diagnostics, findings)
}

fn reconcile_selected_harnesses(
    profile: &super::Profile,
    by_id: &std::collections::BTreeMap<(String, String, String), &super::harness::NativeUnit>,
    totals: &mut super::PropertyTotals,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) -> (String, Vec<super::HarnessNormalizedResult>, super::PropertyTotals) {
    let mut status = "passed";
    let mut normalized = Vec::with_capacity(profile.harnesses.len());
    let mut selected_names = std::collections::BTreeSet::new();
    for spec in &profile.harnesses {
        selected_names.insert(spec.name.as_str());
        let key = (profile.package.clone(), profile.target.clone(), spec.name.clone());
        let Some(native) = by_id.get(&key).copied() else {
            diagnostics.push(format!("selected Kani harness `{}` missing from export", spec.name));
            status = super::harness::worse_exec(status, "incomplete_analysis");
            continue;
        };
        let (h_status, h_res) =
            super::harness::normalize_unit(profile, &spec.cover_properties, native, diagnostics, findings);
        super::harness::accumulate_totals(totals, &h_res.totals);
        status = super::harness::worse_exec(status, h_status);
        normalized.push(h_res);
    }
    for key in by_id.keys() {
        if !selected_names.contains(key.2.as_str()) {
            diagnostics.push(format!("export included undeclared Kani harness `{}`", key.2));
            status = super::harness::worse_exec(status, "tool_error");
        }
    }
    if normalized.is_empty() || totals.total == 0 {
        diagnostics.push("Kani export reported zero properties; passing cannot be inferred".to_owned());
        status = super::harness::worse_exec(status, "incomplete_analysis");
    }
    (status.to_owned(), normalized, *totals)
}

fn compute_receipt_status(exec_status: &str, diagnostics: &[String], findings: &[(String, String, String)]) -> String {
    if !diagnostics.is_empty()
        || findings.iter().any(|(_, sev, _)| sev == "error")
        || !matches!(exec_status, "passed" | "manual_review_required")
    {
        "failed".to_owned()
    } else if findings.iter().any(|(_, sev, _)| sev == "manual_review") || exec_status == "manual_review_required" {
        "manual_review_required".to_owned()
    } else {
        "passed".to_owned()
    }
}

fn failed_evaluation(message: String) -> super::Evaluation {
    super::Evaluation {
        details: None,
        normalized_summary_json: None,
        raw_export_json: None,
        diagnostics: vec![message],
        status: "failed".to_owned(),
        finding_messages: Vec::with_capacity(0),
    }
}
