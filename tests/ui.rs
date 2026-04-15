use std::collections::HashMap;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const FIXTURES: &[&str] = &[
    "ambient_clock",
    "nested_conditionals",
    "compound_assertion",
    "unbounded_loop",
    "no_panic",
    "no_panic_cfg_not",
    "no_panic_exhaustive",
    "no_panic_exhaustive_external",
    "no_panic_exhaustive_non_exhaustive",
    "no_panic_exhaustive_indirect",
    "no_panic_todo_allow",
    "no_unwrap",
    "no_unwrap_cfg_not",
    "no_unwrap_compile_time_known",
    "no_unwrap_guarded",
    "no_recursion",
    "ignored_result",
    "ignored_result_drop",
    "ignored_result_shutdown_send",
    "tokio_async/ignored_result_shutdown_send_tokio",
    "tokio_async/ignored_result_try_send_tokio",
    "unchecked_narrowing",
    "unchecked_narrowing_min",
    "unchecked_narrowing_try_from",
    "unchecked_narrowing_same_block_assert",
    "unchecked_division",
    "catch_all_on_enum",
    "assertion_density",
    "raw_arithmetic_overflow",
    "function_length",
    "compound_condition",
    "acronym_style",
    "numeric_units",
    "bool_naming",
    "float_for_currency",
    "platform_dependent_cast",
    "ambiguous_params",
    "unjustified_allow",
    "verified/verified_purity",
    "contradictory_time",
    "tokio_async/unbounded_loop_tokio_select",
    "tokio_async/unbounded_loop_imported_select",
    "tokio_async/unbounded_loop_non_tokio_select",
    "tokio_async/unbounded_loop_bare_async",
    "tokio_async/unbounded_loop_plain_await",
    "tokio_async/unbounded_loop_bounded_receiver",
    "tokio_async/unbounded_loop_unbounded_receiver",
    "multi_lock_ordering",
    "function_length_custom/function_length_custom",
    "assertion_density_custom/assertion_density_custom",
    "nesting_custom/nesting_custom",
    "acronym_allowlist_custom/acronym_allowlist_custom",
    "no_unwrap_disabled_custom/no_unwrap_disabled_custom",
    "no_unwrap_allow_custom/no_unwrap_allow_custom",
    "no_unwrap_warn_custom/no_unwrap_warn_custom",
    "no_unwrap_deny_custom/no_unwrap_deny_custom",
    "no_unwrap_default_custom/no_unwrap_default_custom",
    "no_unwrap_disabled_deny_custom/no_unwrap_disabled_deny_custom",
    "no_unwrap_disabled_source_attr_custom/no_unwrap_disabled_source_attr_custom",
];

#[cfg(unix)]
fn create_library_alias(plain: &Path, dylint: &Path) {
    symlink(plain, dylint).expect("create dylint library alias");
}

#[cfg(not(unix))]
fn create_library_alias(plain: &Path, dylint: &Path) {
    fs::copy(plain, dylint).expect("copy dylint library alias");
}

fn temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before unix epoch").as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()))
}

