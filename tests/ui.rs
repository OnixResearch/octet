use std::collections::BTreeSet;
use std::collections::HashMap;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use octet_architecture_ir::ArchitectureCollectionLimits;
use octet_architecture_ir::CollectionStatus;
use octet_architecture_ir::CollectorCompilationUnit;
use octet_architecture_ir::CompilationUnit;
use octet_architecture_ir::CompilerArchitectureCollectorContext;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CompilerArchitectureShard;
use octet_architecture_ir::CompilerValueFlowSelection;
use octet_architecture_ir::SourceRoleScope;
use octet_architecture_ir::UnknownFactKind;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UiFixtureSet {
    all_fixtures: Vec<String>,
    expected_failures: Vec<String>,
}

const COMPILER_FIXTURE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const COMPILER_FIXTURE_MAX_FACTS: usize = 4_096;
const COMPILER_FIXTURE_MAX_SPANS: usize = 8_192;
const COMPILER_FIXTURE_MAX_BYTES: usize = 16 * 1_024 * 1_024;
const COMPILER_FIXTURE_MAX_UNITS: usize = 1;

fn fixture_set() -> UiFixtureSet {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/fixtures/ui_fixture_set.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read UI fixture-set artifact {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse UI fixture-set artifact {}: {error}", path.display()))
}

#[cfg(unix)]
fn create_library_alias(plain: &Path, dylint: &Path) {
    symlink(plain, dylint).expect("create dylint library alias");
}

#[cfg(not(unix))]
fn create_library_alias(plain: &Path, dylint: &Path) {
    fs::copy(plain, dylint).expect("copy dylint library alias");
}

fn temp_path(prefix: &str) -> PathBuf {
    // Tests run as parallel threads of one process, and two of them can read the same clock tick,
    // so a per-process counter keeps their scratch paths apart.
    static NEXT_TEMP_PATH: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_TEMP_PATH.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before unix epoch").as_nanos();
    // Fixture names such as `custom/custom` would otherwise nest the scratch dir one level down
    // and leave its parent behind.
    let prefix = prefix.replace(['/', '\\'], "-");
    env::temp_dir().join(format!("{prefix}-{}-{sequence}-{unique}", std::process::id()))
}

/// A scratch directory owned by one fixture run or test, removed when dropped, also while a
/// failed assertion unwinds, so repeated UI runs do not pile up build output in `TMPDIR`.
#[derive(Debug)]
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new(prefix: &str) -> Self {
        Self(temp_path(prefix))
    }
}

impl std::ops::Deref for ScratchDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        // Removal is best effort: a leftover directory must not mask the run's own result.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The UI driver binary. A driver this test builds lives in a scratch target removed with it.
struct UiDriver {
    path: PathBuf,
    _target: Option<ScratchDir>,
}

impl std::ops::Deref for UiDriver {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
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

fn cargo_command() -> Command {
    let mut command = Command::new("cargo");
    command.env("CARGO_BUILD_RUSTC_WRAPPER", "");
    command
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

#[derive(Debug)]
struct FixtureSupport {
    target_search_paths: Vec<PathBuf>,
    host_search_paths: Vec<PathBuf>,
    externs: Vec<(String, PathBuf)>,
    saw_fresh_artifact: bool,
    #[allow(dead_code)]
    owned_target: Option<ScratchDir>,
}

impl FixtureSupport {
    fn all_search_paths(&self) -> Vec<&Path> {
        let mut paths = Vec::with_capacity(self.target_search_paths.len() + self.host_search_paths.len());
        for path in self.target_search_paths.iter().chain(self.host_search_paths.iter()) {
            let candidate = path.as_path();
            if !paths.contains(&candidate) {
                paths.push(candidate);
            }
        }
        paths
    }
}

struct SupportBuildOptions<'a> {
    toolchain: &'a str,
    manifest_path: &'a Path,
    target_triple: Option<&'a str>,
    target_dir: &'a Path,
    build_dir: Option<&'a Path>,
    extra_cargo_args: &'a [&'a str],
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    resolve: Option<CargoResolve>,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    root: Option<String>,
    nodes: Vec<CargoResolveNode>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveNode {
    id: String,
    #[serde(default)]
    deps: Vec<CargoResolveDep>,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveDep {
    name: String,
    pkg: String,
    #[serde(default)]
    dep_kinds: Vec<CargoDepKind>,
}

#[derive(Debug, Deserialize)]
struct CargoDepKind {
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoArtifactMessage {
    package_id: String,
    target: CargoArtifactTarget,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    filenames: Vec<PathBuf>,
    #[serde(default)]
    fresh: bool,
    #[serde(default)]
    profile: CargoArtifactProfile,
}

#[derive(Debug, Deserialize)]
struct CargoArtifactTarget {
    #[serde(default)]
    kind: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CargoArtifactProfile {
    #[serde(default)]
    test: bool,
}

#[derive(Debug, Clone)]
struct DirectSupportDependency {
    extern_name: String,
    package_id: String,
    resolved_features: BTreeSet<String>,
}

fn fixture_support_manifest(fixture_path: &Path) -> Option<PathBuf> {
    let manifest_path = fixture_path.parent()?.join("Cargo.toml");
    manifest_path.exists().then_some(manifest_path)
}

fn fixture_has_dependencies(manifest_path: &Path) -> bool {
    let manifest = fs::read_to_string(manifest_path)
        .unwrap_or_else(|error| panic!("read fixture manifest {}: {error}", manifest_path.display()));
    let parsed = toml::from_str::<toml::Table>(&manifest)
        .unwrap_or_else(|error| panic!("parse fixture manifest {}: {error}", manifest_path.display()));

    parsed.get("dependencies").and_then(toml::Value::as_table).is_some_and(|deps| !deps.is_empty())
}

const RMETA_ARTIFACT_RANK: u8 = 0;
const RLIB_ARTIFACT_RANK: u8 = 1;
const DYNAMIC_LIBRARY_ARTIFACT_RANK: u8 = 2;

fn path_extension_matches(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|candidate| candidate.to_str())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
}

fn is_proc_macro_target(target: &CargoArtifactTarget) -> bool {
    target.kind.iter().any(|kind| kind == "proc-macro")
}

fn is_library_target(target: &CargoArtifactTarget) -> bool {
    target.kind.iter().any(|kind| matches!(kind.as_str(), "lib" | "rlib" | "dylib"))
}

fn is_linkable_dependency_artifact(artifact: &CargoArtifactMessage, root_package_id: &str) -> bool {
    artifact.package_id != root_package_id
        && !artifact.profile.test
        && !artifact.target.kind.iter().any(|kind| kind == "custom-build")
        && (is_library_target(&artifact.target) || is_proc_macro_target(&artifact.target))
}

fn select_artifact_filename(
    artifact: &CargoArtifactMessage,
    manifest_path: &Path,
    extern_name: &str,
    toolchain: &str,
) -> Result<PathBuf, String> {
    let dynamic_library_extension = std::env::consts::DLL_SUFFIX.trim_start_matches('.');
    let proc_macro = is_proc_macro_target(&artifact.target);
    let mut ranked = Vec::new();
    for path in &artifact.filenames {
        let rank = if proc_macro {
            if path_extension_matches(path, dynamic_library_extension) {
                DYNAMIC_LIBRARY_ARTIFACT_RANK
            } else {
                continue;
            }
        } else if path_extension_matches(path, "rmeta") {
            RMETA_ARTIFACT_RANK
        } else if path_extension_matches(path, "rlib") {
            RLIB_ARTIFACT_RANK
        } else if path_extension_matches(path, dynamic_library_extension) {
            DYNAMIC_LIBRARY_ARTIFACT_RANK
        } else {
            continue;
        };
        ranked.push((rank, path.clone()));
    }
    ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let Some((best_rank, first_path)) = ranked.first().cloned() else {
        return Err(format!(
            "no linkable artifact filename in Cargo compiler-artifact for extern `{extern_name}` (package `{}`) in fixture {} (toolchain `{toolchain}`): {:?}",
            artifact.package_id,
            manifest_path.display(),
            artifact.filenames
        ));
    };
    let tied: Vec<PathBuf> = ranked.into_iter().filter(|(rank, _)| *rank == best_rank).map(|(_, path)| path).collect();
    if tied.len() > 1 {
        return Err(format!(
            "ambiguous linkable artifact filenames in Cargo compiler-artifact for extern `{extern_name}` (package `{}`) in fixture {} (toolchain `{toolchain}`): {tied:?}",
            artifact.package_id,
            manifest_path.display(),
        ));
    }
    Ok(first_path)
}

fn path_contains_component(path: &Path, expected: &str) -> bool {
    path.components().any(|component| component.as_os_str() == expected)
}

fn push_unique_path(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.contains(&candidate) {
        paths.push(candidate);
    }
}

fn direct_support_dependencies(
    metadata: &CargoMetadata,
    manifest_path: &Path,
    toolchain: &str,
) -> Result<(String, Vec<DirectSupportDependency>), String> {
    let resolve = metadata.resolve.as_ref().ok_or_else(|| {
        format!(
            "cargo metadata missing resolve graph for fixture {} (toolchain `{toolchain}`)",
            manifest_path.display()
        )
    })?;
    let root_id = resolve.root.as_deref().ok_or_else(|| {
        format!(
            "cargo metadata missing root package for fixture {} (toolchain `{toolchain}`)",
            manifest_path.display()
        )
    })?;
    let nodes_by_id: HashMap<&str, &CargoResolveNode> =
        resolve.nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let root_node = nodes_by_id.get(root_id).copied().ok_or_else(|| {
        format!(
            "cargo metadata resolve graph missing root node `{root_id}` for fixture {} (toolchain `{toolchain}`)",
            manifest_path.display()
        )
    })?;

    let mut direct = Vec::new();
    for dep in &root_node.deps {
        let is_normal = dep.dep_kinds.is_empty() || dep.dep_kinds.iter().any(|kind| kind.kind.is_none());
        if !is_normal {
            continue;
        }
        let resolved_features = nodes_by_id
            .get(dep.pkg.as_str())
            .map(|node| node.features.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        direct.push(DirectSupportDependency {
            extern_name: dep.name.clone(),
            package_id: dep.pkg.clone(),
            resolved_features,
        });
    }
    direct.sort_by(|left, right| {
        left.extern_name.cmp(&right.extern_name).then_with(|| left.package_id.cmp(&right.package_id))
    });
    Ok((root_id.to_owned(), direct))
}

fn collect_dependency_artifacts(
    manifest_path: &Path,
    toolchain: &str,
    root_package_id: &str,
    cargo_stdout: &str,
) -> Result<(Vec<CargoArtifactMessage>, bool), String> {
    let mut artifacts = Vec::new();
    let mut saw_fresh_artifact = false;

    for line in cargo_stdout.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-artifact") {
            continue;
        }
        let artifact: CargoArtifactMessage = serde_json::from_value(value).map_err(|error| {
            format!(
                "parse Cargo compiler-artifact message for fixture {} (toolchain `{toolchain}`): {error}",
                manifest_path.display()
            )
        })?;
        if !is_linkable_dependency_artifact(&artifact, root_package_id) {
            continue;
        }
        if artifact.fresh {
            saw_fresh_artifact = true;
        }
        artifacts.push(artifact);
    }

    Ok((artifacts, saw_fresh_artifact))
}

fn collect_artifact_search_paths(
    artifacts: &[CargoArtifactMessage],
    target_triple: Option<&str>,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut target_search_paths = Vec::new();
    let mut host_search_paths = Vec::new();
    for artifact in artifacts {
        let proc_macro = is_proc_macro_target(&artifact.target);
        for filename in &artifact.filenames {
            let Some(parent) = filename.parent() else {
                continue;
            };
            let is_host_artifact =
                proc_macro || target_triple.is_some_and(|triple| !path_contains_component(filename, triple));
            if is_host_artifact {
                push_unique_path(&mut host_search_paths, parent.to_path_buf());
            } else {
                push_unique_path(&mut target_search_paths, parent.to_path_buf());
            }
        }
    }
    (target_search_paths, host_search_paths)
}

fn resolve_direct_dependency_artifact(
    dep: &DirectSupportDependency,
    artifacts: &[CargoArtifactMessage],
    manifest_path: &Path,
    toolchain: &str,
    target_triple: Option<&str>,
) -> Result<PathBuf, String> {
    let mut matches: Vec<&CargoArtifactMessage> =
        artifacts.iter().filter(|artifact| artifact.package_id == dep.package_id).collect();
    if matches.len() > 1
        && let Some(triple) = target_triple
    {
        let target_matches: Vec<&CargoArtifactMessage> = matches
            .iter()
            .copied()
            .filter(|artifact| {
                is_proc_macro_target(&artifact.target)
                    || artifact.filenames.iter().any(|filename| path_contains_component(filename, triple))
            })
            .collect();
        if !target_matches.is_empty() {
            matches = target_matches;
        }
    }
    if matches.len() > 1 {
        let exact_feature_matches: Vec<&CargoArtifactMessage> = matches
            .iter()
            .copied()
            .filter(|artifact| {
                let features: BTreeSet<String> = artifact.features.iter().cloned().collect();
                features == dep.resolved_features
            })
            .collect();
        if !exact_feature_matches.is_empty() {
            matches = exact_feature_matches;
        }
    }

    let mut selected_paths = Vec::new();
    for artifact in matches {
        let chosen = select_artifact_filename(artifact, manifest_path, &dep.extern_name, toolchain)?;
        push_unique_path(&mut selected_paths, chosen);
    }
    if selected_paths.is_empty() {
        return Err(format!(
            "missing Cargo compiler-artifact for extern `{}` (package `{}`) in fixture {} (toolchain `{toolchain}`)",
            dep.extern_name,
            dep.package_id,
            manifest_path.display()
        ));
    }
    if selected_paths.len() > 1 {
        return Err(format!(
            "ambiguous Cargo compiler-artifact candidates for extern `{}` (package `{}`) in fixture {} (toolchain `{toolchain}`): {selected_paths:?}",
            dep.extern_name,
            dep.package_id,
            manifest_path.display()
        ));
    }
    Ok(selected_paths.remove(0))
}

fn resolve_support_from_cargo_json(
    manifest_path: &Path,
    toolchain: &str,
    target_triple: Option<&str>,
    metadata: &CargoMetadata,
    cargo_stdout: &str,
) -> Result<FixtureSupport, String> {
    let (root_package_id, direct_deps) = direct_support_dependencies(metadata, manifest_path, toolchain)?;
    let (artifacts, saw_fresh_artifact) =
        collect_dependency_artifacts(manifest_path, toolchain, &root_package_id, cargo_stdout)?;
    let (target_search_paths, host_search_paths) = collect_artifact_search_paths(&artifacts, target_triple);

    let mut externs = Vec::with_capacity(direct_deps.len());
    for dep in &direct_deps {
        let chosen = resolve_direct_dependency_artifact(dep, &artifacts, manifest_path, toolchain, target_triple)?;
        externs.push((dep.extern_name.clone(), chosen));
    }

    Ok(FixtureSupport {
        target_search_paths,
        host_search_paths,
        externs,
        saw_fresh_artifact,
        owned_target: None,
    })
}

fn format_cargo_failure(stdout: &str, stderr: &str) -> String {
    let mut rendered = Vec::new();
    for line in stdout.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) == Some("compiler-message")
            && let Some(text) = value.pointer("/message/rendered").and_then(serde_json::Value::as_str)
        {
            rendered.push(text.to_owned());
        }
    }
    let mut details = String::new();
    if !rendered.is_empty() {
        details.push_str("\nrendered compiler messages:\n");
        details.push_str(&rendered.join(""));
    }
    if !stderr.trim().is_empty() {
        details.push_str("\nstderr:\n");
        details.push_str(stderr);
    }
    details
}

