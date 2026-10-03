use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
#[cfg(unix)]
use cap_std::fs::MetadataExt;
#[cfg(unix)]
use cap_std::fs::OpenOptions;
#[cfg(unix)]
use cap_std::fs::OpenOptionsExt;

use crate::baseline;
use crate::baseline::BaselineEvaluation;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::CacheRoot;
use crate::cap_fs::RootRelativePath;
use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::config;
use crate::determinism;
use crate::determinism::RunMetadata;
use crate::output;
use crate::output::Finding;

mod embedded_source {
    include!(concat!(env!("OUT_DIR"), "/embedded_source.rs"));
}

const SOURCE_HASH_MARKER: &str = ".tigerstyle-source-blake3";
const CARGO_COMMAND: &str = "cargo";
const CARGO_CHECK_SUBCOMMAND: &str = "check";
const CARGO_MESSAGE_FORMAT_JSON: &str = "--message-format=json";
const COLOR_ALWAYS_FLAG: &str = "--color=always";
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const JSON_RESULTS_ARTIFACT_NAME: &str = "results.jsonl";
const SARIF_RESULTS_ARTIFACT_NAME: &str = "results.sarif";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const RUNNER_PROVENANCE_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const RUNNER_STATIC_FINDINGS_SOURCE_ID: &str = "octet.runner.static-findings";
const FUNCTION_ADDRESS_PROVENANCE_SOURCE_ID: &str = "octet.function_address.receipt";
const ARCHITECTURE_IR_PROVENANCE_SOURCE_ID: &str = "octet.compiler-architecture-ir.receipt";
const RUNNER_CHECKER_ID: &str = "cargo-octet";
const RUNNER_MANUAL_REVIEW_ACTION: &str =
    "review runner status, command, and summary before using this artifact as release evidence";
const TERMINATED_WITHOUT_EXIT_CODE_ERROR: &str = "cargo check terminated without an exit code";
const LINT_CACHE_DIR: &str = ".octet/cache";
const LINT_CACHE_FILE: &str = "findings.json";

#[derive(Debug, Clone)]
struct StructuredArtifact {
    file_name: &'static str,
    contents: String,
}

#[derive(Debug, Clone)]
struct RunArtifacts {
    summary: String,
    status_json: String,
    provenance_jsonl: String,
    structured_output: Option<StructuredArtifact>,
    additional_artifacts: Vec<StructuredArtifact>,
}

#[derive(Clone)]
struct ArchitectureRunArtifacts {
    compiler: Option<crate::architecture_ir::ArchitectureArtifacts>,
    project: Option<crate::project_architecture::ProjectArchitectureArtifacts>,
    closure: Option<crate::admit::IntegratedClosureArtifacts>,
}

#[derive(Debug, Clone)]
struct FinalizedRun {
    outcome: output::RunOutcome,
    run_artifacts: RunArtifacts,
    error: Option<String>,
}

fn apply_baseline_gate(
    outcome: output::RunOutcome,
    baseline_evaluation: Option<&BaselineEvaluation>,
) -> output::RunOutcome {
    if outcome.cargo_process_exit.is_success() && baseline_evaluation.is_some_and(BaselineEvaluation::has_new_findings)
    {
        return outcome.force_lint_failure();
    }

    outcome
}

