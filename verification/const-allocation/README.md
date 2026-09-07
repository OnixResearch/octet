# Const suggestions and runtime formatting

## Repair boundary

Final implementation: `184793ba3b3bef375dbe62daaf996793b929b0f5`, based on the retained driver repair at `2df5305f4799c9b962cd0f7471dbbae2b6e8d620`.

The source heuristic missed nested `Err(format!(...))` because its formatting exclusion required preceding whitespace.
The repair adds a HIR visitor to `MissingConstFn` in `src/purity/effect_lints.rs`.
It resolves direct calls and rejects candidates that call `alloc::fmt::format`.
It compares definition-owner components, not source spelling or rendered diagnostic paths.
Optional item-name queries handle unnamed definition parents without panics.
The visitor skips closure bodies, which are separate functions.

The existing source filters, test-context rules, lint levels, and registration remain unchanged.
This is a focused runtime-formatting exclusion, not a complete proof of const eligibility or transitive effect analysis.
No consumer source, scope, hook, suppression, pin, or startup guard changes in this repair.

## Build and focused tests

The user authorized the repair. Each Nix plan was inspected before realization.
The initial plan built six derivations: three manifests, one source, dependency artifacts, and the lint library.
The next two plans each built only one lint-library derivation.
Nix used offline mode, no substitutes, no remote builders, one build job, two cores, and bounded timeouts.
No compiler, driver, CLI, Verus, Mantle, Darkhttpd, or package toolchain was rebuilt.

The final build (task 10816) passed all seven `purity::effect_lints::tests` tests.
It completed in 23.469 seconds. Systemd resource counters cover the Nix client, not daemon compiler work.
The existing March-21 compiler was reused.

```text
Derivation: /nix/store/1whqaiyfz8lkn9ppqnh6mdliqnw2hsdr-octet-0.1.0.drv
Library: /nix/store/frd81n941m079sbb9jcq6414cdqnz1m6-octet-0.1.0/lib/liboctet.so
BLAKE3: 796bff49b55751b24dbad6ce10eace12ecd22e6a0e147b5e81e479b350756aaa
```

This output comes from the focused `check.nix` override, not the default package derivation.
Nix content verification passed with `--no-trust`. A private GC root retains the library.
These facts do not establish signatures, reproducibility, whole-tree equivalence, or approved runtime provenance.

## Direct controls

Replay with existing tools and a fresh output directory:

```sh
sh verification/const-allocation/verify.sh OUT DRIVER LIBRARY_FILE COMPILER_PACKAGE
```

All arguments must be absolute. The helper builds no tools and rejects compiler crashes.
Task 10818 passed ten controls:

- Nested formatting, qualified formatting with comments/braces, and an imported macro alias: exit 0.
- An existing const function: exit 0.
- Arithmetic, format-like string text, a custom const function named format, and an associated const function: the missing-const diagnostic remains.
- Bare `todo!()`: the no-todo diagnostic remains.
- Formatting in a declared const function: rustc still rejects it with E0015.

These are diagnostic fixtures. They do not replace the unchanged production command or prove zero findings across the lint catalog.
The original library fails the same helper on the nested-format case (task 10811).

## Retained unsuccessful attempts

The first library compared a rendered definition path. Unit tests passed, but the nested-format control still failed (tasks 10805–10806).
The compiler HIR dump identified the allocation definition. The next revision compared definition-owner components instead.

That revision passed nine controls but crashed on Molten while querying the name of an unnamed impl parent.
The failed consumer attempt is `lifecycle-source-gate-7` (task 10814).
Its ten recorded findings are an incomplete crash result, not a reduction from the prior 93 findings.
The new associated-function control reproduces that crash with the intermediate library (task 10817).
The final revision uses `opt_item_name`; all ten controls pass.

## Consumer and remaining limits

Molten repeats its unchanged canonical command on frozen source `22f3d3a5ebb53050848c6648dfc596481e9e8b12` with the new library explicitly selected.
That diagnostic does not promote the mixed tool set to an approved startup cohort.
The compiler, repaired driver, CLI, hook, metadata, and source archive are unchanged from the preceding diagnostic.
Task 10820 completed the final Molten attempt in 53.924 seconds without a compiler crash.
It reported 89 node-host errors and zero warnings, compared with 93 errors before this repair.
Four runtime-formatting false positives are gone. Three other const suggestions remain outside this fix.
The gate still exits 2 (Cargo 101), so startup and complete workspace coverage remain blocked.

Full Octet UI/workspace/flake acceptance and full Clippy were not established here.
The pre-existing UI vendor mismatch remains outside this repair.
No startup authority, production build, VM, native replay, physical deployment, main integration, or release promotion is claimed.