fn cargo_metadata_for_support(options: &SupportBuildOptions<'_>) -> Result<CargoMetadata, String> {
    let manifest_dir = options
        .manifest_path
        .parent()
        .ok_or_else(|| format!("fixture manifest {} has no parent directory", options.manifest_path.display()))?;
    let mut command = cargo_command();
    command
        .current_dir(manifest_dir)
        .env("RUSTUP_TOOLCHAIN", options.toolchain)
        .args(["metadata", "--format-version=1", "--manifest-path"])
        .arg(options.manifest_path);
    if let Some(triple) = options.target_triple {
        command.args(["--filter-platform", triple]);
    }
    let mut index = 0;
    while index < options.extra_cargo_args.len() {
        let arg = options.extra_cargo_args[index];
        if matches!(arg, "--features" | "-F" | "--config") {
            command.arg(arg);
            if let Some(value) = options.extra_cargo_args.get(index + 1) {
                command.arg(value);
            }
            index += 2;
            continue;
        }
        if matches!(arg, "--all-features" | "--no-default-features" | "--offline" | "--frozen")
            || arg.starts_with("--features=")
            || arg.starts_with("--config=")
            || arg.starts_with("-F")
        {
            command.arg(arg);
        }
        index += 1;
    }

    let output = command.output().map_err(|error| {
        format!(
            "spawn cargo metadata for fixture {} (toolchain `{}`): {error}",
            options.manifest_path.display(),
            options.toolchain
        )
    })?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed for fixture {} (toolchain `{}`):\nstdout:\n{}\nstderr:\n{}",
            options.manifest_path.display(),
            options.toolchain,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| {
        format!(
            "parse cargo metadata JSON for fixture {} (toolchain `{}`): {error}",
            options.manifest_path.display(),
            options.toolchain
        )
    })
}

fn build_fixture_support_with_options(options: &SupportBuildOptions<'_>) -> Result<FixtureSupport, String> {
    let manifest_dir = options
        .manifest_path
        .parent()
        .ok_or_else(|| format!("fixture manifest {} has no parent directory", options.manifest_path.display()))?;
    let metadata = cargo_metadata_for_support(options)?;

    let mut command = cargo_command();
    command
        .current_dir(manifest_dir)
        .env("CARGO_TARGET_DIR", options.target_dir)
        .env("RUSTUP_TOOLCHAIN", options.toolchain)
        .args(["check", "--message-format=json", "--manifest-path"])
        .arg(options.manifest_path);
    if let Some(build_dir) = options.build_dir {
        command.env("CARGO_BUILD_BUILD_DIR", build_dir);
    }
    if let Some(triple) = options.target_triple {
        command.args(["--target", triple]);
    }
    command.args(options.extra_cargo_args);

    let output = command.output().map_err(|error| {
        format!(
            "spawn cargo check for fixture {} (toolchain `{}`): {error}",
            options.manifest_path.display(),
            options.toolchain
        )
    })?;
    let stdout = String::from_utf8(output.stdout).map_err(|error| {
        format!(
            "non-utf8 cargo check stdout for fixture {} (toolchain `{}`): {error}",
            options.manifest_path.display(),
            options.toolchain
        )
    })?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!(
            "failed to build fixture support deps for {} (toolchain `{}`, status {}){}",
            options.manifest_path.display(),
            options.toolchain,
            output.status,
            format_cargo_failure(&stdout, &stderr)
        ));
    }

    let support = resolve_support_from_cargo_json(
        options.manifest_path,
        options.toolchain,
        options.target_triple,
        &metadata,
        &stdout,
    )?;
    for (extern_name, path) in &support.externs {
        if !path.exists() {
            return Err(format!(
                "Cargo reported missing extern artifact `{}` for `{extern_name}` in fixture {} (toolchain `{}`)",
                path.display(),
                options.manifest_path.display(),
                options.toolchain
            ));
        }
    }
    Ok(support)
}

fn build_fixture_support(toolchain: &str, manifest_path: &Path) -> FixtureSupport {
    let target_dir = ScratchDir::new("octet-ui-fixture-deps");
    let options = SupportBuildOptions {
        toolchain,
        manifest_path,
        target_triple: None,
        target_dir: &target_dir,
        build_dir: None,
        extra_cargo_args: &[],
    };
    let mut support = build_fixture_support_with_options(&options).unwrap_or_else(|error| panic!("{error}"));
    support.owned_target = Some(target_dir);
    support
}

fn fixture_support_for<'a>(
    toolchain: &str,
    fixture_path: &Path,
    support_cache: &'a mut HashMap<PathBuf, FixtureSupport>,
) -> Option<&'a FixtureSupport> {
    let manifest_path = fixture_support_manifest(fixture_path)?;
    if !fixture_has_dependencies(&manifest_path) {
        return None;
    }

    if !support_cache.contains_key(&manifest_path) {
        let support = build_fixture_support(toolchain, &manifest_path);
        support_cache.insert(manifest_path.clone(), support);
    }

    support_cache.get(&manifest_path)
}

/// The lint library under test, built and linked once per test process: the parallel tests share
/// the toolchain-suffixed alias Dylint loads instead of each removing and recreating it while
/// another test is loading it.
static LIBRARY: LazyLock<PathBuf> = LazyLock::new(|| build_library_once(&rustup_toolchain()));

fn build_library() -> PathBuf {
    LIBRARY.clone()
}

fn build_library_once(toolchain: &str) -> PathBuf {
    if let Some(path) = prebuilt_path("OCTET_UI_LIBRARY").or_else(|| prebuilt_path("TIGERSTYLE_UI_LIBRARY")) {
        assert!(path.exists(), "missing prebuilt UI library: {}", path.display());
        return path;
    }

    let status = cargo_command()
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--lib"])
        .status()
        .expect("build UI test library");
    assert!(status.success(), "cargo build --lib failed for UI tests");

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"), PathBuf::from);
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

fn build_driver(toolchain: &str) -> UiDriver {
    if let Some(path) = prebuilt_path("OCTET_UI_DRIVER").or_else(|| prebuilt_path("TIGERSTYLE_UI_DRIVER")) {
        assert!(path.exists(), "missing prebuilt UI driver: {}", path.display());
        return UiDriver { path, _target: None };
    }

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/ui-driver/Cargo.toml");
    let driver_target_dir = ScratchDir::new("octet-ui-driver-target");
    let status = cargo_command()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", &*driver_target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--quiet", "--locked", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("build UI driver");
    assert!(status.success(), "failed to build UI driver");

    let debug_dir = driver_target_dir.join("debug");
    let candidates = [
        debug_dir.join(format!("octet-ui-driver{}", std::env::consts::EXE_SUFFIX)),
        debug_dir.join(format!("octet_ui_driver{}", std::env::consts::EXE_SUFFIX)),
    ];
    let path = candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("UI driver binary missing under {}", debug_dir.display()));
    UiDriver {
        path,
        _target: Some(driver_target_dir),
    }
}

