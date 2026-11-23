# Design: Refresh the self-check warning budget for PR #38

## Context

`checks.x86_64-linux.cargo-octet-self-check` runs the packaged
`cargo-octet check` over the workspace with
`--baseline standards/self-check-warning-budget/octet-baseline.json`. On PR #38
at `113861db` it failed with 4,253 warnings, 3,030 accepted identities, 3,025
accepted-current, 19 new and 5 stale. The object-corpus rails in the same run
passed after the owner-approved refresh, so the self-check was the only failed
attribute.

The same run reproduced locally through
`nix build path:$PWD#checks.x86_64-linux.cargo-octet-self-check` with identical
counts.

## Decisions

### Decision: Fix new findings in the PR's own code

**Choice:** Change only the PR's new or newly extended code: rename three
boolean bindings to predicate form; make the summary and build-log frame helpers
return `Result<(), String>` so `writeln!` errors propagate instead of being
discarded or unwrapped; reserve the two marker collections; split
`summary_text`, `inspect_status` and the requirements subcommand match into
named helpers; write `crate::cli::RawLogRetention` at each use instead of
importing it; and pass card path inputs as `CardLocators`.

**Rationale:** These are mechanical, local, and preserve every emitted byte.
The emitted `summary.txt`, status and receipts are unchanged, so the
byte-reproducibility and fixture-parity tests keep their meaning.

### Decision: Accept `evaluate_contract`'s six parameters in the baseline

**Choice:** Leave the signature and record its `too_many_parameters` finding in
the refreshed baseline.

**Rationale:** The function verifies a contract and the exact bytes it was
parsed from against one source root and optional artifact base. A bundled
struct would make it easier to pass bytes that do not correspond to the parsed
value, which is the property the finding is about. The alternative rewrites 17
call sites, mostly tests, for no behavioural gain.

### Decision: Refresh through the documented write-baseline path

**Choice:** Regenerate the baseline with the same packaged runner and scope,
then update both totals in `flake.nix` and the documented stored-budget
paragraph.

**Rationale:** Stable identities change when a file or function length changes,
so the feature's five pre-existing findings (`parse_prove`, `execute_proof`, and
the three over-length files) must be replaced, not suppressed. The documented
gate requires zero new and zero stale identities against the refreshed file.

## Risks / Trade-offs

- The refresh accepts one new finding (`evaluate_contract`) and the replaced
  identities; each is listed in `evidence/finding-delta.md` so review can
  reject them.
- The self-check runs the packaged runner; local reproduction used the same
  derivation. GitHub CI remains a separate result.

## Verification

- `nix build path:$PWD#checks.x86_64-linux.cargo-octet-self-check --no-link -L`
  exits 0 with zero new and zero stale identities.
- `cargo test -p cargo-octet --lib -- prove:: artifact:: requirements::` passes.
- `cargo check -p cargo-octet --lib`, rustfmt on the five touched files,
  Cairn `validate` and the three gates pass.
- The evidence lists every added and removed identity.

## Non-claims

This is an advisory budget refresh, not a correctness, security, or release
claim, and it does not attest GitHub CI.
