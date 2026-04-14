## Phase 1: Safety lints

- [ ] Implement `explicit_defaults` lint (late pass, detect `Default::default()` on external types)
- [ ] Implement `unbounded_channel` lint (late pass, detect known unbounded channel constructors)
- [ ] Register both in `src/lib.rs`
- [ ] UI fixtures for both lints

## Phase 2: Structure lints

- [ ] Implement `usize_in_public_api` lint (late pass, check `pub fn` signatures)
- [ ] Implement `too_many_parameters` lint (early pass, configurable threshold default 5)
- [ ] Add `max_parameters` to `Config` struct
- [ ] Register both in `src/lib.rs`
- [ ] UI fixtures for both lints

## Phase 3: Naming lint

- [ ] Implement `negated_predicate` lint (early pass, check identifier names)
- [ ] Register in `src/lib.rs`
- [ ] UI fixture

## Phase 4: Collection growth lint

- [ ] Implement `unbounded_collection_growth` lint (detect `.push()`/`.insert()`/`.extend()` in loops without capacity)
- [ ] Register in `src/lib.rs`
- [ ] UI fixture

## Phase 5: Downstream validation

- [ ] Run all new lints against Aspen to measure noise level
- [ ] Classify findings and assign each new lint to starter or inventory-first profile
- [ ] Update rollout playbook with new lint families
- [ ] Update spec domain files with new lint requirements
