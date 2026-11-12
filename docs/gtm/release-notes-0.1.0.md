# Octet 0.1.0

> Draft release body. Do not publish until
> [launch-checklist.md](launch-checklist.md) is green and the release owner has
> replaced the qualification block with the final tag's receipts. The old
> `becae962` pin is blocked after nested Git history exposed excluded prose.

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. Existing Rust source runs through pinned static checks,
checked-in policy, and optional scoped proof rails, then produces local evidence
that a CI or release gate can review.

## Install

The supported public channel is a source-only export pinned to a reviewed
revision. Pin the full revision; `main` is not a supported pin.

```bash
nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version
```

Expected:

```text
cargo-octet 0.1.0
```

Run a check from a consumer workspace:

```bash
nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- check
```

Consumer setup:

1. Copy `templates/dylint.toml` to the workspace root as `dylint.toml`.
2. Add runner scope to the workspace `Cargo.toml`:

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run the pinned command above.

For a local checkout, replace the pinned flake reference with
`path:/path/to/octet#cargo-octet`. Direct Cargo installation is an advanced path
that must reproduce the reviewed pins.

## Highlights

- **77 Dylint lints across six domains**: safety 27, naming 9, performance 2,
  structure 21, purity 16, and concurrency 2.
- **Rust-compatible checked path**: no language rewrite and no new syntax.
- **Reviewable artifacts**: `status.json`, `summary.txt`, `results.sarif` or
  `results.jsonl`, and `provenance.jsonl`.
- **Artifact verification**: `cargo octet artifact verify --artifact-dir
  artifacts/octet` recomputes saved sidecar identities and fails closed on
  stale, malformed, or inconsistent evidence.
- **Strict profiles**: `robust`, `bounded-systems`, and `deterministic-core`.
- **Checked-in policy**: `dylint.toml`, `[workspace.metadata.octet]`, and
  Nickel-authored standards policy.
- **Optional proof rails**: Verus, Lean/Aeneas, Kani, and external evidence
  rails remain opt-in and scoped.
- **Local or consumer CI**: no Octet-operated upload service is required.

## The checked path

```text
Rust source
  -> cargo octet check
  -> checked-in policy and config
  -> evidence artifacts
  -> artifact verification
  -> consumer CI or release gate
```

The result is reviewable because each claim points to a command, policy file,
artifact, or receipt. The default path is static and policy evidence. It is not
a whole-program proof, a certification, a security score, or a release
eligibility decision.

## What ships

| Output | Role |
| --- | --- |
| `cargo-octet` | Runner subcommand |
| `cargo-slotcar`, `cargo-tigerstyle` | Compatibility subcommand aliases |
| `octet` | Lint library |
| `slotcar`, `tigerstyle` | Compatibility lint-library aliases |
| `dylint-driver` | Dylint driver |
| `octet-standards` | Standards and policy checker |
| `slotcar-standards`, `tigerstyle-standards` | Compatibility standards aliases |
| `octet-policy-artifacts` | Generated policy artifact producer |
| `octet-policy-export` | Policy export |
| `octet-assurance-profile-onixos-lifecycle` | Assurance profile artifact |

## Release qualification

This section describes the reviewed public cohort
`octet-public-becae962-2026-11-11`. Replace it with the final tag's receipts
before publishing.

| Receipt | Outcome |
| --- | --- |
| Flake resolution | Resolved the pinned public revision and its nar hash |
| Package build | `cargo-octet`, `octet`, `dylint-driver`, `octet-standards`, and `octet-policy-artifacts` built from public inputs |
| Runner version | `cargo-octet 0.1.0` |
| Policy artifacts | Five generated policy artifacts plus manifest |
| Walkthrough | Observed `unchecked_division`, applied the guarded fix, then clean run |
| Artifact verification | `status=valid` |
| Tamper rejection | Rewritten `results.sarif` produced `status=invalid`, exit 1 |

The full `nix flake check -L` gate was not claimed for this cohort. Only the
source-resolution, package-build, runner, policy-artifact, walkthrough, and
artifact-verification receipts above ran, and only for `x86_64-linux`. See
[`docs/release-candidate.md`](../release-candidate.md) for the full checklist
and the development-receipt non-evidence list.

## Requirements and limits

- Nix is the supported public install path. The reviewed cohort was verified on
  `x86_64-linux`.
- Octet uses Dylint and a pinned nightly Rust toolchain. This is a deliberate
  tradeoff for compiler-internal checks.
- Crates.io publication is not prepared.
- No binary distribution is offered for this release.
- Before publication, the source export and every embedded Git history must
  exclude imported standards prose pending permission. The `becae962` export
  fails that condition. Preserve `NOTICE`, `THIRD_PARTY_NOTICES.md`, and
  `LICENSES/` in its clean replacement.
- Octet is `AGPL-3.0-or-later`. Running Octet does not relicense the code being
  checked.
- Static lint cleanliness is not whole-program correctness. Optional proof
  receipts are scoped to their declared property, verifier, source binding, and
  artifact.
- The Aspen phase-1 starter rollout is historical evidence for its archived
  scope, not a claim about current Aspen or future consumers.

## Compatibility

`cargo slotcar` and `cargo tigerstyle` remain compatibility aliases. Legacy
config sections and environment prefixes remain lower-precedence aliases. The
canonical names are Octet, `cargo octet`, `[workspace.metadata.octet]`,
`[octet]`, and `OCTET_*`.

## Links

- Repository: <https://github.com/OnixResearch/octet-public>
- First checked Rust patch:
  [`docs/examples/first-checked-rust-patch/README.md`](https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15/docs/examples/first-checked-rust-patch/README.md)
- Consumer guide:
  [`docs/consumer-guide.md`](https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15/docs/consumer-guide.md)
- Trust boundary and evidence tiers:
  [`docs/trust-boundary-evidence.md`](https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15/docs/trust-boundary-evidence.md)
- Release checklist:
  [`docs/release-candidate.md`](https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15/docs/release-candidate.md)
- Security policy:
  [`SECURITY.md`](https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15/SECURITY.md)

## License

Octet's original material is licensed under `AGPL-3.0-or-later`. See `LICENSE`,
`NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`. Dependencies and imported
material retain their own terms.
