# Design

## Decision

Use the compiler's source-file ownership facts at the lint shell boundary. An imported source file belongs to another crate even when its path is inside the workspace.

Keep deterministic path and filename decisions in the existing helpers. Do not classify ownership through hard-coded vendor directory names or a new allowlist.

Apply the same ownership boundary to the two passes that inspect complete source maps: underscore_in_module_filename and module_file_count.

## Verification

Establish the existing library-test baseline. Add a control that fails under the old source-map scan. Preserve an actionable local filename and an accepted local filename.

Build the pinned producer and run its compiler-coverage fixture, including the missing-shard rejection. Replay Seaglass using the built producer. The imported-file findings must disappear without suppressing genuine consumer findings.

Run relevant library tests, Clippy, formatting, and Nix checks. Record exact failures and scope limits. Publish only after the producer controls pass.

## Risks and limits

Path remapping and shared directories are not reliable ownership signals. Standalone Rust fixtures must retain local-file coverage. This change does not prove consumer architecture acceptance.
