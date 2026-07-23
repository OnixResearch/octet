# Design

## Success contract

Goal: remove one isolated `non_trait_imports` identity from Octet's canonical self-check while proving monotonic baseline reduction and unchanged CLI exit behavior.

Observable completion evidence:

- `cargo-octet/src/lib.rs` uses explicit `std::process::ExitCode` owner paths and has no private `ExitCode` import, suppression, alias, or exception;
- the old baseline reports exactly one stale identity (`b3:8515683389cc483f`), zero new identities, and 2,669 warning occurrences;
- the reviewed replacement baseline is exactly the old stable-ID set minus that identity, contains 1,819 unique identities, and produces zero new or stale identities;
- `non_trait_imports` decreases from 1,114 to 1,113 occurrences without changing the shell's success, integration-error, or delegated exit values;
- CLI positive/negative tests, the permanent positive/negative self-check gate, source-corpus freshness, formatting, Clippy, Nextest, policy freshness, traceability, and the broad compatible-system flake gate pass.

False completion includes deleting the baseline record before observing it stale, accepting any new identity, introducing an alias that evades the lint, changing lint policy or exceptions, suppressing findings, changing exit behavior, relying only on aggregate counts, or claiming that one resolved warning establishes broader correctness.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| isolated CLI exit path | Replace the one-line import with an implementation marker and qualify the public return plus constructors | active | The file has no whole-file lint; its three ambient-environment findings do not overlap the edited exit-code expressions |
| domain formatter path | Qualify the private `std::fmt` import | falsified | Old-baseline run returned 2 new and 3 stale IDs because file-length and filename lints hash full source bytes |
| owner-qualified build shell | Qualify four build-script imports | falsified | Old-baseline run returned 9 new and 13 stale IDs because ambient-env, panic, unwrap, and borrowed-signature snippets changed |
| local type/module aliases | Replace imports with aliases declared in the file | rejected | Hides owner paths and evades rather than satisfies the rule |
| lint exception or source allow | Add configuration or attributes | rejected | Leaves warning debt accepted and weakens policy |
| repository-wide rewrite | Qualify all 1,114 occurrences at once | rejected | Excessive review surface and high incidental identity/regression risk |

An advisory secondary review favored the build-shell candidate, but deterministic old-baseline evidence falsified it. Compiler, self-check, baseline-set, and repository-gate evidence remain authoritative.

## Functional core and imperative shell

No decision core changes. `run_cli_from_env` remains a thin imperative shell over `try_main`: success codes still convert through `u8`, integration errors still print once and return the named error exit, and subcommand decisions stay in existing cores. Only lexical owner qualification changes.

The existing pure stable-ID set comparison remains authoritative. The reduction is observed first with the old immutable baseline, then the runner writes a candidate baseline. A deterministic set-difference check must show no additions and exactly the reviewed removal before replacement.

## Validation sequence

1. Preserve pre-change CLI and permanent self-check results.
2. Compile and test the owner-qualified shell.
3. Run the canonical self-check against the old baseline and require one stale ID, no new IDs, exit `0`, and warning-only status.
4. Generate a candidate baseline, compare identity sets, and update exact documentation and Nix assertions only after review.
5. Refresh source-derived corpus evidence, re-run positive and one-new-ID negative gates, then repository-wide validation.

## Risks and claim boundary

Changing a public return type's spelling must not change its Rust type or exit mapping. Focused tests cover successful and malformed CLI paths, while compilation checks every constructor. Editing a selected source file changes function-object corpus identity; the repo-owned refresh test must update only source-derived receipts and preserve their non-claims.

The bounded search budget is six mechanism families, one independent advisory audit, three old-baseline observations, one candidate refresh, and three validation rounds. Allowed outcomes are validated, blocked with an exact new/stale/exit discrepancy, exhausted, or user-decision-required.

The result proves only removal of one stable warning identity from the pinned workspace-library self-check. It does not prove the remaining warnings correct, CLI correctness, lint completeness, runtime safety, whole-program correctness, or release eligibility.
