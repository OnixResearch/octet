# Changelog

Notable changes that affect consumers. Organized by date since consumers pin to
specific commits via `TIGERSTYLE_DYLINT_REV`.

For the full lint catalog, see `docs/lint-reference.md`.
For rollout profiles, see `docs/downstream-rollout.md`.

## Unreleased: suppression governance, scope matching, and rail fixes

Changes that can add findings or fail a build:

- `unjustified_allow` now governs `allow` and `expect` attributes written in
  source (including `cfg_attr`-expanded ones and those inside a local
  `macro_rules!`) that suppress Clippy lints, Octet lints and groups
  (`tigerstyle::...`), or Octet bare aliases such as `no_unwrap`, at the crate
  root and on statements, fields, variants, and every other lint-level node.
  Scope comes from the lint paths, not the file path. A justification is a
  non-doc `//` comment on the attribute's lines (a trailing comment counts) or
  the line directly above, or a non-empty `reason`; doc comments and the
  lint's own `TODO(octet)` placeholder do not count. Attributes that other
  crates' macros generate, including proc-macro output re-spanned onto user
  code such as `#[tokio::main]`'s allow block, are not governed. Every strict
  profile requires this lint, so unjustified suppressions of Octet lints now
  deny. `#[expect]` findings use a new message, and the lint summary changed.
- Under a strict profile, any `allow`, `expect`, or `warn` written in source
  that reaches `unjustified_allow` or `unjustified_no_todo_allow`, by bare
  name, `tigerstyle::` name, or a group such as `tigerstyle::structure` or
  `tigerstyle::safety`, is a compile error naming the attribute, the lint, and
  the profile, even when justified. The governance lints stay at `deny` and
  nothing is forbidden crate-wide, so a crate's own group forbids next to
  derive output behave as before.
- A selected strict profile fails the compilation, naming the flag, when
  `--force-warn` or a `--cap-lints` below `deny` would weaken a required lint.
- `unjustified_no_todo_allow` checks `no_todo` escape hatches wherever rustc
  honours them: `let` statements, closures, match arms, crate-level
  `#![allow(no_todo)]`, and `#[expect(no_todo)]`. A justified `expect` lets
  `no_todo` fire so rustc counts the expectation as met.
- `unchecked_narrowing` checks `usize` and `isize` sources as 64 bits wide on
  every target, so `v.len() as u32` is flagged unless guarded.
  `raw_arithmetic_overflow` checks `+=`, `-=`, and `*=` on unsigned places.
  `unbounded_collection_growth` combines guards correctly: `if len >= limit &&
  go() { break; }` no longer bounds growth, and an `||` guard now does.
- `path_segment_repetition` compares item names in library crates and their
  unit tests against the crate name; build scripts, binaries, examples,
  integration tests, and benches are not compared. `multi_lock_ordering` needs
  the whole word `lock`, `locking`, or `locked` with `order`, `ordered`,
  `ordering`, or `acquisition` in its comment, and the compatibility notes
  that exempt `renamed_imports` and `path_segment_repetition` items, including
  notes above grouped `use` statements, match whole words.
- Marker attributes are recognised by path. In `state_machine_exhaustiveness`
  and `fragile_exhaustive_enum_match`, a doc comment that names a marker no
  longer opts a function in, satisfies the fail-closed enum-marker check, or
  seals an enum. `pure_attr_contract`, `mutating_input_in_pure`, and
  `zero_alloc_attr_contract` treat a function as marked only through its own
  attribute, so a marked nested helper or closure, or marker text in a string,
  comment, or `#[doc]`, no longer opts the enclosing function in.
- `verified_purity` sees `std::time` module and glob imports when `verified/`
  is a file-backed module of the crate, and function-local imports.

Fewer false positives, and one compiler crash:

- `multi_lock_ordering` no longer crashes rustc on a struct that holds two
  locks and carries a built-in attribute such as `#[repr(C)]`.
- `no_unwrap` no longer reports the `.expect("Failed building the Runtime")`
  that `#[tokio::main]` and `#[tokio::test]` generate at the user's last
  statement, which failed every `robust`-profile build of a tokio binary.
  Calls are flagged only where the source text spells the method name, so
  user-written calls, including those in `macro_rules!` bodies, still are.
- `underscore_in_module_filename` skips crate root files such as
  `src/bin/load_gen.rs`. `unbounded_loop` counts the `return` produced by `?`
  and `ensure!`/`bail!`-style macros as an exit.
- Under `--test`, `verified_purity` and `assertion_density` skip `#[test]` and
  `#[tokio::test]` functions, and `type_accumulation` ignores the harness
  `main` and test registrations. `nested_conditionals` ignores conditionals
  that external macros such as `assert_eq!` expand to, which used to hide
  later deep nests; nesting built by local `macro_rules!` still counts.
  `assertion_density` strips comments with the Rust lexer and skips items
  whose `cfg` requires `test`.
- Suggestions: `unchecked_narrowing` shapes its `try_from` rewrite by the
  enclosing return type, as `platform_dependent_cast` does (`?` for a
  compatible `Result`, `.ok()?` for `Option`, a placeholder otherwise) instead
  of a `?` that often did not compile. `negated_predicate` keeps a
  machine-applicable rename for reads passed into function-like macros:
  `assert!(!no_x)` becomes `assert!(has_x)`, a whole argument under a
  macro-generated `!` becomes `(!has_x)`, and only arguments the macro uses
  directly, such as `if $cond`, need review. `platform_dependent_cast` and
  `unchecked_narrowing` offer no rewrite for casts from macro expansions or in
  `const fn`; `raw_arithmetic_overflow` writes literal left operands as
  `u64::saturating_mul(2, n)`, which compiles; `unbounded_channel` keeps a
  written module path only when that module exports the bounded constructor,
  and otherwise spells the crate path or offers no rewrite.

Configuration:

- `core_source_scopes`, `state_machine_core_scopes`, and
  `state_machine_generated_adapter_scopes` are normalized as `cargo octet`
  normalizes them (trim, drop a leading `./` and a trailing `/`) and matched
  by whole workspace-relative path components. A scope always matches files
  compiled through its own path, and an in-root scope that resolves through a
  symlink or `..` also covers its canonical target. `./crates/domain/src/`
  matches again, an in-root `linked -> src` scope covers `src/`, and
  `crates/core` no longer covers `crates/core-shell`; a scope that matched
  only as a substring (`domain-core/src` for `crates/domain-core/src`) must be
  rewritten from the workspace root. Scopes that normalize to empty warn
  (`empty_source_scope`). Scopes that name nothing or resolve to the workspace
  root warn that they match nothing, and a scope whose symlink target lies
  outside the workspace warns that it covers only its in-root path
  (`unmatched_source_scope`).
- Octet parses only the `dylint.toml` section it uses. A shadowed `[slotcar]`
  or `[tigerstyle]` section produces a warning instead of being dropped
  silently, or crashing the build when malformed. Under `bounded-systems` and
  `deterministic-core`, the inventory-cap warning that contradicted the
  profile's `deny` is gone.

Evidence rails and artifacts:

- Dependency-audit reconciles every cargo-deny 0.20.2 diagnostic code; codes
  that no check owns, including codes from newer releases, become
  `unclassified` findings instead of ending the run with exit 2 and no
  receipt. Configuration diagnostics that cargo-deny leaves out of its summary
  (the `deprecated` warning for moved `deny.toml` keys and code-less warnings)
  are reported as `unclassified` without being reconciled. A rail run killed
  by a signal fails without a receipt. The Loom profiles also set
  `CARGO_ENCODED_RUSTFLAGS`, so an inherited value can no longer drop `--cfg
  loom` and produce a false pass.
- Dependency-trust receipts carry a `dependency_trust` object that binds the
  cargo-vet store's `config.toml`, the declared imports, and the
  `imports.lock` identity. Declaring imports without `imports.lock` fails,
  receipts produced before this change no longer validate, and collection,
  including `--input`, needs a readable cargo-vet store under the workspace
  root.
- Under the implicit whole-workspace function-address scope (`robust`,
  `bounded-systems`), each out-of-root directory link that is skipped is
  listed by workspace-relative path in `skipped_directory_links`, and a stderr
  note names its target. Every new `function-addresses.json` carries the
  field, so the `function_address_artifact_blake3` in effect-closure receipts
  changes.
- Roadmap capsule and artifact-store redaction drops plural, numbered, and
  environment-suffixed tokens (`api_tokens`, `GITHUB_TOKEN_2`,
  `NPM_TOKEN_PROD`), any word containing `password` (`PGPASSWORD`),
  `passphrase`, `Authorization`, bearer, cookie, session-ID, JWT, and auth
  fields. A `pass`, `pwd`, or non-lookup qualified `key` (`DB_PASS`,
  `MYSQL_PWD`, `AWS_ACCESS_KEY_ID`, `STRIPE_KEY_V2`, `SIGNING_KEY_PEM`) is
  dropped unless the next word only describes or locates it. It keeps
  vocabulary, position, and pagination tokens (`stop_token`, `last_token`,
  `PaginationToken`), plural usage counts (`input_tokens`, `prompt_tokens`;
  a singular `INPUT_TOKEN` is still dropped), and descriptive or identity
  fields such as `pass_signal`, `signing_key_path`, and
  `changed_key_components`. Capsule and artifact hashes change for inputs
  that carry those fields.

Proof rails:

- The Aeneas Lean rail reports the union of `Lean.collectAxioms` over every
  declaration in the `Theorem` module, not only the theorem whose line starts
  with `theorem `, so a `sorryAx` in an attributed, `private`, or namespaced
  lemma fails the rail; `lean-axioms.json` keeps its shape but now lists the
  whole module's axioms. The Lean stage also fails when `lake build` warns
  that a declaration in `Theorem.lean` or `Llbc.lean` uses `sorry`. The
  trust-closure token scanner treats a string as interpolated only after the
  syntax Lean 4.31 parses that way, so `panic! "{"` can no longer hide the
  declarations after it.
- `cargo octet prove` probes the verifier identity of every production rail
  except Aeneas ones, whatever its backend label, so a rail that claims the
  Verus receipt schema under another backend name can no longer reach
  `formal_proof` through an unpinned verifier. The Charon semantic shell no
  longer deletes a work root it was refused.

## Unreleased: lint, evidence-rail, admission, and Verus fixes

Behaviour and format changes:

