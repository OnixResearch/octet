# Proposal: Location-independent proof-deck locators

## Why

`cargo octet requirements check` resolves every `proof_decks[].artifact_root`
and `deck` against `--source-root` (`cargo-octet/src/requirements.rs`,
`resolve_path`). A proof deck produced by a realized job lives in the producer
output, not in the source checkout, so the retained pilot evidence names it by
an absolute host path. The decision binds the evidence bytes, so moving the same
proof cards to another store path changes the decision identity even though no
reviewed byte changed: the pilot cards gave `906cc950…` in the first store and
`8abe489a…` after the evidence was repointed at a second store. Cairn already
accepts relative deck locators and reads them inside the case's producer output
(`deck_relative_path`), so only Octet needs the corresponding surface.

## What Changes

- Add `--artifact-base PATH` to `cargo octet requirements check` and
  `requirements verify`. When given, relative `artifact_root` and relative
  `deck` locators resolve against it instead of `--source-root`. Absolute
  locators keep today's behavior and no flag means no change.
- Hold a relative locator to the base: refuse empty, `..`, root, and prefix
  components with a clear diagnostic, mirroring Cairn's `deck_relative_path`.
  An absolute locator that names a file inside the base is held to the same
  rule after the base prefix is stripped, so `<base>/../x` is refused.
- Keep the base path out of the decision bytes so two identical producer outputs
  stored in different directories evaluate to byte-identical decisions.
- Convert `standards/requirements/fixtures/cairn-pilot/evidence.json` to
  relative locators (`artifact_root: "."`,
  `deck: "artifacts/proof-cards.json"`) and refresh the derived decision
  fixture, the policy registry validation command, the pilot README, the
  contracts guide, CLI help, and the CHANGELOG.
- Add positive and negative tests for the two-location identity, escaping
  locators, unchanged absolute locators, and the verify recomputation.

## Impact

- **CLI**: `cargo-octet/src/cli.rs`.
- **Evaluation**: `cargo-octet/src/requirements.rs`,
  `cargo-octet/src/requirements/evaluation/{decision,clause,formal}.rs`.
- **Tests**: `cargo-octet/src/requirements/tests/evaluation.rs` and the CLI
  parser test.
- **Standards**: `standards/requirements/fixtures/cairn-pilot/evidence.json`,
  `standards/requirements/fixtures/cairn-pilot/decision.json`,
  `standards/requirements/cairn-pilot/README.md`,
  `standards/policies/registry.ncl`.
- **Docs**: `docs/requirements-contracts.md`, `CHANGELOG.md`.
- **Lifecycle**: this change package and its `evidence/` receipts.

## Non-goals

- No change to the contract format, the proof-card schema, the producer job, or
  the realized Mantle output.
- No new producer output or fixture tree; the pilot still judges the preserved
  realized store.
- No change to how source bindings, reviewed specifications, or empirical
  checks resolve against `--source-root`.
