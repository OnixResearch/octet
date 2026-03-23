/// UI tests require `rustup` to be available (dylint_testing builds a
/// driver binary for the active toolchain). In nix environments where
/// the toolchain is managed by the flake, this test is skipped.
#[test]
fn ui() {
    let has_rustup = std::process::Command::new("rustup")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rustup {
        eprintln!("skipping UI tests: rustup not available (nix environment)");
        return;
    }

    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests"),
    );
}
