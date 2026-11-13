//! Cross-field release qualification and source-binding rules.

use serde_json::Value;

pub(super) fn check_schema(manifest: &Value, findings: &mut Vec<super::Finding>) {
    if text(manifest, &["schema_version"]) != Some(super::SCHEMA_VERSION) {
        findings
            .push(super::Finding::new("schema-version", format!("expected schema_version {}", super::SCHEMA_VERSION)));
    }
}

pub(super) fn check_source_binding(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let public = text(manifest, &["source", "public", "revision"]);
    let inventory = text(manifest, &["capability_inventory", "source_revision"]);
    if public.is_none() || inventory != public {
        findings.push(super::Finding::new(
            "inventory-source-mismatch",
            format!("inventory revision {inventory:?} does not name the public source revision {public:?}"),
        ));
    }

    let head = text(manifest, &["channel", "observed_branch_head"]);
    if flag(manifest, &["channel", "head_matches_reviewed_revision"]) && head != public {
        findings.push(super::Finding::new(
            "channel-head-mismatch",
            format!("channel head {head:?} does not name the reviewed revision {public:?}"),
        ));
    }

    let origin = text(manifest, &["source", "public", "derived_from_revision"]);
    let inventory_origin = text(manifest, &["capability_inventory", "origin_revision"]);
    if inventory_origin != origin {
        findings.push(super::Finding::new(
            "inventory-origin-mismatch",
            format!("inventory origin {inventory_origin:?} does not name the export origin {origin:?}"),
        ));
    }
}

pub(super) fn check_notices(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let required: Vec<&str> =
        rows(manifest, &["notices", "required"]).iter().filter_map(|row| text(row, &["path"])).collect();
    for path in super::REQUIRED_NOTICE_PATHS {
        if !required.contains(&path) {
            findings.push(super::Finding::new(
                "required-notice-omitted",
                format!("required public notice {path} is missing from notices.required"),
            ));
        }
    }

    let attributions: Vec<&str> =
        rows(manifest, &["notices", "attribution_ids"]).iter().filter_map(Value::as_str).collect();
    for id in super::REQUIRED_ATTRIBUTION_IDS {
        if !attributions.contains(&id) {
            findings.push(super::Finding::new(
                "required-notice-omitted",
                format!("attribution row {id} is missing from notices.attribution_ids"),
            ));
        }
    }

    let qualification = text(manifest, &["qualification", "status"]);
    for row in rows(manifest, &["notices", "imported_material"]) {
        let path = text(row, &["path"]).unwrap_or("<unnamed>");
        let resolution = text(row, &["resolution"]).unwrap_or_default();
        let is_included = flag(row, &["included_in_qualified_cohort"]);
        let evidence = text(row, &["permission_evidence"]).unwrap_or_default();

        if flag(row, &["root_license_relied_upon"]) {
            findings.push(super::Finding::new(
                "root-license-cannot-clear-imported-content",
                format!("{path} relies on the root license to clear imported material"),
            ));
        }
        if is_included && resolution != "permission_recorded" {
            findings.push(super::Finding::new(
                "imported-content-included-without-permission",
                format!("{path} is included in the qualified cohort with resolution {resolution}"),
            ));
        }
        if resolution == "permission_recorded" && evidence.trim().is_empty() {
            findings.push(super::Finding::new(
                "imported-content-included-without-permission",
                format!("{path} claims recorded permission without permission evidence"),
            ));
        }
        if resolution == "unresolved" && qualification == Some("qualified") {
            findings.push(super::Finding::new(
                "unresolved-imported-content-represented-as-cleared",
                format!("{path} is unresolved while the cohort claims qualification"),
            ));
        }
    }

    for row in rows(manifest, &["notices", "required"]) {
        let path = text(row, &["path"]).unwrap_or("<unnamed>");
        if !flag(row, &["present_in_public_tree"]) && qualification == Some("qualified") {
            findings.push(super::Finding::new(
                "notice-absent-with-qualified-claim",
                format!("{path} is absent from the public tree while the cohort claims qualification"),
            ));
        }
    }
}

pub(super) fn check_dependencies(manifest: &Value, findings: &mut Vec<super::Finding>) {
    for row in rows(manifest, &["dependencies"]) {
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        if flag(row, &["warm_cache_dependency"]) {
            findings.push(super::Finding::new(
                "private-input-hidden-by-warm-cache",
                format!("{id} depends on a warm cache or pre-populated private input"),
            ));
        }
        if !flag(row, &["anonymous_at_review"]) {
            findings.push(super::Finding::new(
                "dependency-not-anonymous",
                format!("{id} was not retrievable anonymously at review time"),
            ));
        }
    }
}

