#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceSnapshot {
    pub files: Vec<super::SourceFile>,
    pub identity: String,
    pub selected_paths: Vec<String>,
    pub missing_paths: Vec<String>,
}

/// Resolved executable and its exact bytes identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableObservation {
    pub resolved_path: String,
    pub identity: String,
}

/// One completed child process observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StepOutput {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FailureKind {
    /// The executable name does not resolve on the declared search path.
    NotFound,
    /// The process could not be spawned or its executable could not be inspected.
    Spawn,
    /// The process outlived the declared wall-clock deadline.
    TimedOut,
    /// The process wrote more than the declared per-stream capture limit.
    OutputLimitExceeded,
    /// The process was cancelled.
    Cancelled,
    /// A signal terminated the process before it exited.
    Terminated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StepFailure {
    pub kind: FailureKind,
    pub message: String,
}

impl StepFailure {
    pub(super) fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Process mechanics the producer shell needs. The production implementation is Bounded Exec.
/// Tests use deliberately labeled fixed-SARIF fixtures; retained real-tool evidence is separate.
pub(crate) trait Executor {
    fn resolve(&self, name: &str, current_dir: &std::path::Path) -> Result<ExecutableObservation, StepFailure>;
    fn run(
        &self,
        argv: &[String],
        current_dir: &std::path::Path,
        limits: &super::Limits,
    ) -> Result<StepOutput, StepFailure>;
}

pub(crate) struct ExecuteRequest<'a> {
    pub workspace_root: &'a std::path::Path,
    pub stage_dir: &'a std::path::Path,
}

pub(crate) fn execute(
    profile: &super::Profile,
    request: &ExecuteRequest<'_>,
    executor: &dyn Executor,
) -> Result<(super::Execution, String), String> {
    let (report_path, database_path) = super::command::stage_paths(request.stage_dir);
    std::fs::create_dir_all(request.stage_dir)
        .map_err(|error| format!("create SARIF producer stage directory {}: {error}", request.stage_dir.display()))?;
    super::selection::remove_if_present(&report_path)?;
    let mut execution = initial_execution(profile);
    if !prepare_sources(profile, request.workspace_root, &mut execution)
        || !probe_executable(profile, request.workspace_root, executor, &mut execution)
        || !super::codeql::attach(profile, request, executor, &database_path, &mut execution)?
    {
        return Ok((super::recording::finish_diagnostics(execution), String::new()));
    }
    if !super::analysis::run(
        profile,
        request,
        executor,
        &super::analysis::Paths {
            report: &report_path,
            database: &database_path,
        },
        &mut execution,
    )? {
        return Ok((super::recording::finish_diagnostics(execution), String::new()));
    }
    match super::analysis::ingest(profile, request.workspace_root, &report_path, &mut execution) {
        Ok(report) => {
            execution.outcome = super::OUTCOME_COMPLETED.to_owned();
            Ok((execution, report))
        }
        Err((outcome, diagnostics)) => {
            execution.outcome = outcome.to_owned();
            execution.diagnostics = diagnostics;
            Ok((super::recording::finish_diagnostics(execution), String::new()))
        }
    }
}

fn initial_execution(profile: &super::Profile) -> super::Execution {
    super::Execution {
        schema_version: super::EXECUTION_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: super::PROFILE_IDENTITY_VALUE.to_owned(),
        producer: profile.producer.clone(),
        driver_name: profile.driver_name.clone(),
        intake_profile_id: profile.intake_profile_id.clone(),
        intake_profile_identity: super::super::PROFILE_IDENTITY.to_owned(),
        evidence_origin: super::EVIDENCE_ORIGIN_EXECUTED.to_owned(),
        evidence_tier: "static_lint".to_owned(),
        supported_languages: profile.supported_languages.clone(),
        executable: None,
        rules: super::recording::rule_receipt(&profile.rules),
        query_packs: profile.query_packs.iter().map(super::recording::query_pack_receipt).collect(),
        scope: super::ScopeReceipt {
            declared_paths: profile.scope.paths.clone(),
            selected_paths: Vec::new(),
            missing_paths: Vec::new(),
            extensions: profile.scope.extensions.clone(),
            exclude_globs: profile.scope.exclude_globs.clone(),
        },
        rust: super::RustScopeReceipt {
            build_mode: profile.rust.build_mode.clone(),
            target: profile.rust.target.clone(),
            features: profile.rust.features.clone(),
            packages: profile.rust.packages.clone(),
            unsupported_features: profile.rust.unsupported_features.clone(),
        },
        source_snapshot_identity: String::new(),
        source_files: Vec::new(),
        limits: super::recording::limit_receipt(&profile.limits),
        steps: Vec::new(),
        database: None,
        report: None,
        outcome: super::OUTCOME_TOOL_ERROR.to_owned(),
        diagnostics: Vec::new(),
    }
}

fn prepare_sources(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
    execution: &mut super::Execution,
) -> bool {
    let snapshot = match super::selection::snapshot_sources(profile, workspace_root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            execution.outcome = super::OUTCOME_UNSUPPORTED.to_owned();
            execution.diagnostics = vec![error];
            return false;
        }
    };
    execution.scope.selected_paths = snapshot.selected_paths;
    execution.scope.missing_paths = snapshot.missing_paths;
    execution.source_snapshot_identity = snapshot.identity;
    execution.source_files = snapshot.files;
    if execution.source_files.is_empty() {
        execution.outcome = super::OUTCOME_UNSUPPORTED.to_owned();
        execution.diagnostics = vec![
            "no Rust source file matches the declared SARIF producer scope; an empty scan is not clean evidence"
                .to_owned(),
        ];
        return false;
    }
    if let Err(error) = super::selection::verify_rule_bytes(profile, workspace_root) {
        execution.outcome = super::OUTCOME_TOOL_ERROR.to_owned();
        execution.diagnostics = vec![error];
        return false;
    }
    true
}

fn probe_executable(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
    executor: &dyn Executor,
    execution: &mut super::Execution,
) -> bool {
    let executable = match executor.resolve(&profile.executable.name, workspace_root) {
        Ok(executable) => executable,
        Err(failure) => {
            execution.outcome = super::OUTCOME_UNSUPPORTED.to_owned();
            execution.diagnostics = vec![failure.message];
            return false;
        }
    };
    let argv = super::command::version_argv(profile);
    let probe = match executor.run(&argv, workspace_root, &profile.limits) {
        Ok(output) => output,
        Err(failure) => {
            super::recording::record_failure(execution, failure, "version", argv);
            return false;
        }
    };
    let result = super::command::probe_version(profile, &probe);
    let observed_version = match result {
        Ok(version) => version,
        Err((outcome, message)) => {
            execution.outcome = outcome.to_owned();
            execution.diagnostics = vec![message];
            String::new()
        }
    };
    let is_valid = execution.diagnostics.is_empty();
    execution.executable = Some(super::ExecutableReceipt {
        name: profile.executable.name.clone(),
        declared_version: profile.executable.declared_version.clone(),
        observed_version,
        resolved_path: executable.resolved_path,
        identity: executable.identity,
    });
    execution.steps.push(super::StepReceipt {
        step: "version".to_owned(),
        argv,
        exit_code: Some(probe.exit_code),
        status: if is_valid {
            super::STATUS_SUCCEEDED
        } else {
            super::STATUS_FAILED
        }
        .to_owned(),
    });
    is_valid
}
