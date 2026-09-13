# Verification evidence

## Scope and Baseline

Base revision: `42dc32a6fe921d35182d7aef03c2a83999d55771`.
All lifecycle commands used the dedicated worktree as their explicit root.
The proposal, design, and tasks gates passed before core edits. Repository validation passed with five active changes.

The focused baseline passed 12 tests and skipped one explicitly ignored compiler control.
The final runner suite passed 472 tests, with zero failures and nine ignored tests.
The compiler controls are separate from that ordinary suite and require explicit execution.
Strict Clippy passed for `cargo-octet` across all targets and all features. Rust and Nix format checks passed.

The first baseline attempt could not authenticate an HTTPS dependency fetch.
The corrected run used an isolated Cargo home with the Nix-owned vendor configuration.
No user Cargo configuration changed. The root lockfile and toolchain pins remain unchanged.
A later development-shell build was stopped to avoid rebuilding unchanged tool packages during local iteration.
Local commands then used the immutable base development shell against the current worktree source.

```sh
nix develop 'git+ssh://git@github.com/OnixResearch/octet?rev=42dc32a6fe921d35182d7aef03c2a83999d55771' -c cargo test --locked -p cargo-octet --lib
nix develop 'git+ssh://git@github.com/OnixResearch/octet?rev=42dc32a6fe921d35182d7aef03c2a83999d55771' -c cargo clippy --locked -p cargo-octet --all-targets --all-features -- -D warnings
nix develop 'git+ssh://git@github.com/OnixResearch/octet?rev=42dc32a6fe921d35182d7aef03c2a83999d55771' -c cargo fmt --all -- --check
nixfmt --check flake.nix
```

These commands used process-local `TMPDIR`, `CARGO_HOME`, and `CARGO_TARGET_DIR` values.
They cleared ambient Rust wrappers. Exact environment setup and logs remain in the retained operator evidence.

## Compiler Controls

The `compiler-architecture-targets` Nix check now includes all architecture tests and explicitly includes ignored tests.
The current collector compiles the controlled library, binary, test, example, benchmark, procedural macro, and build-script targets.
Both explicit-feature and all-feature controls observed complete coverage of ten units.
Both controls removed one actual shard and asserted incomplete coverage.
The all-feature control also rejected an under-declared row before any collector context existed.

```sh
nix build .#checks.x86_64-linux.compiler-architecture-targets -L
```

The first Nix run passed all 20 selected tests without skips.
The final rerun passed all 21 selected tests without skips, including the additional dependency-order regression.
The `project-architecture-policy` check passed for the existing policy exports and generated compiler contract.
No emitted DTO, generated contract, lint catalog, or architecture mode changed.

## Full Deny-All Consumer Probe

The probe reused the isolated Noble compatibility fixture, not Noble production source.
It contains a `production` feature, a pure budget function, and a valid architecture policy in `gate` mode.
It has no waivers or warning budget.
The complete policy row declares `production`. The negative row declares no features.
Both policies used valid pinned Nickel exports and freshness manifests.

The real hook used the Nix-built runner through its supported installed-tool path:

```sh
OCTET_PRECOMMIT_USE_INSTALLED=true "$ROOT/hooks/octet-deny-all.sh" --workspace -- --all-targets --all-features
```

`PATH` selected the runner from the checked Nix build and the pinned Rust toolchain.
The positive run also requested an artifact directory. Each run used a fresh target directory.

| Control | Observed result |
|---|---|
| Complete row | Exit 0, zero lint findings, zero architecture findings |
| Under-declared row | Exit 2, no collector context |
| Stored positive artifacts | `cargo octet artifact verify` passed |

The negative diagnostic was:

```text
error: Cargo --all-features plan does not match the explicit architecture policy row
```

The positive architecture receipt was:

```text
ade31ee47217e4f743c8c8dcdf385fecc77a3f0276059274aee3288dddcdea22
```

This result does not establish Noble workspace coverage, extraction, Lean compilation, Miri coverage, or M1 completion.

## Adversarial Review

The review ran serially in one session. It is correlated review, not independent-agent evidence.

Manifest enumeration was rejected because it cannot own Cargo feature resolution.
Metadata-only comparison was rejected because it does not preserve per-unit host and test resolution.
Complete plan comparison retains unknown Cargo fields, dependency edge properties, and selected roots.

Controls reject changed features, dependency-only changes, target descriptions, roots, invalid indices, duplicate identities, and missing fields.
Controls accept reordered equivalent units, features, roots, and dependency edges.
The actual Cargo fixture includes workspace-qualified features, optional dependencies, and a feature-gated binary.
Its failing build script remains planned but never executes.
Disabled automatic defaults still require an explicit row that matches the actual resolved `default` feature.

Existing package, target, default, explicit-feature, and missing-shard checks remain active.
The original invocation remains in compiler identities. No schema or evidence-tier promotion occurred.

## Retention

Raw logs, source fixtures, policies, Nix output roots, and artifact bundles remain outside the disposable worktree:

```text
/home/brittonr/git/octet/.pi/architecture-all-features-1629/
```

Key files include `baseline-vendored.log`, `runner-quality-final.log`, `nix-compiler.log`, `nix-final.log`, `probe-positive.log`, and `probe-negative.log`.
The positive and negative hook tasks were 1662 and 1664. Artifact verification was task 1669.
The final compiler and policy check task was 1666.
The search used one implementation and at most two concurrent processes. No code correction was needed after the first completed focused test run.

The inspected sync plan had no blockers and added the three declared requirements to the accepted specification.
Sync execution and repository validation passed. The inspected archive plan had no blockers, and archive execution passed.
Post-archive validation found four other active changes and no issues.
Publication remains a separate postcondition. Lifecycle and publication receipts remain with the operator evidence.
