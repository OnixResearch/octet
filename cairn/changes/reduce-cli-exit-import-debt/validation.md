# Validation

## Outcome

Octet's first post-ratchet warning-debt reduction removes the unique `non_trait_imports` stable identity for the private `ExitCode` import in `cargo-octet/src/lib.rs`. The thin CLI shell now names `std::process::ExitCode` explicitly without aliases, suppressions, exception changes, lint-policy changes, or exit-behavior changes.

The canonical inventory decreased from 2,670 to 2,669 warning occurrences, from 1,820 to 1,819 unique stable identities, and from 1,114 to 1,113 `non_trait_imports` occurrences. `negated_predicate` remains zero. The replacement baseline BLAKE3 identity is `b3:1327af062e287cdd51eba7cbc26dd63ee4a1f47714ac4491884a644e05e22d5b`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. The permanent behavior-preserving negative mutation still exits `1` with exactly one new `negated_predicate` identity. Malformed metadata and Cargo-child failures remain integration exit `2` cases.

Current Cairn traceability policy hash is `07c49e6837209e8e8938ca86f4cdea1ef78a37669ead2b8c5d29cf8153cfd299`. Traceability reports `missing: []` and `dangling: []`.

## Portfolio audit

The completion contract required an old-baseline run with removals only. Two initially plausible candidates were rejected rather than normalized into success:

- Fully qualifying the four imports in `cargo-octet/build.rs` produced 2,666 occurrences and 1,816 current identities, but changed ambient-environment, panic, unwrap, and borrowed-signature snippets. The old baseline reported 9 new and 13 stale IDs, so this route was falsified.
- Fully qualifying `std::fmt` in `cargo-octet/src/domain_types.rs` produced 2,669 occurrences and 1,819 current identities, but file-length and underscore-filename lints hash full source bytes. The old baseline reported 2 new and 3 stale IDs, so this route was falsified.
- Fully qualifying the unique CLI `ExitCode` import produced 2,669 occurrences and 1,819 current identities. The old baseline exited `0` and reported zero new IDs plus exactly stale ID `b3:8515683389cc483f`; this route validated.

Aliases, lint exceptions, source suppressions, and a repository-wide rewrite were rejected because they would evade the owner-path rule, weaken policy, or create an unreviewable identity surface.

## Baseline-set evidence

The runner-generated candidate was compared as a stable-ID set before installation:

- old count: 1,820;
- candidate count: 1,819;
- additions: `[]`;
- removals: `["b3:8515683389cc483f"]`;
- old hash: `b3:7bce67812e0d57029644efc644ed44035250b2891391b846fc9a2a92caf45de9`;
- candidate hash: `b3:1327af062e287cdd51eba7cbc26dd63ee4a1f47714ac4491884a644e05e22d5b`.

The refreshed canonical run reports 2,669 findings, 1,113 `non_trait_imports` occurrences, zero target-identity occurrences, zero `negated_predicate` occurrences, 1,819 accepted/current identities, and zero new/stale IDs.

## Checks

- Pre-change packaged self-check gate — passed at the 2,670/1,820 baseline.
- Focused CLI/library tests — 380 passed, 4 provider-dependent tests ignored.
- Old-baseline CLI-exit run — exit `0`, `warning-only`, 2,669 warnings, exactly one stale ID, zero new IDs.
- Runner-generated candidate set comparison — exact one-ID removal, no additions.
- Refreshed canonical structured run — exit `0`, `warning-only`, 2,669 warnings, 1,819 accepted/current IDs, zero errors, zero new/stale IDs.
- Hermetic `cargo-octet-self-check` Nix gate — refreshed positive case, exactly-one-new-ID mutation, malformed metadata, and Cargo-child failure contracts passed.
- Repo-owned object-corpus fixture refresh — 1 explicit ignored refresh test passed; only internal and expanded source-identity hashes changed, while object counts, caveats, and non-claims remained unchanged.
- Focused object-corpus tests — 12 passed; fixture freshness and independent fresh-build determinism Nix gates passed.
- Cairn policy export freshness — passed.
- Strict Cairn validation — passed with no issues.
- Cairn traceability — `missing: []`, `dangling: []`.
- Permanent positive/negative `cairn-traceability` Nix gate — passed.
- `cargo fmt --all -- --check` — passed.
- Workspace/all-target Clippy with warnings denied — passed.
- Nextest — 232 passed, 2 configured tests skipped.
- Full compatible-system `nix flake check -L --option secret-key-files ''` — passed.

## Functional core and shell

No decision core changed. `run_cli_from_env` remains the imperative shell over `try_main`; successful and integration-error values use the same `u8` conversion and named error code. The existing pure baseline set comparison remained the only admission authority.

## Non-claims

The result proves only that one reviewed stable warning identity was removed from the pinned workspace-library static-lint run and that no unreviewed identity entered the refreshed baseline. It does not prove the remaining warnings correct, low-noise, harmless, or complete. It does not prove CLI correctness, test/binary/example coverage, dependency behavior, runtime safety, whole-program correctness, lint completeness, cross-machine determinism, or release eligibility.

Function-object receipt refresh proves bounded source-token identity replay only, not semantic equivalence or behavior. Traceability proves selected-policy linkage only, not runtime reachability or requirement truth.

Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings.
