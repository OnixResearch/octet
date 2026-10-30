# Changelog

Notable changes that affect consumers. Organized by date since consumers pin to
specific commits via `TIGERSTYLE_DYLINT_REV`.

For the full lint catalog, see `docs/lint-reference.md`.
For rollout profiles, see `docs/downstream-rollout.md`.

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
