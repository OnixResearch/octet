type DatabaseStepResult = Result<(super::DatabaseReceipt, Result<(), (&'static str, String)>), String>;

pub(super) fn attach(
    profile: &super::Profile,
    request: &super::process::ExecuteRequest<'_>,
    executor: &dyn super::Executor,
    path: &std::path::Path,
    execution: &mut super::Execution,
) -> Result<bool, String> {
    if profile.producer != super::PRODUCER_CODEQL {
        return Ok(true);
    }
    let database = profile.database.as_ref().ok_or_else(|| "CodeQL producer has no database".to_owned())?;
    let executable = execution
        .executable
        .as_ref()
        .ok_or_else(|| "CodeQL producer has no executable observation".to_owned())?;
    let binding = super::DatabaseBinding {
        source_snapshot_identity: execution.source_snapshot_identity.clone(),
        language: database.language.clone(),
        build_mode: database.build_mode.clone(),
        query_packs: profile.query_packs.iter().map(super::recording::query_pack_receipt).collect(),
        executable_identity: executable.identity.clone(),
    };
    match prepare_database(profile, path, &binding, request.workspace_root, executor) {
        Ok((receipt, result)) => {
            execution.database = Some(receipt);
            if let Err((outcome, message)) = result {
                execution.outcome = outcome.to_owned();
                execution.diagnostics = vec![message];
                return Ok(false);
            }
            Ok(true)
        }
        Err(error) => {
            execution.outcome = super::OUTCOME_TOOL_ERROR.to_owned();
            execution.diagnostics = vec![error];
            Ok(false)
        }
    }
}

fn prepare_database(
    profile: &super::Profile,
    path: &std::path::Path,
    binding: &super::DatabaseBinding,
    workspace_root: &std::path::Path,
    executor: &dyn super::Executor,
) -> DatabaseStepResult {
    if path.exists() {
        reuse_database(profile, path, binding, workspace_root)
    } else {
        create_database(profile, path, binding, workspace_root, executor)
    }
}

fn reuse_database(
    profile: &super::Profile,
    database_path: &std::path::Path,
    binding: &super::DatabaseBinding,
    workspace_root: &std::path::Path,
) -> DatabaseStepResult {
    let existing = read_database_binding(&database_path.join(super::DATABASE_BINDING_FILE_NAME))?;
    let Some((existing, existing_identity)) = existing else {
        return Ok((
            super::DatabaseReceipt {
                path: super::command::executable_path_string(database_path),
                created: false,
                binding: binding.clone(),
                binding_identity: String::new(),
                creation: None,
                analysis: None,
            },
            Err((
                super::OUTCOME_FAILED,
                "a CodeQL database exists without an Octet binding; it is stale or foreign".to_owned(),
            )),
        ));
    };
    if existing != *binding {
        return Ok((
            super::DatabaseReceipt {
                path: super::command::executable_path_string(database_path),
                created: false,
                binding: binding.clone(),
                binding_identity: existing_identity,
                creation: None,
                analysis: None,
            },
            Err((
                super::OUTCOME_FAILED,
                "the CodeQL database is stale: its source snapshot, build configuration, query packs, or executable differ"
                    .to_owned(),
            )),
        ));
    }
    Ok((
        super::DatabaseReceipt {
            path: super::command::executable_path_string(database_path),
            created: false,
            binding: binding.clone(),
            binding_identity: existing_identity,
            creation: Some(super::StepReceipt {
                step: "database_create".to_owned(),
                argv: super::command::codeql_create_argv(profile, database_path, workspace_root),
                exit_code: Some(super::EXIT_SUCCESS),
                status: super::STATUS_REUSED.to_owned(),
            }),
            analysis: None,
        },
        Ok(()),
    ))
}

fn create_database(
    profile: &super::Profile,
    database_path: &std::path::Path,
    binding: &super::DatabaseBinding,
    workspace_root: &std::path::Path,
    executor: &dyn super::Executor,
) -> DatabaseStepResult {
    let binding_path = database_path.join(super::DATABASE_BINDING_FILE_NAME);
    let argv = super::command::codeql_create_argv(profile, database_path, workspace_root);
    let output = executor.run(&argv, workspace_root, &profile.limits).map_err(|failure| failure.message)?;
    let creation = super::StepReceipt {
        step: "database_create".to_owned(),
        argv,
        exit_code: Some(output.exit_code),
        status: if output.exit_code == super::EXIT_SUCCESS {
            super::STATUS_SUCCEEDED.to_owned()
        } else {
            super::STATUS_FAILED.to_owned()
        },
    };
    let mut receipt = super::DatabaseReceipt {
        path: super::command::executable_path_string(database_path),
        created: true,
        binding: binding.clone(),
        binding_identity: String::new(),
        creation: Some(creation),
        analysis: None,
    };
    if output.exit_code != super::EXIT_SUCCESS {
        return Ok((
            receipt,
            Err((super::OUTCOME_TOOL_ERROR, format!("CodeQL database creation exited {}", output.exit_code))),
        ));
    }
    std::fs::create_dir_all(database_path)
        .map_err(|error| format!("create CodeQL database directory {}: {error}", database_path.display()))?;
    let bytes =
        serde_json::to_vec_pretty(binding).map_err(|error| format!("serialize CodeQL database binding: {error}"))?;
    std::fs::write(&binding_path, &bytes)
        .map_err(|error| format!("write CodeQL database binding {}: {error}", binding_path.display()))?;
    receipt.binding_identity = super::super::prefixed_identity(&bytes);
    Ok((receipt, Ok(())))
}

pub(super) fn read_database_binding(
    path: &std::path::Path,
) -> Result<Option<(super::DatabaseBinding, String)>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read CodeQL database binding {}: {error}", path.display())),
    };
    let identity = super::super::prefixed_identity(&bytes);
    let binding: super::DatabaseBinding = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse CodeQL database binding {}: {error}", path.display()))?;
    Ok(Some((binding, identity)))
}
