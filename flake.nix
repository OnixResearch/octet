{
  description = "slotcar — Custom lints enforcing Tiger Style coding principles";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    dylint-src = {
      url = "github:trailofbits/dylint/0e0a71eefe6f01563d11acc6e1d7af1d505934a9";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, dylint-src, ... }:
    let
      # Consumer-check helper — usable from any downstream flake.
      # Runs slotcar lints against consumer code via nix flake check.
      mkConsumerCheck = {
        system,
        src,
        packages ? [],
        cargoExtraArgs ? null,
        cargoLock ? null,
        nativeBuildInputs ? [],
        buildInputs ? [],
      }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          tsRustToolchain = pkgs.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml";
          tsHostTriple = pkgs.stdenv.hostPlatform.rust.rustcTarget;
          tsToolchainChannel = (builtins.fromTOML
            (builtins.readFile "${self}/rust-toolchain.toml")).toolchain.channel;
          tsRustupToolchain = if pkgs.lib.hasSuffix tsHostTriple tsToolchainChannel
            then tsToolchainChannel
            else "${tsToolchainChannel}-${tsHostTriple}";
          tsCraneLib = (crane.mkLib pkgs).overrideToolchain tsRustToolchain;
          tsRustToolchainLibPath = pkgs.lib.makeLibraryPath [ tsRustToolchain ];

          tsRustupShim = pkgs.writeShellScriptBin "rustup" ''
            toolchain="''${RUSTUP_TOOLCHAIN:-${tsRustupToolchain}}"
            case "$1" in
              which) which "$2" ;;
              show)
                if [ "$2" = "active-toolchain" ]; then
                  echo "$toolchain (default)"
                else
                  echo "$toolchain"
                fi
                ;;
              *) echo "rustup shim: unsupported: $*" >&2; exit 1 ;;
            esac
          '';
          tsCargoShim = pkgs.writeShellScriptBin "cargo" ''
            export RUSTUP_TOOLCHAIN="''${RUSTUP_TOOLCHAIN:-${tsRustupToolchain}}"
            exec ${tsRustToolchain}/bin/cargo "$@"
          '';
          tsDylintShimPath = pkgs.lib.makeBinPath [ tsCargoShim tsRustupShim ];

          slotcarPkg = self.packages.${system}.default;
          driverPkg = self.packages.${system}.dylint-driver;
          sharedLibSuffix =
            if pkgs.stdenv.hostPlatform.isDarwin then ".dylib" else ".so";
          dllPrefix =
            if pkgs.stdenv.hostPlatform.isDarwin then "lib" else "lib";
          # Dylint parses the library filename as lib<name>@<toolchain>.<ext>
          slotcarDylintLibDir = pkgs.runCommand "tigerstyle-dylint-lib" {} ''
            mkdir -p $out/lib
            ln -s ${slotcarPkg}/lib/${dllPrefix}tigerstyle${sharedLibSuffix} \
              $out/lib/${dllPrefix}tigerstyle@${tsRustupToolchain}${sharedLibSuffix}
          '';
          slotcarDylintLibPath =
            "${slotcarDylintLibDir}/lib/${dllPrefix}tigerstyle@${tsRustupToolchain}${sharedLibSuffix}";

          checkNativeBuildInputs =
            nativeBuildInputs ++ [ pkgs.pkg-config tsCargoShim tsRustupShim ];

          consumerVendorDir = if cargoLock != null
            then tsCraneLib.vendorCargoDeps { inherit cargoLock; }
            else null;

          packageArgs = builtins.concatStringsSep " "
            (map (p: "-p ${p}") packages);
          cargoCheckTail = if cargoExtraArgs != null && cargoExtraArgs != ""
            then " -- ${cargoExtraArgs}"
            else "";
        in
        pkgs.runCommand "slotcar-consumer-check" {
          nativeBuildInputs = checkNativeBuildInputs;
          inherit buildInputs;
          LD_LIBRARY_PATH = tsRustToolchainLibPath;
        } ''
          export PATH="${tsDylintShimPath}:$PATH"
          export HOME="$PWD/home"
          mkdir -p "$HOME"
          cp -r ${src} source
          chmod -R u+w source
          cd source
          export TIGERSTYLE_DRIVER_PATH="${driverPkg}/bin/dylint-driver"
          export TIGERSTYLE_LINT_LIB="${slotcarDylintLibPath}"
          export TIGERSTYLE_TOOLCHAIN="${tsRustupToolchain}"
          ${pkgs.lib.optionalString (consumerVendorDir != null) ''
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${consumerVendorDir}/config.toml" "$CARGO_HOME/config.toml"
          ''}
          ${self.packages.${system}.cargo-slotcar}/bin/cargo-slotcar check ${packageArgs}${cargoCheckTail}
          mkdir -p "$out"
        '';

      # Generated-policy-artifact consumer helper — usable from downstream
      # flakes when they want the Nickel-authored policy bundle exported by this
      # flake, then consumed by slotcar-standards in one reproducible check.
      mkPolicyArtifactConsumerCheck = {
        system,
        src,
        family,
        sourceSubdir ? ".",
        sourceSubdirs ? null,
        expectedDiagnostics ? 0,
        requiredPolicyIds ? [],
        extraArgs ? [],
        nativeBuildInputs ? [],
        buildInputs ? [],
      }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          selectedSourceSubdirs = if sourceSubdirs == null then [ sourceSubdir ] else sourceSubdirs;
          sourceArgFor = subdir:
            if subdir == "." then "--source source" else "--source source/${subdir}";
          sourceArgs = builtins.concatStringsSep " " (map sourceArgFor selectedSourceSubdirs);
          extraCheckArgs = builtins.concatStringsSep " " extraArgs;
          requiredPolicyAssertions = builtins.concatStringsSep "\n" (map (policyId: ''
            jq --exit-status \
              'any(.policy_evaluations[]; .policy_id == "${policyId}")' \
              "$report_json" >/dev/null
          '') requiredPolicyIds);
        in
        pkgs.runCommand "slotcar-policy-artifact-consumer-check" {
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.jq self.packages.${system}.slotcar-standards ];
          inherit buildInputs;
        } ''
          artifact_dir=$(mktemp -d)
          report_json=$(mktemp)

          ${self.packages.${system}.tigerstyle-policy-artifacts}/bin/tigerstyle-policy-artifacts "$artifact_dir"
          test -s "$artifact_dir/tigerstyle-policies.json"
          test -s "$artifact_dir/tigerstyle-check-config.toml"
          test -s "$artifact_dir/tigerstyle-seed-manifests.json"

          cp -r ${src} source
          chmod -R u+w source
          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            ${sourceArgs} \
            --family ${family} \
            --output json \
            ${extraCheckArgs} > "$report_json"

          jq --exit-status \
            '.diagnostics | length == ${builtins.toString expectedDiagnostics}' \
            "$report_json" >/dev/null
          ${requiredPolicyAssertions}
          mkdir -p "$out"
        '';
    in
    {
      lib = { inherit mkConsumerCheck mkPolicyArtifactConsumerCheck; };
    } //
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        hostTriple = pkgs.stdenv.hostPlatform.rust.rustcTarget;
        toolchainChannel = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel;
        rustupToolchain = if pkgs.lib.hasSuffix hostTriple toolchainChannel
          then toolchainChannel
          else "${toolchainChannel}-${hostTriple}";

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              inUiTests = pkgs.lib.hasSuffix "/ui_tests" path || pkgs.lib.hasInfix "/ui_tests/" path;
              inTestFixtures = pkgs.lib.hasSuffix "/tests/fixtures" path || pkgs.lib.hasInfix "/tests/fixtures/" path;
              inDocs = pkgs.lib.hasSuffix "/docs" path || pkgs.lib.hasInfix "/docs/" path;
              inStandards = pkgs.lib.hasSuffix "/standards" path || pkgs.lib.hasInfix "/standards/" path;
              inTemplates = pkgs.lib.hasSuffix "/templates" path || pkgs.lib.hasInfix "/templates/" path;
            in (craneLib.filterCargoSources path type) || inUiTests || inTestFixtures || inDocs || inStandards || inTemplates;
        };
        fmtSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type)
            || builtins.baseNameOf path == "rustfmt.toml";
        };
        uiDriverSrc = ./tests/support;

        nativeBuildInputs = with pkgs; [
          clang
          mold
          pkg-config
        ];

        buildInputs = with pkgs; [ openssl ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        # Dylint driver and UI harness still shell out through cargo/rustup.
        # These shims re-inject the toolchain name Dylint expects.
        rustup-shim = pkgs.writeShellScriptBin "rustup" ''
          toolchain="''${RUSTUP_TOOLCHAIN:-${rustupToolchain}}"
          case "$1" in
            which) which "$2" ;;
            show)
              if [ "$2" = "active-toolchain" ]; then
                echo "$toolchain (default)"
              else
                echo "$toolchain"
              fi
              ;;
            *) echo "rustup shim: unsupported: $*" >&2; exit 1 ;;
          esac
        '';

        cargo-shim = pkgs.writeShellScriptBin "cargo" ''
          export RUSTUP_TOOLCHAIN="''${RUSTUP_TOOLCHAIN:-${rustupToolchain}}"
          exec ${rustToolchain}/bin/cargo "$@"
        '';

        dylintShimPath = pkgs.lib.makeBinPath [ cargo-shim rustup-shim rustToolchain ];
        rustToolchainLibPath = pkgs.lib.makeLibraryPath [ rustToolchain ];
        checkNativeBuildInputs = nativeBuildInputs ++ [ cargo-shim rustup-shim pkgs.git pkgs.python3 ];
        sharedLibraryPrefix = if pkgs.stdenv.hostPlatform.isWindows then "" else "lib";
        sharedLibrarySuffix =
          if pkgs.stdenv.hostPlatform.isDarwin then ".dylib"
          else if pkgs.stdenv.hostPlatform.isWindows then ".dll"
          else ".so";
        slotcarLibraryName = "${sharedLibraryPrefix}tigerstyle${sharedLibrarySuffix}";
        slotcarUiLibraryName = "${sharedLibraryPrefix}tigerstyle@${rustupToolchain}${sharedLibrarySuffix}";

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
          RUSTUP_TOOLCHAIN = rustupToolchain;
        };

        slotcar = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          doCheck = false;
        };

        cargoSlotcarUnwrapped = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "cargo-slotcar";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-slotcar";
          doCheck = false;
        };

        fingerprintDocsDownstreamSmoke = pkgs.writeShellApplication {
          name = "tigerstyle-fingerprint-docs-downstream-smoke";
          runtimeInputs = [ pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.jq ];
          text = ''
            repo_root="''${1:-${src}}"
            scratch="$(mktemp -d)"
            trap 'rm -rf "$scratch"' EXIT

            mkdir -p "$scratch/consumer/src"
            cat > "$scratch/consumer/Cargo.toml" <<'EOF'
            [package]
            name = "tigerstyle-fingerprint-downstream-smoke"
            version = "0.1.0"
            edition = "2024"
            EOF
            cat > "$scratch/consumer/src/lib.rs" <<'EOF'
            pub mod human_summary;
            pub mod json_output;
            pub mod lifecycle;
            pub mod source_scanner;
            pub mod warning_summary;
            EOF

            cp "$repo_root/docs/examples/function-fingerprint-human-summary/reviewed.rs" "$scratch/consumer/src/human_summary.rs"
            cp "$repo_root/docs/examples/function-fingerprint-json-output/reviewed.rs" "$scratch/consumer/src/json_output.rs"
            cp "$repo_root/docs/examples/function-fingerprint-lifecycle/reviewed.rs" "$scratch/consumer/src/lifecycle.rs"
            cp "$repo_root/docs/examples/function-fingerprint-source-scanner/reviewed.rs" "$scratch/consumer/src/source_scanner.rs"
            cp "$repo_root/docs/examples/function-fingerprint-warning-summary/reviewed.rs" "$scratch/consumer/src/warning_summary.rs"
            chmod -R u+w "$scratch/consumer"

            set +e
            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --check --output-format=json "$scratch/consumer/src" > "$scratch/check.json" 2> "$scratch/check.err"
            check_status="$?"
            set -e
            test "$check_status" -eq 1
            jq -e '.schema_version == 1 and .mode == "check"' "$scratch/check.json" > /dev/null
            jq -e '.records | length == 7' "$scratch/check.json" > /dev/null
            jq -e '[.records[].name] | index("bounded_retry_delay_ms") and index("capped_batch_size") and index("capped_retries") and index("describe_batch") and index("explicit_generated_boundary") and index("missing_gate_metadata") and index("unreviewed_macro_path")' "$scratch/check.json" > /dev/null
            jq -e '[.records[].name] | index("generated_boundary") | not' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.macro_warning == true)] | length == 1' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.review == null or .evidence == null)] | length == 1' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.status == "match")] | length == 7' "$scratch/check.json" > /dev/null

            sed -i 's/requested.min(5)/requested.min(8)/' "$scratch/consumer/src/json_output.rs"
            set +e
            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/drift.json" 2> "$scratch/drift.err"
            drift_status="$?"
            set -e
            test "$drift_status" -eq 1
            jq -e '.records | length == 1' "$scratch/drift.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch"' "$scratch/drift.json" > /dev/null

            set +e
            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --diff "$scratch/consumer/src/json_output.rs" > "$scratch/diff.txt" 2> "$scratch/diff.err"
            diff_status="$?"
            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --diff --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/diff.json" 2> "$scratch/diff-json.err"
            diff_json_status="$?"
            set -e
            test "$diff_status" -eq 1
            test "$diff_json_status" -eq 1
            grep -F "slotcar fingerprint diff" "$scratch/diff.txt" > /dev/null
            grep -F "capped_retries mismatch" "$scratch/diff.txt" > /dev/null
            grep -F "summary total=1 match=0 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0" "$scratch/diff.txt" > /dev/null
            jq -e '.schema_version == 1 and .mode == "diff"' "$scratch/diff.json" > /dev/null
            jq -e '.records | length == 1' "$scratch/diff.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch" and .records[0].macro_warning == false' "$scratch/diff.json" > /dev/null
            diff_human_name="$(grep -F "capped_retries mismatch" "$scratch/diff.txt" | sed -E 's/.* ([^ ]+) mismatch .*/\1/')"
            diff_json_name="$(jq -r '.records[0].name' "$scratch/diff.json")"
            test "$diff_human_name" = "$diff_json_name"

            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --update --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/update.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "written"' "$scratch/update.json" > /dev/null
            ${cargoSlotcarUnwrapped}/bin/cargo-slotcar fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/repaired.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "match"' "$scratch/repaired.json" > /dev/null
          '';
        };

        slotcarStandards = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "slotcar-standards";
          version = "0.1.0";
          cargoExtraArgs = "-p slotcar-standards";
          doCheck = false;
        };

        slotcarStandardsNixClosureManifest = pkgs.runCommand "slotcar-standards-nix-closure-manifest" {
          nativeBuildInputs = [ pkgs.b3sum pkgs.python3 ];
          exportReferencesGraph = [ "slotcar-standards-closure" slotcarStandards ];
        } ''
          mkdir -p "$out"
          cp slotcar-standards-closure "$out/slotcar-standards-closure.graph"
          GRAPH="$out/slotcar-standards-closure.graph" python3 - <<'PY' > "$out/slotcar-standards-closure.graph.canonical"
          import os
          from pathlib import Path
          text = Path(os.environ["GRAPH"]).read_text()
          lines = text.splitlines()
          index = 0
          records = []
          while index < len(lines):
              while index < len(lines) and not lines[index].strip():
                  index += 1
              if index >= len(lines):
                  break
              path = lines[index].strip()
              index += 1
              while index < len(lines) and not lines[index].strip():
                  index += 1
              count = int(lines[index].strip())
              index += 1
              refs = []
              for _ in range(count):
                  while index < len(lines) and not lines[index].strip():
                      index += 1
                  refs.append(lines[index].strip())
                  index += 1
              records.append((path, sorted(refs)))
          for path, refs in sorted(records):
              print(path)
              print(len(refs))
              for ref in refs:
                  print(ref)
          PY
          graph_digest=$(b3sum "$out/slotcar-standards-closure.graph.canonical" | cut -d' ' -f1)
          printf '%s\n' "$graph_digest" > "$out/slotcar-standards-closure.graph.blake3"
          cat > "$out/proof-nix-closure.txt" <<EOF
          proof-verifier-nix-derivation: ${slotcarStandards.drvPath}
          proof-verifier-nix-output: ${slotcarStandards}
          proof-verifier-nix-closure: exportReferencesGraph:slotcar-standards-closure
          proof-verifier-nix-closure-graph-blake3: $graph_digest
          EOF
        '';

        uiDriverVendorDir = craneLib.vendorCargoDeps {
          src = uiDriverSrc;
          cargoLock = ./tests/support/ui-driver/Cargo.lock;
        };

        uiDriver = pkgs.stdenv.mkDerivation {
          pname = "tigerstyle-ui-driver";
          version = "0.1.0";
          src = uiDriverSrc;
          nativeBuildInputs = checkNativeBuildInputs ++ [ rustToolchain ];
          inherit buildInputs;
          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            export PATH="${dylintShimPath}:$PATH"
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${uiDriverVendorDir}/config.toml" "$CARGO_HOME/config.toml"
            cargo build --release --locked --offline --manifest-path ui-driver/Cargo.toml
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p "$out/bin"
            driver_bin=""
            for candidate in \
              target/release/tigerstyle-ui-driver \
              target/release/tigerstyle_ui_driver \
              ui-driver/target/release/tigerstyle-ui-driver \
              ui-driver/target/release/tigerstyle_ui_driver
            do
              if [ -f "$candidate" ]; then
                driver_bin="$candidate"
                break
              fi
            done
            if [ -z "$driver_bin" ]; then
              echo "missing slotcar UI driver binary" >&2
              find . -maxdepth 4 \( -name 'tigerstyle-ui-driver' -o -name 'tigerstyle_ui_driver' \) -print >&2 || true
              exit 1
            fi
            cp "$driver_bin" "$out/bin/tigerstyle-ui-driver"
            runHook postInstall
          '';
        };

        slotcarUiLibrary = pkgs.runCommand "tigerstyle-ui-library" { } ''
          mkdir -p "$out/lib"
          ln -s "${slotcar}/lib/${slotcarLibraryName}" "$out/lib/${slotcarUiLibraryName}"
        '';

        # Dylint driver binary — used by mkConsumerCheck to run lints.
        # Patches the upstream driver source to add a binary target, then
        # builds offline with vendored deps.
        dylintDriverSrc = pkgs.runCommand "dylint-driver-src" {} ''
          cp -r ${dylint-src} $out
          chmod -R u+w $out
          cat > $out/driver/src/main.rs <<'MAIN'
