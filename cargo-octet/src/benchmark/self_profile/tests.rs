use super::measureme::parse_mm_profdata;
use super::model::CaptureConfig;
use super::model::ConfigMode;
use super::model::UnitBuildStatus;
use super::phases::collect_phase_records;
use super::phases::write_engine_phase_record;
use super::replay::verify_report;
use super::runner::RunOptions;
use super::runner::execute;

fn shipped_profile() -> (CaptureConfig, String) {
    let raw = include_str!("../../../../standards/benchmark/self-profile.json");
    let blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(raw.as_bytes()));
    let cfg: CaptureConfig = serde_json::from_str(raw).expect("parse shipped self-profile.json");
    (cfg, blake3)
}

#[test]
fn cli_parses_benchmark_self_profile_flags_and_rejects_invalid() {
    let parsed = crate::cli::parse(
        [
            "cargo-octet",
            "benchmark",
            "self-profile",
            "--profile-config",
            "standards/benchmark/self-profile.json",
            "--workspace-root",
            "probe",
            "--artifact-dir",
            "out",
            "--controlled-change",
            "src/lib.rs",
            "--compare",
            "--output-format",
            "json",
        ]
        .map(std::ffi::OsString::from),
    )
    .expect("parse benchmark self-profile");
    match parsed {
        crate::cli::CommandArgs::BenchmarkSelfProfile(args) => {
            assert!(args.is_compare);
            assert!(!args.is_verify);
            assert_eq!(args.workspace_root, std::path::PathBuf::from("probe"));
            assert_eq!(args.controlled_change, Some(std::path::PathBuf::from("src/lib.rs")));
            assert_eq!(args.output_format, crate::cli::OutputFormat::Json);
        }
        other => panic!("unexpected parsed command: {other:?}"),
    }

    let sarif_err = crate::cli::parse(
        ["cargo-octet", "benchmark", "self-profile", "--output-format", "sarif"].map(std::ffi::OsString::from),
    );
    assert!(sarif_err.is_err());
}

#[test]
fn real_rustc_self_profile_measures_cold_warm_controlled_edit_and_verifies_replay() {
    let (profile, profile_blake3) = shipped_profile();
    let workspace = tempfile::TempDir::new().expect("temp workspace");
    let artifact_dir = tempfile::TempDir::new().expect("temp artifacts");
    std::fs::create_dir_all(workspace.path().join("src")).expect("src dir");
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname = \"self-profile-probe\"\nversion = \"0.1.0\"\nedition = \"2024\"\npublish = false\n",
    )
    .expect("Cargo.toml");
    std::fs::write(
        workspace.path().join("src/lib.rs"),
        r#"
pub fn branch_accumulator(xs: &[u32]) -> Result<u32, &'static str> {
    let mut total: u32 = 0;
    for &x in xs {
        total = total.checked_add(x).ok_or("overflow")?;
    }
    Ok(total)
}

