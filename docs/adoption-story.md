# Octet adoption story

Octet is a Rust-compatible, evidence-backed checked path for systems code before
release. Existing Rust source enters Octet checks without a rewrite, and the
result is reviewable because every claim points to a command, checked-in policy,
receipt, transcript, or artifact.

## Checked path promise

The adoption path is deliberately concrete:

```text
Rust source
  -> cargo octet check
  -> checked-in config or policy (`dylint.toml`, `[workspace.metadata.octet]`, policy bundles)
  -> evidence artifacts (`status.json`, SARIF, JSONL findings, transcripts, provenance)
  -> CI or release gate
  -> artifact verification before promotion
```

That path covers lint findings, standards checks, optional scoped proof receipts,
provenance, and artifact verification. It is an incremental Rust adoption path,
not a new programming language and not a Rust replacement.

## Built with Octet case-study format

A public "Built with Octet" case study is reviewable only when it names every
field below:

| Field | Required content |
| --- | --- |
| consumer or archive source | repository, commit, or archived evidence source |
| checked scope | package, crate, source path, policy family, or proof rail scope |
| command or gate | Octet command, CI check, transcript command, or release gate |
| configuration source | `dylint.toml`, `[workspace.metadata.octet]`, policy bundle, proof rail, or archived config snapshot |
| evidence artifact or archive path | transcript, receipt, sidecar, provenance file, policy artifact, or archive path |
| outcome summary | bounded findings, pass/fail status, suppression decision, or rollout status |
| scope boundary | what the evidence does not prove or what remains manual review |

### Built with Octet: Aspen phase-1 starter rollout

- consumer or archive source:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- checked scope: Aspen commit `909e5e9fa`, starter packages `aspen-time`,
  `aspen-hlc`, and `aspen-core`
- command or gate: archived phase-1 default run captured in
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/aspen-default-run.txt`
- configuration source:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`
  plus the archived wrapper/config snapshot named by
  `docs/downstream-rollout-evidence.md`
- evidence artifact or archive path:
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/evidence/validation-counts.txt`
  and the default-run transcript above
- outcome summary: the phase-1 starter run produced six findings, all
  `ambient_clock`, on the starter package set; that bounded signal informed the
  starter rollout profile
- scope boundary: this is historical rollout evidence for the archived starter
  scope, not Aspen HEAD, not whole-program proof, not certification, and not a
  claim that every later lint family is low-noise

The Aspen entry intentionally cites the canonical phase-1 archive. Mutable local
Aspen worktrees and later widened scripts are useful for follow-up experiments,
but they are not the source for the starter case study.

## Methodology before public metrics

Public performance, cache, CI-latency, suppression-rate, false-positive, or
rollout-cost metrics should be published only with methodology. A metric is ready
for publication only when it names:

- command or gate that produced the measurement;
- input scope and configuration;
- environment or platform when it affects the result;
- raw artifact, transcript, or reproducible evidence path;
- interpretation boundary for the metric.

A candidate metric without those fields is held back until the raw command and
artifact path exist. For example, a cache-speed claim should name the exact
`cargo octet check` invocation, package scope, cache status fields, platform,
artifact directory or transcript, and the boundary that the value describes only
that run.

## Boundary-preserving comparison

Octet's adoption advantage is compatibility with existing Rust workspaces and
release gates. It can be compared with new-language or verifier-first tools only
by staying within supported evidence: Rust-compatible static checks, policy
checks, optional scoped proof receipts, provenance, and artifact verification.

Octet does not claim mathematical certainty, whole-program correctness,
certification, a verified runtime, or language-level contract syntax unless a
separate accepted requirement and evidence artifact supports that exact claim.
