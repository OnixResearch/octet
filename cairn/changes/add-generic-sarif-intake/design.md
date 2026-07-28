## Context

Octet already emits SARIF for its own findings and preserves external Clippy JSON. The external-lint seam intentionally has no hosted scanner dependency.

External scanners can produce SARIF, but producer metadata, location forms, severity vocabularies, and partial records vary.

## Decisions

### Decision: Import a bounded SARIF subset

**Choice:** Support SARIF `2.1.0` inputs that contain local runs, declared tool drivers, rules, results, safe artifact locations, regions, and supported severity values.

**Rationale:** A bounded subset provides interoperability without claiming full standard coverage.

### Decision: Preserve raw authority and normalize a projection

**Choice:** Store the exact raw SARIF BLAKE3 identity and derive a deterministic normalized finding projection.

The projection never replaces the source report. It records the selected producer profile and normalization version.

**Rationale:** Reviewers need both byte identity and stable Octet fields.

### Decision: Use producer profiles instead of engine-specific code

**Choice:** Nickel profiles declare accepted driver identities, rule-ID requirements, URI policy, severity mapping, limits, evidence tier, and non-claims.

Initial fixtures cover CodeQL-like and Semgrep-like reports. The runtime does not invoke either engine.

**Rationale:** One generic seam limits product sprawl and keeps hosted services optional.

### Decision: Keep source and gate severity separate

**Choice:** Each normalized finding preserves source level and source rule identity. Octet records its gate outcome in a different field.

**Rationale:** Octet must not rewrite external analyzer meaning.

### Decision: Reject unsafe or ambiguous locations

**Choice:** Accept only safe workspace-relative file locations under an explicit import root.

Remote URIs, parent traversal, non-file schemes, invalid artifact indexes, missing regions, and ambiguous locations fail according to profile policy.

**Rationale:** Imported diagnostics must not grant path or network authority.

## Functional Core and Imperative Shell

The pure core parses supplied bytes, validates profiles, normalizes findings, computes stable IDs, applies limits, and orders diagnostics.

The shell reads one explicit report through a capability root, loads checked profile data, and writes the verified artifact bundle.

## Risks / Trade-offs

- SARIF producers use extensions. Unsupported properties remain raw-only or fail according to explicit profile policy.
- Large reports can exhaust resources. Policy binds byte, run, rule, result, location, message, and artifact limits.
- Normalization can hide source nuance. The receipt retains the raw report identity and producer metadata.

## Validation

Positive fixtures include clean, warning, and denied reports from distinct declared producers.

Negative fixtures cover malformed JSON, wrong version, missing driver or rule identity, remote URIs, traversal, bad indexes, duplicate IDs, unsupported severities, stale hashes, and limits.
