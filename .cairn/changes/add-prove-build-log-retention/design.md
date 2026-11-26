# Design: Keep raw prove logs out of the bundle with a build-log retention mode

## Context

`prove::write_artifacts` writes each rail's captured stdout/stderr under
`proofs/` and lists them in `evidence-log-manifest.json`; `summary.txt` repeats
their BLAKE3 identities. The Aeneas consumer job copies the whole tree, so
timing-dependent tool output changes the derivation output. `artifact verify`
currently assumes every prove bundle that retains logs has a matching manifest.

## Decisions

### Decision: Retain mode is an explicit, default-preserving option

**Choice:** `--raw-log-retention bundle|build-log` on `cargo octet prove`; the
default is `bundle`, and `status.json` omits the field in that mode while
`build-log` records `raw_log_retention: build-log` and
`non_claims: [...]`.

**Rationale:** Existing prove consumers and golden bundles must keep their exact
bytes, so the new field is serialized only when it is not the default. The
explicit value also lets `artifact verify` distinguish a deliberate build-log
bundle from a truncated one.

### Decision: Frame on stderr, do not normalize

**Choice:** In `build-log` mode, print each stage capture to cargo-octet's
stderr between stable `=== cargo-octet prove raw log rail=... stage=... stream=...
===` and `=== end ... ===` lines. Aeneas captures are split on their existing
`[charon]`/`[aeneas]`/`[lean]` stage markers; other rails use one `command`
block.

**Rationale:** The spec requires the raw output to land in the build log of the
job runner. Framing keeps interleaved stages attributable without altering the
text, and splitting on markers already emitted by the capture avoids changing
the execution layer.

### Decision: Verification reads the declared retention mode

**Choice:** `artifact verify` parses `raw_log_retention` from `status.json`.
`build-log` bundles are valid without log files or a manifest, but a manifest or
any `proofs/*.stdout|stderr` file contradicts the declaration and is rejected.
`bundle` mode keeps the existing manifest-driven file-set checks.

**Rationale:** The mode is a claim in the bundle; verification must check the
bundle shape against that claim in both directions. No new requirement is placed
on existing `bundle` bundles, which preserves current tests.

### Decision: Receipt-level log identities become non-claims in build-log mode

**Choice:** The Verus receipt schema requires `stdout_artifact` and
`stderr_artifact`. In `build-log` mode those identities are stable
`build-log/<policy>-rail-<N>.<stream>` markers whose digest covers only the
marker, never the raw bytes. Aeneas receipts never bound raw logs.

**Rationale:** The bundle must not hash raw text. A constant marker satisfies the
receipt contract without binding nondeterministic diagnostics.

## Risks / Trade-offs

- A `build-log` bundle no longer carries the raw logs, so a reviewer must read
  the job's build log; `status.json` states this non-claim explicitly.
- Aeneas stage splitting relies on the capture's own stage markers; a capture
  without markers falls back to one `command` block rather than dropping text.
