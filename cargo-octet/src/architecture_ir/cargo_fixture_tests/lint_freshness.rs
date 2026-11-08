use std::fs;
use std::path::Path;
use std::process::Command;

use super::TOOLCHAIN;
use super::required_artifact;

const STRICT_PROFILE: &str = octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV;
/// Environment variables that change lint levels inside rustc.
const LINT_LEVEL_ENV: &[&str] = &[STRICT_PROFILE];

struct LintCheck {
    succeeded: bool,
    /// `fresh` of the fixture library artifact; `None` when Cargo produced no artifact.
    fresh: Option<bool>,
    /// Levels of every `no_unwrap` diagnostic, replayed ones included.
    no_unwrap_levels: Vec<String>,
}

fn cargo_check(root: &Path, library: &Path, driver: &Path, lint_level_env: &[(&str, &str)]) -> LintCheck {
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .args(["check", "--message-format=json"])
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("library selection"))
        .env_remove(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV);
    for name in LINT_LEVEL_ENV {
        command.env_remove(name);
    }
    command.envs(lint_level_env.iter().copied());
    let output = command.output().expect("check fixture through the Dylint driver");
    let messages = String::from_utf8(output.stdout)
        .expect("Cargo JSON is UTF-8")
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect::<Vec<_>>();
    let fresh = messages
        .iter()
        .filter(|message| message["reason"] == "compiler-artifact")
        .find(|message| message["target"]["name"] == "lint_freshness_fixture")
        .map(|message| message["fresh"].as_bool().expect("artifact freshness"));
    let no_unwrap_levels = messages
        .iter()
        .filter(|message| message["reason"] == "compiler-message")
        .filter(|message| {
            // Diagnostics carry the tool namespace, e.g. `tigerstyle::no_unwrap`.
            message["message"]["code"]["code"].as_str().and_then(|code| code.rsplit("::").next()) == Some("no_unwrap")
        })
        .map(|message| message["message"]["level"].as_str().expect("diagnostic level").to_owned())
        .collect();
    LintCheck {
        succeeded: output.status.success(),
        fresh,
        no_unwrap_levels,
    }
}

fn assert_warns(check: &LintCheck, step: &str, fresh: bool) {
    assert!(check.succeeded, "{step}: warning-level run must succeed");
    assert_eq!(check.fresh, Some(fresh), "{step}: library freshness");
    assert_eq!(check.no_unwrap_levels, ["warning"], "{step}: no_unwrap stays a warning");
}

fn assert_denies(check: &LintCheck, step: &str) {
    assert!(!check.succeeded, "{step}: denied no_unwrap must fail Cargo instead of replaying cached warnings");
    assert_eq!(check.fresh, None, "{step}: a failed unit is never reported fresh");
    assert_eq!(check.no_unwrap_levels, ["error"], "{step}: no_unwrap is raised to an error");
}

// Cargo reuses a check unit unless an input it tracks changes. Strict-profile selection changes
// lint levels only inside rustc, so it must invalidate the unit.
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_lint_level_inputs_invalidate_cached_checks() {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let temporary = tempfile::TempDir::new().expect("fixture crate");
    let root = temporary.path();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"lint-freshness-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("fixture manifest");
    fs::create_dir(root.join("src")).expect("fixture source directory");
    fs::write(root.join("src/lib.rs"), "pub fn first(values: &[u32]) -> u32 {\n    *values.first().unwrap()\n}\n")
        .expect("fixture source");
    let check = |lint_level_env: &[(&str, &str)]| cargo_check(root, &library, &driver, lint_level_env);

    assert_warns(&check(&[]), "first run", false);
    assert_warns(&check(&[]), "unchanged rerun", true);

    assert_denies(&check(&[(STRICT_PROFILE, "robust")]), "robust profile after a cached default run");

    assert_warns(&check(&[]), "default run after the profile", false);
}
