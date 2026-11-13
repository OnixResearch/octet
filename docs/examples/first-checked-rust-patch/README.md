# First checked Rust patch

This is the operator walkthrough for the **0.1.1-rc.1 candidate** from
<https://github.com/OnixResearch/octet>. Initial support is source-only Nix on
`x86_64-linux`. It exercises the actual packaged `cargo-octet`, not a fake Cargo
fixture or a copied historical wrapper. No crates.io installation, binary
distribution, or custom website is required.

**Qualification is pending:** this document does not record a successful run.
The release owner must supply the matching packaged executable and, for public
source-bound evidence, the qualified immutable public commit after source
freeze. A release tag is a discovery label, not an immutable proof pin. Do not
reuse the blocked historical `octet-public` source or its old receipts.

## Run the complete capture

From this checkout, use [`tools/public-release-smoke.py`](../../../tools/public-release-smoke.py).
Prerequisites are Python 3.9+, Git, `b3sum`, Nix with flakes enabled, and a Nix-store Cargo
executable for generating the standalone consumer's lockfile. Obtain Cargo from
the matching release devshell; the packaged runner supplies its own pinned
compiler, driver, and lint library for the actual checks. The helper rejects
non-Nix-store executable paths and rejects an inherited ancestor `.cargo/config`
or `.cargo/config.toml` around the capture root.

Set `OCTET_RUNNER` to the exact packaged `/nix/store/.../bin/cargo-octet` selected
by the release owner, and `OCTET_CARGO` to the matching Nix-store Cargo executable.
These are required real inputs, not paths to a checkout build or fabricated
sample output:

```bash
: "${OCTET_RUNNER:?set the matching packaged cargo-octet path}"
: "${OCTET_CARGO:?set the matching Nix-store cargo path}"
capture_parent=$(mktemp -d /tmp/octet-public-smoke.XXXXXXXX)
python3 tools/public-release-smoke.py \
  --runner "$OCTET_RUNNER" --cargo "$OCTET_CARGO" \
  --output "$capture_parent/capture" --downstream valence
```

Alternatively, set `OCTET_PUBLIC_FLAKE` to
`github:OnixResearch/octet/` followed by the full qualified lowercase 40-hex
commit and `#cargo-octet`. The helper builds that exact source package without
creating a result symlink, then executes the resulting packaged tool:

```bash
: "${OCTET_PUBLIC_FLAKE:?set the qualified immutable public cargo-octet flake ref}"
: "${OCTET_CARGO:?set the matching Nix-store cargo path}"
capture_parent=$(mktemp -d /tmp/octet-public-smoke.XXXXXXXX)
python3 tools/public-release-smoke.py \
  --flake "$OCTET_PUBLIC_FLAKE" --cargo "$OCTET_CARGO" \
  --output "$capture_parent/capture" --downstream valence
```

`--flake` rejects branches, tags (including `v0.1.1-rc.1`), mutable paths,
short revisions, and other repositories. `--runner` establishes a local immutable
package identity only: retain the release owner's source/build receipt to bind
that path to the public source. A version string alone cannot establish that
binding. Cargo defaults to the Nix-store `cargo` on `PATH` if `--cargo` is omitted.

Every invocation requires a **new** output directory. Each run isolates `HOME`,
Cargo/Rustup homes, XDG roots, temporary files, and per-check target directories
under that capture. It drops ambient Rust/Dylint/Octet flags and credentials;
the packaged wrapper still supplies its own execution inputs. The consumer is
fresh even when the operator runs the helper from inside Octet. Capture files
are retained on failure as well as success; a failed prerequisite is never
converted into a passing smoke.

## What the fresh consumer actually checks

The helper creates an independent crate with an explicit `[workspace]`, commits
its manifest, `dylint.toml`, source, and real `cargo generate-lockfile --offline`
output, and checks its library with `--locked`. Generate and commit a consumer's
`Cargo.lock` **before** checking: Octet detects lockfile creation or mutation,
restores the original state, and reports integration failure rather than
silently accepting a changed dependency graph.

The consumer owns this lint configuration; other defaults are not disabled:

```toml
[octet]

[octet.lint_levels]
unchecked_division = "deny"
```

Its first committed function is intentionally unsafe for a zero divisor:

```rust
pub fn average_bytes(total_bytes: u64, completed_operations: u64) -> u64 {
    total_bytes / completed_operations
}
```

The first check must exit **1**, emit an error-level `unchecked_division`
finding, and have no other error-level finding. This is the intended failing
lint gate, not a compiler/setup success invented by the helper. The Rust-emitted
bundle is retained and independently verified even for this failing lint run.

