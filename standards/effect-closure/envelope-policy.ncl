let contracts = import "contracts.ncl" in
{
  schema_version = "octet-effect-closure-envelope-policy/v1",
  default_binding_profile = "implementation-bound-v1",
  supported_binding_profiles = [
    "implementation-bound-v1",
    "lockfile-only-v1",
  ],
  relocation_preserving = true,
  included_extensions = ["rs"],
  included_manifests = ["Cargo.toml"],
  build_script = "build.rs",
  workspace_build_inputs = [
    "Cargo.lock",
    ".cargo/config.toml",
    ".cargo/config",
    "rust-toolchain.toml",
    "rust-toolchain",
  ],
  excluded_root_dirs = [
    "target",
    ".git",
    ".jj",
    ".octet",
  ],
  symlinks = "reject",
  unreadable_inputs = "incomplete",
  generated_inputs_policy = "static-literal-only-else-incomplete",
  max_files_per_package = 4096,
  max_bytes_per_package = 16777216,
  max_resolve_depth = 256,
  non_claims = [
    "Implementation-bound dependency envelopes bind package-relative manifests, sorted Rust source bodies, static include literals, selected transitive dependency closures, target/features/toolchain context, and workspace build inputs; checkout paths and file mtimes are excluded so identical layouts remain relocation-preserving.",
    "Binding dependency implementation bytes and build inputs strengthens freshness of reviewed assumptions; it does not verify dependency implementation semantics, execute build scripts, prove compiler or linker soundness, or promote reviewed assumptions to formal proof.",
  ],
} | contracts.EnvelopePolicy
