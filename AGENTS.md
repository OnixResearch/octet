# Agent Notes

## Downstream rollout evidence
- Use `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical Aspen phase-1 starter-rollout reference. Current `/home/brittonr/git/aspen` scripts may already include later scope and lint promotions.
- Consumer rollout guidance now lives in `docs/downstream-rollout.md`, with copyable templates in `templates/tigerstyle-check.sh` and `templates/dylint.toml`.
- Template default `../tigerstyle` only works when the consumer repo is a direct sibling of `tigerstyle`; nested consumers like `mc/valence` need script-side auto-detection (for example `../../tigerstyle`) or an explicit `TIGERSTYLE_PATH`.
- To reproduce Aspen phase-1 evidence, use Aspen commit `909e5e9fa`. A naive temp worktree fails because Aspen has sibling path dependencies (`aspen-dns`, `aspen-wasm-plugin`); use a sandbox parent with `aspen/` worktree plus sibling symlinks.
- If a change archives helper scripts and transcripts, regenerate the `.txt` outputs from the archived location before committing so saved commands point at `openspec/changes/archive/...`, not the deleted active change path.
- If the exact replay proof uses a helper script, document that helper command as the exact replay path and prepend the same command line to the saved transcript.
- Saved check helpers must assert every printed summary claim, including zero-warning claims like `aspen-hlc: 0 starter findings`.
- If a transcript starts with `$ bash ...`, store and document the capture helper or exact shell wrapper that emits that preamble; do not claim a quieter inner helper produced it.

## Build / test
- Run Rust builds inside the nix devshell: `nix develop -c bash -c '...'`. Plain `cargo` outside the shell can miss OpenSSL/pkg-config and rustc-private setup needed by Dylint.