#![feature(rustc_private)]
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    dylint_driver::dylint_driver(&args).unwrap_or_else(|e| {
        eprintln!("dylint-driver: {e:#}");
        std::process::exit(1);
    });
}
MAIN
        '';

        dylintDriverVendorDir = craneLib.vendorCargoDeps {
          cargoLock = "${dylint-src}/driver/Cargo.lock";
        };

        dylintDriver = pkgs.stdenv.mkDerivation {
          pname = "dylint-driver";
          version = "5.0.0";
          src = dylintDriverSrc;
          nativeBuildInputs = checkNativeBuildInputs ++ [ rustToolchain ];
          buildInputs = buildInputs ++ (with pkgs; [ curl libgit2 ]);
          LD_LIBRARY_PATH = rustToolchainLibPath;
          # Setting RUSTUP_TOOLCHAIN to "nightly-<triple>" (without date)
          # makes the build.rs skip cloning the clippy repo for extra symbols
          # while still parsing as a valid toolchain string.
          RUSTUP_TOOLCHAIN = "nightly-${hostTriple}";
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            export PATH="${dylintShimPath}:$PATH"
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${dylintDriverVendorDir}/config.toml" "$CARGO_HOME/config.toml"
            cd driver
            cargo build --release --offline
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p "$out/bin"
            driver_bin=$(find .. -name 'dylint_driver' -type f -executable | head -1)
            if [ -z "$driver_bin" ]; then
              echo "dylint_driver binary not found" >&2
              exit 1
            fi
            cp "$driver_bin" "$out/bin/dylint-driver"
            runHook postInstall
          '';
        };

        cargoSlotcar = pkgs.symlinkJoin {
          name = "cargo-slotcar-0.1.0";
          paths = [ cargoSlotcarUnwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram "$out/bin/cargo-slotcar" \
              --prefix PATH : "${dylintShimPath}" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${slotcarUiLibrary}/lib/${slotcarUiLibraryName}"
            wrapProgram "$out/bin/cargo-tigerstyle" \
              --prefix PATH : "${dylintShimPath}" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${slotcarUiLibrary}/lib/${slotcarUiLibraryName}"
          '';
        };

        policyExport = pkgs.writeShellApplication {
          name = "tigerstyle-policy-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/policies/registry.ncl
          '';
        };

        policyArtifacts = pkgs.writeShellApplication {
          name = "tigerstyle-policy-artifacts";
          runtimeInputs = with pkgs; [ nickel slotcarStandards ];
          text = ''
            if [ "$#" -ne 1 ]; then
              echo "usage: tigerstyle-policy-artifacts OUT_DIR" >&2
              exit 2
            fi
            cd ${self}
            registry_json=$(mktemp)
            nickel export --format json standards/policies/registry.ncl > "$registry_json"
            exec slotcar-standards export-artifacts --registry "$registry_json" --out-dir "$1"
          '';
        };

        policyValidate = pkgs.writeShellApplication {
          name = "tigerstyle-policy-validate";
          runtimeInputs = with pkgs; [ gawk jq nickel ];
          text = ''
            nickel typecheck standards/policies/registry.ncl
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status '.policies | length > 0' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status '.seed_manifests | length == 3' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "structural" and (.mechanisms | length > 0))' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "verified" and (.proof_rails | length > 0))' >/dev/null

            while IFS=$'\t' read -r policy_id source_path source_heading; do
              if [ ! -f "$source_path" ]; then
                echo "policy $policy_id references missing source path: $source_path" >&2
                exit 1
              fi
              if ! awk -v heading="$source_heading" '
                /^=+ / {
                  current = $0
                  sub(/^=+ /, "", current)
                  if (current == heading) {
                    found = 1
                  }
                }
                END { exit found ? 0 : 1 }
              ' "$source_path"; then
                echo "policy $policy_id references missing source heading: $source_heading" >&2
                exit 1
              fi
            done < <(nickel export --format json standards/policies/registry.ncl \
              | jq --raw-output '.policies[] | [.policy_id, .source_path, .source_heading] | @tsv')

            for file in standards/policies/fixtures/valid/*.ncl; do
              nickel export --format json "$file" >/dev/null
            done

            for file in standards/policies/fixtures/invalid/*.ncl; do
              if nickel export --format json "$file" >/dev/null 2>&1; then
                echo "invalid policy fixture passed: $file" >&2
                exit 1
              fi
            done
          '';
        };

        policyRegistryCheck = pkgs.runCommand "tigerstyle-policy-registry-check"
          {
            nativeBuildInputs = with pkgs; [ gawk jq nickel ];
          } ''
          cd ${./.}
          nickel typecheck standards/policies/registry.ncl
          nickel export --format json standards/policies/registry.ncl \
            | jq --exit-status '.policies | length > 0' >/dev/null
          nickel export --format json standards/policies/registry.ncl \
            | jq --exit-status '.seed_manifests | length == 3' >/dev/null
          nickel export --format json standards/policies/registry.ncl \
            | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "structural" and (.mechanisms | length > 0))' >/dev/null
          nickel export --format json standards/policies/registry.ncl \
            | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "verified" and (.proof_rails | length > 0))' >/dev/null

          while IFS=$'\t' read -r policy_id source_path source_heading; do
            if [ ! -f "$source_path" ]; then
              echo "policy $policy_id references missing source path: $source_path" >&2
              exit 1
            fi
            if ! awk -v heading="$source_heading" '
              /^=+ / {
                current = $0
                sub(/^=+ /, "", current)
                if (current == heading) {
                  found = 1
                }
              }
              END { exit found ? 0 : 1 }
            ' "$source_path"; then
              echo "policy $policy_id references missing source heading: $source_heading" >&2
              exit 1
            fi
          done < <(nickel export --format json standards/policies/registry.ncl \
            | jq --raw-output '.policies[] | [.policy_id, .source_path, .source_heading] | @tsv')

          for file in standards/policies/fixtures/valid/*.ncl; do
            nickel export --format json "$file" >/dev/null
          done

          for file in standards/policies/fixtures/invalid/*.ncl; do
            if nickel export --format json "$file" >/dev/null 2>&1; then
              echo "invalid policy fixture passed: $file" >&2
              exit 1
            fi
          done

          touch "$out"
        '';

        standardsFixtureCheck = pkgs.runCommand "slotcar-standards-fixtures"
          {
            nativeBuildInputs = with pkgs; [ jq nickel slotcarStandards ];
          } ''
          cd ${./.}
          artifact_dir=$(mktemp -d)
          registry_json=$(mktemp)
          compliant_json=$(mktemp)
          noncompliant_json=$(mktemp)
          advisory_json=$(mktemp)
          markdown_report=$(mktemp)

          nickel export --format json standards/policies/registry.ncl > "$registry_json"
          slotcar-standards export-artifacts --registry "$registry_json" --out-dir "$artifact_dir"
          test -s "$artifact_dir/tigerstyle-policies.json"
          test -s "$artifact_dir/tigerstyle-check-config.toml"
          test -s "$artifact_dir/tigerstyle-seed-manifests.json"
          test -s "$artifact_dir/tigerstyle-proof-bundles.json"
          test -s "$artifact_dir/tigerstyle-config-receipt-families.json"
          test -s "$artifact_dir/tigerstyle-artifact-manifest.json"
          jq --exit-status '.schema_version == "tigerstyle-artifact-manifest/v1"' "$artifact_dir/tigerstyle-artifact-manifest.json" >/dev/null
          jq --exit-status '(.artifacts | length) == 5' "$artifact_dir/tigerstyle-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "tigerstyle-policies.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/tigerstyle-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "tigerstyle-config-receipt-families.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/tigerstyle-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "tigerstyle-check-config.toml" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/tigerstyle-artifact-manifest.json" >/dev/null
          jq --exit-status '.schema_version == "tigerstyle-proof-bundle/v1"' "$artifact_dir/tigerstyle-proof-bundles.json" >/dev/null
          jq --exit-status 'any(.proof_bundles[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers")' "$artifact_dir/tigerstyle-proof-bundles.json" >/dev/null
          jq --exit-status 'any(.proof_bundles[]; .proof_rail.receipt_verifier.verifier_nix_closure_graph_paths[] == "proof-nix-closure.graph")' "$artifact_dir/tigerstyle-proof-bundles.json" >/dev/null
          grep --fixed-strings "structural_policy_ids" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_forbidden_apis" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "tokio::sync::mpsc::unbounded_channel" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_capacity_witnesses" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_backpressure_behaviors" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "capability_forbidden_ambient_apis" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "std::process::" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "capability_core_source_scopes" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "capability_shell_constructor_scopes" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "capability_approved_names" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_source_scopes" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_raw_alias_patterns" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_raw_transition_patterns" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_required_state_markers" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_required_transition_methods" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "send_raw(" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_backends" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "local-command-receipt" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_commands" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "bash proof-command.sh" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_pass_signals" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof-result: pass deterministic-effect-handlers" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_bindings" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "blake3-command-and-source" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_schemas" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "tigerstyle-proof-receipt/v1" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_runners" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "slotcar-standards" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_environments" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "local-source-root" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_requires" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "trust-boundary" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifiers" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "slotcar-standards@proof-receipt/v1" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_binaries" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_binary_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "0d16de395c41ce1723da5047b8efc43819f9a81cf1eafac8f63a503f9b5cde27" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closures" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "nix-slotcar-standards-build-closure/v1" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closure_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "4f3f850a5bd3a71171dd24ca76732de451b81bc3759833f242ab9ef21675b8c1" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closure_paths" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof-build-closure.txt" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_derivations" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "slotcar-standards.drv" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_derivation_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "395cf66a51543cf89028075deef9cf172bac902fd777f0ba0bead91aa68afec4" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closures" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "nix-store-recursive-closure/v1" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "4f22aa23fc29fea280cb03e0b69643113681ca81493547e7239e113764990160" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_paths" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof-nix-closure.txt" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "e65bb6d911f9f6a878dc93d876cc03dc6504a4b9078aab71986b10e5ca862349" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_paths" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof-nix-closure.graph" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_toolchains" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "rust-toolchain.toml" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_toolchain_digests" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null
          grep --fixed-strings "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8" "$artifact_dir/tigerstyle-check-config.toml" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/suppressions/compliant \
            --family devtools \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status '.active_suppressions | length == 1' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/compliant \
            --family architecture \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and .target_enforcement_mode == "structural" and .failure_type == "crate-boundary")' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/embedded/compliant \
            --family embedded \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/compliant/typestate-protocol \
            --family idioms \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/compliant \
            --family verification \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers" and .current_enforcement_mode == "verified" and .failure_type == "verified-proof-rail")' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/compliant \
            --family verification \
            --execute-proof-rails \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/advisory \
            --family architecture \
            --output json > "$advisory_json"
          jq --exit-status '.diagnostics | length == 1' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].policy_id == "tigerstyle.architecture.public-api-budget"' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].severity == "warning"' "$advisory_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/suppressions/noncompliant \
            --family devtools \
            --output json > "$noncompliant_json"; then
            echo "noncompliant suppression fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/noncompliant \
            --family architecture \
            --output json > "$noncompliant_json"; then
            echo "noncompliant architecture fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 4' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.external-domain-leakage")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.root-crate-thinness" and (.path | contains("thick-root")))' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and (.message | contains("must depend on tigerstyle-core")))' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.effect-capability-boundaries")' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id != "tigerstyle.idioms.nominal-type-boundaries")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .target_enforcement_mode == "structural")' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; ((.path | contains("thick-root")) | not) or .policy_id != "tigerstyle.architecture.public-api-budget")' "$noncompliant_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/noncompliant/typestate-raw \
            --family idioms \
            --output json > "$noncompliant_json"; then
            echo "noncompliant type-state fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .message | contains("type ConnectionState = bool"))' "$noncompliant_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/noncompliant \
            --family verification \
            --output json > "$noncompliant_json"; then
            echo "noncompliant proof-rail fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .message | contains("missing pass signal"))' "$noncompliant_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/noncompliant \
            --family verification \
            --execute-proof-rails \
            --output json > "$noncompliant_json"; then
            echo "noncompliant proof-command fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .message | contains("missing pass signal"))' "$noncompliant_json" >/dev/null

          if slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/embedded/noncompliant \
            --family embedded \
            --output json > "$noncompliant_json"; then
            echo "noncompliant embedded fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 2' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.embedded.bounded-queues" and (.message | contains("forbidden unbounded API")))' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.embedded.bounded-queues" and (.message | contains("missing capacity or backpressure witness")))' "$noncompliant_json" >/dev/null

          slotcar-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/suppressions/compliant \
            --family devtools \
            --output markdown > "$markdown_report"
          grep --fixed-strings "# Tiger Style Standards Report" "$markdown_report" >/dev/null
          grep --fixed-strings "## Active Suppressions" "$markdown_report" >/dev/null

          touch "$out"
        '';

        policyArtifactConsumerFixtureCheck = mkPolicyArtifactConsumerCheck {
          inherit system;
          src = ./standards/check-fixtures/embedded/compliant;
          family = "embedded";
          expectedDiagnostics = 0;
          requiredPolicyIds = [ "tigerstyle.embedded.bounded-queues" ];
        };
      in
      {
        packages = {
          default = slotcar;
          slotcar = slotcar;
          tigerstyle = slotcar;
          cargo-slotcar = cargoSlotcar;
          cargo-tigerstyle = cargoSlotcar;
          fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmoke;
          dylint-driver = dylintDriver;
          slotcar-standards = slotcarStandards;
          slotcar-standards-nix-closure-manifest = slotcarStandardsNixClosureManifest;
          tigerstyle-policy-export = policyExport;
          tigerstyle-policy-artifacts = policyArtifacts;
          tigerstyle-policy-validate = policyValidate;
        };

        apps = {
          cargo-slotcar = flake-utils.lib.mkApp {
            drv = cargoSlotcar;
            exePath = "/bin/cargo-slotcar";
          };
          cargo-tigerstyle = flake-utils.lib.mkApp {
            drv = cargoSlotcar;
            exePath = "/bin/cargo-tigerstyle";
          };

          fingerprint-docs-downstream-smoke = flake-utils.lib.mkApp {
            drv = fingerprintDocsDownstreamSmoke;
            exePath = "/bin/tigerstyle-fingerprint-docs-downstream-smoke";
          };

          slotcar-standards = flake-utils.lib.mkApp {
            drv = slotcarStandards;
            exePath = "/bin/slotcar-standards";
          };

          tigerstyle-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/tigerstyle-policy-export";
          };

          tigerstyle-policy-artifacts = flake-utils.lib.mkApp {
            drv = policyArtifacts;
            exePath = "/bin/tigerstyle-policy-artifacts";
          };

          tigerstyle-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/tigerstyle-policy-validate";
          };
        };

        checks = let
          mkFingerprintDocsCheck = pname: testFilter:
            craneLib.buildPackage {
              inherit src cargoArtifacts buildInputs;
              cargoToml = ./Cargo.toml;
              inherit pname;
              version = "0.1.0";
              nativeBuildInputs = checkNativeBuildInputs;
              cargoExtraArgs = "-p cargo-slotcar";
              doNotPostBuildInstallCargoBinaries = true;
              buildPhaseCargoCommand = ":";
              cargoTestExtraArgs = "--test fingerprint_consumer ${testFilter} -- --nocapture";
              installPhaseCommand = ''
                mkdir -p "$out"
              '';
              LD_LIBRARY_PATH = rustToolchainLibPath;
              RUSTUP_TOOLCHAIN = rustupToolchain;
              preCheck = ''
                export PATH="${dylintShimPath}:$PATH"
                export CARGO_TIGERSTYLE_BIN="${cargoSlotcarUnwrapped}/bin/cargo-slotcar"
              '';
            };
          fingerprintLifecycleDocs = mkFingerprintDocsCheck
            "tigerstyle-fingerprint-lifecycle-docs"
            "fingerprint_lifecycle_docs_fixture_matches_transcript";
          fingerprintHumanSummaryDocs = mkFingerprintDocsCheck
            "tigerstyle-fingerprint-human-summary-docs"
            "fingerprint_human_summary_docs_fixture_matches_transcript";
          fingerprintWarningSummaryDocs = mkFingerprintDocsCheck
            "tigerstyle-fingerprint-warning-summary-docs"
            "fingerprint_warning_summary_docs_fixture_matches_transcript";
          fingerprintJsonOutputDocs = mkFingerprintDocsCheck
            "tigerstyle-fingerprint-json-output-docs"
            "fingerprint_json_output_docs_fixture_matches_transcript";
          fingerprintSourceScannerDocs = mkFingerprintDocsCheck
            "tigerstyle-fingerprint-source-scanner-docs"
            "fingerprint_source_scanner_docs_fixture_matches_transcript";
          fingerprintDocsDownstreamSmokeCheck = pkgs.runCommand "tigerstyle-fingerprint-docs-downstream-smoke-check" { } ''
            ${fingerprintDocsDownstreamSmoke}/bin/tigerstyle-fingerprint-docs-downstream-smoke
            mkdir -p "$out"
          '';
          fingerprintDocsReviewGateDocsCheck = pkgs.runCommand "tigerstyle-fingerprint-docs-review-gate-docs-check" { } ''
            cd ${self}
            grep -F "For fingerprint docs/examples, use one review command:" docs/contributor-guide.md
            grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" docs/contributor-guide.md
            grep -F "CI/operator gate for executable" docs/contributor-guide.md
            grep -F "focused flake checks or direct Cargo filters only after the aggregate gate" docs/contributor-guide.md
            grep -F "2026-05-05: Function fingerprint docs and checks" CHANGELOG.md
            grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" CHANGELOG.md
            mkdir -p "$out"
          '';
        in {
          inherit slotcar;
          tigerstyle = slotcar;

          nextest = craneLib.cargoNextest {
            inherit src cargoArtifacts buildInputs;
            nativeBuildInputs = checkNativeBuildInputs;
            partitions = 1;
            partitionType = "count";
            CARGO_TIGERSTYLE_BIN = "${cargoSlotcarUnwrapped}/bin/cargo-slotcar";
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export CARGO_TIGERSTYLE_BIN="${cargoSlotcarUnwrapped}/bin/cargo-slotcar"
            '';
          };

          ui = craneLib.buildPackage {
            inherit src cargoArtifacts buildInputs;
            cargoToml = ./Cargo.toml;
            pname = "slotcar-ui";
            version = "0.1.0";
            nativeBuildInputs = checkNativeBuildInputs;
            doNotPostBuildInstallCargoBinaries = true;
            buildPhaseCargoCommand = ":";
            cargoTestExtraArgs = "--test ui -- --ignored --nocapture";
            installPhaseCommand = ''
              mkdir -p "$out"
            '';
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export TIGERSTYLE_UI_DRIVER="${uiDriver}/bin/tigerstyle-ui-driver"
              export TIGERSTYLE_UI_LIBRARY="${slotcarUiLibrary}/lib/${slotcarUiLibraryName}"
            '';
          };

          fingerprint-lifecycle-docs = fingerprintLifecycleDocs;
          fingerprint-human-summary-docs = fingerprintHumanSummaryDocs;
          fingerprint-warning-summary-docs = fingerprintWarningSummaryDocs;
          fingerprint-json-output-docs = fingerprintJsonOutputDocs;
          fingerprint-source-scanner-docs = fingerprintSourceScannerDocs;

          fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmokeCheck;
          fingerprint-docs-review-gate-docs = fingerprintDocsReviewGateDocsCheck;

          docs-examples = pkgs.runCommand "tigerstyle-docs-examples" { } ''
            test -d ${fingerprintLifecycleDocs}
            test -d ${fingerprintHumanSummaryDocs}
            test -d ${fingerprintWarningSummaryDocs}
            test -d ${fingerprintJsonOutputDocs}
            test -d ${fingerprintSourceScannerDocs}
            test -d ${fingerprintDocsDownstreamSmokeCheck}
            test -d ${fingerprintDocsReviewGateDocsCheck}
            mkdir -p "$out"
          '';

          clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts nativeBuildInputs buildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          };

          fmt = craneLib.cargoFmt {
            src = fmtSrc;
          };

          policy-registry = policyRegistryCheck;
          standards-fixtures = standardsFixtureCheck;
          slotcar-standards-nix-closure-manifest = pkgs.runCommand "slotcar-standards-nix-closure-manifest-check" { } ''
            test -s ${slotcarStandardsNixClosureManifest}/proof-nix-closure.txt
            test -s ${slotcarStandardsNixClosureManifest}/slotcar-standards-closure.graph
            test -s ${slotcarStandardsNixClosureManifest}/slotcar-standards-closure.graph.canonical
            test -s ${slotcarStandardsNixClosureManifest}/slotcar-standards-closure.graph.blake3
            grep -F "proof-verifier-nix-derivation:" ${slotcarStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            grep -F "proof-verifier-nix-output: ${slotcarStandards}" ${slotcarStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            grep -F "proof-verifier-nix-closure: exportReferencesGraph:slotcar-standards-closure" ${slotcarStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            graph_digest=$(cat ${slotcarStandardsNixClosureManifest}/slotcar-standards-closure.graph.blake3)
            grep -F "proof-verifier-nix-closure-graph-blake3: $graph_digest" ${slotcarStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            mkdir -p "$out"
          '';
          policy-artifact-consumer-fixture = policyArtifactConsumerFixtureCheck;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            cargo-shim
            rustup-shim
            pkgs.cargo-nextest
            pkgs.cargo-watch
            pkgs.jq
            pkgs.nickel
            pkgs.rust-analyzer
            slotcarStandards
          ];

          inputsFrom = [ slotcar ];

          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          shellHook = ''
            export PATH="${dylintShimPath}:$PATH"
          '';
        };
      }
    );
}
