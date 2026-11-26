# Profiled Hegel property evidence

Octet provides reviewed Hegel evidence profiles across its pure, deterministic cores. Each profile runs selected laws under pinned `hegeltest = 0.14.27` (`native` backend).

Registered profiles in `standards/hegel/profiles.ncl`:

- `octet-pure-core-laws-v1` (`pure_core` normalization laws)
- `octet-evidence-identity-laws-v1` (`identity` laws over `roadmap`, `requirements`, `envelope`, `effect_closure`)
- `octet-evidence-lineage-laws-v1` (`lineage` laws over `evidence_multihop`, `roadmap`)
- `octet-catalog-composition-laws-v1` (`catalog_composition` laws over `effect_closure`, `hegel`, `evidence_contracts` ChaosControl boundary)
- `octet-reduction-acceptance-laws-v1` (`reduction_acceptance` laws over `proof_ref`, `proof_candidates`, `roadmap`)
- `octet-requirement-admission-laws-v1` (`requirement_admission` laws over `requirements`, `effect_closure`)
- `octet-freshness-mutation-laws-v1` (`freshness` laws and 16-dimension Buoy mutation matrix over `requirements`, `envelope`)

`standards/hegel/buoy-mutation-matrix.ncl` (`buoy-mutation-matrix.json`) maps all 16 Buoy `proof_invalidation_reasons` dimensions to Octet native contracts, production validators, dispositions, and single-role mutant controls.

This rail is not part of `cargo octet check`. A Hegel result is transcript evidence, not formal proof.

## Run the checked producer

```bash
cargo octet evidence collect \
  --rail hegel-property \
  --profile octet-pure-core-laws-v1 \
  --config standards/hegel/profiles.json \
  --workspace-root . \
  --run \
  --locked \
  --artifact-dir artifacts/hegel-property

cargo octet artifact verify \
  --artifact-dir artifacts/hegel-property
```

Do not supply `--command`. Octet constructs the fixed Cargo arguments from the checked profile.

The run uses a clear environment. Nix supplies the reviewed Cargo executable, vendor configuration, path, and library path.

The current profile binds the `cargo-octet` package version `0.1.1-rc.1`, Hegel `0.14.27`, and the native backend. The producer package version must match the running Octet binary; a profile from another package version is rejected rather than upgraded during import.

After a package-version change, regenerate `standards/hegel/profiles.json` from `standards/hegel/profiles.ncl` and update its checked BLAKE3 identity. Run the checked producer again against the final source tree to obtain new measured evidence. Do not relabel retained summaries or receipts: their producer version, profile identity, and measured inputs describe the original run.

`standards/evidence/fixtures/valid/hegel_hardened_details_v1.ncl` is a synthetic shape-contract fixture with placeholder identities, not a producer observation. The legacy import fixtures and byte-substitution counterexample fixture are not current-profile execution evidence either.

## Producer output

The producer writes `octet-hegel-producer-summary/v2` to one fixed output path. Octet does not parse stdout or stderr as evidence.

Octet retains these artifacts:

- `hegel-producer-summary.json`;
- `hegel-bounded-log.txt`;
- `hegel-source-manifest.json`;
- `hegel-generator-manifest.json`;
- `hegel-counterexample.json`, when a counterexample exists;
- `hegel-projection.json`.

The receipt remeasures the profile, Cargo lock, package manifest, selected source files, generator catalog, summary, logs, and counterexample.

A stale value causes a deterministic failure. Unsafe paths, symbolic-link substitutions, missing output, and over-limit data also fail.

## Law catalog

`standards/hegel/laws.ncl` owns the selected law catalog. Its generated JSON is reviewable input to the Rust core.

Each law records:

- the pure core, domain boundary, and stable law identifier;
- the bounded generator and any explicitly deferred domains (such as ChaosControl `#27` runtime fields until admission lands);
- positive and negative properties;
- the generated-input benefit;
- the case budget and replay method;
- a complementary concrete fixture;
- the effect-free boundary.

Each producer summary and receipt records `expected_laws`, `executed_laws` (with per-law `status`, `cases_completed`, `shrink_termination`, and `achieved_size`), and `unexecuted_laws`. Zero selected laws, ignored/filtered laws, zero generated cases, output truncation, or resource exhaustion fail closed.

## Counterexamples and replay

A failed law must bind one retained counterexample. The summary identity and counterexample identity must match the exact retained bytes.

When a property fails under `--run`, the producer adapter does not abort the run. It writes the counterexample to the profile `counterexample_path` and sets the summary result to `counterexample`. The counterexample records the failing property, the Hegel failure message, and the shrunk draws. Each text value is bounded to 512 bytes, and the adapter keeps at most 16 draws. The summary reports the failing property as its issue code. Its `cases_completed` value counts the generated cases that passed before the first failure. The adapter then exits successfully, and Octet records a failed receipt with a `hegel-counterexample` finding.

Profiles retain only bounded issue codes and bounded values. They do not authorize arbitrary paths or environment values.

Accepted counterexamples stay replayable through the retained artifact or a checked regression fixture.

## Import compatibility

Legacy `hegel-property-summary/v1` files remain valid in import mode. Use the old import command without the hardened profile:

```bash
cargo octet evidence collect \
  --rail hegel-property \
  --input target/hegel-property-v1.json \
  --artifact-dir artifacts/hegel-property-import
```

Octet preserves the legacy evidence boundary. It does not invent profile, package, source, log, or counterexample measurements.

A hardened request can also be imported with the checked profile. Its measured fields must already be complete and consistent.

## Claim boundary

Hegel checks bounded generated inputs only. It does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety.

Hegel does not replace concrete tests, negative fixtures, cargo-fuzz, Kani, Miri, Loom, or formal verification.
