# Positioning and messaging

## Category

Octet is an evidence-backed checked path for Rust systems code. Existing Rust
source runs through static, policy, and optional proof checks, then produces
reviewable artifacts for a CI or release gate.

Octet is not a new language, a Rust replacement, a hosted verification service,
or a whole-program verifier.

## One-liners

| Length | Copy |
| --- | --- |
| 8 words | Reviewable evidence for Rust, from source to gate. |
| 20 words | Octet checks existing Rust with pinned policy and writes reviewable evidence before your CI or release gate. |
| 50 words | Octet is a Rust-compatible checked path for systems code. It runs 77 Dylint lints across six domains, applies checked-in policy, and writes local artifacts that a reviewer or release gate can verify. Optional proof rails stay explicitly scoped. |
| 100 words | Octet is an evidence-backed checked path for agent-written and human-written Rust systems code. Existing Rust source enters Octet checks without a rewrite. The runner applies Tiger Style lint families, checked-in Nickel policy, and optional scoped proof rails, then writes local evidence: status, findings, provenance, SARIF or JSONL sidecars, and a verifier for the saved bundle. Teams can start warning-only, inventory findings, and promote only the families they understand. Octet runs locally or in consumer CI, requires no Octet-operated upload service, and keeps every claim bounded to the command, source scope, configuration, artifact, and evidence tier that produced it. |

## Audience

| Priority | Segment | Trigger | Job to be done |
| --- | --- | --- | --- |
| Primary | Rust platform, infrastructure, and systems teams with a release gate | A review or audit asks what ran, against which policy, and what artifact proves it | Add a repeatable checked path without rewriting the workspace |
| Secondary | Teams using coding agents to produce Rust patches | Patch volume grows faster than review capacity | Make every promoted patch pass the same checked path and leave evidence |
| Tertiary | Safety-critical and formal-methods-adjacent teams | A scoped property needs verifier-backed evidence, not a blanket proof claim | Keep proof receipts separate, scoped, and reviewable |
| Not a fit | Teams that need stable-Rust-only tooling, zero configuration, or a whole-program correctness certificate | The Dylint/nightly tradeoff or bounded claims are unacceptable | Do not adopt Octet for that requirement |

## Problems

1. **Release evidence is scattered.** Console output, CI logs, SARIF, and proof
   notes do not share an identity or a scope.
2. **Policy is implicit.** Lint levels, suppressions, and architecture rules
   live in review comments instead of checked-in policy.
3. **Agent-written patches increase review load.** A green build says little
   about which source-shape boundaries were checked.
4. **Proof claims outrun their scope.** A fixture proof or a linked receipt can
   be repeated as a whole-program claim.
5. **Another tool can add friction.** Teams reject gates that require a rewrite
   or a hosted service.

## Value pillars

| Pillar | What the reader gets | Proof surface |
| --- | --- | --- |
| Rust-compatible | Existing Rust enters the checked path without a language migration | `cargo octet check`, README quick start, consumer guide |
| Evidence, not only output | Findings, policy results, provenance, and saved artifacts share a reviewable identity | `status.json`, `summary.txt`, `results.sarif` or `results.jsonl`, `provenance.jsonl` |
| Verifiable bundle | A saved artifact directory is checked before promotion | `cargo octet artifact verify --artifact-dir artifacts/octet` |
| Checked-in policy | Lint levels, profiles, and architecture policy are reviewable source | `dylint.toml`, `[workspace.metadata.octet]`, Nickel policy registry |
| Closed by default | APIs, effects, capabilities, proofs, and suppressions require a complete interface | Tiger Style lint families and suppression governance |
| Optional proof, bounded claims | Formal-proof rails are opt-in and scoped to a property, verifier, source binding, and receipt | `cargo octet prove`, proof cards, artifact verification |
| Local or CI | No Octet-operated upload service is required | controlled-deployment posture in `docs/trust-boundary-evidence.md` |

## Proof points

Use these only with their bounds.

| Proof point | Bound |
| --- | --- |
| 77 Dylint lints across six domains | Static source-shape checks, not whole-program correctness |
| Six domains: safety 27, naming 9, performance 2, structure 21, purity 16, concurrency 2 | Counts track the reviewed public inventory and can change between cohorts |
| Three strict profiles: `robust`, `bounded-systems`, `deterministic-core` | Profiles enforce direct source-shape policy; they do not prove transitive effects |
| Artifact bundle plus `artifact verify` | Verifies saved sidecar identities; not a release decision by itself |
| Nickel policy registry and generated bundles | Policy checks validate declared contracts; they do not validate business requirements |
| Optional Verus, Lean/Aeneas, Kani, and external evidence rails | Each rail proves only its named property and receipt scope |
| Public cohort `octet-public-becae962-2026-11-11` is qualified | Bounded to one source pin and its fresh receipts |
| Aspen phase-1 starter rollout produced six `ambient_clock` findings on three starter packages | Historical archived evidence for that scope; not Aspen HEAD and not a noise guarantee |

## Differentiation

