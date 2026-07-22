# Close evidence-provenance traceability

## Why

Cairn currently reports 32 accepted `octet.evidence_provenance.*` requirement IDs without executable or documentation evidence. The underlying typed handoff, formal-proof fixture, Preserves fixture, and Hegel export behavior already exists with positive and negative tests. The drift came from narrowly configured evidence roots that omit four exact implementation files plus missing semantic markers in those files.

Leaving this baseline open makes later traceability reports noisy and can hide new regressions. Removing accepted requirements, scanning lifecycle archives, or attaching markers to unrelated code would create false completion instead of restoring evidence linkage.

## What Changes

- Add only the exact maintained implementation and operator-documentation files needed by the four missing evidence families to both typed Nickel traceability profiles.
- Attach each missing requirement ID to the pure core, thin shell/report, positive test, negative test, or bounded operator documentation that actually establishes it.
- Record and rerun Cairn's maintained function-address stack smoke at Cairn revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176` for the cross-repository handoff-validation requirement.
- Preserve the separate `octet.effect_closure.cli.check_failure` dangling marker for its own change.
- Keep all accepted evidence-provenance requirement text and runtime behavior unchanged.

## Impact

The generated Cairn policy hash changes because traceability evidence roots change. Rust behavior, receipt schemas, evidence tiers, accepted requirement semantics, and release claims do not change. Completion means `missing` is empty while the one pre-existing effect-closure dangling marker remains explicit; it does not mean Tracey coverage proves behavioral correctness or release eligibility.
