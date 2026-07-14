# OnixOS May 17 admission evidence

## Candidate

- OnixOS commit: `292e55c6f954b7b8a3ba71643d4ddfd5c3b75a44`
- Octet provider revision: `374bd16b26cee2af34211a29bfa531c016811f51`
- Candidate receipt BLAKE3: `445e07a3dc5e5be6ebc6eadd75c34a5fe871de088a30c0578b39c2e8672d7d57`
- Verifier: `verus@0.2026.05.17.e479cce`
- Result: `4 verified, 0 errors`, exit `0`
- Timeout observed: `false`
- Promotion eligible: `true`
- Central admission in candidate receipt: `false`
- Pre-promotion row: `local-pin` at `0.2026.04.03.21dfcd2`
- Chain status: `partial-verus-only`
- Trellis imports: none

## Final native evidence

- OnixOS commit: `c275789878fe033cb41245b1bef1e7563bd4dcd8`
- Published Octet provider revision: `16065650552b8901ac81cec677f240fb93f99fe9`
- Final receipt BLAKE3: `1929f66bdf67688e7be82ab98c057e67002db739b0b4fa72a5e64e2e0b07c878`
- Final profile BLAKE3: `0ece3df5312be606be487dde9ce8c2137f75ca73b94c98fa3e33921bcbfeff62`
- Final consumer row: `native` at `0.2026.05.17.e479cce`
- Result: `4 verified, 0 errors`
- Chain status: `partial-verus-only`
- Trellis imports: none

The final and frozen-candidate Nix checks pass, and the OnixOS shim integration evaluates to `true`. The broader OnixOS flake check retains its baseline `vm-service-management-systemd` oneshot/readiness policy failure; untouched OnixOS commit `ce402f7d3c365d4dc2a5bcc2d0acfc392837023f` fails at the same derivation.

Both receipts prove only the four named obligations over the exact OnixOS model. They do not complete the broader formal chain or transfer proof authority to Octet.
