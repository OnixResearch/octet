let contracts = import "../../contracts.ncl" in
{
  schema_version = "tigerstyle-acceptance-receipt/v1",
  run_id = "run-0001",
  started_at = "2026-05-06T14:00:00Z",
  finished_at = "2026-05-06T14:01:00Z",
  tool = "tigerstyle-standards",
  tool_version = "0.1.0",
  git_revision = "d35db7a5",
  inputs = ["standards/profiles/local.ncl"],
  stages = [{ name = "policy-validate", status = "passed" }],
  artifacts = ["tigerstyle-artifact-manifest.json"],
  status = "passed",
  verification_commands = ["nix run .#tigerstyle-policy-validate"],
  verification_evidence = [{ path = "openspec/evidence/focused-checks.txt", blake3 = "0123456789abcdef" }],
} | contracts.AcceptanceReceipt
