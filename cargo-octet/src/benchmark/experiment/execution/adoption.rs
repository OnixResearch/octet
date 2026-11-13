use std::ffi::OsString;
use std::path::Path;

#[cfg(test)]
mod tests;

pub(crate) struct Measurement<'a> {
    pub profile: &'a crate::benchmark::experiment::model::AnalysisProfile,
    pub tool: &'a crate::benchmark::experiment::model::ToolPin,
    pub runtime: &'a super::Runtime,
    pub availability: &'a crate::benchmark::experiment::model::Availability,
}

pub(crate) fn measure_target(
    measurement: &Measurement<'_>,
    root: &Path,
    target: &crate::benchmark::experiment::model::AdoptionTarget,
    plan: &crate::benchmark::experiment::model::AdoptionToolPlan,
) -> (
    crate::benchmark::experiment::model::AdoptionObservation,
    crate::benchmark::experiment::model::AdoptionObservation,
) {
    if *measurement.availability == crate::benchmark::experiment::model::Availability::Unavailable {
        return unavailable_pair(measurement, target, "executable is unavailable");
    }
    if measurement.tool.run_argv.is_empty() {
        return unavailable_pair(measurement, target, "no reviewed adoption command; probe-only tool");
    }
    let scratch = super::shell::scratch_directory(&measurement.tool.tool_id, measurement.runtime);
    measure_at(measurement, root, target, plan, &scratch)
}

fn measure_at(
    measurement: &Measurement<'_>,
    root: &Path,
    target: &crate::benchmark::experiment::model::AdoptionTarget,
    plan: &crate::benchmark::experiment::model::AdoptionToolPlan,
    scratch: &Path,
) -> (
    crate::benchmark::experiment::model::AdoptionObservation,
    crate::benchmark::experiment::model::AdoptionObservation,
) {
    let environment = match super::scratch::prepare(scratch, measurement.runtime) {
        Ok(environment) => environment,
        Err(error) => {
            let mut pair = unavailable_pair(measurement, target, &error);
            pair.0.origin = crate::benchmark::experiment::model::MeasurementOrigin::Local;
            pair.1.origin = crate::benchmark::experiment::model::MeasurementOrigin::Local;
            return pair;
        }
    };
    let session = Session {
        measurement,
        root,
        target,
    };
    let mut initial = session.run_phase(
        crate::benchmark::experiment::model::AdoptionPhase::Initial,
        &plan.initial_command,
        environment.clone(),
    );
    let mut incremental = session.run_phase(
        crate::benchmark::experiment::model::AdoptionPhase::Incremental,
        &plan.incremental_command,
        environment,
    );
    if let Err(error) = super::scratch::cleanup(scratch) {
        for observation in [&mut initial, &mut incremental] {
            observation.measured = false;
            observation.unavailable_reason = Some(match observation.unavailable_reason.take() {
                Some(reason) => format!("{reason}; {error}"),
                None => error.clone(),
            });
        }
    }
    (initial, incremental)
}

pub(super) fn unavailable_pair(
    measurement: &Measurement<'_>,
    target: &crate::benchmark::experiment::model::AdoptionTarget,
    reason: &str,
) -> (
    crate::benchmark::experiment::model::AdoptionObservation,
    crate::benchmark::experiment::model::AdoptionObservation,
) {
    (
        unavailable(measurement, target, crate::benchmark::experiment::model::AdoptionPhase::Initial, reason),
        unavailable(measurement, target, crate::benchmark::experiment::model::AdoptionPhase::Incremental, reason),
    )
}

struct Session<'a> {
    measurement: &'a Measurement<'a>,
    root: &'a Path,
    target: &'a crate::benchmark::experiment::model::AdoptionTarget,
}

