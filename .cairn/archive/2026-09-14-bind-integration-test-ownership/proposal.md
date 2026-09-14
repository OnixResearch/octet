# Bind integration-test ownership before role interpretation

## Why

Noble's all-target gate cannot admit explicit integration-test source roles.
The current fail-closed rule is correct: normalized facts do not retain their compilation-unit memberships, and gate indexes select units by package.
Changing the package role or skipping test-looking paths would risk hiding production effects.

## What Changes

- Retain canonical fact and unknown membership for every observed compilation unit in versioned normalized Architecture IR.
- Bind independent Cargo integration-test roles to explicit source roots and compiler-selected targets.
- Evaluate production architecture over a typed production view, while retaining test observations, coverage, and explicit receipt dispositions.
- Keep conflicting production source roles, missing coverage, stale artifacts, and unsupported configurations blocking.

## Impact

Octet owns the compiler evidence and role interpretation. Noble remains a pinned consumer, not an implementation dependency.
Affected boundaries are Architecture IR, its generated contract, runner planning, project gate, and artifact replay.
This change does not relax the lint catalog, remove targets, introduce arbitrary mixed production roles, or establish Noble M1 acceptance.
The existing canonical specs are `.cairn/specs/compiler-architecture-ir/spec.md` and `.cairn/specs/project-architecture-gate/spec.md`.
