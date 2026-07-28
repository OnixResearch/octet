# Design: Enforce nominal domain types

## Context

Rust newtypes prevent values with equal runtime representation from crossing the wrong API boundary. Octet already recommends this pattern, but its active lints detect only narrow symptoms.

The stack contains legitimate raw representations at serialization, protocol, FFI, and generated boundaries. Hard enforcement therefore needs reviewed domain declarations rather than a global suffix rule.

## Decisions

### Decision: Nickel owns the domain policy

**Choice:** Add a typed Nickel catalog for nominal domains. Each row names the domain, raw representation, core scopes, compatibility scopes, wrapper kind, and constructor policy.

The source is `standards/nominal-domains/catalog.ncl`. The generated runtime projection is `standards/nominal-domains/generated/catalog.json`.

Rust embeds the JSON projection through `octet-lint-metadata`. The policy registry and Nix freshness rail cover both files.

Saved pilot receipts use schema `octet-nominal-domain-inventory/v1`. Octet writes them under `cairn/changes/enforce-nominal-domain-types/evidence/pilots/`.

**Rationale:** Human review must decide which values have domain meaning. Rust source-name heuristics are not sufficient authority for a hard gate.

### Decision: Exact declarations drive errors

**Choice:** Emit hard findings only for exact policy rows in declared core scopes. Emit inventory warnings for unregistered names that match reviewed semantic suffixes.

**Rationale:** Exact declarations provide deterministic precision. Heuristics remain useful for discovery without blocking valid structural records.

### Decision: Add three focused lints

**Choice:** Add these lints:

- `primitive_domain_alias` rejects a declared domain alias whose resolved target is a configured raw representation.
- `raw_domain_value` rejects configured raw representations in declared core fields, parameters, results, enum payloads, and map keys.
- `newtype_invariant_bypass` rejects public inner fields and unchecked primitive conversions for domains marked as refined.

Aliases to generic nominal types remain valid. For example, `type UnitId = DomainId<UnitRole>` is accepted because the resolved type is not the raw representation.

**Rationale:** The checks address distinct failure modes and produce specific remediation.

### Decision: Keep classification logic pure

**Choice:** HIR passes collect resolved descriptors and source scope. Pure helpers classify policy matches, exceptions, wrapper contracts, and findings.

Filesystem reads, rustc registration, source spans, and diagnostic emission remain in the lint shell.

**Rationale:** Policy decisions need ordinary positive and negative unit tests without a compiler process.

### Decision: Improve symptom lints

**Choice:** Extend `ambiguous_params` to associated functions, methods, constructors, owned strings, configured path types, and repeated non-consecutive raw types.

Extend `numeric_units` to fields and parameters. Suppress unit-name findings when the resolved type is a configured unit newtype.

**Rationale:** These lints remain useful before a domain enters the exact policy registry.

### Decision: Preserve compatibility boundaries

**Choice:** Exclude configured wire, FFI, generated, vendored, and protocol DTO scopes from hard findings. Require an explicit adapter from each excluded scope into a declared typed core scope.

**Rationale:** External formats often require strings or integers. The adapter is the correct place to parse and reject malformed values.

### Decision: Promote in stages

**Choice:** Ship the new checks as inventory or warning first. Save signal evidence from Octet, Valence, and Mantle fixtures.

After fixture review, add exact-policy checks to `bounded-systems` and `deterministic-core`. Keep heuristic discovery outside strict-profile denial.

**Rationale:** Octet policy requires proven positive, negative, and compatibility precision before a broad hard gate.

## Test design

Positive fixtures cover private wrappers, fallible refined constructors, generic marker domains, explicit accessors, unit newtypes, and wire-to-core adapters.

Negative fixtures cover primitive aliases, raw core fields, raw API parameters and results, map keys, public refined fields, unchecked `From<String>`, repeated raw parameters, and unitless numeric fields.

Compatibility fixtures cover Serde-transparent scalars, explicit wire DTOs, FFI records, generated modules, generic nominal aliases, and unchanged public JSON.

Compile-fail fixtures prove that two marker-domain aliases cannot be exchanged.

## Risks and trade-offs

- Policy rows can become stale. Freshness checks and unknown-scope diagnostics fail closed.
- An overly broad scope can create noise. Scope exceptions remain explicit and reviewable.
- A wrapper can still expose unsafe construction. The refined-value contract checks common bypasses but does not prove every invariant.
- A type name can imply authority it does not carry. Documentation and diagnostics preserve that non-claim.

## Claim boundary

Passing these lints proves only configured source-shape conformance for the checked crate and scopes. It does not prove runtime correctness, authority, secrecy, wire safety, or release eligibility.
