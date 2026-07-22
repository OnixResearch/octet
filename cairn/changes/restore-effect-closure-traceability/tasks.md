# Tasks

- [x] [serial] V1 Confirm the post-provenance baseline has no missing IDs, exactly one dangling direct-check failure ID, and a passing direct-check refusal integration test. r[octet.effect_closure.cli.check_failure]
- [x] [depends:V1] I1 Register the complete accepted effect-closure spec and exact maintained evidence roots in both typed Nickel profiles; regenerate policy JSON.
- [x] [depends:I1] I2 Link the remaining boundary, dogfood, and verification parent IDs to semantically matching maintained docs/tests.
- [x] [depends:I2] I3 Add a hermetic positive/negative Nix traceability gate using the pinned Cairn checker.
- [x] [depends:I3] V2 Run focused effect-closure tests, Nix gate, policy freshness, strict validation, and require empty missing/dangling arrays.
- [x] [depends:V2] V3 Run formatting, repository-prescribed Clippy, Nextest, and the broad compatible-system flake gate.
- [ ] [depends:V3] V4 Record bounded validation evidence and archive deterministically with `CAIRN_ARCHIVE_DATE=2026-07-21`.
