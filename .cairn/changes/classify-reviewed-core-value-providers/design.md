## Context

The existing classifier distinguishes local ADTs, core/alloc/std ADTs and unclassified external ADTs. It operates on direct new/default result types. Its configuration already carries consumer-owned core source scopes, but cannot carry reviewed value facts.

## Decisions

Add one default-empty `Vec<String>` to the existing native Dylint config, `core_value_provider_types`. Pass it through the existing lint registration to `ConcreteConstructionInCore`. Extend the internal origin enum with `ReviewedValue`. Compare each configured path, split into `::` segments, to the compiler-resolved ADT definition path without joining or allocating comparison strings. Check only external direct constructors; do not turn the setting into a crate-wide, prefix, source-spelling or severity bypass. Missing or nonmatching facts leave the conservative existing classification unchanged.

The declaration is a project-maintainer assertion for an exact dependency cohort, not evidence that the dependency implementation is pure. It changes only this construction heuristic; effect, resource and architecture checks retain their own contracts. Consumers must review the actual implementation and maintain the declaration when dependencies change.

Reuse the existing default construction fixture unchanged. Add a separate fixture selecting `serde_json::map::Map`: direct and renamed Map constructors are clean, while `serde_json::Value::default()` remains reportable despite sharing the dependency. This tests consumer-visible default, identity/alias and sibling boundaries, not config forwarding. Run an additional actual-driver smoke with absent, crate-only and extended paths to prove exact matching without adding incidental config-copy unit tests.

## Verification

Native proposal/design/tasks gates precede source edits. Build the actual complete owning Octet library in the retained exact Nix environment, using the existing verified source transport mapping only if needed, then run the source-built owning UI driver against both constructor fixtures and the const fixture. Execute the configuration's changed path and compare actual compiler diagnostics with fixture snapshots. Parent then owns HWR deny-all replay with the reviewed `blake3::Hasher` fact and remaining coordinated producer gates. Retained-environment proof is not fresh anonymous acquisition or full-suite/release proof. Record exact commands and boundaries before closeout; do not archive or publish on structural gate results alone.
