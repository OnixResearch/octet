struct CellCheckAccumulator {
    summaries: Vec<super::MatrixCellSummary>,
    diagnostics: Vec<String>,
    findings: Vec<String>,
}

pub fn run_input(text: &str, requested_profile: Option<&str>) -> super::Evaluation {
    let parsed: super::Input = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => {
            return super::Evaluation {
                details: None,
                diagnostics: vec![format!("dynamic qualification input is not valid JSON: {error}")],
                status: "failed".to_owned(),
                findings: Vec::with_capacity(0),
            };
        }
    };
    check_parsed(&parsed, requested_profile)
}

fn check_parsed(input: &super::Input, requested_profile: Option<&str>) -> super::Evaluation {
    let mut acc = CellCheckAccumulator {
        summaries: Vec::with_capacity(input.observed_runs.len()),
        diagnostics: Vec::with_capacity(4),
        findings: Vec::with_capacity(4),
    };
    if let Some(req) = requested_profile
        && req != input.profile_id
    {
        acc.diagnostics
            .push(format!("dynamic `--profile {req}` does not match input profile `{}`", input.profile_id));
    }
    let profile = match super::selected_profile(&input.profile_id) {
        Ok(profile) => profile,
        Err(error) => {
            acc.diagnostics.push(error);
            return super::Evaluation {
                details: None,
                diagnostics: acc.diagnostics,
                status: "failed".to_owned(),
                findings: acc.findings,
            };
        }
    };
    validate_envelope_header(input, &profile, &mut acc.diagnostics);
    let status = check_cells(&profile, input, &mut acc);
    let details = build_details(input, &profile, status, acc.summaries);
    acc.diagnostics.sort();
    acc.diagnostics.dedup();
    let final_status = if !acc.diagnostics.is_empty() && status == "qualified_passed" {
        "failed".to_owned()
    } else {
        status.to_owned()
    };
    super::Evaluation {
        details: Some(details),
        diagnostics: acc.diagnostics,
        status: final_status,
        findings: acc.findings,
    }
}

fn validate_envelope_header(input: &super::Input, profile: &super::Profile, diagnostics: &mut Vec<String>) {
    if !matches!(input.schema_version.as_str(), super::QUALIFICATION_INPUT_SCHEMA | super::MATRIX_RECEIPT_SCHEMA) {
        diagnostics.push(format!("unsupported dynamic qualification input schema `{}`", input.schema_version));
    }
    if input.profile_identity != super::EXPECTED_PROFILE_IDENTITY {
        diagnostics.push(format!(
            "dynamic qualification profile identity mismatch: expected `{}`, got `{}`",
            super::EXPECTED_PROFILE_IDENTITY,
            input.profile_identity
        ));
    }
    if !super::is_b3_identity(&input.source_manifest_identity) {
        diagnostics.push("dynamic qualification requires a BLAKE3 source_manifest_identity".to_owned());
    }
    if let Some(expected_source) = input.expected_source_manifest_identity.as_deref()
        && expected_source != input.source_manifest_identity
    {
        diagnostics.push("dynamic qualification source_manifest_identity is stale relative to workspace".to_owned());
    }
    if let Some(expected_lock) = input.expected_lockfile_identity.as_deref()
        && input.lockfile_identity.as_deref() != Some(expected_lock)
    {
        diagnostics.push("dynamic qualification lockfile_identity is stale or missing".to_owned());
    }
    if input.compiler_probe.tool_kind != profile.tool {
        diagnostics
            .push(format!("probed tool kind `{}` != profile tool `{}`", input.compiler_probe.tool_kind, profile.tool));
    }
    if input.command_argv != profile.command_argv {
        diagnostics.push("dynamic qualification command_argv does not match checked profile".to_owned());
    }
    validate_compiler_pin(input, profile, diagnostics);
}

fn validate_compiler_pin(input: &super::Input, profile: &super::Profile, diagnostics: &mut Vec<String>) {
    let Some(miri) = profile.miri.as_ref() else {
        return;
    };
    if input.compiler_probe.rustc_release != miri.toolchain_pin {
        diagnostics.push(format!(
            "wrong compiler `{}`; requires `{}`",
            input.compiler_probe.rustc_release, miri.toolchain_pin
        ));
    }
    if input.compiler_probe.host_triple != miri.target_triple {
        diagnostics
            .push(format!("wrong target `{}`; requires `{}`", input.compiler_probe.host_triple, miri.target_triple));
    }
    if input.compiler_probe.sysroot_identity.as_deref().is_none_or(|id| !super::is_b3_identity(id)) {
        diagnostics.push("Miri qualification requires a BLAKE3 sysroot_identity".to_owned());
    }
}