fn command_stdout(command: &mut Command, context: &str) -> String {
    let output = command.output().expect(context);
    assert!(
        output.status.success(),
        "{context} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

fn rustup_toolchain() -> String {
    if let Some(toolchain) = env::var_os("RUSTUP_TOOLCHAIN") {
        return toolchain.into_string().expect("utf8 RUSTUP_TOOLCHAIN for UI tests");
    }

    let stdout = command_stdout(Command::new("rustup").args(["show", "active-toolchain"]), "detect rustup toolchain");
    stdout.split_ascii_whitespace().next().expect("active rustup toolchain").to_owned()
}

fn host_triple() -> String {
    let stdout = command_stdout(Command::new("rustc").args(["-vV"]), "detect rustc host triple");
    stdout.lines().find_map(|line| line.strip_prefix("host: ")).expect("rustc host line").to_owned()
}

fn prebuilt_path(name: &str) -> Option<PathBuf> {
    env::var_os(name).map(PathBuf::from)
}

#[derive(Clone)]
struct FixtureSupport {
    dependency_search_path: PathBuf,
    externs: Vec<(String, PathBuf)>,
}

fn fixture_support_manifest(fixture_path: &Path) -> Option<PathBuf> {
    let manifest_path = fixture_path.parent()?.join("Cargo.toml");
    manifest_path.exists().then_some(manifest_path)
}

fn fixture_dependency_names(manifest_path: &Path) -> Vec<String> {
    let manifest = fs::read_to_string(manifest_path)
        .unwrap_or_else(|error| panic!("read fixture manifest {}: {error}", manifest_path.display()));
    let parsed = toml::from_str::<toml::Table>(&manifest)
        .unwrap_or_else(|error| panic!("parse fixture manifest {}: {error}", manifest_path.display()));

    parsed
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|deps| deps.keys().map(|name| name.replace('-', "_")).collect())
        .unwrap_or_default()
}

fn extern_artifact_path(deps_dir: &Path, crate_name: &str) -> PathBuf {
    let prefix = format!("{}{}-", std::env::consts::DLL_PREFIX, crate_name);
    let mut candidates = fs::read_dir(deps_dir)
        .unwrap_or_else(|error| panic!("read dependency dir {}: {error}", deps_dir.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if !file_name.starts_with(&prefix) {
                return None;
            }

            let rank = if file_name.ends_with(".rmeta") {
                0
            } else if file_name.ends_with(".rlib") {
                1
            } else if file_name.ends_with(std::env::consts::DLL_SUFFIX) {
                2
            } else {
                return None;
            };

            Some((rank, path))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|(left_rank, left_path), (right_rank, right_path)| {
        left_rank.cmp(right_rank).then_with(|| left_path.file_name().cmp(&right_path.file_name()))
    });

    candidates
        .into_iter()
        .next()
        .map(|(_, path)| path)
        .unwrap_or_else(|| panic!("missing extern artifact for crate `{crate_name}` under {}", deps_dir.display()))
}

fn build_fixture_support(toolchain: &str, manifest_path: &Path, dependency_names: &[String]) -> FixtureSupport {
    let target_dir = temp_path("tigerstyle-ui-fixture-deps");
    let status = Command::new("cargo")
        .current_dir(manifest_path.parent().expect("fixture manifest parent"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["check", "--quiet", "--manifest-path"])
        .arg(manifest_path)
        .status()
        .expect("build fixture support dependencies");
    assert!(status.success(), "failed to build fixture support deps for {}", manifest_path.display());

    let dependency_search_path = target_dir.join("debug/deps");
    let externs = dependency_names
        .iter()
        .map(|name| (name.clone(), extern_artifact_path(&dependency_search_path, name)))
        .collect();

    FixtureSupport {
        dependency_search_path,
        externs,
    }
}

fn fixture_support_for<'a>(
    toolchain: &str,
    fixture_path: &Path,
    support_cache: &'a mut HashMap<PathBuf, FixtureSupport>,
) -> Option<&'a FixtureSupport> {
    let manifest_path = fixture_support_manifest(fixture_path)?;
    let dependency_names = fixture_dependency_names(&manifest_path);
    if dependency_names.is_empty() {
        return None;
    }

    if !support_cache.contains_key(&manifest_path) {
        let support = build_fixture_support(toolchain, &manifest_path, &dependency_names);
        support_cache.insert(manifest_path.clone(), support);
    }

    support_cache.get(&manifest_path)
}

fn build_library(toolchain: &str) -> PathBuf {
    if let Some(path) = prebuilt_path("TIGERSTYLE_UI_LIBRARY") {
        assert!(path.exists(), "missing prebuilt UI library: {}", path.display());
        return path;
    }

    let status = Command::new("cargo")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--lib"])
        .status()
        .expect("build UI test library");
    assert!(status.success(), "cargo build --lib failed for UI tests");

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"));
    let debug_dir = target_dir.join("debug");
    let lib_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let plain = debug_dir.join(format!("{}{}{}", std::env::consts::DLL_PREFIX, lib_name, std::env::consts::DLL_SUFFIX));
    let dylint = debug_dir.join(format!(
        "{}{}@{}{}",
        std::env::consts::DLL_PREFIX,
        lib_name,
        toolchain,
        std::env::consts::DLL_SUFFIX
    ));

    if dylint.exists() {
        fs::remove_file(&dylint).expect("remove stale dylint library alias");
    }
    create_library_alias(&plain, &dylint);
    dylint
}

fn build_driver(toolchain: &str) -> PathBuf {
    if let Some(path) = prebuilt_path("TIGERSTYLE_UI_DRIVER") {
        assert!(path.exists(), "missing prebuilt UI driver: {}", path.display());
        return path;
    }

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/ui-driver/Cargo.toml");
    let driver_target_dir = temp_path("tigerstyle-ui-driver-target");
    let status = Command::new("cargo")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", &driver_target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--quiet", "--locked", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("build UI driver");
    assert!(status.success(), "failed to build UI driver");

    let debug_dir = driver_target_dir.join("debug");
    let candidates = [
        debug_dir.join(format!("tigerstyle-ui-driver{}", std::env::consts::EXE_SUFFIX)),
        debug_dir.join(format!("tigerstyle_ui_driver{}", std::env::consts::EXE_SUFFIX)),
    ];
    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("UI driver binary missing under {}", debug_dir.display()))
}

