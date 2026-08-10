## Why

Octet's packaged self-check exits without a useful diagnostic because its reviewed warning baseline no longer matches the current source. The same revision also fails its lint-catalog freshness test because README text no longer matches the checker contract. These failures hide the exact repair scope and block downstream Octet updates.

## What Changes

- Make the packaged self-check print bounded new and stale finding evidence before it fails.
- Remove unapproved warning additions and admit only reviewed semantic rekeys or removals into the warning baseline.
- Restore README lint-count, namespace, checked-path, and evidence-guide text required by live documentation contracts.
- Restore exact claim-boundary terms in the first checked Rust patch walkthrough.
- Preserve positive and negative self-check behavior without weakening lint limits or baseline admission.

## Impact

- **Files**: self-check Nix gate, Octet source flagged by the warning ratchet, warning baseline, primary documentation, focused lifecycle evidence
- **Testing**: lint-catalog freshness, metadata tests, positive and negative self-check fixtures, strict lifecycle validation, and full Nix validation
