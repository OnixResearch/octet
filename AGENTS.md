# Agent Notes

## Downstream rollout evidence
- Use `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical Aspen phase-1 starter-rollout reference. Current `/home/brittonr/git/aspen` scripts may already include later scope and lint promotions. Replay helpers and checkpoint notes must cite that archive path explicitly, not just a local Aspen worktree.
- Consumer rollout guidance now lives in `docs/downstream-rollout.md`, with copyable templates in `templates/tigerstyle-check.sh` and `templates/dylint.toml`.
- Template default `../tigerstyle` only works when the consumer repo is a direct sibling of `tigerstyle`; nested consumers like `mc/valence` need script-side auto-detection (for example `../../tigerstyle`) or an explicit `TIGERSTYLE_PATH`.
- To reproduce Aspen phase-1 evidence, use Aspen commit `909e5e9fa`. A naive temp worktree fails because Aspen has sibling path dependencies (`aspen-dns`, `aspen-wasm-plugin`); use a sandbox parent with `aspen/` worktree plus sibling symlinks.
- If a change archives helper scripts and transcripts, regenerate the `.txt` outputs from the archived location before committing so saved commands point at `openspec/changes/archive/...`, not the deleted active change path.
- If the exact replay proof uses a helper script, document that helper command as the exact replay path and prepend the same command line to the saved transcript.
- If a saved replay narrows the enabled lint set through per-lint config, make the transcript self-describing by patching the copied wrapper banner for that run or saving the applied `dylint.toml` snapshot alongside the log.
- Saved check helpers must assert every printed summary claim, including zero-warning claims like `aspen-hlc: 0 starter findings`.
- If a transcript starts with `$ bash ...`, store and document the capture helper or exact shell wrapper that emits that preamble; do not claim a quieter inner helper produced it.

## Build / test
- Run Rust builds inside the nix devshell: `nix develop -c bash -c '...'`. Plain `cargo` outside the shell can miss OpenSSL/pkg-config and rustc-private setup needed by Dylint.
- When shared `~/.cargo-target` lock contention or unrelated repo builds interfere, use a temporary `CARGO_TARGET_DIR` with trap-cleanup for reviewable `cargo test --lib` / `cargo test --test ui` runs.
- After adding or removing Cargo dependencies, run `cargo generate-lockfile` before trusting fresh `--locked` Nix builds or `nix flake check`.
- When review needs proof of specific UI cases here, make the saved UI transcript print each fixture name and save a separate artifact that prints snapshot contents; a bare `test ui ... ok` transcript is not enough.
- The custom `tests/ui.rs` harness does not compile `#[cfg(test)]` fixture code unless that fixture gets an explicit `--cfg test`; add per-fixture rustc args when you need to exercise cfg-gated test modules.
- Test-context suppression must treat only cfg predicates that require `test`; a token match on `test` is too broad and wrongly suppresses cases like `#[cfg(not(test))]`.
- Early-pass line-scan fallback for test context must recognize both outer `#[cfg(test)]` and inner `#![cfg(test)]` attributes when recovering stripped cfgs from source text.

## Lint registration / config
- Do not implement per-lint severity by registering cloned `Lint` descriptors with patched `default_level`; passes still emit the original static lint IDs, so `allow`/`deny` overrides silently fail.
- In this repo, per-lint severity overrides must keep original lint statics registered and feed effective levels through rustc lint opts before lint-level computation.
- Do not route pass registration through a runtime function-pointer inventory table here; it breaks source-attribute activation for allow-by-default lints in standalone UI fixtures. Keep `LINT_INVENTORY` as the source of truth, but generate explicit registration code from it.