#[allow(clippy::too_many_lines)]
pub fn run(check: &CheckArgs) -> Result<i32, String> {
    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let cfg = config::load(&cwd)?;
    if let Some(policy) = &cfg.architecture_policy {
        policy.validate_loaded_binding()?;
    }
    let scope_args = if check.scope_args.is_empty() {
        cfg.default_scope.clone()
    } else {
        check.scope_args.clone()
    };
    let cargo_check_args = check.cargo_check_args.clone().unwrap_or(cfg.cargo_check_args.clone());
    let admission_binding = cfg.architecture_policy.as_ref().and_then(|policy| policy.policy.admission.as_ref());
    if let (Some(requested), Some(binding)) = (check.profile.as_deref(), admission_binding)
        && requested != binding.strict_profile
    {
        return Err(format!(
            "architecture admission binds strict profile `{}`, but `--profile` selected `{requested}`",
            binding.strict_profile
        ));
    }
    let selected_profile_id = check
        .profile
        .as_deref()
        .or_else(|| admission_binding.map(|binding| binding.strict_profile.as_str()));
    let selected_profile = crate::strict_profile::prepare_selected_profile(selected_profile_id, &cfg.workspace_root)?;
    if selected_profile
        .as_ref()
        .is_some_and(|profile| profile.policy.required_lints.iter().any(|lint| lint == "raw_bool_parameter"))
        && (check.baseline.is_some() || check.write_baseline.is_some())
    {
        return Err("strict boolean API admission forbids finding baselines and warning budgets".to_owned());
    }
    let prepared_integrated_closure = admission_binding
        .map(|binding| crate::admit::prepare_integrated_closure(&cfg.workspace_root, binding))
        .transpose()?;
    let mut requested_function_addresses = check.function_addresses.clone();
    if let Some(prepared) = &prepared_integrated_closure {
        requested_function_addresses.baseline = Some(prepared.baseline_path.clone());
    }
    let effective_function_addresses = crate::strict_profile::effective_function_address_check(
        selected_profile.as_ref(),
        &requested_function_addresses,
    );

    if cargo_check_args.iter().any(|arg| arg.starts_with("--message-format")) {
        return Err("`--message-format` is managed by `--output-format`; remove it from passthrough args".to_owned());
    }

    let lockfile_path = cfg.workspace_root.join("Cargo.lock");
    let skip_lockfile_guard =
        env::var("TIGERSTYLE_SKIP_LOCKFILE_CHECK").is_ok_and(|value| !value.is_empty() && value != "0");
    let lockfile_before = if skip_lockfile_guard {
        LockfileSnapshot::Missing
    } else {
        snapshot_lockfile(&lockfile_path)?
    };

    let result = (|| {
        let toolchain = resolve_toolchain()?;
        let strict_profile_identity = selected_profile.as_ref().map(|selected| determinism::StrictProfileIdentity {
            profile_id: &selected.policy.profile_id,
            policy_hash: &selected.policy_hash,
        });
        let run_metadata = determinism::build_run_metadata(&determinism::MetadataInput {
            workspace_root: &cfg.workspace_root,
            toolchain: &toolchain,
            scope_args: &scope_args,
            cargo_check_args: &cargo_check_args,
            output_format: check.output_format.as_str(),
            strict_profile: strict_profile_identity,
        });
        let driver = resolve_driver(&toolchain)?;
        let lint_lib = resolve_lint_lib(&toolchain)?;
        let collector = cfg
            .architecture_policy
            .as_ref()
            .map(|policy| {
                crate::architecture_ir::prepare_collector(
                    &cfg.workspace_root,
                    policy,
                    &toolchain,
                    Path::new(lint_lib.path()),
                    &scope_args,
                    &cargo_check_args,
                )
            })
            .transpose()?;
        let dylint_libs = serde_json::to_string(&vec![lint_lib.path().to_owned()])
            .map_err(|error| format!("encode DYLINT_LIBS: {error}"))?;
        let diagnostic_paths =
            DiagnosticPaths::from_cargo_metadata(&cfg.workspace_root, &toolchain, &scope_args, &cargo_check_args)?;

        let mut cmd = Command::new(CARGO_COMMAND);
        cmd.arg(CARGO_CHECK_SUBCOMMAND).arg(CARGO_MESSAGE_FORMAT_JSON);

        if check.output_format == OutputFormat::Human
            && std::io::stderr().is_terminal()
            && !cargo_check_args.iter().any(|argument| argument.starts_with("--color"))
        {
            cmd.arg(COLOR_ALWAYS_FLAG);
        }

        cmd.args(&scope_args)
            .args(&cargo_check_args)
            .current_dir(&cfg.workspace_root)
            .env("RUSTC_WORKSPACE_WRAPPER", driver.path())
            .env("DYLINT_LIBS", &dylint_libs)
            .env("DYLINT_NO_DEPS", "1")
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .stdout(Stdio::piped());
        cmd.env_remove(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV);
        cmd.env_remove(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV);
        if let Some(collector) = &collector {
            debug_assert!(collector.shard_directory.is_absolute());
            cmd.env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &collector.context_path);
        }
        if let Some(selected) = &selected_profile {
            cmd.env(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV, &selected.policy.profile_id);
        }

        if check.output_format != OutputFormat::Human {
            cmd.stderr(Stdio::null());
        }

        let mut child = cmd.spawn().map_err(|error| format!("run cargo check: {error}"))?;
        let child_stdout = child.stdout.take().expect("stdout piped");

        let meta_map = output::lint_meta_map();
        let mut findings: Vec<Finding> = Vec::new();
        let mut finding_counter: usize = 0;
        let mut stdout_lock = std::io::stdout().lock();
        let mut stderr_lock = std::io::stderr().lock();

        for line in BufReader::new(child_stdout).lines() {
            let line = line.map_err(|error| format!("read cargo stdout: {error}"))?;
            let json: serde_json::Value =
                serde_json::from_str(&line).map_err(|error| format!("parse cargo JSON: {error}"))?;

            let reason = json["reason"].as_str().unwrap_or("");

            match reason {
                "compiler-message" => {
                    let message = &json["message"];

                    if check.output_format == OutputFormat::Human
                        && let Some(rendered) = message["rendered"].as_str()
                    {
                        let _ = write!(stderr_lock, "{rendered}");
                    }

                    let code = message["code"]["code"].as_str().unwrap_or("");
                    let lint_name = normalize_lint_name(code);

                    let meta = meta_map.get(lint_name.as_str()).unwrap_or(&output::UNKNOWN_LINT_META);
                    finding_counter += 1;
                    findings.push(build_finding(
                        finding_counter,
                        &json,
                        meta,
                        &lint_name,
                        &run_metadata.config_hash,
                        check.cache,
                        &diagnostic_paths,
                    )?);
                }
                "compiler-artifact" if check.output_format == OutputFormat::Human => {
                    if let Some(banner) = format_compiling_banner(&json) {
                        let _ = writeln!(stderr_lock, "{banner}");
                    }
                }
                "build-finished" if check.output_format == OutputFormat::Human => {
                    let success = json["success"].as_bool().unwrap_or(false);
                    if success {
                        let _ = writeln!(stderr_lock, "    Finished checking");
                    }
                }
                _ => {}
            }
        }

        let cargo_process_exit =
            classify_cargo_process_exit(child.wait().map_err(|error| format!("wait for cargo check: {error}"))?);
        let architecture_artifacts = if cargo_process_exit.is_success() {
            collector.as_ref().map(crate::architecture_ir::normalize_collected_shards).transpose()?
        } else {
            None
        };
        let function_address_artifact = if cargo_process_exit.is_success() {
            match crate::function_address::collect_run(
                &effective_function_addresses,
                &cfg.workspace_root,
                &run_metadata.config_hash,
            )? {
                Some(run) => {
                    findings.extend(run.findings);
                    Some(StructuredArtifact {
                        file_name: crate::function_address::FUNCTION_ADDRESS_ARTIFACT_NAME,
                        contents: run.artifact_json,
                    })
                }
                None => None,
            }
        } else {
            None
        };
        canonicalize_findings(&mut findings);
        if check.cache {
            update_lint_cache(&cfg.workspace_root, &mut findings)?;
        }
        let cargo_process_succeeded = cargo_process_exit.is_success();
        let base_outcome = output::RunOutcome::classify(&findings, cargo_process_exit);
        let baseline_evaluation = match check.baseline.as_deref() {
            Some(path) => Some(baseline::evaluate_baseline(path, &findings)?),
            None => None,
        };
        if let Some(path) = check.write_baseline.as_deref()
            && cargo_process_succeeded
        {
            baseline::write_baseline(path, &findings, &run_metadata)?;
        }
        let architecture_error =
            architecture_artifacts.as_ref().and_then(|artifacts| artifacts.incomplete_error.clone());
        let lint_outcome = apply_baseline_gate(base_outcome, baseline_evaluation.as_ref());
        let mut project_architecture_artifacts =
            match (cfg.architecture_policy.as_ref(), collector.as_ref(), architecture_artifacts.as_ref()) {
                (Some(policy), Some(prepared), Some(architecture)) if architecture.incomplete_error.is_none() => {
                    Some(crate::project_architecture::evaluate(
                        policy,
                        prepared,
                        architecture,
                        &check.command_line,
                        lint_outcome.status.as_str(),
                        None,
                    )?)
                }
                _ => None,
            };
        let mut integrated_closure_artifacts = None;
        if lint_outcome.status == output::RunStatus::Clean
            && project_architecture_artifacts.as_ref().is_some_and(|project| {
                project.receipt.status
                    == octet_standards::project_architecture_gate::ArchitecturePhaseStatus::ClosureRequired
            })
        {
            let binding = admission_binding.ok_or_else(|| "architecture admission binding disappeared".to_owned())?;
            let selected = selected_profile
                .as_ref()
                .ok_or_else(|| "architecture admission requires its selected strict profile".to_owned())?;
            let function_json = function_address_artifact
                .as_ref()
                .map(|artifact| artifact.contents.as_str())
                .ok_or_else(|| "architecture admission requires deny-mode function-address evidence".to_owned())?;
            let prepared_closure = prepared_integrated_closure
                .ok_or_else(|| "architecture admission closure policy was not prepared before Cargo".to_owned())?;
            let pre_closure_architecture = project_architecture_artifacts
                .as_ref()
                .ok_or_else(|| "architecture admission prerequisite receipt disappeared".to_owned())?;
            let parent_status_json = output::render_integrated_status_json(
                &lint_outcome,
                &lint_outcome,
                check.output_format.as_str(),
                &run_metadata,
                baseline_evaluation.as_ref(),
                &pre_closure_architecture.receipt,
            )?;
            let closure = crate::admit::run_integrated_closure(
                &cfg.workspace_root,
                binding,
                selected,
                &cargo_check_args,
                &run_metadata,
                &check.command_line,
                &parent_status_json,
                function_json,
                prepared_closure,
            )?;
            project_architecture_artifacts =
                match (cfg.architecture_policy.as_ref(), collector.as_ref(), architecture_artifacts.as_ref()) {
                    (Some(policy), Some(prepared), Some(architecture)) => Some(crate::project_architecture::evaluate(
                        policy,
                        prepared,
                        architecture,
                        &check.command_line,
                        lint_outcome.status.as_str(),
                        Some(&closure.parent),
                    )?),
                    _ => return Err("architecture admission inputs disappeared before closure composition".to_owned()),
                };
            integrated_closure_artifacts = Some(closure);
        }
        let base_outcome = if architecture_error.is_some() {
            lint_outcome.force_integration_failure()
        } else if let Some(project) = &project_architecture_artifacts {
            match project.receipt.status {
                octet_standards::project_architecture_gate::ArchitecturePhaseStatus::IntegrationFailure => {
                    lint_outcome.force_integration_failure()
                }
                octet_standards::project_architecture_gate::ArchitecturePhaseStatus::Blocked
                | octet_standards::project_architecture_gate::ArchitecturePhaseStatus::ClosureRequired => {
                    lint_outcome.force_lint_failure()
                }
                octet_standards::project_architecture_gate::ArchitecturePhaseStatus::Findings
                    if lint_outcome.status == output::RunStatus::Clean =>
                {
                    lint_outcome.force_warning_only()
                }
                _ => lint_outcome.clone(),
            }
        } else {
            lint_outcome.clone()
        };
        let architecture_run_artifacts = ArchitectureRunArtifacts {
            compiler: architecture_artifacts,
            project: project_architecture_artifacts,
            closure: integrated_closure_artifacts,
        };
        let finalized_run = finalize_run(
            check,
            &findings,
            &meta_map,
            base_outcome,
            &run_metadata,
            baseline_evaluation.as_ref(),
            function_address_artifact,
            architecture_run_artifacts,
            &lint_outcome,
            architecture_error,
        )?;

        if let Some(structured_output) = finalized_run.run_artifacts.structured_output.as_ref() {
            stdout_lock
                .write_all(structured_output.contents.as_bytes())
                .map_err(|error| format!("write stdout: {error}"))?;
        }

        stderr_lock
            .write_all(finalized_run.run_artifacts.summary.as_bytes())
            .map_err(|error| format!("write summary: {error}"))?;

        match finalized_run.error {
            Some(error) => Err(error),
            None => Ok(finalized_run.outcome.exit_code),
        }
    })();

    if !skip_lockfile_guard && lockfile_changed(&lockfile_path, &lockfile_before)? {
        restore_lockfile(&lockfile_path, &lockfile_before)?;
        let lockfile_error =
            format!("cargo octet check modified {}; restored the original lockfile", lockfile_path.display());
        return Err(match result {
            Err(error) => format!("{error}; {lockfile_error}"),
            Ok(_) => lockfile_error,
        });
    }

    result
}

