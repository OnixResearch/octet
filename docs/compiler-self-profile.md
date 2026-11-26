# Compiler Self-Profile and Octet Phase Attribution (#25)

`cargo octet benchmark self-profile` adds an optional diagnostic profiling mode
using `rustc -Z self-profile` (Measureme `MMPD` v9 format) together with explicit
bounded Octet phase timing/count spans.

## Design Boundaries

1. **Optional diagnostic mode**: Ordinary `cargo octet check` and `cargo octet admit`
   runs do not enable `-Z self-profile` or emit diagnostic phase spans.
2. **Instrumented vs. uninstrumented separation**: Uninstrumented runs provide the
   wall-time baseline and verify that semantic/policy outputs remain identical when
   profiling is disabled.
3. **Wall time vs. query/phase sums**: Reports expose elapsed wall time, `rustc`
   self-profile query/activity sums, and Octet phase spans separately so nested or
   parallel spans are never summed as elapsed wall time.
4. **Warm no-op vs. controlled rebuild**: Warm runs explicitly distinguish `compiled`
   vs `not_compiled` units; a warm no-op Cargo run is recorded as `not_compiled`,
   never as zero-cost collection.
5. **Attribution limits**: `rustc`'s `early_lint_checks` / `late_lint_checks` queries
   include compiler traversal overhead and cannot isolate custom lint cost alone;
   Octet-specific phase spans isolate selected MIR analysis, shard serialization,
   shard normalization, and policy evaluation where instrumented.
6. **Replay verification**: `--verify` validates retained `.mm_profdata` BLAKE3
   identities, compiler/unit/source bindings, and phase-span digests, and re-parses
   raw Measureme data to regenerate deterministic query/event counts without claiming
   timing values themselves are deterministic across re-executions.
