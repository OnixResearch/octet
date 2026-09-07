# Audit the dry run first. Reuse the retained compiler and dependency artifacts.
{ repository }:
(import ../const-allocation/check.nix { inherit repository; }).overrideAttrs (_: {
  checkPhase = ''
    runHook preCheck
    cargo test --release --offline --locked --lib safety::state_machine_exhaustiveness::tests -- --nocapture
    cargo test --release --offline --locked --lib purity::effect_lints::tests -- --nocapture
    runHook postCheck
  '';
})
