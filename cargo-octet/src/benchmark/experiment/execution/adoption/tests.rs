use crate::benchmark::experiment::model;
use crate::benchmark::experiment::profile;

fn measurement_inputs() -> (model::AnalysisProfile, model::ToolPin, model::AdoptionToolPlan) {
    let profile = profile::select(model::PROFILE_ID).expect("profile");
    let tool = profile.tools.iter().find(|tool| !tool.run_argv.is_empty()).expect("invokable tool").clone();
    let plan = profile.adoption.tools.iter().find(|plan| plan.tool_id == tool.tool_id).expect("plan").clone();
    (profile, tool, plan)
}

fn runtime(scratch_root: &std::path::Path) -> super::super::Runtime {
    super::super::Runtime {
        environment: super::super::PRESERVED_ENVIRONMENT
            .into_iter()
            .filter_map(|name| std::env::var_os(name).map(|value| (std::ffi::OsString::from(name), value)))
            .collect(),
        scratch_root: scratch_root.to_path_buf(),
        clock: cap_std::time::MonotonicClock::new(cap_std::ambient_authority()),
    }
}

#[test]
fn scratch_preparation_failure_retains_unmeasured_local_attempts_without_deleting_collision() {
    let (profile, tool, plan) = measurement_inputs();
    let directory = tempfile::tempdir().expect("scratch parent");
    let scratch = directory.path().join("collision");
    std::fs::write(&scratch, b"existing owner data").expect("create collision");
    let runtime = runtime(directory.path());
    let measurement = super::Measurement {
        profile: &profile,
        tool: &tool,
        availability: &model::Availability::Local,
        runtime: &runtime,
    };
    let (initial, incremental) =
        super::measure_at(&measurement, directory.path(), &profile.adoption.clean_consumer, &plan, &scratch);
    for observation in [initial, incremental] {
        assert!(!observation.measured);
        assert_eq!(observation.origin, model::MeasurementOrigin::Local);
        assert_eq!(observation.wall_duration_ns, None);
        assert!(
            observation
                .unavailable_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("create experiment scratch"))
        );
    }
    assert_eq!(std::fs::read(&scratch).expect("collision remains"), b"existing owner data");
}

#[cfg(unix)]
#[test]
fn cleanup_failure_keeps_raw_results_but_invalidates_both_timings() {
    let (profile, tool, mut plan) = measurement_inputs();
    let directory = tempfile::tempdir().expect("scratch parent");
    let scratch = directory.path().join("measurement");
    plan.initial_command = vec!["sh".to_owned(), "-c".to_owned(), "printf initial".to_owned()];
    plan.incremental_command = vec![
        "sh".to_owned(), "-c".to_owned(),
        r#"printf incremental; parent=${HOME%/*}; rmdir "$HOME" "$TMPDIR" "$CARGO_TARGET_DIR" && rmdir "$parent" && printf collision > "$parent""#.to_owned(),
    ];
    let runtime = runtime(directory.path());
    let measurement = super::Measurement {
        profile: &profile,
        tool: &tool,
        availability: &model::Availability::Local,
        runtime: &runtime,
    };
    let (initial, incremental) =
        super::measure_at(&measurement, directory.path(), &profile.adoption.clean_consumer, &plan, &scratch);
    assert_eq!(initial.stdout, "initial");
    assert_eq!(incremental.stdout, "incremental");
    for observation in [initial, incremental] {
        assert_eq!(observation.exit_code, Some(0));
        assert!(observation.wall_duration_ns.is_some());
        assert!(!observation.measured);
        assert!(
            observation
                .unavailable_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("clean experiment scratch"))
        );
    }
    assert_eq!(std::fs::read(&scratch).expect("replacement collision remains"), b"collision");
}
