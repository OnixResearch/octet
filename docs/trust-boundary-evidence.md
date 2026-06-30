# Trust-boundary and evidence path

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. It routes source through local checks, policy/config, and
reviewable artifacts before a team promotes the result into CI or release gates.

This page is an index and scope map. It links concrete Octet surfaces instead of
creating new claims.

## Source-to-gate flow

```text
Rust source
  -> cargo octet check / octet-standards checks
  -> checked-in policy and config (`dylint.toml`, `[workspace.metadata.octet]`, Nickel registry exports)
  -> evidence artifacts (`status.json`, SARIF, JSONL findings, transcripts, fingerprints, policy bundles)
  -> consumer CI or release gate
```

The same route applies to code written by agents and code written by humans. The
path is evidence-backed because every gate points to a command, policy file, or
artifact a reviewer can inspect.

## Evidence stack

The Octet evidence stack has four layers. Each layer names what it contributes
and where its boundary sits.

| Layer | Concrete Octet surfaces | Scope boundary |
| --- | --- | --- |
| discipline | Tiger Style guidance, closed-interface review rules, assertion and safety-critical guides | names review obligations and engineering discipline; it is not a substitute for domain review |
| static edge | Dylint lint domains, `cargo octet check`, SARIF and JSONL finding sidecars | rejects or reports known source shapes; lint cleanliness is not whole-program correctness |
| policy layer | Nickel-authored policy registry, `octet-policy-validate`, `octet-standards check`, generated policy bundles | validates declared policy/config contracts and source-shape gates; policy checks do not validate business requirements |
| evidence chain | `status.json`, `summary.txt`, transcripts, semantic fingerprints, policy artifact manifests, lifecycle evidence | preserves reviewable proof of what ran and what it inspected; evidence still needs human interpretation for product risk |

Formal proof artifacts, when present, stay explicitly scoped to their verifier,
model, receipt, and covered property. Other tiers remain validation or review
evidence and must not be upgraded by wording alone.

## Evidence tiers

Emitted evidence records use bounded, machine-readable tiers when the artifact
format carries tier data:

- `static_lint` — a Dylint or compiler lint finding from source analysis.
- `policy_check` — a standards or registry policy evaluation.
- `ui_fixture` — a checked UI fixture or diagnostic snapshot.
- `autofix_verified` — an autofix replay that rechecks cleanly.
- `transcript_assertion` — an executable transcript assertion with expected output.
- `formal_proof` — a proof result with verifier identity and proof artifact identity.
- `manual_review_required` — a boundary where a named human action is still required.

Validation is validation, review is review, and proof is proof for a stated model
and property. No tier may be silently promoted to a stronger claim.

## Provenance chain

A provenance record connects a claim to its source and artifact:

1. source ID: requirement, policy, lint, or transcript assertion;
2. checker ID: the tool or review surface that produced the evidence;
3. evidence tier: one of the bounded tier values above;
4. artifact locator: path or URI for the deterministic artifact;
5. BLAKE3 content hash: identity of the exact artifact bytes;
6. checked scope: source root, crate, policy family, or transcript section.

Release operators inspect the chain from requirement or policy ID to artifact,
then verify that the artifact hash, checker identity, tier, and scope match the
claim being made.

## What Octet does not claim

Octet is useful because its boundaries are explicit:

- Octet does not prove whole-program correctness. It does provide static
  rejection of known-dangerous source shapes, policy validation, and artifacts
  such as SARIF, JSONL findings, transcripts, and fingerprints.
- Octet does not validate business or domain requirements. It does preserve
  requirement and policy IDs so reviewers can trace which engineering claim an
  artifact supports.
- Octet does not make suppressions correct merely because they are present. It
  does require reviewable suppressions, narrow reasons, and follow-up gates for
  teams that opt into those checks.

A clean run is an input to review, not a complete assurance argument.

## Controlled deployment posture

Octet is designed for local or consumer-CI execution:

- local execution: `cargo octet check` runs in the consumer workspace;
- consumer CI execution: Nix and Cargo entry points can run under the team's own
  CI boundary;
- no required hosted service: Octet does not require uploading source or
  artifacts to an Octet-operated service;
- pin-capable runner path: consumers can pin `github:OnixResearch/octet` to a
  release tag or commit, or use `path:/path/to/octet#cargo-octet`;
- source-visible toolchain: this repository contains the runner, lint library,
  standards checker, policy registry, and generated artifact seam;
- local artifact directory: `cargo octet check --artifact-dir artifacts/octet`
  writes review files plus `provenance.jsonl` to a configured local path.

Boundary caveat: Octet itself does not require uploading source, logs, or
artifacts. Consumers still own what their CI provider, log collector, artifact
store, or downstream integrations do after Octet writes local files.

### Regulated-consumer checklist

- Pin the Octet revision or local path used by `nix run ...#cargo-octet` or the
  pre-commit hook `rev`.
- Check in consumer-owned `dylint.toml` and `[workspace.metadata.octet]` runner
  scope.
- Configure a deterministic artifact directory such as `artifacts/octet` for
  `command.txt`, `status.json`, `summary.txt`, `provenance.jsonl`, and structured output.
- Optionally add `templates/pre-commit-config.yaml` as a deny-all pre-commit
  gate after the starter profile is understood.
- Save or reference the release/operator smoke command when promoting the gate.

## Trust-boundary and evidence-path index

| Theme | Where to start | Concrete surface |
| --- | --- | --- |
| checked-path framing | [README quick start](../README.md#quick-start) and [consumer guide](consumer-guide.md) | `cargo octet check`, `octet-standards check`, source-to-gate flow |
| evidence stack | [Evidence stack](#evidence-stack) | discipline, static edge, policy layer, evidence chain |
| evidence tiers | [Evidence tiers](#evidence-tiers) | `evidence_tier` fields in JSONL/SARIF sidecars and provenance DTOs |
| provenance chain | [Provenance chain](#provenance-chain) | `provenance.jsonl` records source ID, checker ID, tier, artifact locator, BLAKE3 hash, and checked scope |
| scope-boundary | [What Octet does not claim](#what-octet-does-not-claim) and [safety-critical guide](safety-critical.md) | unsupported claims paired with supported capabilities |
| controlled-deployment | [Controlled deployment posture](#controlled-deployment-posture) and [downstream rollout kit](downstream-rollout.md) | local/CI execution, pinned runner, source-visible toolchain, local artifacts |