fn normalize_stderr(stderr: &str, fixture_path: &Path) -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_dir = manifest_dir.join("ui_tests");
    let relative = fixture_path.strip_prefix(&ui_dir).expect("fixture under ui_tests").to_string_lossy();
    stderr
        .replace("\r\n", "\n")
        .replace(fixture_path.to_string_lossy().as_ref(), &format!("$DIR/{relative}"))
        .replace(manifest_dir.to_string_lossy().as_ref(), "$ROOT")
}

fn extra_rustc_args(name: &str) -> &'static [&'static str] {
    match name {
        "no_panic"
        | "no_todo"
        | "no_unwrap"
        | "drop_must_use"
        | "fallible_int_fallback"
        | "magic_number"
        | "explicit_defaults/explicit_defaults"
        | "too_many_parameters"
        | "sentinel_fallback"
        | "raw_arithmetic_overflow"
        | "verified/verified_purity" => &["--cfg", "test"],
        "ambient_clock_test_harness"
        | "assertion_density"
        | "function_length_test_harness"
        | "no_unwrap_test_harness"
        | "tokio_async/tokio_entry_generated_code"
        | "tokio_async/verified_purity_harness"
        | "type_accumulation_within_limit" => &["--test"],
        "numeric_units_cli_deny_custom/numeric_units_cli_deny_custom" => &["-D", "numeric_units"],
        "nominal_inventory_cli_deny_custom/nominal_inventory_cli_deny_custom" => &["-D", "primitive_domain_alias"],
        "strict_profile_force_warn_custom/strict_profile_force_warn_custom" => {
            &["--force-warn", "no_unwrap", "--cap-lints", "warn"]
        }
        _ => &[],
    }
}

/// Strict profile selected for a fixture, as `cargo octet check --profile` would set it.
fn fixture_strict_profile(name: &str) -> Option<&'static str> {
    match name {
        "strict_profile_force_warn_custom/strict_profile_force_warn_custom"
        | "strict_profile_governance_custom/strict_profile_governance_accept"
        | "strict_profile_governance_custom/strict_profile_governance_reject"
        | "strict_profile_governance_custom/strict_profile_governance_single_report"
        | "strict_profile_governance_custom/strict_profile_warn_floor"
        | "macro_dependency_custom/macro_dependency_custom" => Some("robust"),
        "nominal_inventory_profile_custom/nominal_inventory_profile_custom" => Some("bounded-systems"),
        _ => None,
    }
}

fn bless_enabled() -> bool {
    env::var_os("BLESS").is_some()
}

fn run_fixture(
    driver: &Path,
    library: &Path,
    host: &str,
    toolchain: &str,
    name: &str,
    expect_success: bool,
    support_cache: &mut HashMap<PathBuf, FixtureSupport>,
) {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let fixture_path = ui_dir.join(format!("{name}.rs"));
    let expected_name = fixture_path.file_stem().expect("fixture file stem").to_string_lossy();
    let expected_path = ui_dir.join(format!("{expected_name}.stderr"));
    let scratch = ScratchDir::new(&format!("octet-ui-case-{name}"));
    let work_dir: &Path = &scratch;
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
        .env_remove(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV)
        .args(extra_rustc_args(name));

    // Keep legacy fixtures focused on their own lint; boolean-API cases
    // exercise both new rules without this fixture-local compiler override.
    if !name.starts_with("raw_bool_parameter") {
        command.args(["-A", "raw_bool_parameter", "-A", "raw_bool_parameter_suppression"]);
    }

    if let Some(profile) = fixture_strict_profile(name) {
        command.env(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV, profile);
    }

    if let Some(support) = fixture_support_for(toolchain, &fixture_path, support_cache) {
        for search_path in support.all_search_paths() {
            command.arg("-L").arg(format!("dependency={}", search_path.display()));
        }
        for (crate_name, artifact_path) in &support.externs {
            command.arg("--extern").arg(format!("{crate_name}={}", artifact_path.display()));
        }
    }

    let output = command
        .arg(&fixture_path)
        .arg("-L")
        .arg(work_dir)
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

    assert_eq!(
        output.status.success(),
        expect_success,
        "fixture {name} exit-status mismatch: expected success={expect_success}, got status={}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = normalize_stderr(&String::from_utf8(output.stderr).expect("utf8 stderr"), &fixture_path);
    if bless_enabled() {
        fs::write(&expected_path, actual)
            .unwrap_or_else(|error| panic!("write expected stderr {}: {error}", expected_path.display()));
        return;
    }

    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|error| panic!("read expected stderr {}: {error}", expected_path.display()));
    assert_eq!(expected.trim_end_matches('\n'), actual.trim_end_matches('\n'), "stderr mismatch for fixture {name}");
}

fn fixed_fixture_names() -> Vec<String> {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let mut fixtures = fs::read_dir(&ui_dir)
        .unwrap_or_else(|error| panic!("read fixed fixture dir {}: {error}", ui_dir.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("collect fixed fixture dir {}: {error}", ui_dir.display()))
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("fixed"))
                .then(|| path.file_stem().expect("fixed fixture file stem").to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    fixtures.sort();
    fixtures
}

fn cargo_fix_autofix_fixture_command(crate_dir: &Path, driver: &Path, dylint_libs: &str, toolchain: &str) -> Command {
    let mut command = cargo_command();
    command
        .current_dir(crate_dir)
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", dylint_libs)
        .env("DYLINT_NO_DEPS", "1")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("CARGO_TARGET_DIR", crate_dir.join("target"))
        .args([
            "fix",
            "--allow-dirty",
            "--allow-staged",
            "--allow-no-vcs",
            "--bin",
            "autofix_fixture",
        ]);
    command
}

fn run_autofix_fixture(driver: &Path, library: &Path, toolchain: &str, name: &str) {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let fixture_path = ui_dir.join(format!("{name}.rs"));
    let fixed_path = ui_dir.join(format!("{name}.fixed"));
    let expected = fs::read_to_string(&fixed_path)
        .unwrap_or_else(|error| panic!("read expected fixed output {}: {error}", fixed_path.display()));
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("read autofix fixture {}: {error}", fixture_path.display()));

    let library = if library.is_absolute() {
        library.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(library)
    };
    let dylint_libs = serde_json::to_string(&vec![library]).expect("encode DYLINT_LIBS for autofix fixture");

    let scratch = ScratchDir::new(&format!("octet-autofix-{name}"));
    let crate_dir: &Path = &scratch;
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create autofix src dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"autofix_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("write autofix Cargo.toml");
    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, source).expect("write autofix main.rs");

    let mut fix = cargo_fix_autofix_fixture_command(crate_dir, driver, &dylint_libs, toolchain);

    eprintln!("[autofix fixture] {name}");
    eprintln!(
        "[autofix command] (cd {} && cargo fix --allow-dirty --allow-staged --allow-no-vcs --bin autofix_fixture)",
        crate_dir.display()
    );
    let fix_output = fix.output().expect("run cargo fix for autofix fixture");
    let fix_stdout = String::from_utf8(fix_output.stdout).expect("utf8 cargo fix stdout");
    let fix_stderr = String::from_utf8(fix_output.stderr).expect("utf8 cargo fix stderr");
    if !fix_stdout.is_empty() {
        eprintln!("[autofix stdout]\n{fix_stdout}");
    }
    if !fix_stderr.is_empty() {
        eprintln!("[autofix stderr]\n{fix_stderr}");
    }
    assert!(
        fix_output.status.success(),
        "cargo fix failed for fixture {name}:\nstdout:\n{fix_stdout}\nstderr:\n{fix_stderr}"
    );

    let actual = fs::read_to_string(&main_rs).expect("read autofixed main.rs");
    assert_eq!(expected, actual, "autofix output mismatch for fixture {name}");

    eprintln!(
        "[autofix idempotence command] (cd {} && cargo fix --allow-dirty --allow-staged --allow-no-vcs --bin autofix_fixture)",
        crate_dir.display()
    );
    let mut second_fix = cargo_fix_autofix_fixture_command(crate_dir, driver, &dylint_libs, toolchain);
    let second_fix_output = second_fix.output().expect("rerun cargo fix for autofix fixture");
    let second_fix_stdout = String::from_utf8(second_fix_output.stdout).expect("utf8 second cargo fix stdout");
    let second_fix_stderr = String::from_utf8(second_fix_output.stderr).expect("utf8 second cargo fix stderr");
    if !second_fix_stdout.is_empty() {
        eprintln!("[autofix idempotence stdout]\n{second_fix_stdout}");
    }
    if !second_fix_stderr.is_empty() {
        eprintln!("[autofix idempotence stderr]\n{second_fix_stderr}");
    }
    assert!(
        second_fix_output.status.success(),
        "second cargo fix failed for fixture {name}:\nstdout:\n{second_fix_stdout}\nstderr:\n{second_fix_stderr}"
    );
    let second_actual = fs::read_to_string(&main_rs).expect("read twice-autofixed main.rs");
    assert_eq!(actual, second_actual, "autofix fixture {name} changed on second cargo fix run");

    eprintln!(
        "[autofix post-check command] (cd {} && cargo check --quiet --bin autofix_fixture)",
        crate_dir.display()
    );
    let mut check = cargo_command();
    check
        .current_dir(crate_dir)
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", &dylint_libs)
        .env("DYLINT_NO_DEPS", "1")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("CARGO_TARGET_DIR", crate_dir.join("target"))
        .args(["check", "--quiet", "--bin", "autofix_fixture"]);

    let check_output = check.output().expect("run cargo check for autofix fixture");
    let check_stdout = String::from_utf8(check_output.stdout).expect("utf8 cargo check stdout");
    let check_stderr = String::from_utf8(check_output.stderr).expect("utf8 cargo check stderr");
    if !check_stdout.is_empty() {
        eprintln!("[autofix post-check stdout]\n{check_stdout}");
    }
    if !check_stderr.is_empty() {
        eprintln!("[autofix post-check stderr]\n{check_stderr}");
    }
    assert!(
        check_output.status.success(),
        "cargo check failed for autofixed fixture {name}:\nstdout:\n{check_stdout}\nstderr:\n{check_stderr}"
    );
    let combined_check_output = format!("{check_stdout}{check_stderr}");
    assert!(
        !combined_check_output.contains("warning:"),
        "autofixed fixture {name} still emitted warnings:\n{combined_check_output}"
    );
}

fn compiler_fixture_limits() -> ArchitectureCollectionLimits {
    ArchitectureCollectionLimits {
        max_units: COMPILER_FIXTURE_MAX_UNITS,
        max_shards: COMPILER_FIXTURE_MAX_UNITS,
        max_modules: COMPILER_FIXTURE_MAX_FACTS,
        max_items: COMPILER_FIXTURE_MAX_FACTS,
        max_types: COMPILER_FIXTURE_MAX_FACTS,
        max_traits: COMPILER_FIXTURE_MAX_FACTS,
        max_implementations: COMPILER_FIXTURE_MAX_FACTS,
        max_signatures: COMPILER_FIXTURE_MAX_FACTS,
        max_calls: COMPILER_FIXTURE_MAX_FACTS,
        max_constructors: COMPILER_FIXTURE_MAX_FACTS,
        max_effects: COMPILER_FIXTURE_MAX_FACTS,
        max_expansions: COMPILER_FIXTURE_MAX_FACTS,
        max_configurations: COMPILER_FIXTURE_MAX_FACTS,
        max_type_shapes: COMPILER_FIXTURE_MAX_FACTS,
        max_function_safety_facts: COMPILER_FIXTURE_MAX_FACTS,
        max_value_provenance_facts: COMPILER_FIXTURE_MAX_FACTS,
        max_value_flow_bodies: COMPILER_FIXTURE_MAX_FACTS,
        max_value_flow_steps: COMPILER_FIXTURE_MAX_FACTS,
        max_value_flow_rounds: COMPILER_FIXTURE_MAX_FACTS,
        max_unknowns: COMPILER_FIXTURE_MAX_FACTS,
        max_spans: COMPILER_FIXTURE_MAX_SPANS,
        max_span_bytes: COMPILER_FIXTURE_MAX_BYTES,
        max_shard_bytes: COMPILER_FIXTURE_MAX_BYTES,
        max_artifact_bytes: COMPILER_FIXTURE_MAX_BYTES,
    }
}

