pub struct AssumptionOutcome {
    pub assumption_status: String,
    pub is_assumptions_reviewed: bool,
    pub record_identity: Option<String>,
}

pub struct SummaryContext<'a> {
    pub obs: &'a super::Observation,
    pub profile: &'a super::Profile,
    pub exec_status: &'a str,
    pub outcome: &'a AssumptionOutcome,
    pub harnesses: &'a [super::HarnessNormalizedResult],
}

pub fn assess_assumptions(
    obs: &super::Observation,
    profile: &super::Profile,
    execution_status: &mut String,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) -> AssumptionOutcome {
    let Some(record) = obs.assumption_review.as_ref() else {
        if profile.is_require_assumption_review && *execution_status == "passed" {
            findings.push((
                "assumption-review-incomplete".to_owned(),
                "manual_review".to_owned(),
                "Kani producer run requires separate identity-bound assumption review".to_owned(),
            ));
            "manual_review_required".clone_into(execution_status);
        }
        return AssumptionOutcome {
            assumption_status: "unreviewed".to_owned(),
            is_assumptions_reviewed: false,
            record_identity: None,
        };
    };
    verify_binding(record, obs, profile, diagnostics);
    let record_bytes = serde_json::to_vec(record).unwrap_or_default();
    let record_identity = Some(super::prefixed_identity(&record_bytes));
    if record.assumption_status == "vacuous" {
        diagnostics.push("Kani assumption review marked assumptions vacuous".to_owned());
        "failed".clone_into(execution_status);
    } else if (!record.is_assumptions_reviewed || record.assumption_status != "reviewed")
        && *execution_status == "passed"
    {
        findings.push((
            "assumption-review-incomplete".to_owned(),
            "manual_review".to_owned(),
            "Kani assumption review record is incomplete".to_owned(),
        ));
        "manual_review_required".clone_into(execution_status);
    }
    AssumptionOutcome {
        assumption_status: record.assumption_status.clone(),
        is_assumptions_reviewed: record.is_assumptions_reviewed,
        record_identity,
    }
}

fn verify_binding(
    record: &super::AssumptionReviewRecord,
    obs: &super::Observation,
    profile: &super::Profile,
    diagnostics: &mut Vec<String>,
) {
    if record.schema_version != super::ASSUMPTION_REVIEW_SCHEMA {
        diagnostics.push(format!("unsupported Kani assumption review schema `{}`", record.schema_version));
    }
    if record.source_manifest_identity != obs.source_manifest_identity {
        diagnostics.push("Kani assumption review source_manifest_identity does not match current source".to_owned());
    }
    let expected: std::collections::BTreeSet<&str> = profile.harnesses.iter().map(|h| h.name.as_str()).collect();
    let reviewed: std::collections::BTreeSet<&str> = record.harness_names.iter().map(String::as_str).collect();
    if expected != reviewed {
        diagnostics.push("Kani assumption review harness_names do not match profile harnesses".to_owned());
    }
}

pub fn render_normalized_summary(ctx: &SummaryContext<'_>, totals: &super::PropertyTotals) -> Option<String> {
    let first = ctx.harnesses.first()?;
    let verdict = if ctx.exec_status == "passed" && totals.failed == 0 {
        "passed"
    } else {
        "failed"
    };
    let unwind_status = if totals.unwind_failed == 0 && totals.unwind_passed > 0 {
        "passed"
    } else {
        "bounded_only"
    };
    let value = serde_json::json!({
        "schema_version": super::KANI_SUMMARY_SCHEMA,
        "harness": {
            "package": ctx.profile.package,
            "target": ctx.profile.target,
            "path": "src/lib.rs",
            "name": first.name,
        },
        "source_identity": ctx.obs.source_manifest_identity,
        "toolchain": {
            "kani": ctx.obs.toolchain.kani_version,
            "rustc": ctx.obs.toolchain.rustc_release,
            "cbmc": ctx.obs.toolchain.cbmc_version,
            "solver": ctx.obs.toolchain.solver_identity,
        },
        "verdict": verdict,
        "assumptions_reviewed": ctx.outcome.is_assumptions_reviewed,
        "assumption_status": ctx.outcome.assumption_status,
        "unwind": { "status": unwind_status },
        "properties": [{
            "name": first.name,
            "property_class": "boundary-panic",
            "status": verdict,
        }],
        "claims": {
            "coverage": "bounded",
            "property_evidence": false,
        }
    });
    serde_json::to_string_pretty(&value).ok()
}