fn build_run_artifacts(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    outcome: &output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
    function_address_artifact: Option<StructuredArtifact>,
    architecture_artifacts: ArchitectureRunArtifacts,
    lint_outcome: &output::RunOutcome,
) -> Result<RunArtifacts, String> {
    let ArchitectureRunArtifacts {
        compiler,
        project,
        closure,
    } = architecture_artifacts;
    let summary = match &project {
        Some(project) => output::render_integrated_summary(
            findings,
            outcome,
            lint_outcome,
            metadata,
            baseline_evaluation,
            &project.receipt,
        )?,
        None => output::render_summary(findings, outcome, metadata, baseline_evaluation)?,
    };
    let status_json = match &project {
        Some(project) => output::render_integrated_status_json(
            outcome,
            lint_outcome,
            check.output_format.as_str(),
            metadata,
            baseline_evaluation,
            &project.receipt,
        )?,
        None => output::render_status_json(outcome, check.output_format.as_str(), metadata, baseline_evaluation)?,
    };
    let structured_output = render_structured_output(
        check,
        findings,
        meta_map,
        outcome,
        lint_outcome,
        metadata,
        baseline_evaluation,
        project.as_ref(),
    )?;
    let mut additional_artifacts = function_address_artifact.into_iter().collect::<Vec<_>>();
    if let Some(architecture) = compiler {
        additional_artifacts.extend([
            StructuredArtifact {
                file_name: crate::architecture_ir::ARCHITECTURE_IR_ARTIFACT_NAME,
                contents: architecture.architecture_ir,
            },
            StructuredArtifact {
                file_name: crate::architecture_ir::ARCHITECTURE_COVERAGE_ARTIFACT_NAME,
                contents: architecture.coverage,
            },
            StructuredArtifact {
                file_name: crate::architecture_ir::ARCHITECTURE_ISSUES_ARTIFACT_NAME,
                contents: architecture.issues,
            },
            StructuredArtifact {
                file_name: crate::architecture_ir::ARCHITECTURE_RECEIPT_ARTIFACT_NAME,
                contents: architecture.receipt,
            },
        ]);
    }
    if let Some(project) = project {
        additional_artifacts.extend([
            StructuredArtifact {
                file_name: crate::project_architecture::PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME,
                contents: project.policy_source,
            },
            StructuredArtifact {
                file_name: crate::project_architecture::PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME,
                contents: project.policy_json,
            },
            StructuredArtifact {
                file_name: crate::project_architecture::PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME,
                contents: project.cargo_graph_json,
            },
            StructuredArtifact {
                file_name: crate::project_architecture::PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME,
                contents: project.receipt_json,
            },
        ]);
    }
    if let Some(closure) = closure {
        additional_artifacts.extend([
            StructuredArtifact {
                file_name: crate::admit::INTEGRATED_CLOSURE_RECEIPT_ARTIFACT_NAME,
                contents: closure.receipt_json,
            },
            StructuredArtifact {
                file_name: crate::admit::INTEGRATED_CLOSURE_POLICY_ARTIFACT_NAME,
                contents: closure.policy_json,
            },
            StructuredArtifact {
                file_name: crate::admit::INTEGRATED_CLOSURE_GRAPH_ARTIFACT_NAME,
                contents: closure.dependency_graph_json,
            },
            StructuredArtifact {
                file_name: crate::admit::INTEGRATED_CLOSURE_PARENT_STATUS_ARTIFACT_NAME,
                contents: closure.parent_status_json,
            },
        ]);
    }
    let provenance_jsonl = render_run_provenance_jsonl(
        &check.command_line,
        &summary,
        &status_json,
        structured_output.as_ref(),
        &additional_artifacts,
        metadata,
    )?;

    Ok(RunArtifacts {
        summary,
        status_json,
        provenance_jsonl,
        structured_output,
        additional_artifacts,
    })
}

fn finalize_run(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    base_outcome: output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
    function_address_artifact: Option<StructuredArtifact>,
    architecture_artifacts: ArchitectureRunArtifacts,
    lint_outcome: &output::RunOutcome,
    architecture_error: Option<String>,
) -> Result<FinalizedRun, String> {
    let terminated_error = architecture_error.or_else(|| terminated_run_error(&base_outcome));

    let Some(artifact_dir) = check.artifact_dir.as_deref() else {
        return Ok(FinalizedRun {
            outcome: base_outcome.clone(),
            run_artifacts: build_run_artifacts(
                check,
                findings,
                meta_map,
                &base_outcome,
                metadata,
                baseline_evaluation,
                function_address_artifact,
                architecture_artifacts,
                lint_outcome,
            )?,
            error: terminated_error,
        });
    };

    let artifact_root = match prepare_artifact_dir(artifact_dir) {
        Ok(root) => root,
        Err(error) => {
            let final_outcome = base_outcome.force_integration_failure();
            return Ok(FinalizedRun {
                outcome: final_outcome.clone(),
                run_artifacts: build_run_artifacts(
                    check,
                    findings,
                    meta_map,
                    &final_outcome,
                    metadata,
                    baseline_evaluation,
                    function_address_artifact,
                    architecture_artifacts,
                    lint_outcome,
                )?,
                error: Some(combine_errors(error, terminated_error)),
            });
        }
    };

    let initial_artifacts = build_run_artifacts(
        check,
        findings,
        meta_map,
        &base_outcome,
        metadata,
        baseline_evaluation,
        function_address_artifact.clone(),
        architecture_artifacts.clone(),
        lint_outcome,
    )?;
    match write_run_artifacts(&artifact_root, &check.command_line, &initial_artifacts) {
        Ok(()) => Ok(FinalizedRun {
            outcome: base_outcome,
            run_artifacts: initial_artifacts,
            error: terminated_error,
        }),
        Err(error) => {
            let final_outcome = base_outcome.force_integration_failure();
            let final_artifacts = build_run_artifacts(
                check,
                findings,
                meta_map,
                &final_outcome,
                metadata,
                baseline_evaluation,
                function_address_artifact,
                architecture_artifacts,
                lint_outcome,
            )?;
            invalidate_failed_bundle_best_effort(&artifact_root, &final_artifacts);
            rewrite_status_sidecars_best_effort(&artifact_root, &check.command_line, &final_artifacts);
            Ok(FinalizedRun {
                outcome: final_outcome,
                run_artifacts: final_artifacts,
                error: Some(combine_errors(error, terminated_error)),
            })
        }
    }
}

fn terminated_run_error(outcome: &output::RunOutcome) -> Option<String> {
    if outcome.cargo_process_exit.is_terminated() {
        Some(TERMINATED_WITHOUT_EXIT_CODE_ERROR.to_owned())
    } else {
        None
    }
}

fn combine_errors(primary: String, secondary: Option<String>) -> String {
    match secondary {
        Some(secondary) => format!("{primary}; {secondary}"),
        None => primary,
    }
}

fn render_structured_output(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    outcome: &output::RunOutcome,
    lint_outcome: &output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
    project_architecture: Option<&crate::project_architecture::ProjectArchitectureArtifacts>,
) -> Result<Option<StructuredArtifact>, String> {
    match check.output_format {
        OutputFormat::Human => Ok(None),
        OutputFormat::Json => Ok(Some(StructuredArtifact {
            file_name: JSON_RESULTS_ARTIFACT_NAME,
            contents: match project_architecture {
                Some(project) => output::render_integrated_json(findings, lint_outcome, &project.receipt)?,
                None => output::render_json_lines(findings)?,
            },
        })),
        OutputFormat::Sarif => Ok(Some(StructuredArtifact {
            file_name: SARIF_RESULTS_ARTIFACT_NAME,
            contents: match project_architecture {
                Some(project) => output::render_integrated_sarif(
                    findings,
                    meta_map,
                    outcome,
                    lint_outcome,
                    &check.command_line,
                    metadata,
                    baseline_evaluation,
                    &project.receipt,
                )?,
                None => output::render_sarif(
                    findings,
                    meta_map,
                    outcome,
                    &check.command_line,
                    metadata,
                    baseline_evaluation,
                )?,
            },
        })),
    }
}

