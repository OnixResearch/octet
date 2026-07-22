# Validation

## Outcome

The inherited provenance traceability baseline is closed without deleting or weakening accepted requirements. Cairn traceability now reports `missing: []`. The only remaining issue is the separately scoped dangling marker `octet.effect_closure.cli.check_failure`, which this change intentionally preserves for the next native change.

Both `octet-default` and `cairn-default` now select the same exact maintained implementation and documentation roots. No lifecycle archive, generated policy file, broad repository root, or historical task list is accepted as evidence. Current generated policy hash is `e4e27389f567bc59f739be95244e3ba056aafdc37c53f66a57304ef9e5ee5a9e`.

## Semantic mapping

- Generic evidence-chain DTO and fail-closed validation IDs map to `cargo-octet/src/evidence_chain.rs`.
- Function-address profile wrapping maps to `cargo-octet/src/function_address.rs`.
- The cross-stack positive/validation IDs map to bounded operator documentation that names the maintained Cairn smoke at revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`.
- Formal-proof Valence fixture IDs map to its pure validator, positive/negative fixture tests, bounded shell report, and exact operator doc.
- Preserves receipt fixture IDs map to its pure validator, positive/negative fixture tests, canonical-byte measurement shell, and exact operator doc.
- Hegel Valence IDs map to the production receipt projection, fail-closed tier/boundary validator, positive/negative export test, and trust-boundary documentation.

## Checks

- Pre-change cargo-octet library baseline — 377 passed, 4 ignored.
- Pre-change effect-closure direct-check failure integration test — passed.
- Post-change cargo-octet library tests — 377 passed, 4 ignored.
- Cairn function-address stack smoke — 2 passed: the frozen positive chain and boundary-mutation matrix.
- `cargo fmt --all -- --check` — passed.
- Repository-prescribed cargo-octet Clippy with warnings denied — passed.
- Cairn policy export freshness — passed.
- Strict Cairn validation and proposal/design/tasks gates — passed.
- Accepted-spec sync — already applied with no accepted-spec byte change.
- Nextest — 231 passed, 2 configured tests skipped.
- Full compatible-system flake check — passed.

An exploratory overlay that forcibly enabled every Clippy pedantic lint reported existing whole-crate style debt across unrelated modules. It was not used as an acceptance gate; the repository-prescribed warnings-denied command and the full flake gate passed.

## Adversarial audit

Portfolio search rejected accepted-requirement deletion, broad source discovery, archive/task evidence, and local-unit substitution for the cross-stack smoke. The advisory review specifically warned that a function-address unit test alone would not prove stack-smoke execution. The selected route instead reran Cairn's maintained positive and negative stack fixture and recorded its exact revision and bounded claims.

The declared exploration budget used twelve grouped retrieval/search passes, one advisory review, and three audit/implementation rounds. Validation terminated because the success contract was met, not because model agreement or task completion was treated as evidence.

## Non-claims

Marker coverage proves only that accepted IDs have maintained, discoverable implementation, test, documentation, or explicit external-smoke evidence. It does not prove arbitrary-version interoperability, whole-program correctness, verifier soundness, deployment, remote trust, release eligibility, or that every implementation branch is semantically complete.

Nix continued locally after the known unavailable Git HTTPS helper and configured remote-builder warnings. A transient documentation fixup emitted a broken-pipe message while the full flake evaluation still completed with all evaluated compatible-system checks passing.
