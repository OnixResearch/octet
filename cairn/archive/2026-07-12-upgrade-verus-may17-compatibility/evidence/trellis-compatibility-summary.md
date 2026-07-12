# Trellis compatibility observation

The final closed-portfolio run consumed published Octet revision `374bd16b26cee2af34211a29bfa531c016811f51` through Trellis's locked GitHub input:

- reviewed Trellis baseline: `10712 verified, 0 errors`, exit `0`;
- May 17 complete Trellis: `10712 verified, 0 errors`, exit `0`;
- adapted IronKV: `312 verified, 0 errors`, linked exit `0`;
- adapted verified-memory-allocator: `732 verified, 0 errors`, linked exit `0` against candidate-Rust libc 0.2.147;
- unmodified verified-node-replication: `254 verified, 0 errors`, exit `0`;
- timeouts: none;
- added assumptions, external bodies, trusted markers, or triggers: none.

The canonical payload and receipt field have BLAKE3 `f69521c26cea77d964755c64f27f9fdd18b44725945ba21c6a4c7213285f14e4`. The receipt binds the matrix provider revision to the observed published Octet input revision, records compatibility status `native`, recommends promotion, and passed a recursive saved-evidence freshness rerun after the final source-format normalization.

Trellis preserves its own receipt authority. Adapted-source passes do not certify the unmodified upstream revisions or authorize runtime adoption. Trellis now ships the locked Octet May 17 package as its default verifier; the March package remains only as the historical comparison baseline.

## Validation

The May 17 production package, profile export, package closure manifest, positive multi-rail proof, both negative proof fixtures, policy registry, and proof-rail manifest Nix checks passed. Octet's flake-pinned Cairn package passed proposal, design, tasks, and full repository validation. Trellis's final full `x86_64-linux` flake check passed all `4871` nextest cases.
