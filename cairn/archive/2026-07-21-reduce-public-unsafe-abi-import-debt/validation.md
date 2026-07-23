# Validation

## Outcome

Octet's second post-ratchet warning-debt reduction removes the unique `non_trait_imports` stable identity for the private `rustc_abi::ExternAbi` import in `src/safety/public_unsafe_api.rs`. The pure `signature_uses_rust_abi` predicate now names `rustc_abi::ExternAbi::Rust` explicitly without aliases, suppressions, exception changes, lint-policy changes, or classification changes.

The canonical inventory decreased from 2,669 to 2,668 warning occurrences, from 1,819 to 1,818 unique stable identities, and from 1,113 to 1,112 `non_trait_imports` occurrences. `negated_predicate` remains zero. The replacement baseline BLAKE3 identity is `b3:23efb13653cef018dd76dc03eb6319bbd1946c67089add0e53db4c8c673b357e`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. The permanent behavior-preserving negative mutation still exits `1` with exactly one new `negated_predicate` identity. Malformed metadata and Cargo-child failures remain integration exit `2` cases.

Current Cairn traceability policy hash is `07ce8291a384737c4c41121a39ad094e96a29d89e949f6df956c8540cef788f0`. Traceability reports `missing: []` and `dangling: []`.

## Portfolio audit

The bounded search inspected 371 remaining `non_trait_imports` identities, including 261 represented by a single occurrence. Completion required old-baseline removals with no additions.

| Family | Result | Evidence |
| --- | --- | --- |
| root configuration import | falsified | Qualifying the unique `Config` import lowered aggregate counts but changed two whole-file identities; the old baseline exited `1` with 2 new and 3 stale IDs |
| leaf ABI classification | validated | Qualifying the one `ExternAbi::Rust` use exited `0` with zero new IDs and exactly stale ID `b3:672b1993827f233b` |
| numeric-lint AST imports | deferred | The file contains only import findings, but changing traversal syntax has a larger semantic audit surface than one ABI enum variant |
| standards runner calls | deferred | The file contains only import findings, but the selected candidate has one use and focused positive/negative classification tests |
| aliases, exceptions, suppressions, broad rewrites | rejected | These evade policy, retain accepted debt, or enlarge unrelated identity and review risk |

The surviving candidate's adversarial audit confirmed that the positive Rust-ABI case remains true, extern ABI remains excluded, the import identity occurs once, no unrelated stable ID changed, and the final runner-generated baseline preserves current finding line context.

Declared budget was one inventory pass, four mechanism families, at most four old-baseline runs, two redirect rounds, and zero advisory models. Used budget was one inventory pass, four families, two old-baseline runs, one redirect round, and zero advisory models. Search stopped on deterministic validation.

## Baseline-set evidence

The final runner-generated candidate was compared as a stable-ID set before installation:

- old count: 1,819;
- candidate count: 1,818;
- additions: `[]`;
- removals: `["b3:672b1993827f233b"]`;
- old hash: `b3:1327af062e287cdd51eba7cbc26dd63ee4a1f47714ac4491884a644e05e22d5b`;
- candidate hash: `b3:23efb13653cef018dd76dc03eb6319bbd1946c67089add0e53db4c8c673b357e`.

The refreshed canonical run reports 2,668 findings, 1,112 `non_trait_imports` occurrences, zero removed-identity occurrences, zero `negated_predicate` occurrences, 1,818 accepted/current identities, and zero new/stale IDs.

## Checks

- Pre-change focused public-unsafe-API tests — 4 positive/negative classification cases passed.
- Pre-change numeric-lint and standards-runner focused baselines — passed.
- Root `Config` old-baseline falsification — exit `1`, 2,668 warnings, 2 new IDs, 3 stale IDs.
- ABI-path old-baseline run — exit `0`, `warning-only`, 2,668 warnings, exactly one stale ID, zero new IDs.
- Post-change focused public-unsafe-API tests — 4 passed.
- Runner-generated final candidate set comparison — exact one-ID removal, no additions, reviewable current context.
- Refreshed canonical structured run — exit `0`, `warning-only`, 2,668 warnings, 1,818 accepted/current IDs, zero errors, zero new/stale IDs.
- Hermetic `cargo-octet-self-check` Nix gate — refreshed positive case, exactly-one-new-ID mutation, malformed metadata, and Cargo-child failure contracts passed.
- Repo-owned object-corpus refresh test — 1 passed and produced no fixture changes because this lint-library source is outside the maintained internal cargo-octet corpus.
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

`signature_uses_rust_abi` remains a pure deterministic predicate over a supplied `FnSig`; only the path spelling of the compared enum variant changed. Rustc lint callbacks remain the imperative shell. Existing positive Rust-ABI and negative extern-ABI fixtures test the classification boundary without mocks.

The existing pure stable-ID set comparison remained the only baseline admission authority.

## Non-claims

The result proves only that one reviewed stable warning identity was removed from the pinned workspace-library static-lint run and that no unreviewed identity entered the refreshed baseline. It does not prove the remaining warnings correct, low-noise, harmless, or complete. It does not prove ABI-model completeness, lint correctness, test/binary/example coverage, dependency behavior, runtime safety, whole-program correctness, cross-machine determinism, or release eligibility.

Traceability proves selected-policy linkage only, not runtime reachability or requirement truth. Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings.
