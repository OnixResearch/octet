# Function fingerprints

Function fingerprints are selective review gates for functions whose exact body was
reviewed, verified, or certified. They are not a workspace-wide style rule and they
do not prove correctness.

Use fingerprints when a small number of functions carry review evidence that should
fail loudly if their signature, attributes, or body drift after review. Do not use
them as a replacement for tests, proofs, or normal code review.

## Marker forms

Comment markers work in stable Rust without adding an attribute macro:

```rust
// tigerstyle-fingerprint: b3:<hash>; review=safety-critical; evidence=openspec/.../evidence/check.txt
fn checked_transition(input: State) -> State { /* ... */ }
```

Attribute markers are also recognized by the scanner:

```rust
#[fingerprint("b3:<hash>", review = "safety-critical", evidence = "openspec/.../evidence/check.txt")]
fn checked_transition(input: State) -> State { /* ... */ }
```

The implementation hashes normalized function tokens and ignores the marker itself.
Formatting and comments do not change the hash; signature, attribute, visibility,
or body token changes do. Macro calls produce a review warning unless the marker
records an explicit macro review allowance such as `macro=allow`.

## Operator workflow

Start in a scratch copy or review branch. `--write` and `--update` rewrite source
markers, so do not run them against a production checkout unless you intend to keep
those marker changes.

1. Add fingerprints to selected reviewed functions:

   ```bash
   cargo tigerstyle fingerprint --write path/to/file.rs
   ```

   Use this only for initial marker creation. It discovers functions in the target
   file or directory and writes markers for functions that do not already have one.

2. Gate reviewed functions in CI or during review:

   ```bash
   cargo tigerstyle fingerprint --check path/to/file-or-directory
   ```

   `--check` inspects marked functions only. It exits nonzero when any marked
   function is missing a hash, mismatches its stored hash, lacks required metadata,
   or has an unallowed macro warning.

3. Preview drift without rewriting markers:

   ```bash
   cargo tigerstyle fingerprint --diff path/to/file-or-directory
   ```

   `--diff` has the same failure conditions as `--check`, but is intended for
   operator diagnosis before deciding whether the source change is legitimate.

4. Repair accepted drift after re-review:

   ```bash
   cargo tigerstyle fingerprint --update path/to/file-or-directory
   ```

   `--update` rewrites only stale or missing marker hashes. Treat each `written`
   record as a review decision: the new function body is now the reviewed body.

## Human output

Human output is the default and is designed for terminals, PR comments, and simple
CI logs:

```text
tigerstyle fingerprint check
src/lib.rs checked_transition mismatch stored=b3:old computed=b3:new review=safety-critical evidence=openspec/.../evidence/check.txt macro_warning=false
summary total=6 match=5 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

Use the summary line for quick operator decisions:

- `mismatch > 0`: a reviewed function changed; inspect the record line and rerun
  `--diff` or re-review before `--update`.
- `missing > 0`: a marker is incomplete or a write/update flow did not finish.
- `written > 0`: `--write` or `--update` changed marker hashes; review and commit
  the source diff with the evidence that justified it.
- `macro_warnings > 0`: a marked function uses macro calls without explicit review
  allowance; either avoid fingerprinting that function or add a deliberate
  `macro=allow` marker after reviewing the macro expansion risk.
- `metadata_missing > 0`: add `review=...` and `evidence=...` metadata before using
  the marker as a review gate.

A clean check has every marked function in `match` status and a summary like:

```text
summary total=12 match=12 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## JSON output

Use JSON for scripts that need exact counts or per-record assertions:

```bash
cargo tigerstyle fingerprint --check path/to/file-or-directory --output-format=json
```

The JSON report contains the mode and one record per marked function, including the
path, function name, stored hash, computed hash, status, review metadata, evidence
metadata, and macro-warning flag. Common script checks are:

```bash
# Count mismatched functions.
cargo tigerstyle fingerprint --check src --output-format=json \
  | jq '[.records[] | select(.status == "mismatch")] | length'

# List changed functions by path/name.
cargo tigerstyle fingerprint --diff src --output-format=json \
  | jq -r '.records[] | select(.status != "match") | "\(.path) \(.name) \(.status)"'

# Assert an update repaired exactly one accepted drift.
cargo tigerstyle fingerprint --update src --output-format=json \
  | jq '[.records[] | select(.status == "written")] | length == 1'
```

## Source-scanner boundaries

The fingerprint command is a source scanner, not a macro-expansion analyzer.
It intentionally skips `fn` tokens inside `macro_rules!` definitions and does not
claim to fingerprint functions generated after macro expansion. Fingerprint explicit
source wrappers around generated code when a reviewed boundary is needed.

Directory inputs are traversed recursively across Rust files. Non-write modes gate
only marked functions, so large `#[cfg(test)]` modules or helper functions are not
pulled into the gate unless they have markers.
