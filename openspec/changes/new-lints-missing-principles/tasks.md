## Phase 1: Safety lints

- [x] Implement `explicit_defaults` lint (late pass, resolve `Default::default` identity, treat sibling workspace crates as foreign, skip current-crate and test contexts)
- [x] Implement `unbounded_channel` lint (late pass, match `std`, `crossbeam`, and Tokio internal unbounded constructor identities)
- [x] Register both in `src/lib.rs`
- [x] Add UI fixtures for both lints, including foreign-vs-local default coverage

## Phase 2: Structure lints

- [x] Implement `usize_in_public_api` lint (late pass, check inherent `pub fn` signatures, skip trait-directed signatures)
- [x] Implement `too_many_parameters` lint (configurable threshold default 5, receivers excluded)
- [x] Add `max_parameters` to `Config` struct and tests
- [x] Register both in `src/lib.rs`
- [x] Add UI fixtures plus a config fixture for custom `max_parameters`

## Phase 3: Naming lint

- [x] Implement `negated_predicate` lint (bool-only, whole-token negative prefixes, near-miss names like `notify` stay quiet)
- [x] Register in `src/lib.rs`
- [x] Add UI fixture coverage for positive, negative, near-miss, and non-bool cases

## Phase 4: Collection growth lint

- [x] Implement `unbounded_collection_growth` lint as a same-block local-binding rule only
- [x] Support reservation / bound suppressions for `with_capacity`, `reserve`, `reserve_exact`, `try_reserve`, and explicit local len/capacity guards
- [x] Register in `src/lib.rs`
- [x] Add UI fixtures for positive, reserved, guarded, and ambiguous-binding no-fire cases

## Phase 5: Downstream validation

- [ ] Run all new lints against Aspen and save per-lint finding counts
- [ ] Classify each lint as starter-candidate or inventory-first; do not promote any lint without low-noise evidence
- [ ] Update rollout playbook with the classification and any new lint families
- [ ] Update spec domain files with the synced lint requirements
