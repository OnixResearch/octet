# Octet-authored ordinary derivation. No verifier policy lives in Mantle.
{
  pkgs,
  nickel,
  provider,
  cargoOctet,
  policyArtifacts,
  rustToolchain,
  sourceRoot,
  sourceRevision,
}:
let
  consumerProfile = sourceRoot + "/standards/proof-rails/aeneas-lean/generated/consumer-profile.json";
  providerRoot = "${provider}/share/octet/aeneas-lean";
  jobPolicy = sourceRoot + "/standards/proof-rails/aeneas-lean/mantle-job.ncl";
in
pkgs.runCommand "octet-mantle-aeneas-consumer-job-v1"
  {
    nativeBuildInputs = [
      nickel
      pkgs.coreutils
      pkgs.jq
      pkgs.bash
    ];
    # Avoid impure/local preference: this derivation must be sandboxable by Mantle.
    preferLocalBuild = false;
    allowSubstitutes = true;
  }
  ''
    set -eu
    export HOME="$TMPDIR/home"
    mkdir -p "$HOME" "$out/source" "$out/artifacts"
    cp -R ${sourceRoot}/standards/proof-rails/aeneas-lean/fixtures/external-consumer/. "$out/source/"
    chmod -R u+w "$out/source"
    # Expected artifacts are not producer inputs. Only actual execution may create them.
    rm -rf "$out/source/expected"
    cp -R "$out/source" "$TMPDIR/source"
    cp ${consumerProfile} "$out/consumer-profile.json"
    cp ${providerRoot}/identity.json "$out/provider-identity.json"
    cp ${sourceRoot}/flake.lock "$out/flake.lock"
    nickel export --format json ${jobPolicy} > "$out/job.json"
    printf '%s\n' '${sourceRevision}' > "$out/source-revision"
    ${policyArtifacts}/bin/octet-policy-artifacts "$TMPDIR/policy"
    export PATH="${
      pkgs.lib.makeBinPath [
        rustToolchain
        pkgs.coreutils
        pkgs.bash
        pkgs.jq
      ]
    }"
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ rustToolchain ]}"
    export CHARON_TOOLCHAIN_IS_IN_PATH=1
    export CARGO_NET_OFFLINE=true
    export MATHLIB_NO_CACHE_ON_UPDATE=1
    export OCTET_AENEAS_CHARON="${providerRoot}/charon"
    export OCTET_AENEAS_AENEAS="${providerRoot}/aeneas"
    export OCTET_AENEAS_LEAN="${providerRoot}/lake"
    export OCTET_AENEAS_LEAN_LIBRARY="${providerRoot}/aeneas-lean-library"
    export OCTET_AENEAS_NETWORK_SANDBOX="${providerRoot}/bwrap"
    cd "$TMPDIR/source"
    ${cargoOctet}/bin/cargo-octet prove \
      --policy-artifacts-dir "$TMPDIR/policy" \
      --source src/lib.rs \
      --rail octet.aeneas_consumer_contract.external-fixture#rail-0 \
      --artifact-dir "$out/artifacts" \
      --output-format json > "$out/producer-prove.json"
    ${cargoOctet}/bin/cargo-octet artifact verify \
      --artifact-dir "$out/artifacts" --format json \
      --emit-valence-provenance "$out/valence-provenance.json" \
      > "$out/producer-artifact-verification.json"
    jq --exit-status '.schema_version == 1 and .status == "valid" and (.diagnostics | length == 0)' \
      "$out/producer-artifact-verification.json" >/dev/null
    receipt="$out/artifacts/proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.receipt"
    jq --exit-status '.schema_version == "octet-formal-proof-envelope/v1" and .status == "passed" and .consumer.outcome == "supported" and .consumer.verification_role == "property"' "$receipt" >/dev/null
    # Copy original bytes, never reconstruct a receipt from process exit status.
    cp "$receipt" "$out/external-receipt.json"
    cp ${consumerProfile} "$out/external-profile.json"
    # Preserve every original report/envelope locator for full consumer replay.
    cp -R "$out/artifacts/." "$out/"
    mkdir -p "$out/src"
    cp "$out/source/src/lib.rs" "$out/src/lib.rs"
  ''
