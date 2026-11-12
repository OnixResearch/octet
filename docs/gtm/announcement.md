# Announcing Octet 0.1.0

> Draft launch post. Publish only after
> [launch-checklist.md](launch-checklist.md) is green. Keep the claim
> boundaries; do not turn this into a whole-program verification or
> AI-safety post.

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

Octet 0.1.0 ships 77 Dylint lints across six domains:

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

The reviewed public walkthrough uses the guarded `if`/`else` form because that
public source pin recognizes an enclosing zero guard. The newer development lint
also accepts an early return, but the public bytes are intentionally pinned.
This is the kind of distinction the evidence path is supposed to keep visible.

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
internals that stable plugin APIs do not expose. The supported public channel
pins Rust, Dylint, Nickel, and support tools through Nix so consumers do not
assemble that matrix alone. Direct Cargo use is an advanced path that must
reproduce the same pins.

Octet is licensed `AGPL-3.0-or-later`, and the public channel is source-only.
Teams that cannot accept those obligations should not adopt it.

## Install

The supported public channel is pinned to a reviewed source-only export:

```bash
nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version
```

Expected:

```text
cargo-octet 0.1.0
```

Then copy `templates/dylint.toml` into a workspace, add the runner scope, and
run the same pinned command with `check`. Pin the full revision; `main` is not a
supported pin.

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

- Repository: <https://github.com/OnixResearch/octet-public>
- First checked patch: `docs/examples/first-checked-rust-patch/`
- Consumer guide: `docs/consumer-guide.md`
- Trust boundary: `docs/trust-boundary-evidence.md`
- Security policy: `SECURITY.md`

If you want to evaluate Octet on a real workspace, bring one crate with a
release gate and a bounded scope. We will start with warning-only evidence and
keep the claim boundary visible.