fn run_compiler_architecture_fixture(
    driver: &Path,
    library: &Path,
    host: &str,
    fixture_name: &str,
) -> CompilerArchitectureShard {
    run_compiler_architecture_fixture_with_selection(driver, library, host, fixture_name, None, Vec::new())
}

fn run_compiler_architecture_fixture_with_selection(
    driver: &Path,
    library: &Path,
    host: &str,
    fixture_name: &str,
    package_name: Option<&str>,
    value_flow_selections: Vec<CompilerValueFlowSelection>,
) -> CompilerArchitectureShard {
    run_compiler_architecture_target(
        driver,
        library,
        host,
        CompilerFixtureTarget {
            fixture_name,
            package_name,
            crate_type: "rlib",
            test_configuration: false,
            features: &[],
        },
        value_flow_selections,
    )
}

#[derive(Clone, Copy)]
struct CompilerFixtureTarget<'a> {
    fixture_name: &'a str,
    /// Cargo package name when it differs from the rustc crate name.
    package_name: Option<&'a str>,
    crate_type: &'a str,
    test_configuration: bool,
    features: &'a [&'a str],
}

fn compiler_fixture_unit(
    crate_name: &str,
    source_path: String,
    host: &str,
    target: CompilerFixtureTarget<'_>,
) -> CompilationUnit {
    let mut unit = CompilationUnit {
        unit_id: String::new(),
        package_name: target.package_name.unwrap_or(crate_name).to_owned(),
        package_version: "0.1.0".to_owned(),
        target_name: crate_name.to_owned(),
        target_kind: if target.test_configuration {
            format!("{}+test", target.crate_type)
        } else {
            target.crate_type.to_owned()
        },
        source_path,
        crate_types: vec![
            if target.test_configuration {
                "bin"
            } else {
                target.crate_type
            }
            .to_owned(),
        ],
        target_triple: host.to_owned(),
        features: target.features.iter().map(|feature| (*feature).to_owned()).collect(),
        default_features: true,
        role: CompilerArchitectureRole::DomainCore,
        toolchain_blake3: COMPILER_FIXTURE_HASH.to_owned(),
        collector_blake3: COMPILER_FIXTURE_HASH.to_owned(),
        policy_blake3: COMPILER_FIXTURE_HASH.to_owned(),
        source_blake3: COMPILER_FIXTURE_HASH.to_owned(),
        invocation_blake3: COMPILER_FIXTURE_HASH.to_owned(),
    };
    unit.unit_id = octet_architecture_ir::compilation_unit_identity(&unit);
    unit
}

fn run_compiler_architecture_target(
    driver: &Path,
    library: &Path,
    host: &str,
    target: CompilerFixtureTarget<'_>,
    value_flow_selections: Vec<CompilerValueFlowSelection>,
) -> CompilerArchitectureShard {
    run_compiler_architecture_target_with_limits(
        driver,
        library,
        host,
        target,
        value_flow_selections,
        compiler_fixture_limits(),
    )
}

fn run_compiler_architecture_target_with_limits(
    driver: &Path,
    library: &Path,
    host: &str,
    target: CompilerFixtureTarget<'_>,
    value_flow_selections: Vec<CompilerValueFlowSelection>,
    limits: ArchitectureCollectionLimits,
) -> CompilerArchitectureShard {
    let fixture_name = target.fixture_name;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_scope = "fixtures/compiler-architecture-ir";
    let fixture_path = root.join(source_scope).join(fixture_name);
    let work_dir = ScratchDir::new("octet-compiler-architecture-fixture");
    let shard_dir = work_dir.join("shards");
    fs::create_dir_all(&shard_dir).expect("create compiler architecture fixture shard dir");
    let crate_name = fixture_path.file_stem().expect("fixture stem").to_string_lossy().replace('-', "_");
    let unit = compiler_fixture_unit(&crate_name, format!("{source_scope}/{fixture_name}"), host, target);
    let context = CompilerArchitectureCollectorContext {
        schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION.to_owned(),
        workspace_root: root.display().to_string(),
        output_directory: shard_dir.display().to_string(),
        limits,
        selected_units: vec![CollectorCompilationUnit {
            rustc_crate_name: crate_name.clone(),
            manifest_dir: String::new(),
            source_path: format!("{source_scope}/{fixture_name}"),
            test_configuration: target.test_configuration,
            source_roles: vec![SourceRoleScope {
                source_scope: source_scope.to_owned(),
                role: CompilerArchitectureRole::DomainCore,
            }],
            unit,
        }],
        value_flow_selections,
        non_claims: vec!["whole-program-correctness".to_owned()],
    };
    let context_path = work_dir.join("collector-context.json");
    fs::write(&context_path, serde_json::to_vec_pretty(&context).expect("serialize collector context"))
        .expect("write collector context");
    let dylint_libs = serde_json::to_string(&vec![library]).expect("encode compiler fixture DYLINT_LIBS");
    let mut command = Command::new(driver);
    if target.test_configuration {
        command.arg("--test");
    } else {
        command.arg("--crate-type").arg(target.crate_type);
    }
    for feature in target.features {
        command.arg("--cfg").arg(format!("feature=\"{feature}\""));
    }
    let output = command
        .current_dir(root)
        .env("DYLINT_LIBS", dylint_libs)
        .env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &context_path)
        .env("CARGO_CRATE_NAME", &crate_name)
        .env("CARGO_MANIFEST_DIR", root)
        .arg(&fixture_path)
        .arg("--crate-name")
        .arg(&crate_name)
        .args(["--edition=2024", "--emit=metadata", "-A", "unused"])
        .arg(format!("--target={host}"))
        .arg("-o")
        .arg(work_dir.join("fixture.rmeta"))
        .output()
        .expect("run compiler architecture fixture");
    assert!(
        output.status.success(),
        "compiler architecture fixture {fixture_name} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut shard_paths = fs::read_dir(&shard_dir)
        .expect("read compiler fixture shard dir")
        .map(|entry| entry.expect("compiler fixture shard entry").path())
        .collect::<Vec<_>>();
    shard_paths.sort();
    assert_eq!(shard_paths.len(), COMPILER_FIXTURE_MAX_UNITS);
    serde_json::from_slice::<CompilerArchitectureShard>(
        &fs::read(&shard_paths[0]).expect("read compiler fixture shard"),
    )
    .expect("parse compiler fixture shard")
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture_ir_fixtures -- --ignored --nocapture"]
// One end-to-end scenario shares a single expensive library and driver build across every
// fixture; splitting it would rebuild both per test without making any assertion clearer.
#[allow(clippy::too_many_lines)]
fn compiler_architecture_ir_fixtures() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);

    let positive = run_compiler_architecture_fixture(&driver, &library, &host, "positive/resolved_semantics.rs");
    assert_eq!(positive.collection_status, CollectionStatus::Complete);
    for expected in [
        "trait",
        "implementation",
        "signature",
        "call",
        "constructor",
        "expansion",
        "configuration",
    ] {
        let has_fact = positive.facts.iter().any(|fact| {
            matches!(
                (expected, fact),
                ("trait", CompilerArchitectureFact::Trait { .. })
                    | ("implementation", CompilerArchitectureFact::Implementation { .. })
                    | ("signature", CompilerArchitectureFact::Signature { .. })
                    | ("call", CompilerArchitectureFact::Call { .. })
                    | ("constructor", CompilerArchitectureFact::Constructor { .. })
                    | ("expansion", CompilerArchitectureFact::Expansion { .. })
                    | ("configuration", CompilerArchitectureFact::Configuration { .. })
            )
        });
        assert!(has_fact, "positive compiler fixture omitted {expected} fact");
    }

    let value_flow_selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec![
            "authorized_effect_provenance::publish".to_owned(),
            "authorized_effect_provenance::PublicationReceipt::from_observation".to_owned(),
        ],
        tracked_type_definitions: vec![
            "authorized_effect_provenance::ActorAuthority".to_owned(),
            "authorized_effect_provenance::AuthorizedPublication".to_owned(),
            "authorized_effect_provenance::PublicationPlanId".to_owned(),
            "authorized_effect_provenance::PublicationAuthority".to_owned(),
            "authorized_effect_provenance::PublicationObservation".to_owned(),
            "authorized_effect_provenance::PublicationReceipt".to_owned(),
        ],
        root_origin_definitions: vec![
            "authorized_effect_provenance::authorize_publication".to_owned(),
            "authorized_effect_provenance::PublicationObservation::success".to_owned(),
        ],
        propagation_definitions: vec![
            "authorized_effect_provenance::PublicationAuthority::from_decision".to_owned(),
            "authorized_effect_provenance::PublicationReceipt::from_observation".to_owned(),
        ],
    };
    let authorized = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "positive/authorized_effect_provenance.rs",
        Some("authorized-effect-provenance"),
        vec![value_flow_selection],
    );
    assert!(
        authorized.facts.iter().any(|fact| matches!(
            fact,
            CompilerArchitectureFact::Type { definition_path: Some(definition), origin_crate, .. }
                if definition == "authorized_effect_provenance::PublicationAuthority"
                    && origin_crate == "authorized-effect-provenance"
        )),
        "authorized compiler facts: {:#?}",
        authorized.facts
    );
    assert!(
        authorized.facts.iter().any(|fact| matches!(
            fact,
            CompilerArchitectureFact::TypeShape { definition_path, field_visibilities, .. }
                if definition_path == "authorized_effect_provenance::PublicationAuthority"
                    && field_visibilities == &["restricted:authorized_effect_provenance"]
        )),
        "authorized compiler facts: {:#?}",
        authorized.facts
    );
    assert!(authorized.facts.iter().any(|fact| matches!(
        fact,
        CompilerArchitectureFact::FunctionSafety { qualified_path, is_unsafe: false, .. }
            if qualified_path == "authorized_effect_provenance::PublicationAuthority::from_decision"
    )));
    assert!(authorized.facts.iter().any(|fact| matches!(
        fact,
        CompilerArchitectureFact::CallArgumentProvenance {
            callee_definition,
            origin_definitions,
            status: octet_architecture_ir::ValueProvenanceStatus::Exact,
            ..
        } if callee_definition == "authorized_effect_provenance::publish"
            && origin_definitions == &["authorized_effect_provenance::authorize_publication"]
    )));

    let ambiguous_selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec!["ambiguous_effect_provenance::publish".to_owned()],
        tracked_type_definitions: vec![
            "ambiguous_effect_provenance::ActorAuthority".to_owned(),
            "ambiguous_effect_provenance::PublicationAuthority".to_owned(),
        ],
        root_origin_definitions: vec![
            "ambiguous_effect_provenance::authorize_publication".to_owned(),
            "ambiguous_effect_provenance::forge_publication".to_owned(),
        ],
        propagation_definitions: Vec::new(),
    };
    let ambiguous = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "negative/ambiguous_effect_provenance.rs",
        None,
        vec![ambiguous_selection],
    );
    assert!(ambiguous.facts.iter().any(|fact| matches!(
        fact,
        CompilerArchitectureFact::CallArgumentProvenance {
            callee_definition,
            status: octet_architecture_ir::ValueProvenanceStatus::Merged,
            ..
        } if callee_definition == "ambiguous_effect_provenance::publish"
    )));
    assert!(
        ambiguous
            .unknowns
            .iter()
            .any(|unknown| { unknown.kind == UnknownFactKind::AmbiguousValueProvenance })
    );

    let negative =
        run_compiler_architecture_fixture(&driver, &library, &host, "negative/source_text_and_dynamic_dispatch.rs");
    assert_eq!(negative.collection_status, CollectionStatus::Complete);
    assert!(negative.unknowns.iter().any(|unknown| unknown.kind == UnknownFactKind::DynamicDispatch));
    assert!(!negative.facts.iter().any(|fact| {
        matches!(
            fact,
            CompilerArchitectureFact::Trait { qualified_path, .. }
                | CompilerArchitectureFact::Item { qualified_path, .. }
                if qualified_path.contains("FakePort")
        )
    }));
}

