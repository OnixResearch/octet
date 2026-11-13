mod controls;
mod projection;
mod validation;

pub fn assess(
    profile_id: &str,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> crate::benchmark::experiment::model::Evaluation {
    let profile = match super::profile::select(profile_id) {
        Ok(profile) => profile,
        Err(error) => return failed(vec![error]),
    };
    let mut diagnostics = validation::profile::validate(&profile);
    if observations.schema_version != crate::benchmark::experiment::model::OBSERVATIONS_SCHEMA
        || observations.profile_id != profile.profile_id
    {
        diagnostics.push("experiment observations schema or profile is unsupported".to_owned());
    }
    diagnostics.extend(validation::observations::validate(&profile, observations));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return failed(diagnostics);
    }
    let mut artifact = match derive_artifact(&profile, observations) {
        Ok(artifact) => artifact,
        Err(errors) => return failed(errors),
    };
    let controls = controls::run(&profile, observations, &artifact);
    let escaped: Vec<&str> = controls
        .iter()
        .filter(|record| record.outcome != crate::benchmark::experiment::model::ControlOutcome::Contained)
        .map(|record| record.control_id.as_str())
        .collect();
    if !escaped.is_empty() {
        return failed(escaped.into_iter().map(|id| format!("experiment negative control `{id}` escaped")).collect());
    }
    if let Ok(bytes) = crate::benchmark::pretty_bytes(&controls, "experiment negative controls") {
        artifact.controls_identity = crate::benchmark::prefixed_identity(&bytes);
    } else {
        return failed(vec!["experiment negative controls could not be serialized".to_owned()]);
    }
    artifact.controls = controls;
    let diagnostics = verify_artifact(&profile, &artifact);
    if diagnostics.is_empty() {
        crate::benchmark::experiment::model::Evaluation {
            artifact: Some(artifact),
            diagnostics: Vec::new(),
        }
    } else {
        failed(diagnostics)
    }
}

pub fn verify(profile_id: &str, artifact: &crate::benchmark::experiment::model::Artifact) -> Vec<String> {
    let profile = match super::profile::select(profile_id) {
        Ok(profile) => profile,
        Err(error) => return vec![error],
    };
    let mut diagnostics = verify_artifact(&profile, artifact);
    let observations = crate::benchmark::experiment::model::ObservationSet {
        schema_version: crate::benchmark::experiment::model::OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: artifact.profile.profile_id.clone(),
        tool_availability: artifact.tool_availability.clone(),
        observations: artifact.observations.clone(),
        adoption: artifact.adoption.clone(),
    };
    let controls = controls::run(&profile, &observations, artifact);
    for record in &controls {
        if record.outcome != crate::benchmark::experiment::model::ControlOutcome::Contained {
            diagnostics.push(format!("experiment negative control `{}` escaped", record.control_id));
        }
    }
    if controls != artifact.controls {
        diagnostics.push("experiment negative-control records do not reproduce".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn derive_artifact(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> Result<crate::benchmark::experiment::model::Artifact, Vec<String>> {
    let mut diagnostics = validation::profile::validate(profile);
    diagnostics.extend(validation::observations::validate(profile, observations));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let derived = projection::derive(profile, observations)?;
    let observations_identity = identity_of(observations, "experiment observations").map_err(|error| vec![error])?;
    let adoption_identity =
        identity_of(&observations.adoption, "experiment adoption observations").map_err(|error| vec![error])?;
    let derived_identity = identity_of(&derived, "experiment derived projection").map_err(|error| vec![error])?;
    Ok(crate::benchmark::experiment::model::Artifact {
        schema_version: crate::benchmark::experiment::model::ARTIFACT_SCHEMA.to_owned(),
        profile_identity: super::profile::PROFILE_IDENTITY.to_owned(),
        profile: profile.clone(),
        tool_availability: observations.tool_availability.clone(),
        observations_identity,
        observations: observations.observations.clone(),
        adoption_identity,
        adoption: observations.adoption.clone(),
        derived,
        derived_identity,
        controls: Vec::new(),
        controls_identity: identity_of(
            &Vec::<crate::benchmark::experiment::model::NegativeControlRecord>::new(),
            "experiment negative controls",
        )
        .map_err(|error| vec![error])?,
    })
}

pub(crate) fn verify_artifact(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(8);
    if artifact.schema_version != crate::benchmark::experiment::model::ARTIFACT_SCHEMA {
        diagnostics.push("experiment artifact schema is unsupported".to_owned());
    }
    if artifact.profile_identity != super::profile::PROFILE_IDENTITY {
        diagnostics.push("experiment artifact profile identity is stale".to_owned());
    }
    if artifact.profile != *profile {
        diagnostics.push("experiment artifact profile does not match the embedded profile".to_owned());
    }
    if artifact.profile.profile_id != crate::benchmark::experiment::model::PROFILE_ID
        || artifact.profile.coverage_boundary.complete_rust_coverage
        || !artifact.profile.coverage_boundary.bounded
    {
        diagnostics.push("experiment artifact presents the bounded corpus as complete Rust coverage".to_owned());
    }
    let observations = crate::benchmark::experiment::model::ObservationSet {
        schema_version: crate::benchmark::experiment::model::OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: artifact.profile.profile_id.clone(),
        tool_availability: artifact.tool_availability.clone(),
        observations: artifact.observations.clone(),
        adoption: artifact.adoption.clone(),
    };
    diagnostics.extend(validation::observations::validate(profile, &observations));
    match identity_of(&observations, "experiment observations") {
        Ok(identity) if identity == artifact.observations_identity => {}
        Ok(_) => diagnostics.push("experiment observation identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    match identity_of(&artifact.adoption, "experiment adoption observations") {
        Ok(identity) if identity == artifact.adoption_identity => {}
        Ok(_) => diagnostics.push("experiment adoption identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    match projection::derive(profile, &observations) {
        Ok(derived) => {
            if derived != artifact.derived {
                diagnostics.push("experiment derived comparison does not reproduce from retained inputs".to_owned());
            }
            match identity_of(&derived, "experiment derived projection") {
                Ok(identity) if identity == artifact.derived_identity => {}
                Ok(_) => diagnostics.push("experiment derived projection identity is stale".to_owned()),
                Err(error) => diagnostics.push(error),
            }
        }
        Err(errors) => diagnostics.extend(errors),
    }
    if let Ok(bytes) = crate::benchmark::pretty_bytes(&artifact.controls, "experiment negative controls")
        && crate::benchmark::prefixed_identity(&bytes) != artifact.controls_identity
    {
        diagnostics.push("experiment negative-control identity is stale".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn identity_of<T: serde::Serialize>(value: &T, label: &str) -> Result<String, String> {
    crate::benchmark::pretty_bytes(value, label).map(|bytes| crate::benchmark::prefixed_identity(&bytes))
}

pub(crate) fn scope_identity(scope: &crate::benchmark::experiment::model::ObservationScope) -> String {
    let mut selected = scope.selected_sources.clone();
    selected.sort();
    let mut analyzed = scope.analyzed_sources.clone();
    analyzed.sort();
    let payload = serde_json::json!({
        "selected_sources": selected,
        "analyzed_sources": analyzed,
    });
    match serde_json::to_vec(&payload) {
        Ok(bytes) => crate::benchmark::prefixed_identity(&bytes),
        Err(_) => String::new(),
    }
}

fn is_b3(value: &str) -> bool {
    value
        .strip_prefix("b3:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn failed(mut diagnostics: Vec<String>) -> crate::benchmark::experiment::model::Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    crate::benchmark::experiment::model::Evaluation {
        artifact: None,
        diagnostics,
    }
}
