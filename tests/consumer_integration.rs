use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

struct StandaloneCargoTigerstyle {
    _tmp: TempDir,
    source_root: PathBuf,
    bin_path: PathBuf,
}

const COMPILED_CARGO_TIGERSTYLE_BIN: Option<&str> = option_env!("CARGO_TIGERSTYLE_BIN");
static BASH_PATH: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static BUILT_CARGO_TIGERSTYLE_BIN: OnceLock<Result<PathBuf, String>> = OnceLock::new();

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards").as_nanos();
        let path =
            env::temp_dir().join(format!("tigerstyle-consumer-integration-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
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

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, contents).expect("write file");
}

fn write_executable(path: &Path, contents: &str) {
    write_file(path, contents);
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod +x");
}

fn copy_path(from: &Path, to: &Path) {
    let metadata = fs::symlink_metadata(from).expect("symlink metadata");
    if metadata.is_dir() {
        fs::create_dir_all(to).expect("create copied dir");
        for entry in fs::read_dir(from).expect("read copied dir") {
            let entry = entry.expect("dir entry");
            copy_path(&entry.path(), &to.join(entry.file_name()));
        }
    } else {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).expect("create copied parent dir");
        }
        fs::copy(from, to).expect("copy file");
    }
}

struct ConsumerFixture {
    _tmp: TempDir,
    repo: PathBuf,
    fakebin: PathBuf,
    log_path: PathBuf,
    original_lockfile: String,
    driver_path: PathBuf,
    lint_lib_path: PathBuf,
    target_dir: PathBuf,
}