fn render_run_provenance_jsonl(
    command_line: &str,
    summary: &str,
    status_json: &str,
    structured_output: Option<&StructuredArtifact>,
    additional_artifacts: &[StructuredArtifact],
    metadata: &RunMetadata,
) -> Result<String, String> {
    let command_contents = format!("{command_line}\n");
    let checked_scope = format!("profile:{}", metadata.profile_name);
    let mut records = vec![
        artifact_provenance_record(
            COMMAND_ARTIFACT_NAME,
            &command_contents,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
        artifact_provenance_record(
            STATUS_ARTIFACT_NAME,
            status_json,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
        artifact_provenance_record(
            SUMMARY_ARTIFACT_NAME,
            summary,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
    ];

    if let Some(structured_output) = structured_output {
        records.push(artifact_provenance_record(
            structured_output.file_name,
            &structured_output.contents,
            crate::evidence::EvidenceTier::StaticLint,
            RUNNER_STATIC_FINDINGS_SOURCE_ID,
            &checked_scope,
        ));
    }

    for artifact in additional_artifacts {
        let (source_id, evidence_tier) = if artifact.file_name.starts_with("compiler-architecture-") {
            (ARCHITECTURE_IR_PROVENANCE_SOURCE_ID, crate::evidence::EvidenceTier::StaticLint)
        } else if artifact.file_name.starts_with("project-architecture-") {
            (
                crate::project_architecture::PROJECT_ARCHITECTURE_PROVENANCE_SOURCE_ID,
                crate::evidence::EvidenceTier::PolicyCheck,
            )
        } else if artifact.file_name.starts_with("effect-closure") {
            (crate::admit::INTEGRATED_CLOSURE_PROVENANCE_SOURCE_ID, crate::evidence::EvidenceTier::PolicyCheck)
        } else {
            (FUNCTION_ADDRESS_PROVENANCE_SOURCE_ID, crate::evidence::EvidenceTier::StaticLint)
        };
        records.push(artifact_provenance_record(
            artifact.file_name,
            &artifact.contents,
            evidence_tier,
            source_id,
            &checked_scope,
        ));
    }

    let mut rendered = String::new();
    for record in records {
        let line = serde_json::to_string(&record).map_err(|error| format!("write provenance JSON: {error}"))?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn artifact_provenance_record(
    file_name: &str,
    contents: &str,
    evidence_tier: crate::evidence::EvidenceTier,
    source_id: &str,
    checked_scope: &str,
) -> crate::evidence::ProvenanceRecord {
    crate::evidence::ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source_id.to_owned(),
        checker_id: RUNNER_CHECKER_ID.to_owned(),
        evidence_tier,
        artifact_locator: file_name.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents.as_bytes()),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: (evidence_tier == crate::evidence::EvidenceTier::ManualReviewRequired)
            .then(|| RUNNER_MANUAL_REVIEW_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    }
}

fn write_run_artifacts(
    artifact_root: &ArtifactRoot,
    command_line: &str,
    run_artifacts: &RunArtifacts,
) -> Result<(), String> {
    let expected_files = expected_artifact_files(run_artifacts);
    remove_extra_artifact_paths(artifact_root, &expected_files)?;
    write_status_sidecars(artifact_root, command_line, run_artifacts)?;

    if let Some(structured_output) = run_artifacts.structured_output.as_ref() {
        let structured_path = RootRelativePath::new(structured_output.file_name)?;
        artifact_root.write(&structured_path, structured_output.contents.as_bytes())?;
    }

    for artifact in &run_artifacts.additional_artifacts {
        let path = RootRelativePath::new(artifact.file_name)?;
        artifact_root.write(&path, artifact.contents.as_bytes())?;
    }

    Ok(())
}

fn prepare_artifact_dir(artifact_dir: &Path) -> Result<ArtifactRoot, String> {
    ArtifactRoot::open_or_create(artifact_dir)
}

fn write_status_sidecars(
    artifact_root: &ArtifactRoot,
    command_line: &str,
    run_artifacts: &RunArtifacts,
) -> Result<(), String> {
    artifact_root.write(&RootRelativePath::new(COMMAND_ARTIFACT_NAME)?, format!("{command_line}\n").as_bytes())?;
    artifact_root.write(&RootRelativePath::new(STATUS_ARTIFACT_NAME)?, run_artifacts.status_json.as_bytes())?;
    artifact_root.write(&RootRelativePath::new(SUMMARY_ARTIFACT_NAME)?, run_artifacts.summary.as_bytes())?;
    artifact_root
        .write(&RootRelativePath::new(PROVENANCE_ARTIFACT_NAME)?, run_artifacts.provenance_jsonl.as_bytes())?;
    Ok(())
}

fn rewrite_status_sidecars_best_effort(artifact_root: &ArtifactRoot, command_line: &str, run_artifacts: &RunArtifacts) {
    let _ = write_status_sidecars(artifact_root, command_line, run_artifacts);
}

fn invalidate_failed_bundle_best_effort(artifact_root: &ArtifactRoot, run_artifacts: &RunArtifacts) {
    let Some(structured_output) = run_artifacts.structured_output.as_ref() else {
        return;
    };
    let Ok(relative) = RootRelativePath::new(structured_output.file_name) else {
        return;
    };

    let _ = artifact_root.remove_file_or_dir_all(&relative);
}

fn expected_artifact_files(run_artifacts: &RunArtifacts) -> Vec<&'static str> {
    let mut files = vec![
        COMMAND_ARTIFACT_NAME,
        PROVENANCE_ARTIFACT_NAME,
        STATUS_ARTIFACT_NAME,
        SUMMARY_ARTIFACT_NAME,
    ];
    if let Some(structured_artifact) = run_artifacts.structured_output.as_ref() {
        files.push(structured_artifact.file_name);
    }
    files.extend(run_artifacts.additional_artifacts.iter().map(|artifact| artifact.file_name));
    files
}

fn remove_extra_artifact_paths(artifact_root: &ArtifactRoot, expected_files: &[&'static str]) -> Result<(), String> {
    for file_name in artifact_root.read_entry_names()? {
        if expected_files.iter().any(|expected| *expected == file_name) {
            continue;
        }

        let relative = RootRelativePath::new(file_name.as_str())?;
        artifact_root.remove_file_or_dir_all(&relative)?;
    }

    Ok(())
}

fn classify_cargo_process_exit(status: std::process::ExitStatus) -> output::CargoProcessExit {
    match status.code() {
        Some(code) => output::CargoProcessExit::exit_code(code),
        None => output::CargoProcessExit::terminated(),
    }
}

fn update_lint_cache(workspace_root: &Path, findings: &mut [Finding]) -> Result<(), String> {
    let cache_root = CacheRoot::open_or_create(&workspace_root.join(LINT_CACHE_DIR))?;
    let cache_locator = RootRelativePath::new(LINT_CACHE_FILE)?;
    let mut cached: std::collections::BTreeSet<String> = match cache_root.read_to_string_optional(&cache_locator)? {
        Some(contents) => {
            serde_json::from_str(&contents).map_err(|error| format!("parse cache path `{LINT_CACHE_FILE}`: {error}"))?
        }
        None => std::collections::BTreeSet::new(),
    };

    let mut changed = false;
    for finding in findings {
        if finding.identity_status != "computed" {
            finding.cache_status.clear();
            finding.cache_status.push_str("bypassed");
            continue;
        }
        if cached.contains(&finding.semantic_id) {
            finding.cache_status.clear();
            finding.cache_status.push_str("hit");
        } else {
            cached.insert(finding.semantic_id.clone());
            finding.cache_status.clear();
            finding.cache_status.push_str("miss");
            changed = true;
        }
    }

    if changed {
        let mut rendered =
            serde_json::to_string_pretty(&cached).map_err(|error| format!("serialize lint cache: {error}"))?;
        rendered.push('\n');
        cache_root.write(&cache_locator, rendered.as_bytes())?;
    }

    Ok(())
}

fn canonicalize_findings(findings: &mut [Finding]) {
    findings.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.crate_name.cmp(&right.crate_name))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.lint.cmp(&right.lint))
            .then(left.stable_id.cmp(&right.stable_id))
    });

    for (offset, finding) in findings.iter_mut().enumerate() {
        finding.index = format!("F{}", offset + 1);
    }
}

fn normalize_lint_name(code: &str) -> String {
    let normalized = code.strip_prefix("tigerstyle::").unwrap_or(code);
    if normalized.is_empty() {
        "compiler_error".to_owned()
    } else {
        normalized.to_owned()
    }
}

fn json_u64_to_u32(value: Option<u64>) -> u32 {
    value.and_then(|line| u32::try_from(line).ok()).filter(|line| *line > 0).unwrap_or(1)
}

// r[impl octet.runner_self_check.inventory]
// r[verify octet.runner_self_check.inventory.paths]
struct DiagnosticPaths {
    root: PathBuf,
    reported_root: PathBuf,
    package_dirs: BTreeMap<String, PathBuf>,
    dir: Dir,
}

