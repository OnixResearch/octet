$ grep -nE "nix run .*(cargo-tigerstyle|cargo-slotcar)|cargo (tigerstyle|slotcar) check|Cargo binary" README.md
28:cargo tigerstyle check
34:nix run path:/path/to/slotcar#cargo-tigerstyle -- check
40:cargo tigerstyle check --output-format sarif --artifact-dir artifacts/slotcar
51:before running `cargo slotcar check`:
74:- `cargo tigerstyle check --cache --artifact-dir artifacts/slotcar` records
102:| lint runner command | `cargo tigerstyle check` (`cargo slotcar check` compatibility alias) |

$ nix run .#cargo-slotcar -- --help | grep -nE "cargo slotcar check|nix run .*#cargo-tigerstyle"
4:  cargo slotcar check [SCOPE] [-- CARGO_CHECK_ARGS]
11:Nix: nix run path:/path/to/slotcar#cargo-tigerstyle -- check