impl ConsumerFixture {
    fn new(label: &str, metadata_block: &str) -> Self {
        let bash = bash_path().expect("find bash");
        let tmp = TempDir::new(label);
        let repo = tmp.path().join("consumer");
        let fakebin = tmp.path().join("fakebin");
        let log_path = tmp.path().join("cargo.log");
        let driver_path = tmp.path().join("bin/dylint-driver");
        let lint_lib_path = tmp.path().join("lib/libtigerstyle.so");
        let target_dir = tmp.path().join("target");

        fs::create_dir_all(&repo).expect("create repo dir");
        fs::create_dir_all(&fakebin).expect("create fakebin");
        fs::create_dir_all(target_dir.as_path()).expect("create target dir");

        write_file(
            &repo.join("Cargo.toml"),
            &format!(
                r#"[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[workspace]
{metadata_block}
"#
            ),
        );
        write_file(&repo.join("src/lib.rs"), "pub fn noop() {}\n");
        write_file(&repo.join("dylint.toml"), "[tigerstyle]\n");
        write_file(&repo.join("Cargo.lock"), "# original lockfile\n");
        write_executable(
            &fakebin.join("cargo"),
            &format!(
                r#"#!{bash}
set -euo pipefail
printf 'ARGS:%s\n' "$*" >> "$TS_LOG"
printf 'RUSTC_WORKSPACE_WRAPPER=%s\n' "${{RUSTC_WORKSPACE_WRAPPER:-}}" >> "$TS_LOG"
printf 'DYLINT_LIBS=%s\n' "${{DYLINT_LIBS:-}}" >> "$TS_LOG"
printf 'DYLINT_NO_DEPS=%s\n' "${{DYLINT_NO_DEPS:-}}" >> "$TS_LOG"
printf 'RUSTUP_TOOLCHAIN=%s\n' "${{RUSTUP_TOOLCHAIN:-}}" >> "$TS_LOG"
case "${{1:-}}" in
  build)
    out_dir="${{CARGO_TARGET_DIR:?}}/release"
    mkdir -p "$out_dir"
    if [[ "${{TS_MUTATE_LOCKFILE_ON_BUILD:-0}}" == "1" ]]; then
      cat >> "${{TS_CONSUMER_LOCKFILE:?}}" <<'EOF'
# mutated by fake cargo build
EOF
    fi
    if [[ " $* " == *" -p tigerstyle "* ]]; then
      cat > "$out_dir/libtigerstyle.so" <<'EOF'
fake built tigerstyle dylint library
EOF
      toolchain="${{RUSTUP_TOOLCHAIN:?}}"
      ln -sf "$out_dir/libtigerstyle.so" "$out_dir/libtigerstyle@${{toolchain}}.so"
    else
      cat > "$out_dir/tigerstyle-dylint-driver" <<'EOF'
#!{bash}
echo bootstrapped fake dylint driver
EOF
      chmod +x "$out_dir/tigerstyle-dylint-driver"
    fi
    exit 0
    ;;
  check)
    if [[ "${{TS_MUTATE_LOCKFILE:-0}}" == "1" ]]; then
      cat >> Cargo.lock <<'EOF'
# mutated by fake cargo
EOF
    fi
    if [[ -n "${{TS_STDOUT_FILE:-}}" ]]; then
      cat "$TS_STDOUT_FILE"
    fi
    if [[ -n "${{TS_STDERR:-}}" ]]; then
      printf '%s\n' "$TS_STDERR" >&2
    fi
    if [[ "${{TS_KILL_SELF:-0}}" == "1" ]]; then
      kill -9 $$
    fi
    exit "${{TS_CARGO_EXIT_STATUS:-0}}"
    ;;
  *)
    echo "unexpected cargo invocation: $*" >&2
    exit 97
    ;;
esac
"#,
                bash = bash.display(),
            ),
        );
        write_executable(&driver_path, &format!("#!{}\necho fake dylint driver\n", bash.display()));
        write_file(&lint_lib_path, "fake tigerstyle dylint library\n");

        let original_lockfile = fs::read_to_string(repo.join("Cargo.lock")).expect("read original lockfile");

        Self {
            _tmp: tmp,
            repo,
            fakebin,
            log_path,
            original_lockfile,
            driver_path,
            lint_lib_path,
            target_dir,
        }
    }

    fn run(
        &self,
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            None,
            &[],
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    fn run_raw_os(
        &self,
        args: Vec<OsString>,
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        let path = format!("{}:{}", self.fakebin.display(), env::var("PATH").expect("PATH"));

        let mut command = match env::var("CARGO_TIGERSTYLE_BIN") {
            Ok(bin) if !bin.is_empty() => Command::new(bin),
            _ => match COMPILED_CARGO_TIGERSTYLE_BIN.filter(|bin| !bin.is_empty()) {
                Some(bin) => Command::new(bin),
                None => Command::new(built_cargo_tigerstyle_bin().expect("build cargo-tigerstyle binary")),
            },
        };
        command
            .args(args)
            .current_dir(&self.repo)
            .env("PATH", &path)
            .env("TS_LOG", &self.log_path)
            .env("TS_CARGO_EXIT_STATUS", cargo_exit_status.to_string())
            .env("TS_MUTATE_LOCKFILE", if mutate_lockfile { "1" } else { "0" })
            .env("TS_MUTATE_LOCKFILE_ON_BUILD", "0")
            .env("TS_CONSUMER_LOCKFILE", self.repo.join("Cargo.lock"))
            .env("CARGO_TARGET_DIR", &self.target_dir);

        fs::write(
            &self.log_path,
            format!(
                "LAUNCHER:{}\nPATH:{}\n",
                command.get_program().to_string_lossy(),
                path,
            ),
        )
        .expect("seed cargo log");

        match lint_lib_env {
            Some(value) => {
                command.env("TIGERSTYLE_LINT_LIB", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_LINT_LIB");
            }
        }

        match driver_env {
            Some(value) => {
                command.env("TIGERSTYLE_DRIVER_PATH", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DRIVER_PATH");
            }
        }

        match toolchain_env {
            Some(value) => {
                command.env("TIGERSTYLE_TOOLCHAIN", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_TOOLCHAIN");
            }
        }

        match dylint_rev_env {
            Some(value) => {
                command.env("TIGERSTYLE_DYLINT_REV", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DYLINT_REV");
            }
        }

        command.output().expect("run cargo-tigerstyle")
    }

    fn run_with_launcher(
        &self,
        launcher: Option<&Path>,
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_with_launcher_and_env(
            launcher,
            &[],
            args,
            cargo_exit_status,
            mutate_lockfile,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    fn run_with_launcher_and_env(
        &self,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            launcher,
            extra_env,
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    fn run_from_dir_with_launcher_and_env(
        &self,
        current_dir: &Path,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            Some(current_dir),
            launcher,
            extra_env,
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    fn run_with_bootstrap_lockfile_mutation(
        &self,
        args: &[&str],
        cargo_exit_status: i32,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            None,
            &[],
            args,
            cargo_exit_status,
            false,
            true,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    fn run_internal(
        &self,
        current_dir: Option<&Path>,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        mutate_lockfile_on_build: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        let path = format!("{}:{}", self.fakebin.display(), env::var("PATH").expect("PATH"));

        let mut command = match launcher {
            Some(bin) => Command::new(bin),
            None => match env::var("CARGO_TIGERSTYLE_BIN") {
                Ok(bin) if !bin.is_empty() => Command::new(bin),
                _ => match COMPILED_CARGO_TIGERSTYLE_BIN.filter(|bin| !bin.is_empty()) {
                    Some(bin) => Command::new(bin),
                    None => Command::new(built_cargo_tigerstyle_bin().expect("build cargo-tigerstyle binary")),
                },
            },
        };
        command
            .arg("check")
            .args(args)
            .current_dir(current_dir.unwrap_or(self.repo.as_path()))
            .env("PATH", &path)
            .env("TS_LOG", &self.log_path)
            .env("TS_CARGO_EXIT_STATUS", cargo_exit_status.to_string())
            .env("TS_MUTATE_LOCKFILE", if mutate_lockfile { "1" } else { "0" })
            .env(
                "TS_MUTATE_LOCKFILE_ON_BUILD",
                if mutate_lockfile_on_build { "1" } else { "0" },
            )
            .env("TS_CONSUMER_LOCKFILE", self.repo.join("Cargo.lock"))
            .env("CARGO_TARGET_DIR", &self.target_dir);

        fs::write(
            &self.log_path,
            format!(
                "LAUNCHER:{}\nPATH:{}\n",
                command.get_program().to_string_lossy(),
                path,
            ),
        )
        .expect("seed cargo log");

        match lint_lib_env {
            Some(value) => {
                command.env("TIGERSTYLE_LINT_LIB", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_LINT_LIB");
            }
        }

        match driver_env {
            Some(value) => {
                command.env("TIGERSTYLE_DRIVER_PATH", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DRIVER_PATH");
            }
        }

        match toolchain_env {
            Some(value) => {
                command.env("TIGERSTYLE_TOOLCHAIN", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_TOOLCHAIN");
            }
        }

        match dylint_rev_env {
            Some(value) => {
                command.env("TIGERSTYLE_DYLINT_REV", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DYLINT_REV");
            }
        }

        for (key, value) in extra_env {
            command.env(key, value);
        }

        command.output().expect("run cargo-tigerstyle")
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log_path).unwrap_or_default()
    }

    fn lockfile(&self) -> String {
        fs::read_to_string(self.repo.join("Cargo.lock")).expect("read lockfile")
    }
}

fn metadata(default_scope: &str, cargo_check_args: &str) -> String {
    format!(
        r#"[workspace.metadata.tigerstyle]
default_scope = {default_scope}
cargo_check_args = {cargo_check_args}
"#
    )
}

fn bash_path() -> Result<PathBuf, String> {
    BASH_PATH
        .get_or_init(|| {
            if let Some(path) = env::var_os("BASH").map(PathBuf::from).filter(|path| path.is_file()) {
                return Ok(path);
            }

            let path = env::var_os("PATH").ok_or_else(|| "missing `PATH` env var".to_owned())?;
            env::split_paths(&path)
                .map(|dir| dir.join("bash"))
                .find(|candidate| candidate.is_file())
                .ok_or_else(|| "could not find `bash` on PATH".to_owned())
        })
        .clone()
}

fn built_cargo_tigerstyle_bin() -> Result<PathBuf, String> {
    BUILT_CARGO_TIGERSTYLE_BIN
        .get_or_init(|| {
            let cargo = env::var("CARGO").map_err(|_| "missing `CARGO` env var".to_owned())?;
            let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cargo-tigerstyle/Cargo.toml");
            let target_dir = env::temp_dir().join("tigerstyle-consumer-integration-cli-target");

            let status = Command::new(cargo)
                .arg("build")
                .arg("-q")
                .arg("--locked")
                .arg("--manifest-path")
                .arg(&manifest_path)
                .arg("--bin")
                .arg("cargo-tigerstyle")
                .arg("--target-dir")
                .arg(&target_dir)
                .status()
                .map_err(|error| format!("build cargo-tigerstyle binary: {error}"))?;
            if !status.success() {
                return Err("cargo-tigerstyle build failed".to_owned());
            }

            let binary_path = target_dir.join("debug").join(format!("cargo-tigerstyle{}", env::consts::EXE_SUFFIX));
            if !binary_path.is_file() {
                return Err(format!("built cargo-tigerstyle binary missing at {}", binary_path.display()));
            }

            Ok(binary_path)
        })
        .clone()
}

fn build_standalone_cargo_tigerstyle() -> StandaloneCargoTigerstyle {
    let tmp = TempDir::new("installed-cargo-tigerstyle");
    let source_root = tmp.path().join("source");
    let target_dir = tmp.path().join("target");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src", "cargo-tigerstyle"] {
        copy_path(&repo_root.join(relative), &source_root.join(relative));
    }

    let cargo = env::var("CARGO").expect("CARGO env var");
    let manifest_path = source_root.join("cargo-tigerstyle/Cargo.toml");
    let status = Command::new(cargo)
        .arg("build")
        .arg("-q")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--bin")
        .arg("cargo-tigerstyle")
        .arg("--target-dir")
        .arg(&target_dir)
        .status()
        .expect("build standalone cargo-tigerstyle");
    assert!(status.success(), "standalone cargo-tigerstyle build failed: {status}");

    let bin_path = target_dir.join("debug").join(format!("cargo-tigerstyle{}", env::consts::EXE_SUFFIX));
    assert!(bin_path.is_file(), "standalone cargo-tigerstyle binary missing at {}", bin_path.display());

    fs::remove_dir_all(&source_root).expect("remove standalone source root");
    assert!(!source_root.exists(), "standalone source root should be gone: {}", source_root.display());

    StandaloneCargoTigerstyle {
        _tmp: tmp,
        source_root,
        bin_path,
    }
}

fn expected_repo_toolchain() -> String {
    let rust_toolchain = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml"))
        .expect("read repo rust-toolchain.toml");
    let document = rust_toolchain.parse::<toml::Table>().expect("parse repo rust-toolchain.toml");
    let channel = document
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("channel"))
        .and_then(toml::Value::as_str)
        .expect("toolchain.channel");

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc).arg("-vV").output().expect("run rustc -vV for host triple");
    assert!(output.status.success(), "rustc -vV failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("rustc -vV stdout utf8");
    let host = stdout.lines().find_map(|line| line.strip_prefix("host: ")).expect("rustc host triple");

    if channel.ends_with(host) {
        channel.to_owned()
    } else {
        format!("{channel}-{host}")
    }
}

#[test]
fn cargo_style_argv_prefix_parses_like_bare_check() {
    let fixture = ConsumerFixture::new("cargo-style-argv", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_raw_os(
        vec![OsString::from("tigerstyle"), OsString::from("check")],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-style argv prefix should behave like bare check: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "cargo-style argv prefix should still invoke cargo check, got: {log}");
    assert!(log.contains("-p consumer"), "cargo-style argv prefix should preserve metadata default scope, got: {log}");
    assert!(log.contains("--lib"), "cargo-style argv prefix should preserve metadata cargo_check_args, got: {log}");
}

#[test]
fn no_argument_check_uses_workspace_metadata_defaults() {
    let fixture = ConsumerFixture::new("metadata-defaults", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "expected cargo check invocation, got: {log}");
    assert!(log.contains("-p consumer"), "expected metadata default scope, got: {log}");
    assert!(log.contains("--lib"), "expected metadata cargo_check_args, got: {log}");
    assert!(log.contains("DYLINT_NO_DEPS=1"), "expected DYLINT_NO_DEPS=1, got: {log}");
}

#[test]
fn no_argument_check_from_nested_workspace_subdirectory_uses_workspace_metadata_defaults() {
    let fixture = ConsumerFixture::new("metadata-defaults-subdir", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let nested_dir = fixture.repo.join("nested/deeper");
    fs::create_dir_all(&nested_dir).unwrap_or_else(|error| panic!("create {}: {error}", nested_dir.display()));

    let output = fixture.run_from_dir_with_launcher_and_env(
        &nested_dir,
        None,
        &[],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle from a nested workspace subdirectory should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "expected cargo check invocation, got: {log}");
    assert!(log.contains("-p consumer"), "expected metadata default scope from nested dir, got: {log}");
    assert!(log.contains("--lib"), "expected metadata cargo_check_args from nested dir, got: {log}");
    assert!(log.contains("DYLINT_NO_DEPS=1"), "expected DYLINT_NO_DEPS=1 from nested dir, got: {log}");
}

#[test]
fn explicit_workspace_scope_override_beats_metadata_defaults() {
    let fixture = ConsumerFixture::new("workspace-override", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--workspace"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed with explicit --workspace: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("--workspace"), "expected explicit --workspace scope override, got: {log}");
    assert!(!log.contains("-p consumer"), "explicit --workspace should replace metadata default scope: {log}");
    assert!(log.contains("--lib"), "metadata cargo_check_args should still apply without explicit cargo tail args: {log}");
}

#[test]
fn repeated_package_scope_flags_forward_in_order_and_replace_metadata_defaults() {
    let fixture = ConsumerFixture::new("repeated-package-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--package", "consumer", "-p", "other"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed with repeated package scope flags: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let consumer_pos = log.find("--package consumer").expect("expected --package consumer in cargo log");
    let other_pos = log.find("-p other").expect("expected -p other in cargo log");
    assert!(consumer_pos < other_pos, "repeated package scope flags should preserve order: {log}");
    assert!(!log.contains("-p consumer"), "repeated package scope flags should replace metadata default scope: {log}");
    assert!(log.contains("--lib"), "metadata cargo_check_args should still apply without explicit cargo tail args: {log}");
}

#[test]
fn bare_double_dash_replaces_metadata_cargo_check_args_with_nothing() {
    let fixture = ConsumerFixture::new("bare-double-dash", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed with a bare -- override: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("-p consumer"), "bare -- should keep metadata default scope: {log}");
    assert!(!log.contains("--lib"), "bare -- should replace metadata cargo_check_args with an empty explicit tail: {log}");
}

#[test]
fn explicit_workspace_scope_with_cargo_tail_replaces_metadata_scope_and_args() {
    let fixture = ConsumerFixture::new("workspace-tail-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--workspace", "--", "--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed with explicit --workspace plus cargo tail: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("--workspace"), "expected explicit --workspace scope override, got: {log}");
    assert!(log.contains("--all-targets"), "expected explicit cargo tail args, got: {log}");
    assert!(!log.contains("-p consumer"), "explicit --workspace should replace metadata default scope: {log}");
    assert!(!log.contains("--lib"), "explicit cargo tail args should replace metadata defaults even with --workspace: {log}");
}

#[test]
fn explicit_scope_and_check_args_override_metadata_defaults() {
    let fixture = ConsumerFixture::new("explicit-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["-p", "override-crate", "--", "--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-tigerstyle should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("-p override-crate"), "expected explicit scope override, got: {log}");
    assert!(log.contains("--all-targets"), "expected explicit cargo tail args, got: {log}");
    assert!(!log.contains("--lib"), "explicit cargo tail args should replace metadata defaults: {log}");
}

#[test]
fn resolves_toolchain_from_repo_when_env_is_missing() {
    let fixture = ConsumerFixture::new("toolchain-fallback", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "toolchain fallback should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let expected = expected_repo_toolchain();
    assert!(
        log.contains(&format!("RUSTUP_TOOLCHAIN={expected}")),
        "expected derived toolchain `{expected}` in cargo env log, got: {log}"
    );
}

#[test]
fn bootstraps_driver_when_explicit_path_is_missing() {
    let fixture = ConsumerFixture::new("driver-bootstrap", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    let log = fixture.log();
    assert!(
        output.status.success(),
        "driver bootstrap should succeed: stdout={} stderr={} log={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        log,
    );

    let build_pos = log.find("ARGS:build").expect("saw bootstrap build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "bootstrap build should happen before cargo check: {log}");
    assert!(
        !log.contains(&format!("RUSTC_WORKSPACE_WRAPPER={}\n", fixture.driver_path.display())),
        "bootstrap path should not reuse the explicit fixture driver path: {log}"
    );
}

#[test]
fn invalid_metadata_exits_with_integration_failure_code() {
    let fixture = ConsumerFixture::new(
        "invalid-metadata",
        r#"[workspace.metadata.tigerstyle]
default_scope = ["-p", "consumer"]
"#,
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid metadata should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "invalid metadata should fail before invoking cargo: {log}");
}

#[test]
fn missing_metadata_block_exits_with_integration_failure_code() {
    let fixture = ConsumerFixture::new("missing-metadata", "");

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing workspace.metadata.tigerstyle should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "missing metadata should fail before invoking cargo: {log}");
}

#[test]
fn non_string_metadata_array_entries_exit_with_integration_failure_code() {
    let fixture = ConsumerFixture::new(
        "non-string-metadata",
        r#"[workspace.metadata.tigerstyle]
default_scope = ["-p", 7]
cargo_check_args = ["--lib"]
"#,
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "non-string metadata entries should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "non-string metadata should fail before invoking cargo: {log}");
}

#[test]
fn equals_sign_package_form_is_rejected_as_unsupported() {
    let fixture = ConsumerFixture::new("equals-sign-package", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--package=consumer"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "equals-sign package form should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: unsupported argument before `--`: `--package=consumer`"),
        "equals-sign package form should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "equals-sign package form should fail before invoking cargo: {log}");
}

#[test]
fn unsupported_arguments_before_double_dash_exit_with_integration_failure_code() {
    let fixture = ConsumerFixture::new("unsupported-arg", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "unsupported arguments before `--` should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "unsupported arguments should fail before invoking cargo: {log}");
}

#[test]
fn unsupported_arguments_before_double_dash_preserve_diagnostic_stderr() {
    let fixture = ConsumerFixture::new("unsupported-arg-stderr", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: unsupported argument before `--`: `--all-targets`"),
        "unsupported-argument parse failure should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
}

#[test]
fn non_utf8_cli_arguments_exit_with_integration_failure_code_and_preserve_diagnostic() {
    let fixture = ConsumerFixture::new("non-utf8-arg", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_raw_os(
        vec![OsString::from("check"), OsString::from_vec(vec![b'f', 0x80, b'o'])],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "non-utf8 CLI arguments should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: non-utf8 arguments are not supported"),
        "non-utf8 CLI arguments should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "non-utf8 CLI arguments should fail before invoking cargo: {log}");
}

#[test]
fn missing_and_unknown_subcommands_preserve_diagnostics_and_skip_cargo() {
    let assert_subcommand_failure = |label: &str, args: Vec<OsString>, expected_stderr: &str| {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run_raw_os(
            args,
            0,
            false,
            Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
            Some(fixture.driver_path.as_path()),
            Some(fixture.lint_lib_path.as_path()),
            None,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for CLI misuse: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "{label} should preserve exact diagnostic stderr: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    assert_subcommand_failure("missing-subcommand", vec![], "error: missing subcommand; expected `check`");
    assert_subcommand_failure(
        "unknown-subcommand",
        vec![OsString::from("lint")],
        "error: unknown subcommand `lint`; expected `check`",
    );
}

#[test]
fn missing_package_value_after_scope_flag_exits_with_integration_failure_code_and_preserves_diagnostic() {
    let assert_missing_value_failure = |label: &str, args: &[&str], expected_stderr: &str| {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run(
            args,
            0,
            false,
            Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
            Some(fixture.driver_path.as_path()),
            Some(fixture.lint_lib_path.as_path()),
            None,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for a dangling scope flag: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "{label} should preserve exact diagnostic stderr: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    assert_missing_value_failure("missing-package-short", &["-p"], "error: missing value after `-p`");
    assert_missing_value_failure(
        "missing-package-long",
        &["--package"],
        "error: missing value after `--package`",
    );
}

#[test]
fn cargo_termination_without_exit_code_exits_two_and_restores_lockfile() {
    let fixture = ConsumerFixture::new("signal-terminated-cargo", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_KILL_SELF", "1")],
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "signal-terminated cargo should exit 2 from cargo-tigerstyle: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: cargo check terminated without an exit code"),
        "signal-terminated cargo should preserve the no-exit-code diagnostic: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(
        stderr.contains("cargo tigerstyle check modified"),
        "signal-terminated cargo should also report fail-closed lockfile restoration: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "signal-terminated cargo should restore the consumer lockfile");
}

#[test]
fn cargo_termination_without_exit_code_and_without_lockfile_mutation_reports_only_root_cause() {
    let fixture = ConsumerFixture::new("signal-terminated-clean-lockfile", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_KILL_SELF", "1")],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "clean signal-terminated cargo should exit 2 from cargo-tigerstyle: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: cargo check terminated without an exit code"),
        "clean signal-terminated cargo should preserve the no-exit-code diagnostic: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(
        !stderr.contains("cargo tigerstyle check modified"),
        "clean signal-terminated cargo should not claim lockfile restoration when no mutation happened: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "clean signal-terminated cargo should leave the consumer lockfile unchanged");
}

#[test]
fn explicit_empty_tigerstyle_env_vars_exit_with_integration_failure_code() {
    let assert_empty_env_failure = |
        label: &str,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    | {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run(
            &[],
            0,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for an explicit empty TIGERSTYLE env var: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    let toolchain_fixture = ConsumerFixture::new("empty-toolchain-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-toolchain",
        Some(""),
        Some(toolchain_fixture.driver_path.as_path()),
        Some(toolchain_fixture.lint_lib_path.as_path()),
        None,
    );

    let driver_fixture = ConsumerFixture::new("empty-driver-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-driver-path",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(Path::new("")),
        Some(driver_fixture.lint_lib_path.as_path()),
        None,
    );

    let lint_fixture = ConsumerFixture::new("empty-lint-lib-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-lint-lib",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(lint_fixture.driver_path.as_path()),
        Some(Path::new("")),
        None,
    );

    let dylint_rev_fixture = ConsumerFixture::new("empty-dylint-rev-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-dylint-rev",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(dylint_rev_fixture.lint_lib_path.as_path()),
        Some(""),
    );
}

#[test]
fn error_level_lint_findings_exit_with_code_one() {
    let fixture = ConsumerFixture::new("lint-failure", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        1,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "lint failures should exit 1: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "lint failure should not dirty lockfile");
}

#[test]
fn warning_level_lint_findings_exit_with_code_zero() {
    let fixture = ConsumerFixture::new("lint-warning", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_STDERR", "warning: tigerstyle warning-only finding")],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "warning-only lint findings should stay non-fatal: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("warning: tigerstyle warning-only finding"),
        "warning-only stderr should be preserved: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "warning-only lint finding should not dirty lockfile");
}

#[test]
fn lockfile_mutation_restores_original_and_exits_two() {
    let fixture = ConsumerFixture::new("lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "lockfile should be restored after mutation");
}

#[test]
fn missing_lockfile_mutation_removes_generated_lockfile_and_exits_two() {
    let fixture = ConsumerFixture::new("missing-lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    fs::remove_file(fixture.repo.join("Cargo.lock")).expect("remove consumer Cargo.lock before test");
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "consumer Cargo.lock should start missing for the missing-lockfile contract"
    );

    let output = fixture.run(
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing-lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "generated consumer Cargo.lock should be removed again after fail-closed restoration"
    );
}

#[test]
fn missing_lockfile_clean_success_preserves_absence() {
    let fixture = ConsumerFixture::new("missing-lockfile-success", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    fs::remove_file(fixture.repo.join("Cargo.lock")).expect("remove consumer Cargo.lock before clean-success test");
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "consumer Cargo.lock should start missing for the clean-success missing-lockfile contract"
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    assert!(
        output.status.success(),
        "missing-lockfile clean success should stay green: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "clean successful run should preserve the absence of consumer Cargo.lock"
    );
}

#[test]
fn bootstrap_lockfile_mutation_restores_original_and_exits_two() {
    let fixture = ConsumerFixture::new("bootstrap-lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_bootstrap_lockfile_mutation(
        &[],
        0,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "bootstrap-phase lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "bootstrap mutation should restore the consumer lockfile");
}

#[test]
fn bootstraps_lint_library_when_explicit_path_is_missing() {
    let fixture = ConsumerFixture::new("lint-lib-bootstrap", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "lint-library bootstrap should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let build_pos = log.find("ARGS:build").expect("saw lint-library build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "lint-library build should happen before cargo check: {log}");
    assert!(
        !log.contains(&format!("DYLINT_LIBS=[\"{}\"]\n", fixture.lint_lib_path.display())),
        "lint-library bootstrap should not reuse the explicit fixture lint path: {log}"
    );
    assert!(
        log.contains("libtigerstyle@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"),
        "bootstrapped lint library should use the toolchain-suffixed dylint path: {log}"
    );
}

fn first_embedded_source_dir(cache_home: &Path) -> PathBuf {
    let sources_root = cache_home.join("tigerstyle/sources");
    let entries = fs::read_dir(&sources_root)
        .unwrap_or_else(|error| panic!("read embedded source cache {}: {error}", sources_root.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "expected exactly one embedded source dir under {}", sources_root.display());
    entries.into_iter().next().expect("embedded source dir")
}

fn set_tree_permissions(root: &Path, dir_mode: u32, file_mode: u32) {
    let metadata = fs::metadata(root).unwrap_or_else(|error| panic!("metadata {}: {error}", root.display()));
    let mode = if metadata.is_dir() { dir_mode } else { file_mode };
    let mut perms = metadata.permissions();
    perms.set_mode(mode);
    fs::set_permissions(root, perms).unwrap_or_else(|error| panic!("chmod {}: {error}", root.display()));
    if metadata.is_dir() {
        for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("read dir {}: {error}", root.display())) {
            let entry = entry.unwrap_or_else(|error| panic!("dir entry under {}: {error}", root.display()));
            set_tree_permissions(&entry.path(), dir_mode, file_mode);
        }
    }
}

#[test]
fn installed_cargo_subcommand_works_without_repo_checkout() {
    let standalone = build_standalone_cargo_tigerstyle();
    let fixture = ConsumerFixture::new("installed-subcommand", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher(
        Some(&standalone.bin_path),
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "installed cargo-tigerstyle should succeed without a repo checkout: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let build_pos = log.find("ARGS:build").expect("saw embedded lint-library build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "lint-library build should happen before cargo check: {log}");
    assert!(
        !log.contains(&standalone.source_root.display().to_string()),
        "installed cargo-tigerstyle should not rely on the removed standalone source tree: {log}"
    );
}

#[test]
fn installed_cargo_subcommand_reuses_valid_embedded_source_cache_without_reextracting() {
    let standalone = build_standalone_cargo_tigerstyle();
    let fixture = ConsumerFixture::new("installed-cache-hit", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let cache_home_tmp = TempDir::new("installed-cache-hit-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    let cache_home_value = cache_home.display().to_string();

    let first_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        first_output.status.success(),
        "first installed cargo-tigerstyle run should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&first_output.stdout),
        String::from_utf8_lossy(&first_output.stderr)
    );

    let source_dir = first_embedded_source_dir(&cache_home);
    set_tree_permissions(&source_dir, 0o555, 0o444);

    let second_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        second_output.status.success(),
        "cached installed cargo-tigerstyle run should succeed without re-extracting: stdout={} stderr={}",
        String::from_utf8_lossy(&second_output.stdout),
        String::from_utf8_lossy(&second_output.stderr)
    );
    let marker_mode = fs::metadata(source_dir.join(".tigerstyle-source-blake3"))
        .expect("embedded source marker metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(marker_mode, 0o444, "embedded source marker should stay read-only when cache is reused");
    set_tree_permissions(&source_dir, 0o755, 0o644);
}

#[test]
fn installed_cargo_subcommand_recovers_corrupt_embedded_source_cache() {
    let standalone = build_standalone_cargo_tigerstyle();
    let fixture = ConsumerFixture::new("installed-cache-recovery", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let cache_home_tmp = TempDir::new("installed-cache-recovery-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    let cache_home_value = cache_home.display().to_string();

    let first_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        first_output.status.success(),
        "first installed cargo-tigerstyle run should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&first_output.stdout),
        String::from_utf8_lossy(&first_output.stderr)
    );

    let source_dir = first_embedded_source_dir(&cache_home);
    let marker_path = source_dir.join(".tigerstyle-source-blake3");
    fs::write(&marker_path, "corrupt-marker\n").unwrap_or_else(|error| panic!("write {}: {error}", marker_path.display()));

    let second_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        second_output.status.success(),
        "installed cargo-tigerstyle should recover corrupt embedded source cache: stdout={} stderr={}",
        String::from_utf8_lossy(&second_output.stdout),
        String::from_utf8_lossy(&second_output.stderr)
    );
    let marker = fs::read_to_string(&marker_path).unwrap_or_else(|error| panic!("read {}: {error}", marker_path.display()));
    assert_ne!(marker.trim_end(), "corrupt-marker", "embedded source cache marker should be rewritten after recovery");
}

#[test]
fn installed_cargo_subcommand_falls_back_to_temp_cache_when_home_caches_are_unwritable() {
    let standalone = build_standalone_cargo_tigerstyle();
    let fixture = ConsumerFixture::new("installed-temp-cache-fallback", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let blocked_roots = TempDir::new("installed-temp-cache-blocked");
    let blocked_xdg = blocked_roots.path().join("blocked-xdg");
    let blocked_home = blocked_roots.path().join("blocked-home");
    fs::create_dir_all(&blocked_xdg).unwrap_or_else(|error| panic!("create {}: {error}", blocked_xdg.display()));
    fs::create_dir_all(&blocked_home).unwrap_or_else(|error| panic!("create {}: {error}", blocked_home.display()));
    set_tree_permissions(&blocked_xdg, 0o555, 0o444);
    set_tree_permissions(&blocked_home, 0o555, 0o444);

    let temp_root_tmp = TempDir::new("installed-temp-cache-root");
    let temp_root = temp_root_tmp.path().join("tmp-root");
    fs::create_dir_all(&temp_root).unwrap_or_else(|error| panic!("create {}: {error}", temp_root.display()));

    let blocked_xdg_value = blocked_xdg.display().to_string();
    let blocked_home_value = blocked_home.display().to_string();
    let temp_root_value = temp_root.display().to_string();
    let output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[
            ("XDG_CACHE_HOME", blocked_xdg_value.as_str()),
            ("HOME", blocked_home_value.as_str()),
            ("TMPDIR", temp_root_value.as_str()),
        ],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "installed cargo-tigerstyle should fall back to temp cache when home caches are unwritable: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let temp_cache_root = temp_root.join("tigerstyle-cache");
    let source_dir = first_embedded_source_dir(&temp_cache_root);
    assert!(
        source_dir.starts_with(&temp_cache_root),
        "embedded source cache should be materialized under temp fallback root {} but got {}",
        temp_cache_root.display(),
        source_dir.display()
    );
    assert!(
        !blocked_xdg.join("tigerstyle").exists(),
        "unwritable XDG cache root should stay unused: {}",
        blocked_xdg.display()
    );
    assert!(
        !blocked_home.join(".cache").exists(),
        "unwritable HOME cache root should stay unused: {}",
        blocked_home.display()
    );

    set_tree_permissions(&blocked_xdg, 0o755, 0o644);
    set_tree_permissions(&blocked_home, 0o755, 0o644);
}

#[test]
fn installed_cargo_subcommand_handles_concurrent_first_use_of_shared_embedded_source_cache() {
    let standalone = build_standalone_cargo_tigerstyle();
    let cache_home_tmp = TempDir::new("installed-concurrent-cache-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    fs::create_dir_all(&cache_home).unwrap_or_else(|error| panic!("create {}: {error}", cache_home.display()));

    let mut handles = Vec::new();
    for label in ["installed-concurrent-cache-a", "installed-concurrent-cache-b"] {
        let bin_path = standalone.bin_path.clone();
        let cache_home = cache_home.clone();
        handles.push(std::thread::spawn(move || {
            let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
            let cache_home_value = cache_home.display().to_string();
            let output = fixture.run_with_launcher_and_env(
                Some(bin_path.as_path()),
                &[("XDG_CACHE_HOME", cache_home_value.as_str())],
                &[],
                0,
                false,
                None,
                Some(fixture.driver_path.as_path()),
                None,
                None,
            );
            (label, output)
        }));
    }

    for handle in handles {
        let (label, output) = handle.join().expect("join concurrent installed cargo-tigerstyle run");
        assert!(
            output.status.success(),
            "{label} should succeed during concurrent first-use cache extraction: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let source_dir = first_embedded_source_dir(&cache_home);
    let file_name = source_dir
        .file_name()
        .and_then(|value| value.to_str())
        .expect("embedded source dir name");
    assert!(
        !file_name.starts_with('.'),
        "concurrent cache extraction should leave the final hash dir, not a staging dir: {}",
        source_dir.display()
    );
    assert!(
        source_dir.join(".tigerstyle-source-blake3").is_file(),
        "concurrent cache extraction should leave a valid hash marker in {}",
        source_dir.display()
    );
}

#[test]
fn checked_in_nix_consumer_fixture_is_reviewable() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/consumer-integration/nix-consumer");
    let cargo_toml = fs::read_to_string(fixture_root.join("Cargo.toml"))
        .unwrap_or_else(|error| panic!("read checked-in nix consumer Cargo.toml under {}: {error}", fixture_root.display()));
    let flake = fs::read_to_string(fixture_root.join("flake.nix"))
        .unwrap_or_else(|error| panic!("read checked-in nix consumer flake under {}: {error}", fixture_root.display()));

    assert!(
        cargo_toml.contains("[workspace.metadata.tigerstyle]"),
        "checked-in nix consumer fixture must carry workspace tigerstyle metadata: {}",
        fixture_root.display()
    );
    assert!(
        flake.contains("mkConsumerCheck"),
        "checked-in nix consumer fixture must exercise the first-party Nix helper: {}",
        fixture_root.display()
    );
}

#[test]
fn downstream_rollout_docs_point_to_first_class_cli_path() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/downstream-rollout.md");
    let source = fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("read downstream rollout doc {}: {error}", doc_path.display()));

    for required in [
        "cargo tigerstyle check",
        "[workspace.metadata.tigerstyle]",
        "nix run",
        "cargo-tigerstyle",
    ] {
        assert!(
            source.contains(required),
            "downstream rollout doc must mention `{required}`: {}",
            doc_path.display()
        );
    }
    assert!(
        !source.contains("Copy `templates/tigerstyle-check.sh`"),
        "downstream rollout doc should not keep the copied template as the active recommended path: {}",
        doc_path.display()
    );
}

#[test]
fn thin_wrapper_fixture_delegates_without_policy_logic() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/consumer-integration/justfile");
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("read thin-wrapper fixture {}: {error}", fixture_path.display()));

    assert!(
        source.contains("cargo tigerstyle check"),
        "thin-wrapper fixture must delegate to cargo tigerstyle check: {}",
        fixture_path.display()
    );
    for forbidden in [
        "RUSTC_WORKSPACE_WRAPPER",
        "DYLINT_LIBS",
        "DYLINT_NO_DEPS",
        "TIGERSTYLE_TOOLCHAIN",
        "cargo check",
        "cargo dylint",
    ] {
        assert!(
            !source.contains(forbidden),
            "thin-wrapper fixture must not reimplement rollout policy with `{forbidden}`: {}",
            fixture_path.display()
        );
    }
}

// --- Structured output integration tests ---

fn cargo_compiler_message(target_name: &str, lint_name: &str, message: &str, file: &str, line: u32, col: u32, suggestion: Option<(&str, &str)>) -> String {
    let children = match suggestion {
        Some((replacement, applicability)) => format!(
            r#"[{{"message":"try this","code":null,"level":"help","spans":[{{"file_name":"{file}","byte_start":0,"byte_end":1,"line_start":{line},"line_end":{line},"column_start":{col},"column_end":{col},"is_primary":true,"text":[],"label":null,"suggested_replacement":"{replacement}","suggestion_applicability":"{applicability}","expansion":null}}],"children":[],"rendered":null}}]"#
        ),
        None => "[]".to_owned(),
    };
    format!(
        r#"{{"reason":"compiler-message","package_id":"path+file:///fake#{target_name}@0.1.0","manifest_path":"/fake/Cargo.toml","target":{{"kind":["lib"],"crate_types":["lib"],"name":"{target_name}","src_path":"/fake/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true}},"message":{{"$message_type":"diagnostic","message":"{message}","code":{{"code":"{lint_name}","explanation":null}},"level":"warning","spans":[{{"file_name":"{file}","byte_start":0,"byte_end":1,"line_start":{line},"line_end":{line},"column_start":{col},"column_end":{col},"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}],"children":{children},"rendered":"warning[{lint_name}]: {message}\n --> {file}:{line}:{col}\n"}}}}"#
    )
}

fn cargo_build_finished(success: bool) -> String {
    format!(r#"{{"reason":"build-finished","success":{success}}}"#)
}

fn cargo_compiler_artifact(target_name: &str, fresh: bool) -> String {
    format!(
        r#"{{"reason":"compiler-artifact","package_id":"path+file:///fake#{target_name}@0.1.0","manifest_path":"/fake/Cargo.toml","target":{{"kind":["lib"],"crate_types":["lib"],"name":"{target_name}","src_path":"/fake/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true}},"profile":{{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false}},"features":[],"filenames":["/fake/target/debug/lib{target_name}.rlib"],"executable":null,"fresh":{fresh}}}"#
    )
}

fn write_json_stdout_file(fixture: &ConsumerFixture, lines: &[String]) -> PathBuf {
    let path = fixture.repo.join("../.cargo-json-stdout");
    fs::write(&path, lines.join("\n") + "\n").expect("write JSON stdout file");
    path
}

fn run_with_output_format(
    fixture: &ConsumerFixture,
    output_format: &str,
    json_lines: &[String],
    cargo_exit: i32,
) -> std::process::Output {
    let stdout_file = write_json_stdout_file(fixture, json_lines);
    fixture.run_with_launcher_and_env(
        None,
        &[
            ("TS_STDOUT_FILE", stdout_file.to_str().unwrap()),
        ],
        &["--output-format", output_format],
        cargo_exit,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    )
}

fn three_finding_json_lines() -> Vec<String> {
    vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message(
            "aspen-core", "no_unwrap",
            "use of `.unwrap()`",
            "src/connection.rs", 42, 5,
            Some((".expect(\\\"reason\\\")", "MaybeIncorrect")),
        ),
        cargo_compiler_message(
            "aspen-core", "function_length",
            "function `handle_request` is 82 lines long (limit: 70)",
            "src/router.rs", 15, 1,
            None,
        ),
        cargo_compiler_artifact("aspen-dns", false),
        cargo_compiler_message(
            "aspen-dns", "unbounded_channel",
            "use of unbounded channel",
            "src/resolver.rs", 28, 9,
            None,
        ),
        cargo_build_finished(true),
    ]
}

#[test]
fn structured_json_output_contains_all_required_fields() {
    let fixture = ConsumerFixture::new("json-output", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "json", &three_finding_json_lines(), 0);

    assert_eq!(output.status.code(), Some(1), "findings should exit 1: stderr={}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse JSON finding"))
        .collect();

    assert_eq!(findings.len(), 3, "expected 3 findings, got: {stdout}");

    for (i, f) in findings.iter().enumerate() {
        assert_eq!(f["version"], 1, "finding {i} missing version");
        assert!(f["index"].as_str().is_some(), "finding {i} missing index");
        assert!(f["lint"].as_str().is_some(), "finding {i} missing lint");
        assert!(f["group"].as_str().is_some(), "finding {i} missing group");
        assert!(f["severity"].as_str().is_some(), "finding {i} missing severity");
        assert!(f["file"].as_str().is_some(), "finding {i} missing file");
        assert!(f["line"].is_number(), "finding {i} missing line");
        assert!(f["column"].is_number(), "finding {i} missing column");
        assert!(f["message"].as_str().is_some(), "finding {i} missing message");
        assert!(f["principle"].as_str().is_some(), "finding {i} missing principle");
        assert!(f["crate_name"].as_str().is_some(), "finding {i} missing crate_name");
    }

    assert_eq!(findings[0]["lint"], "no_unwrap");
    assert_eq!(findings[0]["group"], "tigerstyle::safety");
    assert_eq!(findings[0]["file"], "src/connection.rs");
    assert_eq!(findings[0]["line"], 42);
    assert_eq!(findings[0]["crate_name"], "aspen-core");

    assert_eq!(findings[1]["lint"], "function_length");
    assert_eq!(findings[1]["group"], "tigerstyle::structure");

    assert_eq!(findings[2]["lint"], "unbounded_channel");
    assert_eq!(findings[2]["crate_name"], "aspen-dns");
}

#[test]
fn json_output_coexists_with_summary_on_stderr() {
    let fixture = ConsumerFixture::new("json-stderr", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "json", &three_finding_json_lines(), 0);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // stdout is pure JSON-lines
    for line in stdout.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "stdout line is not valid JSON: {line}"
        );
    }

    // stderr has summary but no human diagnostics
    assert!(stderr.contains("--- tigerstyle summary ---"), "stderr missing summary header: {stderr}");
    assert!(stderr.contains("Findings: 3"), "stderr missing finding count: {stderr}");
    assert!(!stderr.contains("warning[no_unwrap]"), "stderr should not contain human diagnostics in json mode: {stderr}");
}

#[test]
fn sarif_output_has_required_structural_fields() {
    let fixture = ConsumerFixture::new("sarif-output", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "sarif", &three_finding_json_lines(), 0);

    assert_eq!(output.status.code(), Some(1), "findings should exit 1: stderr={}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("parse SARIF JSON");

    assert_eq!(sarif["$schema"].as_str().unwrap(), "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json");
    assert_eq!(sarif["version"], "2.1.0");

    let run = &sarif["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "cargo-tigerstyle");
    assert!(run["tool"]["driver"]["version"].as_str().is_some(), "missing driver version");

    let rules = run["tool"]["driver"]["rules"].as_array().expect("rules array");
    assert_eq!(rules.len(), 3, "expected 3 rules for 3 distinct lints");

    let results = run["results"].as_array().expect("results array");
    assert_eq!(results.len(), 3, "expected 3 results");

    for (i, r) in results.iter().enumerate() {
        assert!(r["ruleId"].as_str().is_some(), "result {i} missing ruleId");
        assert!(r["level"].as_str().is_some(), "result {i} missing level");
        assert!(r["message"]["text"].as_str().is_some(), "result {i} missing message text");
        assert!(
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"].as_str().is_some(),
            "result {i} missing location uri"
        );
        assert!(
            r["properties"]["findingIndex"].as_str().is_some(),
            "result {i} missing findingIndex property"
        );
    }

    assert_eq!(results[0]["properties"]["findingIndex"], "F1");
    assert_eq!(results[2]["properties"]["findingIndex"], "F3");
}

#[test]
fn summary_format_for_noisy_run() {
    let fixture = ConsumerFixture::new("summary-noisy", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "json", &three_finding_json_lines(), 0);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("--- tigerstyle summary ---"), "missing summary header: {stderr}");
    assert!(stderr.contains("Findings: 3"), "missing total count: {stderr}");
    assert!(stderr.contains("By crate:"), "missing by-crate section: {stderr}");
    assert!(stderr.contains("aspen-core"), "missing aspen-core in summary: {stderr}");
    assert!(stderr.contains("aspen-dns"), "missing aspen-dns in summary: {stderr}");
    assert!(stderr.contains("By lint:"), "missing by-lint section: {stderr}");
    assert!(stderr.contains("no_unwrap"), "missing no_unwrap in summary: {stderr}");
    assert!(stderr.contains("Index:"), "missing index section: {stderr}");
    assert!(stderr.contains("F1"), "missing F1 in index: {stderr}");
    assert!(stderr.contains("F3"), "missing F3 in index: {stderr}");
    assert!(stderr.ends_with("---\n"), "missing summary closing delimiter: {stderr}");
}

#[test]
fn summary_for_clean_run() {
    let fixture = ConsumerFixture::new("summary-clean", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![cargo_build_finished(true)];
    let output = run_with_output_format(&fixture, "json", &lines, 0);

    assert!(output.status.success(), "clean run should exit 0: stderr={}", String::from_utf8_lossy(&output.stderr));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- tigerstyle summary ---"), "missing summary header: {stderr}");
    assert!(stderr.contains("Findings: 0"), "clean run should show Findings: 0: {stderr}");
    assert!(!stderr.contains("By crate:"), "clean run should not have by-crate section: {stderr}");
    assert!(!stderr.contains("By lint:"), "clean run should not have by-lint section: {stderr}");
}

#[test]
fn finding_indices_are_unique_and_monotonic() {
    let fixture = ConsumerFixture::new("finding-index", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "json", &three_finding_json_lines(), 0);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("parse JSON finding"))
        .collect();

    let indices: Vec<&str> = findings.iter().map(|f| f["index"].as_str().unwrap()).collect();
    assert_eq!(indices, vec!["F1", "F2", "F3"], "indices should be F1, F2, F3 in order");

    // Check uniqueness
    let mut unique = indices.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), indices.len(), "all indices must be unique");
}

#[test]
fn default_human_mode_preserves_diagnostics_on_stderr_with_summary() {
    let fixture = ConsumerFixture::new("default-human", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let json_lines = three_finding_json_lines();
    let stdout_file = write_json_stdout_file(&fixture, &json_lines);
    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_STDOUT_FILE", stdout_file.to_str().unwrap())],
        &[],  // no --output-format flag = human default
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(output.status.code(), Some(1), "findings should exit 1");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // (a) stderr has human-readable diagnostics
    assert!(stderr.contains("warning[no_unwrap]"), "human mode should show rendered diagnostics on stderr: {stderr}");

    // (b) stderr has trailing summary
    assert!(stderr.contains("--- tigerstyle summary ---"), "human mode should append summary: {stderr}");

    // (c) no structured output on stdout
    assert!(stdout.is_empty(), "human mode should produce no stdout output: {stdout}");
}

#[test]
fn invalid_output_format_rejected() {
    let fixture = ConsumerFixture::new("invalid-format", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = fixture.run(
        &["--output-format", "xml"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(output.status.code(), Some(2), "invalid format should exit 2: stderr={}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported output format"), "should report unsupported format: {stderr}");
}

#[test]
fn compilation_error_during_json_output_produces_valid_json() {
    let fixture = ConsumerFixture::new("error-json", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    // Simulate compilation error: cargo exits non-zero, with a non-lint compiler-message
    let lines = vec![
        cargo_compiler_message(
            "consumer", "no_unwrap",
            "use of `.unwrap()`",
            "src/lib.rs", 10, 5,
            None,
        ),
        cargo_build_finished(false),
    ];
    let output = run_with_output_format(&fixture, "json", &lines, 1);

    assert_eq!(output.status.code(), Some(1), "error path should exit 1");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // All stdout lines must be valid JSON
    for line in stdout.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "stdout line is not valid JSON during error path: {line}"
        );
    }

    // Summary still printed
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- tigerstyle summary ---"), "summary should print even during error: {stderr}");
}

#[test]
fn compilation_error_during_sarif_output_produces_valid_sarif() {
    let fixture = ConsumerFixture::new("error-sarif", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message(
            "consumer", "no_unwrap",
            "use of `.unwrap()`",
            "src/lib.rs", 10, 5,
            None,
        ),
        cargo_build_finished(false),
    ];
    let output = run_with_output_format(&fixture, "sarif", &lines, 1);

    assert_eq!(output.status.code(), Some(1), "error path should exit 1");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("SARIF must be valid JSON even on error path: {e}\nstdout: {stdout}"));
    assert_eq!(sarif["version"], "2.1.0", "SARIF version must be present on error path");
    assert!(sarif["runs"][0]["results"].as_array().is_some(), "SARIF results must be present on error path");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- tigerstyle summary ---"), "summary should print even during error: {stderr}");
}

#[test]
fn conflicting_message_format_in_passthrough_args_rejected() {
    let fixture = ConsumerFixture::new("msg-format-conflict", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = fixture.run(
        &["--", "--message-format=short"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(output.status.code(), Some(2), "conflicting --message-format should exit 2: stderr={}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--message-format"), "should mention --message-format in error: {stderr}");
}
