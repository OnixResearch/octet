# Keep file-level lint findings within the current crate

## Why

Rustc source maps can contain files imported from dependencies. Octet currently treats those files as consumer files when they share a workspace directory.

Seaglass reproduces this defect with patched Polars dependencies. Twenty filename findings abort dataframe-model compilation. Filtering those findings afterward cannot establish successful compiler completion.

## What Changes

Use compiler source ownership to exclude imported files from file-level naming and module-count checks. Preserve checks for local files and existing scoped fixture exceptions.

Add accepted and rejected controls. Replay the published compiler fixture and the Seaglass consumer without weakening lint levels.

## Impact

The producer owns this correction. No new configuration, receipt schema, dependency, or lint suppression is required. Seaglass consumes a published immutable revision after verification.

The change does not complete Seaglass architecture, cancellation, or application-state contracts.
