# Octet 0.1.1-rc.1: candidate announcement draft

> Pending qualification and publication. Publish only after
> [launch-checklist.md](launch-checklist.md) and required owner approvals are
> complete. This is not a current qualified-release claim. The planned immutable
> tag is `v0.1.1-rc.1`; never move or reuse the existing `v0.1.0` tag.

Rust teams are accumulating more patches than they can review carefully. Some
of those patches are written by people. Some are written by coding agents. Both
arrive with the same question: what did we actually check before this reached a
release gate?

A green build is not an answer. A wall of lint output is not an answer either.
The reviewer needs to know which source scope ran, which policy applied, which
findings were suppressed, what artifact was produced, and whether that artifact
still verifies.

Octet is our answer: an evidence-backed checked path for agent-written and
human-written Rust systems code. Existing Rust enters the path without a
rewrite. The runner applies pinned static checks and checked-in policy, then
writes local evidence that a reviewer or CI gate can inspect.

## The checked path

```text
Rust source
  -> cargo octet check
  -> checked-in policy and config
  -> evidence artifacts
  -> artifact verification
  -> consumer CI or release gate
```

The default path is deliberately concrete. It covers static lint findings,
standards and policy checks, provenance, and saved artifact verification.
Optional proof rails can add a scoped `formal_proof` tier for a declared
property. No tier is silently promoted into a stronger claim.

## What is in the box

The candidate inventory contains 77 Dylint lints across six domains; validate
the exact qualified source before publication:

| Domain | Count | Focus |
| --- | ---: | --- |
| safety | 27 | panics, bounds, defaults, loops, queues, unsafe code, failures, durability claims |
| naming | 9 | predicates, acronyms, units, currency, paths, imports |
| performance | 2 | clones, allocation, ownership |
| structure | 21 | size, nesting, parameters, casts, APIs, cohesion |
| purity | 16 | clocks, environment, randomness, effects, allocation, verified purity |
| concurrency | 2 | lock order, task lifecycle |

Those checks are only one layer. The runner also provides:

- checked-in `dylint.toml` lint policy and `[workspace.metadata.octet]` runner
  scope;
- Nickel-authored standards policy and generated policy artifacts;
- three strict profiles: `robust`, `bounded-systems`, and
  `deterministic-core`;
- SARIF or JSONL findings, a human summary, status, and `provenance.jsonl`;
- `cargo octet artifact verify` for saved bundles;
- opt-in evidence rails and scoped proof receipts;
- stable exit status values for clean, finding, and integration outcomes.

The path works locally or inside consumer CI. There is no required
Octet-operated upload service. Consumers still own their CI logs, artifact
stores, and release decisions.

## Why evidence instead of another lint dashboard

The interesting question is not "did a linter emit a warning?" It is "what can
a reviewer defend?"

Octet keeps the answer in files. A run can write `status.json`, `summary.txt`,
`results.sarif` or `results.jsonl`, and `provenance.jsonl` to a local artifact
directory. The verifier recomputes saved sidecar identities and fails closed on
missing, stale, malformed, or inconsistent evidence.

That is a small idea with a large consequence: a release gate can point at an
artifact instead of a memory of a terminal session.

## A first checked patch

The walkthrough starts with an ordinary helper:

```rust
pub fn average_bytes(total_bytes: u64, completed_operations: u64) -> u64 {
    total_bytes / completed_operations
}
```

`completed_operations` can be zero. Octet reports `unchecked_division`. A
bounded fix keeps the logic in the core and names the invalid case:

```rust
pub fn average_bytes(
    total_bytes: u64,
    completed_operations: u64,
) -> Result<u64, AverageBytesError> {
    if completed_operations > 0 {
        Ok(total_bytes / completed_operations)
    } else {
        Err(AverageBytesError::NoCompletedOperations)
    }
}
```

This guarded example illustrates the intended functional-core fix; the
[walkthrough](../examples/first-checked-rust-patch/README.md) owns the actual
setup and capture contract. Generate, review, and commit the consumer's
`Cargo.lock` first. Capture the deliberately denied finding (exit `1`), the
same-scope/config guarded fix (exit `0`), valid bundles, and tamper rejection.
Do not relabel the old public-export walkthrough as this candidate's evidence.

The walkthrough then reruns the check and verifies the saved bundle:

```bash
cargo octet check --artifact-dir artifacts/octet --output-format sarif
cargo octet artifact verify --artifact-dir artifacts/octet
```

## What Octet is not

Octet is not a new language, a Rust replacement, or a certification program. It
does not prove whole-program correctness, provide a verified runtime, establish
C performance parity, or turn every lint finding into a formal counterexample.
It does not replace Clippy, Miri, Kani, Verus, or your dependency policy. It is a
checked path that keeps those inputs reviewable and bounded.

The Dylint and nightly-Rust tradeoff is real. Several checks need compiler
internals that stable plugin APIs do not expose. The canonical source-only Nix
path pins Rust, Dylint, Nickel, and support tools for `x86_64-linux`. Direct
Cargo use is an advanced developer path that must reproduce the same pins;
stable-only and other platforms are not supported release configurations.

Octet is licensed `AGPL-3.0-or-later`, and the public channel is source-only.
Teams that cannot accept those obligations should not adopt it.

## Install

After qualification and publication of the planned immutable candidate tag,
the evaluation command is:

```bash
nix run github:OnixResearch/octet/v0.1.1-rc.1#cargo-octet -- --version
```

This draft does not claim that the tag exists, that the command ran, or that
qualification passed. Record the actual version and resolved full revision in
the release record. The smoke capture helper requires that full revision, not
a tag. Then follow the consumer guide and walkthrough for lockfile preparation,
bounded scope, findings, artifacts, verification, and tamper rejection.

`OnixResearch/octet` is the sole canonical source/release/support home. The
old `octet-public` cohort remains blocked and unsupported, not a fallback route.
There is no crates.io, binary, hosted-service, or custom-website release path.

## Who should try it

Try Octet if you maintain Rust systems code, you already have a release gate,
and you want the gate to point at reviewable evidence rather than an ephemeral
log. Try it if coding agents are increasing patch volume and you need one
checked path for every promoted patch.

Do not start with Octet if you need stable-Rust-only tooling, zero
configuration, a hosted verification service, a whole-program correctness
certificate, or a permissively licensed dependency.

Start warning-only. Save artifacts. Inventory the findings. Promote one family
at a time. The archived Aspen phase-1 starter rollout produced six
`ambient_clock` findings on its three-package starter scope; that is bounded
historical evidence, not a promise about the noise level in another workspace.

## What comes next

The next step is not a larger claim. It is a smaller loop: run the first check,
verify the bundle, and tell us where the path was unclear. Issues that name the
revision, platform, command, configured scope, and expected/observed result are
the ones we can act on.

- Repository: <https://github.com/OnixResearch/octet>
- First checked patch: `docs/examples/first-checked-rust-patch/`
- Consumer guide: `docs/consumer-guide.md`
- Trust boundary: `docs/trust-boundary-evidence.md`
- Security policy: `SECURITY.md`

Evaluation should start with one owned crate and retain actual findings,
without a baseline, blanket lint disable, or zero-noise promise in the first
independent capture. Support is best-effort, with no response SLA or historical
pin LTS promise. Report redacted issues at
<https://github.com/OnixResearch/octet/issues>; use `SECURITY.md` for private
vulnerability handling, conditional on verified GitHub reporting availability.
