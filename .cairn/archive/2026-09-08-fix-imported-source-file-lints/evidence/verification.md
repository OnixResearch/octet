# Verification

## Scope and result

The imported-source ownership defect is repaired. Both source-map passes now reject compiler-imported SourceFile values before any path or filename decision.

The predicate uses rustc SourceFile::is_imported, not directory names. A local file remains eligible even when its path resembles a patched dependency. Existing local filename rules and module-count limits remain unchanged.

## Baseline

The published associated-item fix `08f225f1310838bdb97ca80519f2038196232a78` passed the dedicated compiler-architecture Nix check. Seaglass nevertheless failed compilation with 20 filename findings from patched Polars files during dataframe-model compilation.

The focused filename baseline passed four library tests. Its path-only controls did not catch imported files inside the workspace.

## Repair controls

- Full lint-library suite: 148 passed.
- New source ownership controls admit local source and reject the same source filename after rustc marks its source as imported.
- A new valid-local-filename control passes. The existing invalid-local-filename control remains actionable.
- Octet all-target Clippy with `-D warnings`: passed.
- Cargo formatting check: passed.
- Nix builds for cargo-octet, compiler-architecture-targets, and project-architecture-policy: passed.
- The compiler fixture retains its missing-shard rejection control.

## Consumer replay

A Seaglass replay used the built local producer through an explicit test-only package override. The prior 20 imported-file findings disappeared. The run advanced to 12 genuine seaglass-config-nickel findings and returned integration-failure.

This is not Seaglass architecture acceptance. It proves removal of the observed producer defect without hiding consumer findings. No lint level, warning budget, or source allowlist changed.

## Exact commands

Owner commands ran from the dedicated Octet worktree:

```sh
nix develop -c sh -c 'set -eu; export RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/home/brittonr/.cache/octet-architecture-targets/build; cargo test -p octet --lib; cargo clippy -p octet --all-targets -- -D warnings; cargo fmt --all -- --check'
nix build .#cargo-octet .#checks.x86_64-linux.compiler-architecture-targets .#checks.x86_64-linux.project-architecture-policy --no-link -L
```

The Seaglass probe ran `scripts/check-octet-project-architecture.sh` from its architecture worktree with `OCTET_ARCHITECTURE_PACKAGE=git+file:///home/brittonr/git-worktrees/octet-imported-source-20260908#cargo-octet`. That override is probe evidence, not a durable consumer dependency.

Compressed transcripts accompany this report. Raw logs also remain under `/home/brittonr/.local/state/seaglass-drain-20260908/`. Failed consumer transcripts are intentional negative evidence, not acceptance claims.