- Safety lints the strict profiles require: `no_unwrap` flags
  `Result::unwrap` passed as a function and UFCS calls such as
  `Option::expect(x, "m")`; `no_recursion` follows self-calls into closures and
  `async fn` bodies; `unchecked_division` covers `/=`, `%=`, and reference
  divisors; `sentinel_fallback` flags `MAX`/`MIN` through type aliases and
  `Self::MAX`. `drop_must_use` no longer flags `drop(guard)` on RAII guards
  (must-use types with a destructor), and `unbounded_channel` suggestions name
  real functions for aliased imports.
- `function_length` no longer flags `#[test]` functions in `--test` builds and
  no longer counts lines that hold only the body's own braces. Counts for
  multi-line bodies drop by 2 (by less when code shares a brace line); the
  count is part of the message, so those finding IDs change, and findings
  within two lines of the limit can disappear. `type_accumulation` counts
  impls by resolved type and checks the crate root; impl-block warnings now
  name the type by its crate-relative path, such as `crowded::Thing`, so those
  finding IDs change too. Refresh baselines for both lints.
  `excessive_file_length` and `module_file_count` check workspace modules
  named `registry`.
  `raw_domain_value` covers `HashMap`/`IndexMap` keys and borrowed values, and
  `borrowed_argument_types` decides its extern exemption from the actual ABI.
- `bool_naming` and `negated_predicate` renames fall back to a review-required
  suggestion when the new name is already in use, and the `negated_predicate`
  local rename now negates the value with the name. `contradictory_time` sees
  clock reads in closures and `async fn` bodies; `verified_purity` covers async
  methods, `async` blocks, and `std::time` module and glob imports, and exempts
  `#[cfg(test)]` code. `numeric_units` accepts spelled-out and binary unit
  suffixes, `float_for_currency` matches whole words, `ambient_clock` sees
  through import renames and type aliases, and `acronym_style` catches
  trailing and digit-adjacent acronyms.
- Evidence rails: dependency-trust reads real `cargo vet --output-format=json`
  conclusions (failures and violations fail, exemptions need manual review,
  documents without a conclusion are rejected; the old
  `imports`/`exemptions`/`unaudited` shape is no longer accepted).
  Dependency-audit reports must contain `diagnostics`. Run mode executes
  `cargo deny --format json [--config PATH] check`, adding `--locked --offline`
  before `check` when `--locked` is given without `--update-advisory-database`,
  so every dependency-audit receipt's `command_argv` gains `--format json`;
  `--run` converts cargo-deny's JSON lines into the pinned report.
  Dynamic-evidence imports must carry their own profile, command, and integer
  `finding_count`; Miri and Loom runs pass on exit status. The Loom profiles
  run `cargo test` with `RUSTFLAGS=--cfg loom`, so Loom imports must record
  `cargo test`: receipts recorded with `cargo test --cfg loom` are rejected,
  and the cargo-fuzz profile identity changes. Z4 receipts accept only
  recorded non-proof roles, and Kani summaries without complete assumption
  review are review-incomplete or fail. `evidence collect --run` and the
  ast-grep rail run their tools through Bounded Exec: each output stream is
  capped at 16 MiB and each run at four hours, and an overrun fails without a
  receipt. The profiled cargo-fuzz, coverage, mutation, benchmark, and Hegel
  producers keep their own profile limits.
- `cargo octet admit verify` and `cargo octet artifact verify` re-derive an
  admitted closure's module roles and reviewed-summary evidence from the
  recorded inputs, and `artifact verify` also validates the stored closure
  policy, record root, and source envelopes, so a denied closure cannot be
  relabelled admitted. Under an architecture admission binding, a requested
  function-address baseline that differs from the binding's fails with exit
  `2`.
- `cargo octet prove` asks a production Verus verifier for its `--identity`
  report and fails the rail as `unpinned-verifier` unless its binary, solver,
  and toolchain digests equal the pins. Structured Verus receipts record
  closure digests recomputed from the configured paths.

## Unreleased: review fixes to this cycle's changes

The entries below were corrected in place where a review of this cycle's
changes refined them. These fixes have no earlier entry:

- The Nickel export import scanner follows Nickel's lexer for multiline-string
  delimiters and interpolation, and honours `import "path" as 'Format`: a
  dependency is scanned exactly when Nickel parses it as Nickel source.
- SARIF intake rejects a colon only in the first path segment (drive letters
  and schemes), so Octet's own SARIF with paths like `src/a:b.rs` imports back.
- The `cargo octet prove --explain` remediation carries the run's `--family`
  selection.
- Effect-closure call occurrences again number a `let`, `if let`, `while let`,
  or `for` pattern, including a `let` type annotation, before its initializer,
  matching syn's visit order. This removes one source of renumbering; new
  `value:` edges still shift the occurrences of later calls.

## Unreleased: Aeneas, SARIF, architecture, standards, Wasm, and dependency fixes

Behaviour and format changes:

- The `aeneas-lean` proof rail checks each stage's output before the next
  stage runs. Charon's options are read from the produced LLBC and compared
  with the reviewed extraction contract, and `translation.json` is checked
  against that LLBC and the generated Lean. The trust closure comes from Lean
  itself: after `lake build`, a separate check program reports the axioms
  `Lean.collectAxioms` finds for the approved theorem, so `sorryAx`,
  native-code trust axioms, and undeclared axioms fail even when no source
  token names them. A token scan of the approved theorem and generated Lean
  stays as an early filter. The approved theorem must be declared at its plain
  name in the `Theorem` module, and the Lean package receives no source-root
  `.lean` files or `aeneas-lean-library`. Receipts record the observed
  extraction policy and trust closure and the limits and environment allowlist
  actually enforced, and a rail whose allowlist differs from the provider
  profile is rejected. Lean always checks the approved `proof_paths` theorem.
  The profile's `include_rules` are recorded but not applied by the executed
  Charon command, and are not compared.
- SARIF import rejects runs whose `results` or rule tables are missing,
  `null`, or not arrays, runs that report a failed tool execution,
  percent-encoded traversal, out-of-range or disagreeing artifact and rule
  indexes, and unsupported `uriBaseId` values. Genuine CodeQL CLI reports now
  import under `codeql-local`: `%SRCROOT%` maps to the workspace root, rule
  tables under `tool.extensions` are accepted, and percent-encoded paths are
  stored decoded, which changes the identity of such imported findings.
- SARIF output declares `columnKind: unicodeCodePoints`, percent-encodes
  artifact URIs, and marks open Integration architecture findings as errors in
  every mode.
- A `--baseline` run in which `cargo check` did not succeed records
  `scan_complete: false` and reports the stale baseline IDs as unknown
  (`null`) instead of listing IDs from crates that were never checked.
- Architecture scope-level findings (`no-std-obligation`, port and
  protected-protocol resolution and provenance) bind the source of the units
  that own their scope, so source edits make their waivers stale; existing
  waivers on those kinds go stale once. Phase records report
  `waived_finding_count`, and the summary shows `clean with N waived`.
- The compiler architecture collector matches units by Rust target tuple,
  records a dynamic-dispatch unknown for trait calls through `&dyn`, `Box`,
  `Arc`, `Rc`, or `Pin` of a trait object (including standard traits with
  forwarding impls, such as `write_all` on a `Box<dyn Write>`) and qualified
  paths, records calls and effects for function items passed as values such as
  `unwrap_or_else(Instant::now)`, classifies standard-stream output as
  observability and runtime synchronizing `std::sync` and atomic calls as
  concurrency (calls in `const` and `static` initializers, constructors,
  `get_mut`, and `into_inner` record none), and marks a shard incomplete when a
  value-flow limit is hit. Items in non-library targets get target-qualified
  identities; library identities are unchanged. These can newly produce
  findings, and waivers on findings from non-library targets go stale once.
- `octet-standards` dependency-direction checks fail closed when the crate
  graph is incomplete: no `--source` inside a Cargo workspace, `cargo
  metadata` output without a `resolve` graph, or a declared crate that matches
  no package. Valid `line` and `next-line` suppressions now remove the
  diagnostic they cover, and the report lists it under
  `suppressed_diagnostics`. Strict capability manifests need their own replay
  command and artifact for every deterministic handler.
- The policy-package customs gate catches PEM private-key headers and
  credential assignments in common spellings. Promotable and existing packages
  must name a registry family, and policy, profile, template, and contract
  packages must declare a generated artifact or a deterministic export command.
  Nickel export manifests and architecture-policy loading check dependencies
  against the files the source actually imports.
- Wasm profile registries must claim the `wasmparser` and `wasm-encoder`
  versions cargo-octet is built with.

Fixes:

- Aeneas stages run in their own process group inside a private PID
  namespace, so a timeout tears down Cargo, rustc, and Lean subprocesses. Each
  run claims its work root atomically and removes it afterwards, so the same
  rail can be rerun and concurrent runs cannot share one root.
- The architecture gate matches policy target `features` regardless of order,
  lets adapters construct their own types, and its text port checks ignore
  comments, strings, and name prefixes.
- `evidence collect` for the `wasm-artifact`, `charon-semantic`, and `ast-grep`
  rails binds exactly the bytes it read: each input is read once, so piped
  inputs bundle correctly. ast-grep rejects `--command` before it runs.
- Dependencies: `tokio` 1.54.1 (RUSTSEC-2026-0386) and `chacha20` 0.10.2
  (0.10.1 was yanked).

## Unreleased: admission, fingerprint, roadmap, proof-ref, and contract fixes

Behaviour and format changes:

- Effect-closure admission links a call to a local function only when the path
  proves the target. `<Type>::name()`, `<Type as Trait>::name()`, and
  `::name()` never resolve locally, and neither do paths whose first segment
  is a parameter, local binding, generic parameter, or a `use` or item declared
  inside the function. A function or other path used as a value, such as
  `run(std::process::abort)` or `None`, is a `value:` edge that needs a local
  proof or an exact summary; a local variable exempts a value only where it is
  in scope, so a same-named binding in another block, arm, loop, or closure no
  longer hides an imported function. Paths in compile-time positions (types,
  patterns, array lengths, const generic arguments and defaults, enum
  discriminants, and `const`, `static`, and associated-const initializers) are
  not value edges, though calls there are still recorded. Policies may need new
  summaries for constants and variants used as values, and because value edges
  share the call-site occurrence sequence, summaries for later calls in those
  functions go stale and must be re-reviewed. Qualified-self calls keep their
  prefix in the call shape (`call:new` becomes `call:<Vec>::new`), so
  summaries for those calls must be re-reviewed too.