pub fn loop_transform(n: u32) -> u32 {
    let mut acc = 1u32;
    let mut i = 0u32;
    while i < n {
        acc = acc.wrapping_mul(3).wrapping_add(i);
        i += 1;
    }
    acc
}
"#,
    )
    .expect("src/lib.rs");

    let report = execute(&RunOptions {
        profile: &profile,
        profile_config_blake3: &profile_blake3,
        workspace_root: workspace.path(),
        artifact_dir: artifact_dir.path(),
        is_compare_configurations: false,
        controlled_change_file: Some(std::path::Path::new("src/lib.rs")),
        driver_path: None,
        lint_library_path: None,
    })
    .expect("execute self-profile on real rustc");

    assert!(report.baseline_uninstrumented_wall_ns > 0);
    assert!(report.instrumented_cold_wall_ns > 0);
    assert_eq!(report.comparisons.len(), 4);

    let baseline = &report.comparisons[0];
    let cold = &report.comparisons[1];
    let warm = &report.comparisons[2];
    let rebuilt = &report.comparisons[3];

    assert!(!baseline.is_instrumented);
    assert_eq!(baseline.policy_exit_code, cold.policy_exit_code);
    assert!(cold.is_instrumented);
    assert_eq!(cold.cache_state, "cold");
    assert!(!cold.units.is_empty());
    let cold_unit = &cold.units[0];
    assert_eq!(cold_unit.build_status, UnitBuildStatus::Compiled);
    assert!(cold_unit.total_events > 0, "{cold_unit:?}");
    assert!(cold_unit.query_events > 0, "{cold_unit:?}");
    assert!(cold_unit.summed_query_span_ns > 0, "{cold_unit:?}");
    assert!(cold_unit.distinct_threads >= 1, "{cold_unit:?}");
    assert!(!cold_unit.top_queries.is_empty(), "{cold_unit:?}");

    assert_eq!(warm.cache_state, "warm-noop");
    assert_eq!(warm.units.len(), 1);
    assert_eq!(warm.units[0].build_status, UnitBuildStatus::NotCompiled);

    assert_eq!(rebuilt.cache_state, "controlled-rebuild");
    assert_eq!(rebuilt.units[0].build_status, UnitBuildStatus::Compiled);
    assert_ne!(rebuilt.units[0].source_blake3, cold_unit.source_blake3);

    let verification = verify_report(&report, &profile, Some(artifact_dir.path()));
    assert_eq!(verification.status, "verified", "{:?}", verification.diagnostics);
    assert!(verification.diagnostics.is_empty());
    assert!(verification.verified_unit_count >= 2);

    // Negative replay checks: tampered digest and tampered raw profdata fail closed.
    let mut tampered_report = report.clone();
    tampered_report.report_blake3 = "b3:0000".to_owned();
    let bad_digest = verify_report(&tampered_report, &profile, Some(artifact_dir.path()));
    assert_eq!(bad_digest.status, "invalid");

    let raw_file = std::path::PathBuf::from(&cold_unit.raw_profdata_path);
    let mut raw_bytes = std::fs::read(&raw_file).expect("read raw profdata");
    raw_bytes.push(0xAA);
    std::fs::write(&raw_file, raw_bytes).expect("tamper raw profdata");
    let bad_raw = verify_report(&report, &profile, Some(artifact_dir.path()));
    assert_eq!(bad_raw.status, "invalid");
    assert!(bad_raw.diagnostics.iter().any(|d| d.contains("digest mismatch")));
}

#[test]
fn measureme_parser_rejects_malformed_truncated_wrong_version_and_budget_overflow() {
    assert!(parse_mm_profdata(b"MMP", 1024, 100).is_err());
    assert!(parse_mm_profdata(b"NOPE\x09\x00\x00\x00", 1024, 100).is_err());

    let mut wrong_ver = Vec::new();
    wrong_ver.extend_from_slice(b"MMPD");
    wrong_ver.extend_from_slice(&8u32.to_le_bytes());
    wrong_ver.extend_from_slice(&[0u8; 32]);
    assert!(parse_mm_profdata(&wrong_ver, 1024, 100).is_err());

    let mut oversized = vec![0u8; 128];
    oversized[..4].copy_from_slice(b"MMPD");
    assert!(parse_mm_profdata(&oversized, 64, 100).is_err());
}

#[test]
fn concurrent_and_engine_phase_records_collect_deterministically() {
    let dir = tempfile::TempDir::new().expect("phase temp dir");
    write_engine_phase_record(dir.path(), 1_250, 3_500).expect("write engine phase");
    for (unit, pid) in [("unit-b", 202u32), ("unit-a", 101u32)] {
        let rec = super::model::CollectorPhaseRecord {
            schema_version: super::model::PHASE_SPAN_SCHEMA_VERSION.to_owned(),
            unit_label: unit.to_owned(),
            process_id: pid,
            selected_mir_analysis_ns: 900,
            value_flow_bodies_analyzed: 4,
            value_flow_steps: 28,
            value_flow_rounds: 2,
            semantic_bodies_analyzed: 4,
            shard_serialization_ns: 400,
            shard_bytes_written: 1024,
            shard_normalization_ns: 0,
            policy_evaluation_ns: 0,
            unattributed_note: "bounded phase".to_owned(),
        };
        let bytes = serde_json::to_vec_pretty(&rec).expect("serialize");
        std::fs::write(dir.path().join(format!("{unit}.{pid}.phases.json")), bytes).expect("write");
    }
    let collected = collect_phase_records(ConfigMode::SelectedValueFlow, 10_000, Some(dir.path()), 16, &[])
        .expect("collect concurrent phase spans");
    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0].unit_label, "cargo-octet-engine");
    assert_eq!(collected[0].shard_normalization_ns, 1_250);
    assert_eq!(collected[0].policy_evaluation_ns, 3_500);
    assert_eq!(collected[1].unit_label, "unit-a");
    assert_eq!(collected[2].unit_label, "unit-b");
    assert_eq!(collected[2].value_flow_steps, 28);

    assert!(collect_phase_records(ConfigMode::SelectedValueFlow, 10_000, Some(dir.path()), 2, &[]).is_err());
}
