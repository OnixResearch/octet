let contracts = import "./octet-self-architecture.contracts.ncl" in
{
  schema_version = "octet-project-architecture-policy/v1",
  project_id = "octet",
  mode = "inventory",
  production = {
    packages = [
      "octet-lint-metadata",
      "octet-standards",
      "octet",
      "cargo-octet",
    ],
    source_scopes = [
      "crates/octet-lint-metadata/src",
      "crates/octet-standards/src",
      "src",
      "cargo-octet/src",
    ],
    targets = [{
      triple = "x86_64-unknown-linux-gnu",
      features = [],
      default_features = true,
    }],
  },
  roles = [
    {
      scope_kind = "package",
      scope = "octet-lint-metadata",
      role = "domain-core",
    },
    {
      scope_kind = "package",
      scope = "octet-standards",
      role = "application-shell",
    },
    {
      scope_kind = "package",
      scope = "octet",
      role = "inbound-adapter",
    },
    {
      scope_kind = "package",
      scope = "cargo-octet",
      role = "composition-root",
    },
    {
      scope_kind = "source",
      scope = "crates/octet-lint-metadata/src",
      role = "domain-core",
    },
    {
      scope_kind = "source",
      scope = "crates/octet-standards/src",
      role = "application-shell",
    },
    {
      scope_kind = "source",
      scope = "src",
      role = "inbound-adapter",
    },
    {
      scope_kind = "source",
      scope = "cargo-octet/src",
      role = "composition-root",
    },
    {
      scope_kind = "source",
      scope = "tests",
      role = "test",
    },
    {
      scope_kind = "source",
      scope = "ui_tests",
      role = "test",
    },
    {
      scope_kind = "source",
      scope = "standards/project-architecture/generated",
      role = "generated",
    },
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
      allowed_callee_roles = ["inbound-adapter", "application-shell", "domain-core"],
      edge_kinds = ["normal", "build"],
    },
    {
      caller_role = "composition-root",
      allowed_callee_roles = [
        "composition-root",
        "application-shell",
        "inbound-adapter",
        "outbound-adapter",
        "domain-core",
      ],
      edge_kinds = ["normal", "build"],
    },
  ],
  ports = [],
  capability_classifications = [
    {
      family = "time",
      core_handling = "explicit-fact",
      executor_role = "composition-root",
      rationale = "runner and adapter shells own time observation",
    },
    {
      family = "randomness",
      core_handling = "explicit-fact",
      executor_role = "composition-root",
      rationale = "runner shells own random value generation",
    },
    {
      family = "filesystem",
      core_handling = "effect-plan",
      executor_role = "composition-root",
      rationale = "runner and standards shells own filesystem access",
    },
    {
      family = "network",
      core_handling = "effect-plan",
      executor_role = "composition-root",
      rationale = "runner shells own network access",
    },
    {
      family = "process",
      core_handling = "effect-plan",
      executor_role = "composition-root",
      rationale = "cargo-octet owns child process execution",
    },
    {
      family = "environment",
      core_handling = "explicit-fact",
      executor_role = "composition-root",
      rationale = "runner shells read and validate environment inputs",
    },
    {
      family = "secrets",
      core_handling = "forbidden",
      executor_role = "composition-root",
      rationale = "architecture policy does not carry raw secret material",
    },
    {
      family = "unsafe-code",
      core_handling = "forbidden",
      executor_role = "composition-root",
      rationale = "unsafe operations remain outside deterministic policy data",
    },
    {
      family = "global-state",
      core_handling = "forbidden",
      executor_role = "composition-root",
      rationale = "policy cores do not depend on ambient mutable state",
    },
    {
      family = "concurrency",
      core_handling = "effect-plan",
      executor_role = "composition-root",
      rationale = "runner shells own task and process lifecycle",
    },
    {
      family = "observability",
      core_handling = "effect-plan",
      executor_role = "composition-root",
      rationale = "shells render diagnostics and evidence",
    },
    {
      family = "ffi",
      core_handling = "forbidden",
      executor_role = "composition-root",
      rationale = "project policy does not authorize foreign calls",
    },
    {
      family = "allocation",
      core_handling = "effect-plan",
      executor_role = "application-shell",
      rationale = "bounded policy evaluation uses application-owned allocation limits",
    },
  ],
  core_no_std = [],
  waivers = [],
  non_claims = [
    "complete-fcis-proof",
    "complete-hexagonal-proof",
    "runtime-determinism",
    "whole-program-correctness",
    "release-eligibility",
  ],
} | contracts.ProjectArchitecturePolicy | contracts.ProjectArchitecturePolicyRules
