{
  description = "Octet — Custom lints enforcing Tiger Style coding principles under closed interfaces";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    cairn = {
      url = "github:onixresearch/cairn";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dylint-src = {
      url = "github:trailofbits/dylint/0e0a71eefe6f01563d11acc6e1d7af1d505934a9";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, cairn, dylint-src, ... }:
    let
      # Consumer-check helper — usable from any downstream flake.
      # Runs Octet lints against consumer code via nix flake check.
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

          octetPkg = self.packages.${system}.default;
          driverPkg = self.packages.${system}.dylint-driver;
          sharedLibSuffix =
            if pkgs.stdenv.hostPlatform.isDarwin then ".dylib" else ".so";
          dllPrefix =
            if pkgs.stdenv.hostPlatform.isDarwin then "lib" else "lib";
          # Dylint parses the library filename as lib<name>@<toolchain>.<ext>
          octetDylintLibDir = pkgs.runCommand "octet-dylint-lib" {} ''
            mkdir -p $out/lib
            ln -s ${octetPkg}/lib/${dllPrefix}octet${sharedLibSuffix} \
              $out/lib/${dllPrefix}octet@${tsRustupToolchain}${sharedLibSuffix}
          '';
          octetDylintLibPath =
            "${octetDylintLibDir}/lib/${dllPrefix}octet@${tsRustupToolchain}${sharedLibSuffix}";

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
        pkgs.runCommand "octet-consumer-check" {
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
          export TIGERSTYLE_LINT_LIB="${octetDylintLibPath}"
          export TIGERSTYLE_TOOLCHAIN="${tsRustupToolchain}"
          ${pkgs.lib.optionalString (consumerVendorDir != null) ''
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${consumerVendorDir}/config.toml" "$CARGO_HOME/config.toml"
          ''}
          ${self.packages.${system}.cargo-octet}/bin/cargo-octet check ${packageArgs}${cargoCheckTail}
          mkdir -p "$out"
        '';

      # Generated-policy-artifact consumer helper — usable from downstream
      # flakes when they want the Nickel-authored policy bundle exported by this
      # flake, then consumed by octet-standards in one reproducible check.
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
        pkgs.runCommand "octet-policy-artifact-consumer-check" {
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.jq self.packages.${system}.octet-standards ];
          inherit buildInputs;
        } ''
          artifact_dir=$(mktemp -d)
          report_json=$(mktemp)

          ${self.packages.${system}.octet-policy-artifacts}/bin/octet-policy-artifacts "$artifact_dir"
          test -s "$artifact_dir/octet-policies.json"
          test -s "$artifact_dir/octet-check-config.toml"
          test -s "$artifact_dir/octet-seed-manifests.json"

          cp -r ${src} source
          chmod -R u+w source
          octet-standards check \
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
              inHooks = pkgs.lib.hasSuffix "/hooks" path || pkgs.lib.hasInfix "/hooks/" path;
              isPreCommitHooks = pkgs.lib.hasSuffix "/.pre-commit-hooks.yaml" path;
              isReadme = builtins.baseNameOf path == "README.md";
            in (craneLib.filterCargoSources path type) || inUiTests || inTestFixtures || inDocs || inStandards || inTemplates || inHooks || isPreCommitHooks || isReadme;
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
        productionVerusVersion = "0.2026.04.03.21dfcd2";
        productionVerusReleaseSha256 = "sha256-JGqLytgA8C9xUJg0RpuqAB95pu+2hTxschfvi+Op2MY=";
        productionVerusReleaseSha256Hex = "246a8bcad800f02f71509834469baa001f79a6efb6853c6c7217ef8be3a9d8c6";
        productionVerusRustVersion = "1.94.0";
        productionVerusToolchainLabel = "${productionVerusRustVersion}-${hostTriple}";
        productionVerusRustToolchain = pkgs.rust-bin.stable.${productionVerusRustVersion}.default;
        productionVerusRustcLibPath = "${productionVerusRustToolchain}/lib";
        cargoOctetRuntimePath = pkgs.symlinkJoin {
          name = "cargo-octet-runtime-path";
          paths = [ cargo-shim rustup-shim rustToolchain octetVerus octetProductionVerus ];
        };
        checkNativeBuildInputs = nativeBuildInputs ++ [ cargo-shim rustup-shim pkgs.git pkgs.python3 ];
        sharedLibraryPrefix = if pkgs.stdenv.hostPlatform.isWindows then "" else "lib";
        sharedLibrarySuffix =
          if pkgs.stdenv.hostPlatform.isDarwin then ".dylib"
          else if pkgs.stdenv.hostPlatform.isWindows then ".dll"
          else ".so";
        octetLibraryName = "${sharedLibraryPrefix}octet${sharedLibrarySuffix}";
        octetUiLibraryName = "${sharedLibraryPrefix}octet@${rustupToolchain}${sharedLibrarySuffix}";

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
          RUSTUP_TOOLCHAIN = rustupToolchain;
        };

        octet = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          doCheck = false;
        };

        cargoOctetUnwrapped = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "cargo-octet";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        valenceCore = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "valence-core";
          version = "0.1.0";
          cargoExtraArgs = "-p valence-core";
          doCheck = false;
        };

        cargoOctetFreshBuildA = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "cargo-octet-fresh-build-a";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        cargoOctetFreshBuildB = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "cargo-octet-fresh-build-b";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        fingerprintDocsDownstreamSmoke = pkgs.writeShellApplication {
          name = "octet-fingerprint-docs-downstream-smoke";
          runtimeInputs = [ pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.jq ];
          text = ''
            repo_root="''${1:-${src}}"
            scratch="$(mktemp -d)"
            trap 'rm -rf "$scratch"' EXIT

            mkdir -p "$scratch/consumer/src"
            cat > "$scratch/consumer/Cargo.toml" <<'EOF'
            [package]
            name = "octet-fingerprint-downstream-smoke"
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
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src" > "$scratch/check.json" 2> "$scratch/check.err"
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
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/drift.json" 2> "$scratch/drift.err"
            drift_status="$?"
            set -e
            test "$drift_status" -eq 1
            jq -e '.records | length == 1' "$scratch/drift.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch"' "$scratch/drift.json" > /dev/null

            set +e
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --diff "$scratch/consumer/src/json_output.rs" > "$scratch/diff.txt" 2> "$scratch/diff.err"
            diff_status="$?"
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --diff --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/diff.json" 2> "$scratch/diff-json.err"
            diff_json_status="$?"
            set -e
            test "$diff_status" -eq 1
            test "$diff_json_status" -eq 1
            grep -F "octet fingerprint diff" "$scratch/diff.txt" > /dev/null
            grep -F "capped_retries mismatch" "$scratch/diff.txt" > /dev/null
            grep -F "summary total=1 match=0 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0" "$scratch/diff.txt" > /dev/null
            jq -e '.schema_version == 1 and .mode == "diff"' "$scratch/diff.json" > /dev/null
            jq -e '.records | length == 1' "$scratch/diff.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch" and .records[0].macro_warning == false' "$scratch/diff.json" > /dev/null
            diff_human_name="$(grep -F "capped_retries mismatch" "$scratch/diff.txt" | sed -E 's/.* ([^ ]+) mismatch .*/\1/')"
            diff_json_name="$(jq -r '.records[0].name' "$scratch/diff.json")"
            test "$diff_human_name" = "$diff_json_name"

            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --update --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/update.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "written"' "$scratch/update.json" > /dev/null
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/repaired.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "match"' "$scratch/repaired.json" > /dev/null
          '';
        };

        octetStandards = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
          pname = "octet-standards";
          version = "0.1.0";
          cargoExtraArgs = "-p octet-standards";
          doCheck = false;
        };

        octetStandardsNixClosureManifest = pkgs.runCommand "octet-standards-nix-closure-manifest" {
          nativeBuildInputs = [ pkgs.b3sum pkgs.python3 ];
          exportReferencesGraph = [ "octet-standards-closure" octetStandards ];
        } ''
          mkdir -p "$out"
          cp octet-standards-closure "$out/octet-standards-closure.graph"
          GRAPH="$out/octet-standards-closure.graph" python3 - <<'PY' > "$out/octet-standards-closure.graph.canonical"
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
          graph_digest=$(b3sum "$out/octet-standards-closure.graph.canonical" | cut -d' ' -f1)
          printf '%s\n' "$graph_digest" > "$out/octet-standards-closure.graph.blake3"
          cat > "$out/proof-nix-closure.txt" <<EOF
          proof-verifier-nix-derivation: ${octetStandards.drvPath}
          proof-verifier-nix-output: ${octetStandards}
          proof-verifier-nix-closure: exportReferencesGraph:octet-standards-closure
          proof-verifier-nix-closure-graph-blake3: $graph_digest
          EOF
        '';

        octetVerus = pkgs.writeShellApplication {
          name = "octet-verus";
          runtimeInputs = with pkgs; [ coreutils gnugrep ];
          text = ''
            identity="octet-verus@fixture-v1"
            digest="9f5a7f8c9e1d2c3b4a59687766554433221100ffeeddccbbaa99887766554433"
            pass_signal="proof-result: pass octet-addition-bound"

            if [ "''${1:-}" = "--identity" ]; then
              printf 'proof-verifier-mode: fixture\n'
              printf 'proof-verifier: %s\n' "$identity"
              printf 'proof-verifier-blake3: %s\n' "$digest"
              printf 'proof-verifier-binary: octet-verus\n'
              printf 'proof-verifier-binary-blake3: %s\n' "$digest"
              printf 'proof-toolchain: verus-fixture-wrapper/v1\n'
              printf 'proof-toolchain-blake3: %s\n' "$digest"
              printf 'trust-boundary: fixture verifier validates proof plumbing only\n'
              exit 0
            fi

            if [ "$#" -eq 0 ]; then
              echo "octet-verus: expected at least one proof file" >&2
              exit 2
            fi

            for proof in "$@"; do
              test -s "$proof"
              grep --fixed-strings 'verus!' "$proof" >/dev/null
              if grep --fixed-strings 'ensures false' "$proof" >/dev/null; then
                echo "octet-verus: rejected false postcondition in $proof" >&2
                exit 1
              fi
            done

            printf '%s\n' "$pass_signal"
          '';
        };

        octetProductionVerus = pkgs.stdenvNoCC.mkDerivation {
          pname = "octet-production-verus";
          version = productionVerusVersion;
          src = pkgs.fetchurl {
            url = "https://github.com/verus-lang/verus/releases/download/release/${productionVerusVersion}/verus-${productionVerusVersion}-x86-linux.zip";
            hash = productionVerusReleaseSha256;
          };
          nativeBuildInputs = with pkgs; [ autoPatchelfHook b3sum makeWrapper unzip ];
          buildInputs = with pkgs; [ stdenv.cc.cc.lib zlib productionVerusRustToolchain ];
          unpackPhase = ''
            unzip "$src"
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p "$out/libexec/verus" "$out/bin" "$out/share"
            cp -R verus-x86-linux/. "$out/libexec/verus/"
            chmod -R u+w "$out/libexec/verus"
            find "$out/libexec/verus" -type f \( -name verus -o -name rust_verify -o -name z3 \) -exec chmod +x {} +

            rustup_shim="$out/libexec/verus/rustup"
            cat > "$rustup_shim" <<'EOF'
            #!${pkgs.runtimeShell}
            set -eu
            case "''${1:-}" in
              --version|-V)
                printf 'rustup 1.28.2 (octet shim)\n'
                ;;
              toolchain)
                if [ "''${2:-}" = "list" ]; then
                  printf '${productionVerusToolchainLabel} (default)\n'
                else
                  echo "octet rustup shim: unsupported toolchain ''${2:-}" >&2
                  exit 1
                fi
                ;;
              run)
                requested="''${2:-}"
                shift 2
                if [ "''${1:-}" = "--" ]; then
                  shift
                fi
                case "$requested" in
                  ${productionVerusToolchainLabel}|${productionVerusRustVersion}) ;;
                  *) echo "octet rustup shim: unsupported toolchain $requested" >&2; exit 1 ;;
                esac
                export LD_LIBRARY_PATH="${productionVerusRustcLibPath}:''${LD_LIBRARY_PATH:-}"
                exec "$@"
                ;;
              which)
                case "''${2:-}" in
                  rustc) printf '%s/bin/rustc\n' "${productionVerusRustToolchain}" ;;
                  cargo) printf '%s/bin/cargo\n' "${productionVerusRustToolchain}" ;;
                  *) echo "octet rustup shim: unsupported which ''${2:-}" >&2; exit 1 ;;
                esac
                ;;
              show)
                if [ "''${2:-}" = "active-toolchain" ]; then
                  printf '${productionVerusToolchainLabel} (default)\n'
                else
                  printf '${productionVerusToolchainLabel}\n'
                fi
                ;;
              *) echo "octet rustup shim: unsupported: $*" >&2; exit 1 ;;
            esac
            EOF
            chmod +x "$rustup_shim"

            makeWrapper "$out/libexec/verus/verus" "$out/bin/verus" \
              --prefix PATH : "$out/libexec/verus" \
              --prefix LD_LIBRARY_PATH : "${productionVerusRustcLibPath}"

            verus_binary_blake3="$(b3sum "$out/libexec/verus/verus")"
            verus_binary_blake3="''${verus_binary_blake3%% *}"
            solver_blake3="$(b3sum "$out/libexec/verus/z3")"
            solver_blake3="''${solver_blake3%% *}"
            toolchain_blake3="$(b3sum "${productionVerusRustToolchain}/bin/rustc")"
            toolchain_blake3="''${toolchain_blake3%% *}"

            cat > "$out/share/identity.txt" <<EOF
            proof-verifier-mode: production
            proof-verifier: verus@${productionVerusVersion}
            proof-verifier-release-sha256: ${productionVerusReleaseSha256Hex}
            proof-verifier-binary: verus
            proof-verifier-binary-blake3: $verus_binary_blake3
            proof-toolchain: ${productionVerusToolchainLabel}
            proof-toolchain-blake3: $toolchain_blake3
            proof-solver: z3@verus-${productionVerusVersion}
            proof-solver-blake3: $solver_blake3
            trust-boundary: production Verus binary release verifies the declared proof target only
            EOF

            production_wrapper="$out/bin/octet-production-verus"
            cat > "$production_wrapper" <<EOF
            #!${pkgs.runtimeShell}
            set -eu
            if [ "\''${1:-}" = "--identity" ]; then
              cat "$out/share/identity.txt"
              exit 0
            fi
            exec "$out/bin/verus" "\$@"
            EOF
            chmod +x "$production_wrapper"
            runHook postInstall
          '';
          passthru = {
            inherit productionVerusReleaseSha256Hex productionVerusToolchainLabel productionVerusVersion;
          };
        };

        octetProductionVerusNixClosureManifest = pkgs.runCommand "octet-production-verus-nix-closure-manifest" {
          nativeBuildInputs = [ pkgs.b3sum pkgs.python3 ];
          exportReferencesGraph = [ "octet-production-verus-closure" octetProductionVerus ];
        } ''
          mkdir -p "$out"
          cp octet-production-verus-closure "$out/octet-production-verus-closure.graph"
          python3 - "$out/octet-production-verus-closure.graph" "$out/octet-production-verus-closure.graph.canonical" <<'PY'
          import json
          import sys

          source, dest = sys.argv[1:]
          records = []
          with open(source, "r", encoding="utf-8") as handle:
              lines = [line.rstrip("\n") for line in handle]
          index = 0
          while index < len(lines):
              path = lines[index]
              index += 1
              if not path:
                  continue
              deriver = lines[index]
              index += 1
              refs_count = int(lines[index])
              index += 1
              refs = sorted(lines[index:index + refs_count])
              index += refs_count
              records.append({"path": path, "deriver": deriver, "refs": refs})
          records.sort(key=lambda record: record["path"])
          with open(dest, "w", encoding="utf-8") as handle:
              for record in records:
                  handle.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
                  handle.write("\n")
          PY
          graph_digest=$(b3sum "$out/octet-production-verus-closure.graph.canonical" | cut -d' ' -f1)
          printf '%s\n' "$graph_digest" > "$out/octet-production-verus-closure.graph.blake3"
          cat > "$out/proof-nix-closure.txt" <<EOF
          proof-verifier-nix-output: ${octetProductionVerus}
          proof-verifier-nix-closure: exportReferencesGraph:octet-production-verus-closure
          proof-verifier-nix-closure-graph-blake3: $graph_digest
          EOF
        '';

        uiDriverVendorDir = craneLib.vendorCargoDeps {
          src = uiDriverSrc;
          cargoLock = ./tests/support/ui-driver/Cargo.lock;
        };

        uiDriver = pkgs.stdenv.mkDerivation {
          pname = "octet-ui-driver";
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
              target/release/octet-ui-driver \
              target/release/octet_ui_driver \
              ui-driver/target/release/octet-ui-driver \
              ui-driver/target/release/octet_ui_driver
            do
              if [ -f "$candidate" ]; then
                driver_bin="$candidate"
                break
              fi
            done
            if [ -z "$driver_bin" ]; then
              echo "missing Octet UI driver binary" >&2
              find . -maxdepth 4 \( -name 'octet-ui-driver' -o -name 'octet_ui_driver' \) -print >&2 || true
              exit 1
            fi
            cp "$driver_bin" "$out/bin/octet-ui-driver"
            runHook postInstall
          '';
        };

        octetUiLibrary = pkgs.runCommand "octet-ui-library" { } ''
          mkdir -p "$out/lib"
          ln -s "${octet}/lib/${octetLibraryName}" "$out/lib/${octetUiLibraryName}"
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

        cargoOctet = pkgs.symlinkJoin {
          name = "cargo-octet-0.1.0";
          paths = [ cargoOctetUnwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram "$out/bin/cargo-octet" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}"
            wrapProgram "$out/bin/cargo-slotcar" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}"
            wrapProgram "$out/bin/cargo-tigerstyle" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}"
          '';
        };

        policyExport = pkgs.writeShellApplication {
          name = "octet-policy-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/policies/registry.ncl
          '';
        };

        proofRailManifestExport = pkgs.writeShellApplication {
          name = "octet-proof-rail-manifest-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/proof-rails/manifest.ncl
          '';
        };

        trellisProofImportsExport = pkgs.writeShellApplication {
          name = "octet-trellis-proof-imports-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/proof-imports/trellis.ncl
          '';
        };

        policyArtifacts = pkgs.writeShellApplication {
          name = "octet-policy-artifacts";
          runtimeInputs = with pkgs; [ nickel octetStandards ];
          text = ''
            if [ "$#" -ne 1 ]; then
              echo "usage: octet-policy-artifacts OUT_DIR" >&2
              exit 2
            fi
            cd ${self}
            registry_json=$(mktemp)
            nickel export --format json standards/policies/registry.ncl > "$registry_json"
            exec octet-standards export-artifacts --registry "$registry_json" --out-dir "$1"
          '';
        };

        policyValidate = pkgs.writeShellApplication {
          name = "octet-policy-validate";
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

        policyRegistryCheck = pkgs.runCommand "octet-policy-registry-check"
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

        proofRailManifestCheck = pkgs.runCommand "octet-proof-rail-manifest-check"
          {
            nativeBuildInputs = with pkgs; [ jq nickel ];
          } ''
          cd ${./.}
          manifest_json=$(mktemp)
          trellis_import_json=$(mktemp)
          nickel export --format json standards/proof-rails/manifest.ncl > "$manifest_json"
          jq --exit-status '.schema_version == "octet-proof-rail-manifest/v1"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound.command_argv[0] == "octet-verus"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound.verifier_mode == "fixture"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound.receipt_paths | length > 0' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production.command_argv[0] == "octet-production-verus"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production.verifier_mode == "production"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production.pass_signal == "verification results:: 1 verified, 0 errors"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.ordering_reflexive_production.command_argv == ["octet-production-verus", "verus/proofs/ordering_reflexive.rs"]' "$manifest_json" >/dev/null
          jq --exit-status '.rails.ordering_reflexive_production.source_paths == ["src/verified/ordering.rs"]' "$manifest_json" >/dev/null
          jq --exit-status '.rails.ordering_reflexive_production.verifier_mode == "production"' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production_negative.command_argv == ["octet-production-verus", "verus/proofs/addition_bound_invalid.rs"]' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production_negative.source_paths == ["verus/failure-fixtures/addition_bound_invalid_source.rs"]' "$manifest_json" >/dev/null
          jq --exit-status '.rails.addition_bound_production_negative.verifier_mode == "production"' "$manifest_json" >/dev/null
          production_graph_digest=$(cat ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3)
          jq --exit-status --arg digest "$production_graph_digest" \
            '.rails.addition_bound_production.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
            "$manifest_json" >/dev/null
          jq --exit-status --arg digest "$production_graph_digest" \
            '.rails.ordering_reflexive_production.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
            "$manifest_json" >/dev/null
          jq --exit-status --arg digest "$production_graph_digest" \
            '.rails.addition_bound_production_negative.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
            "$manifest_json" >/dev/null

          nickel export --format json standards/proof-imports/trellis.ncl > "$trellis_import_json"
          cmp "$trellis_import_json" standards/proof-imports/generated/trellis-proof-imports.json
          jq --exit-status '.schema_version == "octet-imported-proof-bundles/v1"' "$trellis_import_json" >/dev/null
          jq --exit-status '.bundles[0].repository_label == "../trellis"' "$trellis_import_json" >/dev/null
          jq --exit-status '.bundles[0].hash_algorithm == "blake3"' "$trellis_import_json" >/dev/null
          jq --exit-status '.bundles[0].intended_evidence_tier == "manual_review_required"' "$trellis_import_json" >/dev/null

          for file in standards/proof-rails/fixtures/valid/*.ncl; do
            nickel export --format json "$file" >/dev/null
          done
          for file in standards/proof-rails/fixtures/invalid/*.ncl; do
            if nickel export --format json "$file" >/dev/null 2>&1; then
              echo "invalid proof-rail manifest fixture passed: $file" >&2
              exit 1
            fi
          done

          touch "$out"
        '';

        productionVerusNegativeProofCheck = pkgs.runCommand "octet-production-verus-negative-proof" { } ''
          set +e
          ${octetProductionVerus}/bin/octet-production-verus ${./verus/proofs/addition_bound_invalid.rs} > stdout.txt 2> stderr.txt
          proof_status="$?"
          set -e
          if [ "$proof_status" -eq 0 ]; then
            echo "invalid production Verus proof unexpectedly passed" >&2
            exit 1
          fi
          if grep -F "verification results:: 1 verified, 0 errors" stdout.txt >/dev/null; then
            echo "invalid production Verus proof printed the success pass signal" >&2
            exit 1
          fi
          mkdir -p "$out"
          cp stdout.txt stderr.txt "$out/"
        '';

        productionVerusNegativeProveCheck = pkgs.runCommand "octet-production-verus-negative-prove"
          {
            nativeBuildInputs = with pkgs; [ gnugrep jq ];
          } ''
          artifact_dir=$(mktemp -d)
          policy_dir=$(mktemp -d)
          repo="$TMPDIR/source"
          cp -r ${src} "$repo"
          chmod -R u+w "$repo"
          ${policyArtifacts}/bin/octet-policy-artifacts "$policy_dir"
          cd "$repo"
          set +e
          ${cargoOctet}/bin/cargo-octet prove \
            --policy-artifacts-dir "$policy_dir" \
            --source verus/failure-fixtures/addition_bound_invalid_source.rs \
            --artifact-dir "$artifact_dir" \
            --output-format json > prove.json 2> prove.err
          prove_status="$?"
          set -e
          if [ "$prove_status" -ne 1 ]; then
            echo "expected cargo octet prove to exit 1 for the negative production rail, got $prove_status" >&2
            exit 1
          fi
          jq --exit-status '.status == "failed"' "$artifact_dir/status.json" >/dev/null
          grep -F 'verifier-failure: proof command `octet-production-verus verus/proofs/addition_bound_invalid.rs` exited without success' "$artifact_dir/summary.txt" >/dev/null
          receipt="$artifact_dir/proofs/octet.verus_proof_rails.addition-bound-production-negative-rail-0.receipt"
          grep -F '"status": "failed"' "$receipt" >/dev/null
          grep -F '"stdout_artifact"' "$receipt" >/dev/null
          grep -F '"stderr_artifact"' "$receipt" >/dev/null
          if grep -F '"evidence_tier":"formal_proof"' "$artifact_dir/provenance.jsonl" >/dev/null; then
            echo "failed production Verus rail emitted formal_proof provenance" >&2
            exit 1
          fi
          mkdir -p "$out"
          cp -r "$artifact_dir" "$out/artifacts"
          cp prove.json prove.err "$out/"
        '';

        productionVerusMultiRailProveCheck = pkgs.runCommand "octet-production-verus-multi-rail-prove"
          {
            nativeBuildInputs = with pkgs; [ jq ];
          } ''
          expected_total_proof_count=3
          expected_production_proof_count=2
          artifact_dir=$(mktemp -d)
          policy_dir=$(mktemp -d)
          repo="$TMPDIR/source"
          cp -r ${src} "$repo"
          chmod -R u+w "$repo"
          ${policyArtifacts}/bin/octet-policy-artifacts "$policy_dir"
          cd "$repo"
          ${cargoOctet}/bin/cargo-octet prove \
            --policy-artifacts-dir "$policy_dir" \
            --source src/verified \
            --artifact-dir "$artifact_dir" \
            --output-format json > prove.json
          jq --exit-status --argjson expected "$expected_total_proof_count" \
            '.status == "passed" and (.proofs | length) == $expected' prove.json >/dev/null
          jq --exit-status --argjson expected "$expected_production_proof_count" '
            ([.proofs[] | select(.verifier_mode == "production" and .status == "passed")] | length) == $expected
          ' prove.json >/dev/null
          jq --exit-status '
            any(.proofs[]; .policy_id == "octet.verus_proof_rails.addition-bound-production" and .receipt == "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt")
            and any(.proofs[]; .policy_id == "octet.verus_proof_rails.ordering-reflexive-production" and .receipt == "proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt")
          ' prove.json >/dev/null
          jq --slurp --exit-status --argjson expected "$expected_production_proof_count" '
            ([.[] | select(.evidence_tier == "formal_proof" and (.artifact_locator | test("addition-bound-production|ordering-reflexive-production")))] | length) == $expected
          ' "$artifact_dir/provenance.jsonl" >/dev/null
          ${cargoOctet}/bin/cargo-octet artifact verify --artifact-dir "$artifact_dir" --output-format json > verify.json
          jq --exit-status '.status == "valid" and (.diagnostics | length) == 0' verify.json >/dev/null
          mkdir -p "$out"
          cp -r "$artifact_dir" "$out/artifacts"
          cp prove.json verify.json "$out/"
        '';

        standardsFixtureCheck = pkgs.runCommand "octet-standards-fixtures"
          {
            nativeBuildInputs = with pkgs; [ jq nickel octetStandards ];
          } ''
          cd ${./.}
          artifact_dir=$(mktemp -d)
          registry_json=$(mktemp)
          compliant_json=$(mktemp)
          noncompliant_json=$(mktemp)
          advisory_json=$(mktemp)
          markdown_report=$(mktemp)

          nickel export --format json standards/policies/registry.ncl > "$registry_json"
          octet-standards export-artifacts --registry "$registry_json" --out-dir "$artifact_dir"
          test -s "$artifact_dir/octet-policies.json"
          test -s "$artifact_dir/octet-check-config.toml"
          test -s "$artifact_dir/octet-seed-manifests.json"
          test -s "$artifact_dir/octet-proof-bundles.json"
          test -s "$artifact_dir/octet-config-receipt-families.json"
          test -s "$artifact_dir/octet-artifact-manifest.json"
          jq --exit-status '.schema_version == "octet-artifact-manifest/v1"' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
          jq --exit-status '(.artifacts | length) == 5' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "octet-policies.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "octet-config-receipt-families.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
          jq --exit-status 'any(.artifacts[]; .path == "octet-check-config.toml" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
          jq --exit-status '.schema_version == "octet-proof-bundle/v1"' "$artifact_dir/octet-proof-bundles.json" >/dev/null
          jq --exit-status 'any(.proof_bundles[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers")' "$artifact_dir/octet-proof-bundles.json" >/dev/null
          jq --exit-status 'any(.proof_bundles[]; .proof_rail.receipt_verifier.verifier_nix_closure_graph_paths[] == "proof-nix-closure.graph")' "$artifact_dir/octet-proof-bundles.json" >/dev/null
          grep --fixed-strings "structural_policy_ids" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_forbidden_apis" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "tokio::sync::mpsc::unbounded_channel" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_capacity_witnesses" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "bounded_queue_backpressure_behaviors" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "capability_forbidden_ambient_apis" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "std::process::" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "capability_core_source_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "capability_shell_constructor_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "capability_approved_names" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_source_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_raw_alias_patterns" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_raw_transition_patterns" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_required_state_markers" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "type_state_required_transition_methods" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "send_raw(" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_backends" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "local-command-receipt" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_commands" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "bash proof-command.sh" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_command_argv" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "octet-verus" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_verifier_modes" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "fixture" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_pass_signals" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof-result: pass deterministic-effect-handlers" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_bindings" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "blake3-command-and-source" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_schemas" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "tigerstyle-proof-receipt/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_runners" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "octet-standards" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_environments" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "local-source-root" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_requires" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "trust-boundary" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifiers" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "octet-standards@proof-receipt/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_binaries" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_binary_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "ca07415846ec49c45311f7b889ba9be3f316e180ad51eab64b0903c5dcd27821" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closures" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "nix-octet-standards-build-closure/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closure_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "d482e4bbbb8e9e1a102319c19c55327cc6944fbb9abbd67122e1f9f88a9109ee" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_build_closure_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof-build-closure.txt" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_derivations" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "octet-standards.drv" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_derivation_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "395cf66a51543cf89028075deef9cf172bac902fd777f0ba0bead91aa68afec4" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closures" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "nix-store-recursive-closure/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "b1501c0c0d38a17b4a5ddd298136b1c168a499237b872cf02fc72f2650a002f2" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof-nix-closure.txt" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "0ed5ac9f1899c34eeda9035c5a165eefeb0beb254a1e36e354e458ae97d4943b" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof-nix-closure.graph" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_toolchains" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "rust-toolchain.toml" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "proof_rail_receipt_toolchain_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
          grep --fixed-strings "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8" "$artifact_dir/octet-check-config.toml" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/suppressions/compliant \
            --family devtools \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status '.active_suppressions | length == 1' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/compliant \
            --family architecture \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and .target_enforcement_mode == "structural" and .failure_type == "crate-boundary")' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/embedded/compliant \
            --family embedded \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/compliant/typestate-protocol \
            --family idioms \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/compliant \
            --family verification \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers" and .current_enforcement_mode == "verified" and .failure_type == "verified-proof-rail")' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/verification/compliant \
            --family verification \
            --execute-proof-rails \
            --output json > "$compliant_json"
          jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

          octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/architecture/advisory \
            --family architecture \
            --output json > "$advisory_json"
          jq --exit-status '.diagnostics | length == 1' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].policy_id == "tigerstyle.architecture.public-api-budget"' "$advisory_json" >/dev/null
          jq --exit-status '.diagnostics[0].severity == "warning"' "$advisory_json" >/dev/null

          if octet-standards check \
            --policy-artifacts-dir "$artifact_dir" \
            --source standards/check-fixtures/suppressions/noncompliant \
            --family devtools \
            --output json > "$noncompliant_json"; then
            echo "noncompliant suppression fixtures unexpectedly passed" >&2
            exit 1
          fi
          jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null

          if octet-standards check \
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
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and (.message | contains("must depend on core crate `tigerstyle-core`")))' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.effect-capability-boundaries")' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; .policy_id != "tigerstyle.idioms.nominal-type-boundaries")' "$noncompliant_json" >/dev/null
          jq --exit-status 'any(.policy_evaluations[]; .target_enforcement_mode == "structural")' "$noncompliant_json" >/dev/null
          jq --exit-status 'all(.diagnostics[]; ((.path | contains("thick-root")) | not) or .policy_id != "tigerstyle.architecture.public-api-budget")' "$noncompliant_json" >/dev/null

          if octet-standards check \
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

          if octet-standards check \
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

          if octet-standards check \
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

          if octet-standards check \
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

          octet-standards check \
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
          default = octet;
          octet = octet;
          slotcar = octet;
          tigerstyle = octet;
          cargo-octet = cargoOctet;
          valence-core = valenceCore;
          cargo-slotcar = cargoOctet;
          cargo-tigerstyle = cargoOctet;
          fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmoke;
          cairn = cairn.packages.${system}.cairn;
          dylint-driver = dylintDriver;
          octet-standards = octetStandards;
          slotcar-standards = octetStandards;
          tigerstyle-standards = octetStandards;
          octet-standards-nix-closure-manifest = octetStandardsNixClosureManifest;
          octet-verus = octetVerus;
          octet-production-verus = octetProductionVerus;
          octet-production-verus-nix-closure-manifest = octetProductionVerusNixClosureManifest;
          slotcar-standards-nix-closure-manifest = octetStandardsNixClosureManifest;
          octet-policy-export = policyExport;
          tigerstyle-policy-export = policyExport;
          octet-proof-rail-manifest-export = proofRailManifestExport;
          octet-trellis-proof-imports-export = trellisProofImportsExport;
          octet-policy-artifacts = policyArtifacts;
          tigerstyle-policy-artifacts = policyArtifacts;
          octet-policy-validate = policyValidate;
          tigerstyle-policy-validate = policyValidate;
        };

        apps = {
          cargo-octet = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-octet";
          };

          cargo-slotcar = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-slotcar";
          };
          cargo-tigerstyle = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-tigerstyle";
          };

          fingerprint-docs-downstream-smoke = flake-utils.lib.mkApp {
            drv = fingerprintDocsDownstreamSmoke;
            exePath = "/bin/octet-fingerprint-docs-downstream-smoke";
          };

          cairn = flake-utils.lib.mkApp {
            drv = cairn.packages.${system}.cairn;
            exePath = "/bin/cairn";
          };

          octet-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/octet-standards";
          };

          slotcar-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/slotcar-standards";
          };

          tigerstyle-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/tigerstyle-standards";
          };

          octet-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/octet-policy-export";
          };

          tigerstyle-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/octet-policy-export";
          };

          octet-proof-rail-manifest-export = flake-utils.lib.mkApp {
            drv = proofRailManifestExport;
            exePath = "/bin/octet-proof-rail-manifest-export";
          };

          octet-trellis-proof-imports-export = flake-utils.lib.mkApp {
            drv = trellisProofImportsExport;
            exePath = "/bin/octet-trellis-proof-imports-export";
          };

          octet-policy-artifacts = flake-utils.lib.mkApp {
            drv = policyArtifacts;
            exePath = "/bin/octet-policy-artifacts";
          };

          tigerstyle-policy-artifacts = flake-utils.lib.mkApp {
            drv = policyArtifacts;
            exePath = "/bin/octet-policy-artifacts";
          };

          octet-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/octet-policy-validate";
          };

          octet-verus = flake-utils.lib.mkApp {
            drv = octetVerus;
            exePath = "/bin/octet-verus";
          };

          octet-production-verus = flake-utils.lib.mkApp {
            drv = octetProductionVerus;
            exePath = "/bin/octet-production-verus";
          };

          tigerstyle-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/octet-policy-validate";
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
              cargoExtraArgs = "-p cargo-octet";
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
                export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
              '';
            };
          fingerprintLifecycleDocs = mkFingerprintDocsCheck
            "octet-fingerprint-lifecycle-docs"
            "fingerprint_lifecycle_docs_fixture_matches_transcript";
          fingerprintHumanSummaryDocs = mkFingerprintDocsCheck
            "octet-fingerprint-human-summary-docs"
            "fingerprint_human_summary_docs_fixture_matches_transcript";
          fingerprintWarningSummaryDocs = mkFingerprintDocsCheck
            "octet-fingerprint-warning-summary-docs"
            "fingerprint_warning_summary_docs_fixture_matches_transcript";
          fingerprintJsonOutputDocs = mkFingerprintDocsCheck
            "octet-fingerprint-json-output-docs"
            "fingerprint_json_output_docs_fixture_matches_transcript";
          fingerprintSourceScannerDocs = mkFingerprintDocsCheck
            "octet-fingerprint-source-scanner-docs"
            "fingerprint_source_scanner_docs_fixture_matches_transcript";
          releaseOperatorSmokeDocs = mkFingerprintDocsCheck
            "octet-release-operator-smoke-docs"
            "release_operator_smoke_transcript_and_help_cover_operator_surfaces";
          objectCorpusGate = mkFingerprintDocsCheck
            "octet-object-corpus-gate"
            "object_internal_corpus";
          objectCorpusCompatibilityGate = mkFingerprintDocsCheck
            "octet-object-corpus-compatibility-gate"
            "object_corpus_compatibility";
          objectCorpusCaveatTaxonomyGate = mkFingerprintDocsCheck
            "octet-object-corpus-caveat-taxonomy-gate"
            "caveat_taxonomy";
          objectCorpusFixtureFreshnessGate = mkFingerprintDocsCheck
            "octet-object-corpus-fixture-freshness"
            "matches_fresh";
          supplementCompatibilityGate = mkFingerprintDocsCheck
            "octet-supplement-compatibility-gate"
            "supplement_compatibility";
          valenceEvidenceIndexGate = pkgs.runCommand "octet-valence-evidence-index" { } ''
            cd ${src}
            ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index check --page docs/current-head-valence-evidence.md
            mkdir -p "$out"
          '';
          thirSupplementInventoryGate = mkFingerprintDocsCheck
            "octet-thir-supplement-inventory-gate"
            "thir_inventory";
          mirSupplementFixtureGate = mkFingerprintDocsCheck
            "octet-mir-supplement-fixture-gate"
            "mir_fixture";
          mirSupplementInventoryGate = mkFingerprintDocsCheck
            "octet-mir-supplement-inventory-gate"
            "mir_fixture";
          freshBuildObjectCorpusDeterminism = pkgs.runCommand "octet-fresh-build-object-corpus-determinism" {
            nativeBuildInputs = [ pkgs.coreutils pkgs.diffutils pkgs.jq ];
          } ''
            repo=${src}
            scratch="$(mktemp -d)"
            trap 'rm -rf "$scratch"' EXIT
            fixture="$repo/cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json"
            mapfile -t sources < <(jq -r '.source_paths[]' "$fixture")

            cd "$repo"
            ${cargoOctetFreshBuildA}/bin/cargo-octet object corpus receipt --output "$scratch/fresh-a.json" "''${sources[@]}"
            ${cargoOctetFreshBuildB}/bin/cargo-octet object corpus receipt --output "$scratch/fresh-b.json" "''${sources[@]}"

            cmp "$scratch/fresh-a.json" "$scratch/fresh-b.json"
            cmp "$scratch/fresh-a.json" "$fixture"
            for field in schema schema_version object_count source_paths object_set_hash caveat_counts pure_cache_blocked_count boundary; do
              jq -S ".$field" "$scratch/fresh-a.json" > "$scratch/a-$field.json"
              jq -S ".$field" "$scratch/fresh-b.json" > "$scratch/b-$field.json"
              jq -S ".$field" "$fixture" > "$scratch/fixture-$field.json"
              cmp "$scratch/a-$field.json" "$scratch/b-$field.json"
              cmp "$scratch/a-$field.json" "$scratch/fixture-$field.json"
            done
            jq -e '.boundary.claims_correctness == false and .boundary.claims_semantic_equivalence == false' "$scratch/fresh-a.json" > /dev/null
            jq -e '.boundary.detail | contains("do not prove behavioral correctness") and contains("semantic equivalence")' "$scratch/fresh-a.json" > /dev/null

            echo "fresh_build.receipt_producers=2"
            echo "fresh_build.byte_stability: PASS"
            echo "fresh_build.fixture_parity: PASS"
            echo "fresh_build.compared_fields_stable: PASS"
            echo "fresh_build.boundary_claims_false: PASS"
            echo "fresh_build.scope=bounded-local-fresh-build-evidence-replay"
            mkdir -p "$out"
          '';
          valenceObjectEvidenceGate = pkgs.runCommand "octet-valence-object-evidence" { } ''
            test -d ${objectCorpusGate}
            test -d ${objectCorpusCompatibilityGate}
            test -d ${objectCorpusCaveatTaxonomyGate}
            test -d ${objectCorpusFixtureFreshnessGate}
            test -d ${supplementCompatibilityGate}
            test -d ${valenceEvidenceIndexGate}
            test -d ${valenceEvidenceReleaseGate}
            test -d ${valenceCorePackageReadiness}
            test -d ${thirSupplementInventoryGate}
            test -d ${mirSupplementFixtureGate}
            test -d ${mirSupplementInventoryGate}
            test -d ${freshBuildObjectCorpusDeterminism}

            echo "valence_object_evidence.object_corpus_gate: PASS"
            echo "valence_object_evidence.compatibility_gate: PASS"
            echo "valence_object_evidence.caveat_taxonomy_gate: PASS"
            echo "valence_object_evidence.object_corpus_fixture_freshness_gate: PASS"
            echo "valence_object_evidence.supplement_compatibility_gate: PASS"
            echo "valence_object_evidence.evidence_index_gate: PASS"
            echo "valence_object_evidence.evidence_release_gate: PASS"
            echo "valence_object_evidence.valence_core_package_readiness: PASS"
            echo "valence_object_evidence.thir_supplement_inventory_gate: PASS"
            echo "valence_object_evidence.mir_supplement_fixture_gate: PASS"
            echo "valence_object_evidence.mir_supplement_inventory_gate: PASS"
            echo "valence_object_evidence.fresh_build_determinism_gate: PASS"
            echo "valence_object_evidence.scope=bounded-local-selected-corpus-evidence-replay"
            echo "valence_object_evidence.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism"
            mkdir -p "$out"
          '';
          valenceEvidenceReleaseGate = pkgs.runCommand "octet-valence-evidence-release" {
            nativeBuildInputs = [ pkgs.b3sum pkgs.coreutils pkgs.jq ];
          } ''
            repo=${src}
            bundle="$out/bundle"
            mkdir -p "$bundle"

            test -d ${valenceEvidenceIndexGate}
            cd "$repo"
            ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index check --page docs/current-head-valence-evidence.md \
              | tee "$bundle/evidence-index-check.txt"
            grep -F "valence_evidence_index.valid: PASS" "$bundle/evidence-index-check.txt" >/dev/null
            grep -F "valence_evidence_index.issue_count=0" "$bundle/evidence-index-check.txt" >/dev/null
            cp docs/current-head-valence-evidence.md "$bundle/current-head-valence-evidence.md"

            page_hash="$(b3sum "$bundle/current-head-valence-evidence.md" | cut -d ' ' -f1)"
            check_hash="$(b3sum "$bundle/evidence-index-check.txt" | cut -d ' ' -f1)"
            cat > "$bundle/valence-evidence-release-index.json" <<EOF
            {
              "schema": "octet.valence-evidence-release.v1",
              "scope": "bounded-local-selected-corpus-evidence-replay",
              "artifacts": [
                {
                  "path": "current-head-valence-evidence.md",
                  "role": "valence_current_head_evidence",
                  "blake3": "$page_hash"
                },
                {
                  "path": "evidence-index-check.txt",
                  "role": "valence_evidence_index_check",
                  "blake3": "$check_hash"
                }
              ],
              "required_gates": [
                "valence-object-evidence",
                "valence-evidence-index"
              ],
              "non_claims": [
                "correctness",
                "semantic_equivalence",
                "exhaustive_rust_coverage",
                "cross_machine_determinism",
                "cross_toolchain_determinism",
                "general_hir_thir_mir_extraction"
              ]
            }
            EOF

            jq --exit-status '.schema == "octet.valence-evidence-release.v1"' \
              "$bundle/valence-evidence-release-index.json" >/dev/null
            jq --exit-status '.scope == "bounded-local-selected-corpus-evidence-replay"' \
              "$bundle/valence-evidence-release-index.json" >/dev/null
            jq --exit-status '.artifacts | length == 2' \
              "$bundle/valence-evidence-release-index.json" >/dev/null
            jq --exit-status '(.non_claims | index("correctness")) and (.non_claims | index("semantic_equivalence")) and (.non_claims | index("general_hir_thir_mir_extraction"))' \
              "$bundle/valence-evidence-release-index.json" >/dev/null
            jq -r '.artifacts[] | [.path, .blake3] | @tsv' \
              "$bundle/valence-evidence-release-index.json" \
              | while IFS=$'\t' read -r artifact_path expected_hash; do
                  actual_hash="$(b3sum "$bundle/$artifact_path" | cut -d ' ' -f1)"
                  test "$actual_hash" = "$expected_hash"
                done

            echo "valence_evidence_release.bundle_index: PASS"
            echo "valence_evidence_release.artifact_hashes: PASS"
            echo "valence_evidence_release.evidence_index_check: PASS"
            echo "valence_evidence_release.scope=bounded-local-selected-corpus-evidence-replay"
            echo "valence_evidence_release.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism,general_hir_thir_mir_extraction"
          '';
          valenceCorePackageReadiness = pkgs.runCommand "octet-valence-core-package-readiness" {
            nativeBuildInputs = [ pkgs.coreutils pkgs.jq ];
          } ''
            repo=${src}
            manifest="$repo/crates/valence-core/Cargo.toml"
            readme="$repo/crates/valence-core/README.md"
            lib_rs="$repo/crates/valence-core/src/lib.rs"

            test -d ${valenceCore}
            test -s "$readme"
            grep -F 'name = "valence-core"' "$manifest" >/dev/null
            grep -F 'readme = "README.md"' "$manifest" >/dev/null
            grep -F 'repository = "https://github.com/OnixResearch/octet"' "$manifest" >/dev/null
            grep -F 'documentation = "https://github.com/OnixResearch/octet/tree/main/crates/valence-core"' "$manifest" >/dev/null
            grep -F 'keywords = ["valence", "evidence", "octet", "blake3"]' "$manifest" >/dev/null
            grep -F 'categories = ["development-tools"]' "$manifest" >/dev/null
            grep -F 'pub mod evidence_index;' "$lib_rs" >/dev/null
            grep -F 'pub mod function_object;' "$lib_rs" >/dev/null
            grep -F 'not a standalone release cutover' "$readme" >/dev/null
            grep -F 'does not claim behavioral correctness' "$readme" >/dev/null

            echo "valence_core_package_readiness.build: PASS"
            echo "valence_core_package_readiness.metadata: PASS"
            echo "valence_core_package_readiness.public_modules: PASS"
            echo "valence_core_package_readiness.boundary_non_claims: PASS"
            echo "valence_core_package_readiness.scope=in-repository-package-readiness-no-publication"
            mkdir -p "$out"
          '';
          roadmapAgentSurfacesDocs = craneLib.buildPackage {
            inherit src cargoArtifacts buildInputs;
            cargoToml = ./Cargo.toml;
            pname = "octet-roadmap-agent-surfaces-docs";
            version = "0.1.0";
            nativeBuildInputs = checkNativeBuildInputs;
            cargoExtraArgs = "-p cargo-octet";
            doNotPostBuildInstallCargoBinaries = true;
            buildPhaseCargoCommand = ":";
            cargoTestExtraArgs = "--test unison_roadmap roadmap_docs_transcript_covers_agent_surfaces -- --nocapture";
            installPhaseCommand = ''
              mkdir -p "$out"
            '';
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
            '';
          };
          fingerprintDocsDownstreamSmokeCheck = pkgs.runCommand "octet-fingerprint-docs-downstream-smoke-check" { } ''
            ${fingerprintDocsDownstreamSmoke}/bin/octet-fingerprint-docs-downstream-smoke
            mkdir -p "$out"
          '';
          fingerprintDocsReviewGateDocsCheck = pkgs.runCommand "octet-fingerprint-docs-review-gate-docs-check" { } ''
            cd ${self}
            grep -F "For fingerprint docs/examples, use one review command:" docs/contributor-guide.md
            grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" docs/contributor-guide.md
            grep -F "CI/operator gate for executable" docs/contributor-guide.md
            grep -F "focused flake checks or direct Cargo filters only after the aggregate gate" docs/contributor-guide.md
            grep -F "2026-05-05: Function fingerprint docs and checks" CHANGELOG.md
            grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" CHANGELOG.md
            mkdir -p "$out"
          '';
          releaseCandidateSmokeCheck = pkgs.runCommand "octet-release-candidate-smoke" { } ''
            repo=${self}
            grep -F 'Candidate version: `0.1.0`' "$repo/docs/release-candidate.md"
            grep -F 'nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link' "$repo/docs/release-candidate.md"
            grep -F 'nix flake check -L' "$repo/docs/release-candidate.md"
            grep -F 'cargo-octet' "$repo/docs/release-candidate.md"
            grep -F 'cargo-tigerstyle' "$repo/docs/release-candidate.md"
            grep -F 'slotcar' "$repo/docs/release-candidate.md"
            grep -F 'tigerstyle' "$repo/docs/release-candidate.md"
            test -x ${cargoOctetUnwrapped}/bin/cargo-octet
            test -x ${cargoOctetUnwrapped}/bin/cargo-slotcar
            test -x ${cargoOctetUnwrapped}/bin/cargo-tigerstyle
            ${cargoOctetUnwrapped}/bin/cargo-octet --version | tee "$TMPDIR/cargo-octet.version"
            ${cargoOctetUnwrapped}/bin/cargo-slotcar --version | tee "$TMPDIR/cargo-slotcar.version"
            ${cargoOctetUnwrapped}/bin/cargo-tigerstyle --version | tee "$TMPDIR/cargo-tigerstyle.version"
            grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-octet.version"
            grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-slotcar.version"
            grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-tigerstyle.version"
            test -d ${octet}
            test -d ${cargoOctetUnwrapped}
            mkdir -p "$out"
          '';
        in {
          octet = octet;
          slotcar = octet;
          tigerstyle = octet;

          nextest = craneLib.cargoNextest {
            inherit src cargoArtifacts buildInputs;
            nativeBuildInputs = checkNativeBuildInputs;
            partitions = 1;
            partitionType = "count";
            CARGO_OCTET_BIN = "${cargoOctetUnwrapped}/bin/cargo-octet";
            LD_LIBRARY_PATH = rustToolchainLibPath;
            RUSTUP_TOOLCHAIN = rustupToolchain;
            preCheck = ''
              export PATH="${dylintShimPath}:$PATH"
              export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
            '';
          };

          ui = craneLib.buildPackage {
            inherit src cargoArtifacts buildInputs;
            cargoToml = ./Cargo.toml;
            pname = "octet-ui";
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
              export OCTET_UI_DRIVER="${uiDriver}/bin/octet-ui-driver"
              export OCTET_UI_LIBRARY="${octetUiLibrary}/lib/${octetUiLibraryName}"
              export TIGERSTYLE_UI_DRIVER="$OCTET_UI_DRIVER"
              export TIGERSTYLE_UI_LIBRARY="$OCTET_UI_LIBRARY"
            '';
          };

          fingerprint-lifecycle-docs = fingerprintLifecycleDocs;
          fingerprint-human-summary-docs = fingerprintHumanSummaryDocs;
          fingerprint-warning-summary-docs = fingerprintWarningSummaryDocs;
          fingerprint-json-output-docs = fingerprintJsonOutputDocs;
          fingerprint-source-scanner-docs = fingerprintSourceScannerDocs;
          release-operator-smoke-docs = releaseOperatorSmokeDocs;
          object-corpus-gate = objectCorpusGate;
          object-corpus-compatibility-gate = objectCorpusCompatibilityGate;
          object-corpus-caveat-taxonomy-gate = objectCorpusCaveatTaxonomyGate;
          object-corpus-fixture-freshness = objectCorpusFixtureFreshnessGate;
          supplement-compatibility-gate = supplementCompatibilityGate;
          valence-evidence-index = valenceEvidenceIndexGate;
          valence-evidence-release = valenceEvidenceReleaseGate;
          valence-core-package-readiness = valenceCorePackageReadiness;
          thir-supplement-inventory-gate = thirSupplementInventoryGate;
          mir-supplement-fixture-gate = mirSupplementFixtureGate;
          mir-supplement-inventory-gate = mirSupplementInventoryGate;
          fresh-build-object-corpus-determinism = freshBuildObjectCorpusDeterminism;
          valence-object-evidence = valenceObjectEvidenceGate;
          roadmap-agent-surfaces-docs = roadmapAgentSurfacesDocs;
          release-candidate-smoke = releaseCandidateSmokeCheck;

          fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmokeCheck;
          fingerprint-docs-review-gate-docs = fingerprintDocsReviewGateDocsCheck;

          docs-examples = pkgs.runCommand "octet-docs-examples" { } ''
            test -d ${fingerprintLifecycleDocs}
            test -d ${fingerprintHumanSummaryDocs}
            test -d ${fingerprintWarningSummaryDocs}
            test -d ${fingerprintJsonOutputDocs}
            test -d ${fingerprintSourceScannerDocs}
            test -d ${releaseOperatorSmokeDocs}
            test -d ${roadmapAgentSurfacesDocs}
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
          proof-rail-manifest = proofRailManifestCheck;
          production-verus-negative-proof = productionVerusNegativeProofCheck;
          production-verus-negative-prove = productionVerusNegativeProveCheck;
          production-verus-multi-rail-prove = productionVerusMultiRailProveCheck;
          standards-fixtures = standardsFixtureCheck;
          octet-standards-nix-closure-manifest = pkgs.runCommand "octet-standards-nix-closure-manifest-check" { } ''
            test -s ${octetStandardsNixClosureManifest}/proof-nix-closure.txt
            test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph
            test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.canonical
            test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.blake3
            grep -F "proof-verifier-nix-derivation:" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            grep -F "proof-verifier-nix-output: ${octetStandards}" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            grep -F "proof-verifier-nix-closure: exportReferencesGraph:octet-standards-closure" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            graph_digest=$(cat ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.blake3)
            grep -F "proof-verifier-nix-closure-graph-blake3: $graph_digest" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
            mkdir -p "$out"
          '';
          octet-production-verus-nix-closure-manifest = pkgs.runCommand "octet-production-verus-nix-closure-manifest-check" { } ''
            test -s ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt
            test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph
            test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.canonical
            test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3
            grep -F "proof-verifier-nix-output: ${octetProductionVerus}" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
            grep -F "proof-verifier-nix-closure: exportReferencesGraph:octet-production-verus-closure" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
            graph_digest=$(cat ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3)
            grep -F "proof-verifier-nix-closure-graph-blake3: $graph_digest" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
            mkdir -p "$out"
          '';
          slotcar-standards-nix-closure-manifest = self.checks.${system}.octet-standards-nix-closure-manifest;
          policy-artifact-consumer-fixture = policyArtifactConsumerFixtureCheck;
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            cargo-shim
            rustup-shim
            pkgs.cargo-nextest
            pkgs.cargo-watch
            cairn.packages.${system}.cairn
            pkgs.jq
            pkgs.nickel
            pkgs.rust-analyzer
            octetStandards
            octetVerus
            octetProductionVerus
          ];

          inputsFrom = [ octet ];

          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          shellHook = ''
            export PATH="${dylintShimPath}:$PATH"
          '';
        };
      }
    );
}