fn normalize_stderr(stderr: &str, fixture_path: &Path) -> String {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let relative = fixture_path.strip_prefix(&ui_dir).expect("fixture under ui_tests").to_string_lossy();
    stderr
        .replace("\r\n", "\n")
        .replace(fixture_path.to_string_lossy().as_ref(), &format!("$DIR/{relative}"))
}

fn expects_failure(name: &str) -> bool {
    matches!(name, "verified/verified_purity" | "no_unwrap_deny_custom/no_unwrap_deny_custom")
}

fn extra_rustc_args(name: &str) -> &'static [&'static str] {
    match name {
        "no_panic" | "no_unwrap" => &["--cfg", "test"],
        _ => &[],
    }
}

fn run_fixture(
    driver: &Path,
    library: &Path,
    host: &str,
    toolchain: &str,
    name: &str,
    support_cache: &mut HashMap<PathBuf, FixtureSupport>,
) {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let fixture_path = ui_dir.join(format!("{name}.rs"));
    let expected_name = fixture_path.file_stem().expect("fixture file stem").to_string_lossy();
    let expected_path = ui_dir.join(format!("{expected_name}.stderr"));
    let work_dir = temp_path(&format!("tigerstyle-ui-case-{name}"));
    let aux_dir = work_dir.join("aux");
    fs::create_dir_all(&aux_dir).expect("create UI aux dir");

    let library = if library.is_absolute() {
        library.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(library)
    };
    let dylint_libs = serde_json::to_string(&vec![library]).expect("encode DYLINT_LIBS");
    let output_path = work_dir.join(name);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create UI output dir");
    }

    let mut command = Command::new(driver);
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", dylint_libs)
        .args(extra_rustc_args(name));

    if let Some(support) = fixture_support_for(toolchain, &fixture_path, support_cache) {
        command.arg("-L").arg(format!("dependency={}", support.dependency_search_path.display()));
        for (crate_name, artifact_path) in &support.externs {
            command.arg("--extern").arg(format!("{crate_name}={}", artifact_path.display()));
        }
    }

    let output = command
        .arg(&fixture_path)
        .arg("-L")
        .arg(&work_dir)
        .arg(format!("--target={host}"))
        .args(["--error-format", "human"])
        .args(["-C", "prefer-dynamic"])
        .arg("-o")
        .arg(&output_path)
        .args(["-A", "unused"])
        .arg("--edition=2024")
        .args(["--emit=metadata", "-Zui-testing"])
        .arg("-L")
        .arg(&aux_dir)
        .output()
        .expect("run UI fixture");

    assert!(
        output.stdout.is_empty(),
        "fixture {name} produced unexpected stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expect_success = !expects_failure(name);
    assert_eq!(
        output.status.success(),
        expect_success,
        "fixture {name} exit-status mismatch: expected success={expect_success}, got status={}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = normalize_stderr(&String::from_utf8(output.stderr).expect("utf8 stderr"), &fixture_path);
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|error| panic!("read expected stderr {}: {error}", expected_path.display()));
    assert_eq!(expected.trim_end_matches('\n'), actual.trim_end_matches('\n'), "stderr mismatch for fixture {name}");
}

#[test]
#[ignore = "run via nix flake check or cargo test --test ui -- --ignored --nocapture"]
fn ui() {
    let toolchain = rustup_toolchain();
    let library = build_library(&toolchain);
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let mut support_cache = HashMap::new();

    for fixture in FIXTURES {
        eprintln!("[ui fixture] {fixture}");
        run_fixture(&driver, &library, &host, &toolchain, fixture, &mut support_cache);
    }
}
