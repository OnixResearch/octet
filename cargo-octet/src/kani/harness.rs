#[derive(Clone, Debug, serde::Deserialize)]
pub struct NativeUnit {
    pub package: String,
    pub target: String,
    pub name: String,
    pub verification_status: String,
    pub unwind_bound: u64,
    pub properties: Vec<NativeProperty>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct NativeProperty {
    pub name: String,
    pub check_type: String,
    pub status: String,
    pub description: Option<String>,
    pub location: Option<super::CheckLocation>,
}

struct ConstraintCheck<'a> {
    expected_covers: &'a [String],
    native: &'a NativeUnit,
    totals: &'a super::PropertyTotals,
    observed_covers: &'a std::collections::BTreeMap<&'a str, &'a str>,
}

pub fn normalize_unit(
    profile: &super::Profile,
    expected_covers: &[String],
    native: &NativeUnit,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) -> (&'static str, super::HarnessNormalizedResult) {
    let mut totals = empty_totals();
    let mut locations = Vec::with_capacity(native.properties.len());
    let mut observed_covers: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for prop in &native.properties {
        totals.total = totals.total.saturating_add(1);
        classify_property(prop, &mut totals, &mut observed_covers, diagnostics, findings);
        if let Some(loc) = prop.location.as_ref() {
            locations.push(loc.clone());
        }
    }
    let mut status = status_from_wire(native, diagnostics, findings);
    if totals.total == 0 && status == "passed" {
        diagnostics.push(format!("harness `{}` exported zero properties", native.name));
        status = "incomplete_analysis";
    } else if status == "passed" {
        status = verify_constraints(
            profile,
            &ConstraintCheck {
                expected_covers,
                native,
                totals: &totals,
                observed_covers: &observed_covers,
            },
            diagnostics,
        );
    }
    if totals.failed > 0 || totals.unreachable > 0 {
        status = worse_exec(status, "failed");
    }
    (status, super::HarnessNormalizedResult {
        package: native.package.clone(),
        target: native.target.clone(),
        name: native.name.clone(),
        verification_status: native.verification_status.clone(),
        unwind_bound: native.unwind_bound,
        totals,
        check_locations: locations,
    })
}

fn status_from_wire(
    native: &NativeUnit,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) -> &'static str {
    match native.verification_status.as_str() {
        "SUCCESS" => "passed",
        "FAILURE" => {
            findings.push((
                "kani-check-failure".to_owned(),
                "error".to_owned(),
                format!("harness `{}` failed", native.name),
            ));
            "failed"
        }
        "UNSUPPORTED" => {
            diagnostics.push(format!("harness `{}` encountered unsupported Kani construct", native.name));
            "unsupported"
        }
        "ERROR" => {
            diagnostics.push(format!("harness `{}` terminated with Kani tool error", native.name));
            "tool_error"
        }
        "UNKNOWN" => {
            diagnostics.push(format!("harness `{}` returned undetermined status", native.name));
            "incomplete_analysis"
        }
        "UNREACHABLE" => {
            findings.push((
                "kani-vacuous-assumption".to_owned(),
                "error".to_owned(),
                format!("harness `{}` unreachable", native.name),
            ));
            "failed"
        }
        other => {
            diagnostics.push(format!("unknown Kani harness status `{other}`"));
            "tool_error"
        }
    }
}

fn verify_constraints(
    profile: &super::Profile,
    check: &ConstraintCheck<'_>,
    diagnostics: &mut Vec<String>,
) -> &'static str {
    let mut status = "passed";
    if profile.is_require_reachable_cover {
        for cover_name in check.expected_covers {
            if check.observed_covers.get(cover_name.as_str()).copied() != Some("SATISFIED") {
                diagnostics.push(format!("cover witness `{cover_name}` missing or unreachable"));
                status = worse_exec(status, "failed");
            }
        }
        if check.totals.cover_satisfied == 0 || check.totals.cover_unsatisfied > 0 {
            diagnostics.push(format!("harness `{}` lacks satisfied cover witness", check.native.name));
            status = worse_exec(status, "failed");
        }
    }
    if profile.is_require_unwind_assertions
        && (check.totals.unwind_failed > 0 || (check.totals.unwind_passed == 0 && check.native.unwind_bound == 0))
    {
        diagnostics.push(format!("harness `{}` unwind assertions failed or bound zero", check.native.name));
        status = worse_exec(status, "failed");
    }
    status
}

