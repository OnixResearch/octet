## Why

Octet can import or co-run Clippy, but the run path currently executes only Clippy's default lint set and cannot bind a reviewed selection of restriction lints into the receipt. Stack repositories therefore either duplicate ad hoc lint flags or declare restriction lints without one typed policy identity, deterministic command, and positive/negative evidence rail. Enabling the entire `clippy::restriction` group would be both noisy and contrary to Clippy's own guidance.

## What Changes

- Add a typed Nickel catalog for narrow Clippy restriction profiles, checked-in generated JSON, valid/invalid fixtures, and a freshness gate.
- Ship reviewed `production-safety`, `canonical-determinism`, and `shell-boundary-inventory` profiles instead of enabling the blanket restriction group.
- Make `cargo octet evidence collect --rail clippy-co-reporting --run --profile ...` load the selected embedded profile (or a checked `--config` override), reject arbitrary command authority, and execute a deterministic workspace/all-target Clippy command.
- Bind the selected profile, selected-profile BLAKE3 identity, full config BLAKE3 identity, lint level, exact lint list, command argv, and tool exit status into the existing external-lint receipt.
- Preserve unprofiled Clippy co-reporting compatibility and keep Clippy evidence complementary to Octet's semantic source lints.

## Impact

- **Files**: `standards/clippy-restrictions/`, `standards/policies/registry.ncl`, `cargo-octet` profile core/evidence rail/build packaging, Nix freshness checks, Cairn specification, and operator documentation.
- **Behavior**: profiled Clippy run mode gains checked restriction selection and fail-closed policy/command validation; existing import mode and unprofiled run mode remain available.
- **Testing**: positive and negative Nickel fixtures, pure Rust catalog/command tests, clean and violating Clippy consumer runs, receipt tamper tests, focused Cargo tests, Nix freshness checks, and Cairn gates/validation.
