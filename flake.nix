{
  description = "tigerstyle — Custom lints enforcing Tiger Style coding principles";

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
      # Runs tigerstyle lints against consumer code via nix flake check.
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

          tigerstylePkg = self.packages.${system}.default;
          driverPkg = self.packages.${system}.dylint-driver;
          sharedLibSuffix =
            if pkgs.stdenv.hostPlatform.isDarwin then ".dylib" else ".so";
          dllPrefix =
            if pkgs.stdenv.hostPlatform.isDarwin then "lib" else "lib";
          # Dylint parses the library filename as lib<name>@<toolchain>.<ext>
          tigerstyleLibDir = pkgs.runCommand "tigerstyle-dylint-lib" {} ''
            mkdir -p $out/lib
            ln -s ${tigerstylePkg}/lib/${dllPrefix}tigerstyle${sharedLibSuffix} \
              $out/lib/${dllPrefix}tigerstyle@${tsRustupToolchain}${sharedLibSuffix}
          '';
          tigerstyleLibPath =
            "${tigerstyleLibDir}/lib/${dllPrefix}tigerstyle@${tsRustupToolchain}${sharedLibSuffix}";

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
        pkgs.runCommand "tigerstyle-consumer-check" {
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
          export TIGERSTYLE_LINT_LIB="${tigerstyleLibPath}"
          export TIGERSTYLE_TOOLCHAIN="${tsRustupToolchain}"
          ${pkgs.lib.optionalString (consumerVendorDir != null) ''
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${consumerVendorDir}/config.toml" "$CARGO_HOME/config.toml"
          ''}
          ${self.packages.${system}.cargo-tigerstyle}/bin/cargo-tigerstyle check ${packageArgs}${cargoCheckTail}
          mkdir -p "$out"
        '';
    in
    {
      lib = { inherit mkConsumerCheck; };
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
        tigerstyleLibraryName = "${sharedLibraryPrefix}tigerstyle${sharedLibrarySuffix}";
        tigerstyleUiLibraryName = "${sharedLibraryPrefix}tigerstyle@${rustupToolchain}${sharedLibrarySuffix}";

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
          RUSTUP_TOOLCHAIN = rustupToolchain;
        };

        tigerstyle = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          doCheck = false;
        };

        cargoTigerstyleUnwrapped = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "cargo-tigerstyle";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-tigerstyle";
          doCheck = false;
        };

        tigerstyleStandards = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "tigerstyle-standards";
          version = "0.1.0";
          cargoExtraArgs = "-p tigerstyle-standards";
          doCheck = false;
        };

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
              echo "missing tigerstyle UI driver binary" >&2
              find . -maxdepth 4 \( -name 'tigerstyle-ui-driver' -o -name 'tigerstyle_ui_driver' \) -print >&2 || true
              exit 1
            fi
            cp "$driver_bin" "$out/bin/tigerstyle-ui-driver"
            runHook postInstall
          '';
        };

        tigerstyleUiLibrary = pkgs.runCommand "tigerstyle-ui-library" { } ''
          mkdir -p "$out/lib"
          ln -s "${tigerstyle}/lib/${tigerstyleLibraryName}" "$out/lib/${tigerstyleUiLibraryName}"
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

        cargoTigerstyle = pkgs.symlinkJoin {
          name = "cargo-tigerstyle-0.1.0";
          paths = [ cargoTigerstyleUnwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram "$out/bin/cargo-tigerstyle" \
              --prefix PATH : "${dylintShimPath}" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${tigerstyleUiLibrary}/lib/${tigerstyleUiLibraryName}"
          '';
        };

        policyExport = pkgs.writeShellApplication {
          name = "tigerstyle-policy-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/policies/registry.ncl
          '';
        };

        policyValidate = pkgs.writeShellApplication {
          name = "tigerstyle-policy-validate";
          runtimeInputs = with pkgs; [ gawk jq nickel ];
          text = ''
            nickel typecheck standards/policies/registry.ncl
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status '.policies | length > 0' >/dev/null

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

        standardsFixtureCheck = pkgs.runCommand "tigerstyle-standards-fixtures"
          {
            nativeBuildInputs = with pkgs; [ jq nickel tigerstyleStandards ];
          } ''
          cd ${./.}
          registry_json=$(mktemp)
          compliant_json=$(mktemp)
          noncompliant_json=$(mktemp)
          advisory_json=$(mktemp)
          markdown_report=$(mktemp)

          nickel export --format json standards/policies/registry.ncl > "$registry_json"

          tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/suppressions/compliant \
            --family devtools \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status '.active_suppressions | length == 1' "$compliant_json" >/dev/null

          tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/architecture/compliant \
            --family architecture \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/embedded/compliant \
            --family embedded \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/architecture/advisory \
            --family architecture \
            --output json > "$advisory_json"
          jq --exit-status '.diagnostics | length == 1' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].policy_id == "tigerstyle.architecture.public-api-budget"' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].severity == "warning"' "$advisory_json" >/dev/null

          if tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/suppressions/noncompliant \
            --family devtools \
            --output json > "$noncompliant_json"; then
            echo "noncompliant suppression fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null

          if tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/architecture/noncompliant \
            --family architecture \
            --output json > "$noncompliant_json"; then
            echo "noncompliant architecture fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 5' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.external-domain-leakage")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.root-crate-thinness" and (.path | contains("thick-root")))' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; ((.path | contains("thick-root")) | not) or .policy_id != "tigerstyle.architecture.public-api-budget")' "$noncompliant_json" >/dev/null

          if tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/embedded/noncompliant \
            --family embedded \
            --output json > "$noncompliant_json"; then
            echo "noncompliant embedded fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null

          tigerstyle-standards check \
            --registry "$registry_json" \
            --source standards/check-fixtures/suppressions/compliant \
            --family devtools \
            --output markdown > "$markdown_report"
          grep --fixed-strings "# Tiger Style Standards Report" "$markdown_report" >/dev/null
          grep --fixed-strings "## Active Suppressions" "$markdown_report" >/dev/null

          touch "$out"
        '';
      in
      {
        packages = {
          default = tigerstyle;
          tigerstyle = tigerstyle;
          cargo-tigerstyle = cargoTigerstyle;
          dylint-driver = dylintDriver;
          tigerstyle-standards = tigerstyleStandards;
          tigerstyle-policy-export = policyExport;
          tigerstyle-policy-validate = policyValidate;
        };

        apps = {
          cargo-tigerstyle = flake-utils.lib.mkApp {
            drv = cargoTigerstyle;
            exePath = "/bin/cargo-tigerstyle";
          };

          tigerstyle-standards = flake-utils.lib.mkApp {
            drv = tigerstyleStandards;
            exePath = "/bin/tigerstyle-standards";
          };

          tigerstyle-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/tigerstyle-policy-export";
          };

          tigerstyle-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/tigerstyle-policy-validate";
          };
        };

        checks = {
          inherit tigerstyle;

          nextest = craneLib.cargoNextest {
            inherit src cargoArtifacts buildInputs;
            nativeBuildInputs = checkNativeBuildInputs;
            partitions = 1;
            partitionType = "count";
            CARGO_TIGERSTYLE_BIN = "${cargoTigerstyleUnwrapped}/bin/cargo-tigerstyle";
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export CARGO_TIGERSTYLE_BIN="${cargoTigerstyleUnwrapped}/bin/cargo-tigerstyle"
            '';
          };

          ui = craneLib.buildPackage {
            inherit src cargoArtifacts buildInputs;
            cargoToml = ./Cargo.toml;
            pname = "tigerstyle-ui";
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
              export TIGERSTYLE_UI_LIBRARY="${tigerstyleUiLibrary}/lib/${tigerstyleUiLibraryName}"
            '';
          };

          fingerprint-lifecycle-docs = craneLib.buildPackage {
            inherit src cargoArtifacts buildInputs;
            cargoToml = ./Cargo.toml;
            pname = "tigerstyle-fingerprint-lifecycle-docs";
            version = "0.1.0";
            nativeBuildInputs = checkNativeBuildInputs;
            cargoExtraArgs = "-p cargo-tigerstyle";
            doNotPostBuildInstallCargoBinaries = true;
            buildPhaseCargoCommand = ":";
            cargoTestExtraArgs = "--test fingerprint_consumer fingerprint_lifecycle_docs_fixture_matches_transcript -- --nocapture";
            installPhaseCommand = ''
              mkdir -p "$out"
            '';
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export CARGO_TIGERSTYLE_BIN="${cargoTigerstyleUnwrapped}/bin/cargo-tigerstyle"
            '';
          };

          clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts nativeBuildInputs buildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          };

          fmt = craneLib.cargoFmt {
            src = fmtSrc;
          };

          policy-registry = policyRegistryCheck;
          standards-fixtures = standardsFixtureCheck;
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
            tigerstyleStandards
          ];

          inputsFrom = [ tigerstyle ];

          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          shellHook = ''
            export PATH="${dylintShimPath}:$PATH"
          '';
        };
      }
    );
}
