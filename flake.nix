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
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

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

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src nativeBuildInputs buildInputs;
        };

        # dylint_testing calls `rustup which rustc` to find the compiler.
        # In nix there's no rustup — this shim delegates to the nix toolchain.
        rustup-shim = pkgs.writeShellScriptBin "rustup" ''
          case "$1" in
            which) which "$2" ;;
            show) echo "nix-managed toolchain" ;;
            *) echo "rustup shim: unsupported: $*" >&2; exit 1 ;;
          esac
        '';

        tigerstyle = craneLib.buildPackage {
          inherit src cargoArtifacts nativeBuildInputs buildInputs;
        };
      in
      {
        packages = {
          default = tigerstyle;
          tigerstyle = tigerstyle;
        };

        checks = {
          inherit tigerstyle;

          nextest = craneLib.cargoNextest {
            inherit src cargoArtifacts nativeBuildInputs buildInputs;
            partitions = 1;
            partitionType = "count";
          };

          clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts nativeBuildInputs buildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          };

          fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            rustup-shim
            pkgs.cargo-nextest
            pkgs.cargo-watch
            pkgs.rust-analyzer
          ];

          inputsFrom = [ tigerstyle ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ rustToolchain ];
          RUSTUP_TOOLCHAIN = "nightly";
        };
      }
    );
}
