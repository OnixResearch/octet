# Focused lint-library build/tests only. Audit the dry run before realization.
{ repository }:
let
  flake = builtins.getFlake ("path:" + toString repository);
in flake.packages.x86_64-linux.octet.overrideAttrs (_: {
  doCheck = true;
  LD_LIBRARY_PATH = "/nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21/lib";
  checkPhase = ''
    runHook preCheck
    cargo test --release --offline --locked --lib purity::effect_lints::tests -- --nocapture
    runHook postCheck
  '';
})
