## Why

Hardened Wasmtime Runtime must compute its pure component-profile identity in the functional core. `blake3::Hasher::new()` constructs an in-memory deterministic value, not a service or effect capability. Octet currently has no consumer-owned input for that reviewed distinction, so its conservative external-ADT rule reports the constructor. Copying Octet's own architecture policy into another project or exempting a whole dependency crate would be unsound.

## What Changes

- Extend the existing Dylint configuration with `core_value_provider_types`, default empty, containing exact compiler definition paths of project-reviewed pure value ADTs.
- Classify only an exact resolved result ADT match as a reviewed value; aliases retain the definition identity. Crate names, path prefixes, suffixes and unclassified siblings do not match.
- Preserve the existing core-scope, direct-call, local/standard-value and test-context rules. All other external constructors remain reportable.
- Add paired actual UI coverage for a selected value, an alias, an unselected sibling in the same dependency and the unchanged default policy.

## Impact

Octet owns the reusable classification mechanism; each consuming repository owns review of its declared values and exact dependency cohort. HWR is the immediate consumer. Source changes stay in config, lint registration and the existing constructor lint; documentation explains the trust boundary. This is not a purity proof, effect-closure admission, dependency-identity verifier, lint severity exception, general crate exemption or publication authorization. Existing Octet publication holds remain intact.
