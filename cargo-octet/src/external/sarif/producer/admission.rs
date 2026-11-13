pub(crate) fn registry() -> Result<super::Registry, String> {
    let registry: super::Registry = serde_json::from_str(super::PROFILE_JSON)
        .map_err(|error| format!("parse embedded SARIF producer profile registry: {error}"))?;
    if registry.schema_version != super::PROFILE_SCHEMA {
        return Err("embedded SARIF producer profile registry schema is unsupported".to_owned());
    }
    Ok(registry)
}

pub(crate) fn selected_profile(profile_id: &str) -> Result<super::Profile, String> {
    let registry = registry()?;
    let profile = registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown SARIF producer profile `{profile_id}`"))?;
    let diagnostics = validate_profile(&profile);
    if !diagnostics.is_empty() {
        return Err(format!("SARIF producer profile `{profile_id}` is invalid: {}", diagnostics.join("; ")));
    }
    Ok(profile)
}

pub(crate) fn profile_identity() -> &'static str {
    super::PROFILE_IDENTITY_VALUE
}

/// Whether the id names a checked producer execution profile. Unknown or malformed ids are not
/// producer profiles; callers then fall back to import-only handling, which rejects them.
pub(crate) fn contains_profile(profile_id: &str) -> Result<bool, String> {
    Ok(registry()?.profiles.iter().any(|profile| profile.profile_id == profile_id))
}

/// Map an intake or producer profile id to the SARIF intake profile that validates the report.
pub(crate) fn intake_profile_id(profile_id: &str) -> Result<String, String> {
    let registry = registry()?;
    if let Some(profile) = registry.profiles.iter().find(|profile| profile.profile_id == profile_id) {
        return Ok(profile.intake_profile_id.clone());
    }
    Ok(profile_id.to_owned())
}