fn check_cells(profile: &super::Profile, input: &super::Input, acc: &mut CellCheckAccumulator) -> &'static str {
    if input.observed_runs.is_empty() {
        acc.diagnostics.push("dynamic qualification received zero cell observations".to_owned());
        return "incomplete";
    }
    let declared_cells: std::collections::BTreeSet<&str> =
        profile.matrix_cells.iter().map(|spec| spec.cell_id.as_str()).collect();
    for cell in &input.observed_runs {
        if !declared_cells.contains(cell.cell_id.as_str()) {
            acc.diagnostics.push(format!("unexpected undeclared matrix cell `{}`", cell.cell_id));
        }
    }
    let mut overall = "qualified_passed";
    for cell_spec in &profile.matrix_cells {
        let matches: Vec<&super::CellObservation> =
            input.observed_runs.iter().filter(|cell| cell.cell_id == cell_spec.cell_id).collect();
        if matches.len() > 1 {
            acc.diagnostics.push(format!("duplicate matrix cell result for `{}`", cell_spec.cell_id));
            overall = worse_status(overall, "failed");
            continue;
        }
        let Some(run) = matches.first().copied() else {
            if cell_spec.is_required {
                acc.diagnostics.push(format!("missing required matrix cell `{}`", cell_spec.cell_id));
                overall = worse_status(overall, "incomplete");
            }
            continue;
        };
        let cell_status = check_single_cell(profile, cell_spec.target_triple.as_str(), run, acc);
        overall = worse_status(overall, cell_status);
        acc.summaries.push(super::MatrixCellSummary {
            cell_id: run.cell_id.clone(),
            target_triple: run.target_triple.clone(),
            status: cell_status.to_owned(),
            tests_started: run.tests_started,
            tests_passed: run.tests_passed,
            tests_failed: run.tests_failed,
            tests_ignored: run.tests_ignored,
            permutations_explored: run.permutations_explored.unwrap_or(0),
            bounded_log_identity: run.bounded_log_identity.clone(),
        });
    }
    overall
}

fn check_single_cell(
    profile: &super::Profile,
    expected_target: &str,
    run: &super::CellObservation,
    acc: &mut CellCheckAccumulator,
) -> &'static str {
    let diag_start = acc.diagnostics.len();
    let finding_start = acc.findings.len();
    if run.target_triple != expected_target {
        acc.diagnostics
            .push(format!("cell `{}` target `{}` != `{expected_target}`", run.cell_id, run.target_triple));
    }
    if run.timed_out.unwrap_or(false) || run.disposition == "timeout" {
        acc.diagnostics.push(format!("cell `{}` timed out before completion", run.cell_id));
        return "timeout";
    }
    if run.disposition == "unsupported" {
        acc.diagnostics.push(format!("cell `{}` reported `unsupported`", run.cell_id));
        return "unsupported";
    }
    if run.disposition == "unavailable" {
        acc.diagnostics.push(format!("cell `{}` reported `unavailable`", run.cell_id));
        return "unavailable";
    }
    if run.truncated.unwrap_or(false) || run.disposition == "incomplete" {
        acc.diagnostics.push(format!("cell `{}` output was truncated or incomplete", run.cell_id));
        return "incomplete";
    }
    validate_cell_tests(profile, run, acc);
    validate_cell_tool_rules(profile, run, &mut acc.diagnostics);
    if run.disposition == "failed" || run.tests_failed > 0 || run.exit_code != 0 || acc.findings.len() > finding_start {
        if let Some(messages) = run.failure_messages.as_ref() {
            acc.findings.extend(messages.iter().cloned());
        }
        return "failed";
    }
    if acc.diagnostics.len() > diag_start {
        return "failed";
    }
    "qualified_passed"
}