- `cargo octet fingerprint` reports markers that bind to no function (between
  attributes and `fn`, above a doc comment, or a `#[fingerprint(...)]` among
  other attributes): `--check` and `--diff` exit `1` and name each line, and
  `--write` and `--update` refuse to rewrite anything until they move. Macro
  calls in attributes or the signature now raise the macro review warning.
- `cargo octet prove` no longer emits `formal_proof` provenance for a rail
  whose proof goal is blocked, such as a demoted proof card or a provenance
  digest that disagrees with the receipt bytes; that rail's records become
  `manual_review_required` and the run fails. A blocked goal prints a runnable
  `--explain` command.
- `cargo octet check` and `cargo octet admit` exit `2` before running Cargo
  when `DYLINT_TOML` is set: Dylint would lint under that config while finding
  IDs described `dylint.toml`. The consumer guide lists exactly what feeds
  finding IDs and says baselines must be written and checked with the same
  scope, cargo-check arguments, config, profile, and `cargo-octet` version.
- `cargo octet provider-manifest` treats a required lint set to `allow` under
  `lint_levels` as disabled, requires every `--required-lint` to name an Octet
  lint, and rejects a missing rustc executable digest or an `unknown` rustc
  release or commit hash.
- `cargo octet roadmap` fails closed on malformed input: `ability enforce`
  needs string arrays, artifact `inspect` and `replay` accept only full `b3:`
  hashes, re-hash every record, and check the whole dependency chain, and
  `session migrate` reads `schema_version` and requires an `identity`. Artifact
  and capsule identities drop secret-like fields (acronym-aware: `DBPassword`,
  `AWSSecretAccessKey`; a trailing `token` such as `VAULT_TOKEN` or
  `CARGO_REGISTRIES_<NAME>_TOKEN` counts as a credential unless the word before
  it is a listed non-credential word such as `eos` or `page`, and metadata
  fields such as `secret_count` stay) and replace a leading workspace (including
  a symlinked `$PWD`), `HOME`, or temp-dir root, in given and resolved forms,
  with `<workspace>`, `<home>`, or `<temp>`; other absolute strings and
  `*_identity` fields stay verbatim, so in-workspace argv and binary-identity
  changes replay as `drifted`. Artifacts that contain local-root paths get new
  hashes. A `pure-cache` miss reports `miss_reason` and
  `changed_key_components` instead of `changed_key_component`, ignores store
  files that are not cache entries, and a hit with a different `--output-hash`
  exits `1`.
- The `cargo-fuzz-libfuzzer` profile pins `rustc_commit_hash` instead of a
  compiler label. The run withholds `RUSTUP_TOOLCHAIN` and `RUSTC`, observes
  `rustc -vV`, refuses another commit, and then passes the probed compiler to
  cargo-fuzz as `RUSTC` with empty `RUSTC_WRAPPER` and
  `RUSTC_WORKSPACE_WRAPPER`, so cargo config cannot swap it; the profile
  identity changes. libFuzzer `leak-*`, `oom-*`, and `timeout-*` artifacts are
  bound as findings.
- ast-grep import refuses an envelope whose embedded profile differs from the
  checked-in profile.
- The cargo-deny and cargo-llvm-cov external schema contracts are generated
  from their pinned schemas: required fields have no defaults and extra fields
  the schemas allow are accepted.

Fixes:

- The `EvidenceRailReceiptV1` and `FindingV2` contracts accept what Octet
  emits: receipts with `finding_counts`, `findings`, and per-rail details, and
  findings whose `suggestion` and `applicability` are `null`.
- `cargo octet proof-ref` judges overclaims one mention at a time, so Aeneas
  cards scaffold again and a positive claim beside a negated one is rejected.
  It fails when a proof ID matches more than one card and refuses an `--output`
  that points at its own deck or consumer source.
- Fingerprints of literals and doc comments are the same in CRLF and LF
  checkouts; existing LF markers still match. `object update` rejects SARIF
  before writing and replaces a hashless `// octet-object:` marker in place.
- `migrate session verify` rejects duplicate `affected_surfaces` locators, so a
  `live` entry cannot hide an `archive-read-only` one.
- Capability-root reads reject FIFOs and other non-regular files instead of
  hanging.
- The function-fingerprint docs examples are verified again: the five
  `*_docs_fixture_matches_transcript` tests the docs rails select had been
  deleted, so those rails and `docs-examples` always failed. The transcripts
  and `reviewed.rs` markers are regenerated from real output.

## Unreleased: cache inputs, runner variables, CLI flags, and proof receipt fixes

Behaviour and format changes:

- The runner reads `OCTET_DRIVER_PATH`, `OCTET_LINT_LIB`, `OCTET_TOOLCHAIN`,
  and `OCTET_DYLINT_REV`, as the consumer guide documents. The `TIGERSTYLE_*`
  names remain aliases, used only when the `OCTET_*` name is unset; an empty or
  non-UTF-8 value is an error and never falls back to the alias. The Nix-wrapped
  binaries now set the `OCTET_*` names.
- `cargo octet check` exits `2` before Cargo runs when a function-address scope
  from `--function-address-path` or a profile's `core_source_scopes` leaves the
  workspace root (a `..` component, an absolute path elsewhere, or a symlink out
  of the root; the error names the flag or `dylint.toml` entry), refuses
  symlinked directories or `.rs` files inside a declared scope that resolve
  outside the root (the implicit whole-workspace scope skips such directory
  links, like a Nix `result` link, and refuses only escaping `.rs` links), or
  when `--function-address-path` or
  `--function-address-baseline` is given while function-address checks are off:
  no `--function-addresses observe|warn|deny` and no strict profile from
  `--profile` or the workspace's architecture admission binding.
- `cargo octet fingerprint` rejects conflicting or repeated mode flags
  (`--check`, `--write`, `--update`, `--diff`).
- `cargo octet prove` writes to `artifacts/octet-proofs` by default and refuses
  an unsuitable `--artifact-dir` before running any verifier.
- `cargo octet evidence-index` actions reject flags that belong to another
  action; `codebase query --output q.json`, for example, used to print to
  stdout and never write the file.
- `octet-standards` proof receipts fail when configured
  `verifier_build_closure_paths` or `verifier_nix_closure_paths` match no
  checked file, instead of trusting the configured digest, and Verus package
  locks derive their Nix closure digest from those files. Proof-rail path
  patterns that are absolute or contain `..` are rejected and bind nothing.
- Aeneas consumer receipts must carry exactly the five stage-artifact roles,
  with `rust-source` and `maintained-theorem` bound to the selected source and
  approved theorem.
- `export-artifacts --verification-command` records those commands with the new
  status `recorded`, not `passed`, because it never runs them.
- `lib.mkPolicyArtifactConsumerCheck` can assert a nonzero
  `expectedDiagnostics`; it accepts exit `1` only when the report carries
  blocking diagnostics, and fails when exit `0` comes with blocking diagnostics
  or when config and usage errors occur.

Fixes:

- Octet records `OCTET_STRICT_PROFILE` and `DYLINT_RUSTFLAGS` as compile
  inputs. A `--profile` run after a plain `cargo octet check`, or the deny-all
  hook on a warm `target/`, used to replay cached warnings and exit `0`.
- `cargo run --manifest-path <octet>/cargo-octet/Cargo.toml -- check` picks the
  `cargo-octet` binary instead of failing.
- `evidence-index codebase build` skips its own `--output` file during
  directory scans, and the docs write the index outside the check bundle so
  `cargo octet artifact verify` stays valid.
- Aeneas theorem checks catch `True` statements behind binders such as
  `∀ n : Nat, True`.
- Docs: the consumer policy-check examples use the registry ID
  `tigerstyle.embedded.bounded-queues`; the rollout examples and starter
  `dylint.toml` disable every newer warn-by-default lint so they enable exactly
  the lints they name; the config reference lists `excessive_file_length_max`,
  `module_file_count_max`, and `max_parameters`; the release, Verus `prove`,
  and Nickel manifest commands run as written; `cargo octet transcript` is
  described as a structural check that runs no commands.

## Unreleased: exit codes, evidence rails, verify, standards, and purity lint fixes

Behaviour and format changes:

- `cargo octet check` exits `1`, as documented, when denied Octet lints make
  rustc fail the compile. Any other compiler error, an internal compiler error,
  or a signal still exits `2`. `--write-baseline` writes only from a complete
  scan: when denied lints stop Cargo it writes nothing and exits `2`.
- `--baseline`, `--write-baseline`, and the `--cache` directory must lie
  outside `--artifact-dir`, and `check` reuses only an earlier check bundle,
  never a prove or admission bundle.
- `templates/cargo-alias.toml`, which Cargo cannot run, is replaced by a
  `templates/octet-check.sh` wrapper; the gate-posture docs no longer describe
  bypass flags that do not exist.
- The `cargo-llvm-cov-octet-lib` coverage profile passes `--branch`, so its
  identity changes; cargo-llvm-cov export versions 2.0.1 and 3.1.0 are
  accepted.
- `benchmark-evidence` requires `--profile`, `dynamic-evidence` imports take
  their command and signal from the built-in Miri and Loom profiles, and
  Clippy run mode rejects `--command` even without a profile.

Fixes:

- Lines that are not JSON on Cargo's stdout, such as proc-macro output, no
  longer abort `cargo octet check`.
- Evidence rails resolve relative `--config` and `--baseline` paths against
  `--workspace-root`, record the api-stability `--baseline-root` path (baseline
  directories get a tree identity), read cargo-mutants' native missed-mutant
  fields, reject role promotion such as `formal_proof` in any spelling, and
  reject empty verifier-fixture sets. A failing Hegel property under `--run`
  produces a failed receipt with a bounded counterexample.
- `cargo octet artifact verify` refuses a `--emit-valence-provenance` path
  inside the verified bundle, checks `status.json` `phases.effect_closure`
  against the receipt, binds evidence-rail and `formal_proof` records to the
  scope their receipt records, and honours `evidence-log-manifest.json` only in
  prove bundles.
- `octet-standards check` fails closed when the architecture family cannot get
  a crate graph from `cargo metadata`, fails executed proof receipts with a
  non-zero exit status, checks every declared core and shell crate, treats an
  empty dependency-edge scope as normal plus build dependencies,
  requires `--execute-proof-rails` for `--proof-artifact-dir`, and skips files
  that are not UTF-8 during directory walks.
