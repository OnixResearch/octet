# Octet owns this bounded producer job. It selects existing contracts, not new
# theorem policy; Mantle may realize its complete exported ordinary drv graph.
let consumer = import "consumer-profile.ncl" in
let profile = import "profile.ncl" in
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Job = {
  schema_version | String | std.contract.from_predicate (fun value => value == "octet-mantle-producer-job/v1"),
  owner | String,
  job_id | NonEmpty,
  system | NonEmpty,
  package_attribute | NonEmpty,
  profile | { path | NonEmpty, identity | NonEmpty },
  provider | { path | NonEmpty, identity | NonEmpty },
  source | { root | NonEmpty, paths | Array NonEmpty, network_allowed | Bool },
  execution | { command_argv | Array String, artifact_verify_argv | Array String, rail_id | NonEmpty, network_allowed | Bool },
  tools | { closure_format | NonEmpty, dependencies | Array NonEmpty, solver | NonEmpty },
  limits | { timeout_ms | Number, stdout_max_bytes | Number, stderr_max_bytes | Number },
  outputs | Array NonEmpty,
  non_claims | Array NonEmpty,
} in
{
  schema_version = "octet-mantle-producer-job/v1",
  owner = "Octet",
  job_id = "octet.aeneas.external-consumer.mantle.v1",
  system = "x86_64-linux",
  package_attribute = "packages.x86_64-linux.octet-mantle-aeneas-consumer-job",
  profile = {
    path = "standards/proof-rails/aeneas-lean/generated/consumer-profile.json",
    identity = "b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879",
  },
  provider = {
    path = "standards/proof-rails/aeneas-lean/generated/profile.json",
    identity = consumer.provider.profile_identity,
  },
  source = {
    root = "standards/proof-rails/aeneas-lean/fixtures/external-consumer",
    paths = ["Cargo.toml", "src/lib.rs", "Theorem.lean"],
    network_allowed = false,
  },
  execution = {
    command_argv = ["cargo-octet", "prove", "--policy-artifacts-dir", "$POLICY", "--source", "src/lib.rs", "--rail", "octet.aeneas_consumer_contract.external-fixture#rail-0", "--artifact-dir", "$ARTIFACTS", "--output-format", "json"],
    artifact_verify_argv = ["cargo-octet", "artifact", "verify", "--artifact-dir", "$ARTIFACTS", "--format", "json", "--emit-valence-provenance", "$OUTPUT/valence-provenance.json"],
    rail_id = "octet.aeneas_consumer_contract.external-fixture#rail-0",
    network_allowed = false,
  },
  tools = {
    closure_format = "nix-derivation-show-recursive-json; inputDrvs and inputSrcs are mandatory; flake.lock is preserved verbatim",
    dependencies = ["cargo-octet", "octet-policy-artifacts", "octet-aeneas-lean-provider-closure", profile.aeneas_revision, profile.charon_revision, profile.lean_toolchain, "flake.lock:mathlib-src", "flake.lock:nixpkgs", "flake.lock:aeneas"],
    solver = "none: this selected Aeneas/Charon/Lean rail invokes no external SMT solver; imported Verus evidence is not accepted by this job",
  },
  limits = {
    timeout_ms = consumer.resource_policy.timeout_ms,
    stdout_max_bytes = consumer.resource_policy.stdout_max_bytes,
    stderr_max_bytes = consumer.resource_policy.stderr_max_bytes,
  },
  outputs = ["artifacts", "producer-artifact-verification.json", "producer-prove.json", "consumer-profile.json", "provider-identity.json", "job.json", "source", "external-profile.json", "external-receipt.json", "source-revision", "flake.lock", "valence-provenance.json"],
  non_claims = consumer.non_claims @ [
    "Nix evaluation supplies a frozen ordinary derivation graph; it is not theorem admission",
    "Mantle realizes and signs build outputs, not proof truth or external acceptance policy",
    "A cached producer output preserves its recorded external acceptance; cache reuse is not a fresh verifier execution",
    "The recursive tool graph includes declared bootstrap/binary acquisition inputs and does not claim seed-free bootstrap",
  ],
} | Job
