# Proposal: Refresh the self-check warning budget for PR #38

## Why

PR #38 adds the reproducible consumer job, `--artifact-base` and opt-in
`--raw-log-retention build-log`. The full flake check's
`cargo-octet-self-check` compares the workspace warning set with
`standards/self-check-warning-budget/octet-baseline.json`; the branch produced
4,253 warnings against the 3,030-identity baseline: 19 new identities and 5
stale ones.

## What Changes

- Fix the new findings that are cheap and unambiguous: three boolean names,
  one discarded `writeln!` result and ten `.expect()` calls replaced with
  propagated `fmt::Result` values in the summary helpers, two unreserved
  collections, three over-length functions split into named helpers, the
  retention type qualified at every use site instead of imported, and one
  six-parameter card check bundled into a locator struct.
- Keep one new finding in the reviewed baseline: `evaluate_contract` takes six
  parameters because it verifies a contract and its exact bytes against the
  same source root and optional artifact base. Bundling would let a caller pair
  bytes with a different parsed value; 17 call sites (mostly tests) would also
  change.
- Regenerate the baseline with the checked `cargo-octet check --write-baseline`
  path, update the accepted identity/warning totals in the self-check
  derivation, and record every added and removed identity with its source
  location and reason in `evidence/`.
- Update the documented stored-budget paragraph; leave the historical
  reduction tables and their non-claims unchanged.

## Non-claims

The warning budget is an advisory regression baseline, not correctness or
release evidence. The refresh records the current reviewed set; it does not
prove accepted warnings are harmless or complete, and it enables or disables no
lint. Local Nix gates do not attest GitHub CI or release qualification.
