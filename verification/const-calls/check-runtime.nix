# Diagnostic `cargo octet check` runtime only; no proof-tool fallback.
# Audit --dry-run before realization. The compiler must already exist.
{ repository }:
let
  flake = builtins.getFlake ("path:" + toString repository);
  pkgs = flake.inputs.nixpkgs.legacyPackages.x86_64-linux;
  compilerPath = /nix/store/1yvh3d6y3fj3xk2dgwczrp1dj5svd92c-rust-default-1.96.0-nightly-2026-03-21;
  compiler = assert builtins.pathExists (compilerPath + "/bin/rustc"); builtins.storePath compilerPath;
  toolchain = "nightly-2026-03-21-x86_64-unknown-linux-gnu";
  cargo = pkgs.writeShellScriptBin "cargo" ''
    export RUSTUP_TOOLCHAIN="''${RUSTUP_TOOLCHAIN:-${toolchain}}"
    exec ${compiler}/bin/cargo "$@"
  '';
  rustup = pkgs.writeShellScriptBin "rustup" ''
    toolchain="''${RUSTUP_TOOLCHAIN:-${toolchain}}"
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
in pkgs.symlinkJoin {
  name = "cargo-octet-check-runtime";
  paths = [ cargo rustup compiler pkgs.stdenv.cc ];
}
