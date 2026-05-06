# Tiger Style Standards Contract

## Purpose

This repo enforces a reusable Rust safety profile through two explicit seams:

- the Dylint lint engine in `src/` and `cargo-tigerstyle/`
- the policy registry and standards checker in `standards/` and `crates/tigerstyle-standards/`

The machine-readable policy index lives in
[`standards/policies/registry.ncl`](../../standards/policies/registry.ncl). It
links Tiger Style source sections to stable policy IDs, enforcement levels,
current/target enforcement modes, structural mechanisms, proof rails, seed
manifests, checker ownership, suppression requirements, and planned fixture
evidence.

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
- Policies that target `structural` MUST export at least one mechanism contract
  such as crate-boundary, capability-api, bounded-primitive, or type-state.
- Capability-api mechanisms export forbidden ambient authority APIs, declared
  core scopes, shell constructor scopes, and approved capability names so the
  architecture checker can reject core-side ambient effects from generated
  policy artifacts.
- Bounded-primitive mechanisms export forbidden unbounded APIs, approved bounded
  constructors/wrappers, capacity witnesses, and backpressure/overflow witnesses.
- Policies that target `verified` MUST export a proof rail with command, source
  mapping, proof artifact path, pass signal, trust-boundary accounting, a
  receipt-binding mode, and receipt provenance obligations. Verification-family
  checks treat source mappings and proof artifact pass signals as hard artifact
  obligations: by default, a selected verified policy fails unless the declared
  source and proof files are present in the checked source tree and the proof
  artifact contains the exact configured pass signal. Rails with
  `receipt_binding = "blake3-command-and-source"` also require
  `proof-command-blake3: <hex>` and `proof-source-blake3: <path>:<hex>` receipt
  lines matching the configured command and checked source contents. Rails with
  `receipt_provenance` configured also require matching `proof-receipt-schema:`,
  `proof-backend:`, `proof-runner:`, `proof-environment:`, and configured
  `trust-boundary:` lines. Rails with `receipt_verifier` configured also require
  exact verifier/toolchain accounting lines:
  `proof-verifier: <id>`, `proof-verifier-blake3: <hex>`,
  `proof-verifier-binary: <path-or-label>`,
  `proof-verifier-binary-blake3: <hex>`,
  `proof-verifier-build-closure: <id>`,
  `proof-verifier-build-closure-blake3: <hex>`,
  `proof-verifier-nix-derivation: <drv-id>`,
  `proof-verifier-nix-derivation-blake3: <hex>`,
  `proof-verifier-nix-closure: <closure-id>`,
  `proof-verifier-nix-closure-blake3: <hex>`,
  `proof-verifier-nix-closure-graph-blake3: <hex>`, `proof-toolchain: <id>`, and
  `proof-toolchain-blake3: <hex>`. If `verifier_build_closure_paths` or
  `verifier_nix_closure_paths` are configured, the corresponding digest is
  derived from checked manifest file contents instead of trusting ambient build
  state. If `verifier_nix_closure_graph_paths` are configured, the graph digest
  is derived from canonicalized generated Nix `exportReferencesGraph` artifacts
  so fixture receipts bind to recursive closure graph evidence, not just a
  declared closure label. A local Nix package exports the `tigerstyle-standards`
  references graph and digest sidecar so fixture Nix closure receipts have a
  first-class regeneration seam. When
  `tigerstyle-standards check --execute-proof-rails` is enabled, the checker runs
  each relevant declared proof command from the checked source root, captures
  deterministic provenance, verifier/toolchain, verifier-binary, build-closure,
  and Nix derivation/closure metadata, command/source BLAKE3 bindings, command,
  working directory, exit status, stdout, and stderr as an in-memory proof
  receipt at the configured proof path, and then applies the same
  pass-signal, binding, provenance, verifier/toolchain, binary, and closure gates. Command
  execution is explicit opt-in; read-only receipt checking remains the default
  for consumer safety.
- Proof-relevant seed manifests MUST remain untrusted until a local acceptance
  command or human/oracle review records provenance and re-verification.
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