The helper then commits this genuine zero-guard repair, with unchanged manifest,
lockfile, lint policy, and check scope:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AverageBytesError {
    NoCompletedOperations,
}

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

The second check must exit **0**, finish Cargo successfully, and emit **zero
selected `unchecked_division` findings**. Any other default warning is retained
and counted; this is not a promise that every lint family is silent. No finding
baseline, global cap, broad disable, or warning suppression makes the fix pass.

## Real independent downstream inventory

`--downstream valence` fetches the existing public [Valence workspace at
`e250f498a86ac1e62266b8df749ade021a4b4043`](https://github.com/OnixResearch/valence/tree/e250f498a86ac1e62266b8df749ade021a4b4043)
into a separate isolated clone. It uses Valence's already-owned default scope:

```text
-p valence-shared-identity-pilot -- --lib --locked
```

That real package's library points at
`crates/valence-core/src/shared_identity.rs`; the committed `dylint.toml`
declares the matching core scopes. The helper preserves the workspace manifest,
`Cargo.lock`, lint policy, and tracked source bytes. It neither patches the
downstream repository nor replaces its policy with Octet's synthetic smoke
policy. Findings are an actual bounded warning inventory, not an asserted
zero-noise result, a strict-profile admission, or permission to ignore debt.

Before invoking Octet on this workspace, the helper runs anonymous
`cargo fetch --locked` with the isolated Cargo home and no credential settings.
This captures public acquisition of the **complete committed dependency graph**,
including the pinned schema-identity source on `seed.radicle.garden`. A missing
or inaccessible public dependency fails the acquisition step; it is not an
Octet lint finding, engine failure, or clean run. There is no source substitution,
private mirror, dependency edit, or fallback. A subsequent Octet exit **1** or
**2** also fails this warning-only inventory rather than weakening the policy.

Omitting `--downstream valence` explicitly leaves downstream use unexercised.
Use the complete command above for release-readiness capture.

## Inspect and independently verify the evidence

The authoritative artifacts are the Rust-emitted bundles under
`capture/bundles/fresh-bad`, `fresh-fixed`, and, when requested,
`valence-warning-inventory`. Each includes `command.txt`, `status.json`,
`summary.txt`, `results.jsonl`, and `provenance.jsonl`, plus any additional
Rust-owned sidecars. The helper never regenerates or relabels these receipts.
It checks the process/status counts against the actual JSONL records, asserts
the emitted tool version, and requires JSONL bytes to match captured stdout.

Every command also has its exact argv/cwd, exit status, raw stdout, and raw
stderr in a numbered `capture/commands/` directory. `environment.txt` retains
the isolated launch environment. `operator.log` records the helper invocation,
input identities, actual per-lint counts, and assertions only after they pass.
These are raw operator logs, **not** a new Python-owned stable receipt schema.
The fresh Git repository retains both commits and the captured zero-guard diff.

For every original bundle the helper executes the packaged command:

```bash
: "${OCTET_RUNNER:?use the exact packaged runner path recorded in operator.log}"
BUNDLE="$capture_parent/capture/bundles/fresh-fixed"
"$OCTET_RUNNER" artifact verify --artifact-dir "$BUNDLE" --output-format json
```

Verification must exit **0**, report `valid`, have no diagnostics, and leave the
bundle bytes unchanged. The helper also copies the fixed bundle, deliberately
appends JSON whitespace to the copied `results.jsonl`, and verifies that copy.
That command must exit **1**, report `invalid`, identify the `results.jsonl`
hash mismatch, and leave both the negative control and original evidence
unchanged. A malformed JSON parser failure alone is not the negative-control
claim. Keep the tampered bundle separate and do not promote it as evidence.

For automation, check exits are **0** for clean/warning-only results, **1** for
error-level lint findings, and **2** for integration/configuration/underlying
Cargo failures. Artifact verification uses **1** for invalid evidence. The helper
itself exits **0 only when all requested assertions pass**; its intentionally
failing check and tamper commands must return their exact expected nonzero
statuses. Do not use `|| true` around the capture.

## Claim boundaries

This establishes only observed static lint behavior for the named source/scope
and integrity of saved evidence bytes. It is not a solver counterexample,
mathematical proof, whole-program correctness, certification, performance parity,
a verified runtime, or release eligibility. It does not introduce `requires` or
`ensures` syntax or replace Rust. Artifact integrity is not signer authentication
or the truth of the analyzed program. The independent downstream keeps its own
review, lifecycle, and release authority.

For production configuration, rollout ownership, and CI storage, continue with
the [consumer guide](../../consumer-guide.md).
