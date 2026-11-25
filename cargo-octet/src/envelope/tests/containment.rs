#[test]
fn nested_macro_tokens_and_cfg_attr_enforce_completeness() {
    let policy = super::super::policy::embedded_settings().expect("embedded policy");
    let cases = [
        ("println!(\"{}\", env!(\"SECRET\"));", super::super::CompletenessStatus::UnsupportedGeneratedInput),
        (
            "pub fn f() -> String { format!(\"{}\", option_env!(\"OPT\").unwrap_or(\"\")) }",
            super::super::CompletenessStatus::UnsupportedGeneratedInput,
        ),
        ("my_macro!(OUT_DIR);", super::super::CompletenessStatus::UnsupportedGeneratedInput),
        (
            "#[cfg_attr(all(), doc = env!(\"SECRET_ATTR\"))]\npub fn f() {}",
            super::super::CompletenessStatus::UnsupportedGeneratedInput,
        ),
    ];
    for (body, expected) in cases {
        let dir = tempfile::tempdir().expect("dir");
        super::write(&dir.path().join("Cargo.toml"), "[package]\nname = \"m\"\nversion = \"0.1.0\"\n");
        super::write(&dir.path().join("src/lib.rs"), body);
        let snap = super::super::scan::inspect_local_package(&dir.path().join("Cargo.toml"), &[], &policy);
        assert_eq!(snap.completeness, expected, "body={body}");
    }

    let ok_dir = tempfile::tempdir().expect("ok_dir");
    super::write(&ok_dir.path().join("Cargo.toml"), "[package]\nname = \"okm\"\nversion = \"0.1.0\"\n");
    super::write(&ok_dir.path().join("src/data.txt"), "one\n");
    super::write(&ok_dir.path().join("src/doc.txt"), "doc-one\n");
    super::write(
        &ok_dir.path().join("src/lib.rs"),
        "pub const S: &str = concat!(include_str!(\"data.txt\"), \"!\");\n#[cfg_attr(all(), doc = include_str!(\"doc.txt\"))]\npub fn f() {}\n",
    );
    let snap_a = super::super::scan::inspect_local_package(&ok_dir.path().join("Cargo.toml"), &[], &policy);
    assert_eq!(snap_a.completeness, super::super::CompletenessStatus::Complete);
    assert_eq!(snap_a.generated_inputs.len(), 2);
    super::write(&ok_dir.path().join("src/doc.txt"), "doc-two\n");
    let snap_b = super::super::scan::inspect_local_package(&ok_dir.path().join("Cargo.toml"), &[], &policy);
    assert_ne!(snap_a.package_envelope_blake3, snap_b.package_envelope_blake3);

    let esc_parent = tempfile::tempdir().expect("esc_parent");
    super::write(&esc_parent.path().join("outside.txt"), "secret\n");
    let esc_pkg = esc_parent.path().join("pkg");
    super::write(&esc_pkg.join("Cargo.toml"), "[package]\nname = \"esc\"\nversion = \"0.1.0\"\n");
    super::write(
        &esc_pkg.join("src/lib.rs"),
        "pub const S: &str = concat!(include_str!(\"../../outside.txt\"), \"!\");\n",
    );
    let esc_snap = super::super::scan::inspect_local_package(&esc_pkg.join("Cargo.toml"), &[], &policy);
    assert_eq!(esc_snap.completeness, super::super::CompletenessStatus::Incomplete);
}

