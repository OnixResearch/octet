## Why

The workspace dependency catalog admits Cairn revision `695124d459574ba7aeba6097310d237f393c243c` for Octet's validation plane. Octet still pins `70db8fe114910feb5d162fea5db8ad4ffdb6f176`, and the admitted tool requires the native `.cairn/` lifecycle layout.

## What Changes

- Regenerate only the root `cairn` Nix input and its transitive lock graph.
- Use Cairn's hash-bound migration to move lifecycle authority to `.cairn/`.
- Retire the duplicated legacy lifecycle tree after byte-parity review.
- Update maintained Octet policy, fixture, source, and operator paths.
- Preserve active and archived lifecycle content without changing its meaning.

## Impact

- **Files**: generated `flake.lock`, `.cairn/**`, removed legacy `cairn/**`, maintained lifecycle path references, and generated policy artifacts
- **Testing**: migration receipts, lifecycle and traceability checks, policy freshness, focused Octet checks, and full Nix flake validation against the recorded baseline
