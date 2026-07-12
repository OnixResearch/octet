# Trellis compatibility observation

The final closed-portfolio run consumed committed Octet revision `374bd16b26cee2af34211a29bfa531c016811f51` through an exact local Git input:

- reviewed Trellis baseline: `10712 verified, 0 errors`, exit `0`;
- May 17 complete Trellis: `10712 verified, 0 errors`, exit `0`;
- adapted IronKV: `312 verified, 0 errors`, linked exit `0`;
- adapted verified-memory-allocator: `732 verified, 0 errors`, linked exit `0` against candidate-Rust libc 0.2.147;
- unmodified verified-node-replication: `254 verified, 0 errors`, exit `0`;
- timeouts: none;
- added assumptions, external bodies, trusted markers, or triggers: none.

The canonical payload and receipt field have BLAKE3 `2a37daa5540a10917e8c320fd2ccc4d28c48aaae7326f673430628265c7bc40d`. The receipt binds the matrix provider revision to the observed Octet input revision, records compatibility status `native`, recommends promotion, and passed a recursive saved-evidence freshness rerun.

Trellis preserves its own receipt authority. Adapted-source passes do not certify the unmodified upstream revisions or authorize runtime adoption. The exact local Git input is evidence of the committed provider tree, not a durable consumer dependency; Trellis retains its shipped pin until this Octet revision is explicitly authorized for publication and can be locked from the declared remote.

## Validation

The May 17 production package, profile export, package closure manifest, positive multi-rail proof, both negative proof fixtures, policy registry, and proof-rail manifest Nix checks passed. Octet's flake-pinned Cairn package passed proposal, design, tasks, and full repository validation.
