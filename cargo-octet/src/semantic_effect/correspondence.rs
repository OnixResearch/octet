//! Profile, unit summary, and selected-function correspondence validation (#17).

pub(super) fn validate_profile(
    profile: &super::model::MirClosureProfile,
    diagnostics: &mut Vec<super::model::ProfileDiagnostic>,
) {
    if profile.schema_version != super::model::PROFILE_SCHEMA_VERSION {
        push_diag(diagnostics, "profile_schema_invalid", "unsupported semantic effect profile schema", None);
    }
    if profile.claim_tier != super::model::CLAIM_TIER {
        push_diag(
            diagnostics,
            "claim_tier_invalid",
            &format!(
                "semantic profile claim_tier `{}` cannot claim `{}`",
                profile.claim_tier,
                super::model::CLAIM_TIER
            ),
            None,
        );
    }
    if profile.compiler_stage != super::model::COMPILER_STAGE {
        push_diag(diagnostics, "compiler_stage_invalid", "unexpected compiler stage in semantic profile", None);
    }
    if profile.non_claims.is_empty() || profile.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        push_diag(diagnostics, "profile_non_claims_invalid", "semantic profile requires non-empty non_claims", None);
    }
}

pub(super) fn validate_unit_summary(
    summary: &octet_architecture_ir::semantic::UnitSummary,
    profile: &super::model::MirClosureProfile,
    diagnostics: &mut Vec<super::model::ProfileDiagnostic>,
) {
    if summary.schema_version != octet_architecture_ir::semantic::SEMANTIC_MIR_UNIT_SCHEMA_VERSION {
        push_diag(diagnostics, "unit_schema_invalid", "unsupported semantic MIR unit schema", None);
    }
    if summary.compiler_stage != profile.compiler_stage {
        push_diag(diagnostics, "compiler_stage_mismatch", "unit MIR stage does not match profile", None);
    }
    match octet_architecture_ir::semantic::compute_unit_summary_blake3(summary) {
        Ok(expected) if expected == summary.summary_blake3 => {}
        _ => push_diag(diagnostics, "unit_identity_mismatch", "semantic MIR unit summary digest mismatch", None),
    }
    if summary.is_budget_exhausted || !summary.skipped_bodies.is_empty() {
        push_diag(diagnostics, "analysis_budget_exhausted", "semantic MIR analysis budget exhausted", None);
    }
    if summary.bodies.len() > profile.budgets.max_bodies {
        push_diag(diagnostics, "body_budget_exceeded", "unit body count exceeds profile budget", None);
    }
}

pub(super) fn verify_links(
    summary: &octet_architecture_ir::semantic::UnitSummary,
    addresses: &[crate::effect_closure::FunctionAddressRecord],
    profile: &super::model::MirClosureProfile,
    diagnostics: &mut Vec<super::model::ProfileDiagnostic>,
) -> (super::model::CoverageReport, std::collections::BTreeMap<String, String>) {
    let address_by_name = addresses
        .iter()
        .map(|record| (record.name.clone(), record.function_address.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let observed_short_names = summary
        .bodies
        .iter()
        .map(|body| body.qualified_path.rsplit("::").next().unwrap_or(&body.qualified_path).to_owned())
        .collect::<std::collections::BTreeSet<_>>();

    let mut missing = Vec::with_capacity(addresses.len());
    for record in addresses {
        if !observed_short_names.contains(&record.name)
            && !summary.bodies.iter().any(|body| body.qualified_path == record.name)
        {
            missing.push(record.name.clone());
            push_diag(
                diagnostics,
                "correspondence_missing_body",
                &format!("selected function `{}` has no observed MIR body", record.name),
                Some(&record.name),
            );
        }
    }
    for body in &summary.bodies {
        if body.basic_block_count > profile.budgets.max_blocks_per_body || body.step_count > profile.budgets.max_steps {
            push_diag(
                diagnostics,
                "body_budget_exceeded",
                &format!("body `{}` exceeded MIR block or step budget", body.qualified_path),
                Some(&body.qualified_path),
            );
        }
    }
    let is_complete = !summary.is_budget_exhausted
        && summary.skipped_bodies.is_empty()
        && missing.is_empty()
        && !summary.bodies.is_empty();
    let report = super::model::CoverageReport {
        is_complete,
        expected_function_count: addresses.len(),
        observed_body_count: summary.bodies.len(),
        is_budget_exhausted: summary.is_budget_exhausted,
        missing_functions: missing,
        skipped_bodies: summary.skipped_bodies.clone(),
    };
    (report, address_by_name)
}

fn push_diag(
    diagnostics: &mut Vec<super::model::ProfileDiagnostic>,
    code: &str,
    message: &str,
    item_path: Option<&str>,
) {
    diagnostics.push(super::model::ProfileDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        item_path: item_path.map(str::to_owned),
    });
}