#[test]
fn path_attributes_enforce_root_containment_excluded_dirs_and_symlinks() {
    let policy = super::super::policy::embedded_settings().expect("embedded policy");
    let parent = tempfile::tempdir().expect("parent");
    super::write(&parent.path().join("outside.rs"), "pub fn evil() {}\n");

    let bad_cases = [
        ("root_esc", "lib.rs", "#[path = \"../outside.rs\"]\nmod evil;\n"),
        ("src_esc", "src/lib.rs", "#[path = \"../../outside.rs\"]\nmod evil;\n"),
        ("cfg_esc", "src/lib.rs", "#[cfg_attr(all(), path = \"../../outside.rs\")]\nmod evil;\n"),
        ("target_dir", "src/lib.rs", "#[path = \"../target/gen.rs\"]\nmod gen;\n"),
        ("non_rs", "src/lib.rs", "#[path = \"raw.inc\"]\nmod raw;\n"),
    ];
    for (name, rel, src) in bad_cases {
        let pkg = parent.path().join(name);
        super::write(&pkg.join("Cargo.toml"), "[package]\nname = \"p\"\nversion = \"0.1.0\"\n");
        super::write(&pkg.join("target/gen.rs"), "pub fn g() {}\n");
        super::write(&pkg.join("src/raw.inc"), "pub fn r() {}\n");
        super::write(&pkg.join(rel), src);
        let snap = super::super::scan::inspect_local_package(&pkg.join("Cargo.toml"), &[], &policy);
        assert_eq!(snap.completeness, super::super::CompletenessStatus::Incomplete, "case={name}");
    }

    let sym_pkg = parent.path().join("sym_mod");
    super::write(&sym_pkg.join("Cargo.toml"), "[package]\nname = \"sm\"\nversion = \"0.1.0\"\n");
    super::write(&sym_pkg.join("real/inner.rs"), "pub fn i() {}\n");
    super::write(&sym_pkg.join("src/lib.rs"), "#[path = \"sym/inner.rs\"]\nmod inner;\n");
    std::os::unix::fs::symlink(sym_pkg.join("real"), sym_pkg.join("src/sym")).expect("symlink");
    let sym_snap = super::super::scan::inspect_local_package(&sym_pkg.join("Cargo.toml"), &[], &policy);
    assert_eq!(sym_snap.completeness, super::super::CompletenessStatus::Incomplete);

    let ok_pkg = parent.path().join("ok_mod");
    super::write(&ok_pkg.join("Cargo.toml"), "[package]\nname = \"om\"\nversion = \"0.1.0\"\n");
    super::write(&ok_pkg.join("src/alt/inner.rs"), "pub fn i() -> u32 { 1 }\n");
    super::write(&ok_pkg.join("src/lib.rs"), "#[path = \"alt/inner.rs\"]\nmod inner;\n");
    let ok_a = super::super::scan::inspect_local_package(&ok_pkg.join("Cargo.toml"), &[], &policy);
    assert_eq!(ok_a.completeness, super::super::CompletenessStatus::Complete);
    super::write(&ok_pkg.join("src/alt/inner.rs"), "pub fn i() -> u32 { 2 }\n");
    let ok_b = super::super::scan::inspect_local_package(&ok_pkg.join("Cargo.toml"), &[], &policy);
    assert_ne!(ok_a.package_envelope_blake3, ok_b.package_envelope_blake3);
}