fn classify_property<'a>(
    prop: &'a NativeProperty,
    totals: &mut super::PropertyTotals,
    observed_covers: &mut std::collections::BTreeMap<&'a str, &'a str>,
    diagnostics: &mut Vec<String>,
    findings: &mut Vec<(String, String, String)>,
) {
    let is_valid_type = matches!(
        prop.check_type.as_str(),
        "assert" | "cover" | "unwind" | "overflow" | "bounds" | "null-pointer" | "unreachable" | "assume"
    );
    if !is_valid_type {
        diagnostics.push(format!("unknown Kani property check_type `{}`", prop.check_type));
        totals.undetermined = totals.undetermined.saturating_add(1);
        return;
    }
    if prop.check_type == "cover" {
        observed_covers.insert(prop.name.as_str(), prop.status.as_str());
    }
    match prop.status.as_str() {
        "SUCCESS" | "SATISFIED" => {
            totals.passed = totals.passed.saturating_add(1);
            if prop.check_type == "cover" {
                totals.cover_satisfied = totals.cover_satisfied.saturating_add(1);
            } else if prop.check_type == "unwind" {
                totals.unwind_passed = totals.unwind_passed.saturating_add(1);
            }
        }
        "FAILURE" | "UNSATISFIABLE" => {
            totals.failed = totals.failed.saturating_add(1);
            let msg = prop.description.as_deref().unwrap_or(&prop.name);
            findings.push((
                "kani-property-failure".to_owned(),
                "error".to_owned(),
                format!("Kani property `{}` failed: {msg}", prop.name),
            ));
            if prop.check_type == "cover" {
                totals.cover_unsatisfied = totals.cover_unsatisfied.saturating_add(1);
            } else if prop.check_type == "unwind" {
                totals.unwind_failed = totals.unwind_failed.saturating_add(1);
            }
        }
        "UNREACHABLE" => {
            totals.unreachable = totals.unreachable.saturating_add(1);
            if prop.check_type == "cover" {
                totals.cover_unsatisfied = totals.cover_unsatisfied.saturating_add(1);
            }
        }
        "UNDETERMINED" => totals.undetermined = totals.undetermined.saturating_add(1),
        other => {
            diagnostics.push(format!("unknown Kani property `{}` status `{other}`", prop.name));
            totals.undetermined = totals.undetermined.saturating_add(1);
        }
    }
}

pub fn worse_exec<'a>(current: &'a str, candidate: &'a str) -> &'a str {
    let rank = |status: &str| match status {
        "passed" => 0_u8,
        "manual_review_required" => 1,
        "incomplete_analysis" => 2,
        "unsupported" => 3,
        "timeout" => 4,
        "tool_error" => 5,
        _ => 6,
    };
    if rank(candidate) > rank(current) {
        candidate
    } else {
        current
    }
}

pub const fn empty_totals() -> super::PropertyTotals {
    super::PropertyTotals {
        total: 0,
        passed: 0,
        failed: 0,
        undetermined: 0,
        unreachable: 0,
        cover_satisfied: 0,
        cover_unsatisfied: 0,
        unwind_passed: 0,
        unwind_failed: 0,
    }
}

pub fn accumulate_totals(dst: &mut super::PropertyTotals, src: &super::PropertyTotals) {
    dst.total = dst.total.saturating_add(src.total);
    dst.passed = dst.passed.saturating_add(src.passed);
    dst.failed = dst.failed.saturating_add(src.failed);
    dst.undetermined = dst.undetermined.saturating_add(src.undetermined);
    dst.unreachable = dst.unreachable.saturating_add(src.unreachable);
    dst.cover_satisfied = dst.cover_satisfied.saturating_add(src.cover_satisfied);
    dst.cover_unsatisfied = dst.cover_unsatisfied.saturating_add(src.cover_unsatisfied);
    dst.unwind_passed = dst.unwind_passed.saturating_add(src.unwind_passed);
    dst.unwind_failed = dst.unwind_failed.saturating_add(src.unwind_failed);
}
