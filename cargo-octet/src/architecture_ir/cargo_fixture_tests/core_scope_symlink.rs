use std::fs;
use std::process::Command;

use super::TOOLCHAIN;
use super::required_artifact;

const STRICT_PROFILE: &str = octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV;

// `cargo octet check --profile deterministic-core` accepts an in-root symlinked
// core scope (`linked -> src`), while rustc names the compiled file `src/lib.rs`.
// The lint library must resolve the scope the same way, so core lints still
// report through it.
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_symlinked_core_scope_lints_its_target() {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let temporary = tempfile::TempDir::new().expect("fixture crate");
    let root = temporary.path();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"core-scope-symlink-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("fixture manifest");
    fs::create_dir(root.join("src")).expect("fixture source directory");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn load(path: &std::path::Path) -> std::io::Result<String> {\n    std::fs::read_to_string(path)\n}\n",
    )
    .expect("fixture source");
    std::os::unix::fs::symlink(root.join("src"), root.join("linked")).expect("in-root scope symlink");
    fs::write(root.join("dylint.toml"), "[octet]\ncore_source_scopes = [\"linked\"]\n").expect("fixture config");

    let output = Command::new("cargo")
        .current_dir(root)
        .args(["check", "--message-format=json"])
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("library selection"))
        .env_remove("DYLINT_RUSTFLAGS")
        .env_remove(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV)
        .env(STRICT_PROFILE, "deterministic-core")
        .output()
        .expect("check fixture through the Dylint driver");
    let messages = String::from_utf8(output.stdout)
        .expect("Cargo JSON is UTF-8")
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|message| message["reason"] == "compiler-message")
        .collect::<Vec<_>>();
    let impure_calls = messages
        .iter()
        .filter(|message| {
            message["message"]["code"]["code"].as_str().and_then(|code| code.rsplit("::").next())
                == Some("impure_call_in_core")
        })
        .map(|message| {
            let span = &message["message"]["spans"][0];
            (
                message["message"]["level"].as_str().expect("diagnostic level"),
                span["file_name"].as_str().expect("span file"),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(impure_calls, [("error", "src/lib.rs")], "the symlinked scope covers its target: {messages:#?}");
    assert!(
        messages.iter().all(|message| !message["message"]["message"]
            .as_str()
            .is_some_and(|text| text.contains("core_source_scopes"))),
        "an in-root symlinked scope is valid and needs no config warning: {messages:#?}"
    );
}