#[test]
fn target_src_path_enforces_root_containment_and_excluded_dirs() {
    let policy = super::super::policy::embedded_settings().expect("embedded policy");
    let parent = tempfile::tempdir().expect("parent");
    let outside = parent.path().join("outside");
    super::write(&outside.join("lib.rs"), "pub fn x() {}\n");
    let pkg = parent.path().join("pkg");
    super::write(&pkg.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\n");
    super::write(&pkg.join("src/custom_bin.rs"), "fn main() {}\n");
    super::write(&pkg.join("target/gen.rs"), "fn main() {}\n");
    std::os::unix::fs::symlink(&outside, pkg.join("sym_dir")).expect("symlink");

    for bad_path in [
        outside.join("lib.rs").display().to_string(),
        pkg.join("../outside/lib.rs").display().to_string(),
        pkg.join("target/gen.rs").display().to_string(),
        pkg.join("sym_dir/lib.rs").display().to_string(),
    ] {
        let targets = vec![super::super::scan::PackageTargetMeta {
            kind: vec!["lib".to_owned()],
            src_path: bad_path.clone(),
        }];
        let snap = super::super::scan::inspect_local_package(&pkg.join("Cargo.toml"), &targets, &policy);
        assert_eq!(snap.completeness, super::super::CompletenessStatus::Incomplete, "src={bad_path}");
    }

    let ok_pkg = parent.path().join("ok_target_pkg");
    super::write(&ok_pkg.join("Cargo.toml"), "[package]\nname = \"okt\"\nversion = \"0.1.0\"\n");
    super::write(&ok_pkg.join("src/custom_bin.rs"), "fn main() {}\n");
    let ok_targets = vec![super::super::scan::PackageTargetMeta {
        kind: vec!["bin".to_owned()],
        src_path: ok_pkg.join("src/custom_bin.rs").display().to_string(),
    }];
    let snap_a = super::super::scan::inspect_local_package(&ok_pkg.join("Cargo.toml"), &ok_targets, &policy);
    assert_eq!(snap_a.completeness, super::super::CompletenessStatus::Complete);
    super::write(&ok_pkg.join("src/custom_bin.rs"), "fn main() { let _ = 1; }\n");
    let snap_b = super::super::scan::inspect_local_package(&ok_pkg.join("Cargo.toml"), &ok_targets, &policy);
    assert_ne!(snap_a.package_envelope_blake3, snap_b.package_envelope_blake3);
}

#[test]
fn build_context_and_include_reject_symlink_parent_directories() {
    let policy = super::super::policy::embedded_settings().expect("embedded policy");
    let parent = tempfile::tempdir().expect("parent");
    let ext_cfg = parent.path().join("ext_cargo");
    super::write(&ext_cfg.join("config.toml"), "[build]\ntarget = \"x\"\n");

    let ws_sym = parent.path().join("ws_sym");
    super::write(&ws_sym.join("Cargo.lock"), "# lock\n");
    std::os::unix::fs::symlink(&ext_cfg, ws_sym.join(".cargo")).expect("symlink .cargo");
    let sym_ctx = super::super::policy::compute_build_context(
        &super::super::policy::BuildContextInput {
            workspace_root: &ws_sym,
            target: "x86_64-unknown-linux-gnu",
            toolchain: "nightly-2026-03-21",
            rustc_version: "rustc 1.90.0-nightly",
            cargo_lock_blake3: "b3:1111111111111111111111111111111111111111111111111111111111111111",
            metadata_cargo_args: &[],
            rustflags: "",
        },
        &policy,
    );
    assert!(sym_ctx.incomplete_reason.is_some());

    let ws_real = parent.path().join("ws_real");
    super::write(&ws_real.join("Cargo.lock"), "# lock\n");
    super::write(&ws_real.join(".cargo/config.toml"), "[env]\nA = \"1\"\n");
    let ctx_input = super::super::policy::BuildContextInput {
        workspace_root: &ws_real,
        target: "x86_64-unknown-linux-gnu",
        toolchain: "nightly-2026-03-21",
        rustc_version: "rustc 1.90.0-nightly",
        cargo_lock_blake3: "b3:1111111111111111111111111111111111111111111111111111111111111111",
        metadata_cargo_args: &[],
        rustflags: "",
    };
    let real_a = super::super::policy::compute_build_context(&ctx_input, &policy);
    assert!(real_a.incomplete_reason.is_none());
    super::write(&ws_real.join(".cargo/config.toml"), "[env]\nA = \"2\"\n");
    let real_b = super::super::policy::compute_build_context(&ctx_input, &policy);
    assert_ne!(real_a.blake3, real_b.blake3);

    let inc_pkg = parent.path().join("inc_sym");
    super::write(&inc_pkg.join("Cargo.toml"), "[package]\nname = \"isym\"\nversion = \"0.1.0\"\n");
    super::write(&inc_pkg.join("src/lib.rs"), "pub const S: &str = include_str!(\"sub/config.toml\");\n");
    std::os::unix::fs::symlink(&ext_cfg, inc_pkg.join("src/sub")).expect("symlink sub");
    let inc_snap = super::super::scan::inspect_local_package(&inc_pkg.join("Cargo.toml"), &[], &policy);
    assert_eq!(inc_snap.completeness, super::super::CompletenessStatus::Incomplete);
}

#[test]
fn settings_drive_build_inputs_manifests_and_profile_validation() {
    let mut custom = super::super::policy::embedded_settings().expect("embedded policy");
    custom.workspace_build_inputs.push("custom-build.env".to_owned());
    let ws = tempfile::tempdir().expect("ws");
    super::write(&ws.path().join("Cargo.lock"), "# lock\n");
    super::write(&ws.path().join("custom-build.env"), "V=1\n");
    let input = super::super::policy::BuildContextInput {
        workspace_root: ws.path(),
        target: "x86_64-unknown-linux-gnu",
        toolchain: "nightly-2026-03-21",
        rustc_version: "rustc 1.90.0-nightly",
        cargo_lock_blake3: "b3:1111111111111111111111111111111111111111111111111111111111111111",
        metadata_cargo_args: &[],
        rustflags: "",
    };
    let ctx_a = super::super::policy::compute_build_context(&input, &custom);
    super::write(&ws.path().join("custom-build.env"), "V=2\n");
    let ctx_b = super::super::policy::compute_build_context(&input, &custom);
    assert_ne!(ctx_a.blake3, ctx_b.blake3);

    let pkg = tempfile::tempdir().expect("pkg");
    super::write(&pkg.path().join("Custom.toml"), "[package]\nname = \"cm\"\n");
    super::write(&pkg.path().join("src/lib.rs"), "pub fn f() {}\n");
    let default_policy = super::super::policy::embedded_settings().expect("embedded policy");
    let rejected = super::super::scan::inspect_local_package(&pkg.path().join("Custom.toml"), &[], &default_policy);
    assert_eq!(rejected.completeness, super::super::CompletenessStatus::Incomplete);
    custom.included_manifests = vec!["Custom.toml".to_owned()];
    let accepted = super::super::scan::inspect_local_package(&pkg.path().join("Custom.toml"), &[], &custom);
    assert_eq!(accepted.completeness, super::super::CompletenessStatus::Complete);

    let mut bad_default = default_policy.clone();
    bad_default.default_binding_profile = "unknown-v1".to_owned();
    assert!(super::super::policy::validate_settings(&bad_default).is_err());

    let mut bad_supported = default_policy.clone();
    bad_supported.supported_binding_profiles.push("bogus-v1".to_owned());
    assert!(super::super::policy::validate_settings(&bad_supported).is_err());

    let mut missing_default = default_policy;
    missing_default.supported_binding_profiles = vec!["lockfile-only-v1".to_owned()];
    assert!(super::super::policy::validate_settings(&missing_default).is_err());
}