fn assert_valid_compiler_shard(shard: &CompilerArchitectureShard) {
    let issues = octet_architecture_ir::verify_observation_set(
        &shard.facts,
        &shard.unknowns,
        &shard.spans,
        &compiler_fixture_limits(),
    );
    assert!(issues.is_empty(), "compiler fixture shard is invalid: {issues:#?}");
}

fn compiler_type_definition<'a>(shard: &'a CompilerArchitectureShard, type_id: &str) -> Option<&'a str> {
    shard.facts.iter().find_map(|fact| match fact {
        CompilerArchitectureFact::Type {
            type_id: current,
            definition_path,
            ..
        } if current == type_id => definition_path.as_deref(),
        _ => None,
    })
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_struct_literal_construction() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture(&driver, &library, &host, "positive/struct_literal_construction.rs");
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    let constructed = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Constructor {
                constructed_type_id,
                constructor_definition: Some(_),
                ..
            } => compiler_type_definition(&shard, constructed_type_id),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    for expected in [
        "struct_literal_construction::adapters::Http",
        "struct_literal_construction::adapters::SystemClock",
        "struct_literal_construction::adapters::Transport",
    ] {
        assert!(constructed.contains(expected), "missing constructor for {expected}: {constructed:?}");
    }
    // A fallible, wrapped, or awaited constructor selects the adapter inside its result once per
    // written call; the `?` and `.await` desugarings add no second construction.
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/compiler-architecture-ir/positive/struct_literal_construction.rs"),
    )
    .expect("construction fixture source");
    let counts = [
        "adapters::SqlRepo::connect(url)?",
        "adapters::SqlRepo::open(url)",
        "adapters::shared(url)",
        "adapters::SqlRepo::connect_async(url).await?",
    ]
    .map(|call| {
        let line = source.lines().position(|current| current.contains(call)).expect("fixture call line") + 1;
        (call, compiler_constructions_on_line(&shard, "struct_literal_construction::adapters::SqlRepo", line))
    });
    assert!(
        counts.iter().all(|(_, constructions)| *constructions == 1),
        "each written call must construct its wrapped adapter exactly once: {counts:?}"
    );
}

fn compiler_constructions_on_line(shard: &CompilerArchitectureShard, definition: &str, line: usize) -> usize {
    shard
        .facts
        .iter()
        .filter(|fact| match fact {
            CompilerArchitectureFact::Constructor {
                constructed_type_id,
                span_id,
                ..
            } => {
                compiler_type_definition(shard, constructed_type_id) == Some(definition)
                    && shard.spans.iter().any(|span| {
                        span.span_id == *span_id && usize::try_from(span.line_start).is_ok_and(|start| start == line)
                    })
            }
            _ => false,
        })
        .count()
}

// r[verify octet.project_architecture_gate.role_graph]
#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_adapter_use_is_not_construction() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard =
        run_compiler_architecture_fixture(&driver, &library, &host, "positive/adapter_use_without_construction.rs");
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    let callers = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                ..
            } => Some((item_id.as_str(), qualified_path.as_str())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let constructions = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Constructor {
                caller_item_id,
                constructed_type_id,
                ..
            } => Some((
                callers.get(caller_item_id.as_str()).copied().unwrap_or("unknown-caller"),
                compiler_type_definition(&shard, constructed_type_id).unwrap_or("unknown-type"),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let adapter = "adapter_use_without_construction::adapters::SqlRepo";
    let built_by = |function: &str| {
        let caller = format!("adapter_use_without_construction::{function}");
        constructions
            .iter()
            .filter(|(path, definition)| path.starts_with(&caller) && *definition == adapter)
            .count()
    };
    // Borrowing, locking, pinning, cloning a shared handle, moving, popping, or iterating an
    // injected adapter returns an existing value; none of these selects a new adapter.
    for function in [
        "use_shared",
        "use_cloned",
        "use_cell",
        "use_pinned",
        "use_slice",
        "use_owned",
    ] {
        assert_eq!(built_by(function), 0, "{function} constructions: {constructions:?}");
    }
    // `SqlRepo::connect(url)` and `SqlRepo::default()` each construct the adapter once; the
    // `Box::new` around one of them only moves it.
    assert_eq!(built_by("construct_direct"), 2, "construct_direct constructions: {constructions:?}");
    // A type the callee did not receive is one it produced: `into()`, `parse()`, and a generic
    // `make::<SqlRepo>()` each construct the adapter.
    assert_eq!(built_by("construct_generic"), 3, "construct_generic constructions: {constructions:?}");
    // Defaulted allocator and hasher parameters are not constructed by the call.
    let defaults = constructions
        .iter()
        .filter(|(_, definition)| definition.ends_with("::Global") || definition.ends_with("::RandomState"))
        .collect::<Vec<_>>();
    assert!(defaults.is_empty(), "default type parameters recorded as constructions: {defaults:?}");
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_borrowed_witness_signatures_and_narrow_conversions() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec!["authorization_witness_shapes::publish".to_owned()],
        tracked_type_definitions: vec![
            "authorization_witness_shapes::ActorAuthority".to_owned(),
            "authorization_witness_shapes::PublicationAuthority".to_owned(),
        ],
        root_origin_definitions: vec!["authorization_witness_shapes::authorize_publication".to_owned()],
        propagation_definitions: Vec::new(),
    };
    let shard = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "positive/authorization_witness_shapes.rs",
        None,
        vec![selection],
    );
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    // Signature inputs name the referent definition, as protocol checks compare definitions, and
    // keep whether that input is borrowed so a borrowed witness stays distinct from a moved one.
    for (function, parameter, by_reference) in [
        ("authorization_witness_shapes::publish", "authorization_witness_shapes::PublicationAuthority", true),
        (
            "authorization_witness_shapes::publish_once",
            "authorization_witness_shapes::PublicationAuthority",
            false,
        ),
        (
            "authorization_witness_shapes::publish_mut",
            "authorization_witness_shapes::PublicationAuthority",
            true,
        ),
        ("authorization_witness_shapes::mint", "authorization_witness_shapes::AuthorizedPublication", true),
        (
            "authorization_witness_shapes::authorize_publication",
            "authorization_witness_shapes::ActorAuthority",
            true,
        ),
    ] {
        let item_id = shard
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id,
                    qualified_path,
                    ..
                } if qualified_path == function => Some(item_id),
                _ => None,
            })
            .expect("signature owner item");
        let inputs = shard
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Signature {
                    owner_item_id,
                    input_type_ids,
                    input_by_reference,
                    ..
                } if owner_item_id == item_id => Some(input_type_ids.iter().zip(input_by_reference)),
                _ => None,
            })
            .expect("signature fact")
            .map(|(type_id, borrowed)| (compiler_type_definition(&shard, type_id), *borrowed))
            .collect::<Vec<_>>();
        assert!(inputs.contains(&(Some(parameter), by_reference)), "{function} inputs: {inputs:?}");
    }
    // Implemented traits keep their type arguments so a narrow conversion stays distinguishable.
    let traits = shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::TypeShape {
                definition_path,
                implemented_traits,
                ..
            } if definition_path == "authorization_witness_shapes::PublicationAuthority" => Some(implemented_traits),
            _ => None,
        })
        .expect("witness type shape");
    assert_eq!(
        traits,
        &["std::convert::From<authorization_witness_shapes::AuthorizedPublication>"],
        "witness traits"
    );
}

fn closure_value_flow_selection() -> CompilerValueFlowSelection {
    CompilerValueFlowSelection {
        tracked_callee_definitions: vec!["closure_effect_provenance::publish".to_owned()],
        tracked_type_definitions: vec![
            "closure_effect_provenance::ActorAuthority".to_owned(),
            "closure_effect_provenance::PublicationAuthority".to_owned(),
        ],
        root_origin_definitions: vec!["closure_effect_provenance::authorize_publication".to_owned()],
        propagation_definitions: Vec::new(),
    }
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_closure_value_provenance() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "positive/closure_effect_provenance.rs",
        None,
        vec![closure_value_flow_selection()],
    );
    assert_valid_compiler_shard(&shard);
    let closure_item_ids = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                ..
            } if qualified_path.contains("::closure@") => Some(item_id.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        shard.facts.iter().any(|fact| matches!(
            fact,
            CompilerArchitectureFact::CallArgumentProvenance {
                caller_item_id,
                callee_definition,
                origin_definitions,
                status: octet_architecture_ir::ValueProvenanceStatus::Exact,
                ..
            } if callee_definition == "closure_effect_provenance::publish"
                && origin_definitions == &["closure_effect_provenance::authorize_publication"]
                && closure_item_ids.contains(&caller_item_id.as_str())
        )),
        "closure provenance facts: {:#?}",
        shard.facts
    );
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_escaped_value_provenance_stays_unknown() {
    use octet_architecture_ir::ValueProvenanceStatus;

    const PREFIX: &str = "escaped_value_provenance::";
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec![format!("{PREFIX}SuccessReceipt::new")],
        tracked_type_definitions: vec![format!("{PREFIX}Observation"), format!("{PREFIX}SuccessReceipt")],
        root_origin_definitions: vec![format!("{PREFIX}success"), format!("{PREFIX}failure")],
        propagation_definitions: Vec::new(),
    };
    let shard = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "negative/escaped_value_provenance.rs",
        None,
        vec![selection],
    );
    assert_valid_compiler_shard(&shard);
    let items = compiler_item_paths(&shard);
    let statuses = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::CallArgumentProvenance {
                caller_item_id,
                callee_definition,
                origin_definitions,
                status,
                ..
            } if *callee_definition == format!("{PREFIX}SuccessReceipt::new") => Some((
                items.get(caller_item_id.as_str()).copied().expect("provenance caller item"),
                (*status, origin_definitions.clone()),
            )),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let exact = |origin: &str| Some((ValueProvenanceStatus::Exact, vec![format!("{PREFIX}{origin}")]));
    // Only a private function used solely as a direct callee takes its origin from its callers.
    assert_eq!(statuses.get("escaped_value_provenance::finish_direct").cloned(), exact("success"));
    // A call after an untracked prefix still records its provenance.
    assert_eq!(statuses.get("escaped_value_provenance::after_untracked_call").cloned(), exact("failure"));
    for caller in [
        // Also called through a function pointer with a forged observation.
        "finish_escaped",
        // Callable from other crates.
        "finish_public",
        // Overwritten through a mutable borrow.
        "swapped",
        "written_through_reference",
        // Read from a field of a value that was only partly written.
        "field_write",
        // Replaced by the result of a call through a function pointer.
        "replaced_through_pointer",
    ] {
        let status = statuses.get(format!("{PREFIX}{caller}").as_str()).map(|(status, _)| *status);
        assert_eq!(status, Some(ValueProvenanceStatus::Unknown), "{caller}: {statuses:#?}");
    }
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_external_reference_provenance() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec!["external_reference_provenance::publish".to_owned()],
        tracked_type_definitions: vec!["std::path::PathBuf".to_owned()],
        root_origin_definitions: Vec::new(),
        propagation_definitions: Vec::new(),
    };
    let shard = run_compiler_architecture_fixture_with_selection(
        &driver,
        &library,
        &host,
        "positive/external_reference_provenance.rs",
        None,
        vec![selection],
    );
    assert_valid_compiler_shard(&shard);
    let argument_type_ids = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::CallArgumentProvenance {
                callee_definition,
                argument_type_id,
                ..
            } if callee_definition == "external_reference_provenance::publish" => Some(argument_type_id.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!argument_type_ids.is_empty(), "external reference provenance facts: {:#?}", shard.facts);
    for type_id in argument_type_ids {
        assert!(
            shard.facts.iter().any(|fact| matches!(
                fact,
                CompilerArchitectureFact::Type { type_id: current, definition_path: Some(definition), origin_crate, .. }
                    if current == type_id && definition == "std::path::PathBuf" && origin_crate == "std"
            )),
            "argument type {type_id} has no external Type fact: {:#?}",
            shard.facts
        );
    }
}

