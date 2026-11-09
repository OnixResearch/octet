# Exercised verification

Immediate outcome: the actual owning classifier now accepts exact project-reviewed value ADTs without making a crate-wide or severity exception. The reusable mechanism is maintained by Octet; HWR owns its separate `blake3::Hasher` declaration and dependency-cohort review.

## Final-source observations

`final-owning-verification.json` records actual argv, exit, stdout and stderr for:

- `cargo fmt --all -- --check`: exit 0.
- `cargo build -p octet --lib --locked --offline`: exit 0, complete actual owning library.
- `cargo test -p octet --lib --locked --offline`: 145 passed.
- `cargo test -p octet --test ui --locked --offline -- ui --ignored --exact --nocapture`: pass; all 155 maintained fixture names were observed, including the new custom case. The source-built owning UI driver loaded the newly built library, not the old packaged library or a source-isolated replacement.

The unchanged default construction and const snapshots matched in the initial actual-driver smoke. The custom case admits the selected Map and its renamed import, but retains exactly one construction finding for unselected Value from the same dependency; the independent explicit-defaults diagnostic still fires. `combined-compiler-fixtures.json` retains that initial source-bound smoke. Final full-corpus proof supersedes it after a formatter-only change to the constructor's Self literal.

`exact-match-boundaries.json` records actual compiler smoke: absent, crate-only, prefix, suffix and extended facts each retain three construction findings; an exact Map definition path retains only the unselected sibling finding. These smoke files were temporary and removed automatically. Initial scratch setup omitted Dylint's configuration binding and therefore observed zero construction findings; it was corrected using the actual owning Dylint source's documented raw `DYLINT_TOML` input, not a classifier workaround. The corrected observations, not that setup error, support the boundary claims.

`inputs.blake3` binds final source, manifest/lock, toolchain, fixture configuration, new snapshot, actual library and source-built driver. The final library identity is `ee9d0384b9ba145e3685ceb5da93f61f059a579981d3895268372985db7b0f18`.

## Repeatability and limits

Commands ran from the Octet root in the actual retained owning Nix environment exported by the const-classifier worker, using nightly-2026-03-21 / rustc ac7f9ec7da74d37fd28667c86bf117a39ba5b02a. Full tests inherited the normal verified owning vendor source tables through a temporary Cargo home, plus the expressly authorized same-revision HTTPS source table for bounded-exec 44eb74eb639194665ee820562d5b2ce2c3ff6a94. Original directory sources and Cargo checksums were unchanged. Cargo was offline; credential helpers, SSH/askpass and compiler wrappers were disabled. No manifest/source/package patch or alternate compiler was used.

This is actual source/registration/UI evidence in a retained environment. It is not fresh Nix entry, anonymous public acquisition, full-workspace/Nix release proof, HWR downstream proof, semantic purity or provider identity verification. Octet's pre-existing standards-prose and publication prerequisites remain in force. No archive, push or visibility change is authorized by these observations. Parent retains final HWR runner replay, exact source admission and integrated release gates.
