//! Cross-tool architecture, assurance, and adoption experiment contract.
//!
//! The existing `benchmark` module keeps the strict same-command performance
//! comparator. This module is a separate competitive experiment: it consumes a
//! predeclared Nickel-authored profile, retained observations, and an explicit
//! applicability matrix. It never turns unsupported scope into clean evidence.

use std::collections::BTreeMap;

pub const PROFILES_SCHEMA: &str = "octet-benchmark-experiment-profiles/v1";
pub const OBSERVATIONS_SCHEMA: &str = "octet-benchmark-experiment-observations/v1";
pub const ARTIFACT_SCHEMA: &str = "octet-benchmark-experiment/v1";
pub const PROFILE_ID: &str = "octet-cross-tool-architecture-v1";
pub const ARTIFACT_NAME: &str = "benchmark-experiment.json";
pub const OBSERVATIONS_NAME: &str = "benchmark-experiment-observations.json";
pub const REPORT_NAME: &str = "benchmark-experiment-report.md";
pub const REQUIRED_NON_CLAIM: &str = "The experiment is bounded fixture evidence; it does not prove correctness, universal detection, complete Rust coverage, or release eligibility.";

mod plans;
mod records;
mod results;

pub use plans::AdoptionTarget;
pub use plans::AdoptionToolPlan;
pub use plans::AnalysisProfile;
pub use plans::AnalysisTask;
pub use plans::Applicability;
pub use plans::ApplicabilityStatus;
pub use plans::Category;
pub use plans::ConfigurationStatus;
pub use plans::CorpusCase;
pub use plans::ExpectedOutcome;
pub use plans::Registry;
pub use plans::ToolPin;
pub use records::AdoptionObservation;
pub use records::AdoptionPhase;
pub use records::Availability;
pub use records::CacheState;
pub use records::CompileStatus;
pub use records::MeasurementOrigin;
pub use records::ObservationEnvironment;
pub use records::ObservationScope;
pub use records::ObservationSet;
pub use records::ObservationStatus;
pub use records::ObservedFinding;
#[cfg(test)]
pub use records::ObservedSuppression;
pub use records::ToolAvailability;
pub use records::ToolObservation;
pub use records::WarmState;
pub use results::AdoptionSummary;
pub use results::Artifact;
pub use results::ControlOutcome;
pub use results::DerivedProjection;
pub use results::Evaluation;
pub use results::NegativeControlRecord;
pub use results::ProvenanceSummary;
pub use results::ToolTaskMetrics;

/// `BTreeMap` of case id to case for deterministic iteration.
pub(crate) fn case_map(profile: &AnalysisProfile) -> BTreeMap<&str, &CorpusCase> {
    profile.corpus.iter().map(|case| (case.case_id.as_str(), case)).collect()
}

pub(crate) fn task_map(profile: &AnalysisProfile) -> BTreeMap<&str, &AnalysisTask> {
    profile.tasks.iter().map(|task| (task.task_id.as_str(), task)).collect()
}

pub(crate) fn tool_map(profile: &AnalysisProfile) -> BTreeMap<&str, &ToolPin> {
    profile.tools.iter().map(|tool| (tool.tool_id.as_str(), tool)).collect()
}

pub(crate) fn applicability_map(profile: &AnalysisProfile) -> BTreeMap<(&str, &str), &Applicability> {
    profile
        .applicability
        .iter()
        .map(|entry| ((entry.tool_id.as_str(), entry.task_id.as_str()), entry))
        .collect()
}
