# Tasks: Exact Molten component effects artifact profile

- [x] [serial] Integrate only the effects delta on published checker `0e0e5dc8461b8b32c286de362c290b3f3ea2deee`, preserving published baseline policies/identities and regression coverage; use a new native change rather than reopening the archived prerequisite. r[octet.molten_effects.profile] r[octet.molten_effects.identity]
- [x] [serial] Implement scope-local metadata comparison and validated local import binding with retained bound-import evidence and conservative unresolved authority. r[octet.molten_effects.metadata_scope] r[octet.molten_effects.boundary]
- [x] [serial] Implement conservative definition/instantiation resource accounting without counting bound aliases; preserve declaration limits and fail unknown generative resource graphs closed. r[octet.molten_effects.resources]
- [x] [serial] Add the exact structural effects Nickel profile, optional schema fields, static seed, generated projection and guide/non-claims, reusing granular padded-indirect encoding rather than enabling reference types. r[octet.molten_effects.profile] r[octet.molten_effects.identity]
- [x] [serial] Add meaningful paired regression cases for metadata scopes, binding/ambient authority, canonical signatures, baseline closure and resource boundaries. r[octet.molten_effects.metadata_scope] r[octet.molten_effects.boundary] r[octet.molten_effects.resources] r[octet.molten_effects.profile]
- [x] [serial] Run final clean-worktree focused checker/receipt tests, Nickel typecheck/export freshness, genuine normalized guest collection and bundle verification; preserve initial failed attempts and emitted identity evidence. Initial evidence: `21-clean-nickel-typecheck.json` through `26-clean-genuine-effect-verify.json`. Final review-corrected evidence: `40-index-corrected-format.json` through `46-index-corrected-nix-rail.json`; 27 focused tests pass, including canonical definitions and exported-index resource boundaries, unchanged genuine guest collection and verification pass, strict package Clippy and the Nix artifact rail pass. r[octet.molten_effects.metadata_scope] r[octet.molten_effects.boundary] r[octet.molten_effects.resources] r[octet.molten_effects.profile] r[octet.molten_effects.identity]
- [x] [serial] Complete checker-local native Cairn validation, required proposal/design/tasks gates and source review, establishing bounded readiness for native archive and exact immutable publication. Review: `review.md`; pinned lifecycle receipts: `32-pinned-native-proposal.json`, `34-pinned-native-design.json`, `34-pinned-native-tasks.json`, `35-pinned-native-strict.json`. Downstream Mantle/Molten pinning, operation and cohort publication are separate scope and are not prerequisites to checker archive. r[octet.molten_effects.identity]

## Verification Coverage

| Scenario | Focused proof |
| --- | --- |
| Independent nested name sections pass | custom_metadata_duplicates_are_local_to_each_binary_scope |
| Same-scope name duplication denies | custom_metadata_duplicates_are_local_to_each_binary_scope |
| Internally supplied adapter passes | bound_adapters_are_not_external_imports_or_extra_resources |
| Unused or exported adapter retains imports | bound_adapters_are_not_external_imports_or_extra_resources |
| Table alias is not a new table | bound_adapters_are_not_external_imports_or_extra_resources |
| Repeated instantiation exceeds the count bound | bound_adapters_are_not_external_imports_or_extra_resources; effects_resource_limits_count_independent_instantiations_and_check_declarations |
| Effects table boundaries deny above limits | effects_resource_limits_count_independent_instantiations_and_check_declarations |
| Canonical type indexing is immaterial | effects_profile_requires_exact_structural_boundary_and_keeps_baselines_closed |
| Wrong or undeclared boundary denies | effects_profile_requires_exact_structural_boundary_and_keeps_baselines_closed |
| Existing profiles remain closed | effects_profile_requires_exact_structural_boundary_and_keeps_baselines_closed; published baseline bulk/padded/reference/SIMD regression |
| Canonical lowerings/intrinsics consume function budget; aliases/lifts do not | effects_function_budget_counts_canonical_lower_definitions; effects_function_budget_counts_resource_intrinsics_but_not_aliases_or_lifts |
| Enclosing instances multiply canonical definitions | effects_function_budget_multiplies_canonical_definitions_by_enclosing_instances |
| Component exports preserve generative indices and multiplication | effects_resource_limits_preserve_exported_module_and_component_indices |
| Export-only instances do not append generative indices | effects_instance_from_exports_does_not_append_generative_indices |
| Parent replays the real guest and receipts | real normalized guest CLI collect and artifact verify; wasm_artifact_receipt test |

The clean-worktree final transcript and initial-attempt records live under
`evidence/2026-10-28-effects-verification/`. Passing bounded static checks do not
prove product execution or lifecycle release readiness.
