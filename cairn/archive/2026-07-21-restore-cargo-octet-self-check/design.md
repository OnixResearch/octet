## Context

The runner's shell loads the nearest workspace `Cargo.toml` and requires a canonical Octet metadata table before resolving the toolchain, driver, lint library, or Cargo child process. The repository root has `[workspace]` but no `[workspace.metadata]`, so the current Nix app fails at this intended admission boundary. Existing consumer tests already prove that absent and malformed metadata fail with integration exit code `2`; the missing evidence is a checked root configuration and a packaged self-check gate.

## Success Contract

Completion requires all of the following:

1. Root `Cargo.toml` contains `[workspace.metadata.octet]` with exact no-argument scope `--workspace` and exact Cargo tail `--lib`.
2. A deterministic Rust test reads the checked-in root manifest and rejects missing, legacy-only, malformed, narrowed, or broadened values.
3. The packaged runner includes the linker tools required by the checked-in target configuration.
4. `nix run .#cargo-octet -- check` invokes the packaged runner without explicit scope overrides or policy weakening and exits `0` after a successful Cargo child process.
5. A hermetic Nix check reruns that positive path, proves a malformed metadata array exits `2` before a sentinel Cargo child can run, and proves an invoked non-zero Cargo sentinel becomes runner exit `2`.
6. Focused, broad, lifecycle, policy, and traceability gates pass or preserve an explicitly inherited baseline.

False completion includes adding top-level `[metadata]`, teaching the loader implicit defaults, selecting only one package without review, passing command-line scope overrides, using `--allow-style-violations`, testing only TOML parsing, accepting malformed arrays, omitting configured linker tools, returning success after a failed Cargo child, or treating a successful lint run as whole-program correctness or release eligibility.

## Portfolio Search

| Family | Mechanism | State | Discriminating evidence |
| --- | --- | --- | --- |
| Canonical root metadata plus execution gate | Configure the documented table and exercise the packaged no-argument path | selected | exact root-manifest assertion, exit `0`, successful summary, malformed negative with untouched Cargo sentinel |
| Loader-owned implicit defaults | Make missing metadata silently choose a scope | rejected | violates the documented consumer-owned, fail-closed configuration contract |
| Top-level legacy metadata | Add `[metadata]` merely to clear the current diagnostic | falsified | the diagnostic names the missing nested `workspace.metadata` key; the runner requires `[workspace.metadata.octet]` |
| Script-only explicit flags | Keep root invalid but pass `--workspace -- --lib` in contributor commands | rejected | does not restore the published no-argument command and bypasses checked workspace policy |

The search used four mechanism families, one advisory review, repository source inspection, binary-string inspection, and direct baseline execution. The selected family is accepted only after deterministic positive and negative gates; advisory output is not authority.

## Decisions

### Decision: Use Cargo's canonical workspace metadata surface

**Choice:** Add `[workspace.metadata.octet]` directly to root `Cargo.toml`.

**Rationale:** Cargo owns the manifest format, and Octet already documents and parses this exact interoperability surface. Generating the Cargo manifest through Nickel would add an unnecessary export layer and risk workspace drift.

### Decision: Check every workspace library by default

**Choice:** Set `default_scope = ["--workspace"]` and `cargo_check_args = ["--lib"]`.

**Rationale:** This covers all product library targets while keeping the no-argument gate bounded away from tests, examples, benches, and binaries. Narrowing to one package would understate repository coverage; `--all-targets` would materially expand the published default and increase fixture/test-only noise.

### Decision: Preserve fail-closed consumer behavior

**Choice:** Do not change the loader. Keep existing missing/legacy/malformed metadata errors and add a packaged negative gate using a malformed non-string array plus a Cargo sentinel.

**Rationale:** Restoring Octet's own configuration must not make missing consumer policy valid. The sentinel distinguishes parser rejection from a later Cargo failure.

### Decision: Package configured linker tools

**Choice:** Add Clang and Mold to `cargoOctetRuntimePath` alongside the existing Cargo, Rustup, Rust toolchain, C compiler, and evidence tools.

**Rationale:** Root `.cargo/config.toml` explicitly selects `clang` and `-fuse-ld=mold`. Build-time availability alone does not make those executables available to a consumer process launched by `nix run`.

### Decision: Keep the Nix gate hermetic and reviewable

**Choice:** The positive case uses the wrapped `cargoOctet` package against a writable copy of the filtered source and asserts the human summary, configured linker binaries, exact root linker settings, compile banners for all four workspace libraries, and the completed-build marker. One unwrapped negative uses malformed metadata and a sentinel `cargo`, then asserts exit `2`, the exact stable diagnostic, and absence of the sentinel marker. A second valid-metadata negative invokes the sentinel with a non-zero status and requires runner exit `2` plus an integration-failure summary.

**Rationale:** The wrapped path proves released environment wiring. The unwrapped negatives isolate pre-Cargo admission and post-Cargo status propagation without wrapper path injection hiding whether the child ran.

## Risks / Trade-offs

- The default checks libraries only; tests, binaries, examples, and benches remain covered by Nextest, Clippy, and broader Nix checks rather than this one command.
- Warning-only findings may still exit `0` by published runner policy; the gate proves successful execution and absence of error-level findings, not zero warnings.
- Local and Nix success prove behavior for the pinned source and toolchain only. They do not prove downstream configuration correctness, lint completeness, whole-program correctness, or release eligibility.
