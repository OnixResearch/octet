# Proposal: Keep raw prove logs out of the bundle with a build-log retention mode

## Why

`nix/mantle-aeneas-job.nix` is not byte-reproducible. Two builds of the same
derivation differ only in the raw tool logs `cargo octet prove` writes into the
bundle: Lake progress lines and durations, Charon/Aeneas total execution times
and Cargo's `Finished ... in <seconds>s`. Those diagnostics vary with
scheduling, and the evidence-log manifest, provenance, summary and every
consumer file that copies the bundle then differ too.

Nondeterministic diagnostics do not belong in a content-addressed output. The
default prove bundle must not change for other users, so the retention mode is
opt-in.

## What Changes

- Add `cargo octet prove --raw-log-retention bundle|build-log`. `bundle` is the
  default and preserves today's bundle bytes exactly.
- In `build-log` mode, write each rail's raw stage stdout/stderr to
  cargo-octet's stderr, framed by rail, stage and stream; write no
  `proofs/*.stdout|stderr` files and no `evidence-log-manifest.json`; and record
  `raw_log_retention: build-log` plus a non-claim in `status.json` and
  `summary.txt`. Nothing in the bundle hashes the raw text.
- Keep every consumer of the in-memory capture unchanged: pass signals,
  truncation flags, exit statuses, the `lake build` `sorry` scan and the Lean
  axiom check still run, and their verdict stays in the receipt.
- Teach `cargo octet artifact verify` to accept a `build-log` prove bundle and
  reject a bundle that claims `build-log` yet contains raw logs or a manifest;
  `bundle` mode behavior is unchanged.
- Pass `--raw-log-retention build-log` in `nix/mantle-aeneas-job.nix` and keep
  its failure path showing the raw logs, which are now already in the build
  log.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/prove.rs`,
  `cargo-octet/src/artifact.rs`, `nix/mantle-aeneas-job.nix`,
  `docs/aeneas-consumer-contract.md`, `CHANGELOG.md`, and this change package.
- **Testing**: focused `cargo test -p cargo-octet` prove/artifact tests, the
  repository's required checks for this surface, `nix eval --raw
  .#octet-mantle-aeneas-consumer-job.drvPath`, and the Cairn gates.
- **Compatibility**: `bundle` remains the default; existing prove bundles and
  their tests stay byte-identical.

## Non-goals

- No normalization or filtering of log text.
- No change to receipt, theorem-admission or trust policy.
- No rebuild of the real consumer job in this change; reproducibility is
  verified by the orchestrator with `nix build --rebuild`.
