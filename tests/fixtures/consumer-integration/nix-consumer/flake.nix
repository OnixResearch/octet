{
  description = "Checked-in consumer fixture for tigerstyle Nix parity";
  inputs.tigerstyle.url = "path:@TIGERSTYLE_PATH@";
  outputs = { self, tigerstyle }: {
    checks.@SYSTEM@.default = tigerstyle.lib.mkConsumerCheck {
      system = "@SYSTEM@";
      src = ./.;
      cargoLock = ./Cargo.lock;
    };
  };
}
