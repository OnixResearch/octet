# Verification

- `python3 tools/verify-reviewed-object-source.py --repo . --valence-module <pinned 7546a78 module>`:
  `reviewed-v2-source: ADMITTED independently bound source and references`.
- `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism`:
  passed; two receipt producers, byte stability, fixture parity, compared
  fields, and boundary claims all pass.
- `nix build .#checks.x86_64-linux.object-corpus-caveat-taxonomy-gate`:
  passed; the refreshed taxonomy matches the expanded reference.
- Refreshed counts: internal 1,037 objects; expanded 1,579 objects; taxonomy
  1,579 objects across 36 source buckets.
- The approval scope and independent-audit note state that the original audit
  was not repeated.

The full `nix flake check -L` is rerun after this refresh.
