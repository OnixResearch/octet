use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::config;
use crate::output;
use crate::output::Finding;

mod embedded_source {
    include!(concat!(env!("OUT_DIR"), "/embedded_source.rs"));
}

const SOURCE_HASH_MARKER: &str = ".tigerstyle-source-blake3";

pub fn run(check: CheckArgs) -> Result<i32, String> {
    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let cfg = config::load(&cwd)?;
    let scope_args = if check.scope_args.is_empty() {
        cfg.default_scope.clone()
    } else {
        check.scope_args
    };
    let cargo_check_args = check.cargo_check_args.unwrap_or(cfg.cargo_check_args.clone());

    // I7: detect --message-format conflict in passthrough args
    if cargo_check_args.iter().any(|arg| arg.starts_with("--message-format")) {
        return Err("`--message-format` is managed by `--output-format`; remove it from passthrough args".to_owned());
    }

    let lockfile_path = cfg.workspace_root.join("Cargo.lock");
    let skip_lockfile_guard =
        env::var("TIGERSTYLE_SKIP_LOCKFILE_CHECK").map(|v| !v.is_empty() && v != "0").unwrap_or(false);
    let lockfile_before = if skip_lockfile_guard {
        LockfileSnapshot::Missing
    } else {
        snapshot_lockfile(&lockfile_path)?
    };

    let result = (|| {
        let toolchain = resolve_toolchain()?;
        let driver = resolve_driver(&toolchain)?;
        let lint_lib = resolve_lint_lib(&toolchain)?;
        let dylint_libs = serde_json::to_string(&vec![lint_lib.path().to_owned()])
            .map_err(|error| format!("encode DYLINT_LIBS: {error}"))?;

        let mut cmd = Command::new("cargo");
        cmd.arg("check").arg("--message-format=json");

        // I9: inject --color=always when stderr is a TTY and human mode, unless overridden
        if check.output_format == OutputFormat::Human
            && std::io::stderr().is_terminal()
            && !cargo_check_args.iter().any(|a| a.starts_with("--color"))
        {
            cmd.arg("--color=always");
        }

        cmd.args(&scope_args)
            .args(&cargo_check_args)
            .current_dir(&cfg.workspace_root)
            .env("RUSTC_WORKSPACE_WRAPPER", driver.path())
            .env("DYLINT_LIBS", &dylint_libs)
            .env("DYLINT_NO_DEPS", "1")
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .stdout(Stdio::piped());

        // In json/sarif modes, suppress child stderr (cargo progress output)
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
            let line = line.map_err(|e| format!("read cargo stdout: {e}"))?;
            let json: serde_json::Value = serde_json::from_str(&line).map_err(|e| format!("parse cargo JSON: {e}"))?;

            let reason = json["reason"].as_str().unwrap_or("");

            match reason {
                "compiler-message" => {
                    let message = &json["message"];

                    // I8: in human mode, print rendered diagnostic to stderr
                    if check.output_format == OutputFormat::Human {
                        if let Some(rendered) = message["rendered"].as_str() {
                            let _ = write!(stderr_lock, "{rendered}");
                        }
                    }

                    // Check if this is a tigerstyle lint
                    let code = message["code"]["code"].as_str().unwrap_or("");
                    let lint_name = normalize_lint_name(code);

                    if let Some(meta) = meta_map.get(lint_name.as_str()) {
                        finding_counter += 1;
                        let finding = build_finding(finding_counter, &json, meta, &lint_name);

                        if check.output_format == OutputFormat::Json {
                            output::emit_json_line(&finding, &mut stdout_lock)?;
                        }

                        findings.push(finding);
                    }
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

        let status = child.wait().map_err(|error| format!("wait for cargo check: {error}"))?;

        // Emit SARIF after all findings collected
        if check.output_format == OutputFormat::Sarif {
            output::emit_sarif(&findings, &meta_map, &mut stdout_lock)?;
        }

        // I4: always emit summary to stderr
        output::emit_summary(&findings, &mut stderr_lock)?;

        // Exit code: 0 = clean, 1 = findings or cargo failure
        if !status.success() || !findings.is_empty() {
            if status.code().is_some() {
                return Ok(1);
            }
            return Err("cargo check terminated without an exit code".to_owned());
        }

        Ok(0)
    })();

    if !skip_lockfile_guard && lockfile_changed(&lockfile_path, &lockfile_before)? {
        restore_lockfile(&lockfile_path, &lockfile_before)?;
        let lockfile_error =
            format!("cargo tigerstyle check modified {}; restored the original lockfile", lockfile_path.display());
        return Err(match result {
            Err(error) => format!("{error}; {lockfile_error}"),
            Ok(_) => lockfile_error,
        });
    }

    result
}

fn normalize_lint_name(code: &str) -> String {
    code.strip_prefix("tigerstyle::").unwrap_or(code).to_owned()
}

fn build_finding(index: usize, cargo_msg: &serde_json::Value, meta: &output::LintMeta, lint_name: &str) -> Finding {
    let message = &cargo_msg["message"];

    let primary_span = message["spans"]
        .as_array()
        .and_then(|spans| spans.iter().find(|s| s["is_primary"].as_bool() == Some(true)))
        .or_else(|| message["spans"].as_array().and_then(|s| s.first()));

    let file = primary_span.and_then(|s| s["file_name"].as_str()).unwrap_or("unknown").to_owned();
    let line = primary_span.and_then(|s| s["line_start"].as_u64()).unwrap_or(0) as u32;
    let column = primary_span.and_then(|s| s["column_start"].as_u64()).unwrap_or(0) as u32;

    let msg_text = message["message"].as_str().unwrap_or("").to_owned();
    let severity = message["level"].as_str().unwrap_or("warning").to_owned();
    let crate_name = cargo_msg["target"]["name"].as_str().unwrap_or("unknown").to_owned();

    let (suggestion, applicability) = extract_suggestion(message);

    Finding {
        version: 1,
        index: format!("F{index}"),
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
        crate_name,
    }
}

fn extract_suggestion(message: &serde_json::Value) -> (Option<String>, Option<String>) {
    let children = match message["children"].as_array() {
        Some(c) => c,
        None => return (None, None),
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
        let path = env::temp_dir().join(format!("tigerstyle-{label}-{}-{nanos}", std::process::id()));
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
    let built_lib_name = format!("{}tigerstyle.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);
    let dylint_lib_name =
        format!("{}tigerstyle@{toolchain}.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .args(["--release", "--locked", "-p", "tigerstyle", "--lib"])
        .current_dir(&source_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .status()
        .map_err(|error| format!("run cargo build for tigerstyle lint library: {error}"))?;
    if !status.success() {
        return Err("tigerstyle lint library build failed".to_owned());
    }

    let built_lib_path = target_dir.join("release").join(&built_lib_name);
    if !built_lib_path.is_file() {
        return Err(format!("built tigerstyle lint library missing at {}", built_lib_path.display()));
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
    candidates.push(env::temp_dir().join("tigerstyle-cache"));

    let mut failures = Vec::new();
    for candidate in candidates {
        let sources_root = candidate.join("tigerstyle").join("sources");
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
