# Validation: Adopt Cairn COB Collaboration

## Result

Octet exercised one real native Cairn change through Cairn commit `70db8fe114910feb5d162fea5db8ad4ffdb6f176` without adding Cairn, `radicle`, or `radicle-node` product dependencies.

The pilot previewed and executed signed import into an explicit bare Git store, changed the active task artifact, previewed and executed a non-noop signed update, transported exact branch and namespace refs through local Git only, inspected the transported state, and previewed plus executed materialization into a second Octet checkout. The retained complete Git bundle reloads through Cairn's signature-validating status path.

## Bound Identities

- Octet base Git OID: `28ffbe8cfb01879a241cad248008b75252a8b7f5`
- Workspace: `workspace:octet`
- Backend: `radicle-cob.v1`
- Creator actor: `z6MkjPAjbbnXWtKTuUarEF3CbhnPjFzU9ify9Xuz7qdTv4Dn`
- Object/root operation: `8d4cf943a20d666c16cea0190db2541767ef8603`
- Signed update operation and final ref target: `35e76bfd1dc9755f936c53762c9af999b8a666dd`
- Final content root: `2a54f7e9216fd71d7e23ba11114ce6bf3eb3df7a4e5b7e064dee1c25cf66c183`
- Final state hash: `70baef24a418d18478cfdc5169b1dbfa3e093f2da05c8bd3beba5e8fbd46814d`
- Materialized lifecycle NAR/BLAKE3 identity: `blake3-+nSMyo4rJmdEBytxIOC0enxjQIHLWXpR2MjkD0Rkexo=`
- Retained bundle BLAKE3: `f79e2c3bde695876e9e9b62168f5de50b3446662664bc24eff6803d28c0d9b5a`

The private mutation key and both mutable COB stores remained under ignored `target/` and are not retained. The checked-in manifest records only public and artifact identities.

## Portfolio Result

Four mechanism families were considered:

1. **Separate explicit COB store — validated.** This preserved a real Octet lifecycle root while isolating signed refs from Octet's primary `.git`.
2. **Primary worktree Git database — rejected.** It exercised the same provider while creating unnecessary durable operator refs.
3. **Mocked receipts — falsified.** Synthetic JSON could not establish signed Git mutation, ref encoding, or checkout.
4. **Live Radicle node/profile — excluded.** It contradicted the COB-only goal and would invite unsupported publication and synchronization claims.

The selected route remains correlated with Cairn's implementation on both mutation and read paths. The retained Git bundle improves independent Git-level inspection but does not make this a second COB implementation.

## Positive Evidence

- Import preview was valid, dry-run, unblocked, and non-mutating.
- Import execution reused the exact plan/input/content identities and created the declared root object and ref.
- Update preview was valid, non-noop, and identified only the changed `tasks` artifact.
- Update execution cryptographically verified the embedded creator, retained the root operation, and appended the declared update operation.
- Source and destination stores had matching base and namespace ref targets, no configured remotes, and no destination tags.
- Second-root status reported an absent destination, current policy, conflict-free content, and checkout readiness.
- Checkout preview did not mutate; checkout execution materialized four byte-identical artifacts.
- Conflict inspection reported `conflicted: false` without selecting a winner.
- A fresh bare store populated only from `octet-cob.bundle` reloaded the same actor, object, content, and state through Cairn status.

## Negative and Adversarial Evidence

- Executed checkout into a plain Git clone initially failed closed because Git does not retain an empty `cairn/changes/` directory. The failed receipt is retained; the guide now requires recreating the directory before checkout.
- The first local Git fetch auto-followed a reachable tag despite explicit positive refspecs. The accepted replay uses `--no-tags`, starts from a new empty bare store, and verifies an empty tag set.
- The Rust evidence test mutates the manifest's object ID and requires the stable `import-execute.json/object_id` failure.
- Every retained JSON receipt and the complete Git bundle is BLAKE3-bound by the typed manifest.
- The Nix gate re-exports the Nickel manifest, byte-compares it, verifies and imports the bundle, checks exactly two expected heads, compares exact targets, and proves the fresh audit store has no configured remote.
- Advisory review suggested checking invented fields that do not exist in these receipts; those suggestions were rejected. Its useful bundle-head and signature-reload challenge produced the fresh-store bundle replay evidence.

## Validation Rails

Passed:

- `cargo fmt --all -- --check`
- focused positive/negative `cargo test --test cairn_cob_pilot`
- release-floor `cargo clippy --all-targets` with warnings, pedantic, debug, memory-forget, unsafe-block, and multiple-unsafe-operation denials
- Nickel typecheck/export freshness for `fixtures/cairn-cob-pilot/manifest.ncl`
- `git bundle verify fixtures/cairn-cob-pilot/octet-cob.bundle`
- `nix run .#octet-policy-validate`
- native Cairn strict validation plus proposal, design, and tasks gates
- object-corpus fixture refresh and focused freshness replay
- `nix build .#checks.x86_64-linux.cairn-cob-pilot --no-link -L`
- `nix build .#checks.x86_64-linux.nextest --no-link -L`
- `nix flake check -L --option secret-key-files ''` across 450 checks

The first full flake run exposed strict Clippy pass-by-value findings in the new validator; the core now consumes expected JSON values explicitly and the release Clippy gate passes. Later full runs exposed inherited stale object-corpus receipts plus standalone source-copy omissions for previously added Aeneas and Charon generated profiles. The repository-owned corpus updater refreshed those receipts, and the standalone copy allowlist now includes the exact generated profile directories. Focused replays and the final full flake passed.

`nix run .#cargo-octet -- check` remains blocked before lint execution by the repository root's existing lack of the command's required `[metadata]` table in `Cargo.toml`. This pilot did not change `Cargo.toml`; focused Clippy, Nextest, and the full Nix checks provide the accepted checked-path evidence instead.

The pre-change Cairn traceability baseline contained 32 missing accepted IDs and dangling `octet.effect_closure.cli.check_failure`. These unrelated gaps are preserved and reported rather than silently attributed to or repaired by this pilot. The pilot's own IDs are linked from `tests/cairn_cob_pilot.rs`.

## Bounded Claims

This proves local integration observations for the exact Octet base, Cairn commit, Git SHA-1 object format, pinned direct COB library cohort, signer, policy, lifecycle snapshot, and retained bundle.

It does not prove Radicle runtime participation, identity-document authority, publication, peer discovery, availability, authentication, authorization, durability, synchronization, convergence, semantic merge correctness, lifecycle gate truth, evidence truth, source correctness, whole-program correctness, compiler or cryptographic implementation correctness, or release eligibility.

The transported snapshot records the lifecycle artifacts at the signed update point. Later checkbox and validation-file progress is repository lifecycle evidence, not an implicit mutation of that historical COB state.
