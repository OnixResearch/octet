## Context

The owning requirement `octet.purity_effect_lints.missing_const_fn.allows_non_const` forbids findings for non-const calls and trait methods. Source arithmetic/comparison hints currently ignore those disqualifiers. The exact consumer report is `/tmp/hwr-component-proof-20261016.5bwWjw/bounded-profile-octet.json`; the predicates at component.rs:66 and :79 include bytes/all/split_once/parse and closures.

## Decisions

Keep source guardrails as a cheap prefilter. Add a HIR visitor to admit only primitive-valued literals, paths, built-in unary/binary operations, casts, blocks, returns, if/match/let, and compiler DropTemps wrappers. Reject unsupported expressions and any type-dependent operator resolution. Calls (including const calls) are deliberately outside this conservative subset: absence of known effects is not const proof. Primitive-value checks reject overloaded/domain types and destructor-bearing expression values. Restrict the lint to `FnKind::ItemFn`, excluding methods and closures. No automatic const rewrite is emitted.

Reuse the existing UI fixture so its arithmetic positive snapshot remains stable. Add negative cases for string iterator/closure, split/parse, arbitrary call, generic overloaded addition, and a trait implementation; all remain consumer-visible examples of non-const functions, not source-text tests.

## Other diagnostic decisions

`concrete_construction_in_core`'s current contract flags direct new/default producing external ADTs, exempting local and core/alloc/std value types only. The self-architecture policy already reviews blake3 as pure-value, but is project-specific and not supplied to this lint. There is no sound reusable authorized HWR classifier input in the current lint API. Preserve the finding and report the policy-input decision blocker rather than hardcode a cryptographic type or invent a new exemption.

`path_segment_repetition` explicitly includes ancestor crate/module words for public names and has documented compatibility handling. `ComponentLimits`, `ComponentProfile`, `AdmittedComponentProfile`, and `admit_component_profile` meet that rule; no consumer API rewrite is authorized merely to satisfy style. `assertion_density` counts assertions in nontrivial bodies over its threshold, including fallible parsers. The 27-line versioned-interface parser has zero assertions; policy currently does not equate fallible validation with internal invariants. Preserve both rules.

## Verification

After source is complete, exercise the complete existing const fixture using the actual owning lint library and source-built owning UI driver, preserving the existing harness's flags, normalization and exact unchanged snapshot comparison; also compile the fixture to validate the negative examples. Parent owns integrated/full-suite validation and downstream deny-all replay. No claims of passing checks until executed. Native proposal/design/tasks gates precede implementation; receipt outputs are retained in change evidence. A retained exact owning Nix environment is sufficient for scoped source/registration proof, not for fresh-environment or anonymous/public acquisition claims. This change does not authorize archive or publication.
