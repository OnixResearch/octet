# Advisory candidate dogfood run
command: (temp HEAD archive with [workspace.metadata.octet]) nix run path:/home/brittonr/git/octet#cargo-octet -- check --output-format json
exit_code: 0
target_lints: raw_arithmetic_overflow, ambiguous_params, multi_lock_ordering, defaulting_trait_method, numeric_units
disabled_lints_count: 48
artifact_path: cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dogfood.jsonl
stderr_path: cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dogfood.stderr
config_snapshot: cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dylint.toml
