let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nominal-domain-policy/v1",
  default_mode = "deny",
  heuristic_suffixes = ["Ref"],
  domains = [{
    domain_id = "fixture.unchecked-generic",
    symbols = ["SessionRef"],
    raw_representation = "string",
    core_scopes = ["src/core.rs"],
    compatibility_scopes = ["src/wire.rs"],
    compatibility_required = true,
    wrapper_kind = "marker-generic",
    constructor_policy = "private-infallible",
  }],
  non_claims = ["Unchecked generic construction remains inadmissible."],
} | contracts.Catalog | contracts.CatalogRules
