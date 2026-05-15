{
  description = "Checked-in consumer fixture for slotcar Nix parity";
  inputs.slotcar.url = "path:@TIGERSTYLE_PATH@";
  outputs = { self, slotcar }: {
    checks.@SYSTEM@.default = slotcar.lib.mkConsumerCheck {
      system = "@SYSTEM@";
      src = ./.;
      cargoLock = ./Cargo.lock;
    };
  };
}
