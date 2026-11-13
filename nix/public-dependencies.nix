# Octet owns distribution packaging, not upstream product behavior. Every source
# directory is an unmodified, bounded Git archive selection admitted by Nickel.
{ pkgs, craneLib, nickel, root }:
let
  policy = builtins.fromJSON (builtins.readFile (root + "/vendor/dependencies.json"));
  sources = builtins.listToAttrs (map (source: {
    name = source.name;
    value = {
      outPath = root + "/${source.destination}";
      rev = source.revision;
    };
  }) policy.sources);
  cargoSource = source:
    let
      original = sources.${source.name};
      upstream = builtins.fromTOML (builtins.readFile (original.outPath + "/Cargo.toml"));
      manifest = (pkgs.formats.toml { }).generate "${source.name}-workspace.toml" {
        workspace = upstream.workspace // {
          members = source.workspace_members;
          default-members = source.workspace_members;
          exclude = [];
        };
      };
    in
    if source.workspace_members == [] then original
    else pkgs.runCommand "${source.name}-cargo-source-${source.revision}" { } ''
      cp -R ${original}/. "$out"
      chmod u+w "$out/Cargo.toml"
      cp ${manifest} "$out/Cargo.toml"
    '';
  overrideGitSource = packages: drv:
    let
      package = builtins.head packages;
      source = package.source;
      matching = builtins.filter (entry:
        source == "git+${entry.repository}?rev=${entry.revision}#${entry.revision}"
        || source == "git+${builtins.replaceStrings [ "https://github.com/" ] [ "ssh://git@github.com/" ] entry.repository}?rev=${entry.revision}#${entry.revision}"
      ) policy.sources;
    in
    if matching == [] then drv
    else drv.overrideAttrs (_: { src = cargoSource (builtins.head matching); });
  nickelExportSource = sources.nickel-export;
  nickelExportVendor = craneLib.vendorCargoDeps {
    src = nickelExportSource;
  };
  nickelExport = craneLib.buildPackage {
    pname = "nickel-export";
    version = "0.1.0";
    src = nickelExportSource;
    cargoVendorDir = nickelExportVendor;
    cargoExtraArgs = "-p nickel-export";
    doCheck = false;
    meta.license = pkgs.lib.licenses.agpl3Plus;
    postInstall = ''
      install -Dm644 ${nickelExportSource}/LICENSE "$out/share/licenses/nickel-export/LICENSE"
    '';
  };
  cairnSource = sources.cairn;
  cairnVendor = craneLib.vendorCargoDeps {
    src = cairnSource;
    overrideVendorGitCheckout = overrideGitSource;
  };
  cairn = craneLib.buildPackage {
    pname = "cairn";
    version = (builtins.fromTOML (builtins.readFile (cairnSource.outPath + "/Cargo.toml"))).workspace.package.version;
    src = cairnSource;
    cargoVendorDir = cairnVendor;
    cargoExtraArgs = "-p cairn-cli --features redb-registry";
    nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper pkgs.cmake ];
    buildInputs = [ pkgs.openssl pkgs.zlib ];
    doCheck = false;
    meta.license = pkgs.lib.licenses.agpl3Plus;
    postInstall = ''
      mkdir -p "$out/share/licenses/cairn"
      cp -R ${cairnSource}/LICENSES ${cairnSource}/LICENSE-MIT ${cairnSource}/LICENSE-APACHE "$out/share/licenses/cairn/"
      chmod -R u+w "$out/share/licenses/cairn"
      install -Dm644 ${cairnSource}/vendor/cap-root-core/LICENSE "$out/share/licenses/cairn/cap-root-LICENSE"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/LICENSE-MIT "$out/share/licenses/cairn/durable-file-publication-MIT"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/LICENSE-APACHE "$out/share/licenses/cairn/durable-file-publication-APACHE"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/NOTICE "$out/share/licenses/cairn/durable-file-publication/NOTICE"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/provenance/SOURCE.md "$out/share/licenses/cairn/durable-file-publication/provenance/SOURCE.md"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/provenance/source.ncl "$out/share/licenses/cairn/durable-file-publication/provenance/source.ncl"
      install -Dm644 ${cairnSource}/vendor/durable-file-publication/provenance/source.json "$out/share/licenses/cairn/durable-file-publication/provenance/source.json"
      install -Dm644 ${sources.subwayrat-license-notice}/LICENSE "$out/share/licenses/cairn/subwayrat-license-notice-${sources.subwayrat-license-notice.rev}/LICENSE"
      install -Dm644 ${cairnSource}/cairn-policy/generated/cairn-policy.json "$out/share/cairn/cairn-policy.json"
      for binary in cairn ironspec; do
        wrapProgram "$out/bin/$binary" \
          --set-default CAIRN_POLICY "$out/share/cairn/cairn-policy.json" \
          --prefix PATH : ${pkgs.lib.makeBinPath [ nickel nickelExport ]}
      done
    '';
  };
in
{
  inherit sources overrideGitSource cairn;
}
