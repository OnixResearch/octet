//! Opt-in external Dive `validate-trace` pin and schema-4 admission bridge for
//! `OnixResearch/trace-lab#6`.
//!
//! Default-off and pure: this module never invokes Dive, reads files, or adds a
//! second admission rail. It records the pinned `Chelis-Lang/dive` revision and
//! delegates complete schema-4 report bindings to [`super::admit`].

use serde::Deserialize;
use serde::Serialize;

/// Upstream Dive repository pinned for the live-Raft trace-validation tracker.
pub const DIVE_REPOSITORY: &str = "https://github.com/Chelis-Lang/dive";
/// Internal tracker that owns the live-Raft checker driver.
pub const DIVE_TRACKER_REF: &str = "OnixResearch/trace-lab#6";
/// Pinned `Chelis-Lang/dive` `main` commit SHA (observed 2026-11-20).
pub const DIVE_MAIN_COMMIT: &str = "0fc00914b7cd93d88e97e5d7d66aac805c06228b";
/// Pinned Dive `validate-trace` JSON report schema version (`docs/user/traces.md`).
pub const DIVE_REPORT_SCHEMA_VERSION: u32 = 4;
/// Upstream documentation path defining schema 4 at [`DIVE_MAIN_COMMIT`].
pub const DIVE_DOC_PATH: &str = "docs/user/traces.md";
/// Claim boundaries carried by the opt-in Dive pin and schema-4 admission bridge.
pub const DIVE_NON_CLAIMS: [&str; 4] = [
    "default-off opt-in pin for OnixResearch/trace-lab#6 only; does not change default cargo octet check, policy, or existing evidence profiles",
    "does not vendor Chelis-Lang/dive, build a reusable Dive package, or run validate-trace",
    "does not change Aspen #62's separate Dive 0.1.0 third-checker pin",
    "a raw schema-4 report without full model, checker, seed, and trace-set identity bindings stops as unsupported rather than claiming parity",
];

/// Pinned Dive metadata for `OnixResearch/trace-lab#6`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct DivePin {
    /// Upstream repository URL.
    pub repository: &'static str,
    /// Owning internal tracker reference.
    pub tracker: &'static str,
    /// Pinned upstream commit SHA.
    pub commit_sha: &'static str,
    /// Required `validate-trace` report schema version.
    pub report_schema_version: u32,
    /// Upstream schema document path at `commit_sha`.
    pub doc_path: &'static str,
    /// Explicit claim boundaries.
    pub non_claims: &'static [&'static str],
}

/// Return the canonical opt-in Dive pin for `OnixResearch/trace-lab#6`.
#[must_use]
pub const fn dive_pin() -> DivePin {
    DivePin {
        repository: DIVE_REPOSITORY,
        tracker: DIVE_TRACKER_REF,
        commit_sha: DIVE_MAIN_COMMIT,
        report_schema_version: DIVE_REPORT_SCHEMA_VERSION,
        doc_path: DIVE_DOC_PATH,
        non_claims: &DIVE_NON_CLAIMS,
    }
}

/// Opt-in Dive `validate-trace` schema-4 report envelope supplied by an explicit caller.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiveReport {
    /// Reported Dive `validate-trace` JSON schema version (must be `4`).
    pub schema_version: u32,
    /// Exact `Chelis-Lang/dive` commit SHA that produced the report.
    pub dive_commit_sha: String,
    /// Canonical `b3:` BLAKE3 identity of the Dive checker executable.
    pub checker_identity: String,
    /// Declared Dive checker version string.
    pub checker_version: String,
    /// Exact behavioral model bytes when bound by the caller.
    pub model_bytes: Option<Vec<u8>>,
    /// Explicit replay seed when bound by the caller.
    pub seed: Option<u64>,
    /// Canonical `octet.spec-trace/v1` trace set when mapped and bound by the caller.
    pub mapped_set: Option<super::TraceSet>,
}

/// Outcome of attempting to bind a Dive `validate-trace` report into existing
/// `spec_trace_parity` identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiveAdmission {
    /// Schema 4 and `DIVE_MAIN_COMMIT` matched, and every required identity bound
    /// cleanly through `spec_trace_parity::admit`.
    Admitted(Box<super::AdmittedIdentities>),
    /// Complete result when a schema-4 report omits any required identity binding
    /// (model bytes, seed, or mapped canonical trace set).
    Unsupported {
        /// Human-readable list of missing identity bindings.
        reason: String,
        /// Explicit claim boundaries.
        non_claims: Vec<String>,
    },
    /// Fail-closed denial when schema version, commit pin, or bound identities drift.
    Denied {
        /// Ordered denial records reusing the existing `Denial` contract.
        denials: Vec<super::Denial>,
    },
}

fn missing_bindings(report: &DiveReport) -> Vec<&'static str> {
    let mut missing = Vec::with_capacity(3);
    if report.model_bytes.is_none() {
        missing.push("model_bytes");
    }
    if report.seed.is_none() {
        missing.push("seed");
    }
    if report.mapped_set.is_none() {
        missing.push("mapped_set");
    }
    missing
}

/// Admit an opt-in Dive `validate-trace` report against declared pins by delegating
/// to [`super::admit`], or stop cleanly as [`DiveAdmission::Unsupported`] when
/// required identity bindings are absent.
#[must_use]
pub fn admit_dive_report(pins: &super::ParityPins, report: &DiveReport, bounds: super::RunBounds) -> DiveAdmission {
    let mut pin_denials = Vec::with_capacity(2);
    if report.schema_version != DIVE_REPORT_SCHEMA_VERSION {
        pin_denials.push(super::Denial {
            layer: super::DenialLayer::TraceFormat,
            expected: format!("dive.validate-trace.schema/{DIVE_REPORT_SCHEMA_VERSION}"),
            observed: format!("dive.validate-trace.schema/{}", report.schema_version),
        });
    }
    if report.dive_commit_sha != DIVE_MAIN_COMMIT {
        pin_denials.push(super::Denial {
            layer: super::DenialLayer::CheckerVersion,
            expected: DIVE_MAIN_COMMIT.to_owned(),
            observed: report.dive_commit_sha.clone(),
        });
    }
    if !pin_denials.is_empty() {
        return DiveAdmission::Denied { denials: pin_denials };
    }

    let missing = missing_bindings(report);
    if !missing.is_empty() {
        return DiveAdmission::Unsupported {
            reason: format!(
                "Dive schema-{} validate-trace report cannot bind existing spec-trace-parity identities without: {}",
                report.schema_version,
                missing.join(", ")
            ),
            non_claims: DIVE_NON_CLAIMS.iter().map(|entry| (*entry).to_owned()).collect(),
        };
    }

    let (Some(model_bytes), Some(seed), Some(trace_set)) =
        (report.model_bytes.clone(), report.seed, report.mapped_set.clone())
    else {
        return DiveAdmission::Unsupported {
            reason: "Dive schema-4 report missing required identity bindings".to_owned(),
            non_claims: DIVE_NON_CLAIMS.iter().map(|entry| (*entry).to_owned()).collect(),
        };
    };

    let request = super::ParityRequest {
        pins: pins.clone(),
        model_bytes,
        checker: super::CheckerIdentity {
            identity: report.checker_identity.clone(),
            version: report.checker_version.clone(),
        },
        seed,
        trace_set,
        bounds,
    };

    match super::admit(&request, None) {
        super::Admission::Admitted(identities) => DiveAdmission::Admitted(identities),
        super::Admission::Denied(denials) => DiveAdmission::Denied { denials },
    }
}