- Purity lints: `zero_alloc_attr_contract` no longer crashes rustc on nested
  constants, array lengths with calls, or nested functions.
  `resource_return_from_core` checks resolved return types, so `SocketAddr` or
  a local `Command` enum stay clean and a `File` in a tuple or alias is caught.
  `global_state_in_core` flags `static` and `thread_local!` declarations once
  each, `missing_const_fn` skips trait methods and functions that call non-const
  code, `logging_in_core` catches `print!` and `eprint!`, and `ambient_random`
  accepts `fastrand::Rng::with_seed(seed)`.
- `multi_lock_ordering` recognises `parking_lot` locks and locks behind `Arc`,
  and `non_trait_imports` treats `use std::hash::Hash;`-style trait plus derive
  imports as trait imports.

## Unreleased: shard freshness, lint, config, proof, Wasm, and admission fixes

Behaviour and format changes:

- Command-line lint levels, such as the deny-all hook's `-D <lint>` flags,
  now take precedence over `[octet.lint_levels]` in `dylint.toml`; a selected
  strict profile's required denies still override both. Unknown keys in the
  `[octet]` section now produce a `dylint.toml` warning.
- `cargo octet prove` refuses a non-empty `--artifact-dir` that is not an
  earlier prove bundle, and clears an earlier prove bundle before writing.
  Check and prove bundles need separate directories; the prove examples now
  use `artifacts/octet-proofs`.
- Wasm artifact registries reject unknown fields, repeated allow/require
  entries, and invalid `allowed_worlds`. The unused `severity.cohort_drift`
  key is removed, so registries that still set it are rejected and every
  profile identity changes.
- Effect-closure admission treats a call as local only when its path proves
  the target; `crate::…` paths, imports from other modules, and trait-default
  `self` calls now need exact summaries.

Fixes:

- Compiler architecture collection keeps cached shards in step with Cargo's
  freshness. Octet records the collector context and a per-unit stamp under
  `.octet/compiler-architecture/unit-stamps/` in rustc dep-info. A lint-only
  build before an architecture build, a deleted shard directory, or a policy
  change now recompiles the affected units instead of reporting every unit
  missing, and an unchanged rerun still recompiles nothing. When a dependency
  change recompiles a unit whose identity is unchanged, the new shard replaces
  the cached one instead of failing as a duplicate.
- The collector records constructor facts for struct literals and unit
  structs or variants, so `adapter-construction-outside-root` catches
  `adapters::Http { base }` and `adapters::SystemClock`. Value-provenance
  facts inside closures and arguments of another crate's type passed by
  reference no longer invalidate shards. Build-script output under `target/`
  is classified as external generated source, and re-reading a recorded span
  at the span limit no longer marks a shard incomplete.
- Lints: `deref_polymorphism` no longer flags generic guards
  (`type Target = T`). `usize_in_public_api`, `public_unsafe_api`, and
  `deref_polymorphism` use effective visibility, so `pub` items on private
  types or in private modules are not reported. `float_for_currency` matches
  skip words as whole name parts and checks float return types.
  `unjustified_allow` accepts a non-empty `reason = "..."`.
  `platform_dependent_cast` suggests a conversion that compiles where it is
  used (a lossless `usize::from` for small sources is machine-applicable) and
  skips an index already checked against `.len()`. `allocation_in_loop`
  reports real allocations (`with_capacity`, `Box::new`, `String::from`,
  `to_string`, `to_vec`) instead of empty `new()` calls.
  `newtype_invariant_bypass` catches `From<&str>` and `Deref<Target = str>`
  bypasses, and `clone_then_borrow` no longer suggests borrowing a value that
  is mutated while the stored clone is in use.
- With `nominal_domain_mode = "inventory"`, a `deny` for the nominal-domain
  lints is lowered to `warn` with a config warning, and suffix-only
  `primitive_domain_alias` candidates are skipped wherever the effective level
  is above `warn`, so discovery candidates never fail a build.
- `cargo octet prove` fails a rail whose verifier output exceeds its log
  limit, `--list` exits non-zero for a malformed or unmatched `--rail`,
  overlapping or root-level `--source` arguments resolve against one Cargo
  workspace root, and Aeneas stage artifacts are written per rail under
  `proofs/<policy>-rail-<N>.aeneas/`.
- Wasm artifacts: `spacewasm-mvp-host-pilot` admits its `molten` imports by
  signature whatever their core type order, parse-stage resource-limit and
  ABI findings use their own severities, and the SARIF projection reports
  `advisory` as `note` without an empty `region`.
- `cargo octet admit verify` rejects admitted receipts that do not match the
  bound function-address records and workspace-policy summaries, and resolves
  a relative `--artifact-dir` against the workspace root like `admit`. `admit`
  refuses a non-empty directory that is not an admission bundle instead of
  deleting files in it. Nested `fn` items no longer cause a permanent
  `function_address_missing` denial, and `cfg` on an enclosing `impl`, trait,
  module, or file makes the function `unknown`.

## Unreleased: fingerprint, object, proof-reference, and architecture-gate fixes

Format and invocation changes:

- Architecture receipts move to `octet-project-architecture-receipt/v3` with a
  new `waiver_policy_blake3` field. Compiler Architecture IR and unit
  identities bind that waiver-free policy hash, so cached architecture shards
  are collected again once.
- The ast-grep rule-bundle identity now binds each list's length, so
  checked-in identities change; capture fresh ast-grep scan receipts.
  Importing a stored scan envelope now requires `--config`.
- `cargo octet fingerprint` records paths relative to the current directory
  and rejects named paths outside it.
- `cargo octet proof-ref validate` and `scaffold` reject proof-card decks
  without `evidence_origin` and `diagnostics`; `cargo octet prove` always
  writes both.

Fixes:

- Architecture waivers work. A waiver's `policy_blake3` must equal the
  receipt's `waiver_policy_blake3`, a policy hash that ignores the `waivers`
  list; before, waivers had to carry the hash of the file that contained them,
  so every waiver was `stale-waiver`. Adding waivers no longer changes finding
  IDs, and any other policy change still makes them stale.
- Architecture collection accepts feature-matrix policies: of several rows on
  one triple, the row matching the invocation's features and default features
  is used. Module directories named like build directories, such as
  `src/target/`, are part of a unit's source identity, while build directories
  beside any `Cargo.toml` (including `fuzz/target/`) stay excluded. A unit with
  several cached shards discards all of them.
- Fingerprints: markers with an empty or placeholder hash report `missing` and
  fail `--check`; marker metadata keys match exactly; fingerprint and object
  markers can sit above one function in either order; and `macro_rules!`
  definitions in `(...)` or `[...]` no longer hide the next function. Missing
  or non-Rust named paths fail `fingerprint`, `object`, and
  `--function-address-path` instead of checking nothing, and SARIF output is
  rejected before any file is written.
- Function addresses: warn and deny modes report `function_address_stale` for
  baseline entries whose function was deleted or renamed within the checked
  scope, and markers written by `cargo octet fingerprint` are no longer
  reported as stale function addresses.
- `object pointer validate`, `object bundle export --pointers`, and
  `object diff --pointer-manifest` fail on a missing pointer manifest.
  `object supplement check` compares every supplement and the supplement
  count, and a receipt with an empty `supplements` list gets an `invalid`
  report with exit 1.
- `proof-ref validate` catches overclaims regardless of verb or hyphenation
  and accepts negated non-claims such as `not verified runtime`.
  `consumer_source_matched` is true only when the consumer-source check ran
  and passed. Matched cards with diagnostics, or imported Trellis evidence
  presented as `formal_proof`, are rejected.
- Charon semantic analysis reports calls through function pointers, trait
  methods, and builtins as `unknown-call` candidates and unknown facts,
  reports unresolved trait methods as `trait-resolution` unknowns, and rejects
  snapshots that list a source path twice.
- The ast-grep rail enforces the contract's severity posture, lane names, and
  non-empty globs, and imports refuse envelopes captured before a rule, rule
  test, config, or scan-scope change.
- Policy-package customs scans fixtures and generated artifacts for raw
  secrets, honours `secret_exclusion_rationale` only for `non-candidate`
  packages, and rejects unknown package kinds. Artifact manifests with
  duplicate paths fail, and Nickel export manifest verification catches
  identity, generator, family, contract, selector, command, path, and
  duplicate-entry drift.

## Unreleased: test-context accuracy and speed

- Lints that skip test code no longer treat production code as test code when
  a `#[cfg(test)]`-gated one-line item, such as
  `#[derive(Debug)] struct Probe;`, sits above it. `no_unwrap` and the other
  test-skipping lints now report there, so baselines can see new stable IDs.
  `#[cfg(test)] // reason` and several attributes on one line still mark the
  item below as test code.
- The test-context check no longer slows down with module size. On a
  generated crate with 1500 functions and 1500 `#[test]` functions checked
  under `--test`, the debug lint library took 1.60 s instead of 22.08 s
  (multi-line functions) and 1.42 s instead of 92.08 s (one-line items), with
  identical diagnostics.

## Unreleased: lint false-positive, migration, and evidence-rail fixes

`unbounded_loop`, `multi_lock_ordering`, `fallible_int_fallback` (in
`#![no_std]` crates) and `concrete_construction_in_core` now report findings
they previously missed, so baselines can see new stable IDs.

- Crash: `sentinel_fallback` no longer crashes rustc when `Self::MAX` or
  `Self::MIN` names an enum variant.
- `unbounded_loop` sees exits from nested loops: a `return` inside an inner
  loop or a `break 'outer` no longer warns. Breaks that only leave an inner
  loop or a labelled block in the body (`break`, `break 'inner`,
  `break 'block`) no longer count as exits.
- `drop_must_use` no longer warns on `drop(option)` or `drop(&value)`.
  `raw_arithmetic_overflow` skips `#[cfg(test)]` code. `fallible_int_fallback`
  fires in `#![no_std]` crates. `multi_lock_ordering` accepts lock-order
  documentation in the struct's doc comments, `#[doc]` attributes, or the
  comment lines directly above it.
- `borrowed_argument_types` skips `&mut` parameters. `fat_trait` counts the
  trait's associated functions instead of `fn` text in default bodies, doc
  comments and strings. `too_many_parameters` does not count explicit receivers
  such as `self: Box<Self>`. `numeric_units` and `magic_number` match whole
  words, so `proportion`, `report_lines` or `transport` no longer trigger them,
  and `numeric_units` no longer asks for a unit suffix on `port`.