impl Session<'_> {
    fn run_phase(
        &self,
        phase: crate::benchmark::experiment::model::AdoptionPhase,
        argv: &[String],
        environment: Vec<(OsString, OsString)>,
    ) -> crate::benchmark::experiment::model::AdoptionObservation {
        let measurement = self.measurement;
        let profile = measurement.profile;
        let tool = measurement.tool;
        let target = self.target;
        let selection_id = target.selection_id.as_str();
        let (phase_name, cache_state, warm_state) = match phase {
            crate::benchmark::experiment::model::AdoptionPhase::Initial => (
                "initial",
                crate::benchmark::experiment::model::CacheState::Cold,
                crate::benchmark::experiment::model::WarmState::Cold,
            ),
            crate::benchmark::experiment::model::AdoptionPhase::Incremental => (
                "incremental",
                crate::benchmark::experiment::model::CacheState::Warm,
                crate::benchmark::experiment::model::WarmState::Warm,
            ),
        };
        let Some(program) = argv.first() else {
            return unavailable(measurement, target, phase, "adoption command is empty");
        };
        let Some(executable) = super::shell::resolve_executable(program, measurement.runtime) else {
            return unavailable(
                measurement,
                target,
                phase,
                &format!("executable `{program}` is not available on PATH"),
            );
        };
        let command_args = argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
        let command = super::shell::command(&executable, command_args, self.root, environment);
        let outcome = super::shell::execute(
            command,
            super::shell::limits(super::shell::ADOPTION_TIMEOUT_MS),
            &measurement.runtime.clock,
        );
        let (stdout, stderr, exit_code, is_measured, duration, reason) = match outcome {
            Ok(outcome) => (
                outcome.stdout,
                outcome.stderr,
                outcome.exit_code,
                outcome.succeeded,
                Some(outcome.duration_ns),
                outcome.reason,
            ),
            Err(error) => (String::new(), String::new(), None, false, None, Some(error)),
        };
        crate::benchmark::experiment::model::AdoptionObservation {
            observation_id: format!("{}:{selection_id}:{phase_name}", tool.tool_id),
            tool_id: tool.tool_id.clone(),
            target_selection: selection_id.to_owned(),
            phase,
            origin: crate::benchmark::experiment::model::MeasurementOrigin::Local,
            measured: is_measured,
            wall_duration_ns: duration,
            warm_state,
            environment: super::shell::environment_observation(profile, cache_state),
            stdout_identity: crate::benchmark::prefixed_identity(stdout.as_bytes()),
            stderr_identity: crate::benchmark::prefixed_identity(stderr.as_bytes()),
            stdout,
            stderr,
            exit_code,
            unavailable_reason: reason,
            dependencies: profile.sample_plan.dependency_differences.clone(),
        }
    }
}

fn unavailable(
    measurement: &Measurement<'_>,
    target: &crate::benchmark::experiment::model::AdoptionTarget,
    phase: crate::benchmark::experiment::model::AdoptionPhase,
    reason: &str,
) -> crate::benchmark::experiment::model::AdoptionObservation {
    let profile = measurement.profile;
    let tool = measurement.tool;
    let selection_id = target.selection_id.as_str();
    let phase_name = match phase {
        crate::benchmark::experiment::model::AdoptionPhase::Initial => "initial",
        crate::benchmark::experiment::model::AdoptionPhase::Incremental => "incremental",
    };
    let cache_state = match phase {
        crate::benchmark::experiment::model::AdoptionPhase::Initial => {
            crate::benchmark::experiment::model::CacheState::Cold
        }
        crate::benchmark::experiment::model::AdoptionPhase::Incremental => {
            crate::benchmark::experiment::model::CacheState::Warm
        }
    };
    crate::benchmark::experiment::model::AdoptionObservation {
        observation_id: format!("{}:{selection_id}:{phase_name}", tool.tool_id),
        tool_id: tool.tool_id.clone(),
        target_selection: selection_id.to_owned(),
        phase,
        origin: crate::benchmark::experiment::model::MeasurementOrigin::RecordedFixture,
        measured: false,
        wall_duration_ns: None,
        warm_state: match phase {
            crate::benchmark::experiment::model::AdoptionPhase::Initial => {
                crate::benchmark::experiment::model::WarmState::Cold
            }
            crate::benchmark::experiment::model::AdoptionPhase::Incremental => {
                crate::benchmark::experiment::model::WarmState::Warm
            }
        },
        environment: super::shell::environment_observation(profile, cache_state),
        stdout: String::new(),
        stderr: String::new(),
        stdout_identity: crate::benchmark::prefixed_identity(b""),
        stderr_identity: crate::benchmark::prefixed_identity(b""),
        exit_code: None,
        unavailable_reason: Some(reason.to_owned()),
        dependencies: profile.sample_plan.dependency_differences.clone(),
    }
}