| Alternative | What it does well | Where Octet differs | Use together? |
| --- | --- | --- | --- |
| Clippy | Broad compiler lint coverage and a familiar Rust workflow | Octet adds Tiger Style policy families, checked-in policy, evidence artifacts, provenance, and artifact verification | Yes; Octet does not replace Clippy |
| cargo-deny | Dependency, license, and advisory policy | Octet covers source-shape and policy evidence in addition to an optional dependency-audit receipt | Yes |
| Miri | Undefined-behavior detection for supported executions | Octet is static source-shape and policy evidence, not an interpreter | Yes, for different questions |
| Kani, Verus, Lean/Aeneas | Solver- or verifier-backed proofs for selected properties | Octet makes proof rails optional and keeps them scoped inside a broader evidence path | Yes; Octet does not replace a verifier |
| Dylint directly | A framework for custom Rust lints | Octet supplies the lint catalog, policy layer, runner, artifact format, profiles, and release discipline | Octet is built on Dylint |
| Bespoke CI scripts | Full local control | Octet replaces glue with a pinned runner, stable exit codes, replayable artifact bundles, and schema-checked policy | Yes, if scripts call the same runner |

## Objection handling

### "We already use Clippy. Why add Octet?"

Use both. Clippy answers compiler-lint questions. Octet answers a different
question: did this source pass the checked-in policy, which findings were
suppressed, what evidence did the run produce, and can a reviewer verify the
saved bundle? Clippy cleanliness and Octet cleanliness are both review inputs.

### "Dylint and nightly Rust are a hard sell."

That is a real tradeoff. Several checks need compiler internals that stable
plugin APIs do not expose. The supported public channel pins Rust, Dylint,
Nickel, and support tools through Nix so a consumer does not assemble that
matrix alone. Direct Cargo use is documented as an advanced path and must
reproduce the same pins. Teams that require stable-Rust-only tooling should not
adopt this cohort.

### "The AGPL is a problem for us."

Octet is `AGPL-3.0-or-later`, and the public channel is source-only. Do not
describe the project as permissively licensed or as a hosted service with no
source obligations. Teams that cannot accept AGPL obligations should not adopt
it.

### "Will this flood us with findings?"

Start warning-only, save artifacts, inventory findings, and promote one lint
family at a time. The archived Aspen phase-1 starter rollout produced six
findings, all `ambient_clock`, on its three-package starter scope. That is a
bounded historical example, not a promise about another workspace.

### "Does this prove our program is correct?"

No. The default path proves only that selected static and policy checks ran and
emitted reviewable evidence for their scopes. Optional proof rails can support a
`formal_proof` tier for a declared property, verifier, source binding, receipt,
and artifact identity. No evidence tier may be silently promoted.

### "Why not just use Verus or Kani?"

Use them when you have a scoped property and a verifier configuration. Octet is
the surrounding checked path: policy, lint evidence, provenance, artifact
verification, and a place to keep a proof receipt without relabelling every
lower tier as proof.

### "Why is there no crates.io package?"

The current distribution is a source-only public export and pinned Nix flake.
Workspace path dependencies, Git dependencies, and the runner's embedded build
inputs need a separate packaging decision. Do not treat `cargo package --list`
as proof that a standalone crate can be built or published.

## Message rules

Say:

- "evidence-backed checked path";
- "Rust-compatible";
- "reviewable artifacts";
- "bounded evidence tier";
- "local or consumer CI";
- "pinned source-only public channel";
- "optional scoped proof rails".

Do not say:

- "guarantees correctness" or "proves your program";
- "certified", "security score", or "release eligibility";
- "replaces Clippy" or "replaces Verus";
- "zero configuration" or "stable Rust";
- "C performance parity";
- "AI-safe" or "agent-proof";
- "77 guarantees";
- "hosted verification service".

## Naming

| Surface | Name |
| --- | --- |
| Product | Octet |
| Coding philosophy | Tiger Style |
| Runner | `cargo-octet` / `cargo octet check` |
| Canonical lint attributes | `tigerstyle::<lint>` |
| Public repository | `github:OnixResearch/octet-public` |
| Legacy compatibility names | `slotcar`, `tigerstyle`, `cargo-slotcar`, `cargo-tigerstyle` |

Tiger Style is the philosophy and lint namespace, not a separate product.
Legacy names route to the same engine.

## Landing-page outline

Use this structure for the public repository README or a future owned product page. Do not use an unowned domain as a call to action:

1. **Hero**: "Reviewable evidence for Rust, from source to gate."
   Subhead: "Octet checks existing Rust with pinned policy and writes artifacts
   your CI or release gate can verify."
   Primary CTA: "Run the first check." Secondary CTA: "Read the trust boundary."
2. **Problem**: agent-written and human-written Rust both need a defensible
   review path; logs and green builds are not enough.
3. **Checked path**: source -> `cargo octet check` -> policy -> artifacts -> gate.
4. **What ships**: 77 lints, six domains, three strict profiles, artifact
   verifier, optional proof rails.
5. **First patch demo**: the `average_bytes` zero-guard example from
   `docs/examples/first-checked-rust-patch/`.
6. **What Octet does not claim**: bounded evidence tiers, not whole-program
   proof or certification.
7. **Install**: the pinned public Nix command.
8. **Case study**: Aspen phase-1 starter rollout with its scope boundary.
9. **FAQ**: Clippy, Dylint/nightly, AGPL, data locality, false positives, proof.
10. **Final CTA**: start warning-only, save artifacts, verify the bundle.