- `contradictory_time` flags only `std::time::Instant::now` and
  `std::time::SystemTime::now` (including through aliases), not injected
  clocks or local `now()` functions. `compound_condition` still reports every
  condition with three or more operators but offers its extract-predicates
  rewrite only when the rewrite preserves evaluation order and short-circuit
  guards. `concrete_construction_in_core` classifies constructors by the type
  they build, so fallible external constructors such as `Finder::new(..)?` are
  reported.
- Project architecture gate: a waiver can no longer clear an input-integrity
  finding such as incomplete compiler coverage; the finding stays open and
  the waiver is reported as `unwaivable-integration-finding`. Receipt
  constructors need exactly one approved observation origin. The compiler
  collector records Cargo package names for workspace types, so hyphenated
  adapter packages match their roles, and records crate-root private fields as
  `restricted:<crate>`.
- `cargo octet migrate` rejects flags that do not belong to the selected mode,
  and SARIF output, before reading or writing any manifest. `migrate session
  verify` accepts only the documented `mutability`, `operation_kind` and
  `safety` (`safe` or `manual-review`) values and fails when a rewrite or
  replace targets an undeclared surface. `migrate session apply` lists only
  safe `record-only` operations and writes no operation manifest when the
  session has diagnostics. `object pointer` and `object bundle export` reject
  unknown flags and missing flag values.
- Evidence rails: the benchmark copy census leaves runtime `llvm.memcpy`
  lengths unknown and skips intrinsic declarations. `cargo-fuzz-libfuzzer
  --run` runs under Bounded Exec with a cleared environment, a deadline,
  process-group cleanup and output limits, and refuses to run while
  `fuzz/artifacts/octet-profile-intake` holds earlier artifacts. Mutation
  `--run` writes a receipt when cargo-mutants exits 3 (timeouts) or 4 (failed
  baseline). Hegel identities must be lowercase hex, and MIRAI pilot receipts
  with duplicate criteria are rejected.

## Unreleased: stable finding IDs, roadmap, and external evidence fixes

- Finding IDs no longer change when unrelated parts of the root `Cargo.toml`
  change. The config hash in every semantic and stable ID now covers only
  `dylint.toml`, the `[workspace.metadata.octet]` table (plus the legacy
  `slotcar`/`tigerstyle` tables), the effective scope and `cargo check`
  arguments, and the strict profile. Upgrading re-keys existing baselines
  once; regenerate them with `--write-baseline`.
- SARIF output sets `executionSuccessful: false` only for integration failures
  (exit 2); a complete scan that fails on findings is a successful execution.
  With `--cache`, a cold run no longer labels the second finding sharing a
  semantic ID as a cache hit. `cargo octet artifact verify` fails when
  `status.json` declares an architecture or effect-closure phase whose
  artifacts are all missing.
- `cargo octet roadmap` fails closed on malformed input. `capsule replay` exits
  2 for a capsule without a manifest, with a missing compared field or
  non-integer `expected_exit_code`, or with a stale `capsule_hash`. `pointer
  rollback` restores the previous artifact kind and refuses receipts that later
  pointer updates superseded. `session migrate` reports an error instead of
  panicking on non-object JSON, and every subcommand rejects unknown, repeated,
  or valueless flags.
- Charon semantic comparison compares multi-valued facts (several
  `local-call`/`item-dependency` edges or effect candidates) as sorted value
  sets, so results no longer depend on input order. SARIF import gives a result
  without `level` its rule's `defaultConfiguration.level`, skips `pass`,
  `notApplicable` and `informational` results, and rejects undecided
  `review`/`open` results.
- The Aeneas rail judges theorem adequacy on the whole named theorem
  declaration, including statements spanning several lines, instead of a
  helper theorem after it. The Aeneas consumer check compares the selected
  item's LLBC source text with the lines its span selects, so multi-item source
  files validate and mismatched text reports drift.

## Unreleased: lint crash, coverage, and autofix fixes

Several fixes report findings that were previously missed, so consumer
baselines and warning budgets can see new stable IDs; refresh them deliberately.

- Crash: `zero_alloc_attr_contract` no longer panics rustc when a
  `#[zero_alloc]` function contains an array type such as `[u8; 4]`, a repeat
  expression, or a statement-level `const`.
- Lints that skip test code (`no_panic`, `no_todo`, `no_unwrap`, ...) also skip
  `#[cfg(test)]` methods in `impl` and `trait` blocks. `no_todo` checks for an
  allow attribute before the test-context walk, which made it about 5x faster
  on a generated 6000-function module under `--test`, with identical output.
- `no_panic` and `no_todo` identify panic-family macros by definition, so
  zero-argument `panic!()`, `unreachable!()`, `unimplemented!()`, `todo!()`
  and path-qualified forms such as `std::panic!("…")` are reported, and
  justified `#[allow(no_todo)]` over them is no longer called stale. The
  `_ => unreachable!()` exemption no longer treats `Mode::A(0)` as covering
  all of `Mode::A`.
- `no_unwrap` reports `.unwrap()`/`.expect()` on `&Option`/`&Result` receivers.
- `catch_all_on_enum`, `foreign_enum_catch_all` and
  `fragile_exhaustive_enum_match` check matches over enum references, such as
  `match self` in `&self` methods.
- `no_recursion` catches `Self::f(..)`/`Type::f(..)` self-calls and trait impl
  methods that call themselves through the trait.
- `function_length`, `nested_conditionals`, `assertion_density` and
  `unjustified_allow` check `impl` and `trait` methods, not only free
  functions. `nested_conditionals` keeps an `else if` chain at one level, and
  `assertion_density` counts `debug_assert!` once instead of twice.
- `compound_assertion` ignores `&&`/`||` inside the panic message, and its
  split fix stays valid in match arms and closures.
- `unchecked_division` matches zero guards to the divisor variable instead of
  source text: an `== 0` check clears only its else-branch or code after an
  early return, and a guard on another variable (`len` vs `n`) or an assert
  after the division no longer hides a finding. `unchecked_narrowing` requires
  a lower bound before accepting an upper-bound guard or `.min(..)` on a signed
  value, honors `#[allow]`/`#[expect(clippy::cast_possible_truncation)]` on
  impl and trait methods, and no longer treats `#[deny(...)]` as an allow.
- `acronym_style` no longer flags one-letter words such as `XAxis` and checks
  every acronym against `acronym_allowlist`. `path_segment_repetition` and
  `renamed_imports` take their compatibility exemption only from the comment
  above the item, not from identifiers such as `rapid` in its body.
- `excessive_file_length` skips files outside the workspace, so `include!`d
  build-script output under a custom `CARGO_TARGET_DIR` no longer makes the
  runner exit 2 with "absolute path is outside the workspace root".
- `ambient_env`, `ambient_random`, `impure_call_in_core`, `spawn_in_core` and
  `interior_mutability_in_core` match the called function, not the whole call
  text, so a wrapper such as `parse(std::env::var("PORT").ok())`, `Some(..)` or
  `Arc::new(Mutex::new(0))` no longer adds a second report. `logging_in_core`
  reports each `println!`/`eprintln!`/`dbg!`/`log`/`tracing` invocation once,
  at the macro, instead of at the enclosing function body, `if` and blocks.
- `unbounded_collection_growth` accepts `v.try_reserve(n)?` as a reservation
  and reports growth inside `loop`/`while` nested in another loop.
  `state_machine_exhaustiveness` decides wildcard arms from pattern structure,
  so snake_case bindings, underscored paths, and `_`/`..` inside variant
  payloads no longer read as fallbacks. `type_accumulation` counts a module's
  own public items, so file-backed `mod foo;` modules are checked and inline
  modules are no longer over-counted by `pub` fields and impl methods.
- The `generated_searchable_names` exception class now takes effect:
  `non_trait_imports`, `renamed_imports` and `path_segment_repetition` skip only
  files outside the workspace or under whole `target`, `generated`, `OUT_DIR`,
  `rustlib`, `.cargo` or `*vendor-cargo-deps*` directories, so ordinary
  modules such as `crate::registry` or `src/targeting.rs` are linted.
  `verified_purity` also denies importing impure paths in `verified/` modules,
  for example `use std::time::Instant;`.
- Autofix: `bool_naming`/`negated_predicate` renames cover uses in closures,
  async blocks and struct field shorthand; `ignored_result` keeps the `;` in
  `expr?;`; `unbounded_channel` keeps turbofish arguments like `::<()>`; and
  `unbounded_collection_growth` no longer suggests `BTreeMap::with_capacity`.

## Unreleased: runner artifact-dir and compile-failure fixes

`cargo octet check --artifact-dir` no longer deletes the contents of a
directory it did not write: the directory must be new, empty, or an existing
Octet bundle (both `status.json` and `provenance.jsonl` present). Any other
non-empty directory, such as `target/` or the workspace root, is left untouched
and the run fails as an integration failure (exit 2). rustc's informational
top-level `note`, `help` and `failure-note` diagnostics are no longer findings,
so a compile error or ICE under `--output-format json|sarif` produces a
structured integration-failure report instead of aborting with `invalid
finding domain values`. The `Cargo.lock` guard now settles before the bundle
is written, so a run whose lockfile Cargo rewrote reports
`integration-failure` and exit 2 in `status.json`, SARIF and the summary,
matching the process exit code. `cargo octet evidence collect --artifact-dir`
applies the same rule as `check`: only a new, empty, or earlier evidence bundle
(with `evidence-rail-receipt.json` and `provenance.jsonl`) is pruned.

## Unreleased: fail-closed effect-closure, proof, evidence, and Wasm checks

