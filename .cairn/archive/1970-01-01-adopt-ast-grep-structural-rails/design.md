## Design

1. Add a tool-native `sgconfig.yml` plus rule/test directories under an Octet-owned standards surface, with any human-selected rule-pack profiles or thresholds authored in Nickel when they become configurable policy.
2. Treat ast-grep as a structural candidate engine: rules may report source shapes, migration opportunities, and review cues, but they must not claim behavioral proof.
3. Model rule bundles with BLAKE3 identity over the checked-in ast-grep config, rule files, tests, and explicit ignore/include scope.
4. Require every promoted ast-grep rule to carry positive and negative rule tests before it can participate in warning or error posture.
5. Emit machine-readable findings through ast-grep JSON or SARIF and wrap them in Octet evidence artifacts with tool identity, scan scope, rule bundle identity, and non-claims.
6. Keep codemods interactive or dry-run-first until rule tests, focused diffs, and downstream checks prove the migration safe for the declared scope.
7. Define promotion lanes: inventory-only, advisory warning, error gate, and migration recipe. Promotion requires stable signal quality and reviewable false-positive handling.
8. Reuse Valence export only for identity, span, role, and non-claim threading; Octet remains responsible for the source-shape rule meaning and evidence posture.

## Non-Goals

- No replacement for Dylint, rustc, cargo check, Verus, Kani, proof rails, or release-readiness gates.
- No semantic equivalence claim for ast-grep rewrites.
- No default broad rewrite across downstream consumers without a scoped migration session and validation evidence.