fn compiler_item_paths(shard: &CompilerArchitectureShard) -> HashMap<&str, &str> {
    shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                ..
            } => Some((item_id.as_str(), qualified_path.as_str())),
            _ => None,
        })
        .collect()
}

fn compiler_item_id<'a>(shard: &'a CompilerArchitectureShard, qualified_path: &str) -> &'a str {
    let mut matches = shard.facts.iter().filter_map(|fact| match fact {
        CompilerArchitectureFact::Item {
            item_id,
            qualified_path: current,
            ..
        } if current == qualified_path => Some(item_id.as_str()),
        _ => None,
    });
    let item_id = matches.next().unwrap_or_else(|| panic!("missing item {qualified_path}: {:#?}", shard.facts));
    assert!(matches.next().is_none(), "repeated item {qualified_path}");
    item_id
}

fn compiler_type_id<'a>(shard: &'a CompilerArchitectureShard, definition_path: &str) -> &'a str {
    shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                definition_path: Some(current),
                ..
            } if current == definition_path => Some(type_id.as_str()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type {definition_path}: {:#?}", shard.facts))
}

fn compiler_effects(
    shard: &CompilerArchitectureShard,
) -> std::collections::BTreeSet<(&str, octet_architecture_ir::CapabilityFamily, &str)> {
    let items = compiler_item_paths(shard);
    shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Effect {
                caller_item_id,
                capability_family,
                primitive_definition,
                ..
            } => Some((
                items.get(caller_item_id.as_str()).copied().expect("effect caller item"),
                *capability_family,
                primitive_definition.as_str(),
            )),
            _ => None,
        })
        .collect()
}