pub(crate) fn validate_profile(profile: &super::Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.profile_id.trim().is_empty() {
        diagnostics.push("SARIF producer profile_id must not be empty".to_owned());
    }
    if !matches!(profile.producer.as_str(), super::PRODUCER_SEMGREP | super::PRODUCER_CODEQL) {
        diagnostics.push(format!(
            "SARIF producer `{}` is unsupported; expected `{}` or `{}`",
            profile.producer,
            super::PRODUCER_SEMGREP,
            super::PRODUCER_CODEQL
        ));
    }
    if profile.executable.name != profile.producer {
        diagnostics.push("SARIF producer executable.name must equal the declared producer".to_owned());
    }
    if profile.executable.declared_version.trim().is_empty() || !profile.executable.declared_version.contains('.') {
        diagnostics.push("SARIF producer executable.declared_version must be a pinned version".to_owned());
    }
    if profile.supported_languages.len() != 1 || profile.supported_languages[0] != "rust" {
        diagnostics.push("SARIF producer supported_languages must be exactly [\"rust\"]".to_owned());
    }
    match super::super::selected_profile(&profile.intake_profile_id) {
        Ok(intake) if intake.driver_name == profile.driver_name => {}
        Ok(_) => diagnostics.push("SARIF producer driver_name disagrees with its intake profile".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    if profile.non_claims.is_empty() {
        diagnostics.push("SARIF producer profile is missing non-claims".to_owned());
    }
    validate_rule_source(profile, &mut diagnostics);
    validate_scope(profile, &mut diagnostics);
    validate_limits(profile, &mut diagnostics);
    validate_shape(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(super) fn validate_rule_source(profile: &super::Profile, diagnostics: &mut Vec<String>) {
    if profile.rules.identity_source != super::IDENTITY_SOURCE_CHECKED_BYTES
        && profile.rules.identity_source != super::IDENTITY_SOURCE_DECLARED_PIN
    {
        diagnostics.push("SARIF producer rule identity_source is unsupported".to_owned());
    }
    match profile.rules.kind.as_str() {
        super::LOCAL_RULE_FILE => {
            if profile.rules.identity_source != super::IDENTITY_SOURCE_CHECKED_BYTES {
                diagnostics.push("a local SARIF producer rule file must bind checked file bytes".to_owned());
            }
            if !is_safe_relative_path(&profile.rules.path) {
                diagnostics.push("SARIF producer rule file path must be workspace-relative".to_owned());
            }
        }
        super::QUERY_PACK_SET => {
            if profile.rules.identity_source != super::IDENTITY_SOURCE_DECLARED_PIN {
                diagnostics.push("a SARIF producer query pack set must record a declared pin".to_owned());
            }
            if !profile.rules.path.is_empty() {
                diagnostics.push("a SARIF producer query pack set must not declare a local rule path".to_owned());
            }
            if profile.query_packs.is_empty() {
                diagnostics.push("a SARIF producer query pack set requires at least one pack".to_owned());
            }
            if super::recording::query_pack_set_identity(&profile.query_packs).as_ref() != Ok(&profile.rules.identity) {
                diagnostics.push("SARIF producer rules identity does not match its query pack pins".to_owned());
            }
        }
        other => diagnostics.push(format!("SARIF producer rule kind `{other}` is unsupported")),
    }
    let mut names = std::collections::BTreeSet::new();
    for pack in &profile.query_packs {
        if pack.name.trim().is_empty() || pack.version.trim().is_empty() {
            diagnostics.push("SARIF producer query pack needs a name and version".to_owned());
        }
        if pack.identity_source != super::IDENTITY_SOURCE_DECLARED_PIN {
            diagnostics.push("SARIF producer query pack identity_source must be `declared-pin`".to_owned());
        }
        if super::recording::query_pack_identity(&pack.name, &pack.version).as_ref() != Ok(&pack.identity) {
            diagnostics.push(format!("SARIF producer query pack `{}` identity is stale", pack.name));
        }
        if !names.insert(pack.name.as_str()) {
            diagnostics.push(format!("duplicate SARIF producer query pack `{}`", pack.name));
        }
    }
}

pub(super) fn validate_scope(profile: &super::Profile, diagnostics: &mut Vec<String>) {
    if profile.scope.paths.is_empty() {
        diagnostics.push("SARIF producer scope must select at least one path".to_owned());
    }
    for path in &profile.scope.paths {
        if !is_safe_relative_path(path) {
            diagnostics.push(format!("SARIF producer scope path `{path}` must be workspace-relative"));
        }
    }
    if profile.scope.extensions.is_empty() {
        diagnostics.push("SARIF producer scope must declare at least one extension".to_owned());
    }
    for extension in &profile.scope.extensions {
        if !extension.starts_with('.') || extension.len() < 2 {
            diagnostics.push(format!("SARIF producer scope extension `{extension}` must start with `.`"));
        }
    }
    for glob in &profile.scope.exclude_globs {
        if !is_valid_glob(glob) {
            diagnostics.push(format!("SARIF producer exclusion glob `{glob}` is unsupported"));
        }
    }
    if profile.scope.max_files == 0 || profile.scope.max_bytes == 0 {
        diagnostics.push("SARIF producer scope limits must be positive".to_owned());
    }
}

pub(super) fn validate_limits(profile: &super::Profile, diagnostics: &mut Vec<String>) {
    if profile.limits.max_report_bytes != super::super::MAX_REPORT_BYTES {
        diagnostics.push("SARIF producer max_report_bytes must match the intake profile limit".to_owned());
    }
    for (label, is_zero) in [
        ("wall_clock_ms", profile.limits.wall_clock_ms == 0),
        ("stdout_max_bytes", profile.limits.stdout_max_bytes == 0),
        ("stderr_max_bytes", profile.limits.stderr_max_bytes == 0),
        ("max_memory_mb", profile.limits.max_memory_mb == 0),
        ("max_threads", profile.limits.max_threads == 0),
    ] {
        if is_zero {
            diagnostics.push(format!("SARIF producer limit `{label}` must be positive"));
        }
    }
}

pub(super) fn validate_shape(profile: &super::Profile, diagnostics: &mut Vec<String>) {
    match profile.producer.as_str() {
        super::PRODUCER_SEMGREP => {
            if profile.rules.kind != super::LOCAL_RULE_FILE {
                diagnostics.push("Semgrep producer requires a checked local rule file".to_owned());
            }
            if profile.database.is_some() || !profile.query_packs.is_empty() {
                diagnostics.push("Semgrep producer must not declare a CodeQL database or query packs".to_owned());
            }
            if profile.rust.build_mode != super::BUILD_MODE_SOURCE_ONLY {
                diagnostics.push("Semgrep producer rust.build_mode must be `source-only`".to_owned());
            }
        }
        super::PRODUCER_CODEQL => {
            if profile.rules.kind != super::QUERY_PACK_SET {
                diagnostics.push("CodeQL producer requires a query pack set".to_owned());
            }
            match profile.database.as_ref() {
                Some(database) if database.language == "rust" && database.build_mode == super::BUILD_MODE_NONE => {
                    if database.policy != super::DATABASE_POLICY_REUSE_OR_CREATE {
                        diagnostics.push("CodeQL database policy is unsupported".to_owned());
                    }
                }
                Some(_) => diagnostics
                    .push("CodeQL producer database must use language `rust` and build mode `none`".to_owned()),
                None => diagnostics.push("CodeQL producer requires a database declaration".to_owned()),
            }
            if profile.rust.build_mode != super::BUILD_MODE_NONE {
                diagnostics.push("CodeQL producer rust.build_mode must be `none`".to_owned());
            }
        }
        _ => {}
    }
}

pub(super) fn is_safe_relative_path(path: &str) -> bool {
    let candidate = std::path::Path::new(path);
    !path.is_empty()
        && !path.contains('\0')
        && !candidate.is_absolute()
        && candidate.components().all(|component| matches!(component, std::path::Component::Normal(_)))
}

pub(super) fn is_valid_glob(glob: &str) -> bool {
    !glob.is_empty()
        && !glob.contains('\0')
        && !glob.starts_with('/')
        && glob.split('/').all(|segment| !matches!(segment, "" | "." | ".."))
}