- Effect-closure admission treats compound assignment (`+=`, `-=`, `*=`, `/=`,
  `%=`, `<<=`, `>>=`) as panic-capable, so `deterministic-core` no longer
  admits functions that only use those forms. Verifying an `admitted`
  receipt now re-derives admission facts (no unknown call edges, closures
  within the allowed effects and containing each callee's closure), so a
  denied receipt cannot be relabelled admitted by dropping its diagnostics.
  The recorded admission dependency graph now receives the checked build's
  `--manifest-path`, `--config`, `-F`/`--features`, `--offline` and `--frozen`
  arguments, and a value flag without its value is an error.
- `cargo octet prove` drains verifier output while the verifier runs, within
  the rail's log limits, and kills the verifier's process group. Verbose
  verifiers no longer report false timeouts, and leftover background processes
  cannot hang teardown.
- `cargo octet evidence collect` fails JSON rails on empty input, and in
  `--run` mode fails any rail whose tool exits nonzero without reporting a
  failing or review-required finding (previously only Clippy did).
- Wasm artifact receipts derive the handoff role only from `mantle:`,
  `chaoscontrol:` or `molten:` parent identities. Verification rejects
  role/identity mismatches and recomputes the interface identity from the
  recorded facts. Profile registries with unknown `severity.*` or
  `duplicate_custom_sections` values are rejected.
- `cargo octet provider-manifest export` resolves `rustc` the way process
  execution does and both runs and hashes that file; `compatible_rustc.executable`
  is the resolved path, and an unreadable executable is an error instead of a
  missing `executable_blake3`.

## Unreleased: octet-standards family selection and manifest parsing

`octet-standards check` rejects a `--family` that is not a pillar of any
registry policy, or a selection that evaluates no policies, instead of passing
silently. The FCIS crate-boundary checks parse Cargo dependency tables:
`name.workspace = true`, `[dependencies.name]` and target tables count, while
`[features]`, `[build-dependencies]` and `[dev-dependencies]` entries do not.
Declared crate paths match on whole path components, so `core/Cargo.toml` no
longer matches `crates/hardcore/Cargo.toml`, and capability handler coverage
markers must appear as whole words, so `token` no longer counts as `ok`.

`octet-standards check --execute-proof-rails` writes the executed receipt to
the rail's `receipt_paths` and removes committed receipts matching them, so a
failing proof command can no longer pass on a valid checked-in receipt. Rails
that configure `verifier_nix_closure_graph_paths` fail when those paths match no
checked artifact or the matched graph is malformed, instead of accepting the
configured digest; the bundled deterministic-effect-handlers rail now points at
`standards/check-fixtures/verification/*/proof-nix-closure.graph`.

## Unreleased: architecture planning and collection fixes

`cargo octet check` with an active architecture policy no longer exits 2 with
"Cargo feature plan has ambiguous duplicate unit identities" when resolver 2
plans a package twice with equal descriptions, as it does when a build script
and the library resolve different features for a shared dependency (Octet's own
`toml`/`serde_core` graph is one example). Plan identities now bind each unit's
complete dependency subgraph, so such copies stay distinct, while units with
identical subgraphs and dependency cycles still fail closed.

The compiler architecture collector now records each item's real containing
module. It used to turn every impl block, function, and derive-generated
constant that contains items into a pseudo-module, which exhausted
`max_modules` on derive-heavy crates and made coverage incomplete (exit 2).
Module facts and IR identities change accordingly. Octet's own policy now
covers `cargo-octet/build.rs` and allows 128 MiB shards.

## Unreleased: Exact SpaceWasm pilot ABI facts

SpaceWasm analysis now resolves core function export/import signatures and binds
them into normalized interface identity. Hostless `run` must have signature
`() -> ()`. Artifact reads and parser construction are bounded by profile size.
The additive `spacewasm-molten-abi-pilot` profile checks the real Molten
memory/allocator/deallocator/packed-result ABI and closed assert/send hostcalls;
`spacewasm-mvp-host-pilot` separately checks the diagnostic observe/pause ABI.
Both preserve the reviewed immutable NASA source cohort and explicit static-only
boundaries. Operator smoke and positive/negative ABI fixtures are provided;
runtime semantics, hardware, and producer provenance remain separate evidence.

## Unreleased: reviewed Valence source-token-v2 pin and source-bound object rails

- Candidate `cargo-octet` source-backed object, marker, corpus, and supplement
  commands share one typed Valence constructor over each function's original
  byte span and `octet-source-token-v2` tokens. Source line conversion and
  addition reject `u32` overflow instead of silently wrapping. Local integration
  tests cover quoted-URL literals, code after the URL, trivia, corpus
  source/boundary drift, marker identity, and bounded HIR/THIR/MIR selected
  fixture emit, replay, and tamper refusal without claiming correctness.
- Fixed a shared scanner gap after reproducing an ABI-breaking identity
  collision: `#[unsafe(export_name = "alpha")]` versus `"omega"` above a
  comment-separated `fn` exported distinct symbols under rustc but returned
  the same v2 object hash. Attached outer attributes now participate in source
  spans across intervening comments/blank lines. Regression coverage requires
  distinct v2 object identities, a stale reviewed function-address marker,
  stable identity for token-preserving ordinary whitespace/non-doc comment edits,
  and preserved marker/unrelated-item boundaries.
- Closed the remaining multiline attribute and split-visibility collisions:
  valid `#[unsafe(export_name =` followed by a literal-only line and `)]`
  exported distinct `alpha`/`omega` symbols but previously had identical v2
  hashes; valid `pub` on the preceding line also hashed like a private
  function. Balanced bracket matching over the scanner's same-length code
  view now includes stacked/multiline attributes despite nested brackets in
  literals/comments, and item-prefix scanning includes split visibility and
  modifiers. Reviewed function-address and object markers stale on those
  edits; ordinary whitespace/non-doc comment edits that leave Rust TokenStream
  unchanged remain stable.
- Closed a reviewed-marker trust-boundary bug: `// octet-object:` inside an
  enclosing Rust block comment previously produced a false `match` when its
  digest and review fields matched. Object and fingerprint/function-address
  markers now require an active lexical marker start, not marker-looking text
  in a block comment or literal. A closing `*/` on the function's own line
  stays outside its source identity; explicit marker update inserts an active
  marker after the closer and still requires real review metadata. Marker
  association shares one monotone lexical scan per file, rather than
  rescanning every source prefix for each reviewed function.
- Closed source-token-v2 collisions observed with real Rust procedural macros:
  joint versus separated punctuation, numeric literals versus split tokens,
  raw identifiers/lifetimes, attached literal suffixes, and doc comments
  inside macro arguments produced different compiler token streams but shared
  an object hash. The candidate scanner now preserves these lexical boundaries
  while ordinary comments and group-adjacent whitespace remain trivia.
  Macro review caveats never make colliding source identities acceptable.
- The caller requests the v2 label from the typed Valence constructor without
  presenting corrected bytes as v1. The selected reviewed Valence commit
  `7546a78b4eeef8686c6837e187aa12420f4f03bb` admits source-backed v2
  objects; historical v1 source replay does not acquire a false match.
- The fixed quoted-URL source and a locally regenerated v2 diagnostic receipt
  under `cargo-octet/tests/fixtures/object-corpus/source-token-v2/` have an
  explicit `candidate-source-manifest.json` with source, receipt, and exact
  tested Valence module hashes; this historical candidate remains `PENDING`.
  The tested module is identified by canonical repository URI and
  repo-relative path, never a local checkout path. The same directory also
  contains internal (18 paths), expanded (36 paths), and
  expanded caveat-taxonomy v2 references. The candidate manifest binds all 37
  source-byte hashes, four artifact hashes, and prospective Octet
  module hashes. A distinct independently audited
  `reviewed-source-manifest.json` binds these approved H2 identities to
  the immutable candidate snapshot and selected upstream revision/module;
  it excludes later consumer-test edits and does not claim an owner signature.
  Fresh byte-identical replay is diagnostic: changing one ordinary non-doc
  comment byte without changing Rust TokenStream leaves the token object set
  stable but invalidates its source binding. The existing named object rails
  require fail-closed admission of the frozen manifests, exact 37 source-byte
  hashes, four saved reference bytes, scoped owner approval, and the selected
  Cargo/Nix Valence identity before executing source-backed positive and
  negative tests. Two independently built binaries must also match each other
  and both the saved internal and expanded receipts; matching local binaries
  alone do not supply review.
  The in-repository verifier and expected hashes share a PR-controlled trust
  boundary, so an external reviewer must compare the frozen candidate and
  reviewed manifest BLAKE3 anchors independently. H2 also does not freeze
  compiled `valence_core_shim.rs` or `object_core.rs`; observed selected
  producer behavior cannot authenticate arbitrary later producer edits.
  After OpenSpec archive, the manifest's old approval path remains immutable
  historical metadata; the verifier reads the archived H2 approval note with
  an exact byte-hash drift lock and rejects missing, altered, or resurrected
  active approval paths. All eight named children, the two-binary fresh-build
  rail, and the twelve-dependency aggregate passed again after that cutover;
  the in-PR note hash is not an owner signature or external H2 authentication.
- Incidental contributor-guide substring, help-wording, and human-contract-text
  assertions were removed; actual CLI transcript and fixture inventory
  behavior tests remain.
- A selected Valence dependency alone is **not** an approved object-rail PASS.
  Candidate proof used a disposable, non-Git Octet copy with a local Cargo
  `[patch]`; the independently reviewed upstream module has since been published
  at `7546a78b4eeef8686c6837e187aa12420f4f03bb`, and the selected
  Cargo/lock/Nix/manifest pins now identify that exact revision via direct
  SSH Cargo and flake sources. Existing v1 receipts, markers, policies, and
  accepted evidence were not rebound. A named PASS requires its actual Nix
  derivation to build after source admission, not only focused source tests.
- Historical v1 source replay under the selected v2 pin returns
  `status=mismatch` on `object_set_hash`; v1 artifact-only readers still work.
  The owner's H1 `approve` decision permits replacing the obsolete positive
  v1 replay assertion with the honest mismatch and removing the false v1
  object-set match assertion from boundary tampering. A separate test reads
  the saved v1 receipt and exercises diff/pointer/bundle on transparently
  test-owned, real typed Valence v1 objects serialized to JSON (one pinned to
  the upstream core's v1 golden identity), not a nonexistent archived standalone
  v1 object. The selected eight named child rails and aggregate subsequently
  passed actual source-bound Nix derivations, not merely focused source tests.
  The supported OpenSpec archive now promotes the versioned v1-mismatch/v2-match
  contract to the canonical `standards-codification` spec, retaining old v1
  JSON and artifact-only reads without asserting source replay remains valid.
- The exact repo-pinned Cairn CLI commit `695124d459574ba7aeba6097310d237f393c243c`
  is now reachable through immutable SSH flake transport without changing
  Octet's Cairn policy. A separate native spec-driven change added current v2
  Valence compatibility requirements alongside historical Phase 2 v1 text and
  was gated, synced, strictly validated, and archived. The unrelated newer
  installed CLI's additional profile did not authorize a policy migration.

## Unreleased: Consumer fixture matches Cargo metadata and source assets

- The consumer integration harness now delegates `cargo metadata` to Cargo
  against the fixture's actual manifest, without fetching dependencies or
  changing its lockfile. Standalone CLI builds copy the `standards/` source
  assets needed by compile-time includes. This repairs consumer test-harness
  parity; it does not establish that ObjectV1 selected gates pass.

## Unreleased: Object rails blocked; new scanner names need marker re-review

- Object tests that built objects from source were removed rather than re-pinned.
  These covered emit, marker check and update, caveats, determinism, corpus receipt
  round trips, the harness, and HIR/THIR/MIR supplement replays. Two tests now cover
  the refusal: one runs every source-backed object command, and one shows supplement
  receipt validation runs before the refusal. Pointer, bundle, and receipt-validation
  tests now read existing object JSON or the checked-in v1 receipt instead of emitting.
  Tests that only read checked-in artifacts were renamed (`checked_in_*`,
  `*_spike_fixture_listing_*`).
- At this earlier blocked stage, the then-named
  `object_corpus_compatibility_positive_v1_fixture_replays_cleanly` and
  `object_corpus_compatibility_boundary_tamper_fixture_fails_closed` checked
  a v1 source-replay contract and stayed red. The H1 migration above
  replaces that obsolete positive assertion; this entry records history,
  not the current test expectation.
- At this earlier blocked stage, the object, supplement, and corpus Nix rails failed closed with
  `<check>: no test selected by '<filter>'; selected rail unverified: Function Object v1 construction is refused until a reviewed valence-core normalization label is pinned`.
  The compatibility rail was red on its blocked tests, and
  `fresh-build-object-corpus-determinism` stopped at the refused receipt command.
  `valence-object-evidence` therefore could not build or print PASS lines at that stage.
  `docs/current-head-valence-evidence.md` still records an earlier payload commit,
  and its object pass signals do not hold for the current tree.
- The source scanner now also finds functions whose `fn` is followed by a tab, a
  newline, or a comment. It names functions with Unicode identifiers, and raw
  identifiers such as `r#type`, by their full name. Unicode identifiers stay whole in
  canonical tokens and dependency sets, and Unicode-named macro calls count as macro
  context. Names, locators, hashes, and addresses of the affected functions change.
  Existing fingerprint markers, function-address markers, and baselines must be
  re-reviewed; nothing rebinds them. This does not establish complete Rust syntax
  coverage.

## Unreleased historical stage: Source-token identity v2 before reviewed Valence pin

Canonical source tokens now keep every string, raw-string, byte, and character
literal whole, including any `//` inside it. Ordinary line and nested block
comments are dropped; compiler-visible doc comments inside function bodies
are retained. Before this change, each line was cut at its first `//`,
so code after a quoted URL was never hashed. Literal contents were also split
into tokens, so whitespace changes inside a literal went unseen. Dependency sets
now hold only identifiers in code; words inside literals no longer count.

- `cargo octet fingerprint` hashes a version tag ahead of the tokens. A marker
  written before this change never matches, even for an unchanged function.
  Every existing reviewed marker reports `mismatch` and fails closed until its
  owner re-reviews the function and runs `--update`.
- Function addresses use canonicalizer `octet-source-token-function-address-v2`,
  which is hashed into every address. Existing function-address markers and
  baselines report stale until re-reviewed.
- At this earlier stage (superseded by the reviewed pin above), the selected
  v1-only valence-core pin rejected source-backed Function Object
  v2 construction. That pin accepted only the normalization label
  `octet-source-token-v1`, which names the old token bytes.
  Every `cargo octet object` command that builds objects from source (`emit`,
  `check`, `update`, `corpus receipt`, `corpus caveats`, `corpus check`,
  `supplement emit`, `supplement check`) requested `octet-source-token-v2`;
  the then-selected v1 pin compiled but rejected that label with
  ``function object normalization algorithm `octet-source-token-v2` does not match v1 algorithm `octet-source-token-v1` ``.
  `object corpus check` returned `status="invalid"` with error kind
  `source-selection` on `paths`; neither command emitted a v1-labeled object.
  Objects stayed unavailable until the reviewed valence-core label for the
  corrected bytes was pinned. Commands that read
  existing object JSON or list fixtures (`diff`, `pointer`, `bundle export`,
  `corpus contract`, `supplement fixtures`) are unchanged.
- The five fingerprint docs examples still carry pre-change marker hashes.
  Their transcript tests compared stored hashes and transcripts byte for byte,
  so they were deleted rather than re-pinned. Separate tests now cover two
  behaviours that only those tests checked: warning counts failing `--check`,
  and JSON stored and computed hashes across drift. Every
  `mkFingerprintDocsCheck` rail now fails closed with
  `<check>: no test selected by '<filter>'; selected rail unverified` when its
  filter selects no non-ignored test. The five `fingerprint-*-docs` checks
  therefore fail, and so does `fingerprint-docs-downstream-smoke`, until an
  owner re-reviews and regenerates the examples.

## Unreleased: Macro-call detection ignores keywords, comments, and literals

Macro-context detection for function addresses, `cargo octet fingerprint`, and
`cargo octet object` now reads the same comment- and literal-blanked view as the
function scanner. A macro call is a name or path followed by `!` and a `(`, `[`,
or `{` group. Keywords such as `if`, `while`, `match`, and `return` before a unary
`!` are not macro names, and macro-like text in comments or string, raw-string,
byte, or character literals no longer counts. A real macro after a quoted `//`
is now found. Functions whose classification changes move between
`source-scanner-token-set` and `unsupported-macro-context`, which changes their
function address, fingerprint semantic ID, and function object identity;
function-address markers and baselines report them as stale until re-reviewed.
Token hashes and `cargo octet fingerprint` markers are unchanged.

## Unreleased: Object-corpus freshness rail fails closed

Five `cargo-octet` tests compared checked-in internal-corpus receipts and the
caveat taxonomy byte-for-byte with output generated from Octet's own changing
source. They and the ignored fixture re-pin helper were deleted as implementation
snapshots. The fixtures stay and are still checked for structure and boundary
claims; separate tests still cover fresh-receipt determinism and receipt replay.

At this earlier blocked stage, `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness` listed the
tests its `matches_fresh` filter selects before running them. With no
non-ignored test selected, it exits nonzero with
`no test selected by 'matches_fresh'; selected rail unverified: Function Object v1 construction is refused until a reviewed valence-core normalization label is pinned`
instead of passing without running anything. No such test existed then, so the
rail failed, and `valence-object-evidence`, which depended on it, could not pass
until an owner-reviewed freshness comparison replaced the deleted tests.

## Unreleased: Functions with array-typed signatures are scanned

The source scanner now ends a function signature only at a top-level `{` or
`;`. A `;` inside an array type such as `[u8; 2]` no longer makes a function
look bodiless, and a `{` inside generic arguments such as `W<{ N }>` is no
longer taken as the body. Functions with array parameters or return types were
previously left out of function-address receipts, `cargo octet fingerprint`,
and `cargo octet object` without any finding; they are now recorded and checked
like other functions. Bodiless trait and foreign declarations are still
skipped, and functions inside `macro_rules!` definitions are still excluded.

## Unreleased: Source function scanner ignores comments and literals

The source scanner behind function addresses, `cargo octet fingerprint`, and
`cargo octet object` no longer matches `fn`, `macro_rules!`, braces, or
semicolons inside line, doc, or nested block comments, or inside string,
raw-string, byte, or character literals. Documentation examples such as
`compile_fail` doctests no longer produce function records, missing markers,
or duplicate locators. A function containing a quoted or commented brace now
ends at its own closing brace, so its function address changes, and a
function its neighbour previously swallowed is recorded again. Spans, hashing,
and macro-context detection are otherwise unchanged. No lint levels, profiles,
or baselines change.

## Unreleased: Compile-time unsigned arithmetic

`raw_arithmetic_overflow` skips primitive arithmetic composed of literals
and named const items. Associated consts and inline const blocks remain
checked conservatively, as do runtime locals and parameters in `const fn`.

## Unreleased: `#[test]` functions in test-harness builds

Under `--test`, rustc consumes `#[test]` and registers functions through
same-named sibling test-marker constants. The shared test-context classifier
now recognizes these functions without exempting non-test harness helpers.
`ambient_clock` now uses that shared classifier instead of a private copy that
flagged clock reads in harness `#[test]` functions and `#[cfg(test)]` modules;
non-test helpers and `cfg(not(test))` code still warn.
No lint levels, profiles, or exemptions change.

## Unreleased: Compiler diagnostic source paths

The runner resolves compiler-origin absolute spans and package-rooted relative spans containing `..`
only when the package ID matches the complete Cargo graph (including in-workspace local path
dependencies excluded from workspace membership), each physical path component stays inside the
workspace, and the target is an existing regular file. Intermediate files, missing paths, FIFO
targets, and symlink escapes fail closed. On the selected Linux runner, a nonblocking capability
open and Unix inode comparison constrain the check, but concurrent swap-back can still change
source identity: this is a containment check, not an immutable snapshot. Clean relative spans retain their existing
root-relative validation. Non-Unix platforms reject absolute/parent-bearing compiler spans
until an equivalent nonblocking capability check exists; artifact path validation is unchanged.

## Unreleased: Forged question-token compiler control

A real compiler fixture stamps a newly generated filesystem-reading function with an authored `?` token's span. The original item and resolved `std::fs::read` Call/Filesystem Effect retain that source location, while rustc's lowered `Try::branch` span retains an expansion with no resolved macro definition.
The collector retains the exact `UnsupportedExpansion` unknown, and Gate mode blocks both that unknown and the same function's filesystem Effect by their fact identities. A separate directly authored `?` keeps the control tied to the forged operator rather than an unrelated desugaring.
This adds a regression, not a collector exemption or a claim that every possible proc-macro token substitution is authenticated.

## Unreleased: Compiler randomness effects

The collector now classifies the four root `getrandom` entry points: `fill`, `fill_uninit`, `u32`, and `u64`.
Classification uses the compiler's defining crate and function shape, not alias-sensitive diagnostic paths.
Calls and effects retain the same recorded paths, callers, spans, identities, and production observers.
Local same-named helpers and unrelated definitions do not become randomness effects through a prefix match.

Real registry controls cover all four APIs through an extern-crate alias in both production library configurations.
This repair changes no schema, role, target, lint level, or exemption. It does not establish downstream acceptance.

## Unreleased: Non-function compiler body owners

The collector preserves constant and static caller kinds during lazy discovery.
Inline and anonymous constants no longer trigger invalid function-signature or named-item visibility queries.
Expression owners retain private visibility, caller references, and compiler-unit memberships.
Function signatures, closure identities, and production effect checks remain enforced.

Compiler controls cover named, associated, inline, anonymous, and expanded constants, static initialization, methods, closures, and thread-local state.
This repair changes no schemas, roles, target selection, lint levels, or exemptions. It does not establish downstream acceptance.

## Unreleased: Compiler-bound integration-test ownership

Compiler units now bind required source roots and retain fact and unknown membership before deduplication.
Only explicitly declared, compiler-bound independent integration tests receive test dispositions.
Library and binary test configurations retain production obligations. Production imports of test-labelled source still reject.

Production evaluation retains shared observations and validates reference closure after excluding test-only evidence.
Test-only witnesses, ports, types, no-std configuration, and unknowns cannot satisfy production requirements.
Full bundle replay compares the identity-bound disposition, not only receipt self-consistency.

Normalized IR, raw shard, and project receipt schemas advance to v2; collector context advances to v3.
Legacy inputs without required membership, root, or disposition fields reject rather than gaining default ownership.
The collector now reads rustc's parsed `NoStd` attribute instead of an unparsed attribute name.
Active crate configuration remains specific to each compiler unit. Inactive or integration-only attributes cannot satisfy production no-std obligations.
These checks do not authenticate compiler observations or establish downstream acceptance.

## 2026-09-13: Unprotected effect receipt correction

An unprotected-only architecture receipt no longer requires protected authorization fact and type identities.
Protected and mixed effect declarations still require both identity arrays.
Receipt identity, non-claims, compiler coverage, policy binding, and authority-promotion rejection remain enforced.
That correction kept receipt schema `octet-project-architecture-receipt/v1`.

## 2026-08-26: Compiler-visible authorized effect protocols

Project architecture policies can now classify outbound effect families as protected or unprotected. Protected protocols name exact authority inputs, decisions, opaque witnesses, executors, observations, receipts, use cardinality, source scopes, and non-claims.

Activated collection records selected type shapes, constructor safety, and bounded optimized-MIR provenance. Strict modes block public or replayable witnesses, undeclared or unsafe constructors, missing executor witness parameters, unguarded effect sites, unknown origins, and receipt promotion without an approved success observation. Passing evidence does not prove credential validity, revocation freshness, actor trust, external success, or receipt authenticity.

## 2026-08-19: Integrated project architecture gate

An activated policy now makes `cargo octet check` evaluate the resolved Cargo graph and compiler Architecture IR. Lint and architecture phases keep separate statuses, findings, evidence tiers, receipts, and non-claims. Input-integrity failures block every active mode. Policy findings block only `gate` and `admission`.

`admission` now binds the `deterministic-core` profile and one workspace-relative effect-closure policy. Octet runs strict lint, architecture evaluation, and closure in order. It emits separate linked receipts and rejects denied closure or tampered policy, graph, compiler, provider, command, and parent-status identities. Unconfigured projects keep their existing output bytes and emit no architecture or closure artifacts.

## 2026-08-12: Compiler-backed Architecture IR

Activated project policies now collect bounded rustc facts into content-addressed shards. Octet emits normalized Architecture IR, coverage, issue, and receipt sidecars only for selected production compilation units. Incomplete or stale evidence fails closed and keeps explicit non-claims.

## 2026-07-27: Opt-in project architecture policy

Projects can now select a typed Nickel FCIS and Hexagonal policy through `[workspace.metadata.octet].architecture_policy`. Activation verifies workspace-relative paths, the deterministic JSON export, contract identity, evaluator identity, and the BLAKE3 freshness manifest before Cargo runs.

Projects without the pointer retain existing lint and strict-profile behavior. They receive no project architecture diagnostics, receipt, or architecture-clean claim. Start with `inventory` mode and read `docs/project-architecture-policy.md` before promotion.

## 2026-07-26: Machine-readable assurance profiles

Octet now packages a closed Nickel assurance profile for the active OnixOS lifecycle proof chain. The profile links owners, protected properties, assumptions, trusted-base elements, required evidence, validation commands, and explicit non-claims across construction, semantic-design, verification, effect-control, and provenance-readiness planes.

`nix build .#octet-assurance-profile-onixos-lifecycle` exposes the authored profile, contract, generated JSON, and Nickel Export BLAKE3 manifest. `checks.x86_64-linux.assurance-profile-onixos-lifecycle` validates exact generation plus positive and negative fixtures. The profile declares requirements only; it does not report evidence success, cumulative proof, physical readiness, or release eligibility. See `docs/assurance-profiles.md`.

## 2026-07-15: Profile-bound Clippy restrictions

`cargo octet evidence collect --rail clippy-co-reporting --run` now accepts the embedded Nickel-authored `production-safety`, `canonical-determinism`, and `shell-boundary-inventory` restriction profiles through `--profile`; `--config PATH` may supply another checked catalog. Profiled runs construct fixed workspace/all-target Clippy argv, reject arbitrary commands, parse only Clippy JSON stdout, and bind full-catalog plus selected-profile BLAKE3 identities into the receipt.

The catalog deliberately cherry-picks restrictions and rejects the blanket `clippy::restriction` group. Clippy remains optional `static_lint` evidence complementary to Octet's semantic lints; passing does not prove behavioral correctness, panic freedom, memory safety, runtime determinism, compiler correctness, formal proof, whole-program correctness, or release eligibility. See `docs/clippy-restriction-profiles.md`.

## 2026-07-11: Fail-closed effect closure admission

`cargo octet admit --profile deterministic-core --closure` now runs the existing strict check and then computes conservative source-token effect closure over configured core paths. Selected functions bind deny-mode function addresses; unresolved calls require exact provider/version/source/features/target summaries from typed Nickel policy. Unknown effects, stale summaries, forbidden role edges, and provider drift deny admission.

Admission requires an explicit Cargo `--target TARGET` argument after `--`, preventing ambient Cargo configuration from making the direct check and recorded dependency graph disagree. The new `cargo octet admit verify` command checks stored receipt, complete strict-profile parent, status, summary, policy, dependency-graph, and provenance identities without rerunning Cargo. Closure receipts remain bounded reviewed evidence and do not claim complete name resolution, macro/trait/MIR closure, dependency implementation verification, runtime determinism, whole-program correctness, or release eligibility. See `docs/effect-closure-admission.md`.

## 2026-07-11: Strict Rust profile admission

`cargo octet check --profile <PROFILE>` now selects the Nickel-authored
`robust`, `bounded-systems`, or `deterministic-core` policy. Profiles strengthen
their required lints to `deny`, raise function-address checking to a declared
minimum, and bind the selected profile into BLAKE3 run metadata and provenance.

Unprofiled checks remain on `workspace-metadata`. `deterministic-core` requires
non-empty workspace-relative `core_source_scopes` and a reviewed deny-mode
function-address baseline. These profiles validate bounded direct source shapes;
they are not transitive effect proofs or release certificates. See
`docs/strict-rust-profiles.md`.

The packaged runner now carries its linker closure, so dependency-bearing and
proc-macro consumers can run from `nix run` without inheriting a consumer dev
shell. `concrete_construction_in_core` now classifies only direct constructor
calls and excludes local-domain plus `core`/`alloc`/`std` result types; this
removes duplicate parent-expression and local DTO false positives while
preserving external-constructor findings.

## 2026-05-05: Function fingerprint docs and checks

`cargo octet fingerprint` now has checked operator docs for the full
review lifecycle: human summaries, warning/metadata summaries, script-facing
JSON output, source-scanner boundaries, downstream scratch-consumer behavior,
human/JSON `--diff` parity, and deterministic review manifests. `cargo
octet check` outputs now include stable finding IDs plus replay metadata
for toolchain, config hash, and profile hash. Check runs can now write and gate
against `octet-baseline.json` acceptance files so existing warning-only debt
is explicit while new stable IDs fail CI deterministically.

Use one review command for these executable docs:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

That aggregate gate is also part of `nix flake check`. Focused checks such as
`fingerprint-json-output-docs`, `fingerprint-source-scanner-docs`, and
`fingerprint-docs-downstream-smoke` are intended for local debugging after the
aggregate gate identifies a failing slice.

## 2026-04-16: Starter profile expanded

Promoted four low-noise lint families into the starter rollout profile:
`explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
`negated_predicate`.

Consumers using the starter `disabled_lints` list from `templates/dylint.toml`
will see these lints activate after updating. Evidence and classification data
are in `docs/downstream-rollout-evidence.md`.

## 2026-04-15: `todo!()` moved from `no_panic` to `no_todo`

`no_panic` no longer covers `todo!()`. A new `no_todo` lint owns `todo!()`
detection, and `unjustified_no_todo_allow` requires justification for
`allow(no_todo)` escapes.

**Migration:** consumers with justified `todo!()` stubs under `allow(no_panic)`
must migrate to `allow(no_todo)` or `allow(slotcar::no_todo)`.

## 2026-04-15: Tool-scoped lint attrs

Lint attrs migrated from bare names to tool-scoped canonical names:

- `#[allow(slotcar::no_unwrap)]` instead of `#[allow(no_unwrap)]`
- `#![deny(slotcar::naming)]` for domain groups

Bare names still work through compatibility aliases. Standalone raw-rustc
fixtures need `#![feature(register_tool)]` and `#![register_tool(slotcar)]`.

Config keys in `dylint.toml` stay bare (`no_unwrap`, not `slotcar::no_unwrap`).

## 2026-04-14: First-class runner replaces shell wrapper

`cargo octet check` and `nix run .#cargo-octet -- check` are the
supported consumer entry points. Legacy `cargo slotcar` / `cargo tigerstyle` paths are compatibility aliases. The historical shell wrapper is retained only
in archived replay evidence.

New consumers should use `[workspace.metadata.octet] for runner policy
and `dylint.toml` for lint tuning. See `docs/consumer-guide.md`.

## 2026-04-14: Structured output

`cargo octet check` gained `--output-format` (json, sarif) and
`--artifact-dir` for machine-readable output and saved artifact bundles.

Exit codes: `0` clean/warning-only, `1` error-level findings, `2` integration
failures.

## 2026-04-13: Initial Aspen consumer rollout

First evidence-backed downstream rollout against the Aspen distributed systems
project. Established the starter rollout profile with `ambient_clock` and
`compound_assertion`. Evidence archived in
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`.