fn library_fixture(fixture_name: &str) -> CompilerFixtureTarget<'_> {
    CompilerFixtureTarget {
        fixture_name,
        package_name: None,
        crate_type: "rlib",
        test_configuration: false,
        features: &[],
    }
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_selects_units_by_rust_target_tuple() {
    // rustc names this target `riscv64-unknown-linux-gnu` to LLVM; Cargo and the
    // planner name it by its Rust tuple.
    const RUST_TARGET: &str = "riscv64gc-unknown-linux-gnu";
    let toolchain = rustup_toolchain();
    let library = build_library();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_target(
        &driver,
        &library,
        RUST_TARGET,
        library_fixture("targets/cross_target.rs"),
        Vec::new(),
    );
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_eq!(shard.unit.target_triple, RUST_TARGET);
    compiler_item_id(&shard, "cross_target::identity");
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_dynamic_dispatch_through_pointers_and_paths() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture(
        &driver,
        &library,
        &host,
        "negative/smart_pointer_and_qualified_dynamic_dispatch.rs",
    );
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    let items = compiler_item_paths(&shard);
    let unknown_owners = shard
        .unknowns
        .iter()
        .filter(|unknown| unknown.kind == UnknownFactKind::DynamicDispatch)
        .filter_map(|unknown| items.get(unknown.owner_item_id.as_deref()?).copied())
        .collect::<std::collections::BTreeSet<_>>();
    let dynamic_callers = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Call {
                caller_item_id,
                dispatch: octet_architecture_ir::DispatchKind::Dynamic,
                ..
            } => items.get(caller_item_id.as_str()).copied(),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    // Static receivers, including boxed concrete types, a generic parameter, and
    // `Clone` on a shared trait object, stay out of both sets.
    let expected = [
        "smart_pointer_and_qualified_dynamic_dispatch::emit_boxed_writer",
        "smart_pointer_and_qualified_dynamic_dispatch::invoke_boxed",
        "smart_pointer_and_qualified_dynamic_dispatch::invoke_counted",
        "smart_pointer_and_qualified_dynamic_dispatch::invoke_fully_qualified",
        "smart_pointer_and_qualified_dynamic_dispatch::invoke_qualified",
        "smart_pointer_and_qualified_dynamic_dispatch::invoke_shared",
        "smart_pointer_and_qualified_dynamic_dispatch::next_boxed_iterator",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unknown_owners, expected);
    assert_eq!(dynamic_callers, expected);
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_function_references_record_calls_and_effects() {
    use octet_architecture_ir::CapabilityFamily;

    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture(&driver, &library, &host, "negative/function_reference_effects.rs");
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    let effects = compiler_effects(&shard);
    for expected in [
        ("function_reference_effects::deadline_or_now", CapabilityFamily::Time, "std::time::Instant::now"),
        ("function_reference_effects::read_all", CapabilityFamily::Filesystem, "std::fs::read_to_string"),
    ] {
        assert!(effects.contains(&expected), "missing {expected:?}: {effects:#?}");
    }
    assert!(
        effects
            .iter()
            .any(|(caller, family, primitive)| caller.starts_with("function_reference_effects::closure@")
                && *family == CapabilityFamily::Time
                && *primitive == "std::time::Instant::now"),
        "closure call lost its effect: {effects:#?}"
    );
    assert!(!effects.iter().any(|(caller, ..)| *caller == "function_reference_effects::pure_reference"));
    let pure_reference = compiler_item_id(&shard, "function_reference_effects::pure_reference");
    let increment = compiler_item_id(&shard, "function_reference_effects::increment");
    assert!(
        shard.facts.iter().any(|fact| matches!(
            fact,
            CompilerArchitectureFact::Call { caller_item_id, callee_item_id: Some(callee_item_id), resolved_definition: Some(definition), .. }
                if caller_item_id == pure_reference && callee_item_id == increment && definition == "function_reference_effects::increment"
        )),
        "local function reference has no call fact: {:#?}",
        shard.facts
    );
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_observability_and_synchronization_effects() {
    use octet_architecture_ir::CapabilityFamily;

    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture(
        &driver,
        &library,
        &host,
        "negative/observability_and_synchronization_effects.rs",
    );
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    let mut families = std::collections::BTreeMap::<&str, std::collections::BTreeSet<CapabilityFamily>>::new();
    for (caller, family, _) in compiler_effects(&shard) {
        families
            .entry(caller.trim_start_matches("observability_and_synchronization_effects::"))
            .or_default()
            .insert(family);
    }
    // Shared ownership (`Arc`), in-memory formatting, construction, exclusive
    // access, and const-evaluated initializers are not effects. An implicit
    // (`CONFIG.port`) or explicit (`**guard`) overloaded deref is a call.
    let expected = [
        ("atomic", CapabilityFamily::Concurrency),
        ("channel", CapabilityFamily::Concurrency),
        ("guarded_increment", CapabilityFamily::Concurrency),
        ("lazy_port", CapabilityFamily::Concurrency),
        ("locked", CapabilityFamily::Concurrency),
        ("locked_static", CapabilityFamily::Concurrency),
        ("print_error", CapabilityFamily::Observability),
        ("print_value", CapabilityFamily::Observability),
        ("write_stdout", CapabilityFamily::Observability),
    ]
    .into_iter()
    .map(|(caller, family)| (caller, std::collections::BTreeSet::from([family])))
    .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(families, expected, "{:#?}", compiler_effects(&shard));
    let effects = compiler_effects(&shard);
    for expected in [
        ("lazy_port", "std::sync::LazyLock::deref"),
        ("guarded_increment", "std::sync::MutexGuard::deref"),
    ] {
        assert!(
            effects.iter().any(|(caller, family, primitive)| caller
                .strip_prefix("observability_and_synchronization_effects::")
                == Some(expected.0)
                && *family == CapabilityFamily::Concurrency
                && *primitive == expected.1),
            "missing {expected:?}: {effects:#?}"
        );
    }
    let add_ticks = compiler_item_id(&shard, "observability_and_synchronization_effects::add_ticks");
    assert!(
        shard.facts.iter().any(|fact| matches!(
            fact,
            CompilerArchitectureFact::Call { caller_item_id, resolved_definition: Some(definition), .. }
                if caller_item_id == add_ticks && definition == "std::ops::Add::add"
        )),
        "overloaded operator has no call fact: {:#?}",
        shard.facts
    );
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_value_types_under_effect_paths_are_effect_free() {
    use octet_architecture_ir::CapabilityFamily;

    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let shard = run_compiler_architecture_fixture(&driver, &library, &host, "negative/effect_free_value_types.rs");
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_valid_compiler_shard(&shard);
    // Address, ordering, thread-id, exit-status, metadata, duration, and path
    // values, and the span between two given times, compute without I/O.
    // Connecting, reading, measuring elapsed time, and querying the filesystem
    // through a `Path`, including one reached by deref from a `PathBuf`, do not.
    let effects = compiler_effects(&shard)
        .into_iter()
        .map(|(caller, family, primitive)| (caller.trim_start_matches("effect_free_value_types::"), family, primitive))
        .collect::<std::collections::BTreeSet<_>>();
    let expected = std::collections::BTreeSet::from([
        ("age", CapabilityFamily::Time, "std::time::SystemTime::elapsed"),
        ("connect", CapabilityFamily::Network, "std::net::TcpStream::connect"),
        ("listing", CapabilityFamily::Filesystem, "std::path::Path::read_dir"),
        ("present", CapabilityFamily::Filesystem, "std::path::Path::exists"),
        ("present_buffer", CapabilityFamily::Filesystem, "std::path::Path::is_file"),
        ("read", CapabilityFamily::Filesystem, "std::fs::read_to_string"),
        ("waited", CapabilityFamily::Time, "std::time::Instant::elapsed"),
    ]);
    assert_eq!(effects, expected);
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_value_flow_limits_mark_shard_incomplete() {
    const VALUE_FLOW_LIMIT_TARGET: &str = "compiler-selected-value-flow";
    type RestrictLimits = fn(&mut ArchitectureCollectionLimits);
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let run = |limits| {
        run_compiler_architecture_target_with_limits(
            &driver,
            &library,
            &host,
            library_fixture("positive/closure_effect_provenance.rs"),
            vec![closure_value_flow_selection()],
            limits,
        )
    };
    let limited = |shard: &CompilerArchitectureShard| {
        shard
            .unknowns
            .iter()
            .any(|unknown| unknown.target_identity.as_deref() == Some(VALUE_FLOW_LIMIT_TARGET))
    };
    let within = run(compiler_fixture_limits());
    assert_eq!(within.collection_status, CollectionStatus::Complete);
    assert!(!limited(&within));
    let bounds: [(&str, RestrictLimits); 3] = [
        ("bodies", |limits| limits.max_value_flow_bodies = 0),
        ("steps", |limits| limits.max_value_flow_steps = 1),
        ("rounds", |limits| limits.max_value_flow_rounds = 1),
    ];
    for (bound, restrict) in bounds {
        let mut limits = compiler_fixture_limits();
        restrict(&mut limits);
        let shard = run(limits);
        assert!(limited(&shard), "{bound}: {:#?}", shard.unknowns);
        assert_eq!(shard.collection_status, CollectionStatus::Incomplete, "{bound}");
    }
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_value_flow_skips_bodies_without_tracked_values() {
    use octet_architecture_ir::ValueProvenanceStatus;

    const PREFIX: &str = "untracked_value_flow_bodies::";
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let selection = CompilerValueFlowSelection {
        tracked_callee_definitions: vec![format!("{PREFIX}publish"), format!("{PREFIX}record")],
        tracked_type_definitions: vec![format!("{PREFIX}Authority"), format!("{PREFIX}Marker")],
        root_origin_definitions: vec![format!("{PREFIX}authorize")],
        propagation_definitions: Vec::new(),
    };
    // `checksum` alone takes thousands of transfer steps, and it is a sixth
    // body; the five bodies that hold or pass a tracked value fit both bounds.
    let mut limits = compiler_fixture_limits();
    limits.max_value_flow_steps = 256;
    limits.max_value_flow_bodies = 5;
    let shard = run_compiler_architecture_target_with_limits(
        &driver,
        &library,
        &host,
        library_fixture("positive/untracked_value_flow_bodies.rs"),
        vec![selection],
        limits,
    );
    assert_eq!(shard.collection_status, CollectionStatus::Complete, "{:#?}", shard.unknowns);
    assert_valid_compiler_shard(&shard);
    let items = compiler_item_paths(&shard);
    let statuses = shard
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::CallArgumentProvenance {
                caller_item_id,
                callee_definition,
                origin_definitions,
                status,
                ..
            } => Some((
                (
                    items.get(caller_item_id.as_str()).copied().expect("provenance caller item"),
                    callee_definition.as_str(),
                ),
                (*status, origin_definitions.clone()),
            )),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(
        statuses.get(&("untracked_value_flow_bodies::execute", "untracked_value_flow_bodies::publish")),
        Some(&(ValueProvenanceStatus::Exact, vec![format!("{PREFIX}authorize")]))
    );
    // A small body that builds a tracked value inline is still analysed and
    // reports the unapproved constructor as its origin.
    assert_eq!(
        statuses.get(&("untracked_value_flow_bodies::forged", "untracked_value_flow_bodies::record")),
        Some(&(ValueProvenanceStatus::Exact, vec![format!("{PREFIX}Marker")])),
        "{statuses:#?}"
    );
    assert_eq!(statuses.len(), 2, "{statuses:#?}");
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture -- --ignored --nocapture"]
fn compiler_architecture_library_and_binary_identities_differ() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    // One source compiled as a library and as a binary shares its crate name,
    // as `src/lib.rs` and `src/main.rs` do in one package.
    let compile = |crate_type, test_configuration| {
        run_compiler_architecture_target(
            &driver,
            &library,
            &host,
            CompilerFixtureTarget {
                crate_type,
                test_configuration,
                ..library_fixture("targets/shared_crate_name.rs")
            },
            Vec::new(),
        )
    };
    let lib = compile("rlib", false);
    let lib_test = compile("rlib", true);
    let bin = compile("bin", false);
    let bin_test = compile("bin", true);
    let run = "shared_crate_name::run";
    let settings = "shared_crate_name::Settings";
    assert_ne!(compiler_item_id(&lib, run), compiler_item_id(&bin, run));
    assert_ne!(compiler_type_id(&lib, settings), compiler_type_id(&bin, settings));
    // Test configurations of one Cargo target keep that target's identities.
    assert_eq!(compiler_item_id(&lib, run), compiler_item_id(&lib_test, run));
    assert_eq!(compiler_type_id(&lib, settings), compiler_type_id(&lib_test, settings));
    assert_eq!(compiler_item_id(&bin, run), compiler_item_id(&bin_test, run));
    assert_eq!(compiler_type_id(&bin, settings), compiler_type_id(&bin_test, settings));
}

#[test]
#[ignore = "run via cargo test --test ui compiler_architecture_target_fixtures -- --ignored --nocapture"]
fn compiler_architecture_target_fixtures() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let pure = run_compiler_architecture_target(
        &driver,
        &library,
        &host,
        CompilerFixtureTarget {
            fixture_name: "targets/conditional.rs",
            package_name: None,
            crate_type: "rlib",
            test_configuration: false,
            features: &[],
        },
        Vec::new(),
    );
    assert_eq!(pure.collection_status, CollectionStatus::Complete);
    assert!(!pure.facts.iter().any(|fact| matches!(fact, CompilerArchitectureFact::Effect { .. })));
    let test = run_compiler_architecture_target(
        &driver,
        &library,
        &host,
        CompilerFixtureTarget {
            fixture_name: "targets/conditional.rs",
            package_name: None,
            crate_type: "rlib",
            test_configuration: true,
            features: &[],
        },
        Vec::new(),
    );
    assert_eq!(test.collection_status, CollectionStatus::Complete);
    assert_ne!(pure.unit.unit_id, test.unit.unit_id);
    assert!(test.facts.iter().any(|fact| matches!(fact, CompilerArchitectureFact::Effect {
        capability_family: octet_architecture_ir::CapabilityFamily::Filesystem,
        ..
    })));
    for target in [
        CompilerFixtureTarget {
            fixture_name: "targets/conditional.rs",
            package_name: None,
            crate_type: "rlib",
            test_configuration: false,
            features: &["extra"],
        },
        CompilerFixtureTarget {
            fixture_name: "targets/binary.rs",
            package_name: None,
            crate_type: "bin",
            test_configuration: false,
            features: &[],
        },
        CompilerFixtureTarget {
            fixture_name: "targets/procedural.rs",
            package_name: None,
            crate_type: "proc-macro",
            test_configuration: false,
            features: &[],
        },
    ] {
        eprintln!("[compiler target fixture] {} {}", target.fixture_name, target.crate_type);
        let shard = run_compiler_architecture_target(&driver, &library, &host, target, Vec::new());
        assert_eq!(shard.collection_status, CollectionStatus::Complete);
        assert!(shard.facts.iter().any(|fact| matches!(fact, CompilerArchitectureFact::Effect { .. })));
    }
    let expected_units = vec![
        octet_architecture_ir::ExpectedCompilationUnit {
            unit: pure.unit.clone(),
        },
        octet_architecture_ir::ExpectedCompilationUnit {
            unit: test.unit.clone(),
        },
    ];
    let mut limits = compiler_fixture_limits();
    limits.max_units = expected_units.len();
    limits.max_shards = expected_units.len();
    let mut input = octet_architecture_ir::NormalizationInput {
        policy_blake3: COMPILER_FIXTURE_HASH.to_owned(),
        limits,
        expected_units,
        shards: vec![pure, test],
        invalid_shards: Vec::new(),
        non_claims: vec!["whole-program-correctness".to_owned()],
    };
    let complete = octet_architecture_ir::normalize_architecture_shards(input.clone()).expect("normalize both units");
    assert_eq!(complete.receipt.status, CollectionStatus::Complete, "{:?}", complete.issues);
    input.shards.pop();
    let missing = octet_architecture_ir::normalize_architecture_shards(input).expect("normalize missing test unit");
    assert_eq!(missing.receipt.status, CollectionStatus::Incomplete);
    assert!(!missing.coverage.missing_unit_ids.is_empty());
}

#[test]
#[ignore = "run via nix flake check or cargo test --test ui -- --ignored --nocapture"]
fn ui() {
    let fixture_set = fixture_set();
    let expected_failures = fixture_set.expected_failures.into_iter().collect::<std::collections::HashSet<_>>();
    let toolchain = rustup_toolchain();
    let library = build_library();
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let mut support_cache = HashMap::new();

    for fixture in fixture_set.all_fixtures {
        eprintln!("[ui fixture] {fixture}");
        let snapshot_stem = Path::new(&fixture).file_stem().expect("fixture file stem").to_string_lossy();
        eprintln!(
            "[ui snapshot] {}",
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("ui_tests")
                .join(format!("{snapshot_stem}.stderr"))
                .display()
        );
        run_fixture(
            &driver,
            &library,
            &host,
            &toolchain,
            &fixture,
            !expected_failures.contains(&fixture),
            &mut support_cache,
        );
    }
}

#[test]
#[ignore = "run via cargo test --test ui autofix_machine_applicable -- --ignored --nocapture"]
fn autofix_machine_applicable() {
    let toolchain = rustup_toolchain();
    let library = build_library();
    let driver = build_driver(&toolchain);

    for fixture in fixed_fixture_names() {
        run_autofix_fixture(&driver, &library, &toolchain, &fixture);
    }
}

fn create_offline_support_fixture(root: &Path) -> PathBuf {
    let manifest_path = root.join("Cargo.toml");
    let src_dir = root.join("src");
    let trans_dir = root.join("trans_lib/src");
    let actual_dir = root.join("actual_lib/src");
    let macro_dir = root.join("ui_test_macro/src");
    fs::create_dir_all(&src_dir).expect("create support src dir");
    fs::create_dir_all(&trans_dir).expect("create trans_lib src dir");
    fs::create_dir_all(&actual_dir).expect("create actual_lib src dir");
    fs::create_dir_all(&macro_dir).expect("create proc-macro src dir");

    fs::write(
        root.join("trans_lib/Cargo.toml"),
        "[package]\nname = \"trans_lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write trans_lib manifest");
    fs::write(trans_dir.join("lib.rs"), "pub struct TransitiveToken(pub u32);\n").expect("write trans_lib source");

    fs::write(
        root.join("actual_lib/Cargo.toml"),
        "[package]\nname = \"actual-lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[features]\nextra = []\n\n[dependencies]\ntrans_lib = { path = \"../trans_lib\" }\n",
    )
    .expect("write actual_lib manifest");
    fs::write(
        actual_dir.join("lib.rs"),
        "pub fn token() -> trans_lib::TransitiveToken { trans_lib::TransitiveToken(7) }\n\
         #[cfg(feature = \"extra\")]\n\
         pub fn extra_value() -> u32 { 42 }\n",
    )
    .expect("write actual_lib source");

    fs::write(
        root.join("ui_test_macro/Cargo.toml"),
        "[package]\nname = \"ui_test_macro\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nproc-macro = true\n",
    )
    .expect("write proc-macro manifest");
    fs::write(
        macro_dir.join("lib.rs"),
        "extern crate proc_macro;\n\
         #[proc_macro]\n\
         pub fn identity(input: proc_macro::TokenStream) -> proc_macro::TokenStream { input }\n",
    )
    .expect("write proc-macro source");

    fs::write(
        &manifest_path,
        "[package]\nname = \"ui-support-json-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
         [features]\nextra = [\"aliased_lib/extra\"]\n\n\
         [dependencies]\n\
         aliased_lib = { package = \"actual-lib\", path = \"actual_lib\" }\n\
         ui_test_macro = { path = \"ui_test_macro\" }\n\n\
         [workspace]\n",
    )
    .expect("write support root manifest");
    fs::write(src_dir.join("lib.rs"), "").expect("write support root lib.rs");
    manifest_path
}

fn assert_rustc_compiles_with_support(toolchain: &str, support: &FixtureSupport, source_path: &Path, out_path: &Path) {
    let mut rustc = Command::new("rustc");
    rustc
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["--edition=2024", "--crate-type=lib", "--emit=metadata", "-o"])
        .arg(out_path)
        .arg(source_path);
    for dir in support.all_search_paths() {
        rustc.arg("-L").arg(format!("dependency={}", dir.display()));
    }
    for (name, path) in &support.externs {
        rustc.arg("--extern").arg(format!("{name}={}", path.display()));
    }
    let output = rustc.output().expect("run rustc with resolved support externs");
    assert!(
        output.status.success(),
        "rustc failed with Cargo-JSON-resolved externs:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn ui_support_resolution_handles_renamed_transitive_procmacro_cold_warm_and_stale_artifacts() {
    let root = ScratchDir::new("octet-ui-support-json");
    let manifest_path = create_offline_support_fixture(&root);
    let toolchain = rustup_toolchain();
    let target_dir = ScratchDir::new("octet-ui-support-json-target");
    let build_dir = ScratchDir::new("octet-ui-support-json-build-dir");
    let cold_options = SupportBuildOptions {
        toolchain: &toolchain,
        manifest_path: &manifest_path,
        target_triple: None,
        target_dir: &target_dir,
        build_dir: Some(&build_dir),
        extra_cargo_args: &[],
    };
    let cold = build_fixture_support_with_options(&cold_options).expect("cold support build should resolve");
    assert!(!cold.saw_fresh_artifact, "cold build should compile fresh artifacts");
    assert!(!cold.host_search_paths.is_empty(), "proc-macro host search path must be recorded");
    assert!(!cold.target_search_paths.is_empty(), "target search path must be recorded");

    let extern_map: HashMap<&str, &Path> =
        cold.externs.iter().map(|(name, path)| (name.as_str(), path.as_path())).collect();
    let aliased_path = *extern_map.get("aliased_lib").expect("renamed dependency extern `aliased_lib`");
    let macro_path = *extern_map.get("ui_test_macro").expect("proc-macro extern `ui_test_macro`");
    assert!(!extern_map.contains_key("actual_lib"), "manifest alias must be used, not package name");

    let probe_rs = root.join("probe.rs");
    fs::write(
        &probe_rs,
        "pub fn check() -> u32 {\nlet token = aliased_lib::token();\nui_test_macro::identity!(token.0)\n}\n",
    )
    .expect("write probe.rs");
    assert_rustc_compiles_with_support(&toolchain, &cold, &probe_rs, &root.join("libprobe.rmeta"));

    // Plant lexicographically-earlier stale artifacts beside the real ones; warm build must ignore
    // them.
    let aliased_dir = aliased_path.parent().expect("aliased artifact parent");
    fs::write(aliased_dir.join("libaliased_lib-0000000000000000.rmeta"), b"stale").expect("write stale aliased rmeta");
    fs::write(aliased_dir.join("libactual_lib-0000000000000000.rmeta"), b"stale").expect("write stale actual rmeta");
    let stale_macro = macro_path.parent().expect("macro artifact parent").join(format!(
        "{}ui_test_macro-0000000000000000{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    fs::write(&stale_macro, b"stale").expect("write stale macro dylib");

    let warm = build_fixture_support_with_options(&cold_options).expect("warm support build should resolve");
    assert!(warm.saw_fresh_artifact, "warm build should observe Cargo fresh=true compiler-artifact records");
    assert_eq!(warm.externs, cold.externs, "warm build must select exact Cargo-reported artifacts, not stale files");

    let extra_options = SupportBuildOptions {
        toolchain: &toolchain,
        manifest_path: &manifest_path,
        target_triple: None,
        target_dir: &target_dir,
        build_dir: Some(&build_dir),
        extra_cargo_args: &["--features", "extra"],
    };
    let extra = build_fixture_support_with_options(&extra_options).expect("extra feature build should resolve");
    let extra_aliased = extra
        .externs
        .iter()
        .find(|(name, _)| name == "aliased_lib")
        .map(|(_, path)| path.as_path())
        .expect("extra aliased_lib");
    assert_ne!(extra_aliased, aliased_path, "distinct feature variant must resolve its own exact artifact");

    let extra_probe_rs = root.join("probe_extra.rs");
    fs::write(&extra_probe_rs, "pub fn check() -> u32 { aliased_lib::extra_value() }\n").expect("write probe_extra.rs");
    assert_rustc_compiles_with_support(&toolchain, &extra, &extra_probe_rs, &root.join("libprobe_extra.rmeta"));

    let warm_default_again =
        build_fixture_support_with_options(&cold_options).expect("default warm rerun should remain exact");
    assert_eq!(warm_default_again.externs, cold.externs);
}

fn synthetic_support_metadata() -> CargoMetadata {
    serde_json::from_value(serde_json::json!({
        "resolve": {
            "root": "path+file:///virtual/ui_tests/sample#sample@0.1.0",
            "nodes": [
                {
                    "id": "path+file:///virtual/ui_tests/sample#sample@0.1.0",
                    "deps": [
                        {
                            "name": "renamed_dep",
                            "pkg": "registry+https://example#actual-dep@1.0.0",
                            "dep_kinds": [{ "kind": null }]
                        },
                        {
                            "name": "host_macro",
                            "pkg": "registry+https://example#host-macro@1.0.0",
                            "dep_kinds": [{ "kind": null }]
                        },
                        {
                            "name": "dev_only",
                            "pkg": "registry+https://example#dev-only@1.0.0",
                            "dep_kinds": [{ "kind": "dev" }]
                        }
                    ],
                    "features": []
                },
                {
                    "id": "registry+https://example#actual-dep@1.0.0",
                    "deps": [],
                    "features": ["std"]
                },
                {
                    "id": "registry+https://example#host-macro@1.0.0",
                    "deps": [],
                    "features": []
                }
            ]
        }
    }))
    .expect("metadata fixture")
}

fn synthetic_artifact_line(package_id: &str, kind: &str, features: &[&str], filename: &str, fresh: bool) -> String {
    serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": package_id,
        "target": { "kind": [kind] },
        "features": features,
        "filenames": [filename],
        "fresh": fresh,
        "profile": { "test": false }
    })
    .to_string()
}

#[test]
fn ui_support_resolution_handles_custom_build_dir_cross_target_and_rejects_missing_or_ambiguous_messages() {
    let manifest_path = Path::new("/virtual/ui_tests/sample/Cargo.toml");
    let toolchain = "nightly-2026-03-21";
    let dylib_ext = std::env::consts::DLL_SUFFIX;
    let metadata = synthetic_support_metadata();

    let macro_file = format!("/custom/build-dir/host-macros/out/libhost_macro{dylib_ext}");
    let stdout = [
        synthetic_artifact_line(
            "registry+https://example#actual-dep@1.0.0",
            "lib",
            &[],
            "/custom/build-dir/host-unit/out/libactual_dep.rmeta",
            false,
        ),
        synthetic_artifact_line(
            "registry+https://example#actual-dep@1.0.0",
            "lib",
            &["std"],
            "/custom/build-dir/aarch64-unknown-linux-gnu/unit-1/out/libactual_dep.rmeta",
            true,
        ),
        synthetic_artifact_line("registry+https://example#host-macro@1.0.0", "proc-macro", &[], &macro_file, false),
        synthetic_artifact_line(
            "registry+https://example#transitive@0.2.0",
            "lib",
            &[],
            "/custom/build-dir/aarch64-unknown-linux-gnu/unit-2/out/libtransitive.rmeta",
            false,
        ),
    ]
    .join("\n");

    let resolved = resolve_support_from_cargo_json(
        manifest_path,
        toolchain,
        Some("aarch64-unknown-linux-gnu"),
        &metadata,
        &stdout,
    )
    .expect("custom build-dir cross-target resolution should succeed");

    assert!(resolved.saw_fresh_artifact);
    assert_eq!(resolved.externs, vec![
        ("host_macro".to_owned(), PathBuf::from(&macro_file)),
        (
            "renamed_dep".to_owned(),
            PathBuf::from("/custom/build-dir/aarch64-unknown-linux-gnu/unit-1/out/libactual_dep.rmeta")
        ),
    ]);
    assert_eq!(resolved.target_search_paths, vec![
        PathBuf::from("/custom/build-dir/aarch64-unknown-linux-gnu/unit-1/out"),
        PathBuf::from("/custom/build-dir/aarch64-unknown-linux-gnu/unit-2/out"),
    ]);
    assert_eq!(resolved.host_search_paths, vec![
        PathBuf::from("/custom/build-dir/host-unit/out"),
        PathBuf::from("/custom/build-dir/host-macros/out"),
    ]);

    let missing_err = resolve_support_from_cargo_json(manifest_path, toolchain, None, &metadata, "")
        .expect_err("missing compiler-artifact must fail");
    assert!(missing_err.contains("host_macro") && missing_err.contains("host-macro@1.0.0"), "{missing_err}");
    assert!(missing_err.contains("sample/Cargo.toml") && missing_err.contains(toolchain), "{missing_err}");

    let ambiguous_stdout = [
        synthetic_artifact_line(
            "registry+https://example#actual-dep@1.0.0",
            "lib",
            &["std"],
            "/custom/build-dir/a/libactual_dep.rmeta",
            false,
        ),
        synthetic_artifact_line(
            "registry+https://example#actual-dep@1.0.0",
            "lib",
            &["std"],
            "/custom/build-dir/b/libactual_dep.rmeta",
            false,
        ),
        synthetic_artifact_line("registry+https://example#host-macro@1.0.0", "proc-macro", &[], &macro_file, false),
    ]
    .join("\n");
    let ambiguous_err = resolve_support_from_cargo_json(manifest_path, toolchain, None, &metadata, &ambiguous_stdout)
        .expect_err("ambiguous compiler-artifact must fail");
    assert!(ambiguous_err.contains("ambiguous") && ambiguous_err.contains("renamed_dep"), "{ambiguous_err}");
    assert!(ambiguous_err.contains("actual-dep@1.0.0") && ambiguous_err.contains(toolchain), "{ambiguous_err}");
}