impl DiagnosticPaths {
    fn from_cargo_metadata(
        root: &Path,
        toolchain: &str,
        scope: &[String],
        check_args: &[String],
    ) -> Result<Self, String> {
        let reported_root = root.to_path_buf();
        let root =
            fs::canonicalize(root).map_err(|error| format!("canonicalize diagnostic workspace root: {error}"))?;
        let mut command = Command::new(CARGO_COMMAND);
        command
            .args(["metadata", "--format-version", "1"])
            .current_dir(&root)
            .env("RUSTUP_TOOLCHAIN", toolchain);
        // Preserve options affecting the effective manifest and Cargo configuration.
        for args in [scope, check_args] {
            let mut args = args.iter();
            while let Some(arg) = args.next() {
                if ["--manifest-path", "--config", "--features", "--filter-platform"].contains(&arg.as_str()) {
                    let value = args.next().ok_or_else(|| format!("Cargo option `{arg}` is missing its value"))?;
                    command.arg(arg).arg(value);
                } else if [
                    "--locked",
                    "--offline",
                    "--frozen",
                    "--all-features",
                    "--no-default-features",
                ]
                .contains(&arg.as_str())
                    || arg.starts_with("--manifest-path=")
                    || arg.starts_with("--config=")
                    || arg.starts_with("--features=")
                    || arg.starts_with("--filter-platform=")
                {
                    command.arg(arg);
                }
            }
        }
        let output = command.output().map_err(|error| format!("run diagnostic Cargo metadata: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "diagnostic Cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("parse diagnostic Cargo metadata: {error}"))?;
        let metadata_root =
            metadata["workspace_root"].as_str().ok_or("diagnostic Cargo metadata has no workspace root")?;
        if fs::canonicalize(metadata_root).map_err(|error| format!("canonicalize Cargo metadata root: {error}"))?
            != root
        {
            return Err("diagnostic Cargo metadata workspace root differs from the check workspace".to_owned());
        }
        let mut package_dirs = BTreeMap::new();
        let packages = metadata["packages"].as_array().ok_or("diagnostic Cargo metadata has no packages")?;
        for package in packages {
            // Registry/git packages may have paths under an ambient Cargo cache;
            // only local packages can anchor a compiler-origin workspace path.
            match package.get("source") {
                Some(serde_json::Value::Null) => {}
                Some(_) => continue,
                None => return Err("diagnostic Cargo metadata package lacks source provenance".to_owned()),
            }
            let (Some(id), Some(manifest)) = (package["id"].as_str(), package["manifest_path"].as_str()) else {
                return Err("diagnostic Cargo metadata package lacks ID or manifest".to_owned());
            };
            let manifest = fs::canonicalize(manifest)
                .map_err(|error| format!("canonicalize diagnostic package manifest `{manifest}`: {error}"))?;
            let directory = manifest.parent().ok_or("diagnostic package manifest has no parent")?;
            if let Ok(relative) = directory.strip_prefix(&root) {
                let directory = if relative.as_os_str().is_empty() {
                    PathBuf::new()
                } else {
                    RootRelativePath::new(relative)
                        .map_err(|error| format!("invalid diagnostic package directory: {error}"))?
                        .as_path()
                        .to_path_buf()
                };
                if package_dirs.insert(id.to_owned(), directory).is_some() {
                    return Err(format!("duplicate diagnostic Cargo package ID `{id}`"));
                }
            }
        }
        let dir = Dir::open_ambient_dir(&root, ambient_authority())
            .map_err(|error| format!("open diagnostic workspace root: {error}"))?;
        Ok(Self {
            root,
            reported_root,
            package_dirs,
            dir,
        })
    }

    fn normalize(&self, package_id: Option<&str>, reported_path: &str) -> Result<String, String> {
        let path = Path::new(reported_path);
        let has_parent = path.components().any(|part| part == Component::ParentDir);
        if !path.is_absolute() && !has_parent {
            return RootRelativePath::new(path)
                .map(|path| path.display())
                .map_err(|error| format!("invalid Cargo diagnostic path: {error}"));
        }
        #[cfg(unix)]
        {
            self.normalize_physical(package_id, reported_path, path)
        }
        #[cfg(not(unix))]
        {
            let _ = package_id;
            Err("invalid Cargo diagnostic path: absolute and parent-bearing compiler paths are unsupported on this platform".to_owned())
        }
    }

    #[cfg(unix)]
    fn normalize_physical(&self, package_id: Option<&str>, reported_path: &str, path: &Path) -> Result<String, String> {
        const MAX_DIAGNOSTIC_PATH_BYTES: usize = 4096;
        const MAX_DIAGNOSTIC_COMPONENTS: usize = 256;
        if reported_path.len() > MAX_DIAGNOSTIC_PATH_BYTES {
            return Err("invalid Cargo diagnostic path: oversized path".to_owned());
        }
        let package = package_id
            .and_then(|id| self.package_dirs.get(id))
            .ok_or("invalid Cargo diagnostic path: unknown Cargo metadata package ID")?;
        let relative = if path.is_absolute() {
            path.strip_prefix(&self.reported_root)
                .or_else(|_| path.strip_prefix(&self.root))
                .map_err(|_| "invalid Cargo diagnostic path: absolute path is outside the workspace root".to_owned())?
        } else {
            path
        };
        if !path.is_absolute() && !relative.starts_with(package) {
            return Err("invalid Cargo diagnostic path: parent traversal lacks matching package directory".to_owned());
        }
        let mut physical = self.root.clone();
        let mut count = 0;
        // Path::components() elides interior `.` and trailing separators:
        // `file/./..` must still fail ENOTDIR rather than becoming `file/..`.
        let text = relative.to_str().ok_or("invalid Cargo diagnostic path: non-UTF-8 component")?;
        let mut components = text.split('/').filter(|part| !part.is_empty()).peekable();
        while let Some(part) = components.next() {
            count += 1;
            if count > MAX_DIAGNOSTIC_COMPONENTS {
                return Err("invalid Cargo diagnostic path: too many path components".to_owned());
            }
            match part {
                "." => {}
                ".." => {
                    physical.pop();
                }
                name => physical.push(name),
            }
            physical = fs::canonicalize(&physical).map_err(|error| {
                format!("invalid Cargo diagnostic path: missing or inaccessible component: {error}")
            })?;
            if !physical.starts_with(&self.root) {
                return Err("invalid Cargo diagnostic path: component escapes the workspace root".to_owned());
            }
            if (components.peek().is_some() || part == "." || text.ends_with('/'))
                && !fs::metadata(&physical)
                    .map_err(|error| format!("inspect Cargo diagnostic component: {error}"))?
                    .is_dir()
            {
                return Err("invalid Cargo diagnostic path: intermediate component is not a directory".to_owned());
            }
        }
        let relative = physical.strip_prefix(&self.root).expect("checked path containment");
        let locator =
            RootRelativePath::new(relative).map_err(|error| format!("invalid Cargo diagnostic path: {error}"))?;
        // The root-relative capability open prevents a swapped directory from
        // redirecting the checked target outside the workspace before open.
        // O_NONBLOCK ensures a swapped-in FIFO cannot stall the check.
        let file = {
            let mut options = OpenOptions::new();
            options.read(true).custom_flags(libc::O_NONBLOCK);
            self.dir.open_with(locator.as_path(), &options)
        };
        let file = file.map_err(|error| format!("invalid Cargo diagnostic path: cannot open source file: {error}"))?;
        let opened = file.metadata().map_err(|error| format!("inspect Cargo diagnostic file: {error}"))?;
        if !opened.is_file() {
            return Err("invalid Cargo diagnostic path: source is not a regular file".to_owned());
        }
        let physical_again =
            fs::canonicalize(&physical).map_err(|error| format!("recheck Cargo diagnostic path: {error}"))?;
        if physical_again != physical || !physical_again.starts_with(&self.root) {
            return Err("invalid Cargo diagnostic path: source changed during verification".to_owned());
        }
        // This detects an inode replacement between capability-open and the
        // final path check on Unix. It is not a snapshot of the compiler's FD:
        // a concurrent swap-back remains possible, so containment is the
        // security guarantee, not immutable source identity.
        let current = self
            .dir
            .metadata(locator.as_path())
            .map_err(|error| format!("recheck Cargo diagnostic file: {error}"))?;
        if opened.dev() != current.dev() || opened.ino() != current.ino() {
            return Err("invalid Cargo diagnostic path: source changed during verification".to_owned());
        }
        Ok(locator.display())
    }
}

fn normalize_finding_message(workspace_root: &Path, message: &str) -> String {
    let native_root = workspace_root.display().to_string();
    if native_root == "/" {
        return message.to_owned();
    }

    let normalized_root = native_root.replace('\\', "/");
    message.replace(&native_root, ".").replace(&normalized_root, ".")
}

fn build_finding(
    index: usize,
    cargo_msg: &serde_json::Value,
    meta: &output::LintMeta,
    lint_name: &str,
    config_hash: &str,
    cache_enabled: bool,
    diagnostic_paths: &DiagnosticPaths,
) -> Result<Finding, String> {
    let message = &cargo_msg["message"];

    let primary_span = message["spans"]
        .as_array()
        .and_then(|spans| spans.iter().find(|s| s["is_primary"].as_bool() == Some(true)))
        .or_else(|| message["spans"].as_array().and_then(|s| s.first()));

    let reported_path = primary_span.and_then(|s| s["file_name"].as_str()).unwrap_or("unknown");
    let file = diagnostic_paths.normalize(cargo_msg["package_id"].as_str(), reported_path)?;
    let line = json_u64_to_u32(primary_span.and_then(|s| s["line_start"].as_u64()));
    let column = json_u64_to_u32(primary_span.and_then(|s| s["column_start"].as_u64()));

    let msg_text =
        normalize_finding_message(&diagnostic_paths.reported_root, message["message"].as_str().unwrap_or(""));
    let severity = message["level"].as_str().unwrap_or("warning").to_owned();
    let crate_name = cargo_msg["target"]["name"].as_str().unwrap_or("unknown").to_owned();

    let (suggestion, applicability) = extract_suggestion(message);
    let normalized_snippet_hash = primary_span
        .map(snippet_from_span)
        .filter(|snippet| !snippet.trim().is_empty())
        .map_or_else(|| "unsupported".to_owned(), |snippet| determinism::normalized_snippet_hash(&snippet));
    let identity_status = if normalized_snippet_hash == "unsupported" {
        "unsupported"
    } else {
        "computed"
    };
    let semantic_id = determinism::semantic_finding_id(
        lint_name,
        &crate_name,
        &msg_text,
        suggestion.as_deref(),
        &normalized_snippet_hash,
        Some(config_hash),
    );

    Ok(Finding {
        version: output::finding_version(),
        index: format!("F{index}"),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: identity_status.to_owned(),
        normalized_snippet_hash,
        config_hash: config_hash.to_owned(),
        cache_status: if cache_enabled { "pending" } else { "disabled" }.to_owned(),
        lint: lint_name.to_owned(),
        group: meta.group.to_owned(),
        severity,
        file,
        line,
        column,
        message: msg_text,
        suggestion,
        applicability,
        principle: meta.principle.to_owned(),
        effect_family: meta.effect_family.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name,
    })
}

fn snippet_from_span(span: &serde_json::Value) -> String {
    span["text"]
        .as_array()
        .map(|lines| lines.iter().filter_map(|line| line["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

fn extract_suggestion(message: &serde_json::Value) -> (Option<String>, Option<String>) {
    let Some(children) = message["children"].as_array() else {
        return (None, None);
    };

    for child in children {
        if let Some(spans) = child["spans"].as_array() {
            for span in spans {
                if let Some(replacement) = span["suggested_replacement"].as_str() {
                    let applicability = span["suggestion_applicability"].as_str().map(ToOwned::to_owned);
                    return (Some(replacement.to_owned()), applicability);
                }
            }
        }
    }

    (None, None)
}

fn format_compiling_banner(json: &serde_json::Value) -> Option<String> {
    let fresh = json["fresh"].as_bool().unwrap_or(true);
    if fresh {
        return None;
    }

    let target_name = json["target"]["name"].as_str()?;

    let version = json["package_id"].as_str().and_then(|pid| pid.rsplit_once('#')).map(|(_, fragment)| {
        if let Some((_, ver)) = fragment.rsplit_once('@') {
            ver
        } else {
            fragment
        }
    });

    if let Some(ver) = version {
        Some(format!("   Compiling {target_name} v{ver}"))
    } else {
        Some(format!("   Compiling {target_name}"))
    }
}

// --- Everything below is unchanged from the original engine ---

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Result<Self, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| format!("system clock error: {error}"))?
            .as_nanos();
        let path = env::temp_dir().join(format!("octet-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).map_err(|error| format!("create {}: {error}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct ManagedPath {
    path: String,
    _tempdir: Option<TempDir>,
}

impl ManagedPath {
    fn explicit(path: String) -> Self {
        Self { path, _tempdir: None }
    }

    fn generated(path: String, tempdir: TempDir) -> Self {
        Self {
            path,
            _tempdir: Some(tempdir),
        }
    }

    fn path(&self) -> &str {
        &self.path
    }
}

fn resolve_driver(toolchain: &str) -> Result<ManagedPath, String> {
    match env::var("TIGERSTYLE_DRIVER_PATH") {
        Ok(value) if !value.is_empty() => Ok(ManagedPath::explicit(value)),
        Ok(_) => Err("environment variable `TIGERSTYLE_DRIVER_PATH` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => match env::var("TIGERSTYLE_DYLINT_REV") {
            Ok(rev) if !rev.is_empty() => bootstrap_driver(&rev, toolchain),
            Ok(_) => Err("environment variable `TIGERSTYLE_DYLINT_REV` must not be empty".to_owned()),
            Err(env::VarError::NotPresent) => Err(
                "missing required environment variable `TIGERSTYLE_DRIVER_PATH` or bootstrap revision `TIGERSTYLE_DYLINT_REV`"
                    .to_owned(),
            ),
            Err(env::VarError::NotUnicode(_)) => {
                Err("non-utf8 `TIGERSTYLE_DYLINT_REV` is not supported".to_owned())
            }
        },
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_DRIVER_PATH` is not supported".to_owned()),
    }
}

fn bootstrap_driver(rev: &str, toolchain: &str) -> Result<ManagedPath, String> {
    let tempdir = TempDir::new("dylint-driver")?;
    let build_root = tempdir.path().join("bootstrap");
    let manifest_path = build_root.join("Cargo.toml");
    let source_path = build_root.join("src/main.rs");
    let target_dir = tempdir.path().join("target");

    write_bytes(
        &manifest_path,
        format!(
            "[package]\nname = \"tigerstyle-dylint-driver\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\ndylint_driver = {{ git = \"https://github.com/trailofbits/dylint\", rev = \"{rev}\" }}\n"
        )
        .as_bytes(),
    )?;
    write_bytes(
        &source_path,
        br#"#![feature(rustc_private)]

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    dylint_driver::dylint_driver(&args).unwrap_or_else(|error| {
        eprintln!("dylint-driver: {error:#}");
        std::process::exit(1);
    });
}
"#,
    )?;

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--release")
        .current_dir(&build_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("LIBGIT2_SYS_USE_PKG_CONFIG", "1")
        .status()
        .map_err(|error| format!("run cargo build for dylint driver bootstrap: {error}"))?;
    if !status.success() {
        return Err("dylint driver bootstrap failed".to_owned());
    }

    let driver_path = target_dir.join("release/tigerstyle-dylint-driver");
    if !driver_path.is_file() {
        return Err(format!("bootstrapped dylint driver missing at {}", driver_path.display()));
    }

    Ok(ManagedPath::generated(driver_path.to_string_lossy().into_owned(), tempdir))
}

fn resolve_lint_lib(toolchain: &str) -> Result<ManagedPath, String> {
    match env::var("TIGERSTYLE_LINT_LIB") {
        Ok(value) if !value.is_empty() => Ok(ManagedPath::explicit(value)),
        Ok(_) => Err("environment variable `TIGERSTYLE_LINT_LIB` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => build_lint_lib(toolchain),
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_LINT_LIB` is not supported".to_owned()),
    }
}

fn build_lint_lib(toolchain: &str) -> Result<ManagedPath, String> {
    let tempdir = TempDir::new("lint-lib")?;
    let target_dir = tempdir.path().join("target");
    let source_root = materialize_embedded_source()?;
    let manifest_path = source_root.join("Cargo.toml");
    let built_lib_name = format!("{}octet.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);
    let dylint_lib_name =
        format!("{}octet@{toolchain}.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .args(["--release", "--locked", "-p", "octet", "--lib"])
        .current_dir(&source_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .status()
        .map_err(|error| format!("run cargo build for Octet lint library: {error}"))?;
    if !status.success() {
        return Err("Octet lint library build failed".to_owned());
    }

    let built_lib_path = target_dir.join("release").join(&built_lib_name);
    if !built_lib_path.is_file() {
        return Err(format!("built Octet lint library missing at {}", built_lib_path.display()));
    }

    let dylint_lib_path = tempdir.path().join(&dylint_lib_name);
    copy_file(&built_lib_path, &dylint_lib_path)?;
    Ok(ManagedPath::generated(dylint_lib_path.to_string_lossy().into_owned(), tempdir))
}

fn resolve_toolchain() -> Result<String, String> {
    match env::var("TIGERSTYLE_TOOLCHAIN") {
        Ok(value) if !value.is_empty() => Ok(value),
        Ok(_) => Err("environment variable `TIGERSTYLE_TOOLCHAIN` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => derive_embedded_toolchain(),
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_TOOLCHAIN` is not supported".to_owned()),
    }
}

fn derive_embedded_toolchain() -> Result<String, String> {
    let source_root = materialize_embedded_source()?;
    let rust_toolchain_path = source_root.join("rust-toolchain.toml");
    let rust_toolchain = fs::read_to_string(&rust_toolchain_path)
        .map_err(|error| format!("read {}: {error}", rust_toolchain_path.display()))?;
    let document = rust_toolchain
        .parse::<toml::Table>()
        .map_err(|error| format!("parse {}: {error}", rust_toolchain_path.display()))?;
    let channel = document
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("channel"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("missing `toolchain.channel` in {}", rust_toolchain_path.display()))?;
    let host = rustc_host_triple()?;

    if channel.ends_with(&host) {
        Ok(channel.to_owned())
    } else {
        Ok(format!("{channel}-{host}"))
    }
}

fn materialize_embedded_source() -> Result<PathBuf, String> {
    let sources_root = sources_cache_root()?;
    let final_dir = sources_root.join(embedded_source::EMBEDDED_SOURCE_HASH);
    if cache_marker_matches(&final_dir)? {
        return Ok(final_dir);
    }

    let staging_dir = sources_root.join(format!(
        ".{}-{}-{}",
        embedded_source::EMBEDDED_SOURCE_HASH,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| format!("system clock error: {error}"))?
            .as_nanos()
    ));
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(|error| format!("remove {}: {error}", staging_dir.display()))?;
    }

    for (relative_path, contents) in embedded_source::EMBEDDED_SOURCE_FILES {
        write_bytes(&staging_dir.join(relative_path), contents)?;
    }
    write_bytes(&staging_dir.join(SOURCE_HASH_MARKER), embedded_source::EMBEDDED_SOURCE_HASH.as_bytes())?;

    if final_dir.exists() && !cache_marker_matches(&final_dir)? {
        fs::remove_dir_all(&final_dir).map_err(|error| format!("remove {}: {error}", final_dir.display()))?;
    }

    match fs::rename(&staging_dir, &final_dir) {
        Ok(()) => Ok(final_dir),
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            if final_dir.exists() && cache_marker_matches(&final_dir)? {
                Ok(final_dir)
            } else {
                Err(format!("rename {} -> {}: {error}", staging_dir.display(), final_dir.display()))
            }
        }
    }
}

fn sources_cache_root() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        candidates.push(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("HOME") {
        candidates.push(PathBuf::from(path).join(".cache"));
    }
    candidates.push(env::temp_dir().join("octet-cache"));

    let mut failures = Vec::new();
    for candidate in candidates {
        let sources_root = candidate.join("octet").join("sources");
        match fs::create_dir_all(&sources_root) {
            Ok(()) => return Ok(sources_root),
            Err(error) => failures.push(format!("create {}: {error}", sources_root.display())),
        }
    }

    Err(failures.join("; "))
}

fn cache_marker_matches(path: &Path) -> Result<bool, String> {
    let marker_path = path.join(SOURCE_HASH_MARKER);
    let marker = match fs::read_to_string(&marker_path) {
        Ok(marker) => marker,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("read {}: {error}", marker_path.display())),
    };
    Ok(marker.trim_end() == embedded_source::EMBEDDED_SOURCE_HASH)
}

fn rustc_host_triple() -> Result<String, String> {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc).arg("-vV").output().map_err(|error| format!("run rustc -vV: {error}"))?;
    if !output.status.success() {
        return Err(format!("rustc -vV failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| "rustc -vV stdout was not utf8".to_owned())?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(ToOwned::to_owned)
        .ok_or_else(|| "rustc -vV output missing `host:` line".to_owned())
}

enum LockfileSnapshot {
    Missing,
    Present(Vec<u8>),
}

fn snapshot_lockfile(path: &Path) -> Result<LockfileSnapshot, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(LockfileSnapshot::Present(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LockfileSnapshot::Missing),
        Err(error) => Err(format!("read {}: {error}", path.display())),
    }
}

fn lockfile_changed(path: &Path, before: &LockfileSnapshot) -> Result<bool, String> {
    match before {
        LockfileSnapshot::Missing => Ok(path.exists()),
        LockfileSnapshot::Present(original) => match fs::read(path) {
            Ok(current) => Ok(&current != original),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(error) => Err(format!("read {}: {error}", path.display())),
        },
    }
}

fn restore_lockfile(path: &Path, before: &LockfileSnapshot) -> Result<(), String> {
    match before {
        LockfileSnapshot::Missing => fs::remove_file(path)
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(error)
                }
            })
            .map_err(|error| format!("remove {}: {error}", path.display())),
        LockfileSnapshot::Present(original) => write_bytes(path, original),
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|error| format!("copy {} -> {}: {error}", from.display(), to.display()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    #[cfg(unix)]
    use std::fs;
    use std::path::Path;
    #[cfg(unix)]
    use std::path::PathBuf;

    #[cfg(unix)]
    use serde_json::json;

    #[cfg(unix)]
    use super::DiagnosticPaths;
    #[cfg(unix)]
    use super::build_finding;
    use super::embedded_source;
    use super::json_u64_to_u32;
    use super::normalize_finding_message;
    use super::normalize_lint_name;

    const REQUIRED_EMBEDDED_PATHS: &[&str] = &[
        "Cargo.toml",
        "crates/octet-lint-metadata/Cargo.toml",
        "crates/octet-lint-metadata/src/identity.rs",
        "crates/octet-lint-metadata/src/identity/admission.rs",
        "crates/octet-lint-metadata/src/identity/evidence.rs",
        "crates/octet-lint-metadata/src/identity/model.rs",
        "crates/octet-lint-metadata/src/identity/tests.rs",
        "crates/octet-lint-metadata/src/strict_profile.rs",
        "standards/clippy-restrictions/generated/profiles.json",
        "standards/effect-closure/generated/effect-closure-catalog.json",
        "standards/nominal-domains/generated/catalog.json",
        "standards/strict-profiles/generated/strict-rust-profiles.json",
    ];

    #[test]
    fn spanless_compiler_errors_keep_valid_structured_domains() {
        assert_eq!(normalize_lint_name(""), "compiler_error");
        assert_eq!(json_u64_to_u32(None), 1);
        assert_eq!(json_u64_to_u32(Some(0)), 1);
        assert_eq!(json_u64_to_u32(Some(42)), 42);
    }

    #[cfg(unix)]
    #[test]
    fn compiler_spans_require_package_provenance_and_physical_source_containment() {
        let temporary = tempfile::tempdir().expect("workspace fixture");
        let root = temporary.path().join("workspace");
        let package = root.join("crates/noble-cli");
        let proof = root.join("proofs/mc1/NobleContracts");
        let module = package.join("src/core/declared");
        fs::create_dir_all(&proof).expect("proof directory");
        fs::create_dir_all(&module).expect("module directory");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/noble-cli\"]\nresolver = \"2\"\n")
            .expect("workspace manifest");
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"noble-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("package manifest");
        fs::write(proof.join("Rules.lean"), "theorem included : True := True.intro\n").expect("proof source");
        fs::write(module.join("mod.rs"), "compile_error!(\"preserve this error\");\n").expect("module source");
        fs::create_dir_all(package.join("src/workflow")).expect("workflow directory");
        fs::write(package.join("src/main.rs"), "fn main() {}\n").expect("package target");
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_owned());
        let paths = DiagnosticPaths::from_cargo_metadata(&root, &toolchain, &[], &[])
            .expect("real Cargo metadata resolves the package");
        let id = paths.package_dirs.keys().next().expect("package ID");
        assert_eq!(
            paths.normalize(None, "virtual/generated.rs").expect("unchanged clean relative locator"),
            "virtual/generated.rs"
        );
        assert!(paths.normalize(None, "../proofs/mc1/NobleContracts/Rules.lean").is_err());
        let finding = |name: &str, package_id: &str| {
            build_finding(
                1,
                &json!({
                    "package_id": package_id,
                    "target": {"name": "noble-cli"},
                    "message": {
                        "message": "preserve this error",
                        "level": "error",
                        "spans": [{"is_primary": true, "file_name": name, "line_start": 1, "column_start": 1}]
                    }
                }),
                &crate::output::UNKNOWN_LINT_META,
                "compiler_error",
                "fixture-hash",
                false,
                &paths,
            )
        };
        let proof_span = "crates/noble-cli/../../proofs/mc1/NobleContracts/Rules.lean";
        let accepted = finding(proof_span, id).expect("package-origin proof span");
        assert_eq!(accepted.file, "proofs/mc1/NobleContracts/Rules.lean");
        assert_eq!(accepted.severity, "error");
        assert_eq!(accepted.message, "preserve this error");
        assert_eq!(
            finding("crates/noble-cli/src/workflow/../core/declared/mod.rs", id)
                .expect("package-origin module span")
                .file,
            "crates/noble-cli/src/core/declared/mod.rs"
        );
        assert_eq!(
            finding(&proof.join("Rules.lean").display().to_string(), id).expect("absolute compiler span").file,
            accepted.file
        );
        for (name, package_id, expected) in [
            (proof_span, "path+file:///forged#0.1.0", "package ID"),
            (
                "crates/noble-cli-extra/../../proofs/mc1/NobleContracts/Rules.lean",
                id.as_str(),
                "package directory",
            ),
            ("../../proofs/mc1/NobleContracts/Rules.lean", id.as_str(), "package directory"),
            ("crates/noble-cli/../../proofs/mc1/NobleContracts/Absent.lean", id.as_str(), "missing"),
            ("crates/noble-cli/../../proofs", id.as_str(), "regular file"),
            ("crates/noble-cli/src/main.rs/../main.rs", id.as_str(), "not a directory"),
            ("crates/noble-cli/src/main.rs/./../main.rs", id.as_str(), "not a directory"),
            ("crates/noble-cli/../../../workspace/proofs/mc1/NobleContracts/Rules.lean", id.as_str(), "escapes"),
        ] {
            let error = finding(name, package_id).expect_err(name);
            assert!(error.contains(expected), "{name}: {error}");
        }
        let outside = temporary.path().join("outside.rs");
        fs::write(&outside, "outside\n").expect("outside source");
        assert!(finding(&outside.display().to_string(), id).unwrap_err().contains("outside"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&outside, package.join("src/escape.rs")).expect("symlink outside");
            assert!(finding("crates/noble-cli/src/../src/escape.rs", id).unwrap_err().contains("escapes"));
            symlink(temporary.path(), package.join("src/through-outside")).expect("symlink outside directory");
            assert!(
                finding("crates/noble-cli/src/through-outside/../workspace/proofs/mc1/NobleContracts/Rules.lean", id)
                    .unwrap_err()
                    .contains("escapes")
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn root_package_metadata_does_not_break_clean_relative_diagnostics() {
        let temporary = tempfile::tempdir().expect("root package fixture");
        fs::write(
            temporary.path().join("Cargo.toml"),
            "[package]\nname = \"root-package\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("root package manifest");
        fs::create_dir(temporary.path().join("src")).expect("source directory");
        fs::write(temporary.path().join("src/lib.rs"), "pub fn value() {}\n").expect("source file");
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_owned());
        let paths = DiagnosticPaths::from_cargo_metadata(temporary.path(), &toolchain, &[], &[])
            .expect("root package Cargo metadata");
        let id = paths.package_dirs.keys().next().expect("root package ID");
        assert_eq!(paths.normalize(None, "generated/virtual.rs").expect("clean relative path"), "generated/virtual.rs");
        assert_eq!(paths.normalize(Some(id), "src/../Cargo.toml").expect("root-package parent path"), "Cargo.toml");
        assert!(paths.normalize(Some(id), "../Cargo.toml").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn actual_cargo_compiler_error_from_parent_module_is_kept_as_a_finding() {
        let temporary = tempfile::tempdir().expect("compiler fixture");
        let root = temporary.path();
        let package = root.join("crates/noble-cli");
        let proof = root.join("proofs/mc1/NobleContracts");
        fs::create_dir_all(&package).expect("package directory");
        fs::create_dir_all(&proof).expect("proof directory");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/noble-cli\"]\nresolver = \"2\"\n")
            .expect("workspace manifest");
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"noble-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"main.rs\"\n",
        )
        .expect("package manifest");
        fs::write(
            package.join("main.rs"),
            "#[path = \"../../proofs/mc1/NobleContracts/Rules.rs\"] mod rules;\n\
             const RULE: &str = include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../proofs/mc1/NobleContracts/Rules.lean\"));\n",
        )
        .expect("source root");
        fs::write(proof.join("Rules.rs"), "compile_error!(\"retained proof compiler error\");\n")
            .expect("proof Rust module");
        fs::write(proof.join("Rules.lean"), "theorem included : True := True.intro\n").expect("included proof");
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_owned());
        let paths = DiagnosticPaths::from_cargo_metadata(root, &toolchain, &[], &[]).expect("fixture metadata");
        let output = std::process::Command::new("cargo")
            .args(["check", "--message-format=json", "--offline", "-p", "noble-cli"])
            .current_dir(root)
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .env("CARGO_TARGET_DIR", temporary.path().join("target"))
            .output()
            .expect("run fixture compiler");
        assert!(!output.status.success(), "fixture must produce its intended compiler error");
        let diagnostics = output
            .stdout
            .split(|byte| *byte == b'\n')
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .filter(|value| value["reason"] == "compiler-message")
            .collect::<Vec<_>>();
        let error = diagnostics
            .iter()
            .find(|value| {
                value["message"]["message"]
                    .as_str()
                    .is_some_and(|text| text.contains("retained proof compiler error"))
            })
            .expect("actual compiler error in proof module");
        let span = error["message"]["spans"]
            .as_array()
            .and_then(|spans| spans.iter().find(|span| span["is_primary"] == true))
            .and_then(|span| span["file_name"].as_str())
            .expect("compiler primary span");
        assert!(
            span.contains("crates/noble-cli/../../proofs/mc1/NobleContracts/Rules.rs"),
            "expected compiler-origin parent-bearing filename, got `{span}`"
        );
        let finding =
            build_finding(1, error, &crate::output::UNKNOWN_LINT_META, "compiler_error", "fixture-hash", false, &paths)
                .expect("actual Cargo compiler diagnostic");
        assert_eq!(finding.file, "proofs/mc1/NobleContracts/Rules.rs");
        assert_eq!(finding.severity, "error");
        assert!(finding.message.contains("retained proof compiler error"));
    }

    #[cfg(unix)]
    #[test]
    fn excluded_local_dependency_compiler_span_retains_its_error() {
        let temporary = tempfile::tempdir().expect("excluded dependency fixture");
        let root = temporary.path();
        fs::create_dir_all(root.join("app/src")).expect("app source directory");
        fs::create_dir_all(root.join("dep/src")).expect("dependency source directory");
        fs::create_dir_all(root.join("proofs")).expect("proof source directory");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\nexclude = [\"dep\"]\nresolver = \"2\"\n")
            .expect("workspace manifest");
        fs::write(
            root.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
             [dependencies]\ndep = { path = \"../dep\" }\n",
        )
        .expect("app manifest");
        fs::write(root.join("app/src/lib.rs"), "pub fn app() { dep::dependency(); }\n").expect("app source");
        fs::write(root.join("dep/Cargo.toml"), "[package]\nname = \"dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
            .expect("dependency manifest");
        fs::write(
            root.join("dep/src/lib.rs"),
            "#[path = \"../../proofs/Rules.rs\"] mod rules;\npub fn dependency() {}\n",
        )
        .expect("dependency source");
        fs::write(root.join("proofs/Rules.rs"), "compile_error!(\"retained excluded dependency error\");\n")
            .expect("proof module");
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_owned());
        let paths = DiagnosticPaths::from_cargo_metadata(root, &toolchain, &[], &["--offline".to_owned()])
            .expect("complete offline Cargo metadata resolves excluded path dependency");
        let output = std::process::Command::new("cargo")
            .args(["check", "--message-format=json", "--offline", "-p", "app"])
            .current_dir(root)
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .env("CARGO_TARGET_DIR", temporary.path().join("target"))
            .output()
            .expect("compile app and excluded dependency");
        assert!(!output.status.success(), "excluded dependency must emit its compiler error");
        let error = output
            .stdout
            .split(|byte| *byte == b'\n')
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .find(|message| {
                message["reason"] == "compiler-message"
                    && message["message"]["message"]
                        .as_str()
                        .is_some_and(|text| text.contains("retained excluded dependency error"))
            })
            .expect("compiler error originated in the excluded local dependency");
        let id = error["package_id"].as_str().expect("compiler-provided package ID");
        assert_eq!(paths.package_dirs.get(id).map(PathBuf::as_path), Some(Path::new("dep")));
        let span = error["message"]["spans"]
            .as_array()
            .and_then(|spans| spans.iter().find(|span| span["is_primary"] == true))
            .and_then(|span| span["file_name"].as_str())
            .expect("compiler primary span");
        assert!(span.contains("dep/src/../../proofs/Rules.rs"), "expected parent-bearing dependency span: {span}");
        let finding = build_finding(
            1,
            &error,
            &crate::output::UNKNOWN_LINT_META,
            "compiler_error",
            "fixture-hash",
            false,
            &paths,
        )
        .expect("retain excluded local dependency error");
        assert_eq!(finding.file, "proofs/Rules.rs");
        assert_eq!(finding.severity, "error");
        assert!(finding.message.contains("retained excluded dependency error"));
    }

    #[cfg(unix)]
    #[test]
    fn fifo_diagnostic_path_returns_without_blocking() {
        use std::time::Duration;
        use std::time::Instant;

        const ROOT_ENV: &str = "OCTET_FIFO_DIAGNOSTIC_TEST_ROOT";
        let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "stable".to_owned());
        if let Some(root) = std::env::var_os(ROOT_ENV) {
            let paths = DiagnosticPaths::from_cargo_metadata(Path::new(&root), &toolchain, &[], &[])
                .expect("root-package compiler provenance");
            let id = paths.package_dirs.keys().next().expect("package ID");
            assert!(
                paths.normalize(Some(id), "src/../blocked.fifo").unwrap_err().contains("regular file"),
                "FIFO must be rejected after nonblocking capability open"
            );
            return;
        }
        let temporary = tempfile::tempdir().expect("FIFO fixture");
        fs::write(
            temporary.path().join("Cargo.toml"),
            "[package]\nname = \"fifo-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("package manifest");
        fs::create_dir(temporary.path().join("src")).expect("source directory");
        fs::write(temporary.path().join("src/lib.rs"), "pub fn value() {}\n").expect("source file");
        let fifo = temporary.path().join("blocked.fifo");
        assert!(
            std::process::Command::new("mkfifo").arg(&fifo).status().expect("create FIFO").success(),
            "mkfifo must create the test input"
        );
        let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "engine::tests::fifo_diagnostic_path_returns_without_blocking",
                "--nocapture",
            ])
            .env(ROOT_ENV, temporary.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("run isolated FIFO check");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("wait for FIFO check") {
                assert!(status.success(), "isolated FIFO check failed: {status}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().expect("stop blocked FIFO check");
                child.wait().expect("reap blocked FIFO check");
                panic!("diagnostic normalization blocked on a FIFO");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn finding_messages_replace_only_the_explicit_workspace_root() {
        let workspace_root = Path::new("/workspace");
        let message = "module `/workspace/src/safety` differs from `/other/src/safety`";

        assert_eq!(
            normalize_finding_message(workspace_root, message),
            "module `./src/safety` differs from `/other/src/safety`"
        );
        assert_eq!(normalize_finding_message(Path::new("/"), message), message);
    }

    #[test]
    fn embedded_lint_source_contains_profile_and_effect_closure_runtime_inputs() {
        let paths = embedded_source::EMBEDDED_SOURCE_FILES
            .iter()
            .map(|(path, _contents)| *path)
            .collect::<BTreeSet<_>>();

        for required in REQUIRED_EMBEDDED_PATHS {
            assert!(paths.contains(required), "embedded lint source is missing `{required}`");
        }
    }

    #[test]
    fn embedded_lint_source_paths_are_unique_and_workspace_relative() {
        let paths = embedded_source::EMBEDDED_SOURCE_FILES.iter().map(|(path, _contents)| *path).collect::<Vec<_>>();
        let unique = paths.iter().copied().collect::<BTreeSet<_>>();

        assert_eq!(paths.len(), unique.len(), "embedded source paths must be unique");
        assert!(paths.iter().all(|path| !path.starts_with('/') && !path.contains("../")));
    }
}