fn validate_cell_tests(profile: &super::Profile, run: &super::CellObservation, acc: &mut CellCheckAccumulator) {
    if run.tests_started == 0 || run.executed_tests.is_empty() {
        acc.diagnostics.push(format!("cell `{}` executed zero tests", run.cell_id));
    }
    if run.tests_passed == 0 && run.tests_ignored > 0 {
        acc.diagnostics.push(format!("cell `{}` had all tests ignored or skipped", run.cell_id));
    }
    let mut unique = std::collections::BTreeSet::new();
    for test_name in &run.executed_tests {
        if !unique.insert(test_name.as_str()) {
            acc.diagnostics
                .push(format!("cell `{}` recorded duplicate test execution `{test_name}`", run.cell_id));
        }
    }
    for required in &profile.test_inventory {
        if !unique.contains(required.as_str()) {
            acc.diagnostics.push(format!("cell `{}` omitted required inventory test `{required}`", run.cell_id));
        }
    }
    if run.tests_failed > 0 && run.failure_messages.as_ref().is_none_or(Vec::is_empty) {
        acc.findings.push(format!("cell `{}` reported {} failing tests", run.cell_id, run.tests_failed));
    }
}

fn validate_cell_tool_rules(profile: &super::Profile, run: &super::CellObservation, diagnostics: &mut Vec<String>) {
    if let Some(miri) = profile.miri.as_ref()
        && run.miri_flags.as_deref() != Some(miri.miri_flags.as_slice())
    {
        diagnostics.push(format!("cell `{}` Miri flags do not match pinned profile flags", run.cell_id));
    }
    if let Some(loom) = profile.loom.as_ref() {
        let is_entered = run.model_entered.unwrap_or(false);
        let is_completed = run.model_completed.unwrap_or(false);
        let perms = run.permutations_explored.unwrap_or(0);
        if !is_entered || !is_completed || perms == 0 {
            diagnostics.push(format!("cell `{}` lacks required Loom model-entry/completion evidence", run.cell_id));
        }
        if run.rustflags.as_deref() != Some(loom.rustflags.as_str())
            || run.encoded_rustflags.as_deref() != Some(loom.encoded_rustflags.as_str())
        {
            diagnostics.push(format!("cell `{}` Loom rustflags or CARGO_ENCODED_RUSTFLAGS mismatch", run.cell_id));
        }
    }
}

fn worse_status<'a>(current: &'a str, candidate: &'a str) -> &'a str {
    let rank = |status: &str| match status {
        "qualified_passed" => 0_u8,
        "incomplete" => 1,
        "unsupported" => 2,
        "unavailable" => 3,
        "timeout" => 4,
        _ => 5,
    };
    if rank(candidate) > rank(current) {
        candidate
    } else {
        current
    }
}

fn build_details(
    input: &super::Input,
    profile: &super::Profile,
    status: &str,
    matrix_cells: Vec<super::MatrixCellSummary>,
) -> super::ReceiptDetails {
    let first = input.observed_runs.first();
    super::ReceiptDetails {
        schema_version: super::RECEIPT_SCHEMA.to_owned(),
        profile_id: profile.id.clone(),
        profile_identity: input.profile_identity.clone(),
        tool: profile.tool.clone(),
        package: profile.package.clone(),
        target: profile.target.clone(),
        command_argv: profile.command_argv.clone(),
        source_manifest_identity: input.source_manifest_identity.clone(),
        lockfile_identity: input.lockfile_identity.clone(),
        compiler_probe: input.compiler_probe.clone(),
        qualification_status: status.to_owned(),
        matrix_cells,
        tests_started: first.map_or(0, |run| run.tests_started),
        tests_passed: first.map_or(0, |run| run.tests_passed),
        tests_failed: first.map_or(0, |run| run.tests_failed),
        tests_ignored: first.map_or(0, |run| run.tests_ignored),
        tests_filtered_out: first.map_or(0, |run| run.tests_filtered_out),
        executed_tests: first.map_or_else(|| Vec::with_capacity(0), |run| run.executed_tests.clone()),
        filters: first.and_then(|run| run.filters.clone()).unwrap_or_else(|| Vec::with_capacity(0)),
        is_model_entered: first.and_then(|run| run.model_entered).unwrap_or(false),
        is_model_completed: first.and_then(|run| run.model_completed).unwrap_or(false),
        permutations_explored: first.and_then(|run| run.permutations_explored).unwrap_or(0),
        bounded_log_identity: first
            .map_or_else(|| super::prefixed_identity(b""), |run| run.bounded_log_identity.clone()),
        evidence_tier: super::EVIDENCE_TIER.to_owned(),
        non_claims: vec![super::REQUIRED_NON_CLAIM.to_owned()],
    }
}
