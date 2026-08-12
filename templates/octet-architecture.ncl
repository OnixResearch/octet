# Copy this file and `octet-architecture.contracts.ncl` to the workspace root.
# Start with inventory mode. Replace every example package, scope, target, and rule.
# Export after review:
#   nickel export --format json octet-architecture.ncl > octet-architecture.json
let contracts = import "./octet-architecture.contracts.ncl" in
{
  schema_version = "octet-project-architecture-policy/v1",
  project_id = "replace-with-project-id",
  mode = "inventory",
  production = {
    packages = ["domain", "application", "inbound", "outbound", "host"],
    source_scopes = [
      "crates/domain/src",
      "crates/application/src",
      "crates/inbound/src",
      "crates/outbound/src",
      "crates/host/src",
    ],
    targets = [{
      triple = "x86_64-unknown-linux-gnu",
      features = ["production"],
      default_features = false,
    }],
  },
  roles = [
    { scope_kind = "package", scope = "domain", role = "domain-core" },
    { scope_kind = "package", scope = "application", role = "application-shell" },
    { scope_kind = "package", scope = "inbound", role = "inbound-adapter" },
    { scope_kind = "package", scope = "outbound", role = "outbound-adapter" },
    { scope_kind = "package", scope = "host", role = "composition-root" },
    { scope_kind = "source", scope = "crates/domain/src", role = "domain-core" },
    { scope_kind = "source", scope = "crates/application/src", role = "application-shell" },
    { scope_kind = "source", scope = "crates/inbound/src", role = "inbound-adapter" },
    { scope_kind = "source", scope = "crates/outbound/src", role = "outbound-adapter" },
    { scope_kind = "source", scope = "crates/host/src", role = "composition-root" },
    { scope_kind = "source", scope = "tests", role = "test" },
    { scope_kind = "source", scope = "target/generated", role = "generated" },
  ],
  dependency_rules = [
    {
      caller_role = "domain-core",
      allowed_callee_roles = ["domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "application-shell",
      allowed_callee_roles = ["application-shell", "domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "inbound-adapter",
      allowed_callee_roles = ["inbound-adapter", "application-shell"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "outbound-adapter",
      allowed_callee_roles = ["outbound-adapter", "application-shell"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "composition-root",
      allowed_callee_roles = [
        "composition-root",
        "application-shell",
        "inbound-adapter",
        "outbound-adapter",
      ],
      edge_kinds = ["normal", "build"],
    },
  ],
  ports = [{
    port_trait = "application::ports::Repository",
    application_owner_scope = "crates/application/src",
    permitted_adapter_scopes = ["crates/outbound/src"],
    forbidden_concrete_types = ["sqlx::", "diesel::"],
    reserved = false,
  }],
  capability_classifications = [
    {
      family = "time",
      core_handling = "explicit-fact",
      executor_role = "outbound-adapter",
      rationale = "the shell observes time before domain evaluation",
    },
    {
      family = "filesystem",
      core_handling = "effect-plan",
      executor_role = "outbound-adapter",
      rationale = "the domain returns a plan and an adapter performs filesystem work",
    },
  ],
  core_no_std = [{ scope = "domain" }],
  waivers = [],
  non_claims = [
    "complete-fcis-proof",
    "complete-hexagonal-proof",
    "runtime-determinism",
    "whole-program-correctness",
    "release-eligibility",
  ],
} | contracts.ProjectArchitecturePolicy | contracts.ProjectArchitecturePolicyRules
