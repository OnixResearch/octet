# Regression replay only. Callers supply existing tools and audit the dry run.
{ repository, library, driver, runtime, mode ? "ui" }:
assert builtins.pathExists (library + "/lib/liboctet.so");
assert builtins.pathExists (driver + "/bin/dylint-driver");
let
  flake = builtins.getFlake ("path:" + toString repository);
  command =
    if mode == "ui" then
      "cargo test --release --locked --test ui no_todo_expansion_shapes -- --ignored --nocapture"
    else if mode == "quality" then ''
      cargo test --release --locked --lib safety::no_todo::tests -- --nocapture
      cargo clippy --release --locked --lib -- -D warnings
    ''
    else if mode == "all-ui" then
      "cargo test --release --locked --test ui -- --ignored --nocapture"
    else throw "unknown no_todo regression mode";
in flake.checks.x86_64-linux.ui.overrideAttrs (_: {
  pname = "octet-no-todo-${mode}";
  checkPhase = ''
    runHook preCheck
    ${command}
    runHook postCheck
  '';
  preCheck = ''
    unset BLESS
    export PATH="${runtime}/bin:$PATH"
    export OCTET_UI_DRIVER="${driver}/bin/dylint-driver"
    export OCTET_UI_LIBRARY="$TMPDIR/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"
    ln -s '${library}/lib/liboctet.so' "$OCTET_UI_LIBRARY"
  '';
  installPhase = ''
    runHook preInstall
    mkdir -p "$out/snapshots"
    cp ui_tests/no_todo*.stderr "$out/snapshots/"
    runHook postInstall
  '';
})
