# Tiger Style Standards Contract

## Purpose

This repo enforces a reusable Rust safety profile through two explicit seams:

- the Dylint lint engine in `src/` and `cargo-tigerstyle/`
- the policy registry and standards checker in `standards/` and `crates/tigerstyle-standards/`

The machine-readable policy index lives in
[`standards/policies/registry.ncl`](../../standards/policies/registry.ncl). It
links Tiger Style source sections to stable policy IDs, enforcement levels,
checker ownership, suppression requirements, and planned fixture evidence.

## Functional Core / Imperative Shell

- Standards policy logic MUST live in `crates/tigerstyle-standards/src/` as pure
  rule functions over plain inputs.
- Filesystem walking, JSON loading, CLI parsing, and exit-code mapping MUST stay
  in the `tigerstyle-standards` binary shell.
- Dylint lint implementation MUST stay in `src/`; standards policy checks MUST
  not be folded into the compiler-plugin crate unless they need rustc HIR.
- `cargo-tigerstyle/` MUST remain the lint runner seam for downstream projects.

## Policy Registry

- Policy IDs use the `tigerstyle.<pillar>.<rule>` namespace.
- Suppressions use `tigerstyle-allow:` comments and MUST cite policy ID, source
  section, owner, rationale, review trigger, and safer default for hard rules.
- Hard policies need positive and negative fixtures before becoming blocking.
- Advisory and human-process-only policies stay visible without pretending static
  tooling can judge them.

## Testing Rules

- Test suites MUST include both positive and negative cases.
- Standards checker tests MUST exercise pure rule functions directly.
- Fixture rails MUST prove compliant and noncompliant examples separately.
- Error-path tests MUST prove the exact failure surface: returned error variant,
  message, and no hidden panic.

## Numeric and Unsafe Discipline

- Non-obvious numeric literals MUST be named constants, parameters, or config
  values.
- Every unsafe block MUST be documented, or unsafe MUST be absent.
- Core validation helpers SHOULD prefer named input structs over ambiguous
  same-typed parameter lists.

## Verification Commands

Run these inside `nix develop` or through `nix develop path:. -c ...`:

```bash
cargo test -p tigerstyle-standards
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
nix run .#tigerstyle-policy-validate
nix run .#tigerstyle-standards -- --help
nix build .#checks.x86_64-linux.policy-registry --no-link
nix build .#checks.x86_64-linux.standards-fixtures --no-link
nix run .#cargo-tigerstyle -- check
```