pub(super) fn check_receipts(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let public = text(manifest, &["source", "public", "revision"]);
    let evidence = text(manifest, &["acceptance", "evidence_digest"]);

    for row in rows(manifest, &["acceptance", "receipts"]) {
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        if text(row, &["evidence_kind"]) != Some("fresh") {
            findings.push(super::Finding::new("receipt-not-fresh", format!("{id} is not a fresh acceptance receipt")));
        }
        if text(row, &["source_revision"]) != public {
            findings.push(super::Finding::new(
                "receipt-source-mismatch",
                format!("{id} binds a source other than the cohort's public revision"),
            ));
        }
        if text(row, &["evidence_digest"]) != evidence {
            findings.push(super::Finding::new(
                "evidence-digest-mismatch",
                format!("{id} carries an evidence digest other than the cohort's"),
            ));
        }
    }

    for row in rows(manifest, &["acceptance", "non_evidence"]) {
        let reference = text(row, &["reference"]).unwrap_or("<unnamed>");
        if text(row, &["evidence_role"]) != Some("non-evidence") {
            findings.push(super::Finding::new(
                "historical-receipt-labelled-evidence",
                format!("{reference} is historical and must be marked non-evidence"),
            ));
        }
    }
}

pub(super) fn check_profiles(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let fresh: Vec<&str> =
        rows(manifest, &["acceptance", "receipts"]).iter().filter_map(|row| text(row, &["id"])).collect();

    for row in rows(manifest, &["profiles"]) {
        if text(row, &["support_status"]) != Some("verified") {
            continue;
        }
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        let refs: Vec<&str> = rows(row, &["evidence_refs"]).iter().filter_map(Value::as_str).collect();
        let is_unsupported = refs.is_empty() || refs.iter().any(|reference| !fresh.contains(reference));
        if is_unsupported {
            findings.push(super::Finding::new(
                "unsupported-profile-claimed-verified",
                format!("{id} claims verified support without fresh cohort receipts"),
            ));
        }
    }
}

pub(super) fn check_qualification(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let status = text(manifest, &["qualification", "status"]);
    let reasons = rows(manifest, &["qualification", "blocking_reasons"]);
    if status == Some("qualified") && !reasons.is_empty() {
        findings.push(super::Finding::new(
            "qualified-with-blocking-reasons",
            "cohort claims qualification while blocking reasons remain",
        ));
    }
    if status == Some("blocked") && reasons.is_empty() {
        findings
            .push(super::Finding::new("blocked-without-reason", "cohort is blocked without a named blocking reason"));
    }
}

pub(super) fn check_packages(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let outputs: Vec<&str> = rows(manifest, &["packages"]).iter().filter_map(|row| text(row, &["output"])).collect();
    for output in super::REQUIRED_PACKAGE_OUTPUTS {
        if !outputs.contains(&output) {
            findings.push(super::Finding::new(
                "missing-package-output",
                format!("required package output {output} is not listed"),
            ));
        }
    }
}

pub(super) fn check_inventory_shape(manifest: &Value, findings: &mut Vec<super::Finding>) {
    let declared = at(manifest, &["capability_inventory", "count"]).and_then(Value::as_u64);
    let names = rows(manifest, &["capability_inventory", "names"]).len();
    let excluded: Vec<&str> = rows(manifest, &["capability_inventory", "excluded_from_public"])
        .iter()
        .filter_map(|row| text(row, &["name"]))
        .collect();
    let development = at(manifest, &["capability_inventory", "development", "count"]).and_then(Value::as_u64);

    if declared.is_none() || declared.and_then(|count| usize::try_from(count).ok()) != Some(names) {
        findings.push(super::Finding::new(
            "inventory-count-mismatch",
            format!("count {declared:?} does not equal {names} declared names"),
        ));
    }
    let expected_development = names.checked_add(excluded.len());
    if expected_development.is_none()
        || development.is_none()
        || development.and_then(|count| usize::try_from(count).ok()) != expected_development
    {
        findings.push(super::Finding::new(
            "inventory-count-mismatch",
            format!("development count {development:?} is not the public count plus reviewed exclusions"),
        ));
    }
    for name in &excluded {
        if rows(manifest, &["capability_inventory", "names"]).iter().any(|value| value.as_str() == Some(*name)) {
            findings.push(super::Finding::new(
                "inventory-excluded-name-present",
                format!("excluded capability {name} is also listed as public"),
            ));
        }
    }
}

fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = match key.parse::<usize>() {
            Ok(index) => current.get(index)?,
            Err(_) => current.get(*key)?,
        };
    }
    Some(current)
}

pub(super) fn text<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    at(value, path)?.as_str()
}

fn flag(value: &Value, path: &[&str]) -> bool {
    at(value, path).and_then(Value::as_bool).unwrap_or(false)
}

pub(super) fn rows<'a>(value: &'a Value, path: &[&str]) -> &'a [Value] {
    at(value, path).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}
