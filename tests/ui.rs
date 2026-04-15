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
    "no_unwrap",
    "no_unwrap_cfg_not",
    "no_unwrap_compile_time_known",
    "no_unwrap_guarded",
    "no_recursion",
    "ignored_result",
    "ignored_result_drop",
    "unchecked_narrowing",
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

fn run_fixture(driver: &Path, library: &Path, host: &str, name: &str) {
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

    for fixture in FIXTURES {
        eprintln!("[ui fixture] {fixture}");
        run_fixture(&driver, &library, &host, fixture);
    }
}
