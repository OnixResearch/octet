# Valence core cutover

<!-- r[impl octet.valence_project_extraction.phase5.consumer_migration_docs] -->
<!-- r[verify octet.valence_project_extraction.phase5.hardened_upstream_dependency] -->
<!-- r[verify octet.valence_project_extraction.phase5.exact_source_pin] -->

Octet consumes standalone Valence as the canonical owner of Valence pure evidence semantics. Production calls route through `cargo-octet/src/valence_core_shim.rs` to this exact dependency:

```toml
valence-core = { git = "ssh://git@github.com/OnixResearch/valence.git", rev = "7546a78b4eeef8686c6837e187aa12420f4f03bb", package = "valence-core" }
```

The selected revision contains `cairn/archive/2026-07-12-harden-preserves-integrity-boundaries`, including the corrected Preserves integrity and typed claim-boundary profile. The Nickel-authored cutover manifest at `standards/valence-core-cutover/manifest.ncl` binds that revision, Cargo source identity, archive path, module source hashes, fixture commands, and non-claims. Its generated runtime projection is `standards/valence-core-cutover/generated-manifest.json`.

The old Octet-hosted source remains only at `crates/valence-core/` as the non-publishable, non-workspace package `octet-valence-legacy-fixture`. It is a bounded positive/negative compatibility corpus and cannot satisfy a canonical `valence-core` dependency.

The selected upstream `function_object.rs` remains at BLAKE3
`11f0f4233aaf3153966f6aa37de570c2dcd0e75335591934461ae2b25ab3ecb1`.
The active `source-token-v2/candidate-source-manifest.json` is the exact
`0.1.1-rc.1` candidate, BLAKE3
`be3647f9c72a598defa192f4ff0cda440fef22eeb5e2bc7dd6c162abce43d862`.
Its frozen `PENDING`, unselected and null-approval fields are preserved. A
separate reviewed manifest, BLAKE3
`63a4cc8c1e5dbdda1e74182ebb13f830d33772dc97636143b82cc400dcf94961`,
selects it under the direct native
[`RC owner decision`](../.cairn/archive/1970-01-01-resolve-open-assurance-issues/evidence/rc1-import-clean-owner-approval.txt).
The owner approved that candidate and audit envelope
`5892eae1dfa1658381e95684243a252bc8757dad6e21ef14d3950bfadd0d49fc`,
including exact 40 source-byte hashes, 18/39-path selections, three producer
modules, the captured consumer-test snapshot, immutable Valence identity and
four references with object counts 1/1,155/1,764/1,764. The fixed receipt uses
fixture-relative `src/lib.rs`.

The active fixture directory retains the exact approved audit envelope and
original producer source-binding document (BLAKE3
`a3ab4aa9674aecd8ec9648dc436b285ebfff1288a78c94414d561c97acb8d2f4`).
The latter binds the original immutable preflight producer to 42 unique matching
source/module/test paths and the compiled Valence module. It resolves the
candidate's preparation-time unknown source derivation for that original
producer only; it does not establish final H3/release-source qualification.
Neither candidate bytes nor earlier CLI captures are relabelled as a later build.

Native lifecycle closure moved the owner record to the archive without changing
its bytes or the sealed candidate/review/reference records. The reviewed
manifest retains its original pre-archive `evidence_path`; admission reads only
the archived owner record and enforces its unchanged BLAKE3 identity.

The prior `0.1.0` candidate, reviewed manifest and four references remain
byte-preserved and unselected in `source-token-v2/historical-0.1.0/`. The old
archived OpenSpec approval is historical only. Existing named object rails
select only the current tuple and reject missing/altered owner evidence,
candidate/source/reference bytes, audit/source-binding artifacts, producer
version or Cargo/Nix Valence identity. Two independent fresh-build binaries
must each reproduce the saved internal and expanded references; matching one
another is insufficient. Report PASS only after the actual derivation builds.

The verifier checks exact hashes and scoped structured bindings, not approval
prose phrases. Its code and expected digests remain co-located in the same
PR-controlled tree: an in-repository PASS is not authentication against an
adversarial simultaneous edit. Compare the exact candidate/audit identities
with the actual owner decision outside that edit. Approval excludes future
consumer-test code, other future edits, warning-budget changes, correctness,
semantic equivalence and release eligibility. Re-review future producer/test
changes; do not regenerate a current expected reference merely to obtain PASS.

The shared scanner was corrected to balance
outer-attribute brackets through literal/comment content and capture split
visibility/modifiers. A rustc ABI repro showed that distinct multiline
export-name attributes previously yielded one object hash; `pub` on its own
line also previously collided with a private function of the same body.
Lexical marker association now refuses a matching reviewed `// octet-object:`
inside a block comment and marker-like fingerprint comments/attributes inside
non-code; a real marker immediately before outer attributes still attaches.
The current caller requests the v2 label through Valence's typed constructor;
the selected pin emits a source-backed v2 object. The historical v1 corpus
receipt stays readable but reports an `object_set_hash` mismatch under default
v2 source replay. Neither a passing self-produced v2 replay nor a local
diagnostic clears the existing Nix rails without independently bound sources
and saved references.

## Consumer migration

Downstream consumers such as Aspen must replace the Octet-hosted package with the exact standalone dependency above. A local `../valence/crates/valence-core` path may be used only as an explicit development diagnostic; it is not release evidence and must not change the manifest or lock identity.

After changing the dependency, run:

```bash
nix develop -c cargo test -p cargo-octet valence_cutover -- --nocapture
nix develop -c cargo test -p cargo-octet evidence_index -- --nocapture
nix develop -c cargo test -p cargo-octet object -- --nocapture
```

Passing these checks proves selected dependency/source ownership, unique canonical package identity, and compatibility for the declared Function Object, evidence-index, and evidence-codebase-index fixtures only. It does not prove behavioral correctness, arbitrary semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, release eligibility, or whole-program safety.
