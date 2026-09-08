# Preserve anonymous compiler body facts without invalid queries

## Why

The published collector treats every unregistered, non-closure body owner as a function. Seaglass reaches this fallback through a tracing macro's anonymous constant. Rustc then panics on `fn_sig(AnonConst)`. The panic exists before Seaglass's module split.

Changing consumer const helpers or suppressing logging does not repair the producer. Anonymous bodies also lack a supported visibility query.

## What Changes

Classify fallback body owners from rustc's `DefKind`. Query signatures only for function kinds. Record anonymous and inline constants as private, without a visibility query. Preserve their call facts and existing identity rules.

Reuse the two narrow corrections from Octet's local source-ownership work. Do not import its unrelated type-dependency schema or policy changes. Add compiler-backed regression controls to the existing all-target fixture.

## Impact

No new configuration, schema, dependencies, or lint allowances. The producer repair does not establish Seaglass architecture acceptance. Seaglass receives a published immutable producer only after verification.
