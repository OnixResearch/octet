use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

mod adoption;
mod observations;
mod scratch;
mod shell;

#[cfg(test)]
pub(super) use adoption::Measurement;
#[cfg(test)]
pub(super) use adoption::measure_target;
#[cfg(test)]
pub use shell::probe_tool;
#[cfg(test)]
pub use shell::run_command;

pub const PRESERVED_ENVIRONMENT: [&str; 6] = [
    "PATH",
    "RUSTUP_HOME",
    "CARGO_HOME",
    "RUSTUP_TOOLCHAIN",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
];

pub struct Runtime {
    pub environment: Vec<(OsString, OsString)>,
    pub scratch_root: PathBuf,
    pub clock: cap_std::time::MonotonicClock,
}

#[derive(Clone, Copy)]
pub enum Mode<'a> {
    Recorded,
    Execute(&'a Runtime),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutcome {
    pub program: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub succeeded: bool,
    pub timed_out: bool,
    pub duration_ns: u64,
    pub truncated: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Run {
    pub artifact: crate::benchmark::experiment::model::Artifact,
    pub observations: crate::benchmark::experiment::model::ObservationSet,
    pub report: String,
    pub artifact_bytes: Vec<u8>,
}

pub fn run(mode: Mode<'_>, profile_id: &str, input: Option<&str>, corpus_root: &Path) -> Result<Run, Vec<String>> {
    let profile = super::profile::select(profile_id).map_err(|error| vec![error])?;
    let observations = match mode {
        Mode::Recorded => {
            let Some(input) = input else {
                return Err(vec!["experiment recorded mode requires a retained observation input".to_owned()]);
            };
            serde_json::from_str::<crate::benchmark::experiment::model::ObservationSet>(input)
                .map_err(|error| vec![format!("parse experiment observations: {error}")])?
        }
        Mode::Execute(runtime) => observations::collect(&profile, corpus_root, runtime)?,
    };
    let evaluation = super::evaluate::assess(profile_id, &observations);
    let Some(artifact) = evaluation.artifact else {
        return Err(evaluation.diagnostics);
    };
    let report =
        super::report::render(&artifact).map_err(|error| vec![format!("render experiment report: {error}")])?;
    let artifact_bytes =
        crate::benchmark::pretty_bytes(&artifact, "experiment artifact").map_err(|error| vec![error])?;
    Ok(Run {
        artifact,
        observations,
        report,
        artifact_bytes,
    })
}
