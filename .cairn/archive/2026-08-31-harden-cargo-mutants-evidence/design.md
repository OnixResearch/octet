## Context

The current mutation rail recognizes several normalized field aliases. It classifies survived and timed-out mutants as manual review evidence.

Run mode uses fixed argv, but it parses combined stdout and stderr. The receipt does not prove which mutants were generated or whether the unmutated suite passed.

## Decisions

### Decision: Harden the existing rail

**Choice:** Keep `mutation-testing` and add a typed provider profile instead of creating another command family.

**Rationale:** Existing consumers retain a stable rail name while execution and evidence become stronger.

### Decision: Bind a pinned provider and exact scope

**Choice:** Nickel profiles bind cargo-mutants identity, Rust toolchain, Nix closure, package and target scope, features, exclusions, command shape, budgets, limits, and non-claims.

The shell constructs fixed argv. It rejects `--command` and undeclared Cargo arguments.

**Rationale:** Mutation results depend on tool version, selected code, test configuration, and timeout policy.

### Decision: Require native machine artifacts

**Choice:** The pinned profile declares the supported machine-readable outcome artifact set and normalization version.

Octet retains raw artifact identities and derives a deterministic projection. Human stdout and stderr remain logs only.

If the provider lacks a stable artifact contract, the run fails with an unsupported-provider diagnostic.

**Rationale:** Human output is not a durable evidence contract.

### Decision: Require a clean unmutated baseline

**Choice:** A passing mutation receipt requires the same profile's unmutated build and test baseline to complete successfully.

A failed, timed-out, stale, or missing baseline makes all mutant outcomes non-promotable.

**Rationale:** Mutant outcomes are not meaningful when the original test suite is already failing.

### Decision: Preserve every terminal outcome

**Choice:** The normalized inventory records caught, survived, timed-out, unviable, skipped, not-run, and tool-failure states without merging them.

Each row binds mutant identity, operator, safe source location, source hash, outcome, duration class, diagnostic identity, and retained diff identity when policy allows.

**Rationale:** Aggregate mutation scores can hide incomplete or ambiguous work.

### Decision: Keep policy decisions separate from source outcomes

**Choice:** Source outcomes remain exact. Profile policy separately decides pass, fail, or manual review.

**Rationale:** Octet must not relabel cargo-mutants observations.

## Functional Core and Imperative Shell

The pure core validates profiles, baselines, inventories, outcome partitions, identities, limits, policy decisions, and deterministic ordering.

The shell resolves the pinned tool, creates capability-rooted output paths, runs fixed argv, reads bounded artifacts, and writes receipts atomically.

## Risks / Trade-offs

- Mutation runs can be expensive. Profiles bind scope, concurrency, timeout, and total-budget policy.
- Diffs can expose sensitive code. Profiles define retention and redaction behavior.
- Provider artifact formats can change. Exact tool pins and normalization versions prevent silent reinterpretation.

## Validation

Positive fixtures cover a clean baseline, all mutants caught, a reviewable survivor, and a compatible legacy import.

Negative fixtures cover failed baselines, missing mutants, duplicate outcomes, malformed artifacts, stale sources, unsafe paths, timeouts, limits, tool failure, and overclaims.
